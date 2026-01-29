// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # Incremental Update System
//!
//! This module implements Thread's incremental update system for dependency-aware
//! invalidation and targeted re-analysis. It adapts patterns from ReCoco's
//! `FieldDefFingerprint` design to Thread's AST analysis domain.
//!
//! ## Architecture
//!
//! The system consists of three integrated subsystems:
//!
//! - **Types** ([`types`]): Core data structures for fingerprints, dependency edges,
//!   and the dependency graph.
//! - **Graph** ([`graph`]): Dependency graph traversal algorithms including BFS
//!   affected-file detection, topological sort, and cycle detection.
//! - **Storage** ([`storage`]): Trait definitions for persisting dependency graphs
//!   and fingerprints across sessions (Postgres, D1).
//!
//! ## Design Pattern
//!
//! Adapted from ReCoco's `FieldDefFingerprint` (analyzer.rs:69-84):
//! - **Source tracking**: Identifies which files contribute to each analysis result
//! - **Fingerprint composition**: Detects content AND logic changes via Blake3 hashing
//! - **Dependency graph**: Maintains import/export relationships for cascading invalidation
//!
//! ## Example
//!
//! ```rust
//! use thread_flow::incremental::types::{
//!     AnalysisDefFingerprint, DependencyEdge, DependencyType,
//! };
//! use thread_flow::incremental::graph::DependencyGraph;
//! use std::path::PathBuf;
//! use std::collections::HashSet;
//!
//! // Create a dependency graph
//! let mut graph = DependencyGraph::new();
//!
//! // Add a dependency edge: main.rs imports utils.rs
//! graph.add_edge(DependencyEdge {
//!     from: PathBuf::from("src/main.rs"),
//!     to: PathBuf::from("src/utils.rs"),
//!     dep_type: DependencyType::Import,
//!     symbol: None,
//! });
//!
//! // Find files affected by a change to utils.rs
//! let changed = HashSet::from([PathBuf::from("src/utils.rs")]);
//! let affected = graph.find_affected_files(&changed);
//! assert!(affected.contains(&PathBuf::from("src/main.rs")));
//! ```

pub mod graph;
pub mod storage;
pub mod types;

// Re-export core types for ergonomic use
pub use graph::DependencyGraph;
pub use types::{
    AnalysisDefFingerprint, DependencyEdge, DependencyStrength, DependencyType, SymbolDependency,
    SymbolKind,
};
