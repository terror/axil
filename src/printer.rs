use super::*;

pub(crate) struct Printer<'a> {
  code: &'a str,
  tree: &'a Tree,
}

impl<'a> Printer<'a> {
  pub(crate) fn new(tree: &'a Tree, code: &'a str) -> Self {
    Self { code, tree }
  }

  pub(crate) fn print(&self) {
    self.print_node(&self.tree.root_node(), 0);
  }

  fn print_node(&self, node: &Node, depth: usize) {
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

    for i in 0..node.child_count() {
      if let Some(child) = node.child(i) {
        self.print_node(&child, depth + 1);
      }
    }
  }
}
