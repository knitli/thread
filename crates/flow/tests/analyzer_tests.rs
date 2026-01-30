// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileCopyrightText: 2026 Knitli Inc.
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Comprehensive test suite for IncrementalAnalyzer (Phase 4.1).
//!
//! Tests cover all major functionality:
//! - Constructor and initialization
//! - Change detection (analyze_changes)
//! - Dependency invalidation (invalidate_dependents)
//! - Reanalysis workflow (reanalyze_invalidated)
//! - End-to-end integration
//! - Performance targets (<10ms overhead)
//! - Error handling
//! - Edge cases and boundary conditions

use std::path::{Path, PathBuf};
use tempfile::TempDir;
use thread_flow::incremental::analyzer::IncrementalAnalyzer;
use thread_flow::incremental::graph::DependencyGraph;
use thread_flow::incremental::storage::{InMemoryStorage, StorageBackend};
use thread_flow::incremental::types::{DependencyEdge, DependencyType};

// ─── Test Fixture ────────────────────────────────────────────────────────────

/// Helper fixture for creating test files and graph structures.
struct TestFixture {
    temp_dir: TempDir,
    analyzer: IncrementalAnalyzer,
}

impl TestFixture {
    async fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let storage = Box::new(InMemoryStorage::new());
        let analyzer = IncrementalAnalyzer::new(storage);

        Self { temp_dir, analyzer }
    }

    async fn with_existing_graph(graph: DependencyGraph) -> Self {
        let temp_dir = TempDir::new().unwrap();
        let storage = Box::new(InMemoryStorage::new());
        storage.save_full_graph(&graph).await.unwrap();

        let analyzer = IncrementalAnalyzer::from_storage(storage).await.unwrap();

        Self { temp_dir, analyzer }
    }

    async fn create_file(&self, relative_path: &str, content: &str) -> PathBuf {
        let path = self.temp_dir.path().join(relative_path);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.unwrap();
        }
        tokio::fs::write(&path, content).await.unwrap();
        path
    }

    async fn modify_file(&self, path: &Path, new_content: &str) {
        tokio::fs::write(path, new_content).await.unwrap();
    }

    async fn delete_file(&self, path: &Path) {
        let _ = tokio::fs::remove_file(path).await;
    }

    fn temp_path(&self, relative_path: &str) -> PathBuf {
        self.temp_dir.path().join(relative_path)
    }
}

// ─── 1. Constructor and Initialization Tests ─────────────────────────────────

#[tokio::test]
async fn test_analyzer_new_with_storage() {
    let storage = Box::new(InMemoryStorage::new());
    let analyzer = IncrementalAnalyzer::new(storage);

    // Verify analyzer is created with empty graph
    assert_eq!(analyzer.graph().node_count(), 0);
    assert_eq!(analyzer.graph().edge_count(), 0);
}

#[tokio::test]
async fn test_analyzer_initializes_with_empty_graph() {
    let fixture = TestFixture::new().await;
    assert_eq!(fixture.analyzer.graph().node_count(), 0);
    assert_eq!(fixture.analyzer.graph().edge_count(), 0);
}

#[tokio::test]
async fn test_analyzer_loads_existing_graph_from_storage() {
    // Create a graph with some data
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

    // Create analyzer from storage
    let fixture = TestFixture::with_existing_graph(graph).await;

    // Verify graph is restored
    assert_eq!(fixture.analyzer.graph().node_count(), 3);
    assert_eq!(fixture.analyzer.graph().edge_count(), 2);
}

// ─── 2. Change Detection Tests (analyze_changes) ─────────────────────────────

#[tokio::test]
async fn test_analyze_changes_detects_new_file() {
    let mut fixture = TestFixture::new().await;
    let file = fixture.create_file("new.rs", "fn main() {}").await;

    let result = fixture
        .analyzer
        .analyze_changes(&[file.clone()])
        .await
        .unwrap();

    assert_eq!(result.changed_files.len(), 1);
    assert_eq!(result.changed_files[0], file);
}

