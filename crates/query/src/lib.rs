//! Query system for extracting code elements from tree-sitter parse trees
//!
//! This crate provides a flexible, high-performance query system that can
//! extract semantic information from any language supported by tree-sitter.

use thread_core::*;
use std::collections::HashMap;
use tree_sitter::{Query, QueryCursor, Tree, Node};

pub mod patterns;
pub mod extractors;

pub use patterns::*;
pub use extractors::*;

/// Query-based code element extractor
pub struct QueryExtractor {
    /// Compiled queries for different element types
    queries: HashMap<ElementKind, Query>,
    /// Language-specific query patterns
    patterns: QueryPatterns,
    /// Reusable query cursor for performance
    cursor: QueryCursor,
}

impl QueryExtractor {
    /// Create a new query extractor for a specific language
    pub fn new(language: tree_sitter::Language, patterns: QueryPatterns) -> Result<Self> {
        let mut queries = HashMap::new();

        // Compile all query patterns
        for (kind, pattern) in patterns.patterns.iter() {
            let query = Query::new(language, pattern)
                .map_err(|e| ThreadError::TreeSitter(format!("Query compilation failed: {}", e)))?;
            queries.insert(kind.clone(), query);
        }

        Ok(Self {
            queries,
            patterns,
            cursor: QueryCursor::new(),
        })
    }

    /// Extract all code elements from a parse tree
    pub fn extract_elements(&mut self, tree: &Tree, source: &str, file_path: &str) -> Result<Vec<CodeElement>> {
        let mut elements = Vec::new();
        let root_node = tree.root_node();

        // Extract each type of element
        for (kind, query) in &self.queries {
            let matches = self.cursor.matches(query, root_node, source.as_bytes());

            for m in matches {
                if let Some(element) = self.extract_element_from_match(
                    kind,
                    &m,
                    query,
                    source,
                    file_path
                )? {
                    elements.push(element);
                }
            }
        }

        // Sort by location for consistent output
        elements.sort_by_key(|e| (e.location.start_byte, e.location.end_byte));

        Ok(elements)
    }

    /// Extract a single element from a query match
    fn extract_element_from_match(
        &self,
        kind: &ElementKind,
        m: &tree_sitter::QueryMatch,
        query: &Query,
        source: &str,
        file_path: &str,
    ) -> Result<Option<CodeElement>> {
        let capture_names = query.capture_names();
        let mut captures = HashMap::new();

        // Collect all captures
        for capture in m.captures {
            let name = &capture_names[capture.index as usize];
            let node = capture.node;
            let text = node.utf8_text(source.as_bytes())
                .map_err(|e| ThreadError::TreeSitter(format!("UTF-8 error: {}", e)))?;

            captures.insert(name.clone(), (node, text.to_string()));
        }

        // Extract based on element kind
        match kind {
            ElementKind::Function => self.extract_function(&captures, source, file_path),
            ElementKind::Class => self.extract_class(&captures, source, file_path),
            ElementKind::Struct => self.extract_struct(&captures, source, file_path),
            ElementKind::Constant => self.extract_constant(&captures, source, file_path),
            ElementKind::Import => self.extract_import(&captures, source, file_path),
            ElementKind::Export => self.extract_export(&captures, source, file_path),
            _ => Ok(None), // Handle other kinds as needed
        }
    }

    /// Extract function information
    fn extract_function(
        &self,
        captures: &HashMap<String, (Node, String)>,
        source: &str,
        file_path: &str,
    ) -> Result<Option<CodeElement>> {
        let name_capture = captures.get("name")
            .ok_or_else(|| ThreadError::TreeSitter("Function name not captured".to_string()))?;

        let name = name_capture.1.clone();
        let node = name_capture.0;

        // Extract signature (this would be language-specific)
        let signature = self.extract_function_signature(captures, source)?;

        // Extract parameters
        let parameters = self.extract_parameters(captures, source)?;

        // Extract return type if available
        let return_type = captures.get("return_type").map(|(_, text)| text.clone());

        // Extract docstring if available
        let docstring = captures.get("docstring").map(|(_, text)| text.clone());

        // Check visibility
        let visibility = self.extract_visibility(captures);

        // Create location
        let location = SourceLocation::from_node(&node, file_path.to_string(), source);

        // Generate content hash
        let content_hash = ContentHash::from_content(&signature);

        // Create element ID
        let element_id = ElementId(format!("{}::{}", file_path, name));

        let element = CodeElement {
            id: element_id,
            kind: ElementKind::Function,
            name,
            signature,
            location,
            content_hash,
            dependencies: Vec::new(), // TODO: Extract dependencies
            metadata: ElementMetadata {
                visibility,
                is_async: self.is_async_function(captures),
                is_generic: self.is_generic_function(captures),
                docstring,
                annotations: self.extract_annotations(captures),
                return_type,
                parameters,
                extra: HashMap::new(),
            },
        };

        Ok(Some(element))
    }

