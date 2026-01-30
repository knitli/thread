// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Dependency graph builder that coordinates language-specific extractors.
//!
//! This module provides a unified interface for building dependency graphs
//! across multiple programming languages. It uses the extractor subsystem
//! to parse import/dependency statements and constructs a [`DependencyGraph`]
//! representing the file-level and symbol-level dependencies in a codebase.
//!
//! ## Architecture
//!
//! ```text
//! DependencyGraphBuilder
//!   ├─> LanguageDetector (file extension → Language)
//!   ├─> RustDependencyExtractor (use statements)
//!   ├─> TypeScriptDependencyExtractor (import/require)
//!   ├─> PythonDependencyExtractor (import statements)
//!   └─> GoDependencyExtractor (import blocks)
//! ```
//!
//! ## Example Usage
//!
//! ```rust
//! use thread_flow::incremental::dependency_builder::DependencyGraphBuilder;
//! use thread_flow::incremental::storage::InMemoryStorage;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let storage = Box::new(InMemoryStorage::new());
//! let mut builder = DependencyGraphBuilder::new(storage);
//!
//! // Extract dependencies from files
//! builder.extract_file(Path::new("src/main.rs")).await?;
//! builder.extract_file(Path::new("src/utils.ts")).await?;
//!
//! // Access the built graph
//! let graph = builder.graph();
//! println!("Found {} files with {} dependencies",
//!          graph.node_count(), graph.edge_count());
//!
//! // Persist to storage
//! builder.persist().await?;
//! # Ok(())
//! # }
//! ```

use super::extractors::{
    GoDependencyExtractor, PythonDependencyExtractor, RustDependencyExtractor,
    TypeScriptDependencyExtractor, go::ExtractionError as GoExtractionError,
    python::ExtractionError as PyExtractionError, rust::ExtractionError as RustExtractionError,
    typescript::ExtractionError as TsExtractionError,
};
use super::graph::DependencyGraph;
use super::storage::{StorageBackend, StorageError};
use super::types::AnalysisDefFingerprint;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

// ─── Language Types ──────────────────────────────────────────────────────────

/// Supported programming languages for dependency extraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    /// Rust programming language (.rs files)
    Rust,
    /// TypeScript (.ts, .tsx files)
    TypeScript,
    /// JavaScript (.js, .jsx files)
    JavaScript,
    /// Python (.py files)
    Python,
    /// Go (.go files)
    Go,
}

// ─── Language Detection ──────────────────────────────────────────────────────

/// Detects programming language from file extension.
pub struct LanguageDetector;

impl LanguageDetector {
    /// Detects the programming language from a file path.
    ///
    /// Returns `Some(Language)` if the extension is recognized,
    /// or `None` for unsupported file types.
    ///
    /// # Examples
    ///
    /// ```
    /// use thread_flow::incremental::dependency_builder::{Language, LanguageDetector};
    /// use std::path::Path;
    ///
    /// assert_eq!(
    ///     LanguageDetector::detect_language(Path::new("main.rs")),
    ///     Some(Language::Rust)
    /// );
    /// assert_eq!(
    ///     LanguageDetector::detect_language(Path::new("app.ts")),
    ///     Some(Language::TypeScript)
    /// );
    /// assert_eq!(
    ///     LanguageDetector::detect_language(Path::new("file.java")),
    ///     None
    /// );
    /// ```
    pub fn detect_language(path: &Path) -> Option<Language> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext.to_lowercase().as_str() {
                "rs" => Some(Language::Rust),
                "ts" | "tsx" => Some(Language::TypeScript),
                "js" | "jsx" => Some(Language::JavaScript),
                "py" => Some(Language::Python),
                "go" => Some(Language::Go),
                _ => None,
            })
    }
}

// ─── Build Errors ────────────────────────────────────────────────────────────

