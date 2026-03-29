use {
  crate::{
    app::App, arguments::Arguments, language::Language,
    node_handle::NodeHandle, terminal::Terminal,
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
    rc::Rc,
  },
  tree_sitter::{Language as TreeSitterLanguage, Node, Parser, Tree},
};

mod app;
mod arguments;
mod language;
mod node_handle;
mod terminal;

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
