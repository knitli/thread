// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::redundant_pub_crate)]
//! # Pattern Matching Module Organization
//!
//! Conditional module imports for pattern matching functionality with feature flag support.
//!
//! ## Module Structure
//!
//! This module organizes pattern matching components with conditional compilation:
//! - **Core types** (always available) - Pattern definitions and interfaces
//! - **Implementations** (feature-gated) - Actual matching logic and algorithms
//!
//! ## Feature Flag Design
//!
//! The `matching` feature flag controls access to pattern matching implementations:
//! - **With `matching`** - Full pattern matching capabilities available
//! - **Without `matching`** - Only type definitions for API compatibility
//!
//! ## Available Components
//!
//! ### Always Available
//! - [`types`] - Core pattern matching types and traits
//!        - exported here if `matching` feature is not enabled
//!        - exported in `matcher.rs` if `matching` feature is enabled
//!        - Types **always** available from lib.rs:
//!             ```rust,ignore
//!            use thread_ast_engine::{
//!              Matcher, MatcherExt, Pattern, MatchStrictness,
//!              NodeMatch, PatternNode, PatternBuilder, PatternError,
//!              };
//!             ```
//! - [`Matcher`] trait - Interface for all pattern matchers
//!
//! ### Feature-Gated (`matching` feature)
//! - [`pattern`] - Structural pattern matching from source code strings
//! - [`kind`] - AST node type matching
//! - [`text`] - Regex-based text content matching
//!
//! ## Architecture Benefits
//!
//! - **Reduced compilation** - Skip complex matching logic when not needed
//! - **API stability** - Type definitions remain available for library interfaces
//! - **Modular usage** - Enable only required pattern matching features
//!
//! ## Usage
//!
//! ```toml
//! # In Cargo.toml - enable full pattern matching
//! thread-ast-engine = { version = "...", features = ["matching"] }
//! ```
//!
//! ```rust,ignore
//! // Types always available
//! use thread_ast_engine::matchers::types::{Matcher, Pattern};
//!
//! // Implementations require 'matching' feature
//! #[cfg(feature = "matching")]
//! use thread_ast_engine::matchers::pattern::Pattern;
//! ```

#[cfg(feature = "matching")]
pub(crate) mod pattern;

#[cfg(feature = "matching")]
pub(crate) mod kind;

#[cfg(feature = "matching")]
pub(crate) mod text;

pub(crate) mod types;
#[cfg(not(feature = "matching"))]
pub use types::{
    MatchStrictness, Pattern, PatternBuilder, PatternError, PatternNode
};

pub(crate) mod matcher {
    pub use super::types::{Matcher, MatcherExt, NodeMatch};
}
