// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

//! # Pattern Matching Engine
//!
//! Core pattern matching functionality for finding and matching AST nodes.
//!
//! ## Key Traits and Types
//!
//! - [`Matcher`] - Core trait for matching AST nodes against patterns
//! - [`MatcherExt`] - Extension trait providing utility methods like [`MatcherExt::find_node`]
//! - [`Pattern`] - Matches nodes based on AST structure with meta-variables
//! - [`NodeMatch`] - Result of a successful pattern match, containing the matched node and captured variables
//!
//! ## Pattern Types
//!
//! The engine supports several types of matchers:
//!
//! - **`Pattern`** - Structural matching based on AST shape (most common)
//! - **`KindMatcher`** - Simple matching based on node type/kind
//! - **`RegexMatcher`** - Text-based matching using regular expressions
//! - **`MatchAll`** / **`MatchNone`** - Utility matchers for always/never matching
//!
//! ## Examples
//!
//! ### Basic Pattern Matching
//!
//! ```rust,no_run
//! # use thread_ast_engine::Language;
//! # use thread_ast_engine::tree_sitter::LanguageExt;
//! # use thread_ast_engine::matcher::MatcherExt;
//! let ast = Language::Tsx.ast_grep("let x = 42;");
//! let root = ast.root();
//!
//! // Find variable declarations
//! if let Some(decl) = root.find("let $VAR = $VALUE") {
//!     let var_name = decl.get_env().get_match("VAR").unwrap();
//!     let value = decl.get_env().get_match("VALUE").unwrap();
//!     println!("Variable {} = {}", var_name.text(), value.text());
//! }
//! ```
//!
//! ### Finding Multiple Matches
//!
//! ```rust,no_run
//! # use thread_ast_engine::Language;
//! # use thread_ast_engine::tree_sitter::LanguageExt;
//! # use thread_ast_engine::matcher::MatcherExt;
//! let code = "let a = 1; let b = 2; let c = 3;";
//! let ast = Language::Tsx.ast_grep(code);
//! let root = ast.root();
//!
//! // Find all variable declarations
//! for decl in root.find_all("let $VAR = $VALUE") {
//!     let var_name = decl.get_env().get_match("VAR").unwrap();
//!     println!("Found variable: {}", var_name.text());
//! }
//! ```

use crate::Doc;
use crate::{Node, meta_var::MetaVarEnv};

use bit_set::BitSet;
use std::borrow::Cow;

pub use crate::matchers::kind::*;
pub use crate::matchers::matcher::Matcher;
pub use crate::matchers::node_match::*;
pub use crate::matchers::pattern::*;
pub use crate::matchers::text::*;

/// Extension trait providing convenient utility methods for [`Matcher`] implementations.
///
/// Automatically implemented for all types that implement [`Matcher`]. Provides
/// higher-level operations like finding nodes and working with meta-variable environments.
///
/// # Important
///
/// You should not implement this trait manually - it's automatically implemented
/// for all [`Matcher`] types.
///
/// # Example
///
/// ```rust,no_run
/// # use thread_ast_engine::Language;
/// # use thread_ast_engine::tree_sitter::LanguageExt;
/// # use thread_ast_engine::matcher::MatcherExt;
/// let ast = Language::Tsx.ast_grep("const x = 42;");
/// let root = ast.root();
///
/// // Use MatcherExt methods
/// if let Some(node_match) = root.find("const $VAR = $VALUE") {
///     println!("Found constant declaration");
/// }
/// ```
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
