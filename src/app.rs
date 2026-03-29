use {super::*, status_line::StatusLine};

#[derive(Debug)]
pub(crate) struct App {
  code: String,
  language: TreeSitterLanguage,
  message: Option<(String, Instant)>,
  mode: Mode,
  state: State,
  terminal_height: u16,
  tree: Tree,
}

impl App {
  fn clear_input(&mut self) {
    match self.mode {
      Mode::Search => self.state.clear_search(),
      Mode::Query => self.state.clear_query(),
      Mode::Normal => unreachable!(),
    }
  }

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
      Event::InputConfirm => self.mode = Mode::Normal,
      Event::InputCancel => {
        self.clear_input();
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
  ) -> Self {
    Self {
      message: None,
      mode: Mode::default(),
      state: State::new(tree.root_node().id()),
      terminal_height: 0,
      code,
      language,
      tree,
    }
  }

  pub(crate) fn run(mut self) -> Result {
    let mut terminal = Terminal::new()?;

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

      if crossterm::event::poll(timeout)? {
        if let Some(event) =
          Event::from_crossterm(&crossterm::event::read()?, &self.mode)
        {
          if self.handle_event(&event)?.is_break() {
            break;
          }
        }
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
}
