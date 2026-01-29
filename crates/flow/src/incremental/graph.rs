// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Dependency graph construction and traversal algorithms.
//!
//! This module implements the dependency graph that tracks relationships
//! between files in the analyzed codebase. It provides:
//!
//! - **BFS traversal** for finding all files affected by a change
//! - **Topological sort** for ordering reanalysis to respect dependencies
//! - **Cycle detection** during topological sort
//! - **Bidirectional queries** for both dependencies and dependents
//!
//! ## Design Pattern
//!
//! Adapted from ReCoco's scope traversal (analyzer.rs:656-668) and
//! `is_op_scope_descendant` ancestor chain traversal.

use super::types::{AnalysisDefFingerprint, DependencyEdge, DependencyStrength};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::path::{Path, PathBuf};

/// Errors that can occur during dependency graph operations.
#[derive(Debug)]
pub enum GraphError {
    /// A cyclic dependency was detected during topological sort.
    CyclicDependency(PathBuf),
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphError::CyclicDependency(path) => write!(
                f,
                "Cyclic dependency detected involving file: {}\n\
                 Hint: Use `thread deps --cycles` to visualize the cycle",
                path.display()
            ),
        }
    }
}

impl std::error::Error for GraphError {}

/// A dependency graph tracking relationships between source files.
///
/// The graph is directed: edges point from dependent files to their
/// dependencies. For example, if `main.rs` imports `utils.rs`, there is
/// an edge from `main.rs` to `utils.rs`.
///
/// The graph maintains both forward (dependencies) and reverse (dependents)
/// adjacency lists for efficient bidirectional traversal.
///
/// # Examples
///
/// ```rust
/// use thread_flow::incremental::graph::DependencyGraph;
/// use thread_flow::incremental::types::{DependencyEdge, DependencyType};
/// use std::path::PathBuf;
/// use std::collections::HashSet;
///
/// let mut graph = DependencyGraph::new();
///
/// // main.rs depends on utils.rs
/// graph.add_edge(DependencyEdge::new(
///     PathBuf::from("main.rs"),
///     PathBuf::from("utils.rs"),
///     DependencyType::Import,
/// ));
///
/// // Find what main.rs depends on
/// let deps = graph.get_dependencies(&PathBuf::from("main.rs"));
/// assert_eq!(deps.len(), 1);
/// assert_eq!(deps[0].to, PathBuf::from("utils.rs"));
///
/// // Find what depends on utils.rs
/// let dependents = graph.get_dependents(&PathBuf::from("utils.rs"));
/// assert_eq!(dependents.len(), 1);
/// assert_eq!(dependents[0].from, PathBuf::from("main.rs"));
/// ```
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Fingerprint state for each tracked file.
    pub nodes: HashMap<PathBuf, AnalysisDefFingerprint>,

    /// All dependency edges in the graph.
    pub edges: Vec<DependencyEdge>,

    /// Forward adjacency: file -> files it depends on.
    forward_adj: HashMap<PathBuf, Vec<usize>>,

    /// Reverse adjacency: file -> files that depend on it.
    reverse_adj: HashMap<PathBuf, Vec<usize>>,
}

