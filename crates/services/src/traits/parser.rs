// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # Code Parser Service Trait
//!
//! Defines the parser service interface that abstracts over ast-grep parsing
//! functionality while preserving all its capabilities.

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;

use crate::error::{ParseError, ServiceResult};
use crate::types::{AnalysisContext, ExecutionScope, ParsedDocument};
use thread_ast_engine::source::Doc;
use thread_language::SupportLang;

/// Core parser service trait that abstracts ast-grep parsing functionality.
///
/// This trait provides async interfaces for parsing source code into ParsedDocument
/// instances that preserve all ast-grep capabilities while enabling codebase-level
/// analysis. The trait supports both single-file and multi-file parsing operations.
///
/// # Design Philosophy
///
/// - **Preserve Power**: All ast-grep functionality remains accessible through ParsedDocument
/// - **Enable Intelligence**: Add metadata needed for codebase-level graph analysis
/// - **Abstract Execution**: Support different execution environments
/// - **Commercial Ready**: Clear extension points for commercial parsing features
///
/// # Examples
///
/// ## Single File Parsing
/// ```rust,no_run
/// # use thread_services::traits::CodeParser;
/// # use thread_services::types::AnalysisContext;
/// # use thread_language::SupportLang;
/// # struct MyParser;
/// # #[async_trait::async_trait]
/// # impl CodeParser for MyParser {
/// #     async fn parse_content(&self, content: &str, language: SupportLang, context: &AnalysisContext) -> Result<thread_services::types::ParsedDocument<thread_ast_engine::tree_sitter::StrDoc<SupportLang>>, thread_services::error::ServiceError> { todo!() }
/// #     async fn parse_file(&self, file_path: &std::path::Path, context: &AnalysisContext) -> Result<thread_services::types::ParsedDocument<thread_ast_engine::tree_sitter::StrDoc<SupportLang>>, thread_services::error::ServiceError> { todo!() }
/// #     async fn parse_multiple_files(&self, file_paths: &[&std::path::Path], context: &AnalysisContext) -> Result<Vec<thread_services::types::ParsedDocument<thread_ast_engine::tree_sitter::StrDoc<SupportLang>>>, thread_services::error::ServiceError> { todo!() }
/// #     fn capabilities(&self) -> thread_services::traits::ParserCapabilities { todo!() }
/// #     fn supported_languages(&self) -> &[SupportLang] { todo!() }
/// # }
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let parser = MyParser;
/// let context = AnalysisContext::default();
///
/// // Parse a Rust file
/// let document = parser.parse_file(
///     std::path::Path::new("src/main.rs"),
///     &context
/// ).await?;
///
/// // Access underlying ast-grep functionality
/// let root = document.ast_grep_root();
/// let matches = root.root().find_all("fn $NAME($$$PARAMS) { $$$BODY }");
/// # Ok(())
/// # }
/// ```
///
/// ## Multi-File Codebase Parsing
/// ```rust,no_run
/// # use thread_services::traits::CodeParser;
/// # use thread_services::types::{AnalysisContext, ExecutionScope};
/// # use std::path::PathBuf;
/// # struct MyParser;
/// # #[async_trait::async_trait]
/// # impl CodeParser for MyParser {
/// #     async fn parse_content(&self, content: &str, language: thread_language::SupportLang, context: &AnalysisContext) -> Result<thread_services::types::ParsedDocument<thread_ast_engine::tree_sitter::StrDoc<thread_language::SupportLang>>, thread_services::error::ServiceError> { todo!() }
/// #     async fn parse_file(&self, file_path: &std::path::Path, context: &AnalysisContext) -> Result<thread_services::types::ParsedDocument<thread_ast_engine::tree_sitter::StrDoc<thread_language::SupportLang>>, thread_services::error::ServiceError> { todo!() }
/// #     async fn parse_multiple_files(&self, file_paths: &[&std::path::Path], context: &AnalysisContext) -> Result<Vec<thread_services::types::ParsedDocument<thread_ast_engine::tree_sitter::StrDoc<thread_language::SupportLang>>>, thread_services::error::ServiceError> { todo!() }
/// #     fn capabilities(&self) -> thread_services::traits::ParserCapabilities { todo!() }
/// #     fn supported_languages(&self) -> &[thread_language::SupportLang] { todo!() }
/// # }
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let parser = MyParser;
/// let mut context = AnalysisContext::default();
/// context.scope = ExecutionScope::Codebase;
///
/// // Parse entire codebase
/// let files: Vec<&std::path::Path> = vec![
///     std::path::Path::new("src/main.rs"),
///     std::path::Path::new("src/lib.rs"),
///     std::path::Path::new("src/parser.rs"),
/// ];
///
/// let documents = parser.parse_multiple_files(&files, &context).await?;
///
/// // Each document preserves ast-grep capabilities + adds codebase metadata
/// for doc in &documents {
///     println!("File: {:?}", doc.file_path);
///     println!("Symbols: {:?}", doc.metadata().defined_symbols.keys().collect::<Vec<_>>());
/// }
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait CodeParser: Send + Sync {
    /// Parse source content into a ParsedDocument.
    ///
    /// This method wraps ast-grep parsing with additional metadata collection
    /// for codebase-level analysis while preserving all ast-grep functionality.
    ///
    /// # Arguments
    /// * `content` - Source code to parse
    /// * `language` - Programming language of the content
    /// * `context` - Analysis context containing execution configuration
    ///
    /// # Returns
    /// ParsedDocument that wraps ast-grep Root with additional metadata
    async fn parse_content(
        &self,
        content: &str,
        language: SupportLang,
        context: &AnalysisContext,
    ) -> ServiceResult<ParsedDocument<impl Doc>>;

    /// Parse a single file into a ParsedDocument.
    ///
    /// Automatically detects language from file extension and reads file content.
    /// Collects symbols, imports, and other metadata for codebase-level analysis.
    ///
    /// # Arguments
    /// * `file_path` - Path to source file to parse
    /// * `context` - Analysis context containing execution configuration
    ///
    /// # Returns
    /// ParsedDocument with both ast-grep functionality and codebase metadata
    async fn parse_file(
        &self,
        file_path: &Path,
        context: &AnalysisContext,
    ) -> ServiceResult<ParsedDocument<impl Doc>>;

    /// Parse multiple files with efficient parallel execution.
    ///
    /// Uses execution strategy from context to optimize for different environments:
    /// - Rayon for CLI parallel processing
    /// - Chunked execution for cloud workers
    /// - Sequential for single-threaded environments
    ///
    /// # Arguments
    /// * `file_paths` - Slice of file paths to parse
    /// * `context` - Analysis context with execution configuration
    ///
    /// # Returns
    /// Vector of ParsedDocuments in same order as input paths
    async fn parse_multiple_files(
        &self,
        file_paths: &[&Path],
        context: &AnalysisContext,
    ) -> ServiceResult<Vec<ParsedDocument<impl Doc>>>;

    /// Get parser capabilities and configuration.
    ///
    /// Describes what features this parser implementation supports,
    /// including performance characteristics and execution strategies.
    fn capabilities(&self) -> ParserCapabilities;

    /// Get list of supported programming languages.
    ///
    /// Returns slice of SupportLang values that this parser can handle.
    /// Used for language detection and validation.
    fn supported_languages(&self) -> &[SupportLang];

    /// Detect language from file path.
    ///
    /// Default implementation uses file extension matching.
    /// Implementations can override for more sophisticated detection.
    fn detect_language(&self, file_path: &Path) -> ServiceResult<SupportLang> {
        SupportLang::from_path(file_path).map_err(|e| {
            ParseError::LanguageDetectionFailed {
                file_path: file_path.to_path_buf(),
            }
            .into()
        })
    }

    /// Validate content before parsing.
    ///
    /// Default implementation checks for basic validity.
    /// Implementations can override for language-specific validation.
    fn validate_content(&self, content: &str, language: SupportLang) -> ServiceResult<()> {
        if content.is_empty() {
            return Err(ParseError::InvalidSource {
                message: "Content is empty".to_string(),
            }
            .into());
        }

        // Check content size limits based on capabilities
        let capabilities = self.capabilities();
        if let Some(max_size) = capabilities.max_content_size {
            if content.len() > max_size {
                return Err(ParseError::ContentTooLarge {
                    size: content.len(),
                    max_size,
                }
                .into());
            }
        }

        Ok(())
    }

    /// Pre-process content before parsing.
    ///
    /// Default implementation returns content unchanged.
    /// Implementations can override for content normalization.
    fn preprocess_content(&self, content: &str, language: SupportLang) -> String {
        content.to_string()
    }

    /// Post-process parsed document.
    ///
    /// Default implementation returns document unchanged.
    /// Implementations can override to add custom metadata collection.
    async fn postprocess_document<D: Doc>(
        &self,
        mut document: ParsedDocument<D>,
        context: &AnalysisContext,
    ) -> ServiceResult<ParsedDocument<D>> {
        // Default: collect basic metadata
        self.collect_basic_metadata(&mut document, context).await?;
        Ok(document)
    }

    /// Collect basic metadata for codebase-level analysis.
    ///
    /// Default implementation extracts symbols, imports, exports, and function calls.
    /// This bridges ast-grep file-level analysis to codebase-level intelligence.
    async fn collect_basic_metadata<D: Doc>(
        &self,
        document: &mut ParsedDocument<D>,
        _context: &AnalysisContext,
    ) -> ServiceResult<()> {
        // This will be implemented in the conversion utilities
        // For now, this is a placeholder that preserves the interface
        Ok(())
    }
}

