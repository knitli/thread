// crates/thread-core/src/engine.rs
use petgraph::{Graph, NodeIndex};
use std::collections::HashMap;
use std::path::Path;

// Your single source of truth
type CodeGraph = Graph<CodeNode, CodeEdge>;

#[derive(Debug, Clone)]
pub struct CodeNode {
    pub id: u64,              // rapidhash of content
    pub kind: NodeKind,
    pub name: String,
    pub line: usize,
    pub column: usize,
    pub text: String,
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    Function,
    Struct,
    Import,
    Variable,
    FunctionCall,
}

#[derive(Debug, Clone)]
pub enum CodeEdge {
    Calls,
    Imports,
    Defines,
    References,
}

pub struct ThreadEngine {
    graph: CodeGraph,
    parser: thread_parse::Parser,
    store: thread_store::ContentStore,
    editor: Option<thread_edit::Editor>,
    node_index: DashMap<u64, NodeIndex>,  // Fast lookups
}

impl ThreadEngine {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            parser: thread_parse::Parser::new(),
            store: thread_store::ContentStore::new(),
            editor: None,
            node_index: HashMap::new(),
        }
    }
    
    pub fn analyze_file<P: AsRef<Path>>(&mut self, path: P) -> Result<AnalysisResult, ThreadError> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;
        
        // Step 1: Detect language and parse
        let language = self.parser.detect_language(path)?;
        let ast_elements = self.parser.parse(&content, language)?;
        
        // Step 2: Store content with deduplication
        let content_hash = self.store.intern(&content);
        
        // Step 3: Build graph from AST elements
        let mut nodes_added = Vec::new();
        
        for element in ast_elements {
            let node = CodeNode {
                id: element.content_hash,
                kind: element.kind,
                name: element.name,
                line: element.line,
                column: element.column,
                text: element.text,
            };
            
            let node_idx = self.graph.add_node(node.clone());
            self.node_index.insert(node.id, node_idx);
            nodes_added.push(node_idx);
        }
        
        // Step 4: Add relationships (calls, imports, etc.)
        self.add_relationships(&ast_elements)?;
        
        Ok(AnalysisResult {
            path: path.to_path_buf(),
            content_hash,
            nodes_count: nodes_added.len(),
            graph_size: self.graph.node_count(),
        })
    }
    
    pub fn update_file<P: AsRef<Path>>(&mut self, path: P, new_content: &str) -> Result<(), ThreadError> {
        // Step 1: Find what changed using ropey
        if let Some(ref mut editor) = self.editor {
            let changes = editor.compute_changes(path.as_ref(), new_content)?;
            
            // Step 2: Incrementally update only changed parts
            for change in changes {
                self.update_graph_region(change)?;
            }
        } else {
            // Fallback: re-analyze entire file
            self.analyze_file(path)?;
        }
        
        Ok(())
    }
    
    pub fn find_function(&self, name: &str) -> Vec<&CodeNode> {
        self.graph
            .node_weights()
            .filter(|node| matches!(node.kind, NodeKind::Function) && node.name == name)
            .collect()
    }
    
    pub fn get_dependencies(&self, node_id: u64) -> Option<Vec<&CodeNode>> {
        let node_idx = self.node_index.get(&node_id)?;
        
        Some(
            self.graph
                .neighbors(*node_idx)
                .map(|idx| &self.graph[idx])
                .collect()
        )
    }
    
    fn add_relationships(&mut self, elements: &[AstElement]) -> Result<(), ThreadError> {
        // Find function calls, imports, etc. and add edges
        for element in elements {
            if let Some(calls) = &element.calls {
                for call in calls {
                    if let (Some(&caller_idx), Some(&callee_idx)) = 
                        (self.node_index.get(&element.content_hash), 
                         self.node_index.get(&call.target_hash)) {
                        self.graph.add_edge(caller_idx, callee_idx, CodeEdge::Calls);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn update_graph_region(&mut self, change: thread_edit::Change) -> Result<(), ThreadError> {
        // Remove old nodes in changed region
        let old_nodes: Vec<_> = self.graph
            .node_indices()
            .filter(|&idx| {
                let node = &self.graph[idx];
                node.line >= change.start_line && node.line <= change.end_line
            })
            .collect();
            
        for node_idx in old_nodes {
            let node_id = self.graph[node_idx].id;
            self.graph.remove_node(node_idx);
            self.node_index.remove(&node_id);
        }
        
        // Re-parse changed region and add new nodes
        let new_elements = self.parser.parse_region(&change.new_content, change.start_line)?;
        
        for element in new_elements {
            let node = CodeNode {
                id: element.content_hash,
                kind: element.kind,
                name: element.name,
                line: element.line,
                column: element.column,
                text: element.text,
            };
            
            let node_idx = self.graph.add_node(node);
            self.node_index.insert(element.content_hash, node_idx);
        }
        
        Ok(())
    }
}

#[derive(Debug)]
pub struct AnalysisResult {
    pub path: std::path::PathBuf,
    pub content_hash: u64,
    pub nodes_count: usize,
    pub graph_size: usize,
}

#[derive(Debug)]
pub struct AstElement {
    pub content_hash: u64,
    pub kind: NodeKind,
    pub name: String,
    pub line: usize,
    pub column: usize,
    pub text: String,
    pub calls: Option<Vec<FunctionCall>>,
}

#[derive(Debug)]
pub struct FunctionCall {
    pub target_hash: u64,
    pub name: String,
}

#[derive(Debug)]
pub enum ThreadError {
    Io(std::io::Error),
    Parse(String),
    NotFound(String),
}

impl From<std::io::Error> for ThreadError {
    fn from(e: std::io::Error) -> Self {
        ThreadError::Io(e)
    }
}
