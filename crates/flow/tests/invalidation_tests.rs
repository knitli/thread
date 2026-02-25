// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileCopyrightText: 2026 Knitli Inc.
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Integration tests for InvalidationDetector.
//!
//! Tests end-to-end invalidation detection, topological sorting,
//! and cycle detection across complex dependency graphs.

use std::path::PathBuf;
use thread_flow::incremental::graph::DependencyGraph;
use thread_flow::incremental::types::{DependencyEdge, DependencyType};

// Note: InvalidationDetector will be implemented based on these tests (TDD)
// These tests are written BEFORE implementation to validate design

// ─── Construction Tests ───────────────────────────────────────────────────────

#[test]
fn test_invalidation_detector_new() {
    let _graph = DependencyGraph::new();
    // let detector = InvalidationDetector::new(graph);
    // assert!(detector is valid);
}

#[test]
fn test_invalidation_detector_with_populated_graph() {
    let mut _graph = DependencyGraph::new();
    _graph.add_edge(DependencyEdge::new(
        PathBuf::from("A"),
        PathBuf::from("B"),
        DependencyType::Import,
    ));
    // let detector = InvalidationDetector::new(graph);
    // Verify detector has access to graph data
}

// ─── propagate_invalidation Tests ─────────────────────────────────────────────

#[test]
fn test_propagate_single_file_no_dependents() {
    let mut graph = DependencyGraph::new();
    graph.add_node(&PathBuf::from("isolated.rs"));

    // let detector = InvalidationDetector::new(graph);
    // let affected = detector.propagate_invalidation(&PathBuf::from("isolated.rs"));
    // assert_eq!(affected, vec![PathBuf::from("isolated.rs")]);
}

#[test]
fn test_propagate_linear_chain() {
    let mut graph = DependencyGraph::new();
    // A -> B -> C
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("A"),
        PathBuf::from("B"),
        DependencyType::Import,
    ));
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("B"),
        PathBuf::from("C"),
        DependencyType::Import,
    ));

    // let detector = InvalidationDetector::new(graph);
    // let affected = detector.propagate_invalidation(&PathBuf::from("C"));
    // Should return: C, B, A (all transitively affected)
    // assert_eq!(affected.len(), 3);
    // assert!(affected.contains(&PathBuf::from("A")));
    // assert!(affected.contains(&PathBuf::from("B")));
    // assert!(affected.contains(&PathBuf::from("C")));
}

#[test]
fn test_propagate_diamond_dependency() {
    let mut graph = DependencyGraph::new();
    // Diamond: A -> B, A -> C, B -> D, C -> D
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

    // let detector = InvalidationDetector::new(graph);
    // let affected = detector.propagate_invalidation(&PathBuf::from("D"));
    // Should return: D, B, C, A (all transitively affected, no duplicates)
    // assert_eq!(affected.len(), 4);
}

#[test]
fn test_propagate_respects_strong_dependencies_only() {
    let mut graph = DependencyGraph::new();
    // A -> B (strong Import), C -> B (weak Export)
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("A"),
        PathBuf::from("B"),
        DependencyType::Import, // Strong
    ));
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("C"),
        PathBuf::from("B"),
        DependencyType::Export, // Weak
    ));

    // let detector = InvalidationDetector::new(graph);
    // let affected = detector.propagate_invalidation(&PathBuf::from("B"));
    // Should return: B, A (C not affected due to weak dependency)
    // assert!(affected.contains(&PathBuf::from("A")));
    // assert!(affected.contains(&PathBuf::from("B")));
    // assert!(!affected.contains(&PathBuf::from("C")));
}

#[test]
fn test_propagate_unknown_file() {
    let _graph = DependencyGraph::new();
    // let detector = InvalidationDetector::new(graph);
    // let affected = detector.propagate_invalidation(&PathBuf::from("unknown.rs"));
    // Should return just the unknown file itself
    // assert_eq!(affected, vec![PathBuf::from("unknown.rs")]);
}

// ─── topological_sort Tests ───────────────────────────────────────────────────

