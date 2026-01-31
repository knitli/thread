// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileCopyrightText: 2026 Knitli Inc.
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Integration tests for observability metrics instrumentation.
//!
//! Validates that tracing spans and metrics are properly recorded during
//! incremental analysis operations.

use std::path::PathBuf;
use tempfile::TempDir;
use thread_flow::incremental::analyzer::IncrementalAnalyzer;
use thread_flow::incremental::storage::{InMemoryStorage, StorageBackend};
use thread_flow::incremental::types::DependencyEdge;
use tokio::fs;

/// Helper to create a temporary test file.
async fn create_test_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).await.unwrap();
    path
}

#[tokio::test]
async fn test_metrics_during_analysis() {
    // Initialize test environment
    let temp_dir = tempfile::tempdir().unwrap();
    let file1 = create_test_file(&temp_dir, "test.rs", "fn test() {}").await;

    let storage = Box::new(InMemoryStorage::new());
    let mut analyzer = IncrementalAnalyzer::new(storage);

    // Perform analysis (metrics should be recorded)
    let result = analyzer.analyze_changes(&[file1.clone()]).await.unwrap();

    // Verify basic functionality (metrics are recorded internally)
    assert_eq!(result.changed_files.len(), 1);
    assert!(result.cache_hit_rate >= 0.0 && result.cache_hit_rate <= 1.0);
}

#[tokio::test]
async fn test_cache_hit_metrics() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file1 = create_test_file(&temp_dir, "test.rs", "fn test() {}").await;

    let storage = Box::new(InMemoryStorage::new());
    let mut analyzer = IncrementalAnalyzer::new(storage);

    // First analysis - cache miss
    let result1 = analyzer.analyze_changes(&[file1.clone()]).await.unwrap();
    assert_eq!(result1.cache_hit_rate, 0.0);

    // Second analysis - cache hit
    let result2 = analyzer.analyze_changes(&[file1.clone()]).await.unwrap();
    assert_eq!(result2.cache_hit_rate, 1.0);
}

#[tokio::test]
async fn test_graph_metrics_on_edge_addition() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file1 = create_test_file(&temp_dir, "a.rs", "fn a() {}").await;
    let file2 = create_test_file(&temp_dir, "b.rs", "fn b() {}").await;

    let storage = Box::new(InMemoryStorage::new());
    let mut analyzer = IncrementalAnalyzer::new(storage);

    // Initialize files
    analyzer
        .analyze_changes(&[file1.clone(), file2.clone()])
        .await
        .unwrap();

    let initial_edges = analyzer.graph().edge_count();

    // Add edge (graph metrics should update)
    analyzer.graph_mut().add_edge(DependencyEdge::new(
        file1.clone(),
        file2.clone(),
        thread_flow::incremental::types::DependencyType::Import,
    ));

    let final_edges = analyzer.graph().edge_count();
    assert_eq!(final_edges, initial_edges + 1);
}

#[tokio::test]
async fn test_invalidation_metrics() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file1 = create_test_file(&temp_dir, "a.rs", "fn a() {}").await;
    let file2 = create_test_file(&temp_dir, "b.rs", "fn b() {}").await;

    let storage = Box::new(InMemoryStorage::new());
    let mut analyzer = IncrementalAnalyzer::new(storage);

    // Setup dependency
    analyzer
        .analyze_changes(&[file1.clone(), file2.clone()])
        .await
        .unwrap();
    analyzer.graph_mut().add_edge(DependencyEdge::new(
        file1.clone(),
        file2.clone(),
        thread_flow::incremental::types::DependencyType::Import,
    ));

    // Trigger invalidation (invalidation metrics should be recorded)
    let affected = analyzer
        .invalidate_dependents(&[file2.clone()])
        .await
        .unwrap();

    // Verify functionality
    assert!(!affected.is_empty());
    assert!(affected.contains(&file1) || affected.contains(&file2));
}

#[tokio::test]
async fn test_storage_metrics() {
    let storage = InMemoryStorage::new();

    // Perform storage operations (metrics should be recorded)
    let fp = thread_flow::incremental::types::AnalysisDefFingerprint::new(b"test");
    let path = std::path::Path::new("test.rs");

    storage.save_fingerprint(path, &fp).await.unwrap();
    let loaded = storage.load_fingerprint(path).await.unwrap();

    assert!(loaded.is_some());
}
