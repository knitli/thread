// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(dead_code)]
//! # Service Layer Types - Abstraction Glue for Thread
//!
//! This module provides language-agnostic types that abstract over ast-grep functionality
//! while preserving all its powerful capabilities. The service layer acts as glue between
//! file-level ast-grep operations and codebase-level graph intelligence.
//!
//! ## Core Philosophy
//!
//! - **Preserve Power**: All ast-grep capabilities (Matcher, Replacer, Position) remain accessible
//! - **Bridge Levels**: Connect file-level AST operations to codebase-level relational intelligence
//! - **Enable Execution**: Abstract over different execution environments (rayon, cloud workers)
//! - **Commercial Ready**: Clear boundaries for commercial extensions
//!
//! ## Key Types
//!
//! - [`ParsedDocument`] - Wraps ast-grep Root while enabling cross-file intelligence
//! - [`CodeMatch`] - Extends NodeMatch with codebase-level context
//! - [`ExecutionScope`] - Defines execution boundaries (file, module, codebase)
//! - [`AnalysisContext`] - Carries execution and analysis context across service boundaries

use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;

// Conditionally import thread dependencies when available
#[cfg(feature = "ast-grep-backend")]
use thread_ast_engine::{Node, NodeMatch, Position, Root};

#[cfg(feature = "ast-grep-backend")]
use thread_ast_engine::source::Doc;

#[cfg(feature = "ast-grep-backend")]
use thread_ast_engine::pinned::PinnedNodeData;

#[cfg(feature = "ast-grep-backend")]
use thread_language::SupportLang;

/// Re-export key ast-grep types when available
#[cfg(feature = "ast-grep-backend")]
pub use thread_ast_engine::{
    Node as AstNode, NodeMatch as AstNodeMatch, Position as AstPosition, Root as AstRoot,
};

#[cfg(feature = "ast-grep-backend")]
pub use thread_language::{SupportLang, SupportLangErr};

// Stub types for when ast-grep-backend is not available
#[cfg(not(feature = "ast-grep-backend"))]
pub trait Doc = Clone + 'static;

#[cfg(not(feature = "ast-grep-backend"))]
pub type Root<D: Doc> = ();

#[cfg(not(feature = "ast-grep-backend"))]
pub type Node<D: Doc> = ();

#[cfg(not(feature = "ast-grep-backend"))]
pub type NodeMatch<'a, D> = ();

#[cfg(not(feature = "ast-grep-backend"))]
pub type Position = ();

#[cfg(not(feature = "ast-grep-backend"))]
pub type PinnedNodeData<D> = ();

// SupportLang enum stub when not using ast-grep-backend
#[cfg(not(feature = "ast-grep-backend"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportLang {
    Bash,
    C,
    Cpp,
    CSharp,
    Css,
    Go,
    Elixir,
    Haskell,
    Html,
    Java,
    JavaScript,
    Kotlin,
    Lua,
    Nix,
    Php,
    Python,
    Ruby,
    Rust,
    Scala,
    Swift,
    TypeScript,
    Tsx,
    Yaml,
}

#[cfg(not(feature = "ast-grep-backend"))]
#[derive(Debug, Clone)]
pub struct SupportLangErr(pub String);

/// A parsed document that wraps ast-grep Root with additional codebase-level metadata.
///
/// This type preserves all ast-grep functionality while adding context needed for
/// cross-file analysis and graph intelligence. It acts as the bridge between
/// file-level AST operations and codebase-level relational analysis.
#[derive(Debug)]
pub struct ParsedDocument<D: Doc> {
    /// The underlying ast-grep Root - preserves all ast-grep functionality
    pub ast_root: Root<D>,

    /// Source file path for this document
    pub file_path: PathBuf,

    /// Language of this document
    pub language: SupportLang,

    /// Content hash for deduplication and change detection
    pub content_hash: u64,

    /// Codebase-level metadata (symbols, imports, exports, etc.)
    pub metadata: DocumentMetadata,

    /// Internal storage for ast-engine types (type-erased for abstraction)
    pub(crate) internal: Box<dyn Any + Send + Sync>,
}

impl<D: Doc> ParsedDocument<D> {
    /// Create a new ParsedDocument wrapping an ast-grep Root
    pub fn new(
        ast_root: Root<D>,
        file_path: PathBuf,
        language: SupportLang,
        content_hash: u64,
    ) -> Self {
        Self {
            ast_root,
            file_path,
            language,
            content_hash,
            metadata: DocumentMetadata::default(),
            internal: Box::new(()),
        }
    }

