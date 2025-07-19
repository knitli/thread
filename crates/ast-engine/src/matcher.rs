// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

//! This module defines the core `Matcher` trait in ast-grep.
//!
//! `Matcher` has three notable implementations in this module:
//! * `Pattern`: matches against a tree-sitter node based on its tree structure.
//! * `KindMatcher`: matches a node based on its `kind`
//! * `RegexMatcher`: matches a node based on its textual content using regex.

use crate::Doc;
use crate::{Node, meta_var::MetaVarEnv};

use bit_set::BitSet;
use std::borrow::Cow;

pub use crate::matchers::kind::*;
pub use crate::matchers::matcher::Matcher;
pub use crate::matchers::node_match::*;
pub use crate::matchers::pattern::*;
pub use crate::matchers::text::*;

/// `MatcherExt` provides additional utility methods for `Matcher`.
/// It is implemented for all types that implement `Matcher`.
/// N.B. This trait is not intended to be implemented by users.
pub trait MatcherExt: Matcher {
    fn match_node<'tree, D: Doc>(&self, node: Node<'tree, D>) -> Option<NodeMatch<'tree, D>> {
        // in future we might need to customize initial MetaVarEnv
        let mut env = Cow::Owned(MetaVarEnv::new());
        let node = self.match_node_with_env(node, &mut env)?;
        Some(NodeMatch::new(node, env.into_owned()))
    }

    fn find_node<'tree, D: Doc>(&self, node: Node<'tree, D>) -> Option<NodeMatch<'tree, D>> {
        for n in node.dfs() {
            if let Some(ret) = self.match_node(n.clone()) {
                return Some(ret);
            }
        }
        None
    }
}

impl<T> MatcherExt for T where T: Matcher {}

impl Matcher for str {
    fn match_node_with_env<'tree, D: Doc>(
        &self,
        node: Node<'tree, D>,
        env: &mut Cow<MetaVarEnv<'tree, D>>,
    ) -> Option<Node<'tree, D>> {
        let pattern = Pattern::new(self, node.lang());
        pattern.match_node_with_env(node, env)
    }

    fn get_match_len<D: Doc>(&self, node: Node<'_, D>) -> Option<usize> {
        let pattern = Pattern::new(self, node.lang());
        pattern.get_match_len(node)
    }
}

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

pub struct MatchAll;
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

pub struct MatchNone;
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
