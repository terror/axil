use super::*;

#[derive(Clone, Debug)]
pub(crate) struct NodeHandle {
  pub(crate) id: usize,
  tree: Rc<Tree>,
}

impl NodeHandle {
  pub(crate) fn new(tree: Rc<Tree>) -> Self {
    let id = tree.root_node().id();

    Self { id, tree }
  }

  pub(crate) fn node(&self) -> Node {
    Self::find_node_by_id(self.id, self.tree.root_node())
      .expect("Node should always exist")
  }

  pub(crate) fn parent(&self) -> Option<Self> {
    self.node().parent().map(|parent| Self {
      id: parent.id(),
      ..self.clone()
    })
  }

  pub(crate) fn child(&self, index: usize) -> Option<Self> {
    self.node().child(index).map(|child| Self {
      id: child.id(),
      ..self.clone()
    })
  }

  pub(crate) fn prev_sibling(&self) -> Option<Self> {
    self.node().prev_sibling().map(|sibling| Self {
      id: sibling.id(),
      ..self.clone()
    })
  }

  pub(crate) fn next_sibling(&self) -> Option<Self> {
    self.node().next_sibling().map(|sibling| Self {
      id: sibling.id(),
      ..self.clone()
    })
  }

  fn find_node_by_id(id: usize, node: Node) -> Option<Node> {
    if node.id() == id {
      return Some(node);
    }

    for i in 0..node.child_count() {
      if let Some(child) = node.child(i) {
        if let Some(found) = Self::find_node_by_id(id, child) {
          return Some(found);
        }
      }
    }

    None
  }
}