    /// Get the root node - preserves ast-grep API
    pub fn root(&self) -> Node<'_, D> {
        self.ast_root.root()
    }

    /// Get the underlying ast-grep Root for full access to capabilities
    pub fn ast_grep_root(&self) -> &Root<D> {
        &self.ast_root
    }

    /// Get mutable access to ast-grep Root for replacements
    pub fn ast_grep_root_mut(&mut self) -> &mut Root<D> {
        &mut self.ast_root
    }

    /// Create a pinned version for cross-thread/FFI usage
    pub fn pin_for_threading<F, T>(&self, f: F) -> PinnedNodeData<T>
    where
        F: FnOnce(&Root<D>) -> T,
    {
        PinnedNodeData::new(&self.ast_root, f)
    }

    /// Generate the source code (preserves ast-grep replacement functionality)
    pub fn generate(&self) -> String {
        self.ast_root.generate()
    }

    /// Get document metadata for codebase-level analysis
    pub fn metadata(&self) -> &DocumentMetadata {
        &self.metadata
    }

    /// Get mutable document metadata
    pub fn metadata_mut(&mut self) -> &mut DocumentMetadata {
        &mut self.metadata
    }
}

/// A pattern match that extends ast-grep NodeMatch with codebase-level context.
///
/// Preserves all NodeMatch functionality while adding cross-file relationship
/// information needed for graph intelligence.
#[derive(Debug)]
pub struct CodeMatch<'tree, D: Doc> {
    /// The underlying ast-grep NodeMatch - preserves all matching functionality
    pub node_match: NodeMatch<'tree, D>,

    /// Additional context for codebase-level analysis
    pub context: MatchContext,

    /// Cross-file relationships (calls, imports, inheritance, etc.)
    pub relationships: Vec<CrossFileRelationship>,
}

impl<'tree, D: Doc> CodeMatch<'tree, D> {
    /// Create a new CodeMatch wrapping an ast-grep NodeMatch
    pub fn new(node_match: NodeMatch<'tree, D>) -> Self {
        Self {
            node_match,
            context: MatchContext::default(),
            relationships: Vec::new(),
        }
    }

    /// Get the underlying NodeMatch for full ast-grep access
    pub fn ast_node_match(&self) -> &NodeMatch<'tree, D> {
        &self.node_match
    }

    /// Get the matched node (delegate to NodeMatch)
    pub fn node(&self) -> &Node<D> {
        &self.node_match
    }

    #[cfg(any(feature = "ast-grep-backend", feature = "matching"))]
    /// Get captured meta-variables (delegate to NodeMatch)
    pub fn get_env(&self) -> &thread_ast_engine::MetaVarEnv<'tree, D> {
        self.node_match.get_env()
    }

    /// Add cross-file relationship information
    pub fn add_relationship(&mut self, relationship: CrossFileRelationship) {
        self.relationships.push(relationship);
    }

    /// Get all cross-file relationships
    pub fn relationships(&self) -> &[CrossFileRelationship] {
        &self.relationships
    }
}

/// Metadata about a parsed document for codebase-level analysis
#[derive(Debug, Default, Clone)]
pub struct DocumentMetadata {
    /// Symbols defined in this document (functions, classes, variables)
    pub defined_symbols: HashMap<String, SymbolInfo>,

    /// Symbols imported from other files
    pub imported_symbols: HashMap<String, ImportInfo>,

    /// Symbols exported by this file
    pub exported_symbols: HashMap<String, ExportInfo>,

    /// Function calls made in this document
    pub function_calls: Vec<CallInfo>,

    /// Type definitions and usages
    pub type_info: Vec<TypeInfo>,

    /// Language-specific metadata
    pub language_metadata: HashMap<String, String>,
}

/// Information about a symbol definition
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: SymbolKind,
    pub position: Position,
    pub scope: String,
    pub visibility: Visibility,
}

/// Information about an import
#[derive(Debug, Clone)]
pub struct ImportInfo {
    pub symbol_name: String,
    pub source_path: String,
    pub import_kind: ImportKind,
    pub position: Position,
}

/// Information about an export
#[derive(Debug, Clone)]
pub struct ExportInfo {
    pub symbol_name: String,
    pub export_kind: ExportKind,
    pub position: Position,
}

/// Information about a function call
#[derive(Debug, Clone)]
pub struct CallInfo {
    pub function_name: String,
    pub position: Position,
    pub arguments_count: usize,
    pub is_resolved: bool,
    pub target_file: Option<PathBuf>,
}

/// Information about type usage
#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub type_name: String,
    pub position: Position,
    pub kind: TypeKind,
    pub generic_params: Vec<String>,
}

