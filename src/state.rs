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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn move_down_enters_first_child() {
    let tree = parse("fn foo() {}");
    let root = tree.root_node();
    let mut state = State::new(root.id());

    state.move_down(&tree).unwrap();

    let cursor_node = state.node(&tree).unwrap();
    assert_eq!(cursor_node.kind(), "function_item");
  }

  #[test]
  fn move_down_skips_collapsed() {
    let tree = parse("fn foo() {}");
    let root = tree.root_node();
    let mut state = State::new(root.id());

    state.toggle_collapse(&tree).unwrap();
    let before = state.cursor;
    state.move_down(&tree).unwrap();

    assert_eq!(state.cursor, before);
  }

  #[test]
  fn move_left_falls_back_to_parent() {
    let tree = parse("fn foo() {}");
    let root = tree.root_node();
    let first = root.child(0).unwrap();
    let mut state = State::new(first.id());

    state.move_left(&tree).unwrap();

    assert_eq!(state.cursor, root.id());
  }

  #[test]
  fn move_left_goes_to_prev_sibling() {
    let tree = parse("fn foo() {} fn bar() {}");
    let root = tree.root_node();
    let second = root.child(1).unwrap();
    let first = root.child(0).unwrap();
    let mut state = State::new(second.id());

    state.move_left(&tree).unwrap();

    assert_eq!(state.cursor, first.id());
  }

  #[test]
  fn move_right_goes_to_next_sibling() {
    let tree = parse("fn foo() {} fn bar() {}");
    let root = tree.root_node();
    let first = root.child(0).unwrap();
    let second = root.child(1).unwrap();
    let mut state = State::new(first.id());

    state.move_right(&tree).unwrap();

    assert_eq!(state.cursor, second.id());
  }

  #[test]
  fn move_up_goes_to_parent() {
    let tree = parse("fn foo() {}");
    let root = tree.root_node();
    let fn_item = root.child(0).unwrap();
    let mut state = State::new(fn_item.id());

    state.move_up(&tree).unwrap();

    assert_eq!(state.cursor, root.id());
  }

  #[test]
  fn node_not_found() {
    let tree = parse("fn foo() {}");
    let mut state = State::new(tree.root_node().id());
    state.cursor = usize::MAX;

    assert!(state.node(&tree).is_err());
  }

  fn parse(code: &str) -> Tree {
    let mut parser = Parser::new();

    parser
      .set_language(&tree_sitter_rust::LANGUAGE.into())
      .unwrap();

    parser.parse(code, None).unwrap()
  }

  #[test]
  fn scroll() {
    let mut state = State::new(0);

    state.scroll_down();
    assert_eq!(state.scroll_offset, 1);

    state.scroll_up();
    assert_eq!(state.scroll_offset, 0);

    state.scroll_up();
    assert_eq!(state.scroll_offset, 0);
  }

  #[test]
  fn toggle_collapse() {
    let tree = parse("fn foo() {}");
    let root = tree.root_node();
    let mut state = State::new(root.id());

    state.toggle_collapse(&tree).unwrap();
    assert!(state.collapsed_nodes.contains(&root.id()));

    state.toggle_collapse(&tree).unwrap();
    assert!(!state.collapsed_nodes.contains(&root.id()));
  }

  #[test]
  fn toggle_collapse_leaf_is_noop() {
    let tree = parse("fn foo() {}");
    let root = tree.root_node();
    let fn_item = root.child(0).unwrap();
    let fn_keyword = fn_item.child(0).unwrap();
    assert_eq!(fn_keyword.child_count(), 0);
    let mut state = State::new(fn_keyword.id());

    state.toggle_collapse(&tree).unwrap();

    assert!(state.collapsed_nodes.is_empty());
  }

  #[test]
  fn toggle_select() {
    let tree = parse("fn foo() {}");
    let root = tree.root_node();
    let mut state = State::new(root.id());

    state.toggle_select();
    assert_eq!(state.selected, Some(root.id()));

    state.toggle_select();
    assert_eq!(state.selected, None);
  }
}
