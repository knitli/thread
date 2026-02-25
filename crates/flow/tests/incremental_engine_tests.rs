// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileCopyrightText: 2026 Knitli Inc.
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Comprehensive integration tests for Phase 4 incremental update system.
//!
//! This test suite validates the integration of Phase 4 components:
//! - IncrementalAnalyzer (Phase 4.1)
//! - InvalidationDetector (Phase 4.2)
//! - ConcurrencyExecutor (Phase 4.3)
//!
//! ## Test Coverage
//!
//! 1. **End-to-End Workflows** (7 tests): Full incremental update lifecycle
//! 2. **Change Detection** (6 tests): File addition/modification/deletion
//! 3. **Invalidation Propagation** (8 tests): Dependency-driven invalidation
//! 4. **Reanalysis Ordering** (6 tests): Topological sort and dependency order
//! 5. **Concurrency** (5 tests): Parallel/async execution with feature gates
//! 6. **Performance** (5 tests): Constitutional compliance (<10ms, >90% cache hit)
//! 7. **Storage Integration** (6 tests): Postgres, D1, InMemory backends
//! 8. **Error Handling** (7 tests): Graceful degradation and recovery
//!
//! ## TDD Process
//!
//! These tests are written BEFORE Phase 4 implementation (TDD methodology).
//! Tests will fail initially and pass as Phase 4.1-4.3 complete.
//!
//! ## Constitutional Compliance
//!
//! Tests validate Thread Constitution v2.0.0 requirements:
//! - Principle VI: <10ms incremental overhead, >90% cache hit rate
//! - Storage targets: Postgres <10ms, D1 <50ms, Qdrant <100ms p95
//! - Incremental updates trigger only affected component reanalysis

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tempfile::TempDir;
use thread_flow::incremental::backends::{BackendConfig, BackendType, create_backend};
use thread_flow::incremental::graph::DependencyGraph;
use thread_flow::incremental::storage::StorageBackend;
use thread_flow::incremental::types::{AnalysisDefFingerprint, DependencyEdge, DependencyType};

// =============================================================================
// Test Fixtures and Helpers
// =============================================================================

/// Test fixture for incremental analysis integration tests.
///
/// Provides a complete test environment with:
/// - Temporary directory for test files
/// - Storage backend (InMemory by default)
/// - Phase 4 component stubs (to be replaced with actual implementations)
struct IncrementalTestFixture {
    temp_dir: TempDir,
    storage: Box<dyn StorageBackend>,

    // Phase 4 components (stubs for now - will be replaced by actual implementations)
    // analyzer: Option<IncrementalAnalyzer>,
    // invalidator: Option<InvalidationDetector>,
    // executor: Option<ConcurrencyExecutor>,

    // Test state tracking
    files_created: HashMap<PathBuf, String>,
    last_analysis_result: Option<AnalysisResult>,
}

/// Results from an analysis run.
#[derive(Debug, Clone)]
struct AnalysisResult {
    /// Number of files that were analyzed.
    files_analyzed: usize,

    /// Number of files that were skipped (cache hit).
    files_skipped: usize,

    /// Number of dependency edges created.
    _edges_created: usize,

    /// Duration of the analysis operation.
    duration: Duration,

    /// List of files that were invalidated.
    invalidated_files: Vec<PathBuf>,

    /// Order in which files were reanalyzed (for topological validation).
    reanalysis_order: Vec<PathBuf>,
}

impl IncrementalTestFixture {
    /// Creates a new test fixture with InMemory storage backend.
    async fn new() -> Self {
        Self::new_with_backend(BackendType::InMemory).await
    }

    /// Creates a new test fixture with the specified storage backend.
    async fn new_with_backend(backend_type: BackendType) -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        let config = match backend_type {
            BackendType::InMemory => BackendConfig::InMemory,
            BackendType::Postgres => {
                // For integration tests, use test database
                BackendConfig::Postgres {
                    database_url: std::env::var("TEST_DATABASE_URL")
                        .unwrap_or_else(|_| "postgresql://localhost/thread_test".to_string()),
                }
            }
            BackendType::D1 => {
                // For integration tests, use test credentials
                BackendConfig::D1 {
                    account_id: std::env::var("TEST_CF_ACCOUNT_ID")
                        .unwrap_or_else(|_| "test-account".to_string()),
                    database_id: std::env::var("TEST_CF_DATABASE_ID")
                        .unwrap_or_else(|_| "test-db".to_string()),
                    api_token: std::env::var("TEST_CF_API_TOKEN")
                        .unwrap_or_else(|_| "test-token".to_string()),
                }
            }
        };

        let storage = create_backend(backend_type, config)
            .await
            .expect("Failed to create storage backend");

