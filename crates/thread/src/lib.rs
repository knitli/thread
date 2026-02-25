// SPDX-FileCopyrightText: 2026 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # Thread - Safe, Fast, Flexible Code Analysis and Parsing
//!
//! **Thread** is a high-performance ecosystem for code analysis and transformation built in Rust.
//! It combines the power of tree-sitter for robust parsing with a high-level rule engine
//! and content-addressed caching for efficient codebase-wide analysis.
//!
//! This crate serves as the primary entry point for the Thread ecosystem, re-exporting
//! core components from specialized sub-crates.
//!
//! ## Core Architecture
//!
//! Thread is built on a modular "service-library dual architecture":
//!
//! 1. **Library Ecosystem** - Reusable components for AST pattern matching and transformation.
//! 2. **Service Platform** - Persistent analysis with incremental intelligence and caching.
//!
//! ## Key Modules
//!
//! - [`ast`] - Core AST parsing, matching, and transformation engine.
//! - [`language`] - Support for various programming languages via tree-sitter.
//! - [`rule`] - Rule-based scanning and transformation system.
//! - [`services`] - High-level service interfaces and abstractions.
//! - [`flow`] - Dataflow orchestration and incremental analysis (optional).
//! - [`utils`] - Common utilities and performance optimizations.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use thread::language::{Tsx, LanguageExt};
//!
//! // Parse code and find patterns
//! let ast = Tsx.ast_grep("function hello() { console.log('world'); }");
//! let root = ast.root();
//! let matches = root.find_all("console.log($$$ARGS)");
//!
//! for m in matches {
//!     println!("Found console.log with {} arguments", m.get_env().get_multiple_matches("ARGS").len());
//! }
//! ```

/// Core AST engine for parsing, matching, and transformation.
#[cfg(feature = "ast")]
pub mod ast {
    pub use thread_ast_engine::*;
}

/// Language definitions and tree-sitter parser integrations.
#[cfg(feature = "language")]
pub mod language {
    pub use thread_language::*;
}

/// Rule-based scanning and transformation system.
#[cfg(feature = "rule")]
pub mod rule {
    pub use thread_rule_engine::*;
}

/// Dataflow orchestration layer for incremental computation and caching.
#[cfg(feature = "flow")]
pub mod flow {
    pub use thread_flow::*;
}

/// High-level service interfaces and application abstractions.
#[cfg(feature = "services")]
pub mod services {
    pub use thread_services::*;
}

/// Shared utilities and performance-critical primitives.
#[cfg(feature = "utils")]
pub mod utils {
    pub use thread_utils::*;
}

// Re-export common types at the top level for better ergonomics
#[cfg(feature = "ast")]
pub use thread_ast_engine::{AstGrep, Language, Node, Root};

#[cfg(feature = "language")]
pub use thread_language::SupportLang;

#[cfg(feature = "services")]
pub use thread_services::{CodeAnalyzer, CodeParser, ParsedDocument, ServiceError, ServiceResult};
