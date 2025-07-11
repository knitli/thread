use crate::ast::{Doc, Node, Root};
use crate::matcher::Matcher;
use crate::tree_sitter::{LanguageExt, StrDoc, TSCursor, TSNode}

pub struct Visitor<M, A = PreOrder> {
  /// Whether a node will match if it contains or is contained in another match.
  reentrant: bool,
  /// Whether visit named node only
  named_only: bool,
  /// optional matcher to filter nodes
  matcher: M,
  /// The algorithm to traverse the tree, can be pre/post/level order
  algorithm: std::marker::PhantomData<A>,
}

pub struct Visit<'t, D, T, M> {
  reentrant: bool,
  named: bool,
  matcher: M,
  traversal: T,
  lang: std::marker::PhantomData<&'t D>,
}

/// Traversal can iterate over node by using traversal algorithm.
/// The `next` method should only handle normal, reentrant iteration.
/// If reentrancy is not desired, traversal should mutate cursor in `calibrate_for_match`.
/// Visit will maintain the matched node depth so traversal does not need to use extra field.
pub trait Traversal<'t, D: Doc + 't>: Iterator<Item = Node<'t, D>> {
  /// Calibrate cursor position to skip overlapping matches.
  /// node depth will be passed if matched, otherwise None.
  fn calibrate_for_match(&mut self, depth: Option<usize>);
  /// Returns the current depth of cursor depth.
  /// Cursor depth is incremented by 1 when moving from parent to child.
  /// Cursor depth at Root node is 0.
  fn get_current_depth(&self) -> usize;
}

/// Represents a pre-order traversal
pub struct TsPre<'tree> {
  cursor: TSTreeCursor<'tree>,
  // record the starting node, if we return back to starting point
  // we should terminate the dfs.
  start_id: Option<usize>,
  current_depth: usize,
}

pub struct Pre<'tree, L: LanguageExt> {
  root: &'tree Root<StrDoc<L>>,
  inner: TsPre<'tree>,
}

pub trait Algorithm {
  type Traversal<'t, L: LanguageExt>: Traversal<'t, StrDoc<L>>;
  fn traverse<L: LanguageExt>(node: Node<StrDoc<L>>) -> Self::Traversal<'_, L>;
}

pub struct PreOrder;
impl Algorithm for PreOrder {
  type Traversal<'t, L: LanguageExt> = Pre<'t, L>;
  fn traverse<L: LanguageExt>(node: Node<StrDoc<L>>) -> Self::Traversal<'_, L> {
    Pre::new(&node)
  }
}
pub struct PostOrder;
impl Algorithm for PostOrder {
  type Traversal<'t, L: LanguageExt> = Post<'t, L>;
  fn traverse<L: LanguageExt>(node: Node<StrDoc<L>>) -> Self::Traversal<'_, L> {
    Post::new(&node)
  }
}

/// Represents a post-order traversal
pub struct Post<'tree, L: LanguageExt> {
  cursor: TSTreeCursor<'tree>,
  root: &'tree Root<StrDoc<L>>,
  start_id: Option<usize>,
  current_depth: usize,
  match_depth: usize,
}

/// Represents a level-order traversal.
/// It is implemented with [`VecDeque`] since quadratic backtracking is too time consuming.
/// Though level-order is not used as frequently as other DFS traversals,
/// traversing a big AST with level-order should be done with caution since it might increase the memory usage.
pub struct Level<'tree, L: LanguageExt> {
  deque: VecDeque<TSNode<'tree>>,
  cursor: TSCursor<'tree>,
  root: &'tree Root<StrDoc<L>>,
}
