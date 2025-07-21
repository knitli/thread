// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # Code Analyzer Service Trait
//!
//! Defines the analyzer service interface that abstracts over ast-grep analysis
//! functionality while preserving all matching and replacement capabilities.

use async_trait::async_trait;
use std::collections::HashMap;

use crate::types::{ParsedDocument, CodeMatch, AnalysisContext, CrossFileRelationship};
use crate::error::{ServiceResult, AnalysisError};
#[cfg(feature = "matching")]
use thread_ast_engine::source::Doc;
#[cfg(feature = "matching")]
use thread_ast_engine::{Node, NodeMatch};

#[cfg(feature = "matching")]
use thread_ast_engine::{Pattern, Matcher};

/// Core analyzer service trait that abstracts ast-grep analysis functionality.
///
/// This trait provides both single-file analysis (preserving all ast-grep matching
/// and replacement capabilities) and codebase-level analysis (adding graph intelligence
/// and cross-file relationships).
///
/// # Design Philosophy
///
/// - **Preserve Power**: All ast-grep Matcher and Replacer functionality accessible
/// - **Bridge Levels**: Connect file-level AST operations to codebase-level graph intelligence
/// - **Enable Intelligence**: Add cross-file relationships and codebase-wide analysis
/// - **Abstract Execution**: Support different execution environments and strategies
///
/// # Examples
///
/// ## File-Level Pattern Matching (preserves ast-grep power)
/// ```rust,no_run
/// # use thread_services::traits::CodeAnalyzer;
/// # use thread_services::types::{ParsedDocument, AnalysisContext};
/// # struct MyAnalyzer;
/// # #[async_trait::async_trait]
/// # impl CodeAnalyzer for MyAnalyzer {
/// #     async fn find_pattern<D: thread_ast_engine::source::Doc>(&self, document: &ParsedDocument<D>, pattern: &str, context: &AnalysisContext) -> Result<Vec<thread_services::types::CodeMatch<'_, D>>, thread_services::error::ServiceError> { todo!() }
/// #     async fn find_all_patterns<D: thread_ast_engine::source::Doc>(&self, document: &ParsedDocument<D>, patterns: &[&str], context: &AnalysisContext) -> Result<Vec<thread_services::types::CodeMatch<'_, D>>, thread_services::error::ServiceError> { todo!() }
/// #     async fn replace_pattern<D: thread_ast_engine::source::Doc>(&self, document: &mut ParsedDocument<D>, pattern: &str, replacement: &str, context: &AnalysisContext) -> Result<usize, thread_services::error::ServiceError> { todo!() }
/// #     async fn analyze_cross_file_relationships(&self, documents: &[ParsedDocument<impl thread_ast_engine::source::Doc>], context: &AnalysisContext) -> Result<Vec<thread_services::types::CrossFileRelationship>, thread_services::error::ServiceError> { todo!() }
/// #     fn capabilities(&self) -> thread_services::traits::AnalyzerCapabilities { todo!() }
/// # }
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let document: ParsedDocument<thread_ast_engine::tree_sitter::StrDoc<thread_language::SupportLang>> = todo!();
/// let analyzer = MyAnalyzer;
/// let context = AnalysisContext::default();
///
/// // Find all function declarations - preserves ast-grep pattern power
/// let matches = analyzer.find_pattern(
///     &document,
///     "fn $NAME($$$PARAMS) { $$$BODY }",
///     &context
/// ).await?;
///
/// for match_result in matches {
///     // Access ast-grep NodeMatch functionality
///     let node_match = match_result.ast_node_match();
///     let env = node_match.get_env();
///
///     if let Some(name) = env.get_match("NAME") {
///         println!("Function: {}", name.text());
///     }
///
///     // Plus codebase-level context
///     for relationship in match_result.relationships() {
///         println!("Cross-file relationship: {:?}", relationship.kind);
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// ## Codebase-Level Analysis
/// ```rust,no_run
/// # use thread_services::traits::CodeAnalyzer;
/// # use thread_services::types::{ParsedDocument, AnalysisContext, ExecutionScope};
/// # struct MyAnalyzer;
/// # #[async_trait::async_trait]
/// # impl CodeAnalyzer for MyAnalyzer {
/// #     async fn find_pattern<D: thread_ast_engine::source::Doc>(&self, document: &ParsedDocument<D>, pattern: &str, context: &AnalysisContext) -> Result<Vec<thread_services::types::CodeMatch<'_, D>>, thread_services::error::ServiceError> { todo!() }
/// #     async fn find_all_patterns<D: thread_ast_engine::source::Doc>(&self, document: &ParsedDocument<D>, patterns: &[&str], context: &AnalysisContext) -> Result<Vec<thread_services::types::CodeMatch<'_, D>>, thread_services::error::ServiceError> { todo!() }
/// #     async fn replace_pattern<D: thread_ast_engine::source::Doc>(&self, document: &mut ParsedDocument<D>, pattern: &str, replacement: &str, context: &AnalysisContext) -> Result<usize, thread_services::error::ServiceError> { todo!() }
/// #     async fn analyze_cross_file_relationships(&self, documents: &[ParsedDocument<impl thread_ast_engine::source::Doc>], context: &AnalysisContext) -> Result<Vec<thread_services::types::CrossFileRelationship>, thread_services::error::ServiceError> { todo!() }
/// #     fn capabilities(&self) -> thread_services::traits::AnalyzerCapabilities { todo!() }
/// # }
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let documents: Vec<ParsedDocument<thread_ast_engine::tree_sitter::StrDoc<thread_language::SupportLang>>> = vec![];
/// let analyzer = MyAnalyzer;
/// let mut context = AnalysisContext::default();
/// context.scope = ExecutionScope::Codebase;
///
/// // Analyze relationships across entire codebase
/// let relationships = analyzer.analyze_cross_file_relationships(
///     &documents,
///     &context
/// ).await?;
///
/// // Build intelligence on top of ast-grep file-level analysis
/// for rel in relationships {
///     match rel.kind {
///         thread_services::types::RelationshipKind::Calls => {
///             println!("{} calls {} ({}->{})",
///                 rel.source_symbol, rel.target_symbol,
///                 rel.source_file.display(), rel.target_file.display());
///         },
///         thread_services::types::RelationshipKind::Imports => {
///             println!("{} imports from {}",
///                 rel.source_file.display(), rel.target_file.display());
///         },
///         _ => {}
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait CodeAnalyzer: Send + Sync {
    /// Find matches for a pattern in a document.
    ///
    /// Preserves all ast-grep pattern matching power while adding codebase-level
    /// context. Returns CodeMatch instances that wrap NodeMatch and add cross-file
    /// relationship information.
    ///
    /// # Arguments
    /// * `document` - ParsedDocument to search in
    /// * `pattern` - AST pattern using ast-grep meta-variable syntax (e.g., "$VAR")
    /// * `context` - Analysis context for execution configuration
    ///
    /// # Returns
    /// Vector of CodeMatch instances with both ast-grep functionality and codebase context
    async fn find_pattern<D: Doc>(
        &self,
        document: &ParsedDocument<D>,
        pattern: &str,
        context: &AnalysisContext,
    ) -> ServiceResult<Vec<CodeMatch<'_, D>>>;

    /// Find matches for multiple patterns efficiently.
    ///
    /// Optimizes for multiple pattern searches by batching operations and
    /// reusing AST traversals where possible.
    ///
    /// # Arguments
    /// * `document` - ParsedDocument to search in
    /// * `patterns` - Slice of AST patterns to match
    /// * `context` - Analysis context for execution configuration
    ///
    /// # Returns
    /// Vector of CodeMatch instances for all pattern matches
    async fn find_all_patterns<D: Doc>(
        &self,
        document: &ParsedDocument<D>,
        patterns: &[&str],
        context: &AnalysisContext,
    ) -> ServiceResult<Vec<CodeMatch<'_, D>>>;

    /// Replace matches for a pattern with replacement content.
    ///
    /// Preserves all ast-grep replacement power including template-based replacement
    /// with meta-variable substitution and structural replacement.
    ///
    /// # Arguments
    /// * `document` - ParsedDocument to perform replacements in (modified in-place)
    /// * `pattern` - AST pattern to match for replacement
    /// * `replacement` - Replacement template or content
    /// * `context` - Analysis context for execution configuration
    ///
    /// # Returns
    /// Number of replacements made
    async fn replace_pattern<D: Doc>(
        &self,
        document: &mut ParsedDocument<D>,
        pattern: &str,
        replacement: &str,
        context: &AnalysisContext,
    ) -> ServiceResult<usize>;

    /// Analyze relationships across multiple files.
    ///
    /// This is where Thread extends ast-grep from file-level to codebase-level.
    /// Builds graph intelligence on top of ast-grep's powerful file-level analysis.
    ///
    /// # Arguments
    /// * `documents` - Collection of ParsedDocuments to analyze
    /// * `context` - Analysis context with scope and execution configuration
    ///
    /// # Returns
    /// Vector of CrossFileRelationship instances representing codebase-level connections
    async fn analyze_cross_file_relationships<D: Doc>(
        &self,
        documents: &[ParsedDocument<D>],
        context: &AnalysisContext,
    ) -> ServiceResult<Vec<CrossFileRelationship>>;

    /// Get analyzer capabilities and configuration.
    fn capabilities(&self) -> AnalyzerCapabilities;

    /// Find specific AST node types efficiently.
    ///
    /// Default implementation uses pattern matching, but implementations can
    /// override for more efficient node type searches.
    async fn find_nodes_by_kind<D: Doc>(
        &self,
        document: &ParsedDocument<D>,
        node_kind: &str,
        context: &AnalysisContext,
    ) -> ServiceResult<Vec<CodeMatch<'_, D>>> {
        // Default: use pattern matching based on node kind
        let pattern = match node_kind {
            "function_declaration" => "fn $NAME($$$PARAMS) { $$$BODY }",
            "class_declaration" => "class $NAME { $$$BODY }",
            "variable_declaration" => "let $VAR = $VALUE",
            // Add more patterns as needed
            _ => return Err(AnalysisError::InvalidPattern {
                pattern: format!("Unknown node kind: {}", node_kind)
            }.into()),
        };

        self.find_pattern(document, pattern, context).await
    }

    /// Validate pattern syntax before analysis.
    ///
    /// Default implementation performs basic validation.
    /// Implementations can override for language-specific validation.
    fn validate_pattern(&self, pattern: &str) -> ServiceResult<()> {
        if pattern.is_empty() {
            return Err(AnalysisError::InvalidPattern {
                pattern: "Pattern cannot be empty".to_string()
            }.into());
        }

        // Basic meta-variable validation
        if pattern.contains('$') {
            // Check for valid meta-variable format
            let mut chars = pattern.chars();
            let mut found_metavar = false;

            while let Some(ch) = chars.next() {
                if ch == '$' {
                    found_metavar = true;
                    // Next character should be alphabetic or underscore
                    if let Some(next_ch) = chars.next() {
                        if !next_ch.is_alphabetic() && next_ch != '_' {
                            return Err(AnalysisError::MetaVariable {
                                variable: format!("${}", next_ch),
                                message: "Invalid meta-variable format".to_string()
                            }.into());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Compile pattern for efficient reuse.
    ///
    /// Default implementation returns the pattern as-is.
    /// Implementations can override to pre-compile patterns for better performance.
    #[cfg(feature = "matching")]
    async fn compile_pattern(&self, pattern: &str) -> ServiceResult<CompiledPattern> {
        Ok(CompiledPattern {
            pattern: pattern.to_string(),
            compiled_data: None,
        })
    }

    /// Perform batch analysis operations efficiently.
    ///
    /// Optimizes for analyzing multiple documents with multiple patterns
    /// by batching operations and using appropriate execution strategies.
    async fn batch_analyze<D: Doc>(
        &self,
        documents: &[ParsedDocument<D>],
        patterns: &[&str],
        context: &AnalysisContext,
    ) -> ServiceResult<Vec<Vec<CodeMatch<'_, D>>>> {
        let mut results = Vec::new();

        for document in documents {
            let doc_results = self.find_all_patterns(document, patterns, context).await?;
            results.push(doc_results);
        }

        Ok(results)
    }

    /// Extract symbols and metadata from documents.
    ///
    /// Bridges ast-grep file-level analysis to codebase-level intelligence
    /// by extracting symbols, imports, exports, and other metadata.
    async fn extract_symbols<D: Doc>(
        &self,
        document: &mut ParsedDocument<D>,
        context: &AnalysisContext,
    ) -> ServiceResult<()> {
        // This will be implemented in the conversion utilities
        // For now, this is a placeholder that preserves the interface
        Ok(())
    }
}

/// Analyzer capabilities and configuration information
#[derive(Debug, Clone)]
pub struct AnalyzerCapabilities {
    /// Maximum number of patterns that can be analyzed concurrently
    pub max_concurrent_patterns: Option<usize>,

    /// Maximum number of matches to return per pattern
    pub max_matches_per_pattern: Option<usize>,

    /// Whether pattern compilation/caching is supported
    pub supports_pattern_compilation: bool,

    /// Whether cross-file analysis is supported
    pub supports_cross_file_analysis: bool,

    /// Whether batch operations are optimized
    pub supports_batch_optimization: bool,

    /// Whether incremental analysis is supported
    pub supports_incremental_analysis: bool,

    /// Supported analysis depth levels
    pub supported_analysis_depths: Vec<AnalysisDepth>,

    /// Performance characteristics
    pub performance_profile: AnalysisPerformanceProfile,

    /// Additional capability flags
    pub capability_flags: HashMap<String, bool>,
}

impl Default for AnalyzerCapabilities {
    fn default() -> Self {
        Self {
            max_concurrent_patterns: Some(50),
            max_matches_per_pattern: Some(1000),
            supports_pattern_compilation: false,
            supports_cross_file_analysis: false,
            supports_batch_optimization: true,
            supports_incremental_analysis: false,
            supported_analysis_depths: vec![
                AnalysisDepth::Syntax,
                AnalysisDepth::Local,
            ],
            performance_profile: AnalysisPerformanceProfile::Balanced,
            capability_flags: HashMap::new(),
        }
    }
}

/// Analysis depth levels
#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisDepth {
    /// Syntax-only analysis (AST patterns)
    Syntax,
    /// Include local scope analysis
    Local,
    /// Include cross-file dependencies
    CrossFile,
    /// Complete codebase analysis
    Deep,
}

/// Performance profile for analysis operations
#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisPerformanceProfile {
    /// Optimized for low memory usage
    LowMemory,
    /// Optimized for fast analysis
    FastAnalysis,
    /// Balanced memory and speed
    Balanced,
    /// Optimized for complex pattern matching
    ComplexPatterns,
    /// Optimized for large codebases
    LargeCodebase,
}

/// Compiled pattern for efficient reuse
#[derive(Debug, Clone)]
pub struct CompiledPattern {
    /// Original pattern string
    pub pattern: String,
    /// Compiled pattern data (implementation-specific)
    pub compiled_data: Option<Box<dyn std::any::Any + Send + Sync>>,
}

/// Analysis configuration for specific use cases
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    /// Maximum analysis depth
    pub max_depth: AnalysisDepth,

    /// Whether to collect cross-file relationships
    pub collect_relationships: bool,

    /// Whether to enable pattern caching
    pub enable_pattern_caching: bool,

    /// Preferred performance profile
    pub performance_profile: Option<AnalysisPerformanceProfile>,

    /// Custom configuration options
    pub custom_options: HashMap<String, String>,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            max_depth: AnalysisDepth::Local,
            collect_relationships: false,
            enable_pattern_caching: true,
            performance_profile: None, // Auto-detect
            custom_options: HashMap::new(),
        }
    }
}

/// Analyzer factory trait for creating configured analyzer instances
pub trait AnalyzerFactory: Send + Sync {
    /// Create a new analyzer instance with default configuration
    fn create_analyzer(&self) -> Box<dyn CodeAnalyzer>;

    /// Create a new analyzer instance with specific configuration
    fn create_configured_analyzer(&self, config: AnalysisConfig) -> Box<dyn CodeAnalyzer>;

    /// Get available analyzer types
    fn available_analyzers(&self) -> Vec<String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_capabilities_default() {
        let caps = AnalyzerCapabilities::default();
        assert!(!caps.supports_cross_file_analysis);
        assert!(caps.supports_batch_optimization);
        assert!(!caps.supports_pattern_compilation);
        assert_eq!(caps.performance_profile, AnalysisPerformanceProfile::Balanced);
    }

    #[test]
    fn test_analysis_config_default() {
        let config = AnalysisConfig::default();
        assert_eq!(config.max_depth, AnalysisDepth::Local);
        assert!(!config.collect_relationships);
        assert!(config.enable_pattern_caching);
    }
}
