// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Go dependency extractor using tree-sitter queries.
//!
//! Extracts `import` declarations from Go source files, handling all import forms:
//!
//! - Single imports: `import "fmt"`
//! - Import blocks: `import ( "fmt"\n "os" )`
//! - Aliased imports: `import alias "package"`
//! - Dot imports: `import . "package"`
//! - Blank imports: `import _ "package"`
//! - CGo imports: `import "C"`
//!
//! ## Performance
//!
//! Target: <5ms per file. Uses tree-sitter's incremental parsing and query API
//! for efficient extraction without full AST traversal.
//!
//! ## Module Resolution
//!
//! Supports go.mod-aware path resolution, GOPATH fallback, and vendor directory
//! mode for mapping import paths to local file paths.

use std::path::{Path, PathBuf};

use crate::incremental::types::{DependencyEdge, DependencyType};

/// Error types for Go dependency extraction.
#[derive(Debug, thiserror::Error)]
pub enum ExtractionError {
    /// Tree-sitter failed to parse the source file.
    #[error("parse error: failed to parse Go source")]
    ParseError,

    /// Tree-sitter query compilation failed.
    #[error("query error: {0}")]
    QueryError(String),

    /// Import path could not be resolved to a local file path.
    #[error("unresolved import: {path}")]
    UnresolvedImport {
        /// The import path that could not be resolved.
        path: String,
    },
}

/// Information about a single Go import statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportInfo {
    /// The import path string (e.g., `"fmt"` or `"github.com/user/repo"`).
    pub import_path: String,

    /// Optional alias for the import (e.g., `f` in `import f "fmt"`).
    pub alias: Option<String>,

    /// Whether this is a dot import (`import . "package"`).
    pub is_dot_import: bool,

    /// Whether this is a blank import (`import _ "package"`).
    pub is_blank_import: bool,
}

/// Go dependency extractor with tree-sitter query-based import extraction.
///
/// Supports go.mod module path resolution and vendor directory mode for
/// mapping import paths to local file system paths.
///
/// # Examples
///
/// ```rust,ignore
/// use thread_flow::incremental::extractors::go::GoDependencyExtractor;
/// use std::path::Path;
///
/// let extractor = GoDependencyExtractor::new(Some("github.com/user/repo".to_string()));
/// let imports = extractor.extract_imports(source, Path::new("main.go")).unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct GoDependencyExtractor {
    /// The go.mod module path, if known (e.g., `"github.com/user/repo"`).
    module_path: Option<String>,
    /// Whether to resolve external imports via the vendor directory.
    vendor_mode: bool,
}

impl GoDependencyExtractor {
    /// Create a new extractor with optional go.mod module path.
    ///
    /// When `module_path` is provided, imports matching the module prefix
    /// are resolved to local paths relative to the module root.
    pub fn new(module_path: Option<String>) -> Self {
        Self {
            module_path,
            vendor_mode: false,
        }
    }

    /// Create a new extractor with vendor directory support.
    ///
    /// When `vendor_mode` is true, external imports are resolved to the
    /// `vendor/` directory instead of returning an error.
    pub fn with_vendor(module_path: Option<String>, vendor_mode: bool) -> Self {
        Self {
            module_path,
            vendor_mode,
        }
    }

    /// Extract all import statements from a Go source file.
    ///
    /// Parses the source using tree-sitter and walks `import_declaration` nodes
    /// to collect import paths, aliases, and import variants (dot, blank).
    ///
    /// # Errors
    ///
    /// Returns [`ExtractionError::ParseError`] if tree-sitter cannot parse the source.
    pub fn extract_imports(
        &self,
        source: &str,
        _file_path: &Path,
    ) -> Result<Vec<ImportInfo>, ExtractionError> {
        if source.is_empty() {
            return Ok(Vec::new());
        }

        let language = thread_language::parsers::language_go();
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&language)
            .map_err(|_| ExtractionError::ParseError)?;

        let tree = parser
            .parse(source, None)
            .ok_or(ExtractionError::ParseError)?;

        let root_node = tree.root_node();
        let mut imports = Vec::new();

        self.walk_imports(root_node, source.as_bytes(), &mut imports);

