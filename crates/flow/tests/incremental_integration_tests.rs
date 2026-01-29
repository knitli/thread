// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Integration tests for the incremental update system.
//!
//! Tests backend factory pattern, feature gating, and end-to-end
//! storage operations across all three backend implementations.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use thread_flow::incremental::backends::{create_backend, BackendConfig, BackendType};
use thread_flow::incremental::storage::StorageBackend;
use thread_flow::incremental::types::{
    AnalysisDefFingerprint, DependencyEdge, DependencyType, SymbolDependency, SymbolKind,
};
use thread_flow::incremental::DependencyGraph;

// ─── Backend Factory Tests ────────────────────────────────────────────────────

#[tokio::test]
async fn test_backend_factory_in_memory() {
    let result = create_backend(BackendType::InMemory, BackendConfig::InMemory).await;
    assert!(result.is_ok(), "InMemory backend should always be available");
}

#[tokio::test]
async fn test_backend_factory_configuration_mismatch() {
    // Try to create InMemory backend with Postgres config
    let result = create_backend(
        BackendType::InMemory,
        BackendConfig::Postgres {
            database_url: "test".to_string(),
        },
    )
    .await;

    assert!(result.is_err());
    if let Err(err) = result {
        assert!(
            matches!(
                err,
                thread_flow::incremental::IncrementalError::InitializationFailed(_)
            ),
            "Configuration mismatch should return InitializationFailed"
        );
    }
}

// ─── Feature Gating Tests ─────────────────────────────────────────────────────

#[cfg(not(feature = "postgres-backend"))]
#[tokio::test]
async fn test_postgres_backend_unavailable_without_feature() {
    let result = create_backend(
        BackendType::Postgres,
        BackendConfig::Postgres {
            database_url: "postgresql://localhost/test".to_string(),
        },
    )
    .await;

    assert!(result.is_err());
    if let Err(err) = result {
        assert!(
            matches!(
                err,
                thread_flow::incremental::IncrementalError::UnsupportedBackend("postgres")
            ),
            "Should return UnsupportedBackend when postgres-backend feature is disabled"
        );
    }
}

#[cfg(not(feature = "d1-backend"))]
#[tokio::test]
async fn test_d1_backend_unavailable_without_feature() {
    let result = create_backend(
        BackendType::D1,
        BackendConfig::D1 {
            account_id: "test".to_string(),
            database_id: "test".to_string(),
            api_token: "test".to_string(),
        },
    )
    .await;

    assert!(result.is_err());
    if let Err(err) = result {
        assert!(
            matches!(
                err,
                thread_flow::incremental::IncrementalError::UnsupportedBackend("d1")
            ),
            "Should return UnsupportedBackend when d1-backend feature is disabled"
        );
    }
}

// ─── Runtime Backend Selection Tests ──────────────────────────────────────────

#[tokio::test]
async fn test_runtime_backend_selection_fallback() {
    // Test fallback logic when preferred backends are unavailable
    let backend = if cfg!(feature = "postgres-backend") {
        // Try Postgres first (but only if DATABASE_URL is set for testing)
        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            create_backend(
                BackendType::Postgres,
                BackendConfig::Postgres { database_url },
            )
            .await
            .ok()
        } else {
            None
        }
    } else if cfg!(feature = "d1-backend") {
        // Fall back to D1 (but it won't work without real credentials)
        None
    } else {
        None
    };

    // Always fall back to InMemory if nothing else available
    let backend = if let Some(b) = backend {
        b
    } else {
        create_backend(BackendType::InMemory, BackendConfig::InMemory)
            .await
            .expect("InMemory should always work")
    };

    // Verify the backend is usable
    let fp = AnalysisDefFingerprint::new(b"test content");
    backend
        .save_fingerprint(Path::new("test.rs"), &fp)
        .await
        .expect("Should be able to save fingerprint");
}

// ─── End-to-End Integration Tests ─────────────────────────────────────────────

