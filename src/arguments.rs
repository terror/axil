use super::*;

#[derive(Clap, Debug)]
#[clap(author)]
pub(crate) struct Arguments {
  /// Source file to parse into a syntax tree
  file: PathBuf,
  /// Browse the syntax tree in an interactive TUI instead of printing it
  #[clap(long)]
  interactive: bool,
}

impl Arguments {
  fn parse_file(filename: &PathBuf) -> Result<(String, Tree)> {
    let code = fs::read_to_string(filename)?;

    let mut parser = Parser::new();

    parser.set_language(&Language::try_from(filename.clone())?.into())?;

    let tree = parser
      .parse(&code, None)
      .ok_or_else(|| anyhow!("failed to parse code"))?;

    Ok((code, tree))
  }

  pub(crate) fn run(self) -> Result {
    let (code, tree) = Self::parse_file(&self.file)?;

    if self.interactive {
      App::new(code, tree).run()
    } else {
      Printer::new(&tree, &code).print();
      Ok(())
    }
  }
}
