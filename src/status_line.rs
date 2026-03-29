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
        format!("?{} | {error}", self.state.ts_query),
        Style::default().fg(Color::Red),
      ))
    } else if *self.mode == Mode::Query || !self.state.ts_query.is_empty() {
      let prompt = if *self.mode == Mode::Query {
        format!("?{}", self.state.ts_query)
      } else {
        let match_count = self.state.ts_query_matches.len();

        let position = self
          .state
          .ts_query_matches
          .iter()
          .position(|&id| id == self.state.cursor)
          .map(|i| i + 1);

        if let Some(pos) = position {
          format!("[{pos}/{match_count}] ?{}", self.state.ts_query)
        } else {
          format!("[{match_count}] ?{}", self.state.ts_query)
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