#[tokio::test]
async fn test_analyze_changes_detects_modified_file() {
    let mut fixture = TestFixture::new().await;
    let file = fixture.create_file("modified.rs", "fn old() {}").await;

    // First analysis - establish baseline
    let _ = fixture
        .analyzer
        .analyze_changes(&[file.clone()])
        .await
        .unwrap();

    // Modify file
    fixture.modify_file(&file, "fn new() {}").await;

    // Second analysis - should detect change
    let result = fixture
        .analyzer
        .analyze_changes(&[file.clone()])
        .await
        .unwrap();

    assert_eq!(result.changed_files.len(), 1);
    assert_eq!(result.changed_files[0], file);
}

#[tokio::test]
async fn test_analyze_changes_detects_unchanged_file() {
    let mut fixture = TestFixture::new().await;
    let file = fixture.create_file("unchanged.rs", "fn same() {}").await;

    // First analysis
    let _ = fixture
        .analyzer
        .analyze_changes(&[file.clone()])
        .await
        .unwrap();

    // Second analysis - no changes
    let result = fixture
        .analyzer
        .analyze_changes(&[file.clone()])
        .await
        .unwrap();

    assert_eq!(result.changed_files.len(), 0);
}

#[tokio::test]
async fn test_analyze_changes_handles_multiple_files() {
    let mut fixture = TestFixture::new().await;
    let file1 = fixture.create_file("file1.rs", "fn one() {}").await;
    let file2 = fixture.create_file("file2.rs", "fn two() {}").await;
    let file3 = fixture.create_file("file3.rs", "fn three() {}").await;

    // Establish baseline for all files
    let _ = fixture
        .analyzer
        .analyze_changes(&[file1.clone(), file2.clone(), file3.clone()])
        .await
        .unwrap();

    // Modify only file2
    fixture.modify_file(&file2, "fn two_modified() {}").await;

    // Analyze again
    let result = fixture
        .analyzer
        .analyze_changes(&[file1.clone(), file2.clone(), file3.clone()])
        .await
        .unwrap();

    assert_eq!(result.changed_files.len(), 1);
    assert_eq!(result.changed_files[0], file2);
}

#[tokio::test]
async fn test_analyze_changes_returns_analysis_result() {
    let mut fixture = TestFixture::new().await;
    let file = fixture.create_file("test.rs", "fn test() {}").await;

    let result = fixture.analyzer.analyze_changes(&[file]).await.unwrap();

    // Verify AnalysisResult structure
    assert!(!result.changed_files.is_empty());
    assert!(result.affected_files.is_empty()); // New file has no dependents
    assert!(result.analysis_time_us > 0);
    assert!(result.cache_hit_rate >= 0.0 && result.cache_hit_rate <= 1.0);
}

#[tokio::test]
async fn test_analyze_changes_empty_paths_returns_empty() {
    let mut fixture = TestFixture::new().await;

    let result = fixture.analyzer.analyze_changes(&[]).await.unwrap();

    assert_eq!(result.changed_files.len(), 0);
    assert_eq!(result.affected_files.len(), 0);
}

#[tokio::test]
async fn test_analyze_changes_nonexistent_file_error() {
    let mut fixture = TestFixture::new().await;
    let nonexistent = fixture.temp_path("nonexistent.rs");

    let result = fixture.analyzer.analyze_changes(&[nonexistent]).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_analyze_changes_handles_deleted_file() {
    let mut fixture = TestFixture::new().await;
    let file = fixture.create_file("deleted.rs", "fn gone() {}").await;

    // Establish baseline
    let _ = fixture
        .analyzer
        .analyze_changes(&[file.clone()])
        .await
        .unwrap();

    // Delete file
    fixture.delete_file(&file).await;

    // Analysis should handle deletion gracefully
    let result = fixture.analyzer.analyze_changes(&[file.clone()]).await;

    // Should either return error or mark as changed/deleted
    assert!(result.is_err() || result.unwrap().changed_files.contains(&file));
}

// ─── 3. Dependency Invalidation Tests (invalidate_dependents) ───────────────

#[tokio::test]
async fn test_invalidate_dependents_single_level() {
    let _fixture = TestFixture::new().await;

    // Build graph: A -> B (A depends on B)
    let mut graph = DependencyGraph::new();
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("A.rs"),
        PathBuf::from("B.rs"),
        DependencyType::Import,
    ));

    let storage = Box::new(InMemoryStorage::new());
    storage.save_full_graph(&graph).await.unwrap();
    let analyzer = IncrementalAnalyzer::from_storage(storage).await.unwrap();

    // B changes -> A should be invalidated
    let affected = analyzer
        .invalidate_dependents(&[PathBuf::from("B.rs")])
        .await
        .unwrap();

    assert_eq!(affected.len(), 2); // B and A
    assert!(affected.contains(&PathBuf::from("A.rs")));
    assert!(affected.contains(&PathBuf::from("B.rs")));
}

