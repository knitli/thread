// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileCopyrightText: 2026 Knitli Inc.
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Observability instrumentation example demonstrating tracing and metrics collection.
//!
//! This example shows how to initialize the observability system and observe metrics
//! during incremental analysis operations.
//!
//! ## Features Demonstrated
//!
//! - Tracing configuration with env_logger
//! - Metrics collection using the `metrics` crate
//! - Integration with incremental analysis components
//! - Performance monitoring and cache hit rate tracking
//!
//! ## Usage
//!
//! ```bash
//! # Run with INFO level logging
//! RUST_LOG=info cargo run --example observability_example
//!
//! # Run with DEBUG level (includes file paths)
//! RUST_LOG=debug cargo run --example observability_example
//! ```

use std::path::PathBuf;
use std::time::Instant;
use tempfile::TempDir;
use thread_flow::incremental::analyzer::IncrementalAnalyzer;
use thread_flow::incremental::storage::InMemoryStorage;
use thread_flow::incremental::types::DependencyEdge;
use tokio::fs;

/// Initialize observability stack (logging and metrics).
fn init_observability() {
    // Initialize env_logger for tracing
    env_logger::Builder::from_default_env()
        .format_timestamp_micros()
        .init();

    // Initialize metrics recorder
    metrics_exporter_prometheus::PrometheusBuilder::new()
        .install()
        .expect("failed to install metrics recorder");

    tracing::info!("observability initialized");
}

/// Create a temporary test file with the given content.
async fn create_test_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).await.unwrap();
    path
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_observability();

    tracing::info!("=== Observability Example ===");

    // Create temporary directory for test files
    let temp_dir = tempfile::tempdir()?;

    // Create test files
    let file1 = create_test_file(&temp_dir, "main.rs", "fn main() {}").await;
    let file2 = create_test_file(&temp_dir, "utils.rs", "pub fn helper() {}").await;
    let file3 = create_test_file(&temp_dir, "lib.rs", "pub mod utils;").await;

    // Initialize analyzer with in-memory storage
    let storage = Box::new(InMemoryStorage::new());
    let mut analyzer = IncrementalAnalyzer::new(storage);

    tracing::info!("=== Phase 1: Initial Analysis (Cold Cache) ===");
    let start = Instant::now();

    // First analysis - all cache misses
    let result = analyzer
        .analyze_changes(&[file1.clone(), file2.clone(), file3.clone()])
        .await?;

    tracing::info!(
        "initial analysis: {} changed files, cache hit rate: {:.1}%, duration: {:?}",
        result.changed_files.len(),
        result.cache_hit_rate * 100.0,
        start.elapsed()
    );

    tracing::info!("=== Phase 2: Unchanged Analysis (Warm Cache) ===");
    let start = Instant::now();

    // Second analysis - all cache hits (no changes)
    let result = analyzer
        .analyze_changes(&[file1.clone(), file2.clone(), file3.clone()])
        .await?;

    tracing::info!(
        "warm cache analysis: {} changed files, cache hit rate: {:.1}%, duration: {:?}",
        result.changed_files.len(),
        result.cache_hit_rate * 100.0,
        start.elapsed()
    );

    tracing::info!("=== Phase 3: Partial Change (Mixed Cache) ===");

    // Modify one file
    fs::write(&file2, "pub fn helper() { println!(\"updated\"); }")
        .await
        .unwrap();

    let start = Instant::now();

    let result = analyzer
        .analyze_changes(&[file1.clone(), file2.clone(), file3.clone()])
        .await?;

    tracing::info!(
        "mixed cache analysis: {} changed files, cache hit rate: {:.1}%, duration: {:?}",
        result.changed_files.len(),
        result.cache_hit_rate * 100.0,
        start.elapsed()
    );

    tracing::info!("=== Phase 4: Dependency Graph Operations ===");

    // Add dependency edges to graph
    analyzer.graph_mut().add_edge(DependencyEdge::new(
        file3.clone(),
        file2.clone(),
        thread_flow::incremental::types::DependencyType::Import,
    ));

    analyzer.graph_mut().add_edge(DependencyEdge::new(
        file1.clone(),
        file3.clone(),
        thread_flow::incremental::types::DependencyType::Import,
    ));

    tracing::info!(
        "graph: {} nodes, {} edges",
        analyzer.graph().node_count(),
        analyzer.graph().edge_count()
    );

    // Test invalidation
    let start = Instant::now();
    let affected = analyzer
        .invalidate_dependents(std::slice::from_ref(&file2))
        .await?;

    tracing::info!(
        "invalidation: {} affected files, duration: {:?}",
        affected.len(),
        start.elapsed()
    );

    tracing::info!("=== Metrics Summary ===");
    tracing::info!("All operations complete. Metrics recorded:");
    tracing::info!("  - cache_hits_total: counter");
    tracing::info!("  - cache_misses_total: counter");
    tracing::info!("  - cache_hit_rate: gauge (target >90%)");
    tracing::info!("  - analysis_overhead_ms: histogram (target <10ms)");
    tracing::info!("  - invalidation_time_ms: histogram (target <50ms)");
    tracing::info!("  - graph_nodes: gauge");
    tracing::info!("  - graph_edges: gauge");
    tracing::info!("  - storage_reads_total: counter");
    tracing::info!("  - storage_writes_total: counter");
    tracing::info!("  - storage_read_latency_ms: histogram");
    tracing::info!("  - storage_write_latency_ms: histogram");

    Ok(())
}
