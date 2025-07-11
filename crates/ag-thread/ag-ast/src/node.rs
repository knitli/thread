pub use ag_service_types::{Doc, Position, Root};
use crate::pinned::Node; // pinned adds `NodeData` trait to Node
use ag_service_types::{Language, Edit as E, KindId};

pub use Node;

use std::borrow::Cow;

type Edit<D> = E<<D as Doc>::Source>;

impl Position {
  pub fn new(line: usize, byte_column: usize, byte_offset: usize) -> Self {
    Self {
      line,
      byte_column,
      byte_offset,
    }
  }
  pub fn line(&self) -> usize {
    self.line
  }
  /// Returns the column in terms of characters.
  /// Note: node does not have to be a node of matching position.
  pub fn column<D: Doc>(&self, node: &Node<'_, D>) -> usize {
    let source = node.get_doc().get_source();
    source.get_char_column(self.byte_column, self.byte_offset)
  }
  pub fn byte_point(&self) -> (usize, usize) {
    (self.line, self.byte_column)
  }
}

impl<D: Doc> Root<D> {
  pub fn doc(doc: D) -> Self {
    Self { doc }
  }

  pub fn lang(&self) -> &D::Lang {
    self.doc.get_lang()
  }
  /// The root node represents the entire source
  pub fn root(&self) -> Node<D> {
    Node {
      inner: self.doc.root_node(),
      root: self,
    }
  }

  // extract non generic implementation to reduce code size
  pub fn edit(&mut self, edit: Edit<D>) -> Result<&mut Self, String> {
    self.doc.do_edit(&edit)?;
    Ok(self)
  }


  /// Adopt the tree_sitter as the descendant of the root and return the wrapped sg Node.
  /// It assumes `inner` is the under the root and will panic at dev build if wrong node is used.
  pub fn adopt<'r>(&'r self, inner: D::Node<'r>) -> Node<'r, D> {
    debug_assert!(self.check_lineage(&inner));
    Node { inner, root: self }
  }

  fn check_lineage(&self, inner: &D::Node<'_>) -> bool {
    let mut node = inner.clone();
    while let Some(n) = node.parent() {
      node = n;
    }
    node.node_id() == self.doc.root_node().node_id()
  }

  /// P.S. I am your father.
  #[doc(hidden)]
  pub unsafe fn readopt<'a: 'b, 'b>(&'a self, node: &mut Node<'b, D>) {
    debug_assert!(self.check_lineage(&node.inner));
    node.root = self;
  }
}

/// APIs for Node inspection
impl<'r, D: Doc> Node<'r, D> {
  pub fn get_doc(&self) -> &'r D {
    &self.root.doc
  }
  pub fn node_id(&self) -> usize {
    self.inner.node_id()
  }
  pub fn is_leaf(&self) -> bool {
    self.inner.is_leaf()
  }
  /// if has no named children.
  /// N.B. it is different from is_named && is_leaf
  // see https://github.com/ast-grep/ast-grep/issues/276
  pub fn is_named_leaf(&self) -> bool {
    self.inner.is_named_leaf()
  }
  pub fn is_error(&self) -> bool {
    self.inner.is_error()
  }
  pub fn kind(&self) -> Cow<str> {
    self.inner.kind()
  }
  pub fn kind_id(&self) -> KindId {
    self.inner.kind_id()
  }

  pub fn is_named(&self) -> bool {
    self.inner.is_named()
  }
  pub fn is_missing(&self) -> bool {
    self.inner.is_missing()
  }

  /// byte offsets of start and end.
  pub fn range(&self) -> std::ops::Range<usize> {
    self.inner.range()
  }

  /// Nodes' start position in terms of zero-based rows and columns.
  pub fn start_pos(&self) -> Position {
    self.inner.start_pos()
  }

  /// Nodes' end position in terms of rows and columns.
  pub fn end_pos(&self) -> Position {
    self.inner.end_pos()
  }

  pub fn text(&self) -> Cow<'r, str> {
    self.root.doc.get_node_text(&self.inner)
  }

  pub fn lang(&self) -> &'r D::Lang {
    self.root.lang()
  }

  /// the underlying tree-sitter Node
  pub fn get_inner_node(&self) -> D::Node<'r> {
    self.inner.clone()
  }

  pub fn root(&self) -> &'r Root<D> {
    self.root
  }
}


