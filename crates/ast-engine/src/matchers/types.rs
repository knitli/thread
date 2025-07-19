// SPDX-FileCopyrightText: 2025 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

//! # Core Pattern Matching Types
//!
//! Fundamental types and traits for AST pattern matching operations.
//!
//! ## Key Types
//!
//! - [`Matcher`] - Core trait for matching AST nodes
//! - [`Pattern`] - Structural pattern for matching AST shapes
//! - [`MatchStrictness`] - Controls how precisely patterns must match
//! - [`PatternNode`] - Internal representation of pattern structure
//!
//! ## Usage
//!
//! These types are available even without the `matching` feature flag enabled,
//! allowing API definitions that reference them without requiring full
//! implementation dependencies.

use crate::Doc;
use crate::MetaVarEnv;
use crate::meta_var::MetaVariable;
use crate::node::Node;
use bit_set::BitSet;
use std::borrow::Cow;
use thiserror::Error;

/// Core trait for matching AST nodes against patterns.
///
/// Implementors define how to match nodes, whether by structure, content,
/// kind, or other criteria. The matcher can also capture meta-variables
/// during the matching process.
///
/// # Type Parameters
///
/// The trait is generic over document types to support different source
/// encodings and language implementations.
///
/// # Example Implementation
///
/// ```rust,ignore
/// use thread_ast_engine::matchers::types::Matcher;
///
/// struct SimpleKindMatcher {
///     target_kind: String,
/// }
///
/// impl Matcher for SimpleKindMatcher {
///     fn match_node_with_env<'tree, D: Doc>(
///         &self,
///         node: Node<'tree, D>,
///         _env: &mut Cow<MetaVarEnv<'tree, D>>,
///     ) -> Option<Node<'tree, D>> {
///         if node.kind() == self.target_kind {
///             Some(node)
///         } else {
///             None
///         }
///     }
/// }
/// ```
pub trait Matcher {
    /// Attempt to match a node, updating the meta-variable environment.
    ///
    /// Returns the matched node if successful, or `None` if the node doesn't match.
    /// The returned node is usually the input node, but can be different for
    /// matchers like `Has` that match based on descendants.
    ///
    /// # Parameters
    ///
    /// - `node` - The AST node to test for matching
    /// - `env` - Meta-variable environment to capture variables during matching
    ///
    /// # Returns
    ///
    /// The matched node if successful, otherwise `None`
    fn match_node_with_env<'tree, D: Doc>(
        &self,
        _node: Node<'tree, D>,
        _env: &mut Cow<MetaVarEnv<'tree, D>>,
    ) -> Option<Node<'tree, D>>;

    /// Provide a hint about which node types this matcher can match.
    ///
    /// Returns a bitset of node kind IDs that this matcher might match,
    /// or `None` if it needs to test all node types. Used for optimization
    /// to avoid testing matchers against incompatible nodes.
    ///
    /// # Returns
    ///
    /// - `Some(BitSet)` - Specific node kinds this matcher can match
    /// - `None` - This matcher needs to test all node types
    fn potential_kinds(&self) -> Option<BitSet> {
        None
    }

    /// Determine how much of a matched node should be replaced.
    ///
    /// Used during replacement to determine the exact span of text to replace.
    /// Typically skips trailing punctuation or anonymous nodes.
    ///
    /// # Parameters
    ///
    /// - `node` - The matched node
    ///
    /// # Returns
    ///
    /// Number of bytes from the node's start position to replace,
    /// or `None` to replace the entire node.
    fn get_match_len<D: Doc>(&self, _node: Node<'_, D>) -> Option<usize> {
        None
    }
}

/// Controls how precisely patterns must match AST structure.
///
/// Different strictness levels allow patterns to match with varying degrees
/// of precision, from exact CST matching to loose structural matching.
///
/// # Variants
///
/// - **`Cst`** - All nodes must match exactly (concrete syntax tree)
/// - **`Smart`** - Matches meaningful nodes, ignoring trivial syntax
/// - **`Ast`** - Only structural nodes matter (abstract syntax tree)
/// - **`Relaxed`** - Ignores comments and focuses on code structure
/// - **`Signature`** - Matches structure only, ignoring all text content
///
/// # Example
///
/// ```rust,ignore
/// // With Cst strictness, these would be different:
/// // "let x=42;" vs "let x = 42;"
/// //
/// // With Ast strictness, they match the same pattern:
/// // "let $VAR = $VALUE"
/// ```
#[derive(Clone, Debug)]
pub enum MatchStrictness {
    /// Match all nodes exactly (Concrete Syntax Tree)
    Cst,
    /// Match all nodes except trivial syntax elements
    Smart,
    /// Match only structural AST nodes (Abstract Syntax Tree)
    Ast,
    /// Match AST nodes while ignoring comments
    Relaxed,
    /// Match structure only, ignoring all text content
    Signature,
}

/// Structural pattern for matching AST nodes based on their shape and content.
///
/// Patterns represent code structures with support for meta-variables (like `$VAR`)
/// that can capture parts of the matched code. They're built from source code strings
/// and compiled into efficient matching structures.
///
/// # Example
///
/// ```rust,ignore
/// // Pattern for variable declarations
/// let pattern = Pattern::new("let $NAME = $VALUE", language);
///
/// // Can match: "let x = 42", "let result = calculate()", etc.
/// ```
#[derive(Clone)]
pub struct Pattern {
    /// The root pattern node containing the matching logic
    pub node: PatternNode,
    /// Optional hint about the root node kind for optimization
    pub(crate) root_kind: Option<u16>,
    /// How strictly the pattern should match
    pub strictness: MatchStrictness,
}

/// Builder for constructing patterns from source code.
///
/// Handles parsing pattern strings into [`Pattern`] structures,
/// with optional contextual information for more precise matching.
#[derive(Clone, Debug)]
pub struct PatternBuilder<'a> {
    /// Optional CSS-like selector for contextual matching
    pub(crate) selector: Option<&'a str>,
    /// The pattern source code
    pub(crate) src: Cow<'a, str>,
}

/// Internal representation of a pattern's structure.
///
/// Patterns are compiled into a tree of `PatternNode` elements that
/// efficiently represent the matching logic for different AST structures.
#[derive(Clone)]
pub enum PatternNode {
    /// Meta-variable that captures matched content
    MetaVar {
        /// The meta-variable specification (e.g., `$VAR`, `$$$ITEMS`)
        meta_var: MetaVariable,
    },
    /// Leaf node with specific text content
    Terminal {
        /// Expected text content
        text: String,
        /// Whether this represents a named AST node
        is_named: bool,
        /// Node type identifier
        kind_id: u16,
    },
    /// Internal node with child patterns
    Internal {
        /// Node type identifier
        kind_id: u16,
        /// Child pattern nodes
        children: Vec<PatternNode>,
    },
}

#[derive(Debug, Error)]
pub enum PatternError {
    #[error("Fails to parse the pattern query: `{0}`")]
    Parse(String),
    #[error("No AST root is detected. Please check the pattern source `{0}`.")]
    NoContent(String),
    #[error("Multiple AST nodes are detected. Please check the pattern source `{0}`.")]
    MultipleNode(String),
    #[error(transparent)]
    #[cfg(feature = "matching")]
    InvalidKind(#[from] super::kind::KindMatcherError),
    #[error(
        "Fails to create Contextual pattern: selector `{selector}` matches no node in the context `{context}`."
    )]
    NoSelectorInContext { context: String, selector: String },
}
