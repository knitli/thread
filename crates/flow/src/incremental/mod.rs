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
//! The system consists of four integrated subsystems:
//!
//! - **Types** ([`types`]): Core data structures for fingerprints, dependency edges,
//!   and the dependency graph.
//! - **Graph** ([`graph`]): Dependency graph traversal algorithms including BFS
//!   affected-file detection, topological sort, and cycle detection.
//! - **Storage** ([`storage`]): Trait definitions for persisting dependency graphs
//!   and fingerprints across sessions.
//! - **Backends** ([`backends`]): Concrete storage implementations (Postgres, D1, InMemory)
//!   with factory pattern for runtime backend selection.
//!
//! ## Design Pattern
//!
//! Adapted from ReCoco's `FieldDefFingerprint` (analyzer.rs:69-84):
//! - **Source tracking**: Identifies which files contribute to each analysis result
//! - **Fingerprint composition**: Detects content AND logic changes via Blake3 hashing
//! - **Dependency graph**: Maintains import/export relationships for cascading invalidation
//!
//! ## Examples
//!
//! ### Basic Dependency Graph Operations
//!
//! ```rust
//! use thread_flow::incremental::types::{
//!     AnalysisDefFingerprint, DependencyEdge, DependencyType,
//! };
//! use thread_flow::incremental::graph::DependencyGraph;
//! use thread_utilities::RapidSet;
//! use std::path::PathBuf;
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
//! let changed: RapidSet<PathBuf> = [PathBuf::from("src/utils.rs")].into_iter().collect();
//! let affected = graph.find_affected_files(&changed);
//! assert!(affected.contains(&PathBuf::from("src/main.rs")));
//! ```
//!
//! ### Runtime Backend Selection
//!
//! ```rust
//! use thread_flow::incremental::{create_backend, BackendType, BackendConfig};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Select backend based on deployment environment
//! let backend = if cfg!(feature = "postgres-backend") {
//!     create_backend(
//!         BackendType::Postgres,
//!         BackendConfig::Postgres {
//!             database_url: std::env::var("DATABASE_URL")?,
//!         },
//!     ).await?
//! } else if cfg!(feature = "d1-backend") {
//!     create_backend(
//!         BackendType::D1,
//!         BackendConfig::D1 {
//!             account_id: std::env::var("CF_ACCOUNT_ID")?,
//!             database_id: std::env::var("CF_DATABASE_ID")?,
//!             api_token: std::env::var("CF_API_TOKEN")?,
//!         },
//!     ).await?
//! } else {
//!     // Fallback to in-memory for testing
//!     create_backend(BackendType::InMemory, BackendConfig::InMemory).await?
//! };
//! # Ok(())
//! # }
//! ```
//!
//! ### Persistent Storage with Incremental Updates
//!
//! ```rust,ignore
//! use thread_flow::incremental::{
//!     create_backend, BackendType, BackendConfig,
//!     StorageBackend, AnalysisDefFingerprint, DependencyGraph,
//! };
//! use std::path::Path;
//!
//! async fn incremental_analysis(backend: &dyn StorageBackend) -> Result<(), Box<dyn std::error::Error>> {
//!     // Load previous dependency graph
//!     let mut graph = backend.load_full_graph().await?;
//!
//!     // Check if file changed
//!     let file_path = Path::new("src/main.rs");
//!     let new_fp = AnalysisDefFingerprint::new(b"new content");
//!
//!     if let Some(old_fp) = backend.load_fingerprint(file_path).await? {
//!         if !old_fp.content_matches(b"new content") {
//!             // File changed - invalidate and re-analyze
//!             let affected = graph.find_affected_files(&[file_path.to_path_buf()].into());
//!             for affected_file in affected {
//!                 // Re-analyze affected files...
//!             }
//!         }
//!     }
//!
//!     // Save updated state
//!     backend.save_fingerprint(file_path, &new_fp).await?;
//!     backend.save_full_graph(&graph).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Migration Guide
//!
//! ### From Direct Storage Usage to Backend Factory
//!
//! **Before (direct backend instantiation):**
//! ```rust,ignore
//! #[cfg(feature = "postgres-backend")]
//! use thread_flow::incremental::backends::postgres::PostgresIncrementalBackend;
//!
//! let backend = PostgresIncrementalBackend::new(database_url).await?;
//! ```
//!
//! **After (factory pattern):**
//! ```rust,ignore
//! use thread_flow::incremental::{create_backend, BackendType, BackendConfig};
//!
//! let backend = create_backend(
//!     BackendType::Postgres,
//!     BackendConfig::Postgres { database_url },
//! ).await?;
//! ```
//!
//! ### Feature Flag Configuration
//!
//! **CLI deployment (Postgres):**
//! ```toml
//! [dependencies]
//! thread-flow = { version = "*", features = ["postgres-backend", "parallel"] }
//! ```
//!
//! **Edge deployment (D1):**
//! ```toml
//! [dependencies]
//! thread-flow = { version = "*", features = ["d1-backend", "worker"] }
//! ```
//!
//! **Testing (InMemory):**
//! ```toml
//! [dev-dependencies]
//! thread-flow = { version = "*" }  # InMemory always available
//! ```

pub mod analyzer;
pub mod backends;
pub mod concurrency;
pub mod dependency_builder;
pub mod extractors;
pub mod graph;
pub mod invalidation;
pub mod storage;
pub mod types;

// Re-export core types for ergonomic use
pub use analyzer::{AnalysisResult, AnalyzerError, IncrementalAnalyzer};
pub use graph::DependencyGraph;
pub use invalidation::{InvalidationDetector, InvalidationError, InvalidationResult};
pub use types::{
    AnalysisDefFingerprint, DependencyEdge, DependencyStrength, DependencyType, SymbolDependency,
    SymbolKind,
};

// Re-export backend factory and configuration for runtime backend selection
pub use backends::{BackendConfig, BackendType, IncrementalError, create_backend};

// Re-export storage trait for custom backend implementations
pub use storage::{InMemoryStorage, StorageBackend, StorageError};

// Re-export concurrency layer for parallel execution - TODO: Phase 4.3
// pub use concurrency::{create_executor, ConcurrencyMode, ExecutionError, Executor};

// Feature-gated backend re-exports
#[cfg(feature = "postgres-backend")]
pub use backends::PostgresIncrementalBackend;

#[cfg(feature = "d1-backend")]
pub use backends::D1IncrementalBackend;
