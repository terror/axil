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
  pub(crate) fn run(mut self) -> Result {
    let mut terminal = Terminal::new()?;

    loop {
      terminal.draw(|f| {
        let terminal_height = f.area().height;
        self.ensure_cursor_in_view(terminal_height);
        self.draw(f)
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

  fn draw(&self, frame: &mut Frame) {
    let area = frame.area();

    let cursor_id = self.cursor_node.node().id();

    let selected_id = self.selected_node.as_ref().map(|n| n.id);

    let tree_panel = TreePanel::new(
      &self.tree,
      &self.code,
      cursor_id,
      selected_id,
      &self.collapsed_nodes,
      self.scroll_offset,
    );

    if let Some(selected_handle) = &self.selected_node {
      let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

      frame.render_widget(tree_panel, chunks[0]);
      frame.render_widget(
        InfoPanel::new(selected_handle.node(), &self.code),
        chunks[1],
      );
    } else {
      frame.render_widget(tree_panel, area);
    }
  }

  fn ensure_cursor_in_view(&mut self, terminal_height: u16) {
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

  fn move_down(&mut self) {
    let current = self.cursor_node.node();

    if current.child_count() > 0
      && !self.collapsed_nodes.contains(&current.id())
    {
      if let Some(child) = self.cursor_node.child(0) {
        self.cursor_node = child;
      }
    }
  }

  fn move_left(&mut self) {
    if let Some(prev) = self.cursor_node.prev_sibling() {
      self.cursor_node = prev;
    } else if let Some(parent) = self.cursor_node.parent() {
      self.cursor_node = parent;
    }
  }

  fn move_right(&mut self) {
    if let Some(next) = self.cursor_node.next_sibling() {
      self.cursor_node = next;
    }
  }

  fn move_up(&mut self) {
    if let Some(parent) = self.cursor_node.parent() {
      self.cursor_node = parent;
    }
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

  fn toggle_select(&mut self) {
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
