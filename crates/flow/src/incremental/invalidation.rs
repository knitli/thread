// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Invalidation detection and topological sorting for incremental updates.
//!
//! This module provides sophisticated invalidation detection that determines
//! which files require reanalysis after changes. It uses:
//!
//! - **BFS/DFS traversal** from [`DependencyGraph`] to find affected files
//! - **Topological sort** to order reanalysis respecting dependencies
//! - **Tarjan's SCC algorithm** to detect and report circular dependencies
//!
//! ## Design Pattern
//!
//! Wraps [`DependencyGraph`] with higher-level API that packages results
//! into [`InvalidationResult`] with comprehensive cycle detection.

use super::graph::{DependencyGraph, GraphError};
use metrics::histogram;
use std::path::{Path, PathBuf};
use std::time::Instant;
use thread_utilities::{RapidMap, RapidSet};
use tracing::{info, warn};

/// Errors that can occur during invalidation detection.
#[derive(Debug, thiserror::Error)]
pub enum InvalidationError {
    /// A circular dependency was detected during topological sort.
    #[error("Circular dependency detected: {0:?}")]
    CircularDependency(Vec<PathBuf>),

    /// An error occurred in the underlying dependency graph.
    #[error("Graph error: {0}")]
    Graph(String),
}

/// Result of invalidation detection, including cycle information.
///
/// This structure packages all information needed to perform incremental
/// reanalysis: which files are affected, what order to analyze them in,
/// and whether any circular dependencies were detected.
///
/// # Examples
///
/// ```rust
/// use thread_flow::incremental::invalidation::InvalidationDetector;
/// use thread_flow::incremental::DependencyGraph;
/// use thread_utilities::RapishSet;
/// use std::path::PathBuf;
///
/// let graph = DependencyGraph::new();
/// let detector = InvalidationDetector::new(graph);
/// let result = detector.compute_invalidation_set(&[PathBuf::from("main.rs")]);
///
/// if result.circular_dependencies.is_empty() {
///     // Safe to analyze in order
///     for file in &result.analysis_order {
///         // analyze(file);
///     }
/// } else {
///     // Handle cycles
///     eprintln!("Circular dependencies detected: {:?}", result.circular_dependencies);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct InvalidationResult {
    /// All files that require reanalysis (includes changed files).
    pub invalidated_files: Vec<PathBuf>,

    /// Files in topological order (dependencies before dependents).
    /// May be empty or partial if cycles are detected.
    pub analysis_order: Vec<PathBuf>,

    /// Strongly connected components representing circular dependencies.
    /// Each inner Vec contains files involved in a cycle.
    /// Empty if no cycles exist.
    pub circular_dependencies: Vec<Vec<PathBuf>>,
}

/// Detects invalidation scope and computes reanalysis order.
///
/// Wraps [`DependencyGraph`] to provide:
/// - Propagation of invalidation through dependency edges
/// - Topological sorting for correct reanalysis order
/// - Comprehensive cycle detection using Tarjan's algorithm
///
/// # Examples
///
/// ```rust
/// use thread_flow::incremental::invalidation::InvalidationDetector;
/// use thread_flow::incremental::DependencyGraph;
/// use thread_flow::incremental::types::{DependencyEdge, DependencyType};
/// use std::path::PathBuf;
///
/// let mut graph = DependencyGraph::new();
/// graph.add_edge(DependencyEdge::new(
///     PathBuf::from("main.rs"),
///     PathBuf::from("lib.rs"),
///     DependencyType::Import,
/// ));
///
/// let detector = InvalidationDetector::new(graph);
/// let result = detector.compute_invalidation_set(&[PathBuf::from("lib.rs")]);
///
/// assert!(result.invalidated_files.contains(&PathBuf::from("main.rs")));
/// ```
#[derive(Debug, Clone)]
pub struct InvalidationDetector {
    graph: DependencyGraph,
}

impl InvalidationDetector {
    /// Creates a new invalidation detector wrapping the given dependency graph.
    ///
    /// # Arguments
    ///
    /// * `graph` - The dependency graph to use for invalidation detection.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use thread_flow::incremental::invalidation::InvalidationDetector;
    /// use thread_flow::incremental::DependencyGraph;
    ///
    /// let graph = DependencyGraph::new();
    /// let detector = InvalidationDetector::new(graph);
    /// ```
    pub fn new(graph: DependencyGraph) -> Self {
        Self { graph }
    }

