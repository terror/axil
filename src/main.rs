use {
  anyhow::Error,
  crossterm::event::{self, Event},
  ratatui::{Frame, text::Text},
  std::process,
};

fn draw(frame: &mut Frame) {
  frame.render_widget(Text::raw("Hello World!"), frame.area());
}

fn run() -> Result {
  let mut terminal = ratatui::init();

  loop {
    terminal.draw(draw)?;

    if matches!(event::read()?, Event::Key(_)) {
      break;
    }
  }

  ratatui::restore();

  Ok(())
}

type Result<T = (), E = Error> = std::result::Result<T, E>;

fn main() {
  if let Err(error) = run() {
    eprintln!("{error}");
    process::exit(1);
  }
}
