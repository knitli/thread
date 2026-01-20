// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

//! # AST Node Kind Matching
//!
//! Provides matchers that filter AST nodes based on their syntactic type (kind).
//! Every AST node has a "kind" that describes what syntax element it represents
//! (e.g., "`function_declaration`", "identifier", "`string_literal`").
//!
//! ## Core Types
//!
//! - [`KindMatcher`] - Matches nodes of a specific syntactic type
//! - [`KindMatcherError`] - Errors when creating matchers with invalid kinds
//! - [`kind_utils`] - Utilities for working with node kinds
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use thread_ast_engine::matchers::KindMatcher;
//! use thread_ast_engine::matcher::MatcherExt;
//!
//! // Match all function declarations
//! let matcher = KindMatcher::new("function_declaration", &language);
//! let functions: Vec<_> = root.find_all(&matcher).collect();
//!
//! // Match parsing errors in source code
//! let error_matcher = KindMatcher::error_matcher();
//! let errors: Vec<_> = root.find_all(&error_matcher).collect();
//! ```
//!
//! ## Node Kind Concepts
//!
//! - **Named nodes** - Represent actual language constructs (functions, variables, etc.)
//! - **Anonymous nodes** - Represent punctuation and keywords (`{`, `}`, `let`, etc.)
//! - **Error nodes** - Represent unparsable syntax (syntax errors)
//!
//! Kind matching is useful for:
//! - Finding all nodes of a specific type (all functions, all classes, etc.)
//! - Detecting syntax errors in source code
//! - Building language-specific analysis tools

use super::matcher::Matcher;

use crate::language::Language;
use crate::meta_var::MetaVarEnv;
use crate::node::KindId;
use crate::{Doc, Node};

use std::borrow::Cow;

use bit_set::BitSet;
use thiserror::Error;

// 0 is symbol_end for not found, 65535 is builtin symbol ERROR
// see https://tree-sitter.docsforge.com/master/api/#TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION
// and https://tree-sitter.docsforge.com/master/api/ts_language_symbol_for_name/
const TS_BUILTIN_SYM_END: KindId = 0;
const TS_BUILTIN_SYM_ERROR: KindId = 65535;

/// Errors that can occur when creating a [`KindMatcher`].
#[derive(Debug, Error)]
pub enum KindMatcherError {
    /// The specified node kind name doesn't exist in the language grammar.
    ///
    /// This happens when you try to match a node type that isn't defined
    /// in the tree-sitter grammar for the language.
    #[error("Kind `{0}` is invalid.")]
    InvalidKindName(String),
}

/// Matcher that finds AST nodes based on their syntactic type (kind).
///
/// `KindMatcher` is the simplest type of matcher - it matches nodes whose
/// type matches a specific string. Every AST node has a "kind" that describes
/// what syntax element it represents.
///
/// # Examples
///
/// ```rust,ignore
/// // Match all function declarations
/// let matcher = KindMatcher::new("function_declaration", &language);
/// let functions: Vec<_> = root.find_all(&matcher).collect();
///
/// // Match all identifiers
/// let id_matcher = KindMatcher::new("identifier", &language);
/// let identifiers: Vec<_> = root.find_all(&id_matcher).collect();
///
/// // Find syntax errors in code
/// let error_matcher = KindMatcher::error_matcher();
/// let errors: Vec<_> = root.find_all(&error_matcher).collect();
/// ```
///
/// # Common Node Kinds
///
/// The exact node kinds depend on the language, but common examples include:
/// - `"function_declaration"` - Function definitions
/// - `"identifier"` - Variable/function names
/// - `"string_literal"` - String values
/// - `"number"` - Numeric literals
/// - `"ERROR"` - Syntax errors
#[derive(Debug, Clone)]
pub struct KindMatcher {
    /// The numeric ID of the node kind to match
    kind: KindId,
}

impl KindMatcher {
    pub fn new<L: Language>(node_kind: &str, lang: &L) -> Self {
        Self {
            kind: lang.kind_to_id(node_kind),
        }
    }

    pub fn try_new<L: Language>(node_kind: &str, lang: &L) -> Result<Self, KindMatcherError> {
        let s = Self::new(node_kind, lang);
        if s.is_invalid() {
            Err(KindMatcherError::InvalidKindName(node_kind.into()))
        } else {
            Ok(s)
        }
    }

    #[must_use]
    pub const fn from_id(kind: KindId) -> Self {
        Self { kind }
    }

    /// Whether the kind matcher contains undefined tree-sitter kind.
    #[must_use]
    pub const fn is_invalid(&self) -> bool {
        self.kind == TS_BUILTIN_SYM_END
    }

    /// Construct a matcher that only matches ERROR
    #[must_use]
    pub const fn error_matcher() -> Self {
        Self::from_id(TS_BUILTIN_SYM_ERROR)
    }
}

pub mod kind_utils {
    use super::{KindId, TS_BUILTIN_SYM_ERROR};

    /// Whether the kind will match parsing error occurred in the source code.
    ///
    /// This is used to match parsing error in the source code.
    /// for example, we can use `kind: ERROR` in YAML to find invalid syntax in source.
    /// the name `is_error` implies the matcher itself is error.
    /// But here the matcher itself is valid and it is what it matches is error.
    #[must_use]
    pub const fn is_error_kind(kind: KindId) -> bool {
        kind == TS_BUILTIN_SYM_ERROR
    }

    #[must_use]
    pub const fn are_kinds_matching(goal: KindId, candidate: KindId) -> bool {
        goal == candidate || is_error_kind(goal)
    }
}

impl Matcher for KindMatcher {
    fn match_node_with_env<'tree, D: Doc>(
        &self,
        node: Node<'tree, D>,
        _env: &mut Cow<MetaVarEnv<'tree, D>>,
    ) -> Option<Node<'tree, D>> {
        if node.kind_id() == self.kind {
            Some(node)
        } else {
            None
        }
    }

    fn potential_kinds(&self) -> Option<BitSet> {
        let mut set = BitSet::new();
        set.insert(self.kind.into());
        Some(set)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::language::Tsx;
    use crate::matcher::MatcherExt;
    use crate::{Root, tree_sitter::StrDoc};

    fn pattern_node(s: &str) -> Root<StrDoc<Tsx>> {
        Root::str(s, Tsx)
    }
    #[test]
    fn test_kind_match() {
        let kind = "public_field_definition";
        let cand = pattern_node("class A { a = 123 }");
        let cand = cand.root();
        let pattern = KindMatcher::new(kind, &Tsx);
        assert!(
            pattern.find_node(cand.clone()).is_some(),
            "goal: {}, candidate: {}",
            kind,
            cand.get_inner_node().to_sexp(),
        );
    }

    #[test]
    fn test_kind_non_match() {
        let kind = "field_definition";
        let cand = pattern_node("const a = 123");
        let cand = cand.root();
        let pattern = KindMatcher::new(kind, &Tsx);
        assert!(
            pattern.find_node(cand.clone()).is_none(),
            "goal: {}, candidate: {}",
            kind,
            cand.get_inner_node().to_sexp(),
        );
    }

    #[test]
    fn test_kind_potential_kinds() {
        let kind = "field_definition";
        let matcher = KindMatcher::new(kind, &Tsx);
        let potential_kinds = matcher
            .potential_kinds()
            .expect("should have potential kinds");
        // should has exactly one potential kind
        assert_eq!(potential_kinds.len(), 1);
    }
}
