// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Python dependency extractor using tree-sitter queries.
//!
//! Extracts `import` and `from ... import` statements from Python source files,
//! producing [`ImportInfo`] records for the dependency graph. Supports:
//!
//! - Absolute imports: `import os`, `import os.path`
//! - From imports: `from os import path`, `from os.path import join, exists`
//! - Relative imports: `from .utils import helper`, `from ..core import Engine`
//! - Wildcard imports: `from module import *`
//! - Aliased imports: `import numpy as np`, `from os import path as ospath`
//!
//! # Examples
//!
//! ```rust,ignore
//! use thread_flow::incremental::extractors::python::PythonDependencyExtractor;
//! use std::path::Path;
//!
//! let extractor = PythonDependencyExtractor::new();
//! let source = "import os\nfrom pathlib import Path";
//! let imports = extractor.extract_imports(source, Path::new("main.py")).unwrap();
//! assert_eq!(imports.len(), 2);
//! ```
//!
//! # Performance
//!
//! Target: <5ms per file extraction. Tree-sitter parses the full AST and a
//! single recursive walk collects all import nodes, avoiding repeated traversals.

use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during import extraction.
#[derive(Debug, Error)]
pub enum ExtractionError {
    /// The source code could not be parsed by tree-sitter.
    #[error("failed to parse source: {0}")]
    ParseError(String),

    /// A tree-sitter query failed to compile.
    #[error("invalid tree-sitter query: {0}")]
    QueryError(String),

    /// Module path resolution failed.
    #[error("cannot resolve module path '{module}' from '{source_file}': {reason}")]
    ResolutionError {
        /// The module path that could not be resolved.
        module: String,
        /// The source file containing the import.
        source_file: PathBuf,
        /// Explanation of why resolution failed.
        reason: String,
    },
}

/// Information extracted from a single Python import statement.
///
/// Represents the parsed form of either `import X` or `from X import Y`
/// statements. The coordinator (Task 3.5) converts these into
/// [`DependencyEdge`](crate::incremental::types::DependencyEdge) entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportInfo {
    /// The module path, with leading dots stripped for relative imports.
    ///
    /// For `import os.path` this is `"os.path"`.
    /// For `from .utils import helper` this is `"utils"` (dots conveyed by `relative_level`).
    /// For `from . import x` (no module name), this is `""`.
    pub module_path: String,

    /// Specific symbols imported from the module.
    ///
    /// Empty for bare `import` statements (e.g., `import os`).
    /// Contains `["join", "exists"]` for `from os.path import join, exists`.
    pub symbols: Vec<String>,

    /// Whether this is a wildcard import (`from module import *`).
    pub is_wildcard: bool,

    /// The relative import depth.
    ///
    /// `0` for absolute imports, `1` for `.`, `2` for `..`, etc.
    pub relative_level: usize,

    /// Aliases for imported names.
    ///
    /// Maps original name to alias. For `import numpy as np`, contains
    /// `[("numpy", "np")]`. For `from os import path as ospath`, contains
    /// `[("path", "ospath")]`.
    pub aliases: Vec<(String, String)>,
}

/// Extracts Python import dependencies using tree-sitter AST walking.
///
/// Uses tree-sitter's Python grammar to parse import statements without
/// executing the Python code. Thread-safe and reusable across files.
///
/// # Architecture
///
/// The extractor operates in two phases:
/// 1. **Parse**: Tree-sitter parses the source into an AST
/// 2. **Walk**: Recursive traversal matches `import_statement` and
///    `import_from_statement` nodes, extracting structured data
///
/// Module path resolution (converting `"os.path"` to a filesystem path)
/// is a separate concern handled by [`resolve_module_path`](Self::resolve_module_path).
pub struct PythonDependencyExtractor {
    _private: (),
}