/// Parser capabilities and configuration information
#[derive(Debug, Clone)]
pub struct ParserCapabilities {
    /// Maximum content size this parser can handle (in bytes)
    pub max_content_size: Option<usize>,

    /// Maximum number of files that can be parsed concurrently
    pub max_concurrent_files: Option<usize>,

    /// Supported execution strategies
    pub execution_strategies: Vec<ExecutionStrategy>,

    /// Whether incremental parsing is supported
    pub supports_incremental: bool,

    /// Whether error recovery during parsing is supported
    pub supports_error_recovery: bool,

    /// Whether codebase-level metadata collection is supported
    pub supports_metadata_collection: bool,

    /// Whether cross-file analysis is supported
    pub supports_cross_file_analysis: bool,

    /// Performance characteristics
    pub performance_profile: PerformanceProfile,

    /// Additional capability flags
    pub capability_flags: HashMap<String, bool>,
}

impl Default for ParserCapabilities {
    fn default() -> Self {
        Self {
            max_content_size: Some(10 * 1024 * 1024), // 10MB default
            max_concurrent_files: Some(100),
            execution_strategies: vec![ExecutionStrategy::Sequential, ExecutionStrategy::Rayon],
            supports_incremental: false,
            supports_error_recovery: true,
            supports_metadata_collection: true,
            supports_cross_file_analysis: false,
            performance_profile: PerformanceProfile::Balanced,
            capability_flags: HashMap::new(),
        }
    }
}