    /// Computes the complete invalidation set for the given changed files.
    ///
    /// This is the primary high-level API for invalidation detection. It:
    /// 1. Finds all files transitively affected by changes
    /// 2. Attempts topological sort for reanalysis order
    /// 3. Detects and reports any circular dependencies
    ///
    /// Always returns a result (never fails). If cycles are detected,
    /// they are reported in `circular_dependencies` and `analysis_order`
    /// may be empty or partial.
    ///
    /// # Arguments
    ///
    /// * `changed_files` - Files that have been modified or added.
    ///
    /// # Returns
    ///
    /// An [`InvalidationResult`] with:
    /// - All affected files
    /// - Topological order for reanalysis (if no cycles)
    /// - Detected circular dependencies (if any)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use thread_flow::incremental::invalidation::InvalidationDetector;
    /// use thread_flow::incremental::DependencyGraph;
    /// use std::path::PathBuf;
    ///
    /// let graph = DependencyGraph::new();
    /// let detector = InvalidationDetector::new(graph);
    ///
    /// let result = detector.compute_invalidation_set(&[
    ///     PathBuf::from("src/utils.rs"),
    /// ]);
    ///
    /// println!("Files to reanalyze: {}", result.invalidated_files.len());
    /// ```
    pub fn compute_invalidation_set(&self, changed_files: &[PathBuf]) -> InvalidationResult {
        let start = Instant::now();
        info!(
            "computing invalidation set for {} changed files",
            changed_files.len()
        );

        // Step 1: Find all files transitively affected by changes
        let changed_set: RapidSet<PathBuf> = changed_files.iter().cloned().collect();
        let affected = self.graph.find_affected_files(&changed_set);
        let invalidated_files: Vec<PathBuf> = affected.iter().cloned().collect();

        info!(
            "found {} files affected by changes",
            invalidated_files.len()
        );

        // Step 2: Attempt topological sort on affected files
        let result = match self.topological_sort(&invalidated_files) {
            Ok(analysis_order) => {
                // Success - no cycles detected
                info!("topological sort successful");
                InvalidationResult {
                    invalidated_files,
                    analysis_order,
                    circular_dependencies: vec![],
                }
            }
            Err(_) => {
                // Cycle detected - find all strongly connected components
                warn!("circular dependencies detected");
                let cycles = self.find_strongly_connected_components(&affected);

                // Try to provide partial ordering for acyclic parts
                // For now, return empty analysis_order when cycles exist
                InvalidationResult {
                    invalidated_files,
                    analysis_order: vec![],
                    circular_dependencies: cycles,
                }
            }
        };

        let duration_ms = start.elapsed().as_micros() as f64 / 1000.0;
        histogram!("invalidation_time_ms").record(duration_ms);

        info!(
            invalidated_count = result.invalidated_files.len(),
            cycles = result.circular_dependencies.len(),
            duration_ms = %format!("{:.2}", duration_ms),
            "invalidation complete"
        );

        result
    }

    /// Performs topological sort on the given subset of files.
    ///
    /// Returns files in dependency order: dependencies appear before
    /// their dependents. This is a lower-level API that directly exposes
    /// sort failures as errors.
    ///
    /// # Arguments
    ///
    /// * `files` - The subset of files to sort.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidationError::CircularDependency`] if a cycle is detected.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use thread_flow::incremental::invalidation::InvalidationDetector;
    /// use thread_flow::incremental::DependencyGraph;
    /// use std::path::PathBuf;
    ///
    /// let graph = DependencyGraph::new();
    /// let detector = InvalidationDetector::new(graph);
    ///
    /// let sorted = detector.topological_sort(&[
    ///     PathBuf::from("a.rs"),
    ///     PathBuf::from("b.rs"),
    /// ]);
    ///
    /// match sorted {
    ///     Ok(order) => println!("Analysis order: {:?}", order),
    ///     Err(e) => eprintln!("Cycle detected: {}", e),
    /// }
    /// ```
    pub fn topological_sort(&self, files: &[PathBuf]) -> Result<Vec<PathBuf>, InvalidationError> {
        // Delegate to DependencyGraph's topological sort and map errors
        let files_set: RapidSet<PathBuf> = files.iter().cloned().collect();

        self.graph
            .topological_sort(&files_set)
            .map_err(|e| match e {
                GraphError::CyclicDependency(path) => {
                    InvalidationError::CircularDependency(vec![path])
                }
            })
    }