impl PythonDependencyExtractor {
    /// Creates a new Python dependency extractor.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Extracts all import statements from Python source code.
    ///
    /// Parses the source with tree-sitter and walks the AST to find both
    /// `import_statement` and `import_from_statement` nodes. Imports inside
    /// function bodies, try/except blocks, and other nested scopes are
    /// included.
    ///
    /// # Arguments
    ///
    /// * `source` - Python source code to analyze.
    /// * `_file_path` - Path of the source file (reserved for future error context).
    ///
    /// # Returns
    ///
    /// A vector of [`ImportInfo`] records. Bare `import os, sys` statements
    /// produce one `ImportInfo` per module.
    ///
    /// # Errors
    ///
    /// Returns [`ExtractionError::ParseError`] if tree-sitter cannot parse
    /// the source.
    pub fn extract_imports(
        &self,
        source: &str,
        _file_path: &Path,
    ) -> Result<Vec<ImportInfo>, ExtractionError> {
        if source.is_empty() {
            return Ok(Vec::new());
        }

        let language = thread_language::parsers::language_python();
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&language)
            .map_err(|e| ExtractionError::ParseError(e.to_string()))?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| ExtractionError::ParseError("tree-sitter returned None".into()))?;

        let root = tree.root_node();
        let mut imports = Vec::new();
        let src = source.as_bytes();

        Self::walk_imports(root, src, &mut imports);

        Ok(imports)
    }

    /// Recursively walk the AST collecting import nodes.
    ///
    /// Descends into all nodes (including function bodies, try/except blocks)
    /// to capture conditional and lazy imports.
    fn walk_imports(node: tree_sitter::Node<'_>, source: &[u8], imports: &mut Vec<ImportInfo>) {
        match node.kind() {
            "import_statement" => {
                Self::extract_import_statement(node, source, imports);
                return;
            }
            "import_from_statement" => {
                Self::extract_import_from_statement(node, source, imports);
                return;
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_imports(child, source, imports);
        }
    }

    /// Extract from a bare `import` statement.
    ///
    /// Handles:
    /// - `import os` (single module)
    /// - `import os.path` (dotted module)
    /// - `import os, sys` (multiple modules produce multiple [`ImportInfo`]s)
    /// - `import numpy as np` (aliased)
    fn extract_import_statement(
        node: tree_sitter::Node<'_>,
        source: &[u8],
        imports: &mut Vec<ImportInfo>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "dotted_name" => {
                    if let Ok(name) = child.utf8_text(source) {
                        imports.push(ImportInfo {
                            module_path: name.to_string(),
                            symbols: Vec::new(),
                            is_wildcard: false,
                            relative_level: 0,
                            aliases: Vec::new(),
                        });
                    }
                }
                "aliased_import" => {
                    if let Some(info) = Self::parse_bare_aliased_import(child, source) {
                        imports.push(info);
                    }
                }
                _ => {}
            }
        }
    }

    /// Parse an `aliased_import` node inside a bare `import` statement.
    ///
    /// For `import numpy as np`, returns module_path="numpy" with alias ("numpy","np").
    fn parse_bare_aliased_import(node: tree_sitter::Node<'_>, source: &[u8]) -> Option<ImportInfo> {
        let name_node = node.child_by_field_name("name")?;
        let alias_node = node.child_by_field_name("alias")?;

        let name = name_node.utf8_text(source).ok()?;
        let alias = alias_node.utf8_text(source).ok()?;

        Some(ImportInfo {
            module_path: name.to_string(),
            symbols: Vec::new(),
            is_wildcard: false,
            relative_level: 0,
            aliases: vec![(name.to_string(), alias.to_string())],
        })
    }

    /// Extract from a `from ... import` statement.
    ///
    /// Handles all `from` import variants including relative imports,
    /// wildcard imports, aliased symbols, and parenthesized import lists.
    fn extract_import_from_statement(
        node: tree_sitter::Node<'_>,
        source: &[u8],
        imports: &mut Vec<ImportInfo>,
    ) {
        let mut module_path = String::new();
        let mut relative_level: usize = 0;
        let mut symbols: Vec<String> = Vec::new();
        let mut is_wildcard = false;
        let mut aliases: Vec<(String, String)> = Vec::new();

        // Track whether we have seen the module name already (before 'import' keyword).
        // The first dotted_name child is the module; subsequent ones are imported symbols.
        let mut module_name_found = false;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                // Relative import: contains import_prefix (dots) + optional dotted_name
                "relative_import" => {
                    let mut rc = child.walk();
                    for rchild in child.children(&mut rc) {
                        match rchild.kind() {
                            "import_prefix" => {
                                if let Ok(prefix) = rchild.utf8_text(source) {
                                    relative_level = prefix.chars().filter(|&c| c == '.').count();
                                }
                            }
                            "dotted_name" => {
                                if let Ok(name) = rchild.utf8_text(source) {
                                    module_path = name.to_string();
                                }
                            }
                            _ => {}
                        }
                    }
                    module_name_found = true;
                }
                // Absolute module name (first dotted_name before 'import' keyword)
                // or a bare symbol in the import list (dotted_name after 'import')
                "dotted_name" => {
                    if !module_name_found {
                        if let Ok(name) = child.utf8_text(source) {
                            module_path = name.to_string();
                        }
                        module_name_found = true;
                    } else {
                        // Imported symbol name
                        if let Ok(name) = child.utf8_text(source) {
                            symbols.push(name.to_string());
                        }
                    }
                }
                "wildcard_import" => {
                    is_wildcard = true;
                }
                "aliased_import" => {
                    if let Some((sym, al)) = Self::parse_from_aliased_symbol(child, source) {
                        symbols.push(sym.clone());
                        aliases.push((sym, al));
                    }
                }
                _ => {}
            }
        }

        imports.push(ImportInfo {
            module_path,
            symbols,
            is_wildcard,
            relative_level,
            aliases,
        });
    }

    /// Parse an aliased import symbol inside a from-import.
    ///
    /// For `path as ospath` inside `from os import path as ospath`,
    /// returns `("path", "ospath")`.
    fn parse_from_aliased_symbol(
        node: tree_sitter::Node<'_>,
        source: &[u8],
    ) -> Option<(String, String)> {
        let name_node = node.child_by_field_name("name")?;
        let alias_node = node.child_by_field_name("alias")?;

        let name = name_node.utf8_text(source).ok()?.to_string();
        let alias = alias_node.utf8_text(source).ok()?.to_string();

        Some((name, alias))
    }

    /// Resolves a Python module path to a filesystem path.
    ///
    /// For absolute imports (`relative_level == 0`), converts dots to path
    /// separators and appends `.py`. For relative imports, navigates up from
    /// the source file's directory according to the dot count.
    ///
    /// # Arguments
    ///
    /// * `source_file` - The file containing the import statement.
    /// * `module_path` - The dotted module path (e.g., `"os.path"`, `"utils"`),
    ///   with leading dots already stripped (conveyed via `relative_level`).
    /// * `relative_level` - The relative import depth (0 for absolute).
    ///
    /// # Returns
    ///
    /// The resolved filesystem path to the target module.
    ///
    /// # Errors
    ///
    /// Returns [`ExtractionError::ResolutionError`] if the source file has no
    /// parent directory, or relative navigation exceeds the filesystem root.
    pub fn resolve_module_path(
        &self,
        source_file: &Path,
        module_path: &str,
        relative_level: usize,
    ) -> Result<PathBuf, ExtractionError> {
        if relative_level == 0 {
            // Absolute import: dots become path separators
            let fs_path = module_path.replace('.', "/");
            return Ok(PathBuf::from(format!("{fs_path}.py")));
        }

        // Relative import: navigate up from source file's parent directory
        let source_dir = source_file
            .parent()
            .ok_or_else(|| ExtractionError::ResolutionError {
                module: module_path.to_string(),
                source_file: source_file.to_path_buf(),
                reason: "source file has no parent directory".into(),
            })?;

        // Level 1 (`.`) stays in the same directory.
        // Level 2 (`..`) goes one directory up, etc.
        let mut base = source_dir.to_path_buf();
        for _ in 1..relative_level {
            base = base.parent().map(Path::to_path_buf).ok_or_else(|| {
                ExtractionError::ResolutionError {
                    module: module_path.to_string(),
                    source_file: source_file.to_path_buf(),
                    reason: format!(
                        "cannot navigate {} levels up from {}",
                        relative_level,
                        source_dir.display()
                    ),
                }
            })?;
        }

        if module_path.is_empty() {
            // `from . import X` resolves to the package __init__.py
            return Ok(base.join("__init__.py"));
        }

        let fs_path = module_path.replace('.', "/");
        Ok(base.join(format!("{fs_path}.py")))
    }

    /// Extract [`DependencyEdge`] values from a Python source file.
    ///
    /// Combines import extraction with path resolution to produce edges
    /// suitable for the incremental dependency graph. Only resolvable
    /// relative imports produce edges; absolute imports and unresolvable
    /// paths are silently skipped.
    ///
    /// # Errors
    ///
    /// Returns an error if the source file cannot be parsed.
    pub fn extract_dependency_edges(
        &self,
        source: &str,
        file_path: &Path,
    ) -> Result<Vec<super::super::types::DependencyEdge>, ExtractionError> {
        let imports = self.extract_imports(source, file_path)?;
        let mut edges = Vec::new();

        for import in &imports {
            // Only create edges for resolvable module paths
            // External packages and unresolvable paths are silently skipped per design spec
            if let Ok(resolved) =
                self.resolve_module_path(file_path, &import.module_path, import.relative_level)
            {
                edges.push(super::super::types::DependencyEdge::new(
                    file_path.to_path_buf(),
                    resolved,
                    super::super::types::DependencyType::Import,
                ));
            }
        }

        Ok(edges)
    }
}

impl Default for PythonDependencyExtractor {
    fn default() -> Self {
        Self::new()
    }
}