/// tree traversal API
impl<'r, D: Doc> Node<'r, D> {
  #[must_use]
  pub fn parent(&self) -> Option<Self> {
    let inner = self.inner.parent()?;
    Some(Node {
      inner,
      root: self.root,
    })
  }

  pub fn children(&self) -> impl ExactSizeIterator<Item = Node<'r, D>> + '_ {
    self.inner.children().map(|inner| Node {
      inner,
      root: self.root,
    })
  }

  #[must_use]
  pub fn child(&self, nth: usize) -> Option<Self> {
    let inner = self.inner.child(nth)?;
    Some(Node {
      inner,
      root: self.root,
    })
  }

  pub fn field(&self, name: &str) -> Option<Self> {
    let inner = self.inner.field(name)?;
    Some(Node {
      inner,
      root: self.root,
    })
  }

  pub fn child_by_field_id(&self, field_id: u16) -> Option<Self> {
    let inner = self.inner.child_by_field_id(field_id)?;
    Some(Node {
      inner,
      root: self.root,
    })
  }

  pub fn field_children(&self, name: &str) -> impl Iterator<Item = Node<'r, D>> + '_ {
    let field_id = self.lang().field_to_id(name);
    self.inner.field_children(field_id).map(|inner| Node {
      inner,
      root: self.root,
    })
  }

  /// Returns all ancestors nodes of `self`.
  /// Using cursor is overkill here because adjust cursor is too expensive.
  pub fn ancestors(&self) -> impl Iterator<Item = Node<'r, D>> + '_ {
    let root = self.root.doc.root_node();
    self.inner.ancestors(root).map(|inner| Node {
      inner,
      root: self.root,
    })
  }
  #[must_use]
  pub fn next(&self) -> Option<Self> {
    let inner = self.inner.next()?;
    Some(Node {
      inner,
      root: self.root,
    })
  }

  /// Returns all sibling nodes next to `self`.
  // NOTE: Need go to parent first, then move to current node by byte offset.
  // This is because tree_sitter cursor is scoped to the starting node.
  // See https://github.com/tree-sitter/tree-sitter/issues/567
  pub fn next_all(&self) -> impl Iterator<Item = Node<'r, D>> + '_ {
    self.inner.next_all().map(|inner| Node {
      inner,
      root: self.root,
    })
  }

  #[must_use]
  pub fn prev(&self) -> Option<Node<'r, D>> {
    let inner = self.inner.prev()?;
    Some(Node {
      inner,
      root: self.root,
    })
  }

  pub fn prev_all(&self) -> impl Iterator<Item = Node<'r, D>> + '_ {
    self.inner.prev_all().map(|inner| Node {
      inner,
      root: self.root,
    })
  }

  pub fn dfs<'s>(&'s self) -> impl Iterator<Item = Node<'r, D>> + 's {
    self.inner.dfs().map(|inner| Node {
      inner,
      root: self.root,
    })
  }

}

/// Tree manipulation API
impl<D: Doc> Node<'_, D> {
  pub fn after(&self) -> Edit<D> {
    todo!()
  }
  pub fn before(&self) -> Edit<D> {
    todo!()
  }
  pub fn append(&self) -> Edit<D> {
    todo!()
  }
  pub fn prepend(&self) -> Edit<D> {
    todo!()
  }

  /// Empty children. Remove all child node
  pub fn empty(&self) -> Option<Edit<D>> {
    let mut children = self.children().peekable();
    let start = children.peek()?.range().start;
    let end = children.last()?.range().end;
    Some(Edit::<D> {
      position: start,
      deleted_length: end - start,
      inserted_text: Vec::new(),
    })
  }

  /// Remove the node itself
  pub fn remove(&self) -> Edit<D> {
    let range = self.range();
    Edit::<D> {
      position: range.start,
      deleted_length: range.end - range.start,
      inserted_text: Vec::new(),
    }
  }
}
