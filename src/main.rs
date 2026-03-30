use {
  anyhow::{anyhow, Error},
  app::App,
  arguments::Arguments,
  channel_event::ChannelEvent,
  clap::Parser as Clap,
  crossterm::{
    event::{
      DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent, KeyModifiers,
      MouseButton, MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
  },
  event::Event,
  help_panel::HelpPanel,
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
  status_line::StatusLine,
  std::{
    collections::HashSet,
    fmt::{self, Display, Formatter},
    fs,
    io::{self, Read, Stdout},
    ops::ControlFlow,
    path::{Path, PathBuf},
    process,
    str::FromStr,
    sync::mpsc::{channel, RecvTimeoutError, Sender},
    thread,
    time::{Duration, Instant},
  },
  terminal::Terminal,
  tree_panel::TreePanel,
  tree_sitter::{
    Language as TreeSitterLanguage, Node, Parser, Query, QueryCursor,
    StreamingIterator, Tree,
  },
  watcher::Watcher,
};

mod app;
mod arguments;
mod channel_event;
mod event;
mod help_panel;
mod info_panel;
mod language;
mod mode;
mod node_ext;
mod printer;
mod state;
mod status_line;
mod terminal;
mod tree_panel;
mod watcher;

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
