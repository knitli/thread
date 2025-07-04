//! Core traits and types for thread code analysis
//!
//! This crate defines the fundamental abstractions that make thread
//! language-agnostic and highly extensible.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

pub mod error;
pub mod hash;
pub mod location;

pub use error::*;
pub use hash::*;
pub use location::*;

/// A parsed code element with its metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeElement {
    pub id: ElementId,
    pub kind: ElementKind,
    pub name: String,
    pub signature: String,
    pub location: SourceLocation,
    pub content_hash: ContentHash,
    pub dependencies: Vec<String>,
    pub metadata: ElementMetadata,
}

/// Unique identifier for a code element
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ElementId(pub String);

/// Types of code elements we can extract
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElementKind {
    Function,
    Method,
    Class,
    Struct,
    Enum,
    Interface,
    Trait,
    Constant,
    Variable,
    Module,
    Import,
    Export,
    Type,
    Macro,
    // Extensible for language-specific elements
    Custom(String),
}

/// Language-agnostic metadata for code elements
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ElementMetadata {
    pub visibility: Option<Visibility>,
    pub is_async: bool,
    pub is_generic: bool,
    pub docstring: Option<String>,
    pub annotations: Vec<String>,
    pub return_type: Option<String>,
    pub parameters: Vec<Parameter>,
    // Extensible key-value store for language-specific data
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
    Package,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: Option<String>,
    pub default_value: Option<String>,
    pub is_optional: bool,
}

/// Result of parsing a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileParseResult {
    pub file_path: &'static Path,
    pub language: &'static str,
    pub elements: &'static [CodeElement],
    pub imports: &'static [Import],
    pub exports: &'static [Export],
    pub content_hash: ContentHash,
    pub parse_time_ms: &'static u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Import {
    pub module: String,
    pub items: Vec<String>,
    pub alias: Option<String>,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Export {
    pub name: String,
    pub kind: ElementKind,
    pub location: SourceLocation,
}

/// Result of parsing an entire project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectParseResult {
    pub project_id: String,
    pub files: Vec<FileParseResult>,
    pub dependency_graph: DependencyGraph,
    pub total_parse_time_ms: u64,
    pub statistics: ParseStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: Vec<String>, // file paths
    pub edges: Vec<(String, String)>, // (from_file, to_file)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseStatistics {
    pub total_files: usize,
    pub total_elements: usize,
    pub elements_by_kind: HashMap<ElementKind, usize>,
    pub files_by_language: HashMap<String, usize>,
}

/// Core trait for language-specific parsers
///
/// Implementing this trait is all that's needed to add support for a new language.
pub trait LanguageParser: Send + Sync {
    /// Unique identifier for this language (e.g., "rust", "python", "typescript")
    fn language_id(&self) -> &'static str;

    /// File extensions this parser handles (e.g., [".rs", ".rust"])
    fn file_extensions(&self) -> &'static [&'static str];

    /// Parse a single file and extract code elements
    fn parse_file(&self, content: &str, file_path: &Path) -> Result<FileParseResult>;

    /// Parse incrementally if the parser supports it
    /// Default implementation falls back to full parse
    fn parse_incremental(
        &self,
        old_content: &str,
        new_content: &str,
        file_path: &Path,
    ) -> Result<FileParseResult> {
        self.parse_file(new_content, file_path)
    }

    /// Extract dependencies from content (imports, includes, etc.)
    fn extract_dependencies(&self, content: &str, file_path: &Path) -> Result<Vec<String>>;

    /// Check if this parser can handle the given file
    fn can_parse(&self, file_path: &Path) -> bool {
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            let ext_with_dot = format!(".{}", ext);
            self.file_extensions().contains(&ext_with_dot.as_str())
        } else {
            false
        }
    }
}

/// Configuration for parsing operations
#[derive(Debug, Clone)]
pub struct ParseConfig {
    pub include_private: bool,
    pub include_tests: bool,
    pub max_file_size_mb: usize,
    pub parallel_parsing: bool,
    pub incremental_mode: bool,
    pub extract_docstrings: bool,
    pub language_specific: HashMap<String, serde_json::Value>,
}

impl Default for ParseConfig {
    fn default() -> Self {
        Self {
            include_private: true,
            include_tests: false,
            max_file_size_mb: 10,
            parallel_parsing: true,
            incremental_mode: true,
            extract_docstrings: true,
            language_specific: HashMap::new(),
        }
    }
}

/// Trait for custom query extractors
///
/// This allows users to define custom patterns to extract from parsed trees
pub trait QueryExtractor: Send + Sync {
    /// Unique name for this extractor
    fn name(&self) -> &'static str;

    /// Extract custom data from a parsed tree
    fn extract(&self, tree: &tree_sitter::Tree, source: &str) -> Result<Vec<serde_json::Value>>;

    /// Languages this extractor supports
    fn supported_languages(&self) -> &'static [&'static str];
}
