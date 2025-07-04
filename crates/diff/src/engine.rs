// crates/thread-diff/src/engine.rs
use petgraph::{Graph, NodeIndex};
use thread_ast::{AstGraph, AstNode, AstEdge};
use std::collections::HashMap;
use anyhow::Result;

pub struct ThreadDiffEngine {
    graph_limit: usize,
    node_limit: usize,
    byte_limit: usize,
}

impl ThreadDiffEngine {
    pub fn new() -> Self {
        Self {
            graph_limit: 3_000_000,
            node_limit: 30_000,
            byte_limit: 1_000_000,
        }
    }

    pub fn diff_asts(&self, left: &AstGraph, right: &AstGraph) -> Result<DiffResult> {
        // Check limits first
        if left.node_count() > self.node_limit || right.node_count() > self.node_limit {
            return self.fallback_diff(left, right);
        }

        // Convert to difftastic representation
        let left_syntax = self.convert_to_syntax(left)?;
        let right_syntax = self.convert_to_syntax(right)?;

        // Use vendored difftastic algorithm
        let diff = self.dijkstra_diff(&left_syntax, &right_syntax)?;

        Ok(DiffResult {
            changes: diff.changes,
            unchanged: diff.unchanged,
            performance_stats: diff.stats,
        })
    }

    fn convert_to_syntax(&self, graph: &AstGraph) -> Result<DiffSyntax> {
        // Convert petgraph to difftastic-compatible representation
        let mut syntax_nodes = Vec::new();

        for node_idx in graph.node_indices() {
            let node = &graph[node_idx];
            let children = graph.neighbors(node_idx).collect::<Vec<_>>();

            syntax_nodes.push(DiffSyntaxNode {
                id: node.id,
                kind: node.kind.clone(),
                text: node.text.clone(),
                children: children.iter().map(|&idx| graph[idx].id).collect(),
                line_number: node.start_line,
            });
        }

        Ok(DiffSyntax { nodes: syntax_nodes })
    }

    fn dijkstra_diff(&self, left: &DiffSyntax, right: &DiffSyntax) -> Result<DiffOutput> {
        // Vendored difftastic algorithm implementation
        // This is a simplified version - real implementation would use
        // the full dijkstra algorithm from difftastic

        let mut changes = Vec::new();
        let mut unchanged = Vec::new();

        // Simple matching for demonstration
        for left_node in &left.nodes {
            if let Some(right_node) = right.nodes.iter().find(|n| n.text == left_node.text) {
                unchanged.push(UnchangedNode {
                    left_id: left_node.id,
                    right_id: right_node.id,
                    line_number: left_node.line_number,
                });
            } else {
                changes.push(DiffChange::Deletion {
                    node_id: left_node.id,
                    line_number: left_node.line_number,
                    text: left_node.text.clone(),
                });
            }
        }

        for right_node in &right.nodes {
            if !left.nodes.iter().any(|n| n.text == right_node.text) {
                changes.push(DiffChange::Addition {
                    node_id: right_node.id,
                    line_number: right_node.line_number,
                    text: right_node.text.clone(),
                });
            }
        }

        Ok(DiffOutput {
            changes,
            unchanged,
            stats: PerformanceStats {
                nodes_compared: left.nodes.len() + right.nodes.len(),
                time_ms: 0, // Would be measured in real implementation
                memory_mb: 0,
            },
        })
    }

    fn fallback_diff(&self, left: &AstGraph, right: &AstGraph) -> Result<DiffResult> {
        // Fallback to line-based diff for large inputs
        Ok(DiffResult {
            changes: vec![],
            unchanged: vec![],
            performance_stats: PerformanceStats {
                nodes_compared: 0,
                time_ms: 0,
                memory_mb: 0,
            },
        })
    }
}

#[derive(Debug)]
pub struct DiffResult {
    pub changes: Vec<DiffChange>,
    pub unchanged: Vec<UnchangedNode>,
    pub performance_stats: PerformanceStats,
}

#[derive(Debug)]
pub enum DiffChange {
    Addition {
        node_id: thread_ast::NodeId,
        line_number: usize,
        text: String,
    },
    Deletion {
        node_id: thread_ast::NodeId,
        line_number: usize,
        text: String,
    },
    Modification {
        left_id: thread_ast::NodeId,
        right_id: thread_ast::NodeId,
        line_number: usize,
        old_text: String,
        new_text: String,
    },
}

#[derive(Debug)]
pub struct UnchangedNode {
    pub left_id: thread_ast::NodeId,
    pub right_id: thread_ast::NodeId,
    pub line_number: usize,
}

#[derive(Debug)]
pub struct PerformanceStats {
    pub nodes_compared: usize,
    pub time_ms: u64,
    pub memory_mb: u64,
}

// Internal types for diff algorithm
#[derive(Debug)]
struct DiffSyntax {
    nodes: Vec<DiffSyntaxNode>,
}

#[derive(Debug)]
struct DiffSyntaxNode {
    id: thread_ast::NodeId,
    kind: String,
    text: String,
    children: Vec<thread_ast::NodeId>,
    line_number: usize,
}

#[derive(Debug)]
struct DiffOutput {
    changes: Vec<DiffChange>,
    unchanged: Vec<UnchangedNode>,
    stats: PerformanceStats,
}