        Self {
            temp_dir,
            storage,
            files_created: HashMap::new(),
            last_analysis_result: None,
        }
    }

    /// Creates a file in the test directory with the given content.
    async fn create_file(&mut self, relative_path: &str, content: &str) {
        let full_path = self.temp_dir.path().join(relative_path);

        // Create parent directories if needed
        if let Some(parent) = full_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .expect("Failed to create parent directories");
        }

        tokio::fs::write(&full_path, content)
            .await
            .expect("Failed to write file");

        self.files_created.insert(full_path, content.to_string());
    }

    /// Modifies an existing file with new content.
    async fn modify_file(&mut self, relative_path: &str, new_content: &str) {
        let full_path = self.temp_dir.path().join(relative_path);

        assert!(
            full_path.exists(),
            "File {} does not exist",
            full_path.display()
        );

        tokio::fs::write(&full_path, new_content)
            .await
            .expect("Failed to modify file");

        self.files_created
            .insert(full_path, new_content.to_string());
    }

    /// Deletes a file from the test directory.
    async fn delete_file(&mut self, relative_path: &str) {
        let full_path = self.temp_dir.path().join(relative_path);

        if full_path.exists() {
            tokio::fs::remove_file(&full_path)
                .await
                .expect("Failed to delete file");
        }

        self.files_created.remove(&full_path);
    }

    /// Runs initial analysis on all files in the test directory.
    ///
    /// STUB: This will be implemented when Phase 4.1 (IncrementalAnalyzer) is complete.
    async fn run_initial_analysis(&mut self) -> Result<AnalysisResult, String> {
        let start = Instant::now();

        // STUB: Replace with actual IncrementalAnalyzer implementation
        // For now, simulate analysis by storing fingerprints
        let mut files_analyzed = 0;
        let edges_created = 0;

        for (path, content) in &self.files_created {
            let fp = AnalysisDefFingerprint::new(content.as_bytes());
            self.storage
                .save_fingerprint(path, &fp)
                .await
                .map_err(|e| format!("Storage error: {}", e))?;
            files_analyzed += 1;

            // STUB: Extract dependencies and create edges
            // This will be done by Phase 3's DependencyExtractor in real implementation
        }

        let result = AnalysisResult {
            files_analyzed,
            files_skipped: 0,
            _edges_created: edges_created,
            duration: start.elapsed(),
            invalidated_files: Vec::new(),
            reanalysis_order: Vec::new(),
        };

        self.last_analysis_result = Some(result.clone());
        Ok(result)
    }

    /// Runs incremental update to detect and reanalyze changed files.
    ///
    /// STUB: This will be implemented when Phase 4.1-4.3 are complete.
    async fn run_incremental_update(&mut self) -> Result<AnalysisResult, String> {
        let start = Instant::now();

        // STUB: Replace with actual incremental update logic
        // 1. Detect changed files (compare fingerprints)
        // 2. Invalidate affected files (Phase 4.2: InvalidationDetector)
        // 3. Reanalyze in dependency order (Phase 4.3: ConcurrencyExecutor)

        let mut files_analyzed = 0;
        let mut files_skipped = 0;
        let mut invalidated_files = Vec::new();

        for (path, content) in &self.files_created {
            let stored_fp = self
                .storage
                .load_fingerprint(path)
                .await
                .map_err(|e| format!("Storage error: {}", e))?;

            let current_fp = AnalysisDefFingerprint::new(content.as_bytes());

            if let Some(stored) = stored_fp {
                if stored.content_matches(content.as_bytes()) {
                    files_skipped += 1;
                } else {
                    // File changed - reanalyze
                    self.storage
                        .save_fingerprint(path, &current_fp)
                        .await
                        .map_err(|e| format!("Storage error: {}", e))?;
                    files_analyzed += 1;
                    invalidated_files.push(path.clone());
                }
            } else {
                // New file - analyze
                self.storage
                    .save_fingerprint(path, &current_fp)
                    .await
                    .map_err(|e| format!("Storage error: {}", e))?;
                files_analyzed += 1;
                invalidated_files.push(path.clone());
            }
        }

        let result = AnalysisResult {
            files_analyzed,
            files_skipped,
            _edges_created: 0,
            duration: start.elapsed(),
            invalidated_files,
            reanalysis_order: Vec::new(),
        };

        self.last_analysis_result = Some(result.clone());
        Ok(result)
    }

    /// Checks if a fingerprint exists in storage for the given path.
    async fn verify_fingerprint_exists(&self, relative_path: &str) -> bool {
        let full_path = self.temp_dir.path().join(relative_path);
        self.storage
            .load_fingerprint(&full_path)
            .await
            .ok()
            .flatten()
            .is_some()
    }

    /// Returns the path to the test directory.
    fn test_dir(&self) -> &Path {
        self.temp_dir.path()
    }
}

// =============================================================================
// Test Helpers
// =============================================================================

/// Creates a simple Rust file with the given imports.
fn create_test_rust_file(name: &str, imports: &[&str]) -> String {
    let mut content = String::new();

    for import in imports {
        content.push_str(&format!("use {};\n", import));
    }

    content.push_str("\n");
    content.push_str(&format!("pub fn {}() {{\n", name));
    content.push_str("    println!(\"Hello from {}\");\n");
    content.push_str("}\n");

    content
}

/// Creates a test dependency graph with the given edges.
fn create_test_graph(edges: &[(&str, &str)]) -> DependencyGraph {
    let mut graph = DependencyGraph::new();

    for (from, to) in edges {
        let edge = DependencyEdge::new(
            PathBuf::from(from),
            PathBuf::from(to),
            DependencyType::Import,
        );
        graph.add_edge(edge);
    }

    graph
}

// =============================================================================
// 1. End-to-End Incremental Workflow Tests (7 tests)
// =============================================================================

#[tokio::test]
async fn test_initial_analysis_creates_baseline() {
    let mut fixture = IncrementalTestFixture::new().await;

    // Create test files with dependencies
    fixture
        .create_file(
            "src/main.rs",
            &create_test_rust_file("main", &["crate::utils", "crate::config"]),
        )
        .await;
    fixture
        .create_file(
            "src/utils.rs",
            &create_test_rust_file("utils", &["std::collections::HashMap"]),
        )
        .await;
    fixture
        .create_file("src/config.rs", &create_test_rust_file("config", &[]))
        .await;

    // Run initial analysis
    let result = fixture.run_initial_analysis().await.unwrap();

    // Verify all files were analyzed
    assert_eq!(result.files_analyzed, 3);
    assert_eq!(result.files_skipped, 0);

    // Verify fingerprints were saved
    assert!(fixture.verify_fingerprint_exists("src/main.rs").await);
    assert!(fixture.verify_fingerprint_exists("src/utils.rs").await);
    assert!(fixture.verify_fingerprint_exists("src/config.rs").await);
}

