use super::*;

type RatatuiTerminal = ratatui::Terminal<CrosstermBackend<Stdout>>;

#[derive(Debug)]
pub(crate) struct Terminal {
  inner: RatatuiTerminal,
}

impl Drop for Terminal {
  fn drop(&mut self) {
    if let Err(error) = self.restore() {
      eprintln!("failed to restore terminal: {error}");
    }
  }
}

impl Terminal {
  pub(crate) fn draw(&mut self, f: impl FnOnce(&mut Frame)) -> Result {
    self.inner.draw(f)?;
    Ok(())
  }

  fn initialize() -> Result<RatatuiTerminal> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    Ok(ratatui::Terminal::new(CrosstermBackend::new(stdout))?)
  }

  pub(crate) fn new() -> Result<Self> {
    Ok(Self {
      inner: Self::initialize()?,
    })
  }

  fn restore(&mut self) -> Result {
    crossterm::terminal::disable_raw_mode()?;

    execute!(
      self.inner.backend_mut(),
      LeaveAlternateScreen,
      DisableMouseCapture
    )?;

    self.inner.show_cursor()?;

    Ok(())
  }
}