/// Cross-file relationships for graph intelligence
#[derive(Debug, Clone)]
pub struct CrossFileRelationship {
    pub kind: RelationshipKind,
    pub source_file: PathBuf,
    pub target_file: PathBuf,
    pub source_symbol: String,
    pub target_symbol: String,
    pub relationship_data: HashMap<String, String>,
}

/// Context for pattern matches
#[derive(Debug, Default, Clone)]
pub struct MatchContext {
    pub execution_scope: ExecutionScope,
    pub analysis_depth: AnalysisDepth,
    pub context_data: HashMap<String, String>,
}

/// Execution scope for analysis operations
#[derive(Debug, Clone, Default)]
pub enum ExecutionScope {
    /// Single file analysis
    #[default]
    File,
    /// Module or directory level
    Module(PathBuf),
    /// Entire codebase
    Codebase,
    /// Custom scope with specific files
    Custom(Vec<PathBuf>),
}

/// Depth of analysis to perform
#[derive(Debug, Clone, Default)]
pub enum AnalysisDepth {
    /// Syntax-only analysis
    Syntax,
    /// Include local dependencies
    #[default]
    Local,
    /// Include external dependencies
    Deep,
    /// Complete codebase analysis
    Complete,
}

/// Execution context that carries state across service boundaries
#[derive(Debug, Clone)]
pub struct AnalysisContext {
    /// Scope of the current analysis
    pub scope: ExecutionScope,

    /// Depth of analysis
    pub depth: AnalysisDepth,

    /// Base directory for relative path resolution
    pub base_directory: PathBuf,

    /// Include patterns for file filtering
    pub include_patterns: Vec<String>,

    /// Exclude patterns for file filtering
    pub exclude_patterns: Vec<String>,

    /// Maximum number of files to process
    pub max_files: Option<usize>,

    /// Parallel execution configuration
    pub execution_config: ExecutionConfig,

    /// Custom context data
    pub context_data: HashMap<String, String>,
}

impl Default for AnalysisContext {
    fn default() -> Self {
        Self {
            scope: ExecutionScope::File,
            depth: AnalysisDepth::Local,
            base_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            include_patterns: vec!["**/*".to_string()],
            exclude_patterns: vec!["**/node_modules/**".to_string(), "**/target/**".to_string()],
            max_files: None,
            execution_config: ExecutionConfig::default(),
            context_data: HashMap::new(),
        }
    }
}

/// Configuration for execution environments
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// Parallel execution strategy
    pub strategy: ExecutionStrategy,

    /// Maximum number of concurrent operations
    pub max_concurrency: Option<usize>,

    /// Chunk size for batched operations
    pub chunk_size: Option<usize>,

    /// Timeout for individual operations
    pub operation_timeout: Option<std::time::Duration>,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            strategy: ExecutionStrategy::Auto,
            max_concurrency: None,
            chunk_size: None,
            operation_timeout: None,
        }
    }
}

/// Execution strategy for different environments
#[derive(Debug, Clone, Default)]
pub enum ExecutionStrategy {
    /// Choose strategy automatically based on environment
    #[default]
    Auto,
    /// Single-threaded execution
    Sequential,
    /// Rayon-based parallel execution (for CLI)
    Rayon,
    /// Chunked execution for cloud workers
    Chunked,
    /// Custom execution strategy
    Custom(String),
}

// Enums for categorizing symbols and relationships

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Function,
    Class,
    Interface,
    Variable,
    Constant,
    Type,
    Module,
    Namespace,
    Enum,
    Field,
    Property,
    Method,
    Constructor,
    Other(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
    Package,
    Other(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportKind {
    Named,
    Default,
    Namespace,
    SideEffect,
    Dynamic,
    Other(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExportKind {
    Named,
    Default,
    Namespace,
    Reexport,
    Other(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    Primitive,
    Struct,
    Class,
    Interface,
    Union,
    Enum,
    Generic,
    Function,
    Array,
    Other(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelationshipKind {
    /// Function calls another function
    Calls,
    /// Module imports from another module
    Imports,
    /// Class inherits from another class
    Inherits,
    /// Interface implements another interface
    Implements,
    /// Type uses another type
    Uses,
    /// Module depends on another module
    DependsOn,
    /// Symbol references another symbol
    References,
    /// Custom relationship type
    Custom(String),
}

/// Range representing a span of text in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Create a range from ast-grep positions
    pub fn from_ast_positions(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Check if this range contains a position
    pub fn contains(&self, pos: Position) -> bool {
        pos >= self.start && pos <= self.end
    }

    /// Check if this range overlaps with another range
    pub fn overlaps(&self, other: &Range) -> bool {
        self.start <= other.end && other.start <= self.end
    }
}
