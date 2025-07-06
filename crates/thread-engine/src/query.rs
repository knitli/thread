//! Graph query operations for extracting context

use crate::*;
use thread_core::*;
use petgraph::Graph;
use std::collections::HashMap;

/// Query engine for extracting context from code graphs
pub struct QueryEngine<'a> {
    graph: &'a Graph<CodeNode, CodeEdge>,
    node_index: &'a HashMap<ElementId, petgraph::graph::NodeIndex>,
}

impl<'a> QueryEngine<'a> {
    /// Create a new query engine
    pub fn new(
        graph: &'a Graph<CodeNode, CodeEdge>,
        node_index: &'a HashMap<ElementId, petgraph::graph::NodeIndex>,
    ) -> Self {
        Self { graph, node_index }
    }

    /// Find a function by name
    pub fn find_function(&self, name: &str) -> Vec<&CodeElement> {
        self.graph
            .node_weights()
            .filter(|node| {
                matches!(node.element.kind, ElementKind::Function | ElementKind::Method)
                    && node.element.name == name
            })
            .map(|node| &node.element)
            .collect()
    }

    /// Get all functions that call the given function
    pub fn get_callers(&self, function_id: &ElementId) -> Result<Vec<&CodeElement>> {
        let target_index = self.node_index.get(function_id)
            .ok_or_else(|| Error::ElementNotFound(function_id.clone()))?;

        let callers = self.graph
            .neighbors_directed(*target_index, petgraph::Direction::Incoming)
            .filter_map(|idx| self.graph.node_weight(idx))
            .map(|node| &node.element)
            .collect();

        Ok(callers)
    }

    /// Get all functions called by the given function
    pub fn get_callees(&self, function_id: &ElementId) -> Result<Vec<&CodeElement>> {
        let source_index = self.node_index.get(function_id)
            .ok_or_else(|| Error::ElementNotFound(function_id.clone()))?;

        let callees = self.graph
            .neighbors_directed(*source_index, petgraph::Direction::Outgoing)
            .filter_map(|idx| self.graph.node_weight(idx))
            .map(|node| &node.element)
            .collect();

        Ok(callees)
    }

    /// Generate AI-friendly context for a function
    pub fn generate_context(&self, function_name: &str) -> Result<FunctionContext> {
        let functions = self.find_function(function_name);
        if functions.is_empty() {
            return Err(Error::FunctionNotFound(function_name.to_string()));
        }

        // For now, take the first match (TODO: handle multiple matches)
        let function = functions[0];
        let callers = self.get_callers(&function.id)?;
        let callees = self.get_callees(&function.id)?;

        Ok(FunctionContext {
            function: function.clone(),
            callers: callers.into_iter().cloned().collect(),
            callees: callees.into_iter().cloned().collect(),
        })
    }
}

/// Context information for a function (AI-friendly format)
#[derive(Debug, Clone)]
pub struct FunctionContext {
    pub function: CodeElement,
    pub callers: Vec<CodeElement>,
    pub callees: Vec<CodeElement>,
}

impl FunctionContext {
    /// Format as markdown for AI consumption
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        
        md.push_str(&format!("## Function: {} (line {})\n\n", 
            self.function.name, 
            self.function.location.line));
        
        md.push_str("```rust\n");
        md.push_str(&self.function.signature);
        md.push_str("\n```\n\n");

        if !self.callers.is_empty() {
            md.push_str("## Called by\n\n");
            for caller in &self.callers {
                md.push_str(&format!("- {} (line {})\n", caller.name, caller.location.line));
            }
            md.push('\n');
        }

        if !self.callees.is_empty() {
            md.push_str("## Calls\n\n");
            for callee in &self.callees {
                md.push_str(&format!("- {} (line {})\n", callee.name, callee.location.line));
            }
            md.push('\n');
        }

        md
    }
}