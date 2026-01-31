// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later
#![feature(trait_alias)]
//! # Thread Service Layer
//!
//! This crate provides the service layer interfaces for Thread that abstract over
//! ast-grep functionality while preserving all its powerful capabilities.
//!
//! ## Core Philosophy
//!
//! The service layer acts as **abstraction glue** that:
//! - **Preserves Power**: All ast-grep capabilities (Matcher, Replacer, Position) remain accessible
//! - **Bridges Levels**: Connects file-level AST operations to codebase-level relational intelligence  
//! - **Enables Execution**: Abstracts over different execution environments (rayon, cloud workers)
//! - **Commercial Ready**: Clear boundaries for commercial extensions
//!
//! ## Architecture
//!
//! Thread pushes ast-grep from file-level to codebase-level analysis:
//! - **File Level**: ast-grep provides powerful AST pattern matching and replacement
//! - **Codebase Level**: Thread adds graph intelligence and cross-file relationships
//! - **Service Layer**: Abstracts and coordinates both levels seamlessly
//!
//! ## Key Components
//!
//! - [`types`] - Language-agnostic types that wrap ast-grep functionality
//! - [`traits`] - Service interfaces for parsing, analysis, and storage
//! - [`error`] - Comprehensive error handling with recovery strategies
//! - Execution contexts for different environments (CLI, cloud, WASM)
//!
//! ## Examples
//!
//! ### Basic Usage - Preserving ast-grep Power
//! ```rust,no_run
//! use thread_services::types::ParsedDocument;
//! use thread_services::traits::CodeAnalyzer;
//!
//! async fn analyze_code(document: &ParsedDocument<impl thread_ast_engine::source::Doc>) {
//!     // Access underlying ast-grep functionality directly
//!     let root = document.ast_grep_root();
//!     let matches = root.root().find_all("fn $NAME($$$PARAMS) { $$$BODY }");
//!     
//!     // Plus codebase-level metadata
//!     let symbols = document.metadata().defined_symbols.keys();
//!     println!("Found symbols: {:?}", symbols.collect::<Vec<_>>());
//! }
//! ```
//!
//! ### Codebase-Level Intelligence
//! ```rust,no_run
//! use thread_services::traits::CodeAnalyzer;
//! use thread_services::types::{AnalysisContext, ExecutionScope};
//!
//! async fn codebase_analysis(
//!     analyzer: &dyn CodeAnalyzer,
//!     documents: &[thread_services::types::ParsedDocument<impl thread_ast_engine::source::Doc>]
//! ) -> Result<(), Box<dyn std::error::Error>> {
//!     let mut context = AnalysisContext::default();
//!     context.scope = ExecutionScope::Codebase;
//!     
//!     // Analyze relationships across entire codebase
//!     let relationships = analyzer.analyze_cross_file_relationships(documents, &context).await?;
//!     
//!     // This builds on ast-grep's file-level power to create codebase intelligence
//!     for rel in relationships {
//!         println!("Cross-file relationship: {:?} -> {:?}", rel.source_file, rel.target_file);
//!     }
//!     Ok(())
//! }
//! ```

// Core modules
pub mod conversion;
pub mod error;
pub mod facade;
pub mod traits;
pub mod types;

// Re-export key types for convenience
pub use types::{
    AnalysisContext, AnalysisDepth, CodeMatch, CrossFileRelationship, ExecutionScope,
    ParsedDocument, SupportLang, SupportLangErr,
};

pub use error::{
    AnalysisError, ContextualError, ContextualResult, ErrorContextExt, ParseError,
    RecoverableError, ServiceError, ServiceResult,
};

pub use traits::{
    AnalysisPerformanceProfile, AnalyzerCapabilities, CodeAnalyzer, CodeParser, ParserCapabilities,
};

#[cfg(feature = "ast-grep-backend")]
pub use types::{
    AstNode,
    AstNodeMatch,
    // Re-export ast-grep types for compatibility
    AstPosition,
    AstRoot,
};

// Storage traits (commercial boundary)
#[cfg(feature = "storage-traits")]
pub use traits::{CacheService, StorageService};

use std::path::Path;
use thiserror::Error;

/// Legacy error type for backwards compatibility
#[derive(Error, Debug)]
#[deprecated(since = "0.1.0", note = "Use ServiceError instead")]
pub enum LegacyServiceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Execution error: {0}")]
    Execution(String),
}

/// Abstract execution context that can provide code from various sources
///
/// This trait provides a generic interface for accessing source code from
/// different sources (filesystem, memory, network, etc.) to support
/// different execution environments.
pub trait ExecutionContext {
    /// Read content from a source (could be file, memory, network, etc.)
    fn read_content(&self, source: &str) -> Result<String, ServiceError>;

    /// Write content to a destination
    fn write_content(&self, destination: &str, content: &str) -> Result<(), ServiceError>;

    /// List available sources (files, URLs, etc.)
    fn list_sources(&self) -> Result<Vec<String>, ServiceError>;
}

/// File system based execution context
pub struct FileSystemContext {
    base_path: std::path::PathBuf,
}

impl FileSystemContext {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }
}

impl ExecutionContext for FileSystemContext {
    fn read_content(&self, source: &str) -> Result<String, ServiceError> {
        let path = self.base_path.join(source);
        Ok(std::fs::read_to_string(path)?)
    }

    fn write_content(&self, destination: &str, content: &str) -> Result<(), ServiceError> {
        let path = self.base_path.join(destination);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(std::fs::write(path, content)?)
    }

    fn list_sources(&self) -> Result<Vec<String>, ServiceError> {
        // Basic implementation - can be enhanced with glob patterns, etc.
        let mut sources = Vec::new();
        for entry in std::fs::read_dir(&self.base_path)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                if let Some(name) = entry.file_name().to_str() {
                    sources.push(name.to_string());
                }
            }
        }
        Ok(sources)
    }
}

/// In-memory execution context for testing and WASM environments
pub struct MemoryContext {
    content: thread_utils::RapidMap<String, String>,
}

impl MemoryContext {
    pub fn new() -> Self {
        Self {
            content: thread_utils::RapidMap::default(),
        }
    }

    pub fn add_content(&mut self, name: String, content: String) {
        self.content.insert(name, content);
    }
}

impl Default for MemoryContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionContext for MemoryContext {
    fn read_content(&self, source: &str) -> Result<String, ServiceError> {
        self.content
            .get(source)
            .cloned()
            .ok_or_else(|| ServiceError::execution_dynamic(format!("Source not found: {source}")))
    }

    fn write_content(&self, _destination: &str, _content: &str) -> Result<(), ServiceError> {
        // For read-only memory context, we could store writes separately
        // or return an error. For now, we'll just succeed silently.
        Ok(())
    }

    fn list_sources(&self) -> Result<Vec<String>, ServiceError> {
        Ok(self.content.keys().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_context() {
        let mut ctx = MemoryContext::new();
        ctx.add_content("test.rs".to_string(), "fn main() {}".to_string());

        let content = ctx.read_content("test.rs").unwrap();
        assert_eq!(content, "fn main() {}");

        let sources = ctx.list_sources().unwrap();
        assert_eq!(sources, vec!["test.rs"]);
    }
}