impl DependencyGraph {
    /// Creates a new empty dependency graph.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use thread_flow::incremental::graph::DependencyGraph;
    ///
    /// let graph = DependencyGraph::new();
    /// assert_eq!(graph.node_count(), 0);
    /// assert_eq!(graph.edge_count(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            forward_adj: HashMap::new(),
            reverse_adj: HashMap::new(),
        }
    }

    /// Adds a dependency edge to the graph.
    ///
    /// Both the source (`from`) and target (`to`) nodes are automatically
    /// registered if they do not already exist. Adjacency lists are updated
    /// for both forward and reverse lookups.
    ///
    /// # Arguments
    ///
    /// * `edge` - The dependency edge to add.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use thread_flow::incremental::graph::DependencyGraph;
    /// use thread_flow::incremental::types::{DependencyEdge, DependencyType};
    /// use std::path::PathBuf;
    ///
    /// let mut graph = DependencyGraph::new();
    /// graph.add_edge(DependencyEdge::new(
    ///     PathBuf::from("a.rs"),
    ///     PathBuf::from("b.rs"),
    ///     DependencyType::Import,
    /// ));
    /// assert_eq!(graph.edge_count(), 1);
    /// assert_eq!(graph.node_count(), 2);
    /// ```
    pub fn add_edge(&mut self, edge: DependencyEdge) {
        let idx = self.edges.len();

        // Ensure nodes exist
        self.ensure_node(&edge.from);
        self.ensure_node(&edge.to);

        // Update adjacency lists
        self.forward_adj
            .entry(edge.from.clone())
            .or_default()
            .push(idx);
        self.reverse_adj
            .entry(edge.to.clone())
            .or_default()
            .push(idx);

        self.edges.push(edge);
    }

    /// Returns all direct dependencies of a file (files it depends on).
    ///
    /// # Arguments
    ///
    /// * `file` - The file to query dependencies for.
    ///
    /// # Returns
    ///
    /// A vector of references to dependency edges where `from` is the given file.
    pub fn get_dependencies(&self, file: &Path) -> Vec<&DependencyEdge> {
        self.forward_adj
            .get(file)
            .map(|indices| indices.iter().map(|&i| &self.edges[i]).collect())
            .unwrap_or_default()
    }

    /// Returns all direct dependents of a file (files that depend on it).
    ///
    /// # Arguments
    ///
    /// * `file` - The file to query dependents for.
    ///
    /// # Returns
    ///
    /// A vector of references to dependency edges where `to` is the given file.
    pub fn get_dependents(&self, file: &Path) -> Vec<&DependencyEdge> {
        self.reverse_adj
            .get(file)
            .map(|indices| indices.iter().map(|&i| &self.edges[i]).collect())
            .unwrap_or_default()
    }

    /// Finds all files affected by changes to the given set of files.
    ///
    /// Uses BFS traversal following reverse dependency edges (dependents)
    /// to discover the full set of files that need reanalysis. Only
    /// [`DependencyStrength::Strong`] edges trigger cascading invalidation.
    ///
    /// **Algorithm complexity**: O(V + E) where V = files, E = dependency edges.
    ///
    /// # Arguments
    ///
    /// * `changed_files` - Set of files that have been modified.
    ///
    /// # Returns
    ///
    /// Set of all affected files, including the changed files themselves.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use thread_flow::incremental::graph::DependencyGraph;
    /// use thread_flow::incremental::types::{DependencyEdge, DependencyType};
    /// use std::path::PathBuf;
    /// use std::collections::HashSet;
    ///
    /// let mut graph = DependencyGraph::new();
    ///
    /// // A -> B -> C (A depends on B, B depends on C)
    /// graph.add_edge(DependencyEdge::new(
    ///     PathBuf::from("A"), PathBuf::from("B"), DependencyType::Import,
    /// ));
    /// graph.add_edge(DependencyEdge::new(
    ///     PathBuf::from("B"), PathBuf::from("C"), DependencyType::Import,
    /// ));
    ///
    /// // Change C -> affects B and A
    /// let changed = HashSet::from([PathBuf::from("C")]);
    /// let affected = graph.find_affected_files(&changed);
    /// assert!(affected.contains(&PathBuf::from("A")));
    /// assert!(affected.contains(&PathBuf::from("B")));
    /// assert!(affected.contains(&PathBuf::from("C")));
    /// ```
    pub fn find_affected_files(&self, changed_files: &HashSet<PathBuf>) -> HashSet<PathBuf> {
        let mut affected = HashSet::new();
        let mut visited = HashSet::new();
        let mut queue: VecDeque<PathBuf> = changed_files.iter().cloned().collect();

        while let Some(file) = queue.pop_front() {
            if !visited.insert(file.clone()) {
                continue;
            }

            affected.insert(file.clone());

            // Follow reverse edges (files that depend on this file)
            for edge in self.get_dependents(&file) {
                if edge.effective_strength() == DependencyStrength::Strong {
                    queue.push_back(edge.from.clone());
                }
            }
        }

        affected
    }

    /// Performs topological sort on the given subset of files.
    ///
    /// Returns files in dependency order: dependencies appear before
    /// their dependents. This ordering ensures correct incremental
    /// reanalysis.
    ///
    /// Detects cyclic dependencies and returns [`GraphError::CyclicDependency`]
    /// if a cycle is found.
    ///
    /// **Algorithm complexity**: O(V + E) using DFS.
    ///
    /// # Arguments
    ///
    /// * `files` - The subset of files to sort.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError::CyclicDependency`] if a cycle is detected.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use thread_flow::incremental::graph::DependencyGraph;
    /// use thread_flow::incremental::types::{DependencyEdge, DependencyType};
    /// use std::path::PathBuf;
    /// use std::collections::HashSet;
    ///
    /// let mut graph = DependencyGraph::new();
    /// // A depends on B, B depends on C
    /// graph.add_edge(DependencyEdge::new(
    ///     PathBuf::from("A"), PathBuf::from("B"), DependencyType::Import,
    /// ));
    /// graph.add_edge(DependencyEdge::new(
    ///     PathBuf::from("B"), PathBuf::from("C"), DependencyType::Import,
    /// ));
    ///
    /// let files = HashSet::from([
    ///     PathBuf::from("A"), PathBuf::from("B"), PathBuf::from("C"),
    /// ]);
    /// let sorted = graph.topological_sort(&files).unwrap();
    /// // C should come before B, B before A
    /// let pos_a = sorted.iter().position(|p| p == &PathBuf::from("A")).unwrap();
    /// let pos_b = sorted.iter().position(|p| p == &PathBuf::from("B")).unwrap();
    /// let pos_c = sorted.iter().position(|p| p == &PathBuf::from("C")).unwrap();
    /// assert!(pos_c < pos_b);
    /// assert!(pos_b < pos_a);
    /// ```
    pub fn topological_sort(&self, files: &HashSet<PathBuf>) -> Result<Vec<PathBuf>, GraphError> {
        let mut sorted = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_mark = HashSet::new();

        for file in files {
            if !visited.contains(file) {
                self.visit_node(file, files, &mut visited, &mut temp_mark, &mut sorted)?;
            }
        }

        // DFS post-order naturally produces dependency-first ordering:
        // dependencies are pushed before their dependents.
        Ok(sorted)
    }

    /// Returns the number of nodes (files) in the graph.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Checks whether the graph contains a node for the given file.
    pub fn contains_node(&self, file: &Path) -> bool {
        self.nodes.contains_key(file)
    }

    /// Validates graph integrity.
    ///
    /// Checks for dangling edges (edges referencing nodes not in the graph)
    /// and other structural issues.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the graph is structurally valid, or a [`GraphError`] otherwise.
    pub fn validate(&self) -> Result<(), GraphError> {
        for edge in &self.edges {
            if !self.nodes.contains_key(&edge.from) {
                return Err(GraphError::CyclicDependency(edge.from.clone()));
            }
            if !self.nodes.contains_key(&edge.to) {
                return Err(GraphError::CyclicDependency(edge.to.clone()));
            }
        }
        Ok(())
    }

    /// Removes all edges and nodes from the graph.
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
        self.forward_adj.clear();
        self.reverse_adj.clear();
    }

    // ── Private helpers ──────────────────────────────────────────────────

    /// Ensures a node exists in the graph for the given file path.
    /// Creates a default fingerprint entry if the node does not exist.
    fn ensure_node(&mut self, file: &Path) {
        self.nodes
            .entry(file.to_path_buf())
            .or_insert_with(|| AnalysisDefFingerprint::new(b""));
    }

    /// DFS visit for topological sort with cycle detection.
    fn visit_node(
        &self,
        file: &Path,
        subset: &HashSet<PathBuf>,
        visited: &mut HashSet<PathBuf>,
        temp_mark: &mut HashSet<PathBuf>,
        sorted: &mut Vec<PathBuf>,
    ) -> Result<(), GraphError> {
        let file_buf = file.to_path_buf();

        if temp_mark.contains(&file_buf) {
            return Err(GraphError::CyclicDependency(file_buf));
        }

        if visited.contains(&file_buf) {
            return Ok(());
        }

        temp_mark.insert(file_buf.clone());

        // Visit dependencies (forward edges) that are in our subset
        for edge in self.get_dependencies(file) {
            if subset.contains(&edge.to) {
                self.visit_node(&edge.to, subset, visited, temp_mark, sorted)?;
            }
        }

        temp_mark.remove(&file_buf);
        visited.insert(file_buf.clone());
        sorted.push(file_buf);

        Ok(())
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests (TDD: Written BEFORE implementation) ──────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::incremental::types::DependencyType;

    // ── Construction Tests ───────────────────────────────────────────────

    #[test]
    fn test_graph_new_is_empty() {
        let graph = DependencyGraph::new();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_graph_default_is_empty() {
        let graph = DependencyGraph::default();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_graph_add_edge_creates_nodes() {
        let mut graph = DependencyGraph::new();
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
            DependencyType::Import,
        ));

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
        assert!(graph.contains_node(Path::new("a.rs")));
        assert!(graph.contains_node(Path::new("b.rs")));
    }

    #[test]
    fn test_graph_add_multiple_edges() {
        let mut graph = DependencyGraph::new();
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
            DependencyType::Import,
        ));
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("a.rs"),
            PathBuf::from("c.rs"),
            DependencyType::Import,
        ));
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("b.rs"),
            PathBuf::from("c.rs"),
            DependencyType::Import,
        ));

        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 3);
    }

    #[test]
    fn test_graph_add_edge_no_duplicate_nodes() {
        let mut graph = DependencyGraph::new();
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
            DependencyType::Import,
        ));
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("a.rs"),
            PathBuf::from("c.rs"),
            DependencyType::Import,
        ));

        // "a.rs" appears in two edges but should only be one node
        assert_eq!(graph.node_count(), 3);
    }

    // ── get_dependencies Tests ───────────────────────────────────────────

    #[test]
    fn test_get_dependencies_empty_graph() {
        let graph = DependencyGraph::new();
        let deps = graph.get_dependencies(Path::new("nonexistent.rs"));
        assert!(deps.is_empty());
    }

    #[test]
    fn test_get_dependencies_returns_forward_edges() {
        let mut graph = DependencyGraph::new();
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("main.rs"),
            PathBuf::from("utils.rs"),
            DependencyType::Import,
        ));
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("main.rs"),
            PathBuf::from("config.rs"),
            DependencyType::Import,
        ));

        let deps = graph.get_dependencies(Path::new("main.rs"));
        assert_eq!(deps.len(), 2);

        let dep_targets: HashSet<_> = deps.iter().map(|e| &e.to).collect();
        assert!(dep_targets.contains(&PathBuf::from("utils.rs")));
        assert!(dep_targets.contains(&PathBuf::from("config.rs")));
    }

    #[test]
    fn test_get_dependencies_leaf_node_has_none() {
        let mut graph = DependencyGraph::new();
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("main.rs"),
            PathBuf::from("utils.rs"),
            DependencyType::Import,
        ));

        // utils.rs is a leaf - no outgoing edges
        let deps = graph.get_dependencies(Path::new("utils.rs"));
        assert!(deps.is_empty());
    }

    // ── get_dependents Tests ─────────────────────────────────────────────

    #[test]
    fn test_get_dependents_empty_graph() {
        let graph = DependencyGraph::new();
        let deps = graph.get_dependents(Path::new("nonexistent.rs"));
        assert!(deps.is_empty());
    }

    #[test]
    fn test_get_dependents_returns_reverse_edges() {
        let mut graph = DependencyGraph::new();
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("main.rs"),
            PathBuf::from("utils.rs"),
            DependencyType::Import,
        ));
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("lib.rs"),
            PathBuf::from("utils.rs"),
            DependencyType::Import,
        ));

        let dependents = graph.get_dependents(Path::new("utils.rs"));
        assert_eq!(dependents.len(), 2);

        let dependent_sources: HashSet<_> = dependents.iter().map(|e| &e.from).collect();
        assert!(dependent_sources.contains(&PathBuf::from("main.rs")));
        assert!(dependent_sources.contains(&PathBuf::from("lib.rs")));
    }

    #[test]
    fn test_get_dependents_root_node_has_none() {
        let mut graph = DependencyGraph::new();
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("main.rs"),
            PathBuf::from("utils.rs"),
            DependencyType::Import,
        ));

        // main.rs is a root - nothing depends on it
        let dependents = graph.get_dependents(Path::new("main.rs"));
        assert!(dependents.is_empty());
    }

    // ── find_affected_files Tests ────────────────────────────────────────

    #[test]
    fn test_find_affected_files_single_change() {
        let mut graph = DependencyGraph::new();

        // main.rs -> utils.rs
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("main.rs"),
            PathBuf::from("utils.rs"),
            DependencyType::Import,
        ));

        let changed = HashSet::from([PathBuf::from("utils.rs")]);
        let affected = graph.find_affected_files(&changed);

        assert!(affected.contains(&PathBuf::from("utils.rs")));
        assert!(affected.contains(&PathBuf::from("main.rs")));
        assert_eq!(affected.len(), 2);
    }

    #[test]
    fn test_find_affected_files_transitive() {
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

        let changed = HashSet::from([PathBuf::from("C")]);
        let affected = graph.find_affected_files(&changed);

        assert_eq!(affected.len(), 3);
        assert!(affected.contains(&PathBuf::from("A")));
        assert!(affected.contains(&PathBuf::from("B")));
        assert!(affected.contains(&PathBuf::from("C")));
    }

    #[test]
    fn test_find_affected_files_diamond_dependency() {
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

        let changed = HashSet::from([PathBuf::from("D")]);
        let affected = graph.find_affected_files(&changed);

        assert_eq!(affected.len(), 4);
        assert!(affected.contains(&PathBuf::from("A")));
        assert!(affected.contains(&PathBuf::from("B")));
        assert!(affected.contains(&PathBuf::from("C")));
        assert!(affected.contains(&PathBuf::from("D")));
    }

    #[test]
    fn test_find_affected_files_isolated_node() {
        let mut graph = DependencyGraph::new();

        // A -> B, C is isolated
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("A"),
            PathBuf::from("B"),
            DependencyType::Import,
        ));
        // Add C as an isolated node
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("C"),
            PathBuf::from("D"),
            DependencyType::Import,
        ));

        let changed = HashSet::from([PathBuf::from("B")]);
        let affected = graph.find_affected_files(&changed);

        assert!(affected.contains(&PathBuf::from("A")));
        assert!(affected.contains(&PathBuf::from("B")));
        assert!(!affected.contains(&PathBuf::from("C")));
        assert!(!affected.contains(&PathBuf::from("D")));
    }

    #[test]
    fn test_find_affected_files_weak_dependency_not_followed() {
        let mut graph = DependencyGraph::new();

        // A -> B (strong import), C -> B (weak export)
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

        let changed = HashSet::from([PathBuf::from("B")]);
        let affected = graph.find_affected_files(&changed);

        assert!(affected.contains(&PathBuf::from("A")));
        assert!(affected.contains(&PathBuf::from("B")));
        // C has a weak (Export) dependency on B, should NOT be affected
        assert!(
            !affected.contains(&PathBuf::from("C")),
            "Weak dependencies should not propagate invalidation"
        );
    }

    #[test]
    fn test_find_affected_files_empty_changed_set() {
        let mut graph = DependencyGraph::new();
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("A"),
            PathBuf::from("B"),
            DependencyType::Import,
        ));

        let changed = HashSet::new();
        let affected = graph.find_affected_files(&changed);
        assert!(affected.is_empty());
    }

    #[test]
    fn test_find_affected_files_unknown_file() {
        let graph = DependencyGraph::new();
        let changed = HashSet::from([PathBuf::from("nonexistent.rs")]);
        let affected = graph.find_affected_files(&changed);

        // The changed file itself is always included
        assert_eq!(affected.len(), 1);
        assert!(affected.contains(&PathBuf::from("nonexistent.rs")));
    }

    #[test]
    fn test_find_affected_files_multiple_changes() {
        let mut graph = DependencyGraph::new();

        // A -> C, B -> C (both A and B depend on C independently)
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

        let changed = HashSet::from([PathBuf::from("C"), PathBuf::from("D")]);
        let affected = graph.find_affected_files(&changed);

        assert_eq!(affected.len(), 4);
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

        let files = HashSet::from([PathBuf::from("A"), PathBuf::from("B"), PathBuf::from("C")]);

        let sorted = graph.topological_sort(&files).unwrap();
        assert_eq!(sorted.len(), 3);

        let pos_a = sorted.iter().position(|p| p == Path::new("A")).unwrap();
        let pos_b = sorted.iter().position(|p| p == Path::new("B")).unwrap();
        let pos_c = sorted.iter().position(|p| p == Path::new("C")).unwrap();

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

        let files = HashSet::from([
            PathBuf::from("A"),
            PathBuf::from("B"),
            PathBuf::from("C"),
            PathBuf::from("D"),
        ]);

        let sorted = graph.topological_sort(&files).unwrap();
        assert_eq!(sorted.len(), 4);

        let pos_a = sorted.iter().position(|p| p == Path::new("A")).unwrap();
        let pos_b = sorted.iter().position(|p| p == Path::new("B")).unwrap();
        let pos_c = sorted.iter().position(|p| p == Path::new("C")).unwrap();
        let pos_d = sorted.iter().position(|p| p == Path::new("D")).unwrap();

        // D must come before B and C; B and C must come before A
        assert!(pos_d < pos_b);
        assert!(pos_d < pos_c);
        assert!(pos_b < pos_a);
        assert!(pos_c < pos_a);
    }

    #[test]
    fn test_topological_sort_disconnected() {
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

        let files = HashSet::from([
            PathBuf::from("A"),
            PathBuf::from("B"),
            PathBuf::from("C"),
            PathBuf::from("D"),
        ]);

        let sorted = graph.topological_sort(&files).unwrap();
        assert_eq!(sorted.len(), 4);

        // Verify local ordering within each chain
        let pos_a = sorted.iter().position(|p| p == Path::new("A")).unwrap();
        let pos_b = sorted.iter().position(|p| p == Path::new("B")).unwrap();
        let pos_c = sorted.iter().position(|p| p == Path::new("C")).unwrap();
        let pos_d = sorted.iter().position(|p| p == Path::new("D")).unwrap();

        assert!(pos_b < pos_a);
        assert!(pos_d < pos_c);
    }

    #[test]
    fn test_topological_sort_single_node() {
        let graph = DependencyGraph::new();
        let files = HashSet::from([PathBuf::from("only.rs")]);

        let sorted = graph.topological_sort(&files).unwrap();
        assert_eq!(sorted, vec![PathBuf::from("only.rs")]);
    }

    #[test]
    fn test_topological_sort_empty_set() {
        let graph = DependencyGraph::new();
        let files = HashSet::new();

        let sorted = graph.topological_sort(&files).unwrap();
        assert!(sorted.is_empty());
    }

    #[test]
    fn test_topological_sort_subset_of_graph() {
        let mut graph = DependencyGraph::new();

        // Full graph: A -> B -> C -> D
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
            PathBuf::from("D"),
            DependencyType::Import,
        ));

        // Sort only A and B
        let files = HashSet::from([PathBuf::from("A"), PathBuf::from("B")]);

        let sorted = graph.topological_sort(&files).unwrap();
        assert_eq!(sorted.len(), 2);

        let pos_a = sorted.iter().position(|p| p == Path::new("A")).unwrap();
        let pos_b = sorted.iter().position(|p| p == Path::new("B")).unwrap();
        assert!(pos_b < pos_a);
    }

    // ── Cycle Detection Tests ────────────────────────────────────────────

    #[test]
    fn test_topological_sort_detects_simple_cycle() {
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

        let files = HashSet::from([PathBuf::from("A"), PathBuf::from("B")]);
        let result = graph.topological_sort(&files);

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            GraphError::CyclicDependency(path) => {
                assert!(
                    path == PathBuf::from("A") || path == PathBuf::from("B"),
                    "Cycle should involve A or B, got: {}",
                    path.display()
                );
            }
        }
    }

    #[test]
    fn test_topological_sort_detects_longer_cycle() {
        let mut graph = DependencyGraph::new();

        // Cycle: A -> B -> C -> A
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
            PathBuf::from("A"),
            DependencyType::Import,
        ));

        let files = HashSet::from([PathBuf::from("A"), PathBuf::from("B"), PathBuf::from("C")]);
        let result = graph.topological_sort(&files);
        assert!(result.is_err());
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

        let files = HashSet::from([PathBuf::from("A")]);
        let result = graph.topological_sort(&files);
        assert!(result.is_err());
    }

    // ── Validation Tests ─────────────────────────────────────────────────

    #[test]
    fn test_validate_valid_graph() {
        let mut graph = DependencyGraph::new();
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
            DependencyType::Import,
        ));

        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_graph() {
        let graph = DependencyGraph::new();
        assert!(graph.validate().is_ok());
    }

    // ── Clear Tests ──────────────────────────────────────────────────────

    #[test]
    fn test_graph_clear() {
        let mut graph = DependencyGraph::new();
        graph.add_edge(DependencyEdge::new(
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
            DependencyType::Import,
        ));

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);

        graph.clear();

        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    // ── GraphError Display Tests ─────────────────────────────────────────

    #[test]
    fn test_graph_error_display() {
        let err = GraphError::CyclicDependency(PathBuf::from("src/module.rs"));
        let display = format!("{}", err);
        assert!(display.contains("src/module.rs"));
        assert!(display.contains("Cyclic dependency"));
    }

    #[test]
    fn test_graph_error_is_std_error() {
        let err = GraphError::CyclicDependency(PathBuf::from("a.rs"));
        // Verify it implements std::error::Error
        let _: &dyn std::error::Error = &err;
    }
}
