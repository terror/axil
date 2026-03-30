use super::*;

#[derive(Debug)]
pub(crate) struct App {
  code: String,
  language: TreeSitterLanguage,
  last_reload: Option<Instant>,
  message: Option<(String, Instant)>,
  mode: Mode,
  show_help: bool,
  state: State,
  terminal_height: u16,
  tree: Tree,
  watch_path: Option<PathBuf>,
}

impl App {
  fn draw(&self, frame: &mut Frame) {
    let area = frame.area();

    let tree_panel = TreePanel::new(&self.tree, &self.code, &self.state);

    let info_node = self
      .state
      .selected
      .and_then(|_| self.state.node(&self.tree).ok());

    let status_line =
      StatusLine::new(&self.mode, &self.state, self.message.as_ref());

    let main_area = if status_line.visible() {
      let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

      frame.render_widget(status_line, chunks[1]);

      chunks[0]
    } else {
      area
    };

    if let Some(node) = info_node {
      let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(main_area);

      frame.render_widget(tree_panel, chunks[0]);
      frame.render_widget(InfoPanel::new(node, &self.code), chunks[1]);
    } else {
      frame.render_widget(tree_panel, main_area);
    }

    if self.show_help {
      frame.render_widget(HelpPanel, area);
    }
  }

  fn execute_input(&mut self) {
    match self.mode {
      Mode::Search => self.state.search(&self.tree, &self.code),
      Mode::Query => {
        self
          .state
          .execute_query(&self.language, &self.tree, &self.code);
      }
      Mode::Normal => unreachable!(),
    }
  }

  fn find_node_at_byte(node: Node<'_>, byte: usize) -> Option<usize> {
    (byte >= node.start_byte() && byte < node.end_byte()).then(|| {
      (0..node.child_count_u32())
        .filter_map(|i| node.child(i))
        .find_map(|child| Self::find_node_at_byte(child, byte))
        .unwrap_or_else(|| node.id())
    })
  }

  fn handle_event(&mut self, event: &Event) -> Result<ControlFlow<()>> {
    match event {
      Event::Quit => return Ok(ControlFlow::Break(())),
      Event::MoveUp => self.state.move_up(&self.tree)?,
      Event::MoveDown => self.state.move_down(&self.tree)?,
      Event::MoveLeft => self.state.move_left(&self.tree)?,
      Event::MoveRight => self.state.move_right(&self.tree)?,
      Event::ToggleSelect => self.state.toggle_select(),
      Event::ToggleCollapse => self.state.toggle_collapse(&self.tree)?,
      Event::ScrollUp => self.state.scroll_up(&self.tree, self.terminal_height),
      Event::ScrollDown => {
        self.state.scroll_down(&self.tree, self.terminal_height);
      }
      Event::EnterSearch => {
        self.state.clear_search();
        self.mode = Mode::Search;
      }
      Event::EnterQuery => {
        self.state.clear_query();
        self.mode = Mode::Query;
      }
      Event::FileChanged => self.handle_file_changed()?,
      Event::JumpToMatch { forward } => self.state.jump_to_match(*forward),
      Event::MoveToTop => self.state.move_to_top(&self.tree),
      Event::MoveToBottom => self.state.move_to_bottom(&self.tree),
      Event::Yank => {
        let node = self.state.node(&self.tree)?;

        let text = &self.code[node.start_byte()..node.end_byte()];

        cli_clipboard::set_contents(text.to_string())
          .map_err(|e| anyhow!("failed to copy to clipboard: {e}"))?;

        self.message =
          Some(("Copied text to clipboard".to_string(), Instant::now()));
      }
      Event::ClearSearch => self.state.clear_search(),
      Event::ToggleHelp => self.show_help = !self.show_help,
      Event::InputConfirm => self.mode = Mode::Normal,
      Event::InputCancel => {
        match self.mode {
          Mode::Search => self.state.clear_search(),
          Mode::Query => self.state.clear_query(),
          Mode::Normal => unreachable!(),
        }

        self.mode = Mode::Normal;
      }
      Event::InputBackspace => {
        self.input_buffer_mut().pop();
        self.execute_input();
      }
      Event::InputChar(c) => {
        self.input_buffer_mut().push(*c);
        self.execute_input();
      }
      Event::Click { row } => {
        if let Some(id) = self.state.node_at_row(&self.tree, *row) {
          self.state.cursor = id;
        }
      }
    }

    Ok(ControlFlow::Continue(()))
  }

