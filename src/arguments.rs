use super::*;

#[derive(Clap, Debug)]
#[clap(author)]
pub(crate) struct Arguments {
  file: PathBuf,
}

impl Arguments {
  pub(crate) fn run(self) -> Result {
    let mut terminal = setup_terminal()?;

    let mut app = App::new(self.file)?;

    loop {
      terminal.draw(|f| {
        let terminal_height = f.area().height;
        app.ensure_cursor_in_view(terminal_height);
        draw(f, &app)
      })?;

      if let Event::Key(key) = event::read()? {
        match key {
          KeyEvent {
            code: KeyCode::Char('q'),
            ..
          } => break,
          KeyEvent {
            code: KeyCode::Char('k'),
            ..
          } => app.move_up(),
          KeyEvent {
            code: KeyCode::Char('j'),
            ..
          } => app.move_down(),
          KeyEvent {
            code: KeyCode::Char('h'),
            ..
          } => app.move_left(),
          KeyEvent {
            code: KeyCode::Char('l'),
            ..
          } => app.move_right(),
          KeyEvent {
            code: KeyCode::Char(' '),
            ..
          } => app.toggle_select(),
          KeyEvent {
            code: KeyCode::Enter,
            ..
          } => app.toggle_collapse(),
          KeyEvent {
            code: KeyCode::Char('u'),
            modifiers: KeyModifiers::CONTROL,
            ..
          } => app.scroll_up(),
          KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::CONTROL,
            ..
          } => app.scroll_down(),
          _ => {}
        }
      }
    }

    restore_terminal(&mut terminal)?;

    Ok(())
  }
}
