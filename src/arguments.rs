use super::*;

#[derive(Clap, Debug)]
#[clap(author, version)]
pub(crate) struct Arguments {
  /// Source file to parse into a syntax tree (reads from stdin if omitted)
  file: Option<PathBuf>,
  /// Browse the syntax tree in an interactive TUI instead of printing it
  #[clap(short, long)]
  interactive: bool,
  /// Language grammar to use (required when reading from stdin)
  #[clap(short, long)]
  language: Option<Language>,
  /// Tree-sitter query pattern to match against the syntax tree
  #[clap(short, long)]
  query: Option<String>,
  /// Watch the source file for changes and reload automatically
  #[clap(short, long)]
  watch: bool,
}

impl Arguments {
  fn parse_source(&self) -> Result<(String, Tree, TreeSitterLanguage)> {
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

    let ts_language: TreeSitterLanguage = language.into();

    let mut parser = Parser::new();

    parser.set_language(&ts_language)?;

    let tree = parser
      .parse(&code, None)
      .ok_or_else(|| anyhow!("failed to parse code"))?;

    Ok((code, tree, ts_language))
  }

  pub(crate) fn run(self) -> Result {
    if self.watch && !self.interactive {
      return Err(anyhow!("`--watch` requires `--interactive`"));
    }

    if self.watch && self.file.is_none() {
      return Err(anyhow!("`--watch` requires a file argument"));
    }

    let (code, tree, language) = self.parse_source()?;

    if self.interactive {
      let watch_path = if self.watch { self.file.clone() } else { None };

      let mut app = App::new(code, tree, language, watch_path);

      if let Some(query_source) = &self.query {
        app.set_query(query_source);
      }

      app.run()
    } else {
      let matches = if let Some(query_source) = &self.query {
        Self::run_query(query_source, &language, &tree, &code)?
      } else {
        HashSet::new()
      };

      Printer::new(&tree, &code, matches).print();
      Ok(())
    }
  }

  fn run_query(
    query_source: &str,
    language: &TreeSitterLanguage,
    tree: &Tree,
    code: &str,
  ) -> Result<HashSet<usize>> {
    let query = Query::new(language, query_source)?;

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), code.as_bytes());

    let mut matched = HashSet::new();

    while let Some(m) = matches.next() {
      for capture in m.captures {
        matched.insert(capture.node.id());
      }
    }

    Ok(matched)
  }
}