  fn handle_file_changed(&mut self) -> Result {
    if self
      .last_reload
      .is_some_and(|t| t.elapsed() < Duration::from_millis(100))
    {
      return Ok(());
    }

    let path = match &self.watch_path {
      Some(p) => p.clone(),
      None => return Ok(()),
    };

    let code = fs::read_to_string(&path)?;

    let mut parser = Parser::new();
    parser.set_language(&self.language)?;

    let tree = parser
      .parse(&code, None)
      .ok_or_else(|| anyhow!("failed to parse code"))?;

    let cursor_byte = self.state.node(&self.tree).ok().map(|n| n.start_byte());

    self.code = code;
    self.tree = tree;

    let new_cursor = cursor_byte
      .and_then(|offset| Self::find_node_at_byte(self.tree.root_node(), offset))
      .unwrap_or_else(|| self.tree.root_node().id());

    self.state.reconcile(new_cursor);

    if !self.state.ts_query.is_empty() {
      self
        .state
        .execute_query(&self.language, &self.tree, &self.code);
    }

    if !self.state.search_query.is_empty() {
      self.state.search(&self.tree, &self.code);
    }

    self.last_reload = Some(Instant::now());
    self.message = Some(("File reloaded".to_string(), Instant::now()));

    Ok(())
  }

  fn input_buffer_mut(&mut self) -> &mut String {
    match self.mode {
      Mode::Search => &mut self.state.search_query,
      Mode::Query => &mut self.state.ts_query,
      Mode::Normal => unreachable!(),
    }
  }

  pub(crate) fn new(
    code: String,
    tree: Tree,
    language: TreeSitterLanguage,
    watch_path: Option<PathBuf>,
  ) -> Self {
    Self {
      last_reload: None,
      message: None,
      mode: Mode::default(),
      show_help: false,
      state: State::new(tree.root_node().id()),
      terminal_height: 0,
      code,
      language,
      tree,
      watch_path,
    }
  }

  pub(crate) fn run(mut self) -> Result {
    let mut terminal = Terminal::new()?;

    let (tx, rx) = channel();

    let crossterm_tx = tx.clone();

    thread::spawn(move || loop {
      if crossterm::event::poll(Duration::from_millis(100)).unwrap_or(false) {
        if let Ok(event) = crossterm::event::read() {
          if crossterm_tx.send(ChannelEvent::Crossterm(event)).is_err() {
            break;
          }
        }
      }
    });

    let _watcher = self.setup_watcher(&tx)?;

    loop {
      terminal.draw(|f| {
        self.terminal_height = f.area().height;

        self
          .state
          .ensure_cursor_in_view(&self.tree, self.terminal_height);

        self.draw(f);
      })?;

      let timeout = self
        .message
        .as_ref()
        .and_then(|(_, t)| {
          StatusLine::MESSAGE_DURATION.checked_sub(t.elapsed())
        })
        .unwrap_or(Duration::from_secs(60));

      match rx.recv_timeout(timeout) {
        Ok(internal) => {
          let event = match internal {
            ChannelEvent::Crossterm(ct) => {
              Event::from_crossterm(&ct, &self.mode)
            }
            ChannelEvent::FileChanged => Some(Event::FileChanged),
          };

          if let Some(event) = event {
            if self.handle_event(&event)?.is_break() {
              break;
            }
          }
        }
        Err(RecvTimeoutError::Timeout) => {}
        Err(RecvTimeoutError::Disconnected) => break,
      }
    }

    Ok(())
  }

  pub(crate) fn set_query(&mut self, query_source: &str) {
    self.state.ts_query = query_source.to_string();

    self
      .state
      .execute_query(&self.language, &self.tree, &self.code);
  }

  fn setup_watcher(
    &self,
    tx: &Sender<ChannelEvent>,
  ) -> Result<Option<notify::RecommendedWatcher>> {
    let path = match &self.watch_path {
      Some(p) => p.clone(),
      None => return Ok(None),
    };

    let watcher_tx = tx.clone();

    let mut watcher = notify::recommended_watcher(
      move |res: Result<notify::Event, notify::Error>| {
        if let Ok(event) = res {
          if event.kind.is_modify() {
            let _ = watcher_tx.send(ChannelEvent::FileChanged);
          }
        }
      },
    )?;

    Watcher::watch(&mut watcher, &path, notify::RecursiveMode::NonRecursive)?;

    Ok(Some(watcher))
  }
}
