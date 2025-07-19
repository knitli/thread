// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

//! # Pattern Match Results with Meta-Variable Capture
//!
//! Contains the [`NodeMatch`] type that represents the result of a successful
//! pattern match, including both the matched AST node and any captured meta-variables.
//!
//! When a pattern like `"function $NAME($$$PARAMS) { $$$BODY }"` matches an AST node,
//! it creates a [`NodeMatch`] that stores:
//! - The matched node (the function declaration)
//! - The captured variables (`$NAME`, `$PARAMS`, `$BODY`)
//!
//! ## Key Features
//!
//! - **Node access**: Use like a regular [`Node`] through [`Deref`]
//! - **Meta-variable access**: Get captured variables via [`NodeMatch::get_env`]
//! - **Code replacement**: Generate edits with [`NodeMatch::replace_by`]
//! - **Type safety**: Lifetime-bound to ensure memory safety
//!
//! ## Example Usage
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

use super::matcher::Matcher;
use crate::meta_var::MetaVarEnv;
use crate::replacer::Replacer;
use crate::source::Edit as E;
use crate::{Doc, Node};

use std::borrow::Borrow;
use std::ops::Deref;

type Edit<D> = E<<D as Doc>::Source>;

/// Result of a successful pattern match containing the matched node and captured variables.
///
/// `NodeMatch` combines an AST node with the meta-variables captured during
/// pattern matching. It acts like a regular [`Node`] (through [`Deref`]) while
/// also providing access to captured variables through [`get_env`].
///
/// # Lifetime
///
/// The lifetime `'t` ties the match to its source document, ensuring memory safety.
///
/// # Usage Patterns
///
/// ```rust,ignore
/// // Use as a regular node
/// let text = node_match.text();
/// let position = node_match.start_pos();
///
/// // Access captured meta-variables
/// let env = node_match.get_env();
/// let captured_name = env.get_match("VAR_NAME").unwrap();
///
/// // Generate replacement code
/// let edit = node_match.replace_by("new code with $VAR_NAME");
/// ```
///
/// # Type Parameters
///
/// - `'t` - Lifetime tied to the source document
/// - `D: Doc` - Document type containing the source and language info
#[derive(Clone)]
pub struct NodeMatch<'t, D: Doc>(Node<'t, D>, MetaVarEnv<'t, D>);

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