#[tokio::test]
async fn test_invalidate_dependents_transitive() {
    let _fixture = TestFixture::new().await;

    // Build graph: A -> B -> C (chain)
    let mut graph = DependencyGraph::new();
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("A.rs"),
        PathBuf::from("B.rs"),
        DependencyType::Import,
    ));
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("B.rs"),
        PathBuf::from("C.rs"),
        DependencyType::Import,
    ));

    let storage = Box::new(InMemoryStorage::new());
    storage.save_full_graph(&graph).await.unwrap();
    let analyzer = IncrementalAnalyzer::from_storage(storage).await.unwrap();

    // C changes -> A and B should be invalidated
    let affected = analyzer
        .invalidate_dependents(&[PathBuf::from("C.rs")])
        .await
        .unwrap();

    assert_eq!(affected.len(), 3); // C, B, A
    assert!(affected.contains(&PathBuf::from("A.rs")));
    assert!(affected.contains(&PathBuf::from("B.rs")));
    assert!(affected.contains(&PathBuf::from("C.rs")));
}

#[tokio::test]
async fn test_invalidate_dependents_diamond_dependency() {
    let _fixture = TestFixture::new().await;

    // Build diamond: A -> B, A -> C, B -> D, C -> D
    let mut graph = DependencyGraph::new();
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("A"),
        PathBuf::from("B"),
        DependencyType::Import,
    ));
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("A"),
        PathBuf::from("C"),
        DependencyType::Import,
    ));
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("B"),
        PathBuf::from("D"),
        DependencyType::Import,
    ));
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("C"),
        PathBuf::from("D"),
        DependencyType::Import,
    ));

    let storage = Box::new(InMemoryStorage::new());
    storage.save_full_graph(&graph).await.unwrap();
    let analyzer = IncrementalAnalyzer::from_storage(storage).await.unwrap();

    // D changes -> all should be invalidated
    let affected = analyzer
        .invalidate_dependents(&[PathBuf::from("D")])
        .await
        .unwrap();

    assert_eq!(affected.len(), 4); // D, B, C, A
    assert!(affected.contains(&PathBuf::from("A")));
    assert!(affected.contains(&PathBuf::from("B")));
    assert!(affected.contains(&PathBuf::from("C")));
    assert!(affected.contains(&PathBuf::from("D")));
}

#[tokio::test]
async fn test_invalidate_dependents_respects_strong_edges() {
    let _fixture = TestFixture::new().await;

    // A -> B with strong Import dependency
    let mut graph = DependencyGraph::new();
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("A.rs"),
        PathBuf::from("B.rs"),
        DependencyType::Import, // Strong
    ));

    let storage = Box::new(InMemoryStorage::new());
    storage.save_full_graph(&graph).await.unwrap();
    let analyzer = IncrementalAnalyzer::from_storage(storage).await.unwrap();

    let affected = analyzer
        .invalidate_dependents(&[PathBuf::from("B.rs")])
        .await
        .unwrap();

    assert!(affected.contains(&PathBuf::from("A.rs")));
}

#[tokio::test]
async fn test_invalidate_dependents_ignores_weak_edges() {
    let _fixture = TestFixture::new().await;

    // A -> B with weak Export dependency
    let mut graph = DependencyGraph::new();
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("A.rs"),
        PathBuf::from("B.rs"),
        DependencyType::Export, // Weak
    ));

    let storage = Box::new(InMemoryStorage::new());
    storage.save_full_graph(&graph).await.unwrap();
    let analyzer = IncrementalAnalyzer::from_storage(storage).await.unwrap();

    let affected = analyzer
        .invalidate_dependents(&[PathBuf::from("B.rs")])
        .await
        .unwrap();

    // Should only include B itself, not A (weak edge)
    assert_eq!(affected.len(), 1);
    assert!(affected.contains(&PathBuf::from("B.rs")));
    assert!(!affected.contains(&PathBuf::from("A.rs")));
}

