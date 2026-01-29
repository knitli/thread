// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Integration tests for the PostgreSQL incremental storage backend.
//!
//! These tests use `testcontainers` to spin up ephemeral Postgres instances.
//! They require Docker to be running on the host machine.
//!
//! Run with:
//! ```bash
//! cargo nextest run -p thread-flow --test incremental_postgres_tests --all-features
//! ```

#![cfg(feature = "postgres-backend")]

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

use testcontainers::ImageExt;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use thread_flow::incremental::backends::postgres::PostgresIncrementalBackend;
use thread_flow::incremental::graph::DependencyGraph;
use thread_flow::incremental::storage::StorageBackend;
use thread_flow::incremental::types::{
    AnalysisDefFingerprint, DependencyEdge, DependencyStrength, DependencyType, SymbolDependency,
    SymbolKind,
};

/// Helper: creates a Postgres container and returns the backend + container handle.
/// The container is kept alive as long as the returned handle is held.
async fn setup_backend() -> (
    PostgresIncrementalBackend,
    testcontainers::ContainerAsync<Postgres>,
) {
    let container = Postgres::default()
        .with_host_auth()
        .with_tag("16-alpine")
        .start()
        .await
        .expect("Failed to start Postgres container (is Docker running?)");

    let host_port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("Failed to get host port");

    let url = format!("postgresql://postgres@127.0.0.1:{host_port}/postgres");

    let backend = PostgresIncrementalBackend::new(&url)
        .await
        .expect("Failed to create backend");

    backend
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    (backend, container)
}

// ─── Fingerprint CRUD Tests ─────────────────────────────────────────────────

#[tokio::test]
async fn test_save_and_load_fingerprint() {
    let (backend, _container) = setup_backend().await;

    let fp = AnalysisDefFingerprint::new(b"fn main() {}");

    backend
        .save_fingerprint(Path::new("src/main.rs"), &fp)
        .await
        .unwrap();

    let loaded = backend
        .load_fingerprint(Path::new("src/main.rs"))
        .await
        .unwrap();

    assert!(loaded.is_some());
    let loaded = loaded.unwrap();
    assert!(loaded.content_matches(b"fn main() {}"));
}

#[tokio::test]
async fn test_load_nonexistent_fingerprint() {
    let (backend, _container) = setup_backend().await;

    let loaded = backend
        .load_fingerprint(Path::new("nonexistent.rs"))
        .await
        .unwrap();

    assert!(loaded.is_none());
}

#[tokio::test]
async fn test_upsert_fingerprint() {
    let (backend, _container) = setup_backend().await;

    let fp1 = AnalysisDefFingerprint::new(b"version 1");
    backend
        .save_fingerprint(Path::new("file.rs"), &fp1)
        .await
        .unwrap();

    let fp2 = AnalysisDefFingerprint::new(b"version 2");
    backend
        .save_fingerprint(Path::new("file.rs"), &fp2)
        .await
        .unwrap();

    let loaded = backend
        .load_fingerprint(Path::new("file.rs"))
        .await
        .unwrap()
        .unwrap();

    assert!(loaded.content_matches(b"version 2"));
    assert!(!loaded.content_matches(b"version 1"));
}

#[tokio::test]
async fn test_fingerprint_with_source_files() {
    let (backend, _container) = setup_backend().await;

    let sources = HashSet::from([
        PathBuf::from("src/utils.rs"),
        PathBuf::from("src/config.rs"),
    ]);
    let fp = AnalysisDefFingerprint::with_sources(b"content", sources.clone());

    backend
        .save_fingerprint(Path::new("src/main.rs"), &fp)
        .await
        .unwrap();

    let loaded = backend
        .load_fingerprint(Path::new("src/main.rs"))
        .await
        .unwrap()
        .unwrap();

    assert_eq!(loaded.source_files.len(), 2);
    assert!(loaded.source_files.contains(&PathBuf::from("src/utils.rs")));
    assert!(
        loaded
            .source_files
            .contains(&PathBuf::from("src/config.rs"))
    );
}

