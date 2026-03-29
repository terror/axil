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
  fn collect_lines(&self) -> Vec<Line<'a>> {
    let mut lines = Vec::new();

    self.render_node(&self.tree.root_node(), 0, &mut lines);

    lines
  }

  #[allow(clippy::fn_params_excessive_bools)]
  fn format_node(
    &self,
    node: &Node,
    depth: usize,
    is_cursor: bool,
    is_selected: bool,
    is_collapsed: bool,
    has_children: bool,
  ) -> Line<'a> {
    let indent = "  ".repeat(depth);

    let prefix = if is_cursor {
      "> "
    } else if is_selected {
      "* "
    } else {
      "  "
    };

    let node_kind = node.kind();

    let node_color = node.color();

    let node_text = if node.child_count() == 0 {
      format!("\"{}\"", &self.code[node.start_byte()..node.end_byte()])
    } else {
      String::new()
    };

    let mut spans = vec![];

    spans.push(Span::styled(indent, Style::default().fg(Color::DarkGray)));

    if is_cursor {
      spans.push(Span::styled(
        prefix,
        Style::default().add_modifier(Modifier::BOLD),
      ));
    } else if is_selected {
      spans.push(Span::styled(
        prefix,
        Style::default()
          .fg(Color::White)
          .bg(Color::Magenta)
          .add_modifier(Modifier::BOLD),
      ));
    } else {
      spans.push(Span::raw(prefix));
    }

    if has_children {
      let fold_indicator = if is_collapsed { "[+] " } else { "[-] " };

      spans.push(Span::styled(
        fold_indicator,
        Style::default().fg(Color::Gray),
      ));
    } else {
      spans.push(Span::styled("    ", Style::default()));
    }

    spans.push(Span::styled(
      node_kind,
      Style::default().fg(node_color).add_modifier(
        if is_cursor || is_selected {
          Modifier::BOLD
        } else {
          Modifier::empty()
        },
      ),
    ));

    spans.push(Span::styled(
      format!(
        " [{}:{}..{}:{}] ",
        node.start_position().row,
        node.start_position().column,
        node.end_position().row,
        node.end_position().column
      ),
      Style::default().fg(Color::DarkGray),
    ));

    spans.push(Span::styled(
      format!("{} ", node.child_count()),
      Style::default().fg(Color::DarkGray),
    ));

    if !node_text.is_empty() {
      spans.push(Span::styled(node_text, Style::default().fg(Color::Green)));
    }

    Line::from(spans)
  }

  pub(crate) fn new(tree: &'a Tree, code: &'a str, state: &'a State) -> Self {
    Self { code, state, tree }
  }

  fn render_node(&self, node: &Node, depth: usize, lines: &mut Vec<Line<'a>>) {
    let is_collapsed = self.state.collapsed_nodes.contains(&node.id());

    lines.push(self.format_node(
      node,
      depth,
      node.id() == self.state.cursor,
      self.state.selected.is_some_and(|id| node.id() == id),
      is_collapsed,
      node.child_count() > 0,
    ));

    if is_collapsed {
      return;
    }

    for i in 0..node.child_count() {
      if let Some(child) = node.child(i) {
        self.render_node(&child, depth + 1, lines);
      }
    }
  }
}
