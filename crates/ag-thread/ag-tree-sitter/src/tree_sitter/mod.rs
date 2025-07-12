mod traversal;

use ag_service_types::{ContentExt, Edit, SgNode, KindId, Language, Position, TSParseError, LanguageExt}
use ag_service_ast::{Root, Content, Doc, Node, Position, AstGrep};
use ag_service_pattern::{Matcher}
use traversal::Traversal::{Level, Post, Pre, Visit, Visitor, TsPre}
use ag_service_transform::Replacer;

use std::borrow::Cow;
use thread_utils::FastMap;
use std::num::NonZero;
use thiserror::Error;
use ag_service_types::{TSLanguage, TSInputEdit, TSLanguageError, TSTree, TSNode, TSParser, TSPoint, LanguageExt, TSTreeCursor};

#[inline]
fn parse_lang(
  parse_fn: impl Fn(&mut TSParser) -> Option<TSTree>,
  ts_lang: TSLanguage,
) -> Result<TSTree, TSParseError> {
  let mut parser = TSParser::new();
  parser.set_language(&ts_lang)?;
  if let Some(tree) = parse_fn(&mut parser) {
    Ok(tree)
  } else {
    Err(TSParseError::TreeUnavailable)
  }
}

impl<L: LanguageExt> StrDoc<L> {
  pub fn try_new(src: &str, lang: L) -> Result<Self, String> {
    let src = src.to_string();
    let ts_lang = lang.get_ts_language();
    let tree = parse_lang(|p| p.parse(src.as_bytes(), None), ts_lang).map_err(|e| e.to_string())?;
    Ok(Self { src, lang, tree })
  }
  pub fn new(src: &str, lang: L) -> Self {
    Self::try_new(src, lang).expect("TSParser tree error")
  }
  fn parse(&self, old_tree: Option<&TSTree>) -> Result<TSTree, TSParseError> {
    let source = self.get_source();
    let lang = self.get_lang().get_ts_language();
    parse_lang(|p| p.parse(source.as_bytes(), old_tree), lang)
  }
}

impl<L: LanguageExt> Doc for StrDoc<L> {
  type Source = String;
  type Lang = L;
  type TSNode<'r> = TSNode<'r>;
  fn get_lang(&self) -> &Self::Lang {
    &self.lang
  }
  fn get_source(&self) -> &Self::Source {
    &self.src
  }
  fn do_edit(&mut self, edit: &Edit<Self::Source>) -> Result<(), String> {
    let source = &mut self.src;
    perform_edit(&mut self.tree, source, edit);
    self.tree = self.parse(Some(&self.tree)).map_err(|e| e.to_string())?;
    Ok(())
  }
  fn root_node(&self) -> TSNode<'_> {
    self.tree.root_node()
  }
  fn get_node_text<'a>(&'a self, node: &Self::TSNode<'a>) -> Cow<'a, str> {
    Cow::Borrowed(
      node
        .utf8_text(self.src.as_bytes())
        .expect("invalid source text encoding"),
    )
  }
}

struct NodeWalker<'tree> {
  cursor: ag_service_types::TSTreeCursor<'tree>,
  count: usize,
}

impl<'tree> Iterator for NodeWalker<'tree> {
  type Item = TSNode<'tree>;
  fn next(&mut self) -> Option<Self::Item> {
    if self.count == 0 {
      return None;
    }
    let ret = Some(self.cursor.node());
    self.cursor.goto_next_sibling();
    self.count -= 1;
    ret
  }
}

impl ExactSizeIterator for NodeWalker<'_> {
  fn len(&self) -> usize {
    self.count
  }
}