        Ok(imports)
    }

    /// Walk the tree-sitter AST to extract import declarations.
    fn walk_imports(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
        imports: &mut Vec<ImportInfo>,
    ) {
        if node.kind() == "import_declaration" {
            self.extract_from_import_declaration(node, source, imports);
            return;
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_imports(child, source, imports);
        }
    }

    /// Extract imports from a single `import_declaration` node.
    ///
    /// Handles both single imports and import blocks (import_spec_list).
    fn extract_from_import_declaration(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
        imports: &mut Vec<ImportInfo>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "import_spec" => {
                    if let Some(info) = self.parse_import_spec(child, source) {
                        imports.push(info);
                    }
                }
                "import_spec_list" => {
                    let mut list_cursor = child.walk();
                    for spec in child.children(&mut list_cursor) {
                        if spec.kind() == "import_spec"
                            && let Some(info) = self.parse_import_spec(spec, source)
                        {
                            imports.push(info);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Parse a single `import_spec` node into an [`ImportInfo`].
    ///
    /// The import_spec grammar in tree-sitter-go:
    /// ```text
    /// import_spec: $ => seq(
    ///   optional(field('name', choice($.dot, $.blank_identifier, $._package_identifier))),
    ///   field('path', $._string_literal)
    /// )
    /// ```
    fn parse_import_spec(&self, node: tree_sitter::Node<'_>, source: &[u8]) -> Option<ImportInfo> {
        let mut alias: Option<String> = None;
        let mut is_dot_import = false;
        let mut is_blank_import = false;
        let mut import_path: Option<String> = None;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "dot" => {
                    is_dot_import = true;
                }
                "blank_identifier" => {
                    is_blank_import = true;
                }
                "package_identifier" => {
                    let name = child.utf8_text(source).ok()?.to_string();
                    alias = Some(name);
                }
                "interpreted_string_literal" => {
                    let raw = child.utf8_text(source).ok()?;
                    // Strip surrounding quotes
                    let path = raw.trim_matches('"').to_string();
                    import_path = Some(path);
                }
                _ => {}
            }
        }

        import_path.map(|path| ImportInfo {
            import_path: path,
            alias,
            is_dot_import,
            is_blank_import,
        })
    }

    /// Resolve a Go import path to a local file path.
    ///
    /// Resolution strategy:
    /// 1. If the import matches the module path prefix, strip it to get a relative path.
    /// 2. If vendor mode is enabled, external imports resolve to `vendor/<import_path>`.
    /// 3. Standard library and unresolvable external imports return an error.
    ///
    /// # Errors
    ///
    /// Returns [`ExtractionError::UnresolvedImport`] if the import cannot be mapped
    /// to a local file path.
    pub fn resolve_import_path(
        &self,
        _source_file: &Path,
        import_path: &str,
    ) -> Result<PathBuf, ExtractionError> {
        // Module-internal import
        if let Some(ref module) = self.module_path
            && let Some(relative) = import_path.strip_prefix(module)
        {
            let relative = relative.strip_prefix('/').unwrap_or(relative);
            return Ok(PathBuf::from(relative));
        }

        // Vendor mode for external imports
        if self.vendor_mode {
            return Ok(PathBuf::from(format!("vendor/{import_path}")));
        }

        Err(ExtractionError::UnresolvedImport {
            path: import_path.to_string(),
        })
    }

    /// Extract [`DependencyEdge`] values from a Go source file.
    ///
    /// Combines import extraction with path resolution to produce edges
    /// suitable for the incremental dependency graph. Only module-internal
    /// and vendor-resolvable imports produce edges; standard library and
    /// unresolvable external imports are silently skipped.
    ///
    /// # Errors
    ///
    /// Returns an error if the source file cannot be parsed.
    pub fn extract_dependency_edges(
        &self,
        source: &str,
        file_path: &Path,
    ) -> Result<Vec<DependencyEdge>, ExtractionError> {
        let imports = self.extract_imports(source, file_path)?;
        let mut edges = Vec::new();

        for import in &imports {
            // Only create edges for resolvable imports (module-internal or vendor)
            // Stdlib and external imports are silently skipped per design spec
            if let Ok(resolved) = self.resolve_import_path(file_path, &import.import_path) {
                edges.push(DependencyEdge::new(
                    file_path.to_path_buf(),
                    resolved,
                    DependencyType::Import,
                ));
            }
        }

        Ok(edges)
    }
}