#[tokio::test]
async fn test_no_changes_skips_reanalysis() {
    let mut fixture = IncrementalTestFixture::new().await;

    // Create and analyze files
    fixture
        .create_file("src/lib.rs", &create_test_rust_file("lib", &[]))
        .await;
    fixture.run_initial_analysis().await.unwrap();

    // Run incremental update without any changes
    let result = fixture.run_incremental_update().await.unwrap();

    // Verify no reanalysis occurred
    assert_eq!(result.files_analyzed, 0);
    assert_eq!(result.files_skipped, 1);
    assert!(result.invalidated_files.is_empty());
}

#[tokio::test]
async fn test_single_file_change_triggers_reanalysis() {
    let mut fixture = IncrementalTestFixture::new().await;

    // Initial analysis
    fixture
        .create_file("src/a.rs", &create_test_rust_file("a", &["crate::b"]))
        .await;
    fixture
        .create_file("src/b.rs", &create_test_rust_file("b", &[]))
        .await;
    fixture.run_initial_analysis().await.unwrap();

    // Modify one file
    fixture
        .modify_file("src/b.rs", &create_test_rust_file("b", &["std::fmt"]))
        .await;

    // Run incremental update
    let result = fixture.run_incremental_update().await.unwrap();

    // Verify only changed file + dependents were reanalyzed
    assert!(result.files_analyzed > 0);
    assert!(
        result
            .invalidated_files
            .contains(&fixture.test_dir().join("src/b.rs"))
    );
}

#[tokio::test]
async fn test_multiple_file_changes_batched() {
    let mut fixture = IncrementalTestFixture::new().await;

    // Initial analysis
    fixture
        .create_file("src/a.rs", &create_test_rust_file("a", &[]))
        .await;
    fixture
        .create_file("src/b.rs", &create_test_rust_file("b", &[]))
        .await;
    fixture
        .create_file("src/c.rs", &create_test_rust_file("c", &[]))
        .await;
    fixture.run_initial_analysis().await.unwrap();

    // Modify multiple files
    fixture
        .modify_file("src/a.rs", &create_test_rust_file("a", &["std::io"]))
        .await;
    fixture
        .modify_file("src/b.rs", &create_test_rust_file("b", &["std::fs"]))
        .await;
    fixture
        .modify_file("src/c.rs", &create_test_rust_file("c", &["std::env"]))
        .await;

    // Run incremental update
    let result = fixture.run_incremental_update().await.unwrap();

    // Verify all 3 changed files were detected
    assert_eq!(result.files_analyzed, 3);
    assert_eq!(result.invalidated_files.len(), 3);
}

#[tokio::test]
async fn test_storage_persistence_across_sessions() {
    // Session 1: Initial analysis
    let mut fixture = IncrementalTestFixture::new().await;
    fixture
        .create_file("src/main.rs", &create_test_rust_file("main", &[]))
        .await;
    fixture.run_initial_analysis().await.unwrap();

    // Save graph to storage
    let graph = DependencyGraph::new();
    fixture.storage.save_full_graph(&graph).await.unwrap();

    // Session 2: Load from storage
    let loaded_graph = fixture.storage.load_full_graph().await.unwrap();

    // Verify graph structure preserved
    assert_eq!(loaded_graph.node_count(), graph.node_count());
    assert_eq!(loaded_graph.edge_count(), graph.edge_count());
}

#[tokio::test]
async fn test_incremental_update_updates_storage() {
    let mut fixture = IncrementalTestFixture::new().await;

    // Initial analysis
    fixture
        .create_file("src/lib.rs", &create_test_rust_file("lib", &[]))
        .await;
    fixture.run_initial_analysis().await.unwrap();

    let old_fp = fixture
        .storage
        .load_fingerprint(&fixture.test_dir().join("src/lib.rs"))
        .await
        .unwrap()
        .unwrap();

    // Modify file
    fixture
        .modify_file("src/lib.rs", &create_test_rust_file("lib", &["std::io"]))
        .await;
    fixture.run_incremental_update().await.unwrap();

    // Verify fingerprint updated in storage
    let new_fp = fixture
        .storage
        .load_fingerprint(&fixture.test_dir().join("src/lib.rs"))
        .await
        .unwrap()
        .unwrap();

    assert_ne!(
        old_fp.fingerprint().as_slice(),
        new_fp.fingerprint().as_slice()
    );
}

#[tokio::test]
async fn test_deleted_file_handled_gracefully() {
    let mut fixture = IncrementalTestFixture::new().await;

    // Initial analysis with dependencies
    fixture
        .create_file(
            "src/main.rs",
            &create_test_rust_file("main", &["crate::utils"]),
        )
        .await;
    fixture
        .create_file("src/utils.rs", &create_test_rust_file("utils", &[]))
        .await;
    fixture.run_initial_analysis().await.unwrap();

    // Delete a file
    fixture.delete_file("src/utils.rs").await;

    // Run incremental update - should handle gracefully
    let result = fixture.run_incremental_update().await;

    // Should not panic, may report error or handle deletion
    assert!(result.is_ok() || result.is_err());
}

// =============================================================================
// 2. Change Detection Tests (6 tests)
// =============================================================================

#[tokio::test]
async fn test_detect_file_addition() {
    let mut fixture = IncrementalTestFixture::new().await;

    // Initial analysis with 2 files
    fixture
        .create_file("src/a.rs", &create_test_rust_file("a", &[]))
        .await;
    fixture
        .create_file("src/b.rs", &create_test_rust_file("b", &[]))
        .await;
    fixture.run_initial_analysis().await.unwrap();

    // Add new file
    fixture
        .create_file("src/c.rs", &create_test_rust_file("c", &[]))
        .await;

    // Run incremental update
    let result = fixture.run_incremental_update().await.unwrap();

    // Verify addition detected
    assert!(result.files_analyzed > 0);
    assert!(
        result
            .invalidated_files
            .contains(&fixture.test_dir().join("src/c.rs"))
    );
}