/// Execution strategy for parser operations
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionStrategy {
    /// Single-threaded sequential execution
    Sequential,
    /// Rayon-based parallel execution (for CLI)
    Rayon,
    /// Chunked execution
    Chunked { chunk_size: usize },
    /// Custom execution strategy
    Custom(String),
}

/// Performance profile for parser operations
#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceProfile {
    /// Optimized for low memory usage
    LowMemory,
    /// Optimized for fast parsing speed
    FastParsing,
    /// Balanced memory usage and parsing speed
    Balanced,
    /// Optimized for high throughput
    HighThroughput,
}

/// Parser configuration for specific use cases
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Whether to collect metadata during parsing
    pub collect_metadata: bool,

    /// Whether to enable error recovery
    pub enable_error_recovery: bool,

    /// Preferred execution strategy
    pub execution_strategy: Option<ExecutionStrategy>,

    /// Custom configuration options
    pub custom_options: HashMap<String, String>,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            collect_metadata: true,
            enable_error_recovery: true,
            execution_strategy: None, // Auto-detect
            custom_options: HashMap::new(),
        }
    }
}

/// Parser factory trait for creating configured parser instances
pub trait ParserFactory: Send + Sync {
    /// Create a new parser instance with default configuration
    fn create_parser(&self) -> Box<dyn CodeParser>;

    /// Create a new parser instance with specific configuration
    fn create_configured_parser(&self, config: ParserConfig) -> Box<dyn CodeParser>;

    /// Get available parser types
    fn available_parsers(&self) -> Vec<String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parser_capabilities_default() {
        let caps = ParserCapabilities::default();
        assert!(caps.supports_metadata_collection);
        assert!(caps.supports_error_recovery);
        assert!(!caps.supports_cross_file_analysis);
        assert_eq!(caps.performance_profile, PerformanceProfile::Balanced);
    }

    #[test]
    fn test_parser_config_default() {
        let config = ParserConfig::default();
        assert!(config.collect_metadata);
        assert!(config.enable_error_recovery);
        assert!(config.execution_strategy.is_none());
    }
}
