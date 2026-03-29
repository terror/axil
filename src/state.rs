use super::*;

#[derive(Debug)]
pub(crate) struct State {
  pub(crate) collapsed_nodes: HashSet<usize>,
  pub(crate) cursor: usize,
  pub(crate) matches: Vec<usize>,
  pub(crate) scroll_offset: u16,
  pub(crate) search_query: String,
  pub(crate) selected: Option<usize>,
  pub(crate) ts_query: String,
  pub(crate) ts_query_error: Option<String>,
  pub(crate) ts_query_matches: Vec<usize>,
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

  pub(crate) fn clear_query(&mut self) {
    self.ts_query.clear();
    self.ts_query_matches.clear();
    self.ts_query_error = None;
  }

  pub(crate) fn clear_search(&mut self) {
    self.search_query.clear();
    self.matches.clear();
  }

  fn collect_matches(
    node: Node,
    code: &str,
    query: &str,
    matches: &mut Vec<usize>,
  ) {
    let kind_matches = node.kind().to_lowercase().contains(query);

    let text_matches = node.child_count() == 0
      && code[node.start_byte()..node.end_byte()]
        .to_lowercase()
        .contains(query);

    if kind_matches || text_matches {
      matches.push(node.id());
    }

    for i in 0..node.child_count() {
      if let Some(child) = node.child(i) {
        Self::collect_matches(child, code, query, matches);
      }
    }
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

  pub(crate) fn execute_query(
    &mut self,
    language: &TreeSitterLanguage,
    tree: &Tree,
    code: &str,
  ) {
    self.ts_query_matches.clear();
    self.ts_query_error = None;

    if self.ts_query.is_empty() {
      return;
    }

    match Query::new(language, &self.ts_query) {
      Ok(query) => {
        let mut cursor = QueryCursor::new();
        let mut matches =
          cursor.matches(&query, tree.root_node(), code.as_bytes());

        while let Some(m) = matches.next() {
          for capture in m.captures {
            self.ts_query_matches.push(capture.node.id());
          }
        }

        self.ts_query_matches.dedup();

        if let Some(&first) = self.ts_query_matches.first() {
          self.cursor = first;
        }
      }
      Err(e) => {
        self.ts_query_error = Some(e.message);
      }
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

  fn jump_in(&mut self, matches: &[usize], forward: bool) {
    if matches.is_empty() {
      return;
    }

    let current = matches.iter().position(|&id| id == self.cursor);

    let index = if forward {
      match current {
        Some(i) => (i + 1) % matches.len(),
        None => 0,
      }
    } else {
      match current {
        Some(i) => (i + matches.len() - 1) % matches.len(),
        None => matches.len() - 1,
      }
    };

    self.cursor = matches[index];
  }

  pub(crate) fn jump_to_match(&mut self, forward: bool) {
    if !self.matches.is_empty() {
      let matches = self.matches.clone();
      self.jump_in(&matches, forward);
    } else if !self.ts_query_matches.is_empty() {
      let matches = self.ts_query_matches.clone();
      self.jump_in(&matches, forward);
    }
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
      matches: Vec::new(),
      scroll_offset: 0,
      search_query: String::new(),
      selected: None,
      ts_query: String::new(),
      ts_query_error: None,
      ts_query_matches: Vec::new(),
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

  pub(crate) fn search(&mut self, tree: &Tree, code: &str) {
    self.matches.clear();

    if !self.search_query.is_empty() {
      let query = self.search_query.to_lowercase();
      Self::collect_matches(tree.root_node(), code, &query, &mut self.matches);
    }

    if let Some(&first) = self.matches.first() {
      self.cursor = first;
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

  fn language() -> TreeSitterLanguage {
    tree_sitter_rust::LANGUAGE.into()
  }

  #[test]
  fn move_down_enters_first_child() {
    let tree = parse("fn foo() {}");
    let root = tree.root_node();

    let mut state = State::new(root.id());

    state.move_down(&tree).unwrap();

    assert_eq!(state.node(&tree).unwrap().kind(), "function_item");
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

    let mut state = State::new(root.child(0).unwrap().id());

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

    parser.set_language(&language()).unwrap();

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
  fn search_by_kind() {
    let code = "fn foo() {} fn bar() {}";
    let tree = parse(code);

    let mut state = State::new(tree.root_node().id());

    state.search_query = "identifier".to_string();
    state.search(&tree, code);

    assert_eq!(state.matches.len(), 2);
    assert_eq!(state.node(&tree).unwrap().kind(), "identifier");
  }

  #[test]
  fn search_by_text() {
    let code = "fn foo() {} fn bar() {}";
    let tree = parse(code);

    let mut state = State::new(tree.root_node().id());

    state.search_query = "bar".to_string();
    state.search(&tree, code);

    assert_eq!(state.matches.len(), 1);
    assert_eq!(
      &code[state.node(&tree).unwrap().start_byte()
        ..state.node(&tree).unwrap().end_byte()],
      "bar"
    );
  }

  #[test]
  fn search_case_insensitive() {
    let code = "fn Foo() {}";
    let tree = parse(code);

    let mut state = State::new(tree.root_node().id());

    state.search_query = "foo".to_string();
    state.search(&tree, code);

    assert_eq!(state.matches.len(), 1);
  }

  #[test]
  fn search_empty_query() {
    let code = "fn foo() {}";
    let tree = parse(code);

    let mut state = State::new(tree.root_node().id());
    let cursor_before = state.cursor;

    state.search(&tree, code);

    assert!(state.matches.is_empty());
    assert_eq!(state.cursor, cursor_before);
  }

  #[test]
  fn search_jump_forward_and_backward() {
    let code = "fn foo() {} fn bar() {}";
    let tree = parse(code);

    let mut state = State::new(tree.root_node().id());

    state.search_query = "identifier".to_string();
    state.search(&tree, code);

    let first = state.cursor;

    state.jump_to_match(true);
    let second = state.cursor;
    assert_ne!(first, second);

    state.jump_to_match(false);
    assert_eq!(state.cursor, first);
  }

  #[test]
  fn search_no_matches() {
    let code = "fn foo() {}";
    let tree = parse(code);

    let mut state = State::new(tree.root_node().id());
    let cursor_before = state.cursor;

    state.search_query = "zzz".to_string();
    state.search(&tree, code);

    assert!(state.matches.is_empty());
    assert_eq!(state.cursor, cursor_before);
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

  #[test]
  fn ts_query_clear() {
    let code = "fn foo() {}";
    let tree = parse(code);
    let lang = language();

    let mut state = State::new(tree.root_node().id());

    state.ts_query = "(identifier) @name".to_string();
    state.execute_query(&lang, &tree, code);
    assert!(!state.ts_query_matches.is_empty());

    state.clear_query();

    assert!(state.ts_query.is_empty());
    assert!(state.ts_query_matches.is_empty());
    assert!(state.ts_query_error.is_none());
  }

  #[test]
  fn ts_query_empty_is_noop() {
    let code = "fn foo() {}";
    let tree = parse(code);
    let lang = language();

    let mut state = State::new(tree.root_node().id());
    let cursor_before = state.cursor;

    state.execute_query(&lang, &tree, code);

    assert!(state.ts_query_matches.is_empty());
    assert!(state.ts_query_error.is_none());
    assert_eq!(state.cursor, cursor_before);
  }

  #[test]
  fn ts_query_invalid_sets_error() {
    let code = "fn foo() {}";
    let tree = parse(code);
    let lang = language();

    let mut state = State::new(tree.root_node().id());

    state.ts_query = "(not_a_real_node)".to_string();
    state.execute_query(&lang, &tree, code);

    assert!(state.ts_query_matches.is_empty());
    assert!(state.ts_query_error.is_some());
  }

  #[test]
  fn ts_query_jump_forward_and_backward() {
    let code = "fn foo() {} fn bar() {}";
    let tree = parse(code);
    let lang = language();

    let mut state = State::new(tree.root_node().id());

    state.ts_query = "(identifier) @name".to_string();
    state.execute_query(&lang, &tree, code);

    let first = state.cursor;

    state.jump_to_match(true);
    let second = state.cursor;
    assert_ne!(first, second);

    state.jump_to_match(false);
    assert_eq!(state.cursor, first);
  }

  #[test]
  fn ts_query_matches_captures() {
    let code = "fn foo() {} fn bar() {}";
    let tree = parse(code);
    let lang = language();

    let mut state = State::new(tree.root_node().id());

    state.ts_query = "(identifier) @name".to_string();
    state.execute_query(&lang, &tree, code);

    assert_eq!(state.ts_query_matches.len(), 2);
    assert!(state.ts_query_error.is_none());
    assert_eq!(state.node(&tree).unwrap().kind(), "identifier");
  }

  #[test]
  fn ts_query_multiple_patterns() {
    let code = "fn foo() { let x = 42; }";
    let tree = parse(code);
    let lang = language();

    let mut state = State::new(tree.root_node().id());

    state.ts_query = "(identifier) @id (integer_literal) @num".to_string();
    state.execute_query(&lang, &tree, code);

    assert!(state.ts_query_matches.len() >= 3);
    assert!(state.ts_query_error.is_none());
  }

  #[test]
  fn ts_query_no_matches() {
    let code = "fn foo() {}";
    let tree = parse(code);
    let lang = language();

    let mut state = State::new(tree.root_node().id());
    let cursor_before = state.cursor;

    state.ts_query = "(struct_item) @s".to_string();
    state.execute_query(&lang, &tree, code);

    assert!(state.ts_query_matches.is_empty());
    assert!(state.ts_query_error.is_none());
    assert_eq!(state.cursor, cursor_before);
  }

  #[test]
  fn ts_query_syntax_error() {
    let code = "fn foo() {}";
    let tree = parse(code);
    let lang = language();

    let mut state = State::new(tree.root_node().id());

    state.ts_query = "(((".to_string();
    state.execute_query(&lang, &tree, code);

    assert!(state.ts_query_matches.is_empty());
    assert!(state.ts_query_error.is_some());
  }
}
