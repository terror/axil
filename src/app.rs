use super::*;

#[derive(Debug)]
pub(crate) struct App {
  code: String,
  collapsed_nodes: HashSet<usize>,
  cursor: usize,
  scroll_offset: u16,
  selected: Option<usize>,
  tree: Tree,
}

impl App {
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

  fn draw(&self, frame: &mut Frame) {
    let area = frame.area();

    let tree_panel = TreePanel::new(
      &self.tree,
      &self.code,
      self.cursor,
      self.selected,
      &self.collapsed_nodes,
      self.scroll_offset,
    );

    if let Some(selected) = self.selected {
      let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

      frame.render_widget(tree_panel, chunks[0]);
      frame.render_widget(
        InfoPanel::new(self.node(selected), &self.code),
        chunks[1],
      );
    } else {
      frame.render_widget(tree_panel, area);
    }
  }

  #[allow(clippy::cast_possible_truncation)]
  fn ensure_cursor_in_view(&mut self, terminal_height: u16) {
    let mut position = 0;

    let root = self.tree.root_node();

    self.calculate_node_position(&root, self.cursor, &mut position);

    let display_area = terminal_height.saturating_sub(2) as usize;

    if position < self.scroll_offset as usize {
      self.scroll_offset = position as u16;
    } else if position >= (self.scroll_offset as usize + display_area) {
      self.scroll_offset = (position - display_area + 1) as u16;
    }
  }

  fn find_node(id: usize, node: Node<'_>) -> Option<Node<'_>> {
    if node.id() == id {
      return Some(node);
    }

    for i in 0..node.child_count() {
      if let Some(child) = node.child(i) {
        if let Some(found) = Self::find_node(id, child) {
          return Some(found);
        }
      }
    }

    None
  }

  fn move_down(&mut self) {
    let current = self.node(self.cursor);

    if current.child_count() > 0
      && !self.collapsed_nodes.contains(&current.id())
    {
      if let Some(child) = current.child(0) {
        self.cursor = child.id();
      }
    }
  }

  fn move_left(&mut self) {
    let current = self.node(self.cursor);

    if let Some(prev) = current.prev_sibling() {
      self.cursor = prev.id();
    } else if let Some(parent) = current.parent() {
      self.cursor = parent.id();
    }
  }

  fn move_right(&mut self) {
    let current = self.node(self.cursor);

    if let Some(next) = current.next_sibling() {
      self.cursor = next.id();
    }
  }

  fn move_up(&mut self) {
    let current = self.node(self.cursor);

    if let Some(parent) = current.parent() {
      self.cursor = parent.id();
    }
  }

  pub(crate) fn new(filename: PathBuf) -> Result<Self> {
    let code = fs::read_to_string(&filename)?;

    let mut parser = Parser::new();

    let language = Language::try_from(filename)?;

    parser.set_language(&language.into())?;

    let tree = parser
      .parse(&code, None)
      .ok_or_else(|| anyhow!("Failed to parse code"))?;

    let cursor = tree.root_node().id();

    Ok(Self {
      tree,
      code,
      cursor,
      selected: None,
      scroll_offset: 0,
      collapsed_nodes: HashSet::new(),
    })
  }

  fn node(&self, id: usize) -> Node<'_> {
    Self::find_node(id, self.tree.root_node()).expect("node should exist")
  }

  pub(crate) fn run(mut self) -> Result {
    let mut terminal = Terminal::new()?;

    loop {
      terminal.draw(|f| {
        let terminal_height = f.area().height;
        self.ensure_cursor_in_view(terminal_height);
        self.draw(f);
      })?;

      if let Event::Key(key) = event::read()? {
        match key {
          KeyEvent {
            code: KeyCode::Char('q'),
            ..
          } => break,
          KeyEvent {
            code: KeyCode::Char('k'),
            ..
          } => self.move_up(),
          KeyEvent {
            code: KeyCode::Char('j'),
            ..
          } => self.move_down(),
          KeyEvent {
            code: KeyCode::Char('h'),
            ..
          } => self.move_left(),
          KeyEvent {
            code: KeyCode::Char('l'),
            ..
          } => self.move_right(),
          KeyEvent {
            code: KeyCode::Char(' '),
            ..
          } => self.toggle_select(),
          KeyEvent {
            code: KeyCode::Enter,
            ..
          } => self.toggle_collapse(),
          KeyEvent {
            code: KeyCode::Char('u'),
            modifiers: KeyModifiers::CONTROL,
            ..
          } => self.scroll_up(),
          KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::CONTROL,
            ..
          } => self.scroll_down(),
          _ => {}
        }
      }
    }

    Ok(())
  }

  fn scroll_down(&mut self) {
    self.scroll_offset += 1;
  }

  fn scroll_up(&mut self) {
    if self.scroll_offset > 0 {
      self.scroll_offset -= 1;
    }
  }

  fn toggle_collapse(&mut self) {
    let current = self.node(self.cursor);

    let id = current.id();

    let has_children = current.child_count() > 0;

    if has_children && !self.collapsed_nodes.remove(&id) {
      self.collapsed_nodes.insert(id);
    }
  }

  fn toggle_select(&mut self) {
    if self.selected == Some(self.cursor) {
      self.selected = None;
    } else {
      self.selected = Some(self.cursor);
    }
  }
}
