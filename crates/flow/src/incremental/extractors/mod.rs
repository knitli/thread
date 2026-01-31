// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Dependency extractors for various programming languages.
//!
//! Each extractor uses tree-sitter queries to parse import/dependency statements
//! from source files and produce [`DependencyEdge`](super::DependencyEdge) values
//! for the incremental update system.
//!
//! ## Supported Languages
//!
//! - **Go** ([`go`]): Extracts `import` statements including blocks, aliases, dot,
//!   and blank imports with go.mod module path resolution.
//! - **Python** ([`python`]): Extracts `import` and `from...import` statements (pending implementation).
//! - **Rust** ([`rust`]): Extracts `use` declarations and `pub use` re-exports with
//!   crate/super/self path resolution and visibility tracking.
//! - **TypeScript/JavaScript** ([`typescript`]): Extracts ES6 imports, CommonJS requires,
//!   and export declarations with node_modules resolution.

pub mod go;
pub mod python;
pub mod rust;
pub mod typescript;

// Re-export extractors for dependency_builder
pub use go::GoDependencyExtractor;
pub use python::PythonDependencyExtractor;
pub use rust::RustDependencyExtractor;
pub use typescript::TypeScriptDependencyExtractor;

// Re-export language detector
pub use super::dependency_builder::LanguageDetector;
