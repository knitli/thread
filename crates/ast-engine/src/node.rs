// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

//! # AST Node Representation and Navigation
//!
//! Core types for representing and navigating Abstract Syntax Tree nodes.
//!
//! ## Key Types
//!
//! - [`Node`] - A single AST node with navigation and matching capabilities
//! - [`Root`] - The root of an AST tree, owns the source code and tree structure
//! - [`Position`] - Represents a position in source code (line/column)
//!
//! ## Usage
//!
//! ```rust,no_run
//! # use thread_ast_engine::Language;
//! # use thread_ast_engine::tree_sitter::LanguageExt;
//! # use thread_ast_engine::MatcherExt;
//! let ast = Language::Tsx.ast_grep("function foo() { return 42; }");
//! let root_node = ast.root();
//!
//! // Navigate the tree
//! for child in root_node.children() {
//!     println!("Child kind: {}", child.kind());
//! }
//!
//! // Find specific patterns
//! if let Some(func) = root_node.find("function $NAME() { $$$BODY }") {
//!     println!("Found function: {}", func.get_env().get_match("NAME").unwrap().text());
//! }
//! ```

use crate::Doc;
use crate::Language;
#[cfg(feature = "matching")]
use crate::matcher::{Matcher, MatcherExt, NodeMatch};
#[cfg(feature = "matching")]
use crate::replacer::Replacer;
use crate::source::{Content, Edit as E, SgNode};

type Edit<D> = E<<D as Doc>::Source>;

use std::borrow::Cow;

/// Represents a position in source code.
///
/// Positions use zero-based line and column numbers, where line 0 is the first line
/// and column 0 is the first character. Unlike tree-sitter's internal positions,
/// these are character-based rather than byte-based for easier human consumption.
///
/// # Note
///
/// Computing the character column from byte positions is an O(n) operation,
/// so avoid calling [`Position::column`] in performance-critical loops.
///
/// # Example
///
/// ```rust,no_run
/// # use thread_ast_engine::Language;
/// # use thread_ast_engine::tree_sitter::LanguageExt;
/// let ast = Language::Tsx.ast_grep("let x = 42;\nlet y = 24;");
/// let root = ast.root();
///
/// let start_pos = root.start_pos();
/// assert_eq!(start_pos.line(), 0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// Zero-based line number (line 0 = first line)
    line: usize,
    /// Zero-based byte offset within the line
    byte_column: usize,
    /// Absolute byte offset from start of file
    byte_offset: usize,
}

impl Position {
    #[must_use]
    pub const fn new(line: usize, byte_column: usize, byte_offset: usize) -> Self {
        Self {
            line,
            byte_column,
            byte_offset,
        }
    }
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }
    /// Returns the column in terms of characters.
    /// Note: node does not have to be a node of matching position.
    pub fn column<D: Doc>(&self, node: &Node<'_, D>) -> usize {
        let source = node.get_doc().get_source();
        source.get_char_column(self.byte_column, self.byte_offset)
    }
    #[must_use]
    pub const fn byte_point(&self) -> (usize, usize) {
        (self.line, self.byte_column)
    }
}

/// Root of an AST tree that owns the source code and parsed tree structure.
///
/// Root acts as the entry point for all AST operations. It manages the document
/// (source code + parsed tree) and provides methods to get the root node and
/// perform tree-wide operations like replacements.
///
/// # Generic Parameters
///
/// - `D: Doc` - The document type that holds source code and language information
///
/// # Example
///
/// ```rust,no_run
/// # use thread_ast_engine::Language;
/// # use thread_ast_engine::tree_sitter::LanguageExt;
/// # use thread_ast_engine::MatcherExt;
/// let mut ast = Language::Tsx.ast_grep("let x = 42;");
/// let root_node = ast.root();
///
/// // Perform tree-wide replacements
/// ast.replace("let $VAR = $VALUE", "const $VAR = $VALUE");
/// println!("{}", ast.generate());
/// ```
#[derive(Clone, Debug)]
pub struct Root<D: Doc> {
    pub(crate) doc: D,
}