impl<'r> SgNode<'r> for TSNode<'r> {
  fn parent(&self) -> Option<Self> {
    TSNode::parent(self)
  }
  fn ancestors(&self, root: Self) -> impl Iterator<Item = Self> {
    let mut ancestor = Some(root);
    let self_id = self.id();
    std::iter::from_fn(move || {
      let inner = ancestor.take()?;
      if inner.id() == self_id {
        return None;
      }
      ancestor = inner.child_with_descendant(*self);
      Some(inner)
    })
    // We must iterate up the tree to preserve backwards compatibility
    .collect::<Vec<_>>()
    .into_iter()
    .rev()
  }
  fn dfs(&self) -> impl Iterator<Item = Self> {
    TsPre::new(self)
  }
  fn child(&self, nth: usize) -> Option<Self> {
    // TODO remove cast after migrating to tree-sitter
    TSNode::child(self, nth)
  }
  fn children(&self) -> impl ExactSizeIterator<Item = Self> {
    let mut cursor = self.walk();
    cursor.goto_first_child();
    NodeWalker {
      cursor,
      count: self.child_count(),
    }
  }
  fn child_by_field_id(&self, field_id: u16) -> Option<Self> {
    TSNode::child_by_field_id(self, field_id)
  }
  fn next(&self) -> Option<Self> {
    self.next_sibling()
  }
  fn prev(&self) -> Option<Self> {
    self.prev_sibling()
  }
  fn next_all(&self) -> impl Iterator<Item = Self> {
    // if root is none, use self as fallback to return a type-stable Iterator
    let node = self.parent().unwrap_or(*self);
    let mut cursor = node.walk();
    cursor.goto_first_child_for_byte(self.start_byte());
    std::iter::from_fn(move || {
      if cursor.goto_next_sibling() {
        Some(cursor.node())
      } else {
        None
      }
    })
  }
  fn prev_all(&self) -> impl Iterator<Item = Self> {
    // if root is none, use self as fallback to return a type-stable Iterator
    let node = self.parent().unwrap_or(*self);
    let mut cursor = node.walk();
    cursor.goto_first_child_for_byte(self.start_byte());
    std::iter::from_fn(move || {
      if cursor.goto_previous_sibling() {
        Some(cursor.node())
      } else {
        None
      }
    })
  }
  fn is_named(&self) -> bool {
    TSNode::is_named(self)
  }
  /// N.B. it is different from is_named && is_leaf
  /// if a node has no named children.
  fn is_named_leaf(&self) -> bool {
    self.named_child_count() == 0
  }
  fn is_leaf(&self) -> bool {
    self.child_count() == 0
  }
  fn kind(&self) -> Cow<str> {
    Cow::Borrowed(TSNode::kind(self))
  }
  fn kind_id(&self) -> KindId {
    TSNode::kind_id(self)
  }
  fn node_id(&self) -> usize {
    self.id()
  }
  fn range(&self) -> std::ops::Range<usize> {
    self.start_byte()..self.end_byte()
  }
  fn start_pos(&self) -> Position {
    let pos = self.start_position();
    let byte = self.start_byte();
    Position::new(pos.row, pos.column, byte)
  }
  fn end_pos(&self) -> Position {
    let pos = self.end_position();
    let byte = self.end_byte();
    Position::new(pos.row, pos.column, byte)
  }
  // missing node is a tree-sitter specific concept
  fn is_missing(&self) -> bool {
    TSNode::is_missing(self)
  }
  fn is_error(&self) -> bool {
    TSNode::is_error(self)
  }

  fn field(&self, name: &str) -> Option<Self> {
    self.child_by_field_name(name)
  }
  fn field_children(&self, field_id: Option<u16>) -> impl Iterator<Item = Self> {
    let field_id = field_id.and_then(NonZero::new);
    let mut cursor = self.walk();
    cursor.goto_first_child();
    // if field_id is not found, iteration is done
    let mut done = field_id.is_none();

    std::iter::from_fn(move || {
      if done {
        return None;
      }
      while cursor.field_id() != field_id {
        if !cursor.goto_next_sibling() {
          return None;
        }
      }
      let ret = cursor.node();
      if !cursor.goto_next_sibling() {
        done = true;
      }
      Some(ret)
    })
  }
}

