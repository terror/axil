use super::*;

#[derive(Debug)]
pub(crate) struct App {
  code: String,
  state: State,
  tree: Tree,
}

impl App {
  fn draw(&self, frame: &mut Frame) {
    let area = frame.area();

    let tree_panel = TreePanel::new(&self.tree, &self.code, &self.state);

    if self.state.selected.is_some() {
      let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

      frame.render_widget(tree_panel, chunks[0]);

      frame.render_widget(
        InfoPanel::new(self.state.node(&self.tree), &self.code),
        chunks[1],
      );
    } else {
      frame.render_widget(tree_panel, area);
    }
  }

  fn handle_event(&mut self, event: &KeyEvent) -> ControlFlow<()> {
    match event {
      KeyEvent {
        code: KeyCode::Char('q'),
        ..
      } => return ControlFlow::Break(()),
      KeyEvent {
        code: KeyCode::Char('k'),
        ..
      } => self.state.move_up(&self.tree),
      KeyEvent {
        code: KeyCode::Char('j'),
        ..
      } => self.state.move_down(&self.tree),
      KeyEvent {
        code: KeyCode::Char('h'),
        ..
      } => self.state.move_left(&self.tree),
      KeyEvent {
        code: KeyCode::Char('l'),
        ..
      } => self.state.move_right(&self.tree),
      KeyEvent {
        code: KeyCode::Char(' '),
        ..
      } => self.state.toggle_select(),
      KeyEvent {
        code: KeyCode::Enter,
        ..
      } => self.state.toggle_collapse(&self.tree),
      KeyEvent {
        code: KeyCode::Char('u'),
        modifiers: KeyModifiers::CONTROL,
        ..
      } => self.state.scroll_up(),
      KeyEvent {
        code: KeyCode::Char('d'),
        modifiers: KeyModifiers::CONTROL,
        ..
      } => self.state.scroll_down(),
      _ => {}
    }

    ControlFlow::Continue(())
  }

  pub(crate) fn new(filename: PathBuf) -> Result<Self> {
    let code = fs::read_to_string(&filename)?;

    let mut parser = Parser::new();

    parser.set_language(&Language::try_from(filename)?.into())?;

    let tree = parser
      .parse(&code, None)
      .ok_or_else(|| anyhow!("failed to parse code"))?;

    Ok(Self {
      code,
      state: State::new(tree.root_node().id()),
      tree,
    })
  }

  pub(crate) fn run(mut self) -> Result {
    let mut terminal = Terminal::new()?;

    loop {
      terminal.draw(|f| {
        let terminal_height = f.area().height;

        self
          .state
          .ensure_cursor_in_view(&self.tree, terminal_height);

        self.draw(f);
      })?;

      if let Event::Key(key) = event::read()? {
        if self.handle_event(&key).is_break() {
          break;
        }
      }
    }

    Ok(())
  }
}
