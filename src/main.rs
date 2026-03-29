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
  mode::Mode,
  node_ext::NodeExt,
  printer::Printer,
  ratatui::{
    prelude::*,
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph},
  },
  state::State,
  std::{
    collections::HashSet,
    fmt::{self, Display, Formatter},
    fs,
    io::{self, Read, Stdout},
    ops::ControlFlow,
    path::PathBuf,
    process,
    str::FromStr,
  },
  terminal::Terminal,
  tree_panel::TreePanel,
  tree_sitter::{Language as TreeSitterLanguage, Node, Parser, Tree},
};

mod app;
mod arguments;
mod info_panel;
mod language;
mod mode;
mod node_ext;
mod printer;
mod state;
mod terminal;
mod tree_panel;

unsafe extern "C" {
  pub(crate) fn tree_sitter_just() -> TreeSitterLanguage;
}

type Result<T = (), E = Error> = std::result::Result<T, E>;

fn main() {
  if let Err(error) = Arguments::parse().run() {
    eprintln!("error: {error}");

    let causes = error.chain().skip(1).count();

    for (i, err) in error.chain().skip(1).enumerate() {
      eprintln!("       {}─ {err}", if i < causes - 1 { '├' } else { '└' });
    }

    process::exit(1);
  }
}