impl<D: Doc> Root<D> {
    pub const fn doc(doc: D) -> Self {
        Self { doc }
    }

    pub fn lang(&self) -> &D::Lang {
        self.doc.get_lang()
    }
    /// The root node represents the entire source
    pub fn root(&self) -> Node<'_, D> {
        Node {
            inner: self.doc.root_node(),
            root: self,
        }
    }

    // extract non generic implementation to reduce code size
    pub fn edit(&mut self, edit: &Edit<D>) -> Result<&mut Self, String> {
        self.doc.do_edit(edit)?;
        Ok(self)
    }

    #[cfg(feature = "matching")]
    pub fn replace<M: Matcher, R: Replacer<D>>(
        &mut self,
        pattern: M,
        replacer: R,
    ) -> Result<bool, String> {
        let root = self.root();
        if let Some(edit) = root.replace(pattern, replacer) {
            drop(root); // rust cannot auto drop root if D is not specified
            self.edit(&edit)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Adopt the `tree_sitter` as the descendant of the root and return the wrapped sg Node.
    /// It assumes `inner` is under the root and will panic at dev build if wrong node is used.
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

/// A single node in an Abstract Syntax Tree.
///
/// Node represents a specific element in the parsed AST, such as a function declaration,
/// variable assignment, or expression. Each node knows its position in the source code,
/// its type (kind), and provides methods for navigation and pattern matching.
///
/// # Lifetime
///
/// The lifetime `'r` ties the node to its root AST, ensuring memory safety.
/// Nodes cannot outlive the Root that owns the underlying tree structure.
///
/// # Example
///
/// ```rust,no_run
/// # use thread_ast_engine::Language;
/// # use thread_ast_engine::tree_sitter::LanguageExt;
/// # use thread_ast_engine::matcher::MatcherExt;
/// let ast = Language::Tsx.ast_grep("function hello() { return 'world'; }");
/// let root_node = ast.root();
///
/// // Check the node type
/// println!("Root kind: {}", root_node.kind());
///
/// // Navigate to children
/// for child in root_node.children() {
///     println!("Child: {} at {}:{}", child.kind(),
///         child.start_pos().line(), child.start_pos().column(&child));
/// }
///
/// // Find specific patterns
/// if let Some(return_stmt) = root_node.find("return $VALUE") {
///     let value = return_stmt.get_env().get_match("VALUE").unwrap();
///     println!("Returns: {}", value.text());
/// }
/// ```
#[derive(Clone, Debug)]
pub struct Node<'r, D: Doc> {
    pub(crate) inner: D::Node<'r>,
    pub(crate) root: &'r Root<D>,
}

/// Identifier for different AST node types (e.g., "function_declaration", "identifier")
pub type KindId = u16;

/// APIs for Node inspection
impl<'r, D: Doc> Node<'r, D> {
    pub const fn get_doc(&self) -> &'r D {
        &self.root.doc
    }
    pub fn node_id(&self) -> usize {
        self.inner.node_id()
    }
    pub fn is_leaf(&self) -> bool {
        self.inner.is_leaf()
    }
    /// if has no named children.
    /// N.B. it is different from `is_named` && `is_leaf`
    // see https://github.com/ast-grep/ast-grep/issues/276
    pub fn is_named_leaf(&self) -> bool {
        self.inner.is_named_leaf()
    }
    pub fn is_error(&self) -> bool {
        self.inner.is_error()
    }
    pub fn kind(&self) -> Cow<'_, str> {
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

    pub const fn root(&self) -> &'r Root<D> {
        self.root
    }
}

/**
 * Corresponds to inside/has/precedes/follows
 */