    /// Propagates invalidation from a single root file.
    ///
    /// Finds all files transitively affected by changes to the given root.
    /// Uses BFS traversal following reverse dependency edges (dependents).
    ///
    /// # Arguments
    ///
    /// * `root` - The changed file to propagate from.
    ///
    /// # Returns
    ///
    /// All files affected by the change, including the root itself.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use thread_flow::incremental::invalidation::InvalidationDetector;
    /// use thread_flow::incremental::DependencyGraph;
    /// use std::path::PathBuf;
    ///
    /// let graph = DependencyGraph::new();
    /// let detector = InvalidationDetector::new(graph);
    ///
    /// let affected = detector.propagate_invalidation(&PathBuf::from("core.rs"));
    /// println!("Files affected: {}", affected.len());
    /// ```
    pub fn propagate_invalidation(&self, root: &Path) -> Vec<PathBuf> {
        // Delegate to DependencyGraph's find_affected_files for single root
        let root_set: RapidSet<PathBuf> = [root.to_path_buf()].into_iter().collect();
        let affected: RapidSet<PathBuf> = self.graph.find_affected_files(&root_set);
        affected.into_iter().collect()
    }

    // ── Private helpers ──────────────────────────────────────────────────

    /// Finds strongly connected components using Tarjan's algorithm.
    ///
    /// Returns all non-trivial SCCs (size > 1), which represent cycles.
    /// This is O(V + E) time complexity.
    ///
    /// # Arguments
    ///
    /// * `files` - The subset of files to analyze for cycles.
    ///
    /// # Returns
    ///
    /// Vector of strongly connected components, where each component
    /// is a vector of file paths involved in a cycle.
    fn find_strongly_connected_components(&self, files: &RapidSet<PathBuf>) -> Vec<Vec<PathBuf>> {
        // Tarjan's SCC algorithm for finding all cycles
        let mut state = TarjanState::new();
        let mut sccs = Vec::new();

        // Run DFS from each unvisited node
        for file in files {
            if !state.indices.contains_key(file) {
                self.tarjan_dfs(file, &mut state, &mut sccs);
            }
        }

        // Filter to non-trivial SCCs (cycles)
        sccs.into_iter()
            .filter(|scc| {
                // Include if size > 1, or size == 1 with self-loop
                scc.len() > 1 || (scc.len() == 1 && self.has_self_loop(&scc[0]))
            })
            .collect()
    }

    /// DFS helper for Tarjan's algorithm
    fn tarjan_dfs(&self, v: &Path, state: &mut TarjanState, sccs: &mut Vec<Vec<PathBuf>>) {
        // Initialize node
        let index = state.index_counter;
        state.indices.insert(v.to_path_buf(), index);
        state.lowlinks.insert(v.to_path_buf(), index);
        state.index_counter += 1;
        state.stack.push(v.to_path_buf());
        state.on_stack.insert(v.to_path_buf());

        // Visit all successors (dependencies)
        let dependencies = self.graph.get_dependencies(v);
        for edge in dependencies {
            let dep = &edge.to;
            if !state.indices.contains_key(dep) {
                // Successor not yet visited - recurse
                self.tarjan_dfs(dep, state, sccs);

                // Update lowlink
                let w_lowlink = *state.lowlinks.get(dep).unwrap();
                let v_lowlink = state.lowlinks.get_mut(&v.to_path_buf()).unwrap();
                *v_lowlink = (*v_lowlink).min(w_lowlink);
            } else if state.on_stack.contains(dep) {
                // Successor is on stack (part of current SCC)
                let w_index = *state.indices.get(dep).unwrap();
                let v_lowlink = state.lowlinks.get_mut(&v.to_path_buf()).unwrap();
                *v_lowlink = (*v_lowlink).min(w_index);
            }
        }

        // If v is a root node, pop the stack to create an SCC
        let v_index = *state.indices.get(&v.to_path_buf()).unwrap();
        let v_lowlink = *state.lowlinks.get(&v.to_path_buf()).unwrap();

        if v_lowlink == v_index {
            let mut scc = Vec::new();
            loop {
                let w = state.stack.pop().unwrap();
                state.on_stack.remove(&w);
                scc.push(w.clone());
                if w == v {
                    break;
                }
            }
            sccs.push(scc);
        }
    }