#[test]
fn test_topological_sort_linear_chain() {
    let mut graph = DependencyGraph::new();
    // A -> B -> C (A depends on B, B depends on C)
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("A"),
        PathBuf::from("B"),
        DependencyType::Import,
    ));
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("B"),
        PathBuf::from("C"),
        DependencyType::Import,
    ));

    // let detector = InvalidationDetector::new(graph);
    // let sorted = detector.topological_sort(&[
    //     PathBuf::from("A"),
    //     PathBuf::from("B"),
    //     PathBuf::from("C"),
    // ]).unwrap();

    // C must come before B, B before A
    // let pos_a = sorted.iter().position(|p| p == &PathBuf::from("A")).unwrap();
    // let pos_b = sorted.iter().position(|p| p == &PathBuf::from("B")).unwrap();
    // let pos_c = sorted.iter().position(|p| p == &PathBuf::from("C")).unwrap();
    // assert!(pos_c < pos_b);
    // assert!(pos_b < pos_a);
}

#[test]
fn test_topological_sort_diamond() {
    let mut graph = DependencyGraph::new();
    // Diamond: A -> B, A -> C, B -> D, C -> D
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

    // let detector = InvalidationDetector::new(graph);
    // let sorted = detector.topological_sort(&[
    //     PathBuf::from("A"),
    //     PathBuf::from("B"),
    //     PathBuf::from("C"),
    //     PathBuf::from("D"),
    // ]).unwrap();

    // Verify D before B and C, B and C before A
    // let pos_a = sorted.iter().position(|p| p == &PathBuf::from("A")).unwrap();
    // let pos_b = sorted.iter().position(|p| p == &PathBuf::from("B")).unwrap();
    // let pos_c = sorted.iter().position(|p| p == &PathBuf::from("C")).unwrap();
    // let pos_d = sorted.iter().position(|p| p == &PathBuf::from("D")).unwrap();
    // assert!(pos_d < pos_b);
    // assert!(pos_d < pos_c);
    // assert!(pos_b < pos_a);
    // assert!(pos_c < pos_a);
}

#[test]
fn test_topological_sort_cycle_error() {
    let mut graph = DependencyGraph::new();
    // Cycle: A -> B -> A
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("A"),
        PathBuf::from("B"),
        DependencyType::Import,
    ));
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("B"),
        PathBuf::from("A"),
        DependencyType::Import,
    ));

    // let detector = InvalidationDetector::new(graph);
    // let result = detector.topological_sort(&[
    //     PathBuf::from("A"),
    //     PathBuf::from("B"),
    // ]);
    // assert!(result.is_err());
    // Match on InvalidationError::CircularDependency
}

#[test]
fn test_topological_sort_empty_set() {
    let _graph = DependencyGraph::new();
    // let detector = InvalidationDetector::new(graph);
    // let sorted = detector.topological_sort(&[]).unwrap();
    // assert!(sorted.is_empty());
}

// ─── compute_invalidation_set Tests ───────────────────────────────────────────

#[test]
fn test_compute_invalidation_single_change() {
    let mut graph = DependencyGraph::new();
    // A -> B
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("A"),
        PathBuf::from("B"),
        DependencyType::Import,
    ));

    // let detector = InvalidationDetector::new(graph);
    // let result = detector.compute_invalidation_set(&[PathBuf::from("B")]);

    // Verify:
    // - invalidated_files contains B and A
    // - analysis_order has B before A
    // - circular_dependencies is empty
}

#[test]
fn test_compute_invalidation_transitive() {
    let mut graph = DependencyGraph::new();
    // A -> B -> C
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("A"),
        PathBuf::from("B"),
        DependencyType::Import,
    ));
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("B"),
        PathBuf::from("C"),
        DependencyType::Import,
    ));

    // let detector = InvalidationDetector::new(graph);
    // let result = detector.compute_invalidation_set(&[PathBuf::from("C")]);

    // Verify:
    // - invalidated_files: [C, B, A]
    // - analysis_order: [C, B, A] (dependencies first)
    // - circular_dependencies: []
}