#[cfg(feature = "matching")]
impl<D: Doc> Node<'_, D> {
    pub fn matches<M: Matcher>(&self, m: M) -> bool {
        m.match_node(self.clone()).is_some()
    }

    pub fn inside<M: Matcher>(&self, m: M) -> bool {
        self.ancestors().find_map(|n| m.match_node(n)).is_some()
    }

    pub fn has<M: Matcher>(&self, m: M) -> bool {
        self.dfs().skip(1).find_map(|n| m.match_node(n)).is_some()
    }

    pub fn precedes<M: Matcher>(&self, m: M) -> bool {
        self.next_all().find_map(|n| m.match_node(n)).is_some()
    }

    pub fn follows<M: Matcher>(&self, m: M) -> bool {
        self.prev_all().find_map(|n| m.match_node(n)).is_some()
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
    pub fn prev(&self) -> Option<Self> {
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

    #[cfg(feature = "matching")]
    pub fn find<M: Matcher>(&self, pat: M) -> Option<NodeMatch<'r, D>> {
        pat.find_node(self.clone())
    }
    #[cfg(feature = "matching")]
    pub fn find_all<'s, M: Matcher + 's>(
        &'s self,
        pat: M,
    ) -> impl Iterator<Item = NodeMatch<'r, D>> + 's {
        let kinds = pat.potential_kinds();
        self.dfs().filter_map(move |cand| {
            if let Some(k) = &kinds {
                if !k.contains(cand.kind_id().into()) {
                    return None;
                }
            }
            pat.match_node(cand)
        })
    }
}

/// Tree manipulation API
impl<D: Doc> Node<'_, D> {
    #[cfg(feature = "matching")]
    pub fn replace<M: Matcher, R: Replacer<D>>(&self, matcher: M, replacer: R) -> Option<Edit<D>> {
        let matched = matcher.find_node(self.clone())?;
        let edit = matched.make_edit(&matcher, &replacer);
        Some(edit)
    }

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

#[cfg(test)]
mod test {
    use crate::language::{Language, Tsx};
    use crate::tree_sitter::LanguageExt;
    #[test]
    fn test_is_leaf() {
        let root = Tsx.ast_grep("let a = 123");
        let node = root.root();
        assert!(!node.is_leaf());
    }

    #[test]
    fn test_children() {
        let root = Tsx.ast_grep("let a = 123");
        let node = root.root();
        let children: Vec<_> = node.children().collect();
        assert_eq!(children.len(), 1);
        let texts: Vec<_> = children[0]
            .children()
            .map(|c| c.text().to_string())
            .collect();
        assert_eq!(texts, vec!["let", "a = 123"]);
    }
    #[test]
    fn test_empty() {
        let root = Tsx.ast_grep("let a = 123");
        let node = root.root();
        let edit = node.empty().unwrap();
        assert_eq!(edit.inserted_text.len(), 0);
        assert_eq!(edit.deleted_length, 11);
        assert_eq!(edit.position, 0);
    }

    #[test]
    fn test_field_children() {
        let root = Tsx.ast_grep("let a = 123");
        let node = root.root().find("let a = $A").unwrap();
        let children: Vec<_> = node.field_children("kind").collect();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].text(), "let");
    }

    const MULTI_LINE: &str = "
