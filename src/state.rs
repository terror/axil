use super::*;

#[derive(Debug)]
pub(crate) struct State {
  pub(crate) collapsed_nodes: HashSet<usize>,
  pub(crate) cursor: usize,
  pub(crate) scroll_offset: u16,
  pub(crate) selected: Option<usize>,
}

impl State {
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

  #[allow(clippy::cast_possible_truncation)]
  pub(crate) fn ensure_cursor_in_view(
    &mut self,
    tree: &Tree,
    terminal_height: u16,
  ) {
    let mut position = 0;

    let root = tree.root_node();

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

    (0..node.child_count())
      .filter_map(|i| node.child(i))
      .find_map(|child| Self::find_node(id, child))
  }

  pub(crate) fn move_down(&mut self, tree: &Tree) -> Result {
    let current = self.node(tree)?;

    if current.child_count() > 0
      && !self.collapsed_nodes.contains(&current.id())
    {
      if let Some(child) = current.child(0) {
        self.cursor = child.id();
      }
    }

    Ok(())
  }

  pub(crate) fn move_left(&mut self, tree: &Tree) -> Result {
    let current = self.node(tree)?;

    if let Some(prev) = current.prev_sibling() {
      self.cursor = prev.id();
    } else if let Some(parent) = current.parent() {
      self.cursor = parent.id();
    }

    Ok(())
  }

  pub(crate) fn move_right(&mut self, tree: &Tree) -> Result {
    let current = self.node(tree)?;

    if let Some(next) = current.next_sibling() {
      self.cursor = next.id();
    }

    Ok(())
  }

  pub(crate) fn move_up(&mut self, tree: &Tree) -> Result {
    let current = self.node(tree)?;

    if let Some(parent) = current.parent() {
      self.cursor = parent.id();
    }

    Ok(())
  }

  pub(crate) fn new(cursor: usize) -> Self {
    Self {
      collapsed_nodes: HashSet::new(),
      cursor,
      scroll_offset: 0,
      selected: None,
    }
  }

  pub(crate) fn node<'a>(&self, tree: &'a Tree) -> Result<Node<'a>> {
    Self::find_node(self.cursor, tree.root_node())
      .ok_or_else(|| anyhow!("cursor node `{}` not found in tree", self.cursor))
  }

  pub(crate) fn scroll_down(&mut self) {
    self.scroll_offset += 1;
  }

  pub(crate) fn scroll_up(&mut self) {
    if self.scroll_offset > 0 {
      self.scroll_offset -= 1;
    }
  }

  pub(crate) fn toggle_collapse(&mut self, tree: &Tree) -> Result {
    let current = self.node(tree)?;

    let id = current.id();

    let has_children = current.child_count() > 0;

    if has_children && !self.collapsed_nodes.remove(&id) {
      self.collapsed_nodes.insert(id);
    }

    Ok(())
  }

  pub(crate) fn toggle_select(&mut self) {
    if self.selected == Some(self.cursor) {
      self.selected = None;
    } else {
      self.selected = Some(self.cursor);
    }
  }
}