#[tokio::test]
async fn test_fingerprint_with_last_analyzed() {
    let (backend, _container) = setup_backend().await;

    let mut fp = AnalysisDefFingerprint::new(b"content");
    fp.set_last_analyzed(1706400000_000_000);

    backend
        .save_fingerprint(Path::new("file.rs"), &fp)
        .await
        .unwrap();

    let loaded = backend
        .load_fingerprint(Path::new("file.rs"))
        .await
        .unwrap()
        .unwrap();

    assert_eq!(loaded.last_analyzed, Some(1706400000_000_000));
}

#[tokio::test]
async fn test_delete_fingerprint() {
    let (backend, _container) = setup_backend().await;

    let fp = AnalysisDefFingerprint::new(b"content");
    backend
        .save_fingerprint(Path::new("a.rs"), &fp)
        .await
        .unwrap();

    let deleted = backend.delete_fingerprint(Path::new("a.rs")).await.unwrap();
    assert!(deleted);

    let loaded = backend.load_fingerprint(Path::new("a.rs")).await.unwrap();
    assert!(loaded.is_none());
}

#[tokio::test]
async fn test_delete_nonexistent_fingerprint() {
    let (backend, _container) = setup_backend().await;

    let deleted = backend
        .delete_fingerprint(Path::new("none.rs"))
        .await
        .unwrap();
    assert!(!deleted);
}

#[tokio::test]
async fn test_delete_fingerprint_cascades_source_files() {
    let (backend, _container) = setup_backend().await;

    let sources = HashSet::from([PathBuf::from("dep.rs")]);
    let fp = AnalysisDefFingerprint::with_sources(b"content", sources);

    backend
        .save_fingerprint(Path::new("main.rs"), &fp)
        .await
        .unwrap();

    // Delete should cascade to source_files
    backend
        .delete_fingerprint(Path::new("main.rs"))
        .await
        .unwrap();

    // Re-inserting should work without duplicate key errors
    let fp2 = AnalysisDefFingerprint::with_sources(
        b"new content",
        HashSet::from([PathBuf::from("other.rs")]),
    );
    backend
        .save_fingerprint(Path::new("main.rs"), &fp2)
        .await
        .unwrap();

    let loaded = backend
        .load_fingerprint(Path::new("main.rs"))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(loaded.source_files.len(), 1);
    assert!(loaded.source_files.contains(&PathBuf::from("other.rs")));
}

// ─── Edge CRUD Tests ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_save_and_load_edge() {
    let (backend, _container) = setup_backend().await;

    let edge = DependencyEdge::new(
        PathBuf::from("main.rs"),
        PathBuf::from("utils.rs"),
        DependencyType::Import,
    );

    backend.save_edge(&edge).await.unwrap();

    let from_edges = backend.load_edges_from(Path::new("main.rs")).await.unwrap();
    assert_eq!(from_edges.len(), 1);
    assert_eq!(from_edges[0].to, PathBuf::from("utils.rs"));
    assert_eq!(from_edges[0].dep_type, DependencyType::Import);

    let to_edges = backend.load_edges_to(Path::new("utils.rs")).await.unwrap();
    assert_eq!(to_edges.len(), 1);
    assert_eq!(to_edges[0].from, PathBuf::from("main.rs"));
}

#[tokio::test]
async fn test_save_edge_with_symbol() {
    let (backend, _container) = setup_backend().await;

    let symbol = SymbolDependency {
        from_symbol: "handler".to_string(),
        to_symbol: "Router".to_string(),
        kind: SymbolKind::Class,
        strength: DependencyStrength::Strong,
    };

    let edge = DependencyEdge::with_symbol(
        PathBuf::from("api.rs"),
        PathBuf::from("router.rs"),
        DependencyType::Import,
        symbol,
    );

    backend.save_edge(&edge).await.unwrap();

    let loaded = backend.load_edges_from(Path::new("api.rs")).await.unwrap();
    assert_eq!(loaded.len(), 1);

    let sym = loaded[0].symbol.as_ref().expect("Expected symbol");
    assert_eq!(sym.from_symbol, "handler");
    assert_eq!(sym.to_symbol, "Router");
    assert_eq!(sym.kind, SymbolKind::Class);
    assert_eq!(sym.strength, DependencyStrength::Strong);
}

#[tokio::test]
async fn test_edge_upsert_deduplication() {
    let (backend, _container) = setup_backend().await;

    let edge = DependencyEdge::new(
        PathBuf::from("a.rs"),
        PathBuf::from("b.rs"),
        DependencyType::Import,
    );

    // Save the same edge twice
    backend.save_edge(&edge).await.unwrap();
    backend.save_edge(&edge).await.unwrap();

    let loaded = backend.load_edges_from(Path::new("a.rs")).await.unwrap();
    assert_eq!(loaded.len(), 1, "Duplicate edges should be deduplicated");
}