    /// Extract class information
    fn extract_class(
        &self,
        captures: &HashMap<String, (Node, String)>,
        source: &str,
        file_path: &str,
    ) -> Result<Option<CodeElement>> {
        // Similar to extract_function but for classes
        // This would be implemented based on language-specific patterns
        Ok(None) // Placeholder
    }

    /// Extract struct information
    fn extract_struct(
        &self,
        captures: &HashMap<String, (Node, String)>,
        source: &str,
        file_path: &str,
    ) -> Result<Option<CodeElement>> {
        // Similar pattern for structs
        Ok(None) // Placeholder
    }

    /// Extract constant information
    fn extract_constant(
        &self,
        captures: &HashMap<String, (Node, String)>,
        source: &str,
        file_path: &str,
    ) -> Result<Option<CodeElement>> {
        // Similar pattern for constants
        Ok(None) // Placeholder
    }

    /// Extract import information
    fn extract_import(
        &self,
        captures: &HashMap<String, (Node, String)>,
        source: &str,
        file_path: &str,
    ) -> Result<Option<CodeElement>> {
        // Import extraction logic
        Ok(None) // Placeholder
    }

    /// Extract export information
    fn extract_export(
        &self,
        captures: &HashMap<String, (Node, String)>,
        source: &str,
        file_path: &str,
    ) -> Result<Option<CodeElement>> {
        // Export extraction logic
        Ok(None) // Placeholder
    }

    // Helper methods for extracting specific information

    fn extract_function_signature(
        &self,
        captures: &HashMap<String, (Node, String)>,
        source: &str,
    ) -> Result<String> {
        // Language-specific signature extraction
        if let Some((_, name)) = captures.get("name") {
            if let Some((_, params)) = captures.get("parameters") {
                return Ok(format!("{}({})", name, params));
            }
            return Ok(name.clone());
        }

        Err(ThreadError::TreeSitter("Could not extract function signature".to_string()))
    }

    fn extract_parameters(
        &self,
        captures: &HashMap<String, (Node, String)>,
        source: &str,
    ) -> Result<Vec<Parameter>> {
        // Parse parameter list - this would be language-specific
        Ok(Vec::new()) // Placeholder
    }

    fn extract_visibility(&self, captures: &HashMap<String, (Node, String)>) -> Option<Visibility> {
        // Check for visibility modifiers
        if captures.contains_key("public") {
            Some(Visibility::Public)
        } else if captures.contains_key("private") {
            Some(Visibility::Private)
        } else if captures.contains_key("protected") {
            Some(Visibility::Protected)
        } else {
            None
        }
    }

    fn is_async_function(&self, captures: &HashMap<String, (Node, String)>) -> bool {
        captures.contains_key("async")
    }

    fn is_generic_function(&self, captures: &HashMap<String, (Node, String)>) -> bool {
        captures.contains_key("type_parameters")
    }

    fn extract_annotations(&self, captures: &HashMap<String, (Node, String)>) -> Vec<String> {
        // Extract decorators, annotations, attributes, etc.
        Vec::new() // Placeholder
    }
}

/// Language-specific query patterns
#[derive(Debug, Clone)]
pub struct QueryPatterns {
    pub patterns: HashMap<ElementKind, String>,
}

impl QueryPatterns {
    /// Create empty patterns
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
        }
    }

    /// Add a query pattern for an element kind
    pub fn add_pattern(&mut self, kind: ElementKind, pattern: String) {
        self.patterns.insert(kind, pattern);
    }

    /// Create patterns from a configuration
    pub fn from_config(config: &serde_json::Value) -> Result<Self> {
        // Parse patterns from JSON configuration
        let mut patterns = HashMap::new();

        if let Some(obj) = config.as_object() {
            for (key, value) in obj {
                if let Some(pattern_str) = value.as_str() {
                    // Convert string key to ElementKind
                    let kind = match key.as_str() {
                        "function" => ElementKind::Function,
                        "class" => ElementKind::Class,
                        "struct" => ElementKind::Struct,
                        "constant" => ElementKind::Constant,
                        "import" => ElementKind::Import,
                        "export" => ElementKind::Export,
                        _ => ElementKind::Custom(key.clone()),
                    };
                    patterns.insert(kind, pattern_str.to_string());
                }
            }
        }

        Ok(Self { patterns })
    }
}