#[test]
fn test_compute_invalidation_with_cycles() {
    let mut graph = DependencyGraph::new();
    // Cycle: A -> B -> A, plus C -> A
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("A"),
        PathBuf::from("B"),
        DependencyType::Import,
    ));
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("B"),
        PathBuf::from("A"),
        DependencyType::Import,
    ));
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("C"),
        PathBuf::from("A"),
        DependencyType::Import,
    ));

    // let detector = InvalidationDetector::new(graph);
    // let result = detector.compute_invalidation_set(&[PathBuf::from("A")]);

    // Verify:
    // - invalidated_files: [A, B, C]
    // - analysis_order: may be empty or partial due to cycle
    // - circular_dependencies: [[A, B]] (one SCC with A and B)
}

#[test]
fn test_compute_invalidation_multiple_cycles() {
    let mut graph = DependencyGraph::new();
    // Two separate cycles: A -> B -> A, C -> D -> C
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("A"),
        PathBuf::from("B"),
        DependencyType::Import,
    ));
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("B"),
        PathBuf::from("A"),
        DependencyType::Import,
    ));
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("C"),
        PathBuf::from("D"),
        DependencyType::Import,
    ));
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("D"),
        PathBuf::from("C"),
        DependencyType::Import,
    ));

    // let detector = InvalidationDetector::new(graph);
    // let result = detector.compute_invalidation_set(&[
    //     PathBuf::from("A"),
    //     PathBuf::from("C"),
    // ]);

    // Verify:
    // - circular_dependencies has 2 entries: [A,B] and [C,D]
}

#[test]
fn test_compute_invalidation_empty_changes() {
    let _graph = DependencyGraph::new();
    // let detector = InvalidationDetector::new(graph);
    // let result = detector.compute_invalidation_set(&[]);

    // Verify:
    // - invalidated_files: []
    // - analysis_order: []
    // - circular_dependencies: []
}

// ─── Performance Tests ────────────────────────────────────────────────────────

#[test]
fn test_performance_large_graph() {
    // Build graph with 1000+ nodes
    let mut graph = DependencyGraph::new();
    for i in 0..1000 {
        graph.add_edge(DependencyEdge::new(
            PathBuf::from(format!("file_{}", i)),
            PathBuf::from(format!("file_{}", i + 1)),
            DependencyType::Import,
        ));
    }

    // let detector = InvalidationDetector::new(graph);
    // let start = std::time::Instant::now();
    // let result = detector.compute_invalidation_set(&[PathBuf::from("file_500")]);
    // let duration = start.elapsed();

    // Verify O(V+E) complexity: should complete in < 10ms
    // assert!(duration.as_millis() < 10);
    // assert!(result.invalidated_files.len() > 500);
}

#[test]
fn test_performance_wide_fanout() {
    // One file with 100+ dependents
    let mut graph = DependencyGraph::new();
    for i in 0..100 {
        graph.add_edge(DependencyEdge::new(
            PathBuf::from(format!("dependent_{}", i)),
            PathBuf::from("core.rs"),
            DependencyType::Import,
        ));
    }

    // let detector = InvalidationDetector::new(graph);
    // let start = std::time::Instant::now();
    // let result = detector.compute_invalidation_set(&[PathBuf::from("core.rs")]);
    // let duration = start.elapsed();

    // Should handle wide fanout efficiently
    // assert!(duration.as_millis() < 5);
    // assert_eq!(result.invalidated_files.len(), 101); // core + 100 dependents
}

#[test]
fn test_performance_deep_chain() {
    // Deep chain: 100+ levels
    let mut graph = DependencyGraph::new();
    for i in 0..100 {
        graph.add_edge(DependencyEdge::new(
            PathBuf::from(format!("level_{}", i)),
            PathBuf::from(format!("level_{}", i + 1)),
            DependencyType::Import,
        ));
    }

    // let detector = InvalidationDetector::new(graph);
    // let start = std::time::Instant::now();
    // let result = detector.compute_invalidation_set(&[PathBuf::from("level_99")]);
    // let duration = start.elapsed();

    // Should handle deep chains without stack overflow
    // assert!(duration.as_millis() < 5);
    // assert_eq!(result.invalidated_files.len(), 100);
}