#[tokio::test]
async fn test_delete_edges_for_file() {
    let (backend, _container) = setup_backend().await;

    // Create edges: a->b, c->a, d->e
    backend
        .save_edge(&DependencyEdge::new(
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
            DependencyType::Import,
        ))
        .await
        .unwrap();
    backend
        .save_edge(&DependencyEdge::new(
            PathBuf::from("c.rs"),
            PathBuf::from("a.rs"),
            DependencyType::Import,
        ))
        .await
        .unwrap();
    backend
        .save_edge(&DependencyEdge::new(
            PathBuf::from("d.rs"),
            PathBuf::from("e.rs"),
            DependencyType::Import,
        ))
        .await
        .unwrap();

    let deleted = backend.delete_edges_for(Path::new("a.rs")).await.unwrap();
    assert_eq!(deleted, 2, "Should delete both edges involving a.rs");

    // d->e should remain
    let remaining = backend.load_edges_from(Path::new("d.rs")).await.unwrap();
    assert_eq!(remaining.len(), 1);
}

#[tokio::test]
async fn test_save_edges_batch() {
    let (backend, _container) = setup_backend().await;

    let edges = vec![
        DependencyEdge::new(
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
            DependencyType::Import,
        ),
        DependencyEdge::new(
            PathBuf::from("a.rs"),
            PathBuf::from("c.rs"),
            DependencyType::Import,
        ),
        DependencyEdge::new(
            PathBuf::from("b.rs"),
            PathBuf::from("c.rs"),
            DependencyType::Trait,
        ),
    ];

    backend.save_edges_batch(&edges).await.unwrap();

    let from_a = backend.load_edges_from(Path::new("a.rs")).await.unwrap();
    assert_eq!(from_a.len(), 2);

    let from_b = backend.load_edges_from(Path::new("b.rs")).await.unwrap();
    assert_eq!(from_b.len(), 1);
    assert_eq!(from_b[0].dep_type, DependencyType::Trait);
}

// ─── Full Graph Roundtrip Tests ─────────────────────────────────────────────

#[tokio::test]
async fn test_full_graph_save_and_load() {
    let (backend, _container) = setup_backend().await;

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

    backend.save_full_graph(&graph).await.unwrap();

    let loaded = backend.load_full_graph().await.unwrap();
    assert_eq!(loaded.edge_count(), 2);
    assert!(loaded.contains_node(Path::new("a.rs")));
    assert!(loaded.contains_node(Path::new("b.rs")));
    assert!(loaded.contains_node(Path::new("c.rs")));
}

#[tokio::test]
async fn test_full_graph_with_fingerprints_and_sources() {
    let (backend, _container) = setup_backend().await;

    // Save fingerprints with source files
    let sources_a = HashSet::from([PathBuf::from("dep1.rs"), PathBuf::from("dep2.rs")]);
    let mut fp_a = AnalysisDefFingerprint::with_sources(b"content a", sources_a);
    fp_a.set_last_analyzed(1000);

    backend
        .save_fingerprint(Path::new("a.rs"), &fp_a)
        .await
        .unwrap();

    let fp_b = AnalysisDefFingerprint::new(b"content b");
    backend
        .save_fingerprint(Path::new("b.rs"), &fp_b)
        .await
        .unwrap();

    // Save edges
    backend
        .save_edge(&DependencyEdge::new(
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
            DependencyType::Import,
        ))
        .await
        .unwrap();

    // Load full graph
    let graph = backend.load_full_graph().await.unwrap();

    // Verify nodes have correct fingerprints
    let node_a = graph
        .nodes
        .get(Path::new("a.rs"))
        .expect("Node a.rs missing");
    assert!(node_a.content_matches(b"content a"));
    assert_eq!(node_a.source_files.len(), 2);
    assert_eq!(node_a.last_analyzed, Some(1000));

    let node_b = graph
        .nodes
        .get(Path::new("b.rs"))
        .expect("Node b.rs missing");
    assert!(node_b.content_matches(b"content b"));
}