/// Errors that can occur during dependency graph building.
#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    /// Language not supported for dependency extraction.
    #[error("Unsupported language for file: {0}")]
    UnsupportedLanguage(PathBuf),

    /// Failed to read file contents.
    #[error("IO error reading {file}: {error}")]
    IoError {
        file: PathBuf,
        error: std::io::Error,
    },

    /// Dependency extraction failed for a file.
    #[error("Extraction failed for {file}: {error}")]
    ExtractionFailed { file: PathBuf, error: String },

    /// Storage backend operation failed.
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    /// Rust extraction error.
    #[error("Rust extraction error: {0}")]
    RustExtraction(#[from] RustExtractionError),

    /// TypeScript/JavaScript extraction error.
    #[error("TypeScript extraction error: {0}")]
    TypeScriptExtraction(#[from] TsExtractionError),

    /// Python extraction error.
    #[error("Python extraction error: {0}")]
    PythonExtraction(#[from] PyExtractionError),

    /// Go extraction error.
    #[error("Go extraction error: {0}")]
    GoExtraction(#[from] GoExtractionError),
}

// ─── Dependency Graph Builder ────────────────────────────────────────────────

/// Coordinates dependency extraction across multiple languages to build a unified dependency graph.
///
/// The builder uses language-specific extractors to parse import/dependency
/// statements and progressively constructs a [`DependencyGraph`]. It manages
/// the storage backend for persistence and provides batch processing capabilities.
///
/// ## Usage Pattern
///
/// 1. Create builder with storage backend
/// 2. Extract files using `extract_file()` or `extract_files()`
/// 3. Access graph with `graph()`
/// 4. Optionally persist with `persist()`
///
/// # Examples
///
/// ```rust,no_run
/// # use thread_flow::incremental::dependency_builder::DependencyGraphBuilder;
/// # use thread_flow::incremental::storage::InMemoryStorage;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let storage = Box::new(InMemoryStorage::new());
/// let mut builder = DependencyGraphBuilder::new(storage);
///
/// // Extract single file
/// builder.extract_file(std::path::Path::new("src/main.rs")).await?;
///
/// // Batch extraction
/// let files = vec![
///     std::path::PathBuf::from("src/utils.rs"),
///     std::path::PathBuf::from("src/config.ts"),
/// ];
/// builder.extract_files(&files).await?;
///
/// // Access graph
/// println!("Graph has {} nodes", builder.graph().node_count());
///
/// // Persist to storage
/// builder.persist().await?;
/// # Ok(())
/// # }
/// ```
pub struct DependencyGraphBuilder {
    /// The dependency graph being built.
    graph: DependencyGraph,

    /// Storage backend for persistence.
    storage: Box<dyn StorageBackend>,

    /// Language-specific extractors.
    rust_extractor: RustDependencyExtractor,
    typescript_extractor: TypeScriptDependencyExtractor,
    python_extractor: PythonDependencyExtractor,
    go_extractor: GoDependencyExtractor,
}

impl DependencyGraphBuilder {
    /// Creates a new dependency graph builder with the given storage backend.
    ///
    /// # Arguments
    ///
    /// * `storage` - Storage backend for persisting fingerprints and graph data
    ///
    /// # Examples
    ///
    /// ```rust
    /// use thread_flow::incremental::dependency_builder::DependencyGraphBuilder;
    /// use thread_flow::incremental::storage::InMemoryStorage;
    ///
    /// let storage = Box::new(InMemoryStorage::new());
    /// let builder = DependencyGraphBuilder::new(storage);
    /// ```
    pub fn new(storage: Box<dyn StorageBackend>) -> Self {
        Self {
            graph: DependencyGraph::new(),
            storage,
            rust_extractor: RustDependencyExtractor::new(),
            typescript_extractor: TypeScriptDependencyExtractor::new(),
            python_extractor: PythonDependencyExtractor::new(),
            go_extractor: GoDependencyExtractor::new(None), // No module path by default
        }
    }

    /// Accesses the built dependency graph.
    ///
    /// Returns a reference to the [`DependencyGraph`] constructed from
    /// all extracted files.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use thread_flow::incremental::dependency_builder::DependencyGraphBuilder;
    /// # use thread_flow::incremental::storage::InMemoryStorage;
    /// let storage = Box::new(InMemoryStorage::new());
    /// let builder = DependencyGraphBuilder::new(storage);
    /// let graph = builder.graph();
    /// assert_eq!(graph.node_count(), 0); // Empty graph initially
    /// ```
    pub fn graph(&self) -> &DependencyGraph {
        &self.graph
    }

    /// Extracts dependencies from a single file.
    ///
    /// Detects the file's language, uses the appropriate extractor,
    /// and adds the resulting edges to the dependency graph.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the source file to analyze
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file's language is not supported
    /// - The file cannot be read
    /// - Dependency extraction fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use thread_flow::incremental::dependency_builder::DependencyGraphBuilder;
    /// # use thread_flow::incremental::storage::InMemoryStorage;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let storage = Box::new(InMemoryStorage::new());
    /// let mut builder = DependencyGraphBuilder::new(storage);
    ///
    /// builder.extract_file(std::path::Path::new("src/main.rs")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn extract_file(&mut self, file_path: &Path) -> Result<(), BuildError> {
        // Detect language
        let language = LanguageDetector::detect_language(file_path)
            .ok_or_else(|| BuildError::UnsupportedLanguage(file_path.to_path_buf()))?;

        debug!(
            "Extracting dependencies from {:?} ({:?})",
            file_path, language
        );

        // Read file contents
        let content = tokio::fs::read(file_path)
            .await
            .map_err(|error| BuildError::IoError {
                file: file_path.to_path_buf(),
                error,
            })?;

        // Convert to UTF-8 string for extractors
        let source = String::from_utf8_lossy(&content);

        // Compute fingerprint and add node
        let fingerprint = AnalysisDefFingerprint::new(&content);
        self.graph
            .nodes
            .insert(file_path.to_path_buf(), fingerprint);

        // Extract dependencies using language-specific extractor
        let edges = match language {
            Language::Rust => self
                .rust_extractor
                .extract_dependency_edges(&source, file_path)?,

            Language::TypeScript | Language::JavaScript => self
                .typescript_extractor
                .extract_dependency_edges(&source, file_path)?,

            Language::Python => self
                .python_extractor
                .extract_dependency_edges(&source, file_path)?,

            Language::Go => self
                .go_extractor
                .extract_dependency_edges(&source, file_path)?,
        };

        // Add edges to graph
        for edge in edges {
            self.graph.add_edge(edge);
        }

        Ok(())
    }

    /// Extracts dependencies from multiple files in batch.
    ///
    /// Processes all files and continues on individual extraction failures.
    /// Returns an error only if all extractions fail.
    ///
    /// # Arguments
    ///
    /// * `files` - Slice of file paths to analyze
    ///
    /// # Errors
    ///
    /// Returns the last error encountered if ANY extraction fails.
    /// Individual extraction errors are logged as warnings.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use thread_flow::incremental::dependency_builder::DependencyGraphBuilder;
    /// # use thread_flow::incremental::storage::InMemoryStorage;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let storage = Box::new(InMemoryStorage::new());
    /// let mut builder = DependencyGraphBuilder::new(storage);
    ///
    /// let files = vec![
    ///     std::path::PathBuf::from("src/main.rs"),
    ///     std::path::PathBuf::from("src/lib.rs"),
    /// ];
    /// builder.extract_files(&files).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn extract_files(&mut self, files: &[PathBuf]) -> Result<(), BuildError> {
        let mut last_error = None;
        let mut success_count = 0;

        for file in files {
            match self.extract_file(file).await {
                Ok(_) => success_count += 1,
                Err(e) => {
                    warn!("Failed to extract {}: {}", file.display(), e);
                    last_error = Some(e);
                }
            }
        }

        debug!(
            "Batch extraction: {}/{} files succeeded",
            success_count,
            files.len()
        );

        // Return error only if we had failures
        if let Some(err) = last_error {
            if success_count == 0 {
                // All failed - propagate error
                return Err(err);
            }
            // Some succeeded - log warning but continue
            warn!(
                "Batch extraction: {}/{} files failed",
                files.len() - success_count,
                files.len()
            );
        }

        Ok(())
    }

    /// Persists the dependency graph to the storage backend.
    ///
    /// Saves all fingerprints and edges to the configured storage.
    ///
    /// # Errors
    ///
    /// Returns an error if storage operations fail.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use thread_flow::incremental::dependency_builder::DependencyGraphBuilder;
    /// # use thread_flow::incremental::storage::InMemoryStorage;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let storage = Box::new(InMemoryStorage::new());
    /// let mut builder = DependencyGraphBuilder::new(storage);
    ///
    /// // ... extract files ...
    ///
    /// // Persist to storage
    /// builder.persist().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn persist(&self) -> Result<(), BuildError> {
        debug!(
            "Persisting graph: {} nodes, {} edges",
            self.graph.node_count(),
            self.graph.edge_count()
        );

        // Save the full graph
        self.storage.save_full_graph(&self.graph).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::incremental::storage::InMemoryStorage;

    #[test]
    fn test_language_detection() {
        assert_eq!(
            LanguageDetector::detect_language(Path::new("file.rs")),
            Some(Language::Rust)
        );
        assert_eq!(
            LanguageDetector::detect_language(Path::new("file.ts")),
            Some(Language::TypeScript)
        );
        assert_eq!(
            LanguageDetector::detect_language(Path::new("file.tsx")),
            Some(Language::TypeScript)
        );
        assert_eq!(
            LanguageDetector::detect_language(Path::new("file.js")),
            Some(Language::JavaScript)
        );
        assert_eq!(
            LanguageDetector::detect_language(Path::new("file.jsx")),
            Some(Language::JavaScript)
        );
        assert_eq!(
            LanguageDetector::detect_language(Path::new("file.py")),
            Some(Language::Python)
        );
        assert_eq!(
            LanguageDetector::detect_language(Path::new("file.go")),
            Some(Language::Go)
        );

        // Unsupported
        assert_eq!(
            LanguageDetector::detect_language(Path::new("file.java")),
            None
        );

        // Case insensitive
        assert_eq!(
            LanguageDetector::detect_language(Path::new("FILE.RS")),
            Some(Language::Rust)
        );
    }

    #[test]
    fn test_builder_creation() {
        let storage = Box::new(InMemoryStorage::new());
        let builder = DependencyGraphBuilder::new(storage);

        assert_eq!(builder.graph().node_count(), 0);
        assert_eq!(builder.graph().edge_count(), 0);
    }
}
