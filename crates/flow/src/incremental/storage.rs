// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Storage trait definitions for persisting dependency graphs and fingerprints.
//!
//! This module defines the abstract storage interface that enables the
//! incremental update system to persist state across sessions. Concrete
//! implementations are provided for:
//!
//! - **Postgres** (CLI deployment): Full-featured SQL backend
//! - **D1** (Edge deployment): Cloudflare Workers-compatible storage
//!
//! ## Design Pattern
//!
//! Adapted from ReCoco's `build_import_op_exec_ctx` persistence
//! (exec_ctx.rs:55-134) and setup state management.

use super::graph::{DependencyGraph, GraphError};
use super::types::{AnalysisDefFingerprint, DependencyEdge};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// Errors that can occur during storage operations.
#[derive(Debug)]
pub enum StorageError {
    /// The requested item was not found in storage.
    NotFound(String),

    /// A database or I/O error occurred.
    Backend(String),

    /// The stored data is corrupted or invalid.
    Corruption(String),

    /// A graph-level error propagated from graph operations.
    Graph(GraphError),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::NotFound(msg) => write!(f, "Storage item not found: {msg}"),
            StorageError::Backend(msg) => write!(f, "Storage backend error: {msg}"),
            StorageError::Corruption(msg) => write!(f, "Storage data corruption: {msg}"),
            StorageError::Graph(err) => write!(f, "Graph error: {err}"),
        }
    }
}

impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StorageError::Graph(err) => Some(err),
            _ => None,
        }
    }
}

impl From<GraphError> for StorageError {
    fn from(err: GraphError) -> Self {
        StorageError::Graph(err)
    }
}

/// Abstract storage backend for the incremental update system.
///
/// Provides async persistence for fingerprints and dependency edges.
/// Implementations must support both read and write operations, as well
/// as transactional consistency for batch updates.
///
/// # Implementors
///
/// - `PostgresStorage` (Phase 2): Full Postgres backend for CLI deployment
/// - `D1Storage` (Phase 2): Cloudflare D1 backend for edge deployment
///
/// # Examples
///
/// ```rust,ignore
/// # // This example requires a concrete implementation
/// use thread_flow::incremental::storage::StorageBackend;
///
/// async fn example(storage: &dyn StorageBackend) {
///     let fp = storage.load_fingerprint(Path::new("src/main.rs")).await;
/// }
/// ```
#[async_trait]
pub trait StorageBackend: Send + Sync + std::fmt::Debug {
    /// Persists a fingerprint for the given file path.
    ///
    /// Uses upsert semantics: creates a new entry or updates an existing one.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The file this fingerprint belongs to.
    /// * `fingerprint` - The fingerprint data to persist.
    async fn save_fingerprint(
        &self,
        file_path: &Path,
        fingerprint: &AnalysisDefFingerprint,
    ) -> Result<(), StorageError>;

    /// Loads the fingerprint for a file, if one exists.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The file to load the fingerprint for.
    ///
    /// # Returns
    ///
    /// `Ok(Some(fp))` if a fingerprint exists, `Ok(None)` if not found.
    async fn load_fingerprint(
        &self,
        file_path: &Path,
    ) -> Result<Option<AnalysisDefFingerprint>, StorageError>;

    /// Deletes the fingerprint for a file.
    ///
    /// Returns `Ok(true)` if a fingerprint was deleted, `Ok(false)` if
    /// no fingerprint existed for the path.
    async fn delete_fingerprint(&self, file_path: &Path) -> Result<bool, StorageError>;

    /// Persists a dependency edge.
    ///
    /// Uses upsert semantics based on the composite key
    /// (from, to, from_symbol, to_symbol, dep_type).
    async fn save_edge(&self, edge: &DependencyEdge) -> Result<(), StorageError>;

    /// Loads all dependency edges originating from a file.
    async fn load_edges_from(&self, file_path: &Path) -> Result<Vec<DependencyEdge>, StorageError>;

    /// Loads all dependency edges targeting a file.
    async fn load_edges_to(&self, file_path: &Path) -> Result<Vec<DependencyEdge>, StorageError>;

    /// Deletes all dependency edges involving a file (as source or target).
    async fn delete_edges_for(&self, file_path: &Path) -> Result<usize, StorageError>;

    /// Loads the complete dependency graph from storage.
    ///
    /// This is used during initialization to restore the graph state
    /// from the previous session.
    async fn load_full_graph(&self) -> Result<DependencyGraph, StorageError>;

    /// Persists the complete dependency graph to storage.
    ///
    /// This performs a full replacement of the stored graph.
    /// Used after graph rebuilds or major updates.
    async fn save_full_graph(&self, graph: &DependencyGraph) -> Result<(), StorageError>;
}