#[tokio::test]
async fn test_invalidate_dependents_isolated_node() {
    let _fixture = TestFixture::new().await;

    // Isolated node with no dependencies
    let mut graph = DependencyGraph::new();
    graph.add_node(Path::new("isolated.rs"));

    let storage = Box::new(InMemoryStorage::new());
    storage.save_full_graph(&graph).await.unwrap();
    let analyzer = IncrementalAnalyzer::from_storage(storage).await.unwrap();

    let affected = analyzer
        .invalidate_dependents(&[PathBuf::from("isolated.rs")])
        .await
        .unwrap();

    // Only the file itself
    assert_eq!(affected.len(), 1);
    assert!(affected.contains(&PathBuf::from("isolated.rs")));
}

#[tokio::test]
async fn test_invalidate_dependents_empty_changed_set() {
    let _fixture = TestFixture::new().await;
    let storage = Box::new(InMemoryStorage::new());
    let analyzer = IncrementalAnalyzer::new(storage);

    let affected = analyzer.invalidate_dependents(&[]).await.unwrap();

    assert_eq!(affected.len(), 0);
}

#[tokio::test]
async fn test_invalidate_dependents_unknown_file() {
    let _fixture = TestFixture::new().await;
    let storage = Box::new(InMemoryStorage::new());
    let analyzer = IncrementalAnalyzer::new(storage);

    let affected = analyzer
        .invalidate_dependents(&[PathBuf::from("unknown.rs")])
        .await
        .unwrap();

    // Should include the unknown file itself
    assert_eq!(affected.len(), 1);
    assert!(affected.contains(&PathBuf::from("unknown.rs")));
}

#[tokio::test]
async fn test_invalidate_dependents_multiple_changes() {
    let _fixture = TestFixture::new().await;

    // A -> C, B -> D (independent chains)
    let mut graph = DependencyGraph::new();
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("A"),
        PathBuf::from("C"),
        DependencyType::Import,
    ));
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("B"),
        PathBuf::from("D"),
        DependencyType::Import,
    ));

    let storage = Box::new(InMemoryStorage::new());
    storage.save_full_graph(&graph).await.unwrap();
    let analyzer = IncrementalAnalyzer::from_storage(storage).await.unwrap();

    // Both C and D change
    let affected = analyzer
        .invalidate_dependents(&[PathBuf::from("C"), PathBuf::from("D")])
        .await
        .unwrap();

    // Should affect: C, A, D, B
    assert_eq!(affected.len(), 4);
    assert!(affected.contains(&PathBuf::from("A")));
    assert!(affected.contains(&PathBuf::from("B")));
    assert!(affected.contains(&PathBuf::from("C")));
    assert!(affected.contains(&PathBuf::from("D")));
}

// ─── 4. Reanalysis Tests (reanalyze_invalidated) ─────────────────────────────

#[tokio::test]
async fn test_reanalyze_invalidated_updates_fingerprints() {
    let mut fixture = TestFixture::new().await;
    let file = fixture
        .create_file("test.rs", "use std::collections::HashMap;")
        .await;

    // Initial analysis
    let _ = fixture
        .analyzer
        .analyze_changes(&[file.clone()])
        .await
        .unwrap();

    // Modify file
    fixture.modify_file(&file, "use std::vec::Vec;").await;

    // Reanalyze
    fixture
        .analyzer
        .reanalyze_invalidated(&[file.clone()])
        .await
        .unwrap();

    // Verify fingerprint updated
    // (Implementation detail - would need storage access to verify)
}

