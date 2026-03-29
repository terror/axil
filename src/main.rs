use {
  crate::{
    app::App, arguments::Arguments, info_panel::InfoPanel, language::Language,
    terminal::Terminal, tree_panel::TreePanel,
  },
  anyhow::{anyhow, Error},
  clap::Parser as Clap,
  crossterm::{
    event::{
      self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent,
      KeyModifiers,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
  },
  ratatui::{
    prelude::*,
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph},
  },
  std::{
    collections::HashSet,
    fs,
    io::{self, Stdout},
    path::PathBuf,
    process,
  },
  tree_sitter::{Language as TreeSitterLanguage, Node, Parser, Tree},
};

mod app;
mod arguments;
mod info_panel;
mod language;
mod terminal;
mod tree_panel;

unsafe extern "C" {
  pub(crate) fn tree_sitter_just() -> TreeSitterLanguage;
}

type Result<T = (), E = Error> = std::result::Result<T, E>;

fn main() {
  if let Err(error) = Arguments::parse().run() {
    eprintln!("{error}");
    process::exit(1);
  }
}

fn node_color(kind: &str) -> Color {
  match kind {
    "assignment" => Color::Magenta,
    "comment" => Color::DarkGray,
    "function" => Color::Red,
    "identifier" | "parameter" => Color::Yellow,
    "number" => Color::Blue,
    "source_file" | "argument" => Color::Cyan,
    "string" => Color::Green,
    _ => Color::White,
  }
}