/// In-memory storage backend for testing purposes.
///
/// Stores all data in memory with no persistence. Useful for unit tests
/// and development scenarios.
///
/// # Examples
///
/// ```rust
/// use thread_flow::incremental::storage::InMemoryStorage;
///
/// let storage = InMemoryStorage::new();
/// ```
#[derive(Debug)]
pub struct InMemoryStorage {
    fingerprints: tokio::sync::RwLock<std::collections::HashMap<PathBuf, AnalysisDefFingerprint>>,
    edges: tokio::sync::RwLock<Vec<DependencyEdge>>,
}

impl InMemoryStorage {
    /// Creates a new empty in-memory storage backend.
    pub fn new() -> Self {
        Self {
            fingerprints: tokio::sync::RwLock::new(std::collections::HashMap::new()),
            edges: tokio::sync::RwLock::new(Vec::new()),
        }
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StorageBackend for InMemoryStorage {
    async fn save_fingerprint(
        &self,
        file_path: &Path,
        fingerprint: &AnalysisDefFingerprint,
    ) -> Result<(), StorageError> {
        let mut fps = self.fingerprints.write().await;
        fps.insert(file_path.to_path_buf(), fingerprint.clone());
        Ok(())
    }

    async fn load_fingerprint(
        &self,
        file_path: &Path,
    ) -> Result<Option<AnalysisDefFingerprint>, StorageError> {
        let fps = self.fingerprints.read().await;
        Ok(fps.get(file_path).cloned())
    }

    async fn delete_fingerprint(&self, file_path: &Path) -> Result<bool, StorageError> {
        let mut fps = self.fingerprints.write().await;
        Ok(fps.remove(file_path).is_some())
    }

    async fn save_edge(&self, edge: &DependencyEdge) -> Result<(), StorageError> {
        let mut edges = self.edges.write().await;
        edges.push(edge.clone());
        Ok(())
    }

    async fn load_edges_from(&self, file_path: &Path) -> Result<Vec<DependencyEdge>, StorageError> {
        let edges = self.edges.read().await;
        Ok(edges
            .iter()
            .filter(|e| e.from == file_path)
            .cloned()
            .collect())
    }

    async fn load_edges_to(&self, file_path: &Path) -> Result<Vec<DependencyEdge>, StorageError> {
        let edges = self.edges.read().await;
        Ok(edges
            .iter()
            .filter(|e| e.to == file_path)
            .cloned()
            .collect())
    }

    async fn delete_edges_for(&self, file_path: &Path) -> Result<usize, StorageError> {
        let mut edges = self.edges.write().await;
        let before = edges.len();
        edges.retain(|e| e.from != file_path && e.to != file_path);
        Ok(before - edges.len())
    }

    async fn load_full_graph(&self) -> Result<DependencyGraph, StorageError> {
        let edges = self.edges.read().await;
        let fps = self.fingerprints.read().await;

        let mut graph = DependencyGraph::new();

        // Restore fingerprint nodes
        for (path, fp) in fps.iter() {
            graph.nodes.insert(path.clone(), fp.clone());
        }

        // Restore edges
        for edge in edges.iter() {
            graph.add_edge(edge.clone());
        }

        Ok(graph)
    }

    async fn save_full_graph(&self, graph: &DependencyGraph) -> Result<(), StorageError> {
        let mut fps = self.fingerprints.write().await;
        let mut edges = self.edges.write().await;

        fps.clear();
        for (path, fp) in &graph.nodes {
            fps.insert(path.clone(), fp.clone());
        }

        edges.clear();
        edges.extend(graph.edges.iter().cloned());

        Ok(())
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::incremental::types::DependencyType;

    #[tokio::test]
    async fn test_in_memory_storage_save_and_load_fingerprint() {
        let storage = InMemoryStorage::new();
        let fp = AnalysisDefFingerprint::new(b"test content");

        storage
            .save_fingerprint(Path::new("src/main.rs"), &fp)
            .await
            .unwrap();

        let loaded = storage
            .load_fingerprint(Path::new("src/main.rs"))
            .await
            .unwrap();

        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert!(loaded.content_matches(b"test content"));
    }

    #[tokio::test]
    async fn test_in_memory_storage_load_nonexistent_fingerprint() {
        let storage = InMemoryStorage::new();
        let loaded = storage
            .load_fingerprint(Path::new("nonexistent.rs"))
            .await
            .unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_in_memory_storage_delete_fingerprint() {
        let storage = InMemoryStorage::new();
        let fp = AnalysisDefFingerprint::new(b"content");

        storage
            .save_fingerprint(Path::new("a.rs"), &fp)
            .await
            .unwrap();

        let deleted = storage.delete_fingerprint(Path::new("a.rs")).await.unwrap();
        assert!(deleted);

        let loaded = storage.load_fingerprint(Path::new("a.rs")).await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_in_memory_storage_delete_nonexistent_fingerprint() {
        let storage = InMemoryStorage::new();
        let deleted = storage
            .delete_fingerprint(Path::new("none.rs"))
            .await
            .unwrap();
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_in_memory_storage_save_and_load_edges() {
        let storage = InMemoryStorage::new();
        let edge = DependencyEdge::new(
            PathBuf::from("main.rs"),
            PathBuf::from("utils.rs"),
            DependencyType::Import,
        );

        storage.save_edge(&edge).await.unwrap();

        let from_edges = storage.load_edges_from(Path::new("main.rs")).await.unwrap();
        assert_eq!(from_edges.len(), 1);
        assert_eq!(from_edges[0].to, PathBuf::from("utils.rs"));

        let to_edges = storage.load_edges_to(Path::new("utils.rs")).await.unwrap();
        assert_eq!(to_edges.len(), 1);
        assert_eq!(to_edges[0].from, PathBuf::from("main.rs"));
    }

    #[tokio::test]
    async fn test_in_memory_storage_delete_edges() {
        let storage = InMemoryStorage::new();

        storage
            .save_edge(&DependencyEdge::new(
                PathBuf::from("a.rs"),
                PathBuf::from("b.rs"),
                DependencyType::Import,
            ))
            .await
            .unwrap();
        storage
            .save_edge(&DependencyEdge::new(
                PathBuf::from("c.rs"),
                PathBuf::from("a.rs"),
                DependencyType::Import,
            ))
            .await
            .unwrap();
        storage
            .save_edge(&DependencyEdge::new(
                PathBuf::from("d.rs"),
                PathBuf::from("e.rs"),
                DependencyType::Import,
            ))
            .await
            .unwrap();

        let deleted = storage.delete_edges_for(Path::new("a.rs")).await.unwrap();
        assert_eq!(deleted, 2); // Both edges involving a.rs

        // d.rs -> e.rs should remain
        let remaining = storage.load_edges_from(Path::new("d.rs")).await.unwrap();
        assert_eq!(remaining.len(), 1);
    }

    #[tokio::test]
    async fn test_in_memory_storage_full_graph_roundtrip() {
        let storage = InMemoryStorage::new();

        let mut graph = DependencyGraph::new();
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
            DependencyType::Import,
        ));
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("b.rs"),
            PathBuf::from("c.rs"),
            DependencyType::Import,
        ));

        storage.save_full_graph(&graph).await.unwrap();

        let loaded = storage.load_full_graph().await.unwrap();
        assert_eq!(loaded.edge_count(), 2);
        assert!(loaded.contains_node(Path::new("a.rs")));
        assert!(loaded.contains_node(Path::new("b.rs")));
        assert!(loaded.contains_node(Path::new("c.rs")));
    }

    #[tokio::test]
    async fn test_in_memory_storage_upsert_fingerprint() {
        let storage = InMemoryStorage::new();

        let fp1 = AnalysisDefFingerprint::new(b"version 1");
        storage
            .save_fingerprint(Path::new("file.rs"), &fp1)
            .await
            .unwrap();

        let fp2 = AnalysisDefFingerprint::new(b"version 2");
        storage
            .save_fingerprint(Path::new("file.rs"), &fp2)
            .await
            .unwrap();

        let loaded = storage
            .load_fingerprint(Path::new("file.rs"))
            .await
            .unwrap()
            .unwrap();

        assert!(loaded.content_matches(b"version 2"));
        assert!(!loaded.content_matches(b"version 1"));
    }

    // ── StorageError Tests ───────────────────────────────────────────────

    #[test]
    fn test_storage_error_display() {
        let err = StorageError::NotFound("file.rs".to_string());
        assert!(format!("{}", err).contains("file.rs"));

        let err = StorageError::Backend("connection refused".to_string());
        assert!(format!("{}", err).contains("connection refused"));

        let err = StorageError::Corruption("invalid checksum".to_string());
        assert!(format!("{}", err).contains("invalid checksum"));
    }

    #[test]
    fn test_storage_error_from_graph_error() {
        let graph_err = GraphError::CyclicDependency(PathBuf::from("a.rs"));
        let storage_err: StorageError = graph_err.into();

        match storage_err {
            StorageError::Graph(_) => {} // Expected
            _ => panic!("Expected StorageError::Graph"),
        }
    }
}