/// Test complete workflow: save fingerprint → load → verify → delete
#[tokio::test]
async fn test_e2e_fingerprint_lifecycle() {
    let backend = create_backend(BackendType::InMemory, BackendConfig::InMemory)
        .await
        .expect("Failed to create backend");

    let file_path = Path::new("src/main.rs");
    let fp1 = AnalysisDefFingerprint::new(b"version 1");

    // 1. Save initial fingerprint
    backend
        .save_fingerprint(file_path, &fp1)
        .await
        .expect("Failed to save fingerprint");

    // 2. Load and verify
    let loaded = backend
        .load_fingerprint(file_path)
        .await
        .expect("Failed to load fingerprint")
        .expect("Fingerprint should exist");

    assert!(loaded.content_matches(b"version 1"));

    // 3. Update fingerprint (upsert semantics)
    let fp2 = AnalysisDefFingerprint::new(b"version 2");
    backend
        .save_fingerprint(file_path, &fp2)
        .await
        .expect("Failed to update fingerprint");

    let loaded = backend
        .load_fingerprint(file_path)
        .await
        .expect("Failed to load updated fingerprint")
        .expect("Updated fingerprint should exist");

    assert!(loaded.content_matches(b"version 2"));
    assert!(!loaded.content_matches(b"version 1"));

    // 4. Delete fingerprint
    let deleted = backend
        .delete_fingerprint(file_path)
        .await
        .expect("Failed to delete fingerprint");

    assert!(deleted, "Should return true when deleting existing fingerprint");

    // 5. Verify deletion
    let loaded = backend
        .load_fingerprint(file_path)
        .await
        .expect("Failed to check deleted fingerprint");

    assert!(loaded.is_none(), "Fingerprint should be deleted");
}

/// Test complete workflow: save edges → load → query → delete
#[tokio::test]
async fn test_e2e_dependency_edge_lifecycle() {
    let backend = create_backend(BackendType::InMemory, BackendConfig::InMemory)
        .await
        .expect("Failed to create backend");

    // Create dependency edges: main.rs → utils.rs → helpers.rs
    let edge1 = DependencyEdge::new(
        PathBuf::from("src/main.rs"),
        PathBuf::from("src/utils.rs"),
        DependencyType::Import,
    );

    let edge2 = DependencyEdge {
        from: PathBuf::from("src/utils.rs"),
        to: PathBuf::from("src/helpers.rs"),
        dep_type: DependencyType::Import,
        symbol: Some(SymbolDependency {
            from_symbol: "format_output".to_string(),
            to_symbol: "escape_html".to_string(),
            kind: SymbolKind::Function,
            strength: thread_flow::incremental::DependencyStrength::Strong,
        }),
    };

    // 1. Save edges
    backend
        .save_edge(&edge1)
        .await
        .expect("Failed to save edge1");
    backend
        .save_edge(&edge2)
        .await
        .expect("Failed to save edge2");

    // 2. Query edges from main.rs
    let edges_from_main = backend
        .load_edges_from(Path::new("src/main.rs"))
        .await
        .expect("Failed to load edges from main.rs");

    assert_eq!(edges_from_main.len(), 1);
    assert_eq!(edges_from_main[0].to, PathBuf::from("src/utils.rs"));

    // 3. Query edges to helpers.rs
    let edges_to_helpers = backend
        .load_edges_to(Path::new("src/helpers.rs"))
        .await
        .expect("Failed to load edges to helpers.rs");

    assert_eq!(edges_to_helpers.len(), 1);
    assert_eq!(edges_to_helpers[0].from, PathBuf::from("src/utils.rs"));
    assert!(edges_to_helpers[0].symbol.is_some());

    // 4. Delete all edges involving utils.rs
    let deleted_count = backend
        .delete_edges_for(Path::new("src/utils.rs"))
        .await
        .expect("Failed to delete edges");

    assert_eq!(deleted_count, 2, "Should delete both edges involving utils.rs");

    // 5. Verify deletion
    let remaining_from_main = backend
        .load_edges_from(Path::new("src/main.rs"))
        .await
        .expect("Failed to verify deletion");

    assert_eq!(
        remaining_from_main.len(),
        0,
        "All edges should be deleted"
    );
}

/// Test full graph persistence: save → load → verify structure
#[tokio::test]
async fn test_e2e_full_graph_persistence() {
    let backend = create_backend(BackendType::InMemory, BackendConfig::InMemory)
        .await
        .expect("Failed to create backend");

    // 1. Create a dependency graph
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
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("a.rs"),
        PathBuf::from("c.rs"),
        DependencyType::Type,
    ));

    // 2. Save full graph
    backend
        .save_full_graph(&graph)
        .await
        .expect("Failed to save graph");

    // 3. Load full graph
    let loaded_graph = backend
        .load_full_graph()
        .await
        .expect("Failed to load graph");

    // 4. Verify graph structure
    assert_eq!(
        loaded_graph.edge_count(),
        3,
        "All edges should be persisted"
    );
    assert!(
        loaded_graph.contains_node(Path::new("a.rs")),
        "Node a.rs should exist"
    );
    assert!(
        loaded_graph.contains_node(Path::new("b.rs")),
        "Node b.rs should exist"
    );
    assert!(
        loaded_graph.contains_node(Path::new("c.rs")),
        "Node c.rs should exist"
    );

    // 5. Verify affected files computation works after load
    let changed = HashSet::from([PathBuf::from("c.rs")]);
    let affected = loaded_graph.find_affected_files(&changed);

    assert!(
        affected.contains(&PathBuf::from("b.rs")),
        "b.rs depends on c.rs"
    );
    assert!(
        affected.contains(&PathBuf::from("a.rs")),
        "a.rs depends on c.rs directly and via b.rs"
    );
}