    /// Check if a file has a self-referential edge
    fn has_self_loop(&self, file: &Path) -> bool {
        let deps = self.graph.get_dependencies(file);
        deps.iter().any(|edge| edge.to == file)
    }
}

/// State for Tarjan's SCC algorithm
struct TarjanState {
    index_counter: usize,
    indices: RapidMap<PathBuf, usize>,
    lowlinks: RapidMap<PathBuf, usize>,
    stack: Vec<PathBuf>,
    on_stack: RapidSet<PathBuf>,
}

impl TarjanState {
    fn new() -> Self {
        Self {
            index_counter: 0,
            indices: thread_utilities::get_map(),
            lowlinks: thread_utilities::get_map(),
            stack: Vec::new(),
            on_stack: thread_utilities::get_set(),
        }
    }
}

// ─── Tests (TDD: Written BEFORE implementation) ──────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::incremental::types::{DependencyEdge, DependencyType};

    // ── Construction Tests ───────────────────────────────────────────────

    #[test]
    fn test_invalidation_detector_new() {
        let graph = DependencyGraph::new();
        let detector = InvalidationDetector::new(graph);

        // Verify detector is properly constructed
        assert_eq!(detector.graph.node_count(), 0);
        assert_eq!(detector.graph.edge_count(), 0);
    }

    #[test]
    fn test_invalidation_detector_with_populated_graph() {
        let mut graph = DependencyGraph::new();
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("A"),
            PathBuf::from("B"),
            DependencyType::Import,
        ));

        let detector = InvalidationDetector::new(graph);
        assert_eq!(detector.graph.node_count(), 2);
        assert_eq!(detector.graph.edge_count(), 1);
    }

    // ── propagate_invalidation Tests ─────────────────────────────────────

    #[test]
    fn test_propagate_single_file_no_dependents() {
        let mut graph = DependencyGraph::new();
        graph.add_node(&PathBuf::from("isolated.rs"));

        let detector = InvalidationDetector::new(graph);
        let affected = detector.propagate_invalidation(&PathBuf::from("isolated.rs"));

        assert_eq!(affected.len(), 1);
        assert_eq!(affected[0], PathBuf::from("isolated.rs"));
    }

    #[test]
    fn test_propagate_linear_chain() {
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

        let detector = InvalidationDetector::new(graph);
        let affected = detector.propagate_invalidation(&PathBuf::from("C"));

        // C changed -> B affected -> A affected
        assert_eq!(affected.len(), 3);
        assert!(affected.contains(&PathBuf::from("A")));
        assert!(affected.contains(&PathBuf::from("B")));
        assert!(affected.contains(&PathBuf::from("C")));
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

        let detector = InvalidationDetector::new(graph);
        let affected = detector.propagate_invalidation(&PathBuf::from("D"));

        // D changed -> B and C affected -> A affected
        assert_eq!(affected.len(), 4);
        assert!(affected.contains(&PathBuf::from("A")));
        assert!(affected.contains(&PathBuf::from("B")));
        assert!(affected.contains(&PathBuf::from("C")));
        assert!(affected.contains(&PathBuf::from("D")));
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

        let detector = InvalidationDetector::new(graph);
        let affected = detector.propagate_invalidation(&PathBuf::from("B"));

        // B changed -> A affected (strong), C NOT affected (weak)
        assert!(affected.contains(&PathBuf::from("A")));
        assert!(affected.contains(&PathBuf::from("B")));
        assert!(
            !affected.contains(&PathBuf::from("C")),
            "Weak dependencies should not propagate invalidation"
        );
    }

    #[test]
    fn test_propagate_stops_at_frontier() {
        let mut graph = DependencyGraph::new();
        // Two separate chains: A -> B, C -> D
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("A"),
            PathBuf::from("B"),
            DependencyType::Import,
        ));
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("C"),
            PathBuf::from("D"),
            DependencyType::Import,
        ));

        let detector = InvalidationDetector::new(graph);
        let affected = detector.propagate_invalidation(&PathBuf::from("B"));

        // B changed -> A affected, but C and D are independent
        assert_eq!(affected.len(), 2);
        assert!(affected.contains(&PathBuf::from("A")));
        assert!(affected.contains(&PathBuf::from("B")));
        assert!(!affected.contains(&PathBuf::from("C")));
        assert!(!affected.contains(&PathBuf::from("D")));
    }

    #[test]
    fn test_propagate_unknown_file() {
        let graph = DependencyGraph::new();
        let detector = InvalidationDetector::new(graph);
        let affected = detector.propagate_invalidation(&PathBuf::from("unknown.rs"));

        // Unknown file should still be included in result
        assert_eq!(affected.len(), 1);
        assert_eq!(affected[0], PathBuf::from("unknown.rs"));
    }

    // ── topological_sort Tests ───────────────────────────────────────────

    #[test]
    fn test_topological_sort_linear_chain() {
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

        let detector = InvalidationDetector::new(graph);
        let sorted = detector
            .topological_sort(&[PathBuf::from("A"), PathBuf::from("B"), PathBuf::from("C")])
            .unwrap();

        assert_eq!(sorted.len(), 3);

        // C must come before B, B before A (dependencies first)
        let pos_a = sorted
            .iter()
            .position(|p| p == &PathBuf::from("A"))
            .unwrap();
        let pos_b = sorted
            .iter()
            .position(|p| p == &PathBuf::from("B"))
            .unwrap();
        let pos_c = sorted
            .iter()
            .position(|p| p == &PathBuf::from("C"))
            .unwrap();

        assert!(pos_c < pos_b, "C must come before B");
        assert!(pos_b < pos_a, "B must come before A");
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

        let detector = InvalidationDetector::new(graph);
        let sorted = detector
            .topological_sort(&[
                PathBuf::from("A"),
                PathBuf::from("B"),
                PathBuf::from("C"),
                PathBuf::from("D"),
            ])
            .unwrap();

        assert_eq!(sorted.len(), 4);

        let pos_a = sorted
            .iter()
            .position(|p| p == &PathBuf::from("A"))
            .unwrap();
        let pos_b = sorted
            .iter()
            .position(|p| p == &PathBuf::from("B"))
            .unwrap();
        let pos_c = sorted
            .iter()
            .position(|p| p == &PathBuf::from("C"))
            .unwrap();
        let pos_d = sorted
            .iter()
            .position(|p| p == &PathBuf::from("D"))
            .unwrap();

        // D before B and C, B and C before A
        assert!(pos_d < pos_b, "D must come before B");
        assert!(pos_d < pos_c, "D must come before C");
        assert!(pos_b < pos_a, "B must come before A");
        assert!(pos_c < pos_a, "C must come before A");
    }

    #[test]
    fn test_topological_sort_disconnected_components() {
        let mut graph = DependencyGraph::new();
        // Two separate chains: A -> B, C -> D
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("A"),
            PathBuf::from("B"),
            DependencyType::Import,
        ));
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("C"),
            PathBuf::from("D"),
            DependencyType::Import,
        ));

        let detector = InvalidationDetector::new(graph);
        let sorted = detector
            .topological_sort(&[
                PathBuf::from("A"),
                PathBuf::from("B"),
                PathBuf::from("C"),
                PathBuf::from("D"),
            ])
            .unwrap();

        assert_eq!(sorted.len(), 4);

        // Verify local ordering within each component
        let pos_a = sorted
            .iter()
            .position(|p| p == &PathBuf::from("A"))
            .unwrap();
        let pos_b = sorted
            .iter()
            .position(|p| p == &PathBuf::from("B"))
            .unwrap();
        let pos_c = sorted
            .iter()
            .position(|p| p == &PathBuf::from("C"))
            .unwrap();
        let pos_d = sorted
            .iter()
            .position(|p| p == &PathBuf::from("D"))
            .unwrap();

        assert!(pos_b < pos_a, "B must come before A");
        assert!(pos_d < pos_c, "D must come before C");
    }

    #[test]
    fn test_topological_sort_single_file() {
        let graph = DependencyGraph::new();
        let detector = InvalidationDetector::new(graph);
        let sorted = detector
            .topological_sort(&[PathBuf::from("only.rs")])
            .unwrap();

        assert_eq!(sorted, vec![PathBuf::from("only.rs")]);
    }

    #[test]
    fn test_topological_sort_empty_set() {
        let graph = DependencyGraph::new();
        let detector = InvalidationDetector::new(graph);
        let sorted = detector.topological_sort(&[]).unwrap();

        assert!(sorted.is_empty());
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

        let detector = InvalidationDetector::new(graph);
        let result = detector.topological_sort(&[PathBuf::from("A"), PathBuf::from("B")]);

        assert!(result.is_err());
        match result.unwrap_err() {
            InvalidationError::CircularDependency(cycle) => {
                assert!(!cycle.is_empty(), "Cycle should contain file paths");
            }
            _ => panic!("Expected CircularDependency error"),
        }
    }

    #[test]
    fn test_topological_sort_self_loop() {
        let mut graph = DependencyGraph::new();
        // Self-loop: A -> A
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("A"),
            PathBuf::from("A"),
            DependencyType::Import,
        ));

        let detector = InvalidationDetector::new(graph);
        let result = detector.topological_sort(&[PathBuf::from("A")]);

        assert!(result.is_err());
        match result.unwrap_err() {
            InvalidationError::CircularDependency(_) => {
                // Expected
            }
            _ => panic!("Expected CircularDependency error"),
        }
    }

    // ── compute_invalidation_set Tests ───────────────────────────────────

    #[test]
    fn test_compute_invalidation_single_change() {
        let mut graph = DependencyGraph::new();
        // A -> B
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("A"),
            PathBuf::from("B"),
            DependencyType::Import,
        ));

        let detector = InvalidationDetector::new(graph);
        let result = detector.compute_invalidation_set(&[PathBuf::from("B")]);

        // B changed -> A affected
        assert_eq!(result.invalidated_files.len(), 2);
        assert!(result.invalidated_files.contains(&PathBuf::from("A")));
        assert!(result.invalidated_files.contains(&PathBuf::from("B")));

        // Should have valid analysis order
        assert_eq!(result.analysis_order.len(), 2);
        let pos_a = result
            .analysis_order
            .iter()
            .position(|p| p == &PathBuf::from("A"))
            .unwrap();
        let pos_b = result
            .analysis_order
            .iter()
            .position(|p| p == &PathBuf::from("B"))
            .unwrap();
        assert!(pos_b < pos_a, "B must come before A in analysis order");

        // No cycles
        assert!(result.circular_dependencies.is_empty());
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

        let detector = InvalidationDetector::new(graph);
        let result = detector.compute_invalidation_set(&[PathBuf::from("C")]);

        assert_eq!(result.invalidated_files.len(), 3);
        assert!(result.invalidated_files.contains(&PathBuf::from("A")));
        assert!(result.invalidated_files.contains(&PathBuf::from("B")));
        assert!(result.invalidated_files.contains(&PathBuf::from("C")));

        // Verify correct topological order: C, B, A
        assert_eq!(result.analysis_order.len(), 3);
        let pos_a = result
            .analysis_order
            .iter()
            .position(|p| p == &PathBuf::from("A"))
            .unwrap();
        let pos_b = result
            .analysis_order
            .iter()
            .position(|p| p == &PathBuf::from("B"))
            .unwrap();
        let pos_c = result
            .analysis_order
            .iter()
            .position(|p| p == &PathBuf::from("C"))
            .unwrap();
        assert!(pos_c < pos_b);
        assert!(pos_b < pos_a);

        assert!(result.circular_dependencies.is_empty());
    }

    #[test]
    fn test_compute_invalidation_multiple_changes() {
        let mut graph = DependencyGraph::new();
        // A -> C, B -> D (two independent chains)
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

        let detector = InvalidationDetector::new(graph);
        let result = detector.compute_invalidation_set(&[PathBuf::from("C"), PathBuf::from("D")]);

        assert_eq!(result.invalidated_files.len(), 4);
        assert!(result.invalidated_files.contains(&PathBuf::from("A")));
        assert!(result.invalidated_files.contains(&PathBuf::from("B")));
        assert!(result.invalidated_files.contains(&PathBuf::from("C")));
        assert!(result.invalidated_files.contains(&PathBuf::from("D")));

        assert!(result.circular_dependencies.is_empty());
    }

    #[test]
    fn test_compute_invalidation_empty_changes() {
        let graph = DependencyGraph::new();
        let detector = InvalidationDetector::new(graph);
        let result = detector.compute_invalidation_set(&[]);

        assert!(result.invalidated_files.is_empty());
        assert!(result.analysis_order.is_empty());
        assert!(result.circular_dependencies.is_empty());
    }

    #[test]
    fn test_compute_invalidation_unknown_files() {
        let graph = DependencyGraph::new();
        let detector = InvalidationDetector::new(graph);
        let result = detector.compute_invalidation_set(&[PathBuf::from("unknown.rs")]);

        // Unknown file should still be included
        assert_eq!(result.invalidated_files.len(), 1);
        assert!(
            result
                .invalidated_files
                .contains(&PathBuf::from("unknown.rs"))
        );
    }

    #[test]
    fn test_compute_invalidation_with_cycle() {
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

        let detector = InvalidationDetector::new(graph);
        let result = detector.compute_invalidation_set(&[PathBuf::from("A")]);

        // All files should be in invalidated set
        assert_eq!(result.invalidated_files.len(), 3);

        // Should detect the cycle between A and B
        assert!(!result.circular_dependencies.is_empty());
        assert!(
            result.circular_dependencies.iter().any(|cycle| {
                cycle.contains(&PathBuf::from("A")) && cycle.contains(&PathBuf::from("B"))
            }),
            "Should detect cycle involving A and B"
        );
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

        let detector = InvalidationDetector::new(graph);
        let result = detector.compute_invalidation_set(&[PathBuf::from("A"), PathBuf::from("C")]);

        // Should detect both cycles
        assert_eq!(result.circular_dependencies.len(), 2);
    }

    #[test]
    fn test_compute_invalidation_partial_cycle() {
        let mut graph = DependencyGraph::new();
        // Mixed: A -> B -> C -> B (cycle B-C), D -> A (independent)
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
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("C"),
            PathBuf::from("B"),
            DependencyType::Import,
        ));
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("D"),
            PathBuf::from("A"),
            DependencyType::Import,
        ));

        let detector = InvalidationDetector::new(graph);
        let result = detector.compute_invalidation_set(&[PathBuf::from("B")]);

        // Should detect cycle between B and C
        assert!(!result.circular_dependencies.is_empty());
        let cycle = &result.circular_dependencies[0];
        assert!(cycle.contains(&PathBuf::from("B")));
        assert!(cycle.contains(&PathBuf::from("C")));
        // A and D should not be in the cycle
        assert!(!cycle.contains(&PathBuf::from("A")));
        assert!(!cycle.contains(&PathBuf::from("D")));
    }

    // ── Tarjan's SCC Algorithm Tests ─────────────────────────────────────

    #[test]
    fn test_find_scc_no_cycles() {
        let mut graph = DependencyGraph::new();
        // Linear: A -> B -> C
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

        let detector = InvalidationDetector::new(graph);
        let files: RapidSet<PathBuf> = [PathBuf::from("A"), PathBuf::from("B"), PathBuf::from("C")]
            .into_iter()
            .collect();
        let sccs = detector.find_strongly_connected_components(&files);

        // No non-trivial SCCs (all components have size 1)
        assert!(sccs.is_empty());
    }

    #[test]
    fn test_find_scc_simple_cycle() {
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

        let detector = InvalidationDetector::new(graph);
        let files: RapidSet<PathBuf> = [PathBuf::from("A"), PathBuf::from("B")]
            .into_iter()
            .collect();
        let sccs = detector.find_strongly_connected_components(&files);

        assert_eq!(sccs.len(), 1);
        assert_eq!(sccs[0].len(), 2);
        assert!(sccs[0].contains(&PathBuf::from("A")));
        assert!(sccs[0].contains(&PathBuf::from("B")));
    }

    #[test]
    fn test_find_scc_self_loop() {
        let mut graph = DependencyGraph::new();
        // Self-loop: A -> A
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("A"),
            PathBuf::from("A"),
            DependencyType::Import,
        ));

        let detector = InvalidationDetector::new(graph);
        let files: RapidSet<PathBuf> = [PathBuf::from("A")].into_iter().collect();
        let sccs = detector.find_strongly_connected_components(&files);

        // Self-loop creates a non-trivial SCC of size 1
        assert_eq!(sccs.len(), 1);
        assert_eq!(sccs[0].len(), 1);
        assert_eq!(sccs[0][0], PathBuf::from("A"));
    }

    #[test]
    fn test_find_scc_multiple_cycles() {
        let mut graph = DependencyGraph::new();
        // Two cycles: A -> B -> A, C -> D -> C
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

        let detector = InvalidationDetector::new(graph);
        let files: RapidSet<PathBuf> = [
            PathBuf::from("A"),
            PathBuf::from("B"),
            PathBuf::from("C"),
            PathBuf::from("D"),
        ]
        .into_iter()
        .collect();
        let sccs = detector.find_strongly_connected_components(&files);

        assert_eq!(sccs.len(), 2);
    }

    #[test]
    fn test_find_scc_nested_components() {
        let mut graph = DependencyGraph::new();
        // Complex: A -> B -> C -> B (B-C cycle), A -> D
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
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("C"),
            PathBuf::from("B"),
            DependencyType::Import,
        ));
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("A"),
            PathBuf::from("D"),
            DependencyType::Import,
        ));

        let detector = InvalidationDetector::new(graph);
        let files: RapidSet<PathBuf> = [
            PathBuf::from("A"),
            PathBuf::from("B"),
            PathBuf::from("C"),
            PathBuf::from("D"),
        ]
        .into_iter()
        .collect();
        let sccs = detector.find_strongly_connected_components(&files);

        // Should find one SCC containing B and C
        assert_eq!(sccs.len(), 1);
        assert_eq!(sccs[0].len(), 2);
        assert!(sccs[0].contains(&PathBuf::from("B")));
        assert!(sccs[0].contains(&PathBuf::from("C")));
    }

    // ── Performance Tests ────────────────────────────────────────────────

    #[test]
    fn test_large_graph_performance() {
        // Build a graph with 1000 nodes in a chain
        let mut graph = DependencyGraph::new();
        for i in 0..999 {
            graph.add_edge(DependencyEdge::new(
                PathBuf::from(format!("file_{}", i)),
                PathBuf::from(format!("file_{}", i + 1)),
                DependencyType::Import,
            ));
        }

        let detector = InvalidationDetector::new(graph);
        let start = std::time::Instant::now();
        let result = detector.compute_invalidation_set(&[PathBuf::from("file_500")]);
        let duration = start.elapsed();

        // Should complete quickly with O(V+E) complexity
        assert!(
            duration.as_millis() < 50,
            "Large graph processing took {}ms (expected < 50ms)",
            duration.as_millis()
        );
        assert!(result.invalidated_files.len() >= 500);
    }

    #[test]
    fn test_wide_fanout_performance() {
        // One file with 100 dependents
        let mut graph = DependencyGraph::new();
        for i in 0..100 {
            graph.add_edge(DependencyEdge::new(
                PathBuf::from(format!("dependent_{}", i)),
                PathBuf::from("core.rs"),
                DependencyType::Import,
            ));
        }

        let detector = InvalidationDetector::new(graph);
        let start = std::time::Instant::now();
        let result = detector.compute_invalidation_set(&[PathBuf::from("core.rs")]);
        let duration = start.elapsed();

        assert!(duration.as_millis() < 10);
        assert_eq!(result.invalidated_files.len(), 101); // core + 100 dependents
    }
}