// ─── Real-World Scenarios ─────────────────────────────────────────────────────

#[test]
fn test_rust_module_tree() {
    let mut graph = DependencyGraph::new();
    // Typical Rust module structure:
    // main.rs -> lib.rs -> utils.rs, types.rs
    // lib.rs -> config.rs
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("src/main.rs"),
        PathBuf::from("src/lib.rs"),
        DependencyType::Import,
    ));
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("src/lib.rs"),
        PathBuf::from("src/utils.rs"),
        DependencyType::Import,
    ));
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("src/lib.rs"),
        PathBuf::from("src/types.rs"),
        DependencyType::Import,
    ));
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("src/lib.rs"),
        PathBuf::from("src/config.rs"),
        DependencyType::Import,
    ));

    // let detector = InvalidationDetector::new(graph);
    // let result = detector.compute_invalidation_set(&[PathBuf::from("src/utils.rs")]);

    // Changing utils.rs should invalidate lib.rs and main.rs
    // assert!(result.invalidated_files.contains(&PathBuf::from("src/main.rs")));
    // assert!(result.invalidated_files.contains(&PathBuf::from("src/lib.rs")));
    // assert!(result.invalidated_files.contains(&PathBuf::from("src/utils.rs")));
}

#[test]
fn test_typescript_barrel_exports() {
    let mut graph = DependencyGraph::new();
    // TypeScript barrel pattern: index.ts re-exports from multiple files
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("components/index.ts"),
        PathBuf::from("components/Button.tsx"),
        DependencyType::Export,
    ));
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("components/index.ts"),
        PathBuf::from("components/Input.tsx"),
        DependencyType::Export,
    ));
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("App.tsx"),
        PathBuf::from("components/index.ts"),
        DependencyType::Import,
    ));

    // let detector = InvalidationDetector::new(graph);
    // let result = detector.compute_invalidation_set(&[
    //     PathBuf::from("components/Button.tsx")
    // ]);

    // Weak Export dependency should NOT propagate to App.tsx
    // assert!(result.invalidated_files.contains(&PathBuf::from("components/Button.tsx")));
    // assert!(!result.invalidated_files.contains(&PathBuf::from("App.tsx")));
}

// ─── Edge Cases ───────────────────────────────────────────────────────────────

#[test]
fn test_self_loop_detection() {
    let mut graph = DependencyGraph::new();
    // Self-loop: A -> A
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("A"),
        PathBuf::from("A"),
        DependencyType::Import,
    ));

    // let detector = InvalidationDetector::new(graph);
    // let result = detector.compute_invalidation_set(&[PathBuf::from("A")]);

    // Should detect self-loop as a cycle
    // assert!(!result.circular_dependencies.is_empty());
}

#[test]
fn test_mixed_strong_weak_propagation() {
    let mut graph = DependencyGraph::new();
    // Complex: A -> B (Import), B -> C (Export), C -> D (Import)
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("A"),
        PathBuf::from("B"),
        DependencyType::Import,
    ));
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("B"),
        PathBuf::from("C"),
        DependencyType::Export,
    ));
    graph.add_edge(DependencyEdge::new(
        PathBuf::from("C"),
        PathBuf::from("D"),
        DependencyType::Import,
    ));

    // let detector = InvalidationDetector::new(graph);
    // let result = detector.compute_invalidation_set(&[PathBuf::from("D")]);

    // D changed -> C affected (strong Import)
    // C changed -> B NOT affected (weak Export)
    // assert!(result.invalidated_files.contains(&PathBuf::from("C")));
    // assert!(result.invalidated_files.contains(&PathBuf::from("D")));
    // assert!(!result.invalidated_files.contains(&PathBuf::from("B")));
}
