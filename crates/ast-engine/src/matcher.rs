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
//!
//! ### NodeMatch
//!
//! #### Pattern Match Results with Meta-Variable Capture
//!
//! Contains the implementation for the [`NodeMatch`] type that represents
//! the result of a successful pattern match, including both the matched AST node and
//! any captured meta-variables.
//!
//! When a pattern like `"function $NAME($$$PARAMS) { $$$BODY }"` matches an AST node,
//! it creates a [`NodeMatch`] that stores:
//! - The matched node (the function declaration)
//! - The captured variables (`$NAME`, `$PARAMS`, `$BODY`)
//!
//! #### Key Features
//!
//! - **Node access**: Use like a regular [`Node`] through [`Deref`]
//! - **Meta-variable access**: Get captured variables via [`NodeMatch::get_env`]
//! - **Code replacement**: Generate edits with [`NodeMatch::replace_by`]
//! - **Type safety**: Lifetime-bound to ensure memory safety
//!
//! #### Example Usage
//!
//! ```rust,ignore
//! // Find function declarations
//! let matches = root.find_all("function $NAME($$$PARAMS) { $$$BODY }");
//!
//! for match_ in matches {
//!     // Use as a regular node
//!     println!("Function at line {}", match_.start_pos().line());
//!
//!     // Access captured meta-variables
//!     let env = match_.get_env();
//!     let name = env.get_match("NAME").unwrap();
//!     println!("Function name: {}", name.text());
//!
//!     // Generate replacement code
//!     let edit = match_.replace_by("async function $NAME($$$PARAMS) { $$$BODY }");
//! }
//! ```

use crate::{Doc, Node, meta_var::MetaVarEnv, source::Edit as E};

pub use crate::matchers::kind::*;
pub use crate::matchers::matcher::{Matcher, MatcherExt, NodeMatch};
pub use crate::matchers::pattern::*;
pub use crate::matchers::text::*;
use bit_set::BitSet;
use std::borrow::{Borrow, Cow};
use std::ops::Deref;

use crate::replacer::Replacer;

type Edit<D> = E<<D as Doc>::Source>;

impl<'tree, D: Doc> NodeMatch<'tree, D> {
    pub const fn new(node: Node<'tree, D>, env: MetaVarEnv<'tree, D>) -> Self {
        Self(node, env)
    }

    pub const fn get_node(&self) -> &Node<'tree, D> {
        &self.0
    }

    /// Returns the populated `MetaVarEnv` for this match.
    pub const fn get_env(&self) -> &MetaVarEnv<'tree, D> {
        &self.1
    }
    pub const fn get_env_mut(&mut self) -> &mut MetaVarEnv<'tree, D> {
        &mut self.1
    }
    /// # Safety
    /// should only called for readopting nodes
    pub(crate) const unsafe fn get_node_mut(&mut self) -> &mut Node<'tree, D> {
        &mut self.0
    }
}

impl<D: Doc> NodeMatch<'_, D> {
    pub fn replace_by<R: Replacer<D>>(&self, replacer: R) -> Edit<D> {
        let range = self.range();
        let position = range.start;
        let deleted_length = range.len();
        let inserted_text = replacer.generate_replacement(self);
        Edit::<D> {
            position,
            deleted_length,
            inserted_text,
        }
    }

    #[doc(hidden)]
    pub fn make_edit<M, R>(&self, matcher: &M, replacer: &R) -> Edit<D>
    where
        M: Matcher,
        R: Replacer<D>,
    {
        let range = replacer.get_replaced_range(self, matcher);
        let inserted_text = replacer.generate_replacement(self);
        Edit::<D> {
            position: range.start,
            deleted_length: range.len(),
            inserted_text,
        }
    }
}

impl<'tree, D: Doc> From<Node<'tree, D>> for NodeMatch<'tree, D> {
    fn from(node: Node<'tree, D>) -> Self {
        Self(node, MetaVarEnv::new())
    }
}

/// `NodeMatch` is an immutable view to Node
impl<'tree, D: Doc> From<NodeMatch<'tree, D>> for Node<'tree, D> {
    fn from(node_match: NodeMatch<'tree, D>) -> Self {
        node_match.0
    }
}

/// `NodeMatch` is an immutable view to Node
impl<'tree, D: Doc> Deref for NodeMatch<'tree, D> {
    type Target = Node<'tree, D>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// `NodeMatch` is an immutable view to Node
impl<'tree, D: Doc> Borrow<Node<'tree, D>> for NodeMatch<'tree, D> {
    fn borrow(&self) -> &Node<'tree, D> {
        &self.0
    }
}

impl<T> MatcherExt for T
where
    T: Matcher,
{
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::language::Tsx;
    use crate::tree_sitter::{LanguageExt, StrDoc};

    fn use_node<L: LanguageExt>(n: &Node<StrDoc<L>>) -> String {
        n.text().to_string()
    }

    fn borrow_node<'a, D, B>(b: B) -> String
    where
        D: Doc + 'static,
        B: Borrow<Node<'a, D>>,
    {
        b.borrow().text().to_string()
    }

    #[test]
    fn test_node_match_as_node() {
        let root = Tsx.ast_grep("var a = 1");
        let node = root.root();
        let src = node.text().to_string();
        let nm = NodeMatch::from(node);
        let ret = use_node(&*nm);
        assert_eq!(ret, src);
        assert_eq!(use_node(&*nm), borrow_node(nm));
    }

    #[test]
    fn test_node_env() {
        let root = Tsx.ast_grep("var a = 1");
        let find = root.root().find("var $A = 1").expect("should find");
        let env = find.get_env();
        let node = env.get_match("A").expect("should find");
        assert_eq!(node.text(), "a");
    }

    #[test]
    fn test_replace_by() {
        let root = Tsx.ast_grep("var a = 1");
        let find = root.root().find("var $A = 1").expect("should find");
        let fixed = find.replace_by("var b = $A");
        assert_eq!(fixed.position, 0);
        assert_eq!(fixed.deleted_length, 9);
        assert_eq!(fixed.inserted_text, "var b = a".as_bytes());
    }
}
