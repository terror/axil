use super::*;

pub(crate) struct StatusLine<'a> {
  message: Option<&'a (String, Instant)>,
  mode: &'a Mode,
  state: &'a State,
}

impl Widget for StatusLine<'_> {
  fn render(self, area: Rect, buf: &mut Buffer) {
    if let Some((prompt, style)) = self.prompt() {
      Paragraph::new(prompt).style(style).render(area, buf);
    }
  }
}

impl<'a> StatusLine<'a> {
  pub(crate) const MESSAGE_DURATION: Duration = Duration::from_secs(2);

  pub(crate) fn new(
    mode: &'a Mode,
    state: &'a State,
    message: Option<&'a (String, Instant)>,
  ) -> Self {
    Self {
      message,
      mode,
      state,
    }
  }

  fn prompt(&self) -> Option<(String, Style)> {
    if *self.mode == Mode::Search || !self.state.search_query.is_empty() {
      let prompt = if *self.mode == Mode::Search {
        format!("/{}", self.state.search_query)
      } else {
        let match_count = self.state.matches.len();

        let position = self
          .state
          .matches
          .iter()
          .position(|&id| id == self.state.cursor)
          .map(|i| i + 1);

        if let Some(pos) = position {
          format!("[{pos}/{match_count}] /{}", self.state.search_query)
        } else {
          format!("[{match_count}] /{}", self.state.search_query)
        }
      };

      Some((prompt, Style::default().fg(Color::Yellow)))
    } else if let Some(error) = &self.state.ts_query_error {
      Some((
        format!(":{} | {error}", self.state.ts_query),
        Style::default().fg(Color::Red),
      ))
    } else if *self.mode == Mode::Query || !self.state.ts_query.is_empty() {
      let prompt = if *self.mode == Mode::Query {
        format!(":{}", self.state.ts_query)
      } else {
        let match_count = self.state.ts_query_matches.len();

        let position = self
          .state
          .ts_query_matches
          .iter()
          .position(|&id| id == self.state.cursor)
          .map(|i| i + 1);

        if let Some(pos) = position {
          format!("[{pos}/{match_count}] :{}", self.state.ts_query)
        } else {
          format!("[{match_count}] :{}", self.state.ts_query)
        }
      };

      Some((prompt, Style::default().fg(Color::Cyan)))
    } else if self
      .message
      .is_some_and(|(_, t)| t.elapsed() < Self::MESSAGE_DURATION)
    {
      Some((
        self.message.map_or(String::new(), |(s, _)| s.clone()),
        Style::default().fg(Color::Green),
      ))
    } else {
      None
    }
  }

