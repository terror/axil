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

    let info_node = self
      .state
      .selected
      .and_then(|_| self.state.node(&self.tree).ok());

    if let Some(node) = info_node {
      let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

      frame.render_widget(tree_panel, chunks[0]);
      frame.render_widget(InfoPanel::new(node, &self.code), chunks[1]);
    } else {
      frame.render_widget(tree_panel, area);
    }
  }

  fn handle_event(&mut self, event: &KeyEvent) -> Result<ControlFlow<()>> {
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
      } => self.state.scroll_up(),
      KeyEvent {
        code: KeyCode::Char('d'),
        modifiers: KeyModifiers::CONTROL,
        ..
      } => self.state.scroll_down(),
      _ => {}
    }

    Ok(ControlFlow::Continue(()))
  }

  pub(crate) fn new(code: String, tree: Tree) -> Self {
    Self {
      state: State::new(tree.root_node().id()),
      code,
      tree,
    }
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
        if self.handle_event(&key)?.is_break() {
          break;
        }
      }
    }

    Ok(())
  }
}
