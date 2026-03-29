use super::*;

pub(crate) struct Printer<'a> {
  code: &'a str,
  matches: HashSet<usize>,
  tree: &'a Tree,
}

impl<'a> Printer<'a> {
  fn has_match_descendant(&self, node: &Node) -> bool {
    if self.matches.contains(&node.id()) {
      return true;
    }

    for i in 0..node.child_count_u32() {
      if let Some(child) = node.child(i) {
        if self.has_match_descendant(&child) {
          return true;
        }
      }
    }

    false
  }

  pub(crate) fn new(
    tree: &'a Tree,
    code: &'a str,
    matches: HashSet<usize>,
  ) -> Self {
    Self {
      code,
      matches,
      tree,
    }
  }

  pub(crate) fn print(&self) {
    self.print_node(&self.tree.root_node(), 0);
  }

  fn print_node(&self, node: &Node, depth: usize) {
    let filtering = !self.matches.is_empty();

    if filtering && !self.has_match_descendant(node) {
      return;
    }

    let indent = "  ".repeat(depth);

    let text = if node.child_count() == 0 {
      format!(" \"{}\"", &self.code[node.start_byte()..node.end_byte()])
    } else {
      String::new()
    };

    println!(
      "{indent}{} [{}:{}..{}:{}]{text}",
      node.kind(),
      node.start_position().row,
      node.start_position().column,
      node.end_position().row,
      node.end_position().column,
    );

    for i in 0..node.child_count_u32() {
      if let Some(child) = node.child(i) {
        self.print_node(&child, depth + 1);
      }
    }
  }
}