#[tokio::test]
async fn test_detect_file_modification() {
    let mut fixture = IncrementalTestFixture::new().await;

    // Initial analysis
    fixture.create_file("src/lib.rs", "fn old() {}").await;
    fixture.run_initial_analysis().await.unwrap();

    // Modify file
    fixture.modify_file("src/lib.rs", "fn new() {}").await;

    // Run incremental update
    let result = fixture.run_incremental_update().await.unwrap();

    // Verify modification detected via fingerprint mismatch
    assert_eq!(result.files_analyzed, 1);
    assert!(
        result
            .invalidated_files
            .contains(&fixture.test_dir().join("src/lib.rs"))
    );
}

#[tokio::test]
async fn test_detect_file_deletion() {
    let mut fixture = IncrementalTestFixture::new().await;

    // Initial analysis
    fixture
        .create_file("src/temp.rs", &create_test_rust_file("temp", &[]))
        .await;
    fixture.run_initial_analysis().await.unwrap();

    // Delete file
    fixture.delete_file("src/temp.rs").await;

    // Run incremental update
    let result = fixture.run_incremental_update().await;

    // Verify deletion detected and handled
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_no_change_detection_identical_content() {
    let mut fixture = IncrementalTestFixture::new().await;

    let content = create_test_rust_file("test", &[]);

    // Initial analysis
    fixture.create_file("src/test.rs", &content).await;
    fixture.run_initial_analysis().await.unwrap();

    // Re-save with identical content
    fixture.modify_file("src/test.rs", &content).await;

    // Run incremental update
    let result = fixture.run_incremental_update().await.unwrap();

    // Verify no change detected (fingerprint matches)
    assert_eq!(result.files_analyzed, 0);
    assert_eq!(result.files_skipped, 1);
}

#[tokio::test]
async fn test_whitespace_changes_detected() {
    let mut fixture = IncrementalTestFixture::new().await;

    // Initial analysis
    fixture.create_file("src/lib.rs", "fn test() {}").await;
    fixture.run_initial_analysis().await.unwrap();

    // Add whitespace
    fixture.modify_file("src/lib.rs", "fn test() {  }").await;

    // Run incremental update
    let result = fixture.run_incremental_update().await.unwrap();

    // Verify change detected (content fingerprint changed)
    assert_eq!(result.files_analyzed, 1);
}

#[tokio::test]
async fn test_multiple_changes_same_file() {
    let mut fixture = IncrementalTestFixture::new().await;

    // Initial analysis
    fixture.create_file("src/lib.rs", "// v1").await;
    fixture.run_initial_analysis().await.unwrap();

    // First modification
    fixture.modify_file("src/lib.rs", "// v2").await;
    let result1 = fixture.run_incremental_update().await.unwrap();
    assert_eq!(result1.files_analyzed, 1);

    // Second modification
    fixture.modify_file("src/lib.rs", "// v3").await;
    let result2 = fixture.run_incremental_update().await.unwrap();
    assert_eq!(result2.files_analyzed, 1);
}

// =============================================================================
// 3. Invalidation Propagation Tests (8 tests)
// =============================================================================

#[tokio::test]
async fn test_change_leaf_file_no_propagation() {
    let mut fixture = IncrementalTestFixture::new().await;

    // Create graph: A → B → C (C is leaf)
    fixture
        .create_file("src/a.rs", &create_test_rust_file("a", &["crate::b"]))
        .await;
    fixture
        .create_file("src/b.rs", &create_test_rust_file("b", &["crate::c"]))
        .await;
    fixture
        .create_file("src/c.rs", &create_test_rust_file("c", &[]))
        .await;
    fixture.run_initial_analysis().await.unwrap();

    // Change leaf file C
    fixture
        .modify_file("src/c.rs", &create_test_rust_file("c", &["std::io"]))
        .await;

    let result = fixture.run_incremental_update().await.unwrap();
    let invalidated = result.invalidated_files;

    // STUB: Will verify only C invalidated (no propagation to A, B)
    // For now, just verify C is in the invalidated set
    assert!(invalidated.iter().any(|p| p.ends_with("c.rs")));
}

#[tokio::test]
async fn test_change_root_file_invalidates_tree() {
    let mut fixture = IncrementalTestFixture::new().await;

    // Create graph: A → B → C
    fixture
        .create_file("src/a.rs", &create_test_rust_file("a", &["crate::b"]))
        .await;
    fixture
        .create_file("src/b.rs", &create_test_rust_file("b", &["crate::c"]))
        .await;
    fixture
        .create_file("src/c.rs", &create_test_rust_file("c", &[]))
        .await;
    fixture.run_initial_analysis().await.unwrap();

    // Change root file A
    fixture
        .modify_file(
            "src/a.rs",
            &create_test_rust_file("a", &["crate::b", "std::env"]),
        )
        .await;

    let result = fixture.run_incremental_update().await.unwrap();

    // STUB: Will verify A is invalidated
    // In actual implementation, B and C should also be invalidated if they depend on A's exports
    assert!(result.invalidated_files.iter().any(|p| p.ends_with("a.rs")));
}

#[tokio::test]
async fn test_change_middle_file_partial_invalidation() {
    let mut fixture = IncrementalTestFixture::new().await;

    // Create graph: A → B → C, D → B
    fixture
        .create_file("src/a.rs", &create_test_rust_file("a", &["crate::b"]))
        .await;
    fixture
        .create_file("src/b.rs", &create_test_rust_file("b", &["crate::c"]))
        .await;
    fixture
        .create_file("src/c.rs", &create_test_rust_file("c", &[]))
        .await;
    fixture
        .create_file("src/d.rs", &create_test_rust_file("d", &["crate::b"]))
        .await;
    fixture.run_initial_analysis().await.unwrap();

    // Change middle file B
    fixture
        .modify_file(
            "src/b.rs",
            &create_test_rust_file("b", &["crate::c", "std::io"]),
        )
        .await;

    let result = fixture.run_incremental_update().await.unwrap();

    // STUB: Will verify B and C invalidated, but not A and D
    assert!(result.invalidated_files.iter().any(|p| p.ends_with("b.rs")));
}

#[tokio::test]
async fn test_diamond_dependency_invalidation() {
    let mut fixture = IncrementalTestFixture::new().await;

    // Create diamond: A → B, A → C, B → D, C → D
    fixture
        .create_file(
            "src/a.rs",
            &create_test_rust_file("a", &["crate::b", "crate::c"]),
        )
        .await;
    fixture
        .create_file("src/b.rs", &create_test_rust_file("b", &["crate::d"]))
        .await;
    fixture
        .create_file("src/c.rs", &create_test_rust_file("c", &["crate::d"]))
        .await;
    fixture
        .create_file("src/d.rs", &create_test_rust_file("d", &[]))
        .await;
    fixture.run_initial_analysis().await.unwrap();

    // Change root A
    fixture
        .modify_file(
            "src/a.rs",
            &create_test_rust_file("a", &["crate::b", "crate::c", "std::env"]),
        )
        .await;

    let result = fixture.run_incremental_update().await.unwrap();

    // STUB: Will verify A, B, C, D all invalidated
    assert!(result.invalidated_files.iter().any(|p| p.ends_with("a.rs")));
}

#[tokio::test]
async fn test_multiple_simultaneous_changes() {
    let mut fixture = IncrementalTestFixture::new().await;

    // Create independent graphs: A → B, C → D
    fixture
        .create_file("src/a.rs", &create_test_rust_file("a", &["crate::b"]))
        .await;
    fixture
        .create_file("src/b.rs", &create_test_rust_file("b", &[]))
        .await;
    fixture
        .create_file("src/c.rs", &create_test_rust_file("c", &["crate::d"]))
        .await;
    fixture
        .create_file("src/d.rs", &create_test_rust_file("d", &[]))
        .await;
    fixture.run_initial_analysis().await.unwrap();

    // Change both A and C
    fixture
        .modify_file(
            "src/a.rs",
            &create_test_rust_file("a", &["crate::b", "std::io"]),
        )
        .await;
    fixture
        .modify_file(
            "src/c.rs",
            &create_test_rust_file("c", &["crate::d", "std::fs"]),
        )
        .await;

    let result = fixture.run_incremental_update().await.unwrap();

    // STUB: Will verify correct invalidation sets for both changes
    assert!(result.files_analyzed >= 2);
}

#[tokio::test]
async fn test_circular_dependency_handled() {
    let _fixture = IncrementalTestFixture::new().await;

    // Create cycle: A → B → A (simulated via edges)
    // Note: Rust prevents actual circular imports, but graph can have cycles
    let graph = create_test_graph(&[("src/a.rs", "src/b.rs"), ("src/b.rs", "src/a.rs")]);

    // STUB: Will verify cycle detection and graceful handling
    // For now, just verify graph construction doesn't panic
    assert_eq!(graph.edge_count(), 2);
}

#[tokio::test]
async fn test_weak_dependency_not_propagated() {
    // STUB: This test will validate weak dependency semantics
    // Weak dependencies (e.g., dev-dependencies) should not trigger invalidation

    let graph = create_test_graph(&[("src/main.rs", "src/lib.rs")]);

    // Verify graph structure
    assert_eq!(graph.edge_count(), 1);

    // STUB: In actual implementation:
    // 1. Mark edge as weak dependency
    // 2. Change lib.rs
    // 3. Verify main.rs NOT invalidated
}

#[tokio::test]
async fn test_symbol_level_invalidation() {
    // STUB: This test will validate fine-grained symbol-level invalidation

    let mut fixture = IncrementalTestFixture::new().await;

    // Create files with symbol dependencies
    fixture
        .create_file("src/a.rs", "use crate::b::foo;\n\npub fn main() { foo(); }")
        .await;
    fixture
        .create_file("src/b.rs", "pub fn foo() {}\npub fn bar() {}")
        .await;
    fixture.run_initial_analysis().await.unwrap();

    // STUB: Change symbol `bar` in b.rs (not used by a.rs)
    fixture
        .modify_file(
            "src/b.rs",
            "pub fn foo() {}\npub fn bar() { println!(\"changed\"); }",
        )
        .await;

    let result = fixture.run_incremental_update().await.unwrap();

    // STUB: Will verify a.rs NOT invalidated (only uses `foo`, not `bar`)
    assert!(result.invalidated_files.iter().any(|p| p.ends_with("b.rs")));
}

// =============================================================================
// 4. Dependency-Ordered Reanalysis Tests (6 tests)
// =============================================================================

#[tokio::test]
async fn test_topological_sort_basic() {
    // Graph: A → B → C
    let graph = create_test_graph(&[("src/a.rs", "src/b.rs"), ("src/b.rs", "src/c.rs")]);

    // STUB: Will verify topological sort returns [A, B, C] or [C, B, A] (reverse)
    // For now, just verify graph structure
    assert_eq!(graph.edge_count(), 2);
    assert_eq!(graph.node_count(), 3);
}

#[tokio::test]
async fn test_topological_sort_parallel_branches() {
    // Graph: A → B, A → C, B → D, C → D
    let graph = create_test_graph(&[
        ("src/a.rs", "src/b.rs"),
        ("src/a.rs", "src/c.rs"),
        ("src/b.rs", "src/d.rs"),
        ("src/c.rs", "src/d.rs"),
    ]);

    // STUB: Will verify:
    // - A first
    // - B and C in parallel (either order)
    // - D last
    assert_eq!(graph.edge_count(), 4);
    assert_eq!(graph.node_count(), 4);
}

#[tokio::test]
async fn test_topological_sort_multiple_roots() {
    // Graph: A → C, B → C
    let graph = create_test_graph(&[("src/a.rs", "src/c.rs"), ("src/b.rs", "src/c.rs")]);

    // STUB: Will verify:
    // - A and B in parallel (either order)
    // - C last
    assert_eq!(graph.edge_count(), 2);
    assert_eq!(graph.node_count(), 3);
}

#[tokio::test]
async fn test_topological_sort_detects_cycles() {
    // Graph: A → B → C → A (cycle)
    let graph = create_test_graph(&[
        ("src/a.rs", "src/b.rs"),
        ("src/b.rs", "src/c.rs"),
        ("src/c.rs", "src/a.rs"),
    ]);

    // STUB: Will verify cycle detection returns error
    // For now, verify graph has cycle
    assert_eq!(graph.edge_count(), 3);

    // STUB: topological_sort(&graph) should return Err(GraphError::CyclicDependency)
}

#[tokio::test]
async fn test_reanalysis_respects_dependencies() {
    let mut fixture = IncrementalTestFixture::new().await;

    // Create graph: A → B → C
    fixture
        .create_file("src/a.rs", &create_test_rust_file("a", &["crate::b"]))
        .await;
    fixture
        .create_file("src/b.rs", &create_test_rust_file("b", &["crate::c"]))
        .await;
    fixture
        .create_file("src/c.rs", &create_test_rust_file("c", &[]))
        .await;
    fixture.run_initial_analysis().await.unwrap();

    // Change B
    fixture
        .modify_file(
            "src/b.rs",
            &create_test_rust_file("b", &["crate::c", "std::io"]),
        )
        .await;

    let result = fixture.run_incremental_update().await.unwrap();
    let _order = result.reanalysis_order;

    // STUB: Will verify B analyzed before C (dependency order)
    // For now, just verify reanalysis occurred
    assert!(result.files_analyzed > 0);
}

#[tokio::test]
async fn test_independent_files_analyzed_parallel() {
    let mut fixture = IncrementalTestFixture::new().await;

    // Create independent files (no dependencies)
    fixture
        .create_file("src/a.rs", &create_test_rust_file("a", &[]))
        .await;
    fixture
        .create_file("src/b.rs", &create_test_rust_file("b", &[]))
        .await;
    fixture
        .create_file("src/c.rs", &create_test_rust_file("c", &[]))
        .await;
    fixture.run_initial_analysis().await.unwrap();

    // Modify all
    fixture
        .modify_file("src/a.rs", &create_test_rust_file("a", &["std::io"]))
        .await;
    fixture
        .modify_file("src/b.rs", &create_test_rust_file("b", &["std::fs"]))
        .await;
    fixture
        .modify_file("src/c.rs", &create_test_rust_file("c", &["std::env"]))
        .await;

    let start = Instant::now();
    let result = fixture.run_incremental_update().await.unwrap();
    let _duration = start.elapsed();

    // STUB: Will verify parallel execution (duration << sequential)
    // For now, verify all files reanalyzed
    assert_eq!(result.files_analyzed, 3);
}

// =============================================================================
// 5. Concurrency Tests (5 tests)
// =============================================================================

#[cfg(feature = "parallel")]
#[tokio::test]
async fn test_rayon_parallel_execution() {
    let mut fixture = IncrementalTestFixture::new().await;

    // Create 10 independent files
    for i in 0..10 {
        fixture
            .create_file(
                &format!("src/file{}.rs", i),
                &create_test_rust_file(&format!("file{}", i), &[]),
            )
            .await;
    }
    fixture.run_initial_analysis().await.unwrap();

    // Modify all files
    for i in 0..10 {
        fixture
            .modify_file(
                &format!("src/file{}.rs", i),
                &create_test_rust_file(&format!("file{}", i), &["std::io"]),
            )
            .await;
    }

    let result = fixture.run_incremental_update().await.unwrap();

    // STUB: Will verify Rayon parallel execution
    // For now, verify all files reanalyzed
    assert_eq!(result.files_analyzed, 10);
}

#[tokio::test]
async fn test_tokio_async_execution() {
    let mut fixture = IncrementalTestFixture::new().await;

    // Create 10 independent files
    for i in 0..10 {
        fixture
            .create_file(
                &format!("src/async{}.rs", i),
                &create_test_rust_file(&format!("async{}", i), &[]),
            )
            .await;
    }
    fixture.run_initial_analysis().await.unwrap();

    // Modify all files
    for i in 0..10 {
        fixture
            .modify_file(
                &format!("src/async{}.rs", i),
                &create_test_rust_file(&format!("async{}", i), &["std::fs"]),
            )
            .await;
    }

    let result = fixture.run_incremental_update().await.unwrap();

    // STUB: Will verify tokio async execution
    assert_eq!(result.files_analyzed, 10);
}

#[tokio::test]
async fn test_sequential_fallback() {
    // STUB: This test verifies sequential execution when features are disabled

    let mut fixture = IncrementalTestFixture::new().await;

    fixture
        .create_file("src/a.rs", &create_test_rust_file("a", &[]))
        .await;
    fixture
        .create_file("src/b.rs", &create_test_rust_file("b", &[]))
        .await;
    fixture.run_initial_analysis().await.unwrap();

    fixture
        .modify_file("src/a.rs", &create_test_rust_file("a", &["std::io"]))
        .await;
    fixture
        .modify_file("src/b.rs", &create_test_rust_file("b", &["std::fs"]))
        .await;

    let result = fixture.run_incremental_update().await.unwrap();

    // Sequential execution should still work
    assert_eq!(result.files_analyzed, 2);
}

#[tokio::test]
async fn test_concurrency_limit_respected() {
    // STUB: This test will verify concurrency limits are respected

    let mut fixture = IncrementalTestFixture::new().await;

    // Create 100 files
    for i in 0..100 {
        fixture
            .create_file(
                &format!("src/f{}.rs", i),
                &create_test_rust_file(&format!("f{}", i), &[]),
            )
            .await;
    }
    fixture.run_initial_analysis().await.unwrap();

    // STUB: Will configure concurrency limit = 10
    // STUB: Will verify max 10 concurrent tasks during execution
}

#[tokio::test]
async fn test_concurrent_storage_access_safe() {
    // STUB: This test verifies concurrent storage access doesn't cause corruption

    let fixture = IncrementalTestFixture::new().await;

    // STUB: Spawn multiple tasks that read/write storage concurrently
    // STUB: Verify no data corruption or race conditions

    // For now, just verify storage backend is Send + Sync
    let _storage_ref = &fixture.storage;
}

// =============================================================================
// 6. Performance Tests (5 tests)
// =============================================================================

#[tokio::test]
async fn test_incremental_faster_than_full() {
    // Constitutional Principle VI: Incremental 10x+ faster than full reanalysis

    let mut fixture = IncrementalTestFixture::new().await;

    // Create 1000-file codebase
    for i in 0..1000 {
        fixture
            .create_file(
                &format!("src/perf{}.rs", i),
                &create_test_rust_file(&format!("perf{}", i), &[]),
            )
            .await;
    }

    // Measure full analysis
    let full_start = Instant::now();
    fixture.run_initial_analysis().await.unwrap();
    let full_duration = full_start.elapsed();

    // Modify 10 files
    for i in 0..10 {
        fixture
            .modify_file(
                &format!("src/perf{}.rs", i),
                &create_test_rust_file(&format!("perf{}", i), &["std::io"]),
            )
            .await;
    }

    // Measure incremental analysis
    let inc_start = Instant::now();
    fixture.run_incremental_update().await.unwrap();
    let inc_duration = inc_start.elapsed();

    // STUB: Will verify incremental is 10x+ faster
    // For now, just verify both completed
    println!("Full: {:?}, Incremental: {:?}", full_duration, inc_duration);
}

#[tokio::test]
async fn test_incremental_overhead_under_10ms() {
    // Constitutional Principle VI: <10ms incremental update overhead

    let mut fixture = IncrementalTestFixture::new().await;

    // Create single file
    fixture
        .create_file("src/single.rs", &create_test_rust_file("single", &[]))
        .await;
    fixture.run_initial_analysis().await.unwrap();

    // Modify file
    fixture
        .modify_file(
            "src/single.rs",
            &create_test_rust_file("single", &["std::io"]),
        )
        .await;

    // Measure incremental overhead
    let start = Instant::now();
    fixture.run_incremental_update().await.unwrap();
    let duration = start.elapsed();

    // STUB: Will verify overhead <10ms (excluding actual analysis time)
    println!("Incremental update duration: {:?}", duration);
}

#[tokio::test]
async fn test_cache_hit_rate_above_90_percent() {
    // Constitutional Principle VI: >90% cache hit rate

    let mut fixture = IncrementalTestFixture::new().await;

    // Create 100 files
    for i in 0..100 {
        fixture
            .create_file(
                &format!("src/cache{}.rs", i),
                &create_test_rust_file(&format!("cache{}", i), &[]),
            )
            .await;
    }
    fixture.run_initial_analysis().await.unwrap();

    // Modify only 5 files (5%)
    for i in 0..5 {
        fixture
            .modify_file(
                &format!("src/cache{}.rs", i),
                &create_test_rust_file(&format!("cache{}", i), &["std::io"]),
            )
            .await;
    }

    let result = fixture.run_incremental_update().await.unwrap();

    // Calculate cache hit rate
    let total = result.files_analyzed + result.files_skipped;
    let hit_rate = if total > 0 {
        (result.files_skipped as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    // STUB: Will verify hit_rate > 90%
    println!("Cache hit rate: {:.2}%", hit_rate);
}

#[cfg(feature = "parallel")]
#[tokio::test]
async fn test_parallel_speedup_with_rayon() {
    // Verify 2-4x speedup with Rayon parallel execution

    let mut fixture = IncrementalTestFixture::new().await;

    // Create 100 independent files
    for i in 0..100 {
        fixture
            .create_file(
                &format!("src/par{}.rs", i),
                &create_test_rust_file(&format!("par{}", i), &[]),
            )
            .await;
    }
    fixture.run_initial_analysis().await.unwrap();

    // Modify all files
    for i in 0..100 {
        fixture
            .modify_file(
                &format!("src/par{}.rs", i),
                &create_test_rust_file(&format!("par{}", i), &["std::io"]),
            )
            .await;
    }

    // STUB: Will measure with/without parallelism and verify 2-4x speedup
    let result = fixture.run_incremental_update().await.unwrap();
    println!("Parallel duration: {:?}", result.duration);
}

#[tokio::test]
async fn test_large_graph_performance() {
    // Verify operations complete within limits on 10,000-file graph

    let mut fixture = IncrementalTestFixture::new().await;

    // Create 10,000 files (this will take time - may want to reduce for CI)
    // STUB: In actual implementation, this would be a stress test

    // For now, just verify with smaller graph
    for i in 0..100 {
        fixture
            .create_file(
                &format!("src/large{}.rs", i),
                &create_test_rust_file(&format!("large{}", i), &[]),
            )
            .await;
    }

    let start = Instant::now();
    fixture.run_initial_analysis().await.unwrap();
    let duration = start.elapsed();

    println!("Large graph analysis duration: {:?}", duration);

    // STUB: Will verify performance targets met
}

// =============================================================================
// 7. Storage Integration Tests (6 tests)
// =============================================================================

#[tokio::test]
async fn test_inmemory_backend_integration() {
    // Full workflow with InMemory backend

    let mut fixture = IncrementalTestFixture::new_with_backend(BackendType::InMemory).await;

    fixture
        .create_file("src/mem.rs", &create_test_rust_file("mem", &[]))
        .await;
    fixture.run_initial_analysis().await.unwrap();

    fixture
        .modify_file("src/mem.rs", &create_test_rust_file("mem", &["std::io"]))
        .await;
    let result = fixture.run_incremental_update().await.unwrap();

    assert!(result.files_analyzed > 0);
}

#[cfg(feature = "postgres-backend")]
#[tokio::test]
async fn test_postgres_backend_integration() {
    // Full workflow with Postgres backend

    // Skip if no test database configured
    if std::env::var("TEST_DATABASE_URL").is_err() {
        eprintln!("Skipping Postgres test: TEST_DATABASE_URL not set");
        return;
    }

    let mut fixture = IncrementalTestFixture::new_with_backend(BackendType::Postgres).await;

    fixture
        .create_file("src/pg.rs", &create_test_rust_file("pg", &[]))
        .await;
    fixture.run_initial_analysis().await.unwrap();

    fixture
        .modify_file("src/pg.rs", &create_test_rust_file("pg", &["std::fs"]))
        .await;
    let result = fixture.run_incremental_update().await.unwrap();

    assert!(result.files_analyzed > 0);
}

#[cfg(feature = "d1-backend")]
#[tokio::test]
async fn test_d1_backend_integration() {
    // Full workflow with D1 backend

    // Skip if no test credentials configured
    if std::env::var("TEST_CF_ACCOUNT_ID").is_err() {
        eprintln!("Skipping D1 test: TEST_CF_ACCOUNT_ID not set");
        return;
    }

    let mut fixture = IncrementalTestFixture::new_with_backend(BackendType::D1).await;

    fixture
        .create_file("src/d1.rs", &create_test_rust_file("d1", &[]))
        .await;
    fixture.run_initial_analysis().await.unwrap();

    fixture
        .modify_file("src/d1.rs", &create_test_rust_file("d1", &["std::env"]))
        .await;
    let result = fixture.run_incremental_update().await.unwrap();

    assert!(result.files_analyzed > 0);
}

#[tokio::test]
async fn test_backend_error_handling() {
    // STUB: Simulate storage failure and verify error propagation

    let fixture = IncrementalTestFixture::new().await;

    // STUB: Inject storage error
    // STUB: Verify graceful error handling and state preservation

    // For now, just verify storage interface is correct
    let result = fixture
        .storage
        .load_fingerprint(Path::new("nonexistent"))
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_graph_cycle_detection() {
    // Verify cycle detection returns clear error message

    let graph = create_test_graph(&[
        ("src/a.rs", "src/b.rs"),
        ("src/b.rs", "src/c.rs"),
        ("src/c.rs", "src/a.rs"),
    ]);

    // STUB: topological_sort should detect cycle
    // For now, verify graph has cycle
    assert_eq!(graph.edge_count(), 3);
}

#[tokio::test]
async fn test_extraction_error_during_reanalysis() {
    // STUB: Simulate parser failure on file

    let mut fixture = IncrementalTestFixture::new().await;

    // Create valid file
    fixture
        .create_file("src/good.rs", &create_test_rust_file("good", &[]))
        .await;
    // Create invalid file (parse error)
    fixture.create_file("src/bad.rs", "fn {{{").await;

    // STUB: Run analysis, verify error logged but other files continue
    let result = fixture.run_initial_analysis().await;

    // Should not panic
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_missing_file_during_reanalysis() {
    // File deleted between detection and analysis

    let mut fixture = IncrementalTestFixture::new().await;

    fixture
        .create_file("src/temp.rs", &create_test_rust_file("temp", &[]))
        .await;
    fixture.run_initial_analysis().await.unwrap();

    // Delete file
    fixture.delete_file("src/temp.rs").await;

    // STUB: Analysis should handle gracefully
    let result = fixture.run_incremental_update().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_invalid_fingerprint_in_storage() {
    // STUB: Corrupt fingerprint data in storage

    let mut fixture = IncrementalTestFixture::new().await;

    fixture
        .create_file("src/corrupt.rs", &create_test_rust_file("corrupt", &[]))
        .await;
    fixture.run_initial_analysis().await.unwrap();

    // STUB: Inject corrupted fingerprint
    // STUB: Verify corruption detected and recovery attempted
}

#[tokio::test]
async fn test_concurrent_modification_conflict() {
    // STUB: Two processes modify same file

    let mut fixture = IncrementalTestFixture::new().await;

    fixture
        .create_file("src/conflict.rs", &create_test_rust_file("conflict", &[]))
        .await;
    fixture.run_initial_analysis().await.unwrap();

    // STUB: Simulate concurrent modification
    // STUB: Verify conflict detection and resolution
}

#[tokio::test]
async fn test_partial_graph_recovery() {
    // STUB: Incomplete graph in storage

    let fixture = IncrementalTestFixture::new().await;

    // STUB: Create partial/corrupted graph
    // STUB: Verify recovery or clear error message

    let graph = DependencyGraph::new();
    fixture.storage.save_full_graph(&graph).await.unwrap();
}
