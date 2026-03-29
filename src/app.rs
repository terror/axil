use super::*;

const FLASH_DURATION: Duration = Duration::from_secs(2);

#[derive(Debug)]
pub(crate) struct App {
  code: String,
  flash: Option<(String, Instant)>,
  language: TreeSitterLanguage,
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

    let main_area = if let Some((prompt, style)) = self.status_line() {
      let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

      frame.render_widget(Paragraph::new(prompt).style(style), chunks[1]);

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

  fn handle_event(&mut self, event: &KeyEvent) -> Result<ControlFlow<()>> {
    match self.mode {
      Mode::Normal => self.handle_normal_event(event),
      Mode::Search | Mode::Query => Ok(self.handle_input_event(event)),
    }
  }

  fn handle_input_event(&mut self, event: &KeyEvent) -> ControlFlow<()> {
    match event.code {
      KeyCode::Enter => {
        self.mode = Mode::Normal;
      }
      KeyCode::Esc => {
        self.clear_input();
        self.mode = Mode::Normal;
      }
      KeyCode::Backspace => {
        self.input_buffer_mut().pop();
        self.execute_input();
      }
      KeyCode::Char(c) => {
        self.input_buffer_mut().push(c);
        self.execute_input();
      }
      _ => {}
    }

    ControlFlow::Continue(())
  }

  fn handle_mouse_event(&mut self, event: crossterm::event::MouseEvent) {
    match event.kind {
      MouseEventKind::Down(MouseButton::Left) => {
        if let Some(id) = self.state.node_at_row(&self.tree, event.row) {
          self.state.cursor = id;
        }
      }
      MouseEventKind::ScrollUp => {
        self.state.scroll_up(&self.tree, self.terminal_height);
      }
      MouseEventKind::ScrollDown => {
        self.state.scroll_down(&self.tree, self.terminal_height);
      }
      _ => {}
    }
  }

  fn handle_normal_event(
    &mut self,
    event: &KeyEvent,
  ) -> Result<ControlFlow<()>> {
    match event {
      KeyEvent {
        code: KeyCode::Char('q'),
        ..
      } => return Ok(ControlFlow::Break(())),
      KeyEvent {
        code: KeyCode::Char('k'),
        ..
      } => self.state.move_up(&self.tree)?,
      KeyEvent {
        code: KeyCode::Char('j'),
        ..
      } => self.state.move_down(&self.tree)?,
      KeyEvent {
        code: KeyCode::Char('h'),
        ..
      } => self.state.move_left(&self.tree)?,
      KeyEvent {
        code: KeyCode::Char('l'),
        ..
      } => self.state.move_right(&self.tree)?,
      KeyEvent {
        code: KeyCode::Char(' '),
        ..
      } => self.state.toggle_select(),
      KeyEvent {
        code: KeyCode::Enter,
        ..
      } => self.state.toggle_collapse(&self.tree)?,
      KeyEvent {
        code: KeyCode::Char('u'),
        modifiers: KeyModifiers::CONTROL,
        ..
      } => self.state.scroll_up(&self.tree, self.terminal_height),
      KeyEvent {
        code: KeyCode::Char('d'),
        modifiers: KeyModifiers::CONTROL,
        ..
      } => self.state.scroll_down(&self.tree, self.terminal_height),
      KeyEvent {
        code: KeyCode::Char('/'),
        ..
      } => {
        self.state.clear_search();
        self.mode = Mode::Search;
      }
      KeyEvent {
        code: KeyCode::Char('n'),
        ..
      } => self.state.jump_to_match(true),
      KeyEvent {
        code: KeyCode::Char('N'),
        ..
      } => self.state.jump_to_match(false),
      KeyEvent {
        code: KeyCode::Char('?'),
        ..
      } => {
        self.state.clear_query();
        self.mode = Mode::Query;
      }
      KeyEvent {
        code: KeyCode::Char('g'),
        ..
      } => self.state.move_to_top(&self.tree),
      KeyEvent {
        code: KeyCode::Char('G'),
        ..
      } => self.state.move_to_bottom(&self.tree),
      KeyEvent {
        code: KeyCode::Char('y'),
        ..
      } => self.yank()?,
      KeyEvent {
        code: KeyCode::Esc, ..
      } => self.state.clear_search(),
      _ => {}
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
      flash: None,
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
        .flash
        .as_ref()
        .and_then(|(_, t)| FLASH_DURATION.checked_sub(t.elapsed()))
        .unwrap_or(Duration::from_secs(60));

      if event::poll(timeout)? {
        match event::read()? {
          Event::Key(key) => {
            if self.handle_event(&key)?.is_break() {
              break;
            }
          }
          Event::Mouse(mouse) => self.handle_mouse_event(mouse),
          _ => {}
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

  fn status_line(&self) -> Option<(String, Style)> {
    if self.mode == Mode::Search || !self.state.search_query.is_empty() {
      let prompt = if self.mode == Mode::Search {
        format!("/{}", self.state.search_query)
      } else {
        let match_count = self.state.matches.len();

        let position = self
          .state
          .matches
          .iter()
          .position(|&id| id == self.state.cursor)
          .map(|i| i + 1);

        if let Some(pos) = position {
          format!("[{pos}/{match_count}] /{}", self.state.search_query)
        } else {
          format!("[{match_count}] /{}", self.state.search_query)
        }
      };

      Some((prompt, Style::default().fg(Color::Yellow)))
    } else if let Some(error) = &self.state.ts_query_error {
      Some((
        format!("?{} | {error}", self.state.ts_query),
        Style::default().fg(Color::Red),
      ))
    } else if self.mode == Mode::Query || !self.state.ts_query.is_empty() {
      let prompt = if self.mode == Mode::Query {
        format!("?{}", self.state.ts_query)
      } else {
        let match_count = self.state.ts_query_matches.len();

        let position = self
          .state
          .ts_query_matches
          .iter()
          .position(|&id| id == self.state.cursor)
          .map(|i| i + 1);

        if let Some(pos) = position {
          format!("[{pos}/{match_count}] ?{}", self.state.ts_query)
        } else {
          format!("[{match_count}] ?{}", self.state.ts_query)
        }
      };

      Some((prompt, Style::default().fg(Color::Cyan)))
    } else if self
      .flash
      .as_ref()
      .is_some_and(|(_, t)| t.elapsed() < FLASH_DURATION)
    {
      Some((
        self
          .flash
          .as_ref()
          .map_or(String::new(), |(s, _)| s.clone()),
        Style::default().fg(Color::Green),
      ))
    } else {
      None
    }
  }

  fn yank(&mut self) -> Result {
    let node = self.state.node(&self.tree)?;

    let text = &self.code[node.start_byte()..node.end_byte()];

    cli_clipboard::set_contents(text.to_string())
      .map_err(|e| anyhow!("failed to copy to clipboard: {e}"))?;

    self.flash = Some(("Copied text to clipboard".to_string(), Instant::now()));

    Ok(())
  }
}
