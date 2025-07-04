//! High-performance tree-sitter based parsing engine
//!
//! This crate provides the core parsing infrastructure that's optimized
//! for speed and supports live language server capabilities.

use thread_core::*;
use dashmap::DashMap;
use rayon::prelude::*;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

pub mod registry;
pub mod incremental;
pub mod parallel;

pub use registry::*;
pub use incremental::*;

/// High-performance parsing engine with language registry
pub struct ParseEngine {
    /// Registry of language parsers - thread-safe concurrent access
    parsers: DashMap<String, Arc<dyn LanguageParser>>,
    /// Custom query extractors
    extractors: DashMap<String, Arc<dyn QueryExtractor>>,
    /// Configuration
    config: ParseConfig,
}

impl ParseEngine {
    /// Create a new parsing engine
    pub fn new() -> Self {
        Self {
            parsers: DashMap::new(),
            extractors: DashMap::new(),
            config: ParseConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: ParseConfig) -> Self {
        Self {
            parsers: DashMap::new(),
            extractors: DashMap::new(),
            config,
        }
    }

    /// Register a language parser
    pub fn register_parser<P>(&self, parser: P) -> Result<()>
    where
        P: LanguageParser + 'static,
    {
        let language_id = parser.language_id().to_string();
        self.parsers.insert(language_id, Arc::new(parser));
        Ok(())
    }

    /// Register a custom query extractor
    pub fn register_extractor<E>(&self, extractor: E) -> Result<()>
    where
        E: QueryExtractor + 'static,
    {
        let name = extractor.name().to_string();
        self.extractors.insert(name, Arc::new(extractor));
        Ok(())
    }

    /// Parse a single file
    pub fn parse_file(&self, content: &str, file_path: &Path) -> Result<FileParseResult> {
        let start_time = Instant::now();

        // Check file size limits
        if content.len() > self.config.max_file_size_mb * 1024 * 1024 {
            return Err(ThreadError::FileTooLarge {
                size_mb: content.len() / (1024 * 1024),
                limit_mb: self.config.max_file_size_mb,
            });
        }

        // Find appropriate parser
        let parser = self.find_parser_for_file(file_path)?;

        // Parse the file
        let mut result = parser.parse_file(content, file_path)?;
        result.parse_time_ms = start_time.elapsed().as_millis() as u64;

        // Apply custom extractors if any
        self.apply_extractors(&mut result, content)?;

        Ok(result)
    }

    /// Parse multiple files in parallel
    pub fn parse_files(&self, files: &[(String, &Path)]) -> Result<Vec<FileParseResult>> {
        if self.config.parallel_parsing && files.len() > 1 {
            // Parallel processing for multiple files
            files
                .par_iter()
                .map(|(content, path)| self.parse_file(content, path))
                .collect()
        } else {
            // Sequential processing
            files
                .iter()
                .map(|(content, path)| self.parse_file(content, path))
                .collect()
        }
    }

    /// Parse an entire project
    pub fn parse_project(&self, project_files: &[(String, &Path)]) -> Result<ProjectParseResult> {
        let start_time = Instant::now();

        // Parse all files
        let files = self.parse_files(project_files)?;

        // Build dependency graph
        let dependency_graph = self.build_dependency_graph(&files)?;

        // Collect statistics
        let statistics = self.collect_statistics(&files);

        Ok(ProjectParseResult {
            project_id: "".to_string(), // Will be set by caller
            files,
            dependency_graph,
            total_parse_time_ms: start_time.elapsed().as_millis() as u64,
            statistics,
        })
    }

    /// Parse incrementally for live updates
    pub fn parse_incremental(
        &self,
        old_content: &str,
        new_content: &str,
        file_path: &Path,
    ) -> Result<FileParseResult> {
        if !self.config.incremental_mode {
            return self.parse_file(new_content, file_path);
        }

        let parser = self.find_parser_for_file(file_path)?;
        parser.parse_incremental(old_content, new_content, file_path)
    }

    /// Get list of supported languages
    pub fn supported_languages(&self) -> Vec<String> {
        self.parsers.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Check if a language is supported
    pub fn supports_language(&self, language: &str) -> bool {
        self.parsers.contains_key(language)
    }

    /// Find parser for a given file
    fn find_parser_for_file(&self, file_path: &Path) -> Result<Arc<dyn LanguageParser>> {
        for parser_ref in self.parsers.iter() {
            if parser_ref.value().can_parse(file_path) {
                return Ok(parser_ref.value().clone());
            }
        }

        Err(ThreadError::UnsupportedLanguage(
            file_path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("unknown")
                .to_string()
        ))
    }

    /// Apply custom extractors to parse results
    fn apply_extractors(&self, result: &mut FileParseResult, content: &str) -> Result<()> {
        // This would apply any registered custom extractors
        // For now, we'll keep it simple
        Ok(())
    }

    /// Build dependency graph from parsed files
    fn build_dependency_graph(&self, files: &[FileParseResult]) -> Result<DependencyGraph> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Build map of exports for quick lookup
        let mut exports_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();

        for file in files {
            nodes.push(file.file_path.clone());

            // Map exports to files
            for export in &file.exports {
                exports_map.insert(export.name.clone(), file.file_path.clone());
            }
        }

        // Build edges based on imports
        for file in files {
            for import in &file.imports {
                if let Some(target_file) = exports_map.get(&import.module) {
                    edges.push((file.file_path.clone(), target_file.clone()));
                }
            }
        }

        Ok(DependencyGraph { nodes, edges })
    }

    /// Collect parsing statistics
    fn collect_statistics(&self, files: &[FileParseResult]) -> ParseStatistics {
        let mut elements_by_kind = std::collections::HashMap::new();
        let mut files_by_language = std::collections::HashMap::new();
        let mut total_elements = 0;

        for file in files {
            // Count elements by kind
            for element in &file.elements {
                *elements_by_kind.entry(element.kind.clone()).or_insert(0) += 1;
                total_elements += 1;
            }

            // Count files by language
            *files_by_language.entry(file.language.clone()).or_insert(0) += 1;
        }

        ParseStatistics {
            total_files: files.len(),
            total_elements,
            elements_by_kind,
            files_by_language,
        }
    }
}

impl Default for ParseEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder pattern for easy engine configuration
pub struct ParseEngineBuilder {
    config: ParseConfig,
    parsers: Vec<Box<dyn LanguageParser>>,
    extractors: Vec<Box<dyn QueryExtractor>>,
}

impl ParseEngineBuilder {
    pub fn new() -> Self {
        Self {
            config: ParseConfig::default(),
            parsers: Vec::new(),
            extractors: Vec::new(),
        }
    }

    pub fn with_config(mut self, config: ParseConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_parser<P>(mut self, parser: P) -> Self
    where
        P: LanguageParser + 'static,
    {
        self.parsers.push(Box::new(parser));
        self
    }

    pub fn with_extractor<E>(mut self, extractor: E) -> Self
    where
        E: QueryExtractor + 'static,
    {
        self.extractors.push(Box::new(extractor));
        self
    }

    pub fn build(self) -> Result<ParseEngine> {
        let engine = ParseEngine::with_config(self.config);

        // Register all parsers
        for parser in self.parsers {
            let language_id = parser.language_id().to_string();
            engine.parsers.insert(language_id, Arc::from(parser));
        }

        // Register all extractors
        for extractor in self.extractors {
            let name = extractor.name().to_string();
            engine.extractors.insert(name, Arc::from(extractor));
        }

        Ok(engine)
    }
}

impl Default for ParseEngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}
