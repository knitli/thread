// crates/thread-ast/src/graph.rs
use petgraph::{Graph, NodeIndex, EdgeIndex};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

pub type AstGraph = Graph<AstNode, AstEdge>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstNode {
    pub id: NodeId,
    pub kind: String,
    pub text: String,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub metadata: NodeMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub is_named: bool,
    pub is_error: bool,
    pub byte_range: (usize, usize),
    pub content_hash: blake3::Hash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AstEdge {
    Child { field_name: Option<String> },
    Reference { ref_type: ReferenceType },
    DataFlow { flow_type: DataFlowType },
    ControlFlow { flow_type: ControlFlowType },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReferenceType {
    Definition,
    Usage,
    Import,
    Export,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataFlowType {
    Assignment,
    Parameter,
    Return,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlFlowType {
    Conditional,
    Loop,
    Call,
    Exception,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u32);

pub struct AstGraphBuilder {
    graph: AstGraph,
    node_map: HashMap<NodeId, NodeIndex>,
    next_id: u32,
}

impl AstGraphBuilder {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            node_map: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn add_node(&mut self, node: AstNode) -> NodeIndex {
        let node_id = node.id;
        let index = self.graph.add_node(node);
        self.node_map.insert(node_id, index);
        index
    }

    pub fn add_edge(&mut self, parent: NodeId, child: NodeId, edge: AstEdge) -> Option<EdgeIndex> {
        let parent_idx = self.node_map.get(&parent)?;
        let child_idx = self.node_map.get(&child)?;
        Some(self.graph.add_edge(*parent_idx, *child_idx, edge))
    }

    pub fn build(self) -> AstGraph {
        self.graph
    }

    fn next_id(&mut self) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;
        id
    }
}

// Convert from type-sitter nodes to petgraph
impl From<&thread_parser::TypedNode<'_>> for AstNode {
    fn from(node: &thread_parser::TypedNode<'_>) -> Self {
        let text = node.text();
        let content_hash = blake3::hash(text.as_bytes());
        let (start_line, start_column) = node.start_position();
        let (end_line, end_column) = node.end_position();

        AstNode {
            id: NodeId(0), // Will be set by builder
            kind: node.kind().to_string(),
            text: text.to_string(),
            start_line,
            start_column,
            end_line,
            end_column,
            metadata: NodeMetadata {
                is_named: node.node.is_named(),
                is_error: node.node.is_error(),
                byte_range: (node.node.start_byte(), node.node.end_byte()),
                content_hash,
            },
        }
    }
}
