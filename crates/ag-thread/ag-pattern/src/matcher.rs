//! This module defines the core `Matcher` trait in ast-grep.
//!
//! `Matcher` has three notable implementations in this module:
//! * Pattern: matches against a tree-sitter node based on its tree structure.
//! * KindMatcher: matches a node based on its `kind`
//! * RegexMatcher: matches a node based on its textual content using regex.

mod kind;
mod node_match;
mod pattern;
mod text;

use ag_service_types::{Doc, Node, MatchAll, MatchNone, Matcher, MatcherExt};
use crate::meta_var::MetaVarEnv;

use bit_set::BitSet;
use std::borrow::Cow;

use pattern::Pattern;

impl<T> Matcher for &T
where
  T: Matcher + ?Sized,
{
  fn match_node_with_env<'tree, D: Doc>(
    &self,
    node: Node<'tree, D>,
    env: &mut Cow<MetaVarEnv<'tree, D>>,
  ) -> Option<Node<'tree, D>> {
    (**self).match_node_with_env(node, env)
  }

  fn potential_kinds(&self) -> Option<BitSet> {
    (**self).potential_kinds()
  }

  fn get_match_len<D: Doc>(&self, node: Node<'_, D>) -> Option<usize> {
    (**self).get_match_len(node)
  }
}

impl<T> MatcherExt for T where T: Matcher {}

impl Matcher for str {
  fn match_node_with_env<'tree, D: Doc>(
    &self,
    node: Node<'tree, D>,
    env: &mut Cow<MetaVarEnv<'tree, D>>,
  ) -> Option<Node<'tree, D>> {
    let pattern = Pattern::new(self, node.lang().clone());
    pattern.match_node_with_env(node, env)
  }

  fn get_match_len<D: Doc>(&self, node: Node<'_, D>) -> Option<usize> {
    let pattern = Pattern::new(self, node.lang().clone());
    pattern.get_match_len(node)
  }
}


impl Matcher for MatchAll {
  fn match_node_with_env<'tree, D: Doc>(
    &self,
    node: Node<'tree, D>,
    _env: &mut Cow<MetaVarEnv<'tree, D>>,
  ) -> Option<Node<'tree, D>> {
    Some(node)
  }

  fn potential_kinds(&self) -> Option<BitSet> {
    // return None to match anything
    None
  }
}

impl Matcher for MatchNone {
  fn match_node_with_env<'tree, D: Doc>(
    &self,
    _node: Node<'tree, D>,
    _env: &mut Cow<MetaVarEnv<'tree, D>>,
  ) -> Option<Node<'tree, D>> {
    None
  }

  fn potential_kinds(&self) -> Option<BitSet> {
    // matches nothing
    Some(BitSet::new())
  }
}

pub mod matchers {
  pub use kind::{kind_utils, KindMatcher, KindMatcherError};
  pub use node_match::NodeMatch;
  pub use pattern::{Pattern, PatternBuilder, PatternError, PatternNode};
  pub use super::{Matcher, MatcherExt, MatchAll, MatchNone, Pattern};
  pub use text::{RegexMatcher, RegexMatcherError};
}