  pub(crate) fn visible(&self) -> bool {
    self.prompt().is_some()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn expired_message_returns_none() {
    let tree = parse("fn foo() {}");

    let state = State::new(tree.root_node().id());

    assert_eq!(
      prompt(
        &Mode::Normal,
        &state,
        Some(&(
          "foo".into(),
          Instant::now()
            .checked_sub(StatusLine::MESSAGE_DURATION)
            .unwrap(),
        ))
      ),
      None
    );
  }

  fn language() -> TreeSitterLanguage {
    tree_sitter_rust::LANGUAGE.into()
  }

  #[test]
  fn message_is_green() {
    let tree = parse("fn foo() {}");

    let state = State::new(tree.root_node().id());

    assert_eq!(
      prompt_color(
        &Mode::Normal,
        &state,
        Some(&("foo".into(), Instant::now()))
      ),
      Some(Color::Green),
    );
  }

  #[test]
  fn message_shows_when_fresh() {
    let tree = parse("fn foo() {}");

    let state = State::new(tree.root_node().id());

    assert_eq!(
      prompt(&Mode::Normal, &state, Some(&("foo".into(), Instant::now()))),
      Some("foo".into()),
    );
  }

  #[test]
  fn no_prompt_in_normal_mode() {
    let tree = parse("fn foo() {}");

    let state = State::new(tree.root_node().id());

    assert_eq!(prompt(&Mode::Normal, &state, None), None);
  }

  #[test]
  fn not_visible_in_normal_mode() {
    let tree = parse("fn foo() {}");

    let state = State::new(tree.root_node().id());

    assert!(!StatusLine::new(&Mode::Normal, &state, None).visible());
  }

  fn parse(code: &str) -> Tree {
    let mut parser = Parser::new();
    parser.set_language(&language()).unwrap();
    parser.parse(code, None).unwrap()
  }

  fn prompt(
    mode: &Mode,
    state: &State,
    message: Option<&(String, Instant)>,
  ) -> Option<String> {
    StatusLine::new(mode, state, message)
      .prompt()
      .map(|(text, _)| text)
  }

  fn prompt_color(
    mode: &Mode,
    state: &State,
    message: Option<&(String, Instant)>,
  ) -> Option<Color> {
    StatusLine::new(mode, state, message)
      .prompt()
      .and_then(|(_, style)| style.fg)
  }

  #[test]
  fn query_error_is_red() {
    let tree = parse("fn foo() {}");

    let mut state = State::new(tree.root_node().id());
    state.ts_query = "(bad".into();
    state.ts_query_error = Some("syntax error".into());

    assert_eq!(prompt_color(&Mode::Normal, &state, None), Some(Color::Red),);
  }

  #[test]
  fn query_error_shows_error() {
    let tree = parse("fn foo() {}");

    let mut state = State::new(tree.root_node().id());
    state.ts_query = "(bad".into();
    state.ts_query_error = Some("syntax error".into());

    assert_eq!(
      prompt(&Mode::Normal, &state, None),
      Some(":(bad | syntax error".into()),
    );
  }

  #[test]
  fn query_error_takes_priority_over_query_results() {
    let tree = parse("fn foo() {}");

    let mut state = State::new(tree.root_node().id());
    state.ts_query = "(bad".into();
    state.ts_query_error = Some("syntax error".into());
    state.ts_query_matches = vec![1, 2];

    assert_eq!(
      prompt(&Mode::Normal, &state, None),
      Some(":(bad | syntax error".into()),
    );
  }

  #[test]
  fn query_mode_is_cyan() {
    let tree = parse("fn foo() {}");

    let mut state = State::new(tree.root_node().id());
    state.ts_query = "(identifier)".into();

    assert_eq!(prompt_color(&Mode::Query, &state, None), Some(Color::Cyan),);
  }

  #[test]
  fn query_mode_shows_query() {
    let tree = parse("fn foo() {}");

    let mut state = State::new(tree.root_node().id());
    state.ts_query = "(identifier)".into();

    assert_eq!(
      prompt(&Mode::Query, &state, None),
      Some(":(identifier)".into()),
    );
  }

  #[test]
  fn query_results_in_normal_mode_show_count() {
    let tree = parse("fn foo() {}");

    let mut state = State::new(tree.root_node().id());
    state.ts_query = "(identifier)".into();
    state.ts_query_matches = vec![1, 2];

    assert_eq!(
      prompt(&Mode::Normal, &state, None),
      Some("[2] :(identifier)".into()),
    );
  }

  #[test]
  fn query_results_show_position_when_cursor_on_match() {
    let tree = parse("fn foo() {}");

    let mut state = State::new(tree.root_node().id());
    state.ts_query = "(identifier)".into();
    state.ts_query_matches = vec![10, 20];
    state.cursor = 10;

    assert_eq!(
      prompt(&Mode::Normal, &state, None),
      Some("[1/2] :(identifier)".into()),
    );
  }

  #[test]
  fn query_takes_priority_over_message() {
    let tree = parse("fn foo() {}");

    let mut state = State::new(tree.root_node().id());
    state.ts_query = "(identifier)".into();
    state.ts_query_matches = vec![1];

    assert_eq!(
      prompt(&Mode::Normal, &state, Some(&("foo".into(), Instant::now()))),
      Some("[1] :(identifier)".into()),
    );
  }

  #[test]
  fn search_mode_is_yellow() {
    let tree = parse("fn foo() {}");

    let mut state = State::new(tree.root_node().id());
    state.search_query = "bar".into();

    assert_eq!(
      prompt_color(&Mode::Search, &state, None),
      Some(Color::Yellow),
    );
  }

  #[test]
  fn search_mode_shows_query() {
    let tree = parse("fn foo() {}");

    let mut state = State::new(tree.root_node().id());
    state.search_query = "bar".into();

    assert_eq!(prompt(&Mode::Search, &state, None), Some("/bar".into()),);
  }

  #[test]
  fn search_results_in_normal_mode_show_count() {
    let tree = parse("fn foo() {}");

    let mut state = State::new(tree.root_node().id());
    state.search_query = "bar".into();
    state.matches = vec![1, 2, 3];

    assert_eq!(prompt(&Mode::Normal, &state, None), Some("[3] /bar".into()),);
  }

  #[test]
  fn search_results_show_position_when_cursor_on_match() {
    let tree = parse("fn foo() {}");

    let mut state = State::new(tree.root_node().id());
    state.search_query = "bar".into();
    state.matches = vec![10, 20, 30];
    state.cursor = 20;

    assert_eq!(
      prompt(&Mode::Normal, &state, None),
      Some("[2/3] /bar".into()),
    );
  }

  #[test]
  fn search_takes_priority_over_query_error() {
    let tree = parse("fn foo() {}");

    let mut state = State::new(tree.root_node().id());
    state.search_query = "bar".into();
    state.ts_query = "(bad".into();
    state.ts_query_error = Some("syntax error".into());

    assert_eq!(prompt(&Mode::Normal, &state, None), Some("[0] /bar".into()),);
  }

  #[test]
  fn visible_when_searching() {
    let tree = parse("fn foo() {}");

    let mut state = State::new(tree.root_node().id());
    state.search_query = "bar".into();

    assert!(StatusLine::new(&Mode::Search, &state, None).visible());
  }
}