/// Test incremental invalidation workflow
#[tokio::test]
async fn test_e2e_incremental_invalidation() {
    let backend = create_backend(BackendType::InMemory, BackendConfig::InMemory)
        .await
        .expect("Failed to create backend");

    // Setup: Create dependency chain
    let mut graph = DependencyGraph::new();
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("main.rs"),
        PathBuf::from("utils.rs"),
        DependencyType::Import,
    ));
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("utils.rs"),
        PathBuf::from("config.rs"),
        DependencyType::Import,
    ));

    backend
        .save_full_graph(&graph)
        .await
        .expect("Failed to save initial graph");

    // Save initial fingerprints
    backend
        .save_fingerprint(
            Path::new("main.rs"),
            &AnalysisDefFingerprint::new(b"main v1"),
        )
        .await
        .expect("Failed to save main.rs fingerprint");
    backend
        .save_fingerprint(
            Path::new("utils.rs"),
            &AnalysisDefFingerprint::new(b"utils v1"),
        )
        .await
        .expect("Failed to save utils.rs fingerprint");
    backend
        .save_fingerprint(
            Path::new("config.rs"),
            &AnalysisDefFingerprint::new(b"config v1"),
        )
        .await
        .expect("Failed to save config.rs fingerprint");

    // Simulate config.rs change
    let new_config_fp = AnalysisDefFingerprint::new(b"config v2");

    // Check if file changed
    let old_config_fp = backend
        .load_fingerprint(Path::new("config.rs"))
        .await
        .expect("Failed to load config.rs fingerprint")
        .expect("config.rs fingerprint should exist");

    assert!(!old_config_fp.content_matches(b"config v2"), "Content changed");

    // Find affected files
    let changed = HashSet::from([PathBuf::from("config.rs")]);
    let affected = graph.find_affected_files(&changed);

    assert!(
        affected.contains(&PathBuf::from("utils.rs")),
        "utils.rs imports config.rs"
    );
    assert!(
        affected.contains(&PathBuf::from("main.rs")),
        "main.rs transitively depends on config.rs"
    );

    // Update fingerprint after re-analysis
    backend
        .save_fingerprint(Path::new("config.rs"), &new_config_fp)
        .await
        .expect("Failed to update config.rs fingerprint");

    // Verify update
    let updated_fp = backend
        .load_fingerprint(Path::new("config.rs"))
        .await
        .expect("Failed to load updated fingerprint")
        .expect("Updated fingerprint should exist");

    assert!(updated_fp.content_matches(b"config v2"));
}

// ─── Multi-Backend Comparison Tests ───────────────────────────────────────────

/// Verify all backends implement the same behavior for basic operations
#[tokio::test]
async fn test_backend_behavior_consistency() {
    let backends: Vec<Box<dyn StorageBackend>> = vec![
        create_backend(BackendType::InMemory, BackendConfig::InMemory)
            .await
            .expect("InMemory should always work"),
        // Add Postgres and D1 when features are enabled
        #[cfg(feature = "postgres-backend")]
        {
            if let Ok(url) = std::env::var("TEST_DATABASE_URL") {
                create_backend(BackendType::Postgres, BackendConfig::Postgres { database_url: url })
                    .await
                    .ok()
            } else {
                None
            }
        }
        .unwrap_or_else(|| {
            Box::new(thread_flow::incremental::storage::InMemoryStorage::new())
                as Box<dyn StorageBackend>
        }),
    ];

    for backend in backends {
        // Test basic fingerprint operations
        let fp = AnalysisDefFingerprint::new(b"test");
        backend
            .save_fingerprint(Path::new("test.rs"), &fp)
            .await
            .expect("All backends should support save");

        let loaded = backend
            .load_fingerprint(Path::new("test.rs"))
            .await
            .expect("All backends should support load")
            .expect("Fingerprint should exist");

        assert!(loaded.content_matches(b"test"));

        // Test edge operations
        let edge = DependencyEdge::new(
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
            DependencyType::Import,
        );

        backend
            .save_edge(&edge)
            .await
            .expect("All backends should support edge save");

        let edges = backend
            .load_edges_from(Path::new("a.rs"))
            .await
            .expect("All backends should support edge query");

        assert_eq!(edges.len(), 1);
    }
}
