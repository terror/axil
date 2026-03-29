use super::*;

pub(crate) struct TreePanel<'a> {
  code: &'a str,
  state: &'a State,
  tree: &'a Tree,
}

impl Widget for TreePanel<'_> {
  fn render(self, area: Rect, buf: &mut Buffer) {
    let lines = self
      .collect_lines()
      .into_iter()
      .skip(self.state.scroll_offset as usize)
      .take(area.height as usize)
      .collect::<Vec<_>>();

    Paragraph::new(lines)
      .block(
        Block::default()
          .title_style(
            Style::default()
              .fg(Color::Cyan)
              .add_modifier(Modifier::BOLD),
          )
          .border_style(Style::default().fg(Color::DarkGray)),
      )
      .render(area, buf);
  }
}

impl<'a> TreePanel<'a> {
  fn child_count_span(node: &Node) -> Span<'a> {
    Span::styled(
      format!("{} ", node.child_count()),
      Style::default().fg(Color::DarkGray),
    )
  }

  fn collect_lines(&self) -> Vec<Line<'a>> {
    let mut lines = Vec::new();

    self.render_node(&self.tree.root_node(), 0, &mut lines);

    lines
  }

  fn fold_span(&self, node: &Node) -> Span<'a> {
    if node.child_count() == 0 {
      return Span::styled("    ", Style::default());
    }

    let collapsed = self.state.collapsed_nodes.contains(&node.id());

    Span::styled(
      if collapsed { "[+] " } else { "[-] " },
      Style::default().fg(Color::Gray),
    )
  }

  fn format_node(&self, node: &Node, depth: usize) -> Line<'a> {
    Line::from(vec![
      Self::indent_span(depth),
      self.prefix_span(node),
      self.fold_span(node),
      self.kind_span(node),
      Self::position_span(node),
      Self::child_count_span(node),
      self.text_span(node),
    ])
  }

  fn indent_span(depth: usize) -> Span<'a> {
    Span::styled("  ".repeat(depth), Style::default().fg(Color::DarkGray))
  }

  fn kind_span(&self, node: &Node) -> Span<'a> {
    let id = node.id();

    let is_cursor = id == self.state.cursor;
    let is_match = self.state.matches.contains(&id);
    let is_selected = self.state.selected.is_some_and(|s| s == id);

    let style = if is_match {
      Style::default()
        .fg(Color::Black)
        .bg(Color::Yellow)
        .add_modifier(Modifier::BOLD)
    } else {
      Style::default().fg(node.color()).add_modifier(
        if is_cursor || is_selected {
          Modifier::BOLD
        } else {
          Modifier::empty()
        },
      )
    };

    Span::styled(node.kind(), style)
  }

  pub(crate) fn new(tree: &'a Tree, code: &'a str, state: &'a State) -> Self {
    Self { code, state, tree }
  }

  fn position_span(node: &Node) -> Span<'a> {
    Span::styled(
      format!(
        " [{}:{}..{}:{}] ",
        node.start_position().row,
        node.start_position().column,
        node.end_position().row,
        node.end_position().column
      ),
      Style::default().fg(Color::DarkGray),
    )
  }

  fn prefix_span(&self, node: &Node) -> Span<'a> {
    let id = node.id();

    let is_cursor = id == self.state.cursor;
    let is_selected = self.state.selected.is_some_and(|s| s == id);

    if is_cursor {
      Span::styled("> ", Style::default().add_modifier(Modifier::BOLD))
    } else if is_selected {
      Span::styled(
        "* ",
        Style::default()
          .fg(Color::White)
          .bg(Color::Magenta)
          .add_modifier(Modifier::BOLD),
      )
    } else {
      Span::raw("  ")
    }
  }

  fn render_node(&self, node: &Node, depth: usize, lines: &mut Vec<Line<'a>>) {
    lines.push(self.format_node(node, depth));

    if self.state.collapsed_nodes.contains(&node.id()) {
      return;
    }

    for i in 0..node.child_count() {
      if let Some(child) = node.child(i) {
        self.render_node(&child, depth + 1, lines);
      }
    }
  }

  fn text_span(&self, node: &Node) -> Span<'a> {
    if node.child_count() > 0 {
      return Span::raw("");
    }

    Span::styled(
      format!("\"{}\"", &self.code[node.start_byte()..node.end_byte()]),
      Style::default().fg(Color::Green),
    )
  }
}