pub fn perform_edit<S: ContentExt>(tree: &mut TSTree, input: &mut S, edit: &Edit<S>) -> TSInputEdit {
  let edit = input.accept_edit(edit);
  tree.edit(&edit);
  edit
}

fn position_for_offset(input: &[u8], offset: usize) -> TSPoint {
  debug_assert!(offset <= input.len());
  let (mut row, mut col) = (0, 0);
  for c in &input[0..offset] {
    if *c as char == '\n' {
      row += 1;
      col = 0;
    } else {
      col += 1;
    }
  }
  TSPoint::new(row, col)
}

impl<L: LanguageExt> AstGrep<StrDoc<L>> {
  pub fn new<S: AsRef<str>>(src: S, lang: L) -> Self {
    Root::str(src.as_ref(), lang)
  }

  pub fn source(&self) -> &str {
    self.doc.get_source().as_str()
  }

  pub fn generate(self) -> String {
    self.doc.src
  }
}

impl ContentExt for String {
  fn accept_edit(&mut self, edit: &Edit<Self>) -> TSInputEdit {
    let start_byte = edit.position;
    let old_end_byte = edit.position + edit.deleted_length;
    let new_end_byte = edit.position + edit.inserted_text.len();
    let input = unsafe { self.as_mut_vec() };
    let start_position = position_for_offset(input, start_byte);
    let old_end_position = position_for_offset(input, old_end_byte);
    input.splice(start_byte..old_end_byte, edit.inserted_text.clone());
    let new_end_position = position_for_offset(input, new_end_byte);
    TSInputEdit {
      start_byte,
      old_end_byte,
      new_end_byte,
      start_position,
      old_end_position,
      new_end_position,
    }
  }
}

impl<L: LanguageExt> Root<StrDoc<L>> {
  pub fn str(src: &str, lang: L) -> Self {
    Self::try_new(src, lang).expect("should parse")
  }
  pub fn try_new(src: &str, lang: L) -> Result<Self, String> {
    let doc = StrDoc::try_new(src, lang)?;
    Ok(Self { doc })
  }
  pub fn get_text(&self) -> &str {
    &self.doc.src
  }

  pub fn get_injections<F: Fn(&str) -> Option<L>>(&self, get_lang: F) -> Vec<Self> {
    let root = self.root();
    let range = self.lang().extract_injections(root);
    let roots = range
      .into_iter()
      .filter_map(|(lang, ranges)| {
        let lang = get_lang(&lang)?;
        let source = self.doc.get_source();
        let mut parser = TSParser::new();
        parser.set_included_ranges(&ranges).ok()?;
        parser.set_language(&lang.get_ts_language()).ok()?;
        let tree = parser.parse(source, None)?;
        Some(Self {
          doc: StrDoc {
            src: self.doc.src.clone(),
            lang,
            tree,
          },
        })
      })
      .collect();
    roots
  }
}

/// these methods are only for `StrDoc`
impl<'r, L: LanguageExt> Node<'r, StrDoc<L>> {
  #[doc(hidden)]
  pub fn display_context(&self, before: usize, after: usize) -> DisplayContext<'r> {
    let source = self.root.doc.get_source().as_str();
    let bytes = source.as_bytes();
    let start = self.inner.start_byte();
    let end = self.inner.end_byte();
    let (mut leading, mut trailing) = (start, end);
    let mut lines_before = before + 1;
    while leading > 0 {
      if bytes[leading - 1] == b'\n' {
        lines_before -= 1;
        if lines_before == 0 {
          break;
        }
      }
      leading -= 1;
    }
    let mut lines_after = after + 1;
    // tree-sitter will append line ending to source so trailing can be out of bound
    trailing = trailing.min(bytes.len());
    while trailing < bytes.len() {
      if bytes[trailing] == b'\n' {
        lines_after -= 1;
        if lines_after == 0 {
          break;
        }
      }
      trailing += 1;
    }
    // lines_before means we matched all context, offset is `before` itself
    let offset = if lines_before == 0 {
      before
    } else {
      // otherwise, there are fewer than `before` line in src, compute the actual line
      before + 1 - lines_before
    };
    DisplayContext {
      matched: self.text(),
      leading: &source[leading..start],
      trailing: &source[end..trailing],
      start_line: self.start_pos().line() - offset,
    }
  }

  pub fn replace_all<M: Matcher, R: Replacer<StrDoc<L>>>(
    &self,
    matcher: M,
    replacer: R,
  ) -> Vec<Edit<String>> {
    // TODO: support nested matches like Some(Some(1)) with pattern Some($A)
    Visitor::new(&matcher)
      .reentrant(false)
      .visit(self.clone())
      .map(|matched| matched.make_edit(&matcher, &replacer))
      .collect()
  }
}

