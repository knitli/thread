// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Graph building and manipulation utilities

use crate::*;
use thread_core::*;
use petgraph::Graph;
use std::collections::HashMap;

/// Graph builder for constructing code graphs from parsed elements
pub struct GraphBuilder {
    graph: Graph<CodeNode, CodeEdge>,
    node_index: HashMap<ElementId, petgraph::graph::NodeIndex>,
}

impl GraphBuilder {
    /// Create a new graph builder
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            node_index: HashMap::new(),
        }
    }

    /// Add a code element as a node in the graph
    pub fn add_element(&mut self, element: CodeElement) -> petgraph::graph::NodeIndex {
        let node = CodeNode { element: element.clone() };
        let index = self.graph.add_node(node);
        self.node_index.insert(element.id, index);
        index
    }

    /// Add a relationship between two elements
    pub fn add_relationship(
        &mut self,
        from: &ElementId,
        to: &ElementId,
        kind: EdgeKind,
        metadata: EdgeMetadata,
    ) -> Result<()> {
        let from_index = self.node_index.get(from)
            .ok_or_else(|| Error::ElementNotFound(from.clone()))?;
        let to_index = self.node_index.get(to)
            .ok_or_else(|| Error::ElementNotFound(to.clone()))?;

        let edge = CodeEdge { kind, metadata };
        self.graph.add_edge(*from_index, *to_index, edge);
        Ok(())
    }

    /// Build the final graph (consumes the builder)
    pub fn build(self) -> (Graph<CodeNode, CodeEdge>, HashMap<ElementId, petgraph::graph::NodeIndex>) {
        (self.graph, self.node_index)
    }

    /// Get current graph statistics
    pub fn stats(&self) -> GraphStats {
        GraphStats {
            node_count: self.graph.node_count(),
            edge_count: self.graph.edge_count(),
        }
    }
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about a code graph
#[derive(Debug, Clone)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
}