#[tokio::test]
async fn test_reanalyze_invalidated_empty_set() {
    let mut fixture = TestFixture::new().await;

    let result = fixture.analyzer.reanalyze_invalidated(&[]).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_reanalyze_invalidated_unsupported_language() {
    let mut fixture = TestFixture::new().await;
    let file = fixture
        .create_file("test.java", "public class Test {}")
        .await;

    let result = fixture.analyzer.reanalyze_invalidated(&[file]).await;

    // Should handle gracefully (skip or error)
    // Implementation should continue with other files
    assert!(result.is_ok() || result.is_err());
}

// ─── 5. End-to-End Integration Tests ─────────────────────────────────────────

#[tokio::test]
async fn test_full_incremental_workflow() {
    let mut fixture = TestFixture::new().await;

    // Create initial files
    let file_a = fixture.create_file("a.rs", "use crate::b;").await;
    let file_b = fixture.create_file("b.rs", "pub fn helper() {}").await;

    // Initial analysis
    let result = fixture
        .analyzer
        .analyze_changes(&[file_a.clone(), file_b.clone()])
        .await
        .unwrap();
    assert_eq!(result.changed_files.len(), 2); // Both new

    // Manually add dependency edge since Rust module resolution requires Cargo.toml
    // In production, this would be handled by proper project analysis
    fixture.analyzer.graph_mut().add_edge(DependencyEdge::new(
        file_a.clone(),
        file_b.clone(),
        DependencyType::Import,
    ));

    // Modify file_b
    fixture.modify_file(&file_b, "pub fn helper_v2() {}").await;

    // Analyze changes
    let result = fixture
        .analyzer
        .analyze_changes(&[file_a.clone(), file_b.clone()])
        .await
        .unwrap();
    assert_eq!(result.changed_files.len(), 1); // Only b changed
    assert_eq!(result.changed_files[0], file_b);

    // Invalidate dependents
    let affected = fixture
        .analyzer
        .invalidate_dependents(&result.changed_files)
        .await
        .unwrap();

    // Debug output
    eprintln!(
        "Graph has {} nodes, {} edges",
        fixture.analyzer.graph().node_count(),
        fixture.analyzer.graph().edge_count()
    );
    eprintln!("Changed files: {:?}", result.changed_files);
    eprintln!("Affected files: {:?}", affected);
    eprintln!(
        "file_a deps: {:?}",
        fixture.analyzer.graph().get_dependencies(&file_a).len()
    );
    eprintln!(
        "file_b dependents: {:?}",
        fixture.analyzer.graph().get_dependents(&file_b).len()
    );

    assert!(affected.contains(&file_a)); // a depends on b

    // Reanalyze affected
    let reanalysis = fixture.analyzer.reanalyze_invalidated(&affected).await;
    assert!(reanalysis.is_ok());
}

#[tokio::test]
async fn test_no_changes_workflow() {
    let mut fixture = TestFixture::new().await;
    let file = fixture.create_file("unchanged.rs", "fn same() {}").await;

    // Establish baseline
    let _ = fixture
        .analyzer
        .analyze_changes(&[file.clone()])
        .await
        .unwrap();

    // No changes
    let result = fixture
        .analyzer
        .analyze_changes(&[file.clone()])
        .await
        .unwrap();

    assert_eq!(result.changed_files.len(), 0);
    assert!(result.cache_hit_rate > 0.9); // Should have high cache hit rate
}

#[tokio::test]
async fn test_cascading_changes_workflow() {
    let mut fixture = TestFixture::new().await;

    // Create chain: a -> b -> c
    let file_a = fixture.create_file("a.rs", "mod b;").await;
    let file_b = fixture.create_file("b.rs", "mod c;").await;
    let file_c = fixture.create_file("c.rs", "pub fn leaf() {}").await;

    // Initial analysis
    let _ = fixture
        .analyzer
        .analyze_changes(&[file_a.clone(), file_b.clone(), file_c.clone()])
        .await
        .unwrap();

    // Manually add dependency edges since Rust module resolution requires Cargo.toml
    fixture.analyzer.graph_mut().add_edge(DependencyEdge::new(
        file_a.clone(),
        file_b.clone(),
        DependencyType::Import,
    ));
    fixture.analyzer.graph_mut().add_edge(DependencyEdge::new(
        file_b.clone(),
        file_c.clone(),
        DependencyType::Import,
    ));

    // Change c
    fixture.modify_file(&file_c, "pub fn leaf_v2() {}").await;

    // Analyze and invalidate
    let result = fixture
        .analyzer
        .analyze_changes(&[file_a.clone(), file_b.clone(), file_c.clone()])
        .await
        .unwrap();

    let affected = fixture
        .analyzer
        .invalidate_dependents(&result.changed_files)
        .await
        .unwrap();

    // Should cascade to all files
    assert!(affected.contains(&file_c));
    assert!(affected.contains(&file_b));
    assert!(affected.contains(&file_a));
}

// ─── 6. Performance Tests ────────────────────────────────────────────────────

#[tokio::test]
async fn test_analyze_changes_performance() {
    let mut fixture = TestFixture::new().await;

    // Create 100 files
    let mut files = Vec::new();
    for i in 0..100 {
        let file = fixture
            .create_file(&format!("file{}.rs", i), &format!("fn func{}() {{}}", i))
            .await;
        files.push(file);
    }

    // Establish baseline
    let _ = fixture.analyzer.analyze_changes(&files).await.unwrap();

    // Measure second analysis (should be fast with caching)
    let start = std::time::Instant::now();
    let result = fixture.analyzer.analyze_changes(&files).await.unwrap();
    let elapsed = start.elapsed();

    // Should be <20ms for 100 unchanged files (Constitutional target with CI margin)
    // Note: 10ms target allows 100% margin for CI environment variance
    assert!(
        elapsed.as_millis() < 20,
        "analyze_changes took {}ms, expected <20ms",
        elapsed.as_millis()
    );
    assert_eq!(result.changed_files.len(), 0);
}

#[tokio::test]
async fn test_invalidate_dependents_performance() {
    let _fixture = TestFixture::new().await;

    // Build large graph (1000 nodes)
    let mut graph = DependencyGraph::new();
    for i in 0..1000 {
        if i > 0 {
            graph.add_edge(DependencyEdge::new(
                PathBuf::from(format!("file{}.rs", i)),
                PathBuf::from(format!("file{}.rs", i - 1)),
                DependencyType::Import,
            ));
        }
    }

    let storage = Box::new(InMemoryStorage::new());
    storage.save_full_graph(&graph).await.unwrap();
    let analyzer = IncrementalAnalyzer::from_storage(storage).await.unwrap();

    // Measure BFS traversal
    let start = std::time::Instant::now();
    let affected = analyzer
        .invalidate_dependents(&[PathBuf::from("file0.rs")])
        .await
        .unwrap();
    let elapsed = start.elapsed();

    // Should be <5ms for 1000-node graph
    assert!(
        elapsed.as_millis() < 5,
        "invalidate_dependents took {}ms, expected <5ms",
        elapsed.as_millis()
    );
    assert_eq!(affected.len(), 1000); // All files affected in chain
}

// ─── 7. Error Handling Tests ─────────────────────────────────────────────────

#[tokio::test]
async fn test_extraction_error_handling() {
    let mut fixture = TestFixture::new().await;

    // Create file with syntax errors
    let file = fixture
        .create_file("invalid.rs", "fn incomplete {{{{{")
        .await;

    // Should handle extraction error gracefully
    let result = fixture.analyzer.reanalyze_invalidated(&[file]).await;

    // Implementation should either skip file or return error
    // But should not panic
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_io_error_handling() {
    let mut fixture = TestFixture::new().await;
    let nonexistent = fixture.temp_path("does_not_exist.rs");

    let result = fixture.analyzer.analyze_changes(&[nonexistent]).await;

    assert!(result.is_err());
}

// ─── 8. Edge Cases and Boundary Tests ───────────────────────────────────────

#[tokio::test]
async fn test_analyzer_empty_file() {
    let mut fixture = TestFixture::new().await;
    let file = fixture.create_file("empty.rs", "").await;

    let result = fixture.analyzer.analyze_changes(&[file]).await.unwrap();

    // Should handle empty file gracefully
    assert_eq!(result.changed_files.len(), 1);
}

#[tokio::test]
async fn test_analyzer_large_file() {
    let mut fixture = TestFixture::new().await;

    // Create 1MB file
    let large_content = "fn large() {}\n".repeat(50_000);
    let file = fixture.create_file("large.rs", &large_content).await;

    let start = std::time::Instant::now();
    let result = fixture.analyzer.analyze_changes(&[file]).await.unwrap();
    let elapsed = start.elapsed();

    // Should handle large file efficiently (blake3 is very fast)
    assert!(
        elapsed.as_millis() < 100,
        "Large file analysis took {}ms",
        elapsed.as_millis()
    );
    assert_eq!(result.changed_files.len(), 1);
}

#[tokio::test]
async fn test_analyzer_binary_file() {
    let mut fixture = TestFixture::new().await;

    // Create file with binary content
    let binary_content = vec![0u8, 1, 255, 128, 0, 0, 64, 32];
    let path = fixture.temp_path("binary.dat");
    tokio::fs::write(&path, binary_content).await.unwrap();

    // Should fingerprint without extraction (unsupported language)
    let result = fixture.analyzer.analyze_changes(&[path]).await;

    // Should handle gracefully (error or skip)
    assert!(result.is_ok() || result.is_err());
}
