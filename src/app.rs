use super::*;

#[derive(Debug)]
pub(crate) struct App {
  code: String,
  collapsed_nodes: HashSet<usize>,
  cursor_node: NodeHandle,
  scroll_offset: u16,
  selected_node: Option<NodeHandle>,
  tree: Rc<Tree>,
}

impl App {
  pub(crate) fn new(filename: PathBuf) -> Result<Self> {
    let code = fs::read_to_string(&filename)?;

    let mut parser = Parser::new();

    let language = Language::try_from(filename)?;

    parser.set_language(&language.into())?;

    let tree = parser
      .parse(&code, None)
      .ok_or_else(|| anyhow!("Failed to parse code"))?;

    let tree_rc = Rc::new(tree);

    let cursor_node = NodeHandle::new(tree_rc.clone());

    Ok(Self {
      tree: tree_rc,
      code,
      cursor_node,
      selected_node: None,
      scroll_offset: 0,
      collapsed_nodes: HashSet::new(),
    })
  }

  fn calculate_node_position(
    &self,
    node: &Node,
    target_id: usize,
    position: &mut usize,
  ) -> bool {
    if node.id() == target_id {
      return true;
    }

    *position += 1;

    if self.collapsed_nodes.contains(&node.id()) {
      return false;
    }

    for i in 0..node.child_count() {
      if let Some(child) = node.child(i) {
        if self.calculate_node_position(&child, target_id, position) {
          return true;
        }
      }
    }

    false
  }

  pub(crate) fn draw(&self, frame: &mut Frame) {
    let area = frame.area();

    let mut tree_lines = Vec::new();

    let cursor_node = self.cursor_node.node();

    let selected_id = self.selected_node.as_ref().map(|n| n.id);

    Self::render_tree(
      &self.tree.root_node(),
      &self.code,
      cursor_node.id(),
      selected_id,
      &self.collapsed_nodes,
      0,
      &mut tree_lines,
    );

    let tree_widget = Paragraph::new(
      tree_lines
        .iter()
        .skip(self.scroll_offset as usize)
        .take(area.height as usize)
        .cloned()
        .collect::<Vec<_>>(),
    )
    .block(
      Block::default()
        .title_style(
          Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(Color::DarkGray)),
    );

    if let Some(selected_handle) = &self.selected_node {
      let selected_node = selected_handle.node();

      let node_text =
        &self.code[selected_node.start_byte()..selected_node.end_byte()];

      let node_kind = selected_node.kind();

      let node_color = Self::get_node_color(node_kind);

      let node_info = Text::from(vec![
        Line::from(vec![Span::styled(
          node_kind,
          Style::default().fg(node_color).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
          format!(
            "[{}:{} - {}:{}]",
            selected_node.start_position().row,
            selected_node.start_position().column,
            selected_node.end_position().row,
            selected_node.end_position().column
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

      let info_widget = Paragraph::new(node_info).block(
        Block::default()
          .borders(Borders::ALL)
          .title_style(
            Style::default()
              .fg(Color::Magenta)
              .add_modifier(Modifier::BOLD),
          )
          .border_style(Style::default().fg(Color::DarkGray)),
      );

      let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

      frame.render_widget(tree_widget, chunks[0]);
      frame.render_widget(info_widget, chunks[1]);
    } else {
      let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)])
        .split(area);

      frame.render_widget(tree_widget, chunks[0]);
    }
  }

  pub(crate) fn ensure_cursor_in_view(&mut self, terminal_height: u16) {
    let mut current = self.cursor_node.clone();

    while let Some(parent) = current.parent() {
      current = parent;
    }

    let mut position = 0;

    let root = self.tree.root_node();

    self.calculate_node_position(
      &root,
      self.cursor_node.node().id(),
      &mut position,
    );

    let display_area = terminal_height.saturating_sub(2) as usize;

    if position < self.scroll_offset as usize {
      self.scroll_offset = position as u16;
    } else if position >= (self.scroll_offset as usize + display_area) {
      self.scroll_offset = (position - display_area + 1) as u16;
    }
  }

  fn format_node<'a>(
    node: &Node,
    code: &str,
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

    let node_color = Self::get_node_color(node_kind);

    let node_text = if node.child_count() == 0 {
      format!("\"{}\"", &code[node.start_byte()..node.end_byte()])
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

  fn get_node_color(node_kind: &str) -> Color {
    match node_kind {
      "source_file" => Color::Cyan,
      "assignment" => Color::Magenta,
      "comment" => Color::DarkGray,
      "string" => Color::Green,
      "identifier" => Color::Yellow,
      "number" => Color::Blue,
      "function" => Color::Red,
      "parameter" => Color::Yellow,
      "argument" => Color::Cyan,
      _ => Color::White,
    }
  }

  pub(crate) fn move_down(&mut self) {
    let current = self.cursor_node.node();

    if current.child_count() > 0
      && !self.collapsed_nodes.contains(&current.id())
    {
      if let Some(child) = self.cursor_node.child(0) {
        self.cursor_node = child;
      }
    }
  }

  pub(crate) fn move_left(&mut self) {
    if let Some(prev) = self.cursor_node.prev_sibling() {
      self.cursor_node = prev;
    } else if let Some(parent) = self.cursor_node.parent() {
      self.cursor_node = parent;
    }
  }

  pub(crate) fn move_right(&mut self) {
    if let Some(next) = self.cursor_node.next_sibling() {
      self.cursor_node = next;
    }
  }

  pub(crate) fn move_up(&mut self) {
    if let Some(parent) = self.cursor_node.parent() {
      self.cursor_node = parent;
    }
  }

  fn render_tree(
    node: &Node,
    code: &str,
    cursor_id: usize,
    selected_id: Option<usize>,
    collapsed_nodes: &HashSet<usize>,
    depth: usize,
    lines: &mut Vec<Line>,
  ) {
    let is_cursor = node.id() == cursor_id;

    let is_selected = selected_id.is_some_and(|id| node.id() == id);

    let is_collapsed = collapsed_nodes.contains(&node.id());

    let has_children = node.child_count() > 0;

    lines.push(Self::format_node(
      node,
      code,
      depth,
      is_cursor,
      is_selected,
      is_collapsed,
      has_children,
    ));

    if is_collapsed {
      return;
    }

    for i in 0..node.child_count() {
      if let Some(child) = node.child(i) {
        Self::render_tree(
          &child,
          code,
          cursor_id,
          selected_id,
          collapsed_nodes,
          depth + 1,
          lines,
        );
      }
    }
  }

  pub(crate) fn scroll_down(&mut self) {
    self.scroll_offset += 1;
  }

  pub(crate) fn scroll_up(&mut self) {
    if self.scroll_offset > 0 {
      self.scroll_offset -= 1;
    }
  }

  pub(crate) fn toggle_collapse(&mut self) {
    let node_id = self.cursor_node.node().id();

    let has_children = self.cursor_node.node().child_count() > 0;

    if has_children {
      if self.collapsed_nodes.contains(&node_id) {
        self.collapsed_nodes.remove(&node_id);
      } else {
        self.collapsed_nodes.insert(node_id);
      }
    }
  }

  pub(crate) fn toggle_select(&mut self) {
    if self
      .selected_node
      .as_ref()
      .is_some_and(|n| n.id == self.cursor_node.id)
    {
      self.selected_node = None;
    } else {
      self.selected_node = Some(self.cursor_node.clone());
    }
  }
}