#[tokio::test]
async fn test_full_graph_replace_clears_old_data() {
    let (backend, _container) = setup_backend().await;

    // Save initial graph
    let mut graph1 = DependencyGraph::new();
    graph1.add_edge(DependencyEdge::new(
        PathBuf::from("old_a.rs"),
        PathBuf::from("old_b.rs"),
        DependencyType::Import,
    ));
    backend.save_full_graph(&graph1).await.unwrap();

    // Save replacement graph
    let mut graph2 = DependencyGraph::new();
    graph2.add_edge(DependencyEdge::new(
        PathBuf::from("new_x.rs"),
        PathBuf::from("new_y.rs"),
        DependencyType::Trait,
    ));
    backend.save_full_graph(&graph2).await.unwrap();

    let loaded = backend.load_full_graph().await.unwrap();
    assert_eq!(loaded.edge_count(), 1);
    assert!(!loaded.contains_node(Path::new("old_a.rs")));
    assert!(loaded.contains_node(Path::new("new_x.rs")));
    assert!(loaded.contains_node(Path::new("new_y.rs")));
}

// ─── Performance Tests ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_single_operation_performance() {
    let (backend, _container) = setup_backend().await;

    // Warm up the connection
    let fp = AnalysisDefFingerprint::new(b"warmup");
    backend
        .save_fingerprint(Path::new("warmup.rs"), &fp)
        .await
        .unwrap();

    // Measure single save operation
    let mut durations = Vec::with_capacity(100);
    for i in 0..100 {
        let content = format!("content {i}");
        let fp = AnalysisDefFingerprint::new(content.as_bytes());
        let path_str = format!("perf_test_{i}.rs");
        let path = Path::new(&path_str);

        let start = Instant::now();
        backend.save_fingerprint(path, &fp).await.unwrap();
        durations.push(start.elapsed());
    }

    // Sort for percentile calculation
    durations.sort();
    let p95_index = (durations.len() as f64 * 0.95) as usize;
    let p95 = durations[p95_index];

    // Constitutional requirement: <10ms p95
    assert!(
        p95.as_millis() < 10,
        "p95 latency ({:?}) exceeds 10ms target",
        p95
    );

    // Also measure load operations
    let mut load_durations = Vec::with_capacity(100);
    for i in 0..100 {
        let path_str = format!("perf_test_{i}.rs");
        let path = Path::new(&path_str);

        let start = Instant::now();
        backend.load_fingerprint(path).await.unwrap();
        load_durations.push(start.elapsed());
    }

    load_durations.sort();
    let load_p95 = load_durations[p95_index];

    assert!(
        load_p95.as_millis() < 10,
        "Load p95 latency ({:?}) exceeds 10ms target",
        load_p95
    );
}

#[tokio::test]
async fn test_full_graph_load_performance() {
    let (backend, _container) = setup_backend().await;

    // Build a graph with 1000 nodes
    let mut graph = DependencyGraph::new();
    for i in 0..1000 {
        let from = PathBuf::from(format!("file_{i}.rs"));
        let to = PathBuf::from(format!("file_{}.rs", (i + 1) % 1000));
        graph.add_edge(DependencyEdge::new(from, to, DependencyType::Import));
    }

    backend.save_full_graph(&graph).await.unwrap();

    // Measure full graph load
    let start = Instant::now();
    let loaded = backend.load_full_graph().await.unwrap();
    let duration = start.elapsed();

    assert_eq!(loaded.edge_count(), 1000);

    // Constitutional target: <50ms for 1000 nodes
    assert!(
        duration.as_millis() < 50,
        "Full graph load ({:?}) exceeds 50ms target for 1000 nodes",
        duration
    );
}

// ─── Migration Idempotency Test ─────────────────────────────────────────────

#[tokio::test]
async fn test_migration_idempotent() {
    let (backend, _container) = setup_backend().await;

    // Running migrations again should not fail
    backend.run_migrations().await.unwrap();
    backend.run_migrations().await.unwrap();

    // And operations should still work
    let fp = AnalysisDefFingerprint::new(b"after re-migration");
    backend
        .save_fingerprint(Path::new("test.rs"), &fp)
        .await
        .unwrap();

    let loaded = backend
        .load_fingerprint(Path::new("test.rs"))
        .await
        .unwrap();
    assert!(loaded.is_some());
}
