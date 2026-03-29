use super::*;

#[derive(Clap, Debug)]
#[clap(author)]
pub(crate) struct Arguments {
  /// Source file to parse into a syntax tree (reads from stdin if omitted)
  file: Option<PathBuf>,
  /// Browse the syntax tree in an interactive TUI instead of printing it
  #[clap(long)]
  interactive: bool,
  /// Language grammar to use (required when reading from stdin)
  #[clap(long)]
  language: Option<Language>,
}

impl Arguments {
  fn parse_source(&self) -> Result<(String, Tree)> {
    let (code, language) = if let Some(file) = &self.file {
      let code = fs::read_to_string(file)?;

      let language = self
        .language
        .map_or_else(|| Language::try_from(file.clone()), Ok)?;

      (code, language)
    } else {
      let language = self.language.ok_or_else(|| {
        anyhow!("`--language` is required when reading from stdin")
      })?;

      let mut code = String::new();

      io::stdin().read_to_string(&mut code)?;

      (code, language)
    };

    let mut parser = Parser::new();

    parser.set_language(&language.into())?;

    let tree = parser
      .parse(&code, None)
      .ok_or_else(|| anyhow!("failed to parse code"))?;

    Ok((code, tree))
  }

  pub(crate) fn run(self) -> Result {
    let (code, tree) = self.parse_source()?;

    if self.interactive {
      App::new(code, tree).run()
    } else {
      Printer::new(&tree, &code).print();
      Ok(())
    }
  }
}
