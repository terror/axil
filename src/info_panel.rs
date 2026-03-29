use super::*;

pub(crate) struct InfoPanel<'a> {
  code: &'a str,
  node: Node<'a>,
}

impl<'a> InfoPanel<'a> {
  pub(crate) fn new(node: Node<'a>, code: &'a str) -> Self {
    Self { code, node }
  }
}

impl Widget for InfoPanel<'_> {
  fn render(self, area: Rect, buf: &mut Buffer) {
    let node_text = &self.code[self.node.start_byte()..self.node.end_byte()];

    let node_kind = self.node.kind();

    let node_info = Text::from(vec![
      Line::from(vec![Span::styled(
        node_kind,
        Style::default()
          .fg(self.node.color())
          .add_modifier(Modifier::BOLD),
      )]),
      Line::from(vec![Span::styled(
        format!(
          "[{}:{} - {}:{}]",
          self.node.start_position().row,
          self.node.start_position().column,
          self.node.end_position().row,
          self.node.end_position().column
        ),
        Style::default().fg(Color::Yellow),
      )]),
      Line::from(vec![Span::styled(
        if node_text.len() > 100 {
          format!("{}... ({})", &node_text[..100], node_text.len())
        } else {
          node_text.to_string()
        },
        Style::default().fg(Color::Green),
      )]),
    ]);

    Paragraph::new(node_info)
      .block(
        Block::default()
          .borders(Borders::ALL)
          .title_style(
            Style::default()
              .fg(Color::Magenta)
              .add_modifier(Modifier::BOLD),
          )
          .border_style(Style::default().fg(Color::DarkGray)),
      )
      .render(area, buf);
  }
}
