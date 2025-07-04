// crates/thread-core/src/engine.rs
use thread_ast::{AstGraph, AstGraphBuilder};
use thread_parser::TypedParser;
use thread_lang::LanguageDetector;
use thread_diff::ThreadDiffEngine;
use std::path::Path;
use anyhow::Result;

pub struct ThreadAnalysisEngine {
    language_detector: LanguageDetector,
    parsers: std::collections::HashMap<String, TypedParser>,
    diff_engine: ThreadDiffEngine,
    content_store: ContentStore,
}

impl ThreadAnalysisEngine {
    pub fn new() -> Result<Self> {
        Ok(Self {
            language_detector: LanguageDetector::new(),
            parsers: std::collections::HashMap::new(),
            diff_engine: ThreadDiffEngine::new(),
            content_store: ContentStore::new(),
        })
    }

    pub fn analyze_file<P: AsRef<Path>>(&mut self, file_path: P) -> Result<AnalysisResult> {
        let path = file_path.as_ref();
        let content = std::fs::read_to_string(path)?;

        // Detect language
        let language = self.language_detector.detect_language(path)
            .ok_or_else(|| anyhow::anyhow!("Unsupported file type"))?;

        // Parse with appropriate parser
        let ast_graph = self.parse_to_graph(&content, &language)?;

        // Store in content-addressable storage
        let content_hash = self.content_store.intern(&content);

        Ok(AnalysisResult {
            file_path: path.to_path_buf(),
            content_hash,
            language,
            ast_graph,
            line_count: content.lines().count(),
        })
    }

    pub fn diff_files<P: AsRef<Path>>(&mut self, old_path: P, new_path: P) -> Result<DiffResult> {
        let old_result = self.analyze_file(old_path)?;
        let new_result = self.analyze_file(new_path)?;

        self.diff_engine.diff_asts(&old_result.ast_graph, &new_result.ast_graph)
    }

    fn parse_to_graph(&mut self, content: &str, language: &SupportedLanguage) -> Result<AstGraph> {
        let lang_key = format!("{:?}", language);

        if !self.parsers.contains_key(&lang_key) {
            let parser = TypedParser::new(language.clone())?;
            self.parsers.insert(lang_key.clone(), parser);
        }

        let parser = self.parsers.get_mut(&lang_key).unwrap();
        let typed_tree = parser.parse(content)?;

        // Convert to petgraph
        let mut builder = AstGraphBuilder::new();
        self.build_graph_recursive(&typed_tree.root_node(), &mut builder, None)?;

        Ok(builder.build())
    }

    fn build_graph_recursive(
        &self,
        node: &thread_parser::TypedNode,
        builder: &mut AstGraphBuilder,
        parent_id: Option<thread_ast::NodeId>,
    ) -> Result<thread_ast::NodeId> {
        let ast_node = thread_ast::AstNode::from(node);
        let node_id = ast_node.id;
        let node_idx = builder.add_node(ast_node);

        // Add parent edge
        if let Some(parent) = parent_id {
            builder.add_edge(parent, node_id, thread_ast::AstEdge::Child { field_name: None });
        }

        // Process children
        for child in node.children() {
            self.build_graph_recursive(&child, builder, Some(node_id))?;
        }

        Ok(node_id)
    }
}

#[derive(Debug)]
pub struct AnalysisResult {
    pub file_path: std::path::PathBuf,
    pub content_hash: blake3::Hash,
    pub language: thread_lang::SupportedLanguage,
    pub ast_graph: AstGraph,
    pub line_count: usize,
}

// Content-addressable storage for deduplication
use std::collections::HashMap;
use std::sync::Arc;

pub struct ContentStore {
    store: HashMap<blake3::Hash, Arc<str>>,
    reverse_index: HashMap<Arc<str>, blake3::Hash>,
}

impl ContentStore {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            reverse_index: HashMap::new(),
        }
    }

    pub fn intern(&mut self, content: &str) -> blake3::Hash {
        let hash = blake3::hash(content.as_bytes());

        if self.store.contains_key(&hash) {
            return hash;
        }

        let interned: Arc<str> = Arc::from(content);
        self.store.insert(hash, interned.clone());
        self.reverse_index.insert(interned, hash);
        hash
    }

    pub fn get(&self, hash: &blake3::Hash) -> Option<&Arc<str>> {
        self.store.get(hash)
    }
}
