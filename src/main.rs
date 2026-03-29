use {
  anyhow::{anyhow, Error},
  app::App,
  arguments::Arguments,
  clap::Parser as Clap,
  crossterm::{
    event::{
      self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent,
      KeyModifiers,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
  },
  info_panel::InfoPanel,
  language::Language,
  node_ext::NodeExt,
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
  terminal::Terminal,
  tree_panel::TreePanel,
  tree_sitter::{Language as TreeSitterLanguage, Node, Parser, Tree},
};

mod app;
mod arguments;
mod info_panel;
mod language;
mod node_ext;
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