if (a) {
  test(1)
} else {
  x
}
";

    #[test]
    fn test_display_context() {
        // src, matcher, lead, trail
        let cases = [
            ["i()", "i()", "", ""],
            ["i()", "i", "", "()"],
            [MULTI_LINE, "test", "  ", "(1)"],
        ];
        // display context should not panic
        for [src, matcher, lead, trail] in cases {
            let root = Tsx.ast_grep(src);
            let node = root.root().find(matcher).expect("should match");
            let display = node.display_context(0, 0);
            assert_eq!(display.leading, lead);
            assert_eq!(display.trailing, trail);
        }
    }

    #[test]
    fn test_multi_line_context() {
        let cases = [
            ["i()", "i()", "", ""],
            [MULTI_LINE, "test", "if (a) {\n  ", "(1)\n} else {"],
        ];
        // display context should not panic
        for [src, matcher, lead, trail] in cases {
            let root = Tsx.ast_grep(src);
            let node = root.root().find(matcher).expect("should match");
            let display = node.display_context(1, 1);
            assert_eq!(display.leading, lead);
            assert_eq!(display.trailing, trail);
        }
    }

    #[test]
    fn test_replace_all_nested() {
        let root = Tsx.ast_grep("Some(Some(1))");
        let node = root.root();
        let edits = node.replace_all("Some($A)", "$A");
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].inserted_text, "Some(1)".as_bytes());
    }

    #[test]
    fn test_replace_all_multiple_sorted() {
        let root = Tsx.ast_grep("Some(Some(1)); Some(2)");
        let node = root.root();
        let edits = node.replace_all("Some($A)", "$A");
        // edits must be sorted by position
        assert_eq!(edits.len(), 2);
        assert_eq!(edits[0].inserted_text, "Some(1)".as_bytes());
        assert_eq!(edits[1].inserted_text, "2".as_bytes());
    }

    #[test]
    fn test_inside() {
        let root = Tsx.ast_grep("Some(Some(1)); Some(2)");
        let root = root.root();
        let node = root.find("Some(1)").expect("should exist");
        assert!(node.inside("Some($A)"));
    }
    #[test]
    fn test_has() {
        let root = Tsx.ast_grep("Some(Some(1)); Some(2)");
        let root = root.root();
        let node = root.find("Some($A)").expect("should exist");
        assert!(node.has("Some(1)"));
    }
    #[test]
    fn precedes() {
        let root = Tsx.ast_grep("Some(Some(1)); Some(2);");
        let root = root.root();
        let node = root.find("Some($A);").expect("should exist");
        assert!(node.precedes("Some(2);"));
    }
    #[test]
    fn follows() {
        let root = Tsx.ast_grep("Some(Some(1)); Some(2);");
        let root = root.root();
        let node = root.find("Some(2);").expect("should exist");
        assert!(node.follows("Some(Some(1));"));
    }

    #[test]
    fn test_field() {
        let root = Tsx.ast_grep("class A{}");
        let root = root.root();
        let node = root.find("class $C {}").expect("should exist");
        assert!(node.field("name").is_some());
        assert!(node.field("none").is_none());
    }
    #[test]
    fn test_child_by_field_id() {
        let root = Tsx.ast_grep("class A{}");
        let root = root.root();
        let node = root.find("class $C {}").expect("should exist");
        let id = Tsx.field_to_id("name").unwrap();
        assert!(node.child_by_field_id(id).is_some());
        assert!(node.child_by_field_id(id + 1).is_none());
    }

    #[test]
    fn test_remove() {
        let root = Tsx.ast_grep("Some(Some(1)); Some(2);");
        let root = root.root();
        let node = root.find("Some(2);").expect("should exist");
        let edit = node.remove();
        assert_eq!(edit.position, 15);
        assert_eq!(edit.deleted_length, 8);
    }

    #[test]
    fn test_ascii_pos() {
        let root = Tsx.ast_grep("a");
        let root = root.root();
        let node = root.find("$A").expect("should exist");
        assert_eq!(node.start_pos().line(), 0);
        assert_eq!(node.start_pos().column(&*node), 0);
        assert_eq!(node.end_pos().line(), 0);
        assert_eq!(node.end_pos().column(&*node), 1);
    }

    #[test]
    fn test_unicode_pos() {
        let root = Tsx.ast_grep("🦀");
        let root = root.root();
        let node = root.find("$A").expect("should exist");
        assert_eq!(node.start_pos().line(), 0);
        assert_eq!(node.start_pos().column(&*node), 0);
        assert_eq!(node.end_pos().line(), 0);
        assert_eq!(node.end_pos().column(&*node), 1);
        let root = Tsx.ast_grep("\n  🦀🦀");
        let root = root.root();
        let node = root.find("$A").expect("should exist");
        assert_eq!(node.start_pos().line(), 1);
        assert_eq!(node.start_pos().column(&*node), 2);
        assert_eq!(node.end_pos().line(), 1);
        assert_eq!(node.end_pos().column(&*node), 4);
    }
}