pub mod Ts {
  //! This module provides tree-sitter specific APIs for ast-grep.
  //! //! It includes:
  //! //! - Tree-sitter specific implementations
  //! //! - Language extensions for tree-sitter
  //! //! - Document parsing and editing
  //! //! It is used to parse and manipulate source code using tree-sitter.
  //! //! It provides a set of APIs to work with tree-sitter, including parsing, editing, and traversing the syntax tree.
  pub use super::{ContentExt, AstGrep, Level, Post, Pre, Visit, Visitor, TsPre,
    TSLanguage, TSPoint, TSRange, TSTree, TSNode, TSInputEdit, TSLanguageError, TSParseError, TSParser, StrDoc, LanguageExt};
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::language::Tsx;
  use ag_service_types::TSPoint;

  fn parse(src: &str) -> Result<TSTree, TSParseError> {
    parse_lang(|p| p.parse(src, None), Tsx.get_ts_language())
  }

  #[test]
  fn test_tree_sitter() -> Result<(), TSParseError> {
    let tree = parse("var a = 1234")?;
    let root_node = tree.root_node();
    assert_eq!(root_node.kind(), "program");
    assert_eq!(root_node.start_position().column, 0);
    assert_eq!(root_node.end_position().column, 12);
    assert_eq!(
      root_node.to_sexp(),
      "(program (variable_declaration (variable_declarator name: (identifier) value: (number))))"
    );
    Ok(())
  }

  #[test]
  fn test_object_literal() -> Result<(), TSParseError> {
    let tree = parse("{a: $X}")?;
    let root_node = tree.root_node();
    // wow this is not label. technically it is wrong but practically it is better LOL
    assert_eq!(root_node.to_sexp(), "(program (expression_statement (object (pair key: (property_identifier) value: (identifier)))))");
    Ok(())
  }

  #[test]
  fn test_string() -> Result<(), TSParseError> {
    let tree = parse("'$A'")?;
    let root_node = tree.root_node();
    assert_eq!(
      root_node.to_sexp(),
      "(program (expression_statement (string (string_fragment))))"
    );
    Ok(())
  }

  #[test]
  fn test_row_col() -> Result<(), TSParseError> {
    let tree = parse("😄")?;
    let root = tree.root_node();
    assert_eq!(root.start_position(), TSPoint::new(0, 0));
    // NOTE: TSPoint in tree-sitter is counted in bytes instead of char
    assert_eq!(root.end_position(), TSPoint::new(0, 4));
    Ok(())
  }

  #[test]
  fn test_edit() -> Result<(), TSParseError> {
    let mut src = "a + b".to_string();
    let mut tree = parse(&src)?;
    let _ = perform_edit(
      &mut tree,
      &mut src,
      &Edit {
        position: 1,
        deleted_length: 0,
        inserted_text: " * b".into(),
      },
    );
    let tree2 = parse_lang(|p| p.parse(&src, Some(&tree)), Tsx.get_ts_language())?;
    assert_eq!(
      tree.root_node().to_sexp(),
      "(program (expression_statement (binary_expression left: (identifier) right: (identifier))))"
    );
    assert_eq!(tree2.root_node().to_sexp(), "(program (expression_statement (binary_expression left: (binary_expression left: (identifier) right: (identifier)) right: (identifier))))");
    Ok(())
  }
}
