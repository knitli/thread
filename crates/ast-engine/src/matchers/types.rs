// SPDX-FileCopyrightText: 2025 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT
#![allow(dead_code, reason = "Some fields report they're dead if the `matching` feature is not enabled.")]
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
use crate::meta_var::{MetaVariable, MetaVarEnv};
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
/// use thread_ast_engine::Matcher;
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
/// # use thread_ast_engine::MatcherExt;
/// let ast = Language::Tsx.ast_grep("const x = 42;");
/// let root = ast.root();
///
/// // Use MatcherExt methods
/// if let Some(node_match) = root.find("const $VAR = $VALUE") {
///     println!("Found constant declaration");
/// }
/// ```
pub trait MatcherExt: Matcher {
    fn match_node<'tree, D: Doc>(&self, node: Node<'tree, D>) -> Option<NodeMatch<'tree, D>>;

    fn find_node<'tree, D: Doc>(&self, node: Node<'tree, D>) -> Option<NodeMatch<'tree, D>>;
}

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
#[cfg_attr(not(feature = "matching"), allow(dead_code))]
pub struct NodeMatch<'t, D: Doc>(pub(crate) Node<'t, D>, pub(crate) MetaVarEnv<'t, D>);


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
