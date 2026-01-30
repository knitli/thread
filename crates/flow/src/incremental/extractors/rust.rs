// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Rust dependency extractor using tree-sitter AST traversal.
//!
//! Extracts `use` declarations and `pub use` re-exports from Rust source files,
//! producing [`RustImportInfo`] and [`ExportInfo`] records for the dependency
//! graph. Supports:
//!
//! - Simple imports: `use std::collections::HashMap;`
//! - Nested imports: `use std::collections::{HashMap, HashSet};`
//! - Wildcard imports: `use module::*;`
//! - Aliased imports: `use std::io::Result as IoResult;`
//! - Crate-relative: `use crate::core::Engine;`
//! - Super-relative: `use super::utils;`
//! - Self-relative: `use self::types::Config;`
//! - Re-exports: `pub use types::Config;`, `pub(crate) use internal::Helper;`
//!
//! # Examples
//!
//! ```rust,ignore
//! use thread_flow::incremental::extractors::rust::RustDependencyExtractor;
//! use std::path::Path;
//!
//! let extractor = RustDependencyExtractor::new();
//! let source = "use std::collections::HashMap;\nuse crate::config::Settings;";
//! let imports = extractor.extract_imports(source, Path::new("src/main.rs")).unwrap();
//! assert_eq!(imports.len(), 2);
//! ```
//!
//! # Performance
//!
//! Target: <5ms per file extraction. Tree-sitter parsing and AST traversal
//! operate in a single pass without backtracking.

use std::path::{Path, PathBuf};

/// Errors that can occur during Rust dependency extraction.
#[derive(Debug, thiserror::Error)]
pub enum ExtractionError {
    /// Tree-sitter failed to parse the Rust source file.
    #[error("parse error: failed to parse Rust source")]
    ParseError,

    /// Module path could not be resolved to a local file path.
    #[error("unresolved module: {module} from {source_file}: {reason}")]
    ResolutionError {
        /// The module path that could not be resolved.
        module: String,
        /// The source file containing the use statement.
        source_file: PathBuf,
        /// The reason resolution failed.
        reason: String,
    },
}

/// Visibility level of a Rust re-export (`pub use`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Visibility {
    /// `pub use` -- visible to all.
    Public,
    /// `pub(crate) use` -- visible within the crate.
    Crate,
    /// `pub(super) use` -- visible to the parent module.
    Super,
    /// `pub(in path) use` -- visible to a specific path.
    Restricted,
}

/// Information extracted from a single Rust `use` declaration.
///
/// Represents the parsed form of a `use` statement. The coordinator (Task 3.5)
/// converts these into [`DependencyEdge`](crate::incremental::types::DependencyEdge)
/// entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustImportInfo {
    /// The module path as written in the source code, excluding the final
    /// symbol(s).
    ///
    /// For `use std::collections::HashMap` this is `"std::collections"`.
    /// For `use crate::config::Settings` this is `"crate::config"`.
    /// For `use serde;` (bare crate import) this is `"serde"`.
    pub module_path: String,

    /// Specific symbols imported from the module.
    ///
    /// Contains `["HashMap"]` for `use std::collections::HashMap`.
    /// Contains `["HashMap", "HashSet"]` for `use std::collections::{HashMap, HashSet}`.
    /// Empty for bare imports like `use serde;` or wildcard imports.
    pub symbols: Vec<String>,

    /// Whether this is a wildcard import (`use module::*`).
    pub is_wildcard: bool,

    /// Aliases for imported names.
    ///
    /// Maps original name to alias. For `use std::io::Result as IoResult`,
    /// contains `[("Result", "IoResult")]`.
    pub aliases: Vec<(String, String)>,
}

/// Information extracted from a Rust `pub use` re-export.
///
/// Represents a single re-exported symbol. For `pub use types::{Config, Settings}`,
/// two `ExportInfo` records are produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportInfo {
    /// The name of the re-exported symbol.
    ///
    /// For `pub use types::Config` this is `"Config"`.
    /// For `pub use module::*` this is `"*"`.
    pub symbol_name: String,

    /// The source module path of the re-export.
    ///
    /// For `pub use types::Config` this is `"types"`.
    pub module_path: String,

    /// The visibility level of this re-export.
    pub visibility: Visibility,
}

/// Extracts Rust import and export dependencies using tree-sitter AST traversal.
///
/// Uses tree-sitter's Rust grammar to parse `use` and `pub use` declarations
/// without executing Rust code. Thread-safe and reusable across files.
///
/// # Architecture
///
/// The extractor operates in two phases:
/// 1. **Parse**: Tree-sitter parses the source into an AST
/// 2. **Walk**: Recursive traversal extracts `use_declaration` nodes and their
///    nested structure (scoped identifiers, use lists, wildcards, aliases)
///
/// Module path resolution (converting `"crate::config"` to `"src/config.rs"`)
/// is handled separately by [`resolve_module_path`](Self::resolve_module_path).
pub struct RustDependencyExtractor {
    _private: (),
}

impl RustDependencyExtractor {
    /// Creates a new Rust dependency extractor.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Parse Rust source code into a tree-sitter tree.
    fn parse_source(source: &str) -> Result<tree_sitter::Tree, ExtractionError> {
        let language = thread_language::parsers::language_rust();
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&language)
            .map_err(|_| ExtractionError::ParseError)?;
        parser
            .parse(source, None)
            .ok_or(ExtractionError::ParseError)
    }

    /// Extracts all `use` declarations from Rust source code.
    ///
    /// Parses the source with tree-sitter and walks the AST to find all
    /// `use_declaration` nodes. Both public and private `use` statements are
    /// returned as imports (the caller may filter by visibility if needed).
    ///
    /// # Arguments
    ///
    /// * `source` - Rust source code to analyze.
    /// * `_file_path` - Path of the source file (reserved for error context).
    ///
    /// # Returns
    ///
    /// A vector of [`RustImportInfo`] records, one per `use` declaration.
    ///
    /// # Errors
    ///
    /// Returns [`ExtractionError::ParseError`] if tree-sitter cannot parse
    /// the source.
    pub fn extract_imports(
        &self,
        source: &str,
        _file_path: &Path,
    ) -> Result<Vec<RustImportInfo>, ExtractionError> {
        if source.is_empty() {
            return Ok(Vec::new());
        }

        let tree = Self::parse_source(source)?;
        let root = tree.root_node();
        let src = source.as_bytes();
        let mut imports = Vec::new();

        self.walk_use_declarations(root, src, &mut imports);
        self.walk_mod_declarations(root, src, &mut imports);

        Ok(imports)
    }

    /// Extracts all `pub use` re-export declarations from Rust source code.
    ///
    /// Only public or restricted-visibility `use` statements are returned.
    ///
    /// # Arguments
    ///
    /// * `source` - Rust source code to analyze.
    /// * `_file_path` - Path of the source file (reserved for error context).
    ///
    /// # Returns
    ///
    /// A vector of [`ExportInfo`] records, one per re-exported symbol.
    /// For `pub use types::{Config, Settings}`, two records are returned.
    ///
    /// # Errors
    ///
    /// Returns [`ExtractionError::ParseError`] if tree-sitter cannot parse
    /// the source.
    pub fn extract_exports(
        &self,
        source: &str,
        _file_path: &Path,
    ) -> Result<Vec<ExportInfo>, ExtractionError> {
        if source.is_empty() {
            return Ok(Vec::new());
        }

        let tree = Self::parse_source(source)?;
        let root = tree.root_node();
        let src = source.as_bytes();
        let mut exports = Vec::new();

        self.walk_export_declarations(root, src, &mut exports);

        Ok(exports)
    }

    /// Resolves a Rust module path to a filesystem path.
    ///
    /// Handles the three Rust-specific path prefixes:
    /// - `crate::` - resolves from the `src/` root of the crate
    /// - `super::` - resolves from the parent module directory
    /// - `self::` - resolves from the current module directory
    ///
    /// External crate paths (e.g., `std::collections`) cannot be resolved
    /// to local files and return an error.
    ///
    /// # Arguments
    ///
    /// * `source_file` - The file containing the `use` statement.
    /// * `module_path` - The module path (e.g., `"crate::config"`, `"super::utils"`).
    ///
    /// # Returns
    ///
    /// The resolved filesystem path to the target module file (e.g., `src/config.rs`).
    ///
    /// # Errors
    ///
    /// Returns [`ExtractionError::ResolutionError`] if:
    /// - The path is an external crate (no `crate::`, `super::`, or `self::` prefix)
    /// - The source file has no parent directory for `super::` resolution
    pub fn resolve_module_path(
        &self,
        source_file: &Path,
        module_path: &str,
    ) -> Result<PathBuf, ExtractionError> {
        if let Some(rest) = module_path.strip_prefix("crate::") {
            // crate:: resolves from src/ root
            let relative = rest.replace("::", "/");
            return Ok(PathBuf::from(format!("src/{relative}.rs")));
        }

        if let Some(rest) = module_path.strip_prefix("super::") {
            // super:: resolves relative to the parent module
            let super_dir = self.super_directory(source_file)?;
            let relative = rest.replace("::", "/");
            return Ok(super_dir.join(format!("{relative}.rs")));
        }

        if module_path == "super" {
            // Bare `super` -- resolve to the parent module itself
            let super_dir = self.super_directory(source_file)?;
            return Ok(super_dir.join("mod.rs"));
        }

        if let Some(rest) = module_path.strip_prefix("self::") {
            // self:: resolves from current module directory
            let dir = self.module_directory(source_file)?;
            let relative = rest.replace("::", "/");
            return Ok(dir.join(format!("{relative}.rs")));
        }

        // Simple module name without prefix (e.g., `mod lib;` in main.rs)
        // Resolves to sibling file (lib.rs) or directory module (lib/mod.rs)
        if !module_path.contains("::") && !module_path.is_empty() {
            let dir = self.module_directory(source_file)?;
            // Return sibling file path (lib.rs)
            // Note: Could also be lib/mod.rs, but we prefer the simpler form
            return Ok(dir.join(format!("{module_path}.rs")));
        }

        // External crate -- cannot resolve to local file
        Err(ExtractionError::ResolutionError {
            module: module_path.to_string(),
            source_file: source_file.to_path_buf(),
            reason: "external crate path cannot be resolved to a local file".to_string(),
        })
    }

    /// Extract [`DependencyEdge`] values from a Rust source file.
    ///
    /// Combines import extraction with path resolution to produce edges
    /// suitable for the incremental dependency graph. Only resolvable
    /// internal imports produce edges; external crates are silently skipped.
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
            // External crates are silently skipped per design spec
            if let Ok(resolved) = self.resolve_module_path(file_path, &import.module_path) {
                // Create symbol-level tracking if specific symbols are imported
                let symbol = if !import.symbols.is_empty() && !import.is_wildcard {
                    // For now, track the first symbol (could be enhanced to create multiple edges)
                    Some(super::super::types::SymbolDependency {
                        from_symbol: import.symbols[0].clone(),
                        to_symbol: import.symbols[0].clone(),
                        kind: super::super::types::SymbolKind::Module,
                        strength: super::super::types::DependencyStrength::Strong,
                    })
                } else {
                    None
                };

                let mut edge = super::super::types::DependencyEdge::new(
                    file_path.to_path_buf(),
                    resolved,
                    super::super::types::DependencyType::Import,
                );
                edge.symbol = symbol;

                edges.push(edge);
            }
        }

        Ok(edges)
    }

    /// Determine the module directory for a source file.
    ///
    /// For `mod.rs` or `lib.rs`, the module *is* the directory (these files
    /// define the module that contains sibling files). So `self::` resolves
    /// to the same directory and `super::` resolves to the parent directory.
    ///
    /// For regular files like `auth.rs`, the file is a leaf module. Its parent
    /// module is the directory it lives in. So `self::` is meaningless (leaf
    /// modules have no children), and `super::` resolves to the same directory
    /// (siblings in the parent module).
    fn module_directory(&self, source_file: &Path) -> Result<PathBuf, ExtractionError> {
        source_file
            .parent()
            .map(|p| p.to_path_buf())
            .ok_or_else(|| ExtractionError::ResolutionError {
                module: String::new(),
                source_file: source_file.to_path_buf(),
                reason: "source file has no parent directory".to_string(),
            })
    }

    /// Check if a source file is a module root (`mod.rs` or `lib.rs`).
    ///
    /// Module root files define a module that owns the directory, so `super::`
    /// from these files goes up one directory level.
    fn is_module_root(source_file: &Path) -> bool {
        source_file
            .file_name()
            .map(|f| f == "mod.rs" || f == "lib.rs")
            .unwrap_or(false)
    }

    /// Determine the directory that `super::` resolves to.
    ///
    /// - For `mod.rs`/`lib.rs`: `super::` goes to the parent directory.
    /// - For regular files (e.g., `auth.rs`): `super::` stays in the same
    ///   directory (siblings in the parent module).
    fn super_directory(&self, source_file: &Path) -> Result<PathBuf, ExtractionError> {
        let dir = self.module_directory(source_file)?;
        if Self::is_module_root(source_file) {
            // mod.rs/lib.rs: super is the parent directory
            dir.parent()
                .map(|p| p.to_path_buf())
                .ok_or_else(|| ExtractionError::ResolutionError {
                    module: String::new(),
                    source_file: source_file.to_path_buf(),
                    reason: "no parent directory for super resolution from module root".to_string(),
                })
        } else {
            // Regular file: super is the same directory (parent module)
            Ok(dir)
        }
    }

    // =========================================================================
    // Import extraction (private helpers)
    // =========================================================================

    /// Walk the AST looking for `use_declaration` nodes and extract import info.
    fn walk_use_declarations(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
        imports: &mut Vec<RustImportInfo>,
    ) {
        if node.kind() == "use_declaration" {
            self.extract_use_declaration(node, source, imports);
            return;
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_use_declarations(child, source, imports);
        }
    }

    /// Walk the AST looking for `mod_item` nodes and extract module dependencies.
    ///
    /// Extracts `mod foo;` declarations which create module dependencies.
    /// Note: This extracts declarations like `mod lib;`, not inline modules `mod lib { ... }`.
    fn walk_mod_declarations(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
        imports: &mut Vec<RustImportInfo>,
    ) {
        if node.kind() == "mod_item" {
            self.extract_mod_declaration(node, source, imports);
            return;
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_mod_declarations(child, source, imports);
        }
    }

    /// Extract module dependency from a `mod_item` node.
    ///
    /// Handles: `mod foo;` (external module file)
    /// Skips: `mod foo { ... }` (inline module - no file dependency)
    fn extract_mod_declaration(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
        imports: &mut Vec<RustImportInfo>,
    ) {
        // Check if this is an external module (has semicolon) vs inline (has block)
        let has_block = node
            .children(&mut node.walk())
            .any(|c| c.kind() == "declaration_list");
        if has_block {
            // Inline module - no file dependency
            return;
        }

        // Extract module name
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                if let Ok(name) = child.utf8_text(source) {
                    // Create import info for module dependency
                    imports.push(RustImportInfo {
                        module_path: name.to_string(),
                        symbols: Vec::new(),
                        is_wildcard: false,
                        aliases: Vec::new(),
                    });
                }
                return;
            }
        }
    }

    /// Extract import info from a single `use_declaration` node.
    ///
    /// Tree-sitter Rust grammar for `use_declaration`:
    /// ```text
    /// use_declaration -> visibility_modifier? "use" use_clause ";"
    /// use_clause -> scoped_identifier | identifier | use_as_clause
    ///            | scoped_use_list | use_wildcard | use_list
    /// ```
    fn extract_use_declaration(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
        imports: &mut Vec<RustImportInfo>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "scoped_identifier" | "scoped_use_list" | "use_as_clause" | "use_wildcard"
                | "use_list" | "identifier" => {
                    let mut info = RustImportInfo {
                        module_path: String::new(),
                        symbols: Vec::new(),
                        is_wildcard: false,
                        aliases: Vec::new(),
                    };
                    self.extract_use_clause(child, source, &mut info);
                    imports.push(info);
                }
                _ => {}
            }
        }
    }

    /// Extract use clause details into a [`RustImportInfo`].
    ///
    /// Dispatches based on the node kind to handle all Rust use syntax variants.
    fn extract_use_clause(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
        info: &mut RustImportInfo,
    ) {
        match node.kind() {
            "identifier" => {
                // Bare import: `use serde;`
                info.module_path = self.node_text(node, source);
            }
            "scoped_identifier" => {
                // `use std::collections::HashMap;`
                // Split into path (all but last) and name (last identifier)
                let full_path = self.node_text(node, source);
                if let Some((path, symbol)) = full_path.rsplit_once("::") {
                    info.module_path = path.to_string();
                    info.symbols.push(symbol.to_string());
                } else {
                    info.module_path = full_path;
                }
            }
            "use_as_clause" => {
                self.extract_use_as_clause(node, source, info);
            }
            "scoped_use_list" => {
                self.extract_scoped_use_list(node, source, info);
            }
            "use_wildcard" => {
                self.extract_use_wildcard(node, source, info);
            }
            "use_list" => {
                self.extract_use_list(node, source, info);
            }
            _ => {}
        }
    }

    /// Extract a `use_as_clause` node: `path as alias`.
    fn extract_use_as_clause(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
        info: &mut RustImportInfo,
    ) {
        let mut cursor = node.walk();
        let children: Vec<_> = node
            .children(&mut cursor)
            .filter(|c| c.is_named())
            .collect();

        // Structure: use_as_clause -> path "as" alias
        // Named children: [scoped_identifier|identifier, identifier(alias)]
        if children.len() >= 2 {
            let path_node = children[0];
            let alias_node = children[children.len() - 1];

            let full_path = self.node_text(path_node, source);
            let alias = self.node_text(alias_node, source);

            if let Some((path, symbol)) = full_path.rsplit_once("::") {
                info.module_path = path.to_string();
                info.symbols.push(symbol.to_string());
                info.aliases.push((symbol.to_string(), alias));
            } else {
                // `use serde as s;`
                info.module_path = full_path.clone();
                info.aliases.push((full_path, alias));
            }
        }
    }

    /// Extract a `scoped_use_list` node: `path::{items}`.
    fn extract_scoped_use_list(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
        info: &mut RustImportInfo,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" | "scoped_identifier" | "self" | "crate" | "super" => {
                    info.module_path = self.node_text(child, source);
                }
                "use_list" => {
                    self.extract_use_list(child, source, info);
                }
                _ => {}
            }
        }
    }

    /// Extract items from a `use_list` node: `{Item1, Item2, ...}`.
    fn extract_use_list(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
        info: &mut RustImportInfo,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    info.symbols.push(self.node_text(child, source));
                }
                "use_as_clause" => {
                    // `HashMap as Map` inside a use list
                    let mut inner_cursor = child.walk();
                    let named: Vec<_> = child
                        .children(&mut inner_cursor)
                        .filter(|c| c.is_named())
                        .collect();
                    if named.len() >= 2 {
                        let original = self.node_text(named[0], source);
                        let alias = self.node_text(named[named.len() - 1], source);
                        info.symbols.push(original.clone());
                        info.aliases.push((original, alias));
                    }
                }
                "self" => {
                    info.symbols.push("self".to_string());
                }
                "use_wildcard" => {
                    info.is_wildcard = true;
                }
                _ => {}
            }
        }
    }

    /// Extract a `use_wildcard` node: `path::*`.
    fn extract_use_wildcard(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
        info: &mut RustImportInfo,
    ) {
        info.is_wildcard = true;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" || child.kind() == "scoped_identifier" {
                info.module_path = self.node_text(child, source);
            }
        }
    }

    // =========================================================================
    // Export extraction (private helpers)
    // =========================================================================

    /// Walk the AST looking for `pub use` declarations and extract export info.
    fn walk_export_declarations(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
        exports: &mut Vec<ExportInfo>,
    ) {
        if node.kind() == "use_declaration" {
            if let Some(vis) = self.get_visibility(node, source) {
                self.extract_export_from_use(node, source, vis, exports);
            }
            return;
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_export_declarations(child, source, exports);
        }
    }

    /// Check if a `use_declaration` has a visibility modifier.
    /// Returns `Some(Visibility)` for pub/pub(crate)/pub(super)/pub(in ...).
    fn get_visibility(&self, node: tree_sitter::Node<'_>, source: &[u8]) -> Option<Visibility> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "visibility_modifier" {
                let text = self.node_text(child, source);
                return Some(self.parse_visibility(&text));
            }
        }
        None
    }

    /// Parse a visibility modifier string into a [`Visibility`] enum value.
    fn parse_visibility(&self, text: &str) -> Visibility {
        let trimmed = text.trim();
        if trimmed == "pub" {
            Visibility::Public
        } else if trimmed.starts_with("pub(crate)") {
            Visibility::Crate
        } else if trimmed.starts_with("pub(super)") {
            Visibility::Super
        } else if trimmed.starts_with("pub(in") {
            Visibility::Restricted
        } else {
            Visibility::Public
        }
    }

    /// Extract export info from a `pub use` declaration.
    fn extract_export_from_use(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
        visibility: Visibility,
        exports: &mut Vec<ExportInfo>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "scoped_identifier" => {
                    let full = self.node_text(child, source);
                    if let Some((path, symbol)) = full.rsplit_once("::") {
                        exports.push(ExportInfo {
                            symbol_name: symbol.to_string(),
                            module_path: path.to_string(),
                            visibility,
                        });
                    }
                }
                "scoped_use_list" => {
                    let mut module_path = String::new();
                    let mut symbols = Vec::new();

                    let mut inner_cursor = child.walk();
                    for inner in child.children(&mut inner_cursor) {
                        match inner.kind() {
                            "identifier" | "scoped_identifier" => {
                                module_path = self.node_text(inner, source);
                            }
                            "use_list" => {
                                let mut list_cursor = inner.walk();
                                for item in inner.children(&mut list_cursor) {
                                    if item.kind() == "identifier" {
                                        symbols.push(self.node_text(item, source));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    for sym in symbols {
                        exports.push(ExportInfo {
                            symbol_name: sym,
                            module_path: module_path.clone(),
                            visibility,
                        });
                    }
                }
                "use_wildcard" => {
                    let mut module_path = String::new();
                    let mut wc_cursor = child.walk();
                    for wc_child in child.children(&mut wc_cursor) {
                        if wc_child.kind() == "identifier" || wc_child.kind() == "scoped_identifier"
                        {
                            module_path = self.node_text(wc_child, source);
                        }
                    }
                    exports.push(ExportInfo {
                        symbol_name: "*".to_string(),
                        module_path,
                        visibility,
                    });
                }
                "use_as_clause" => {
                    let mut inner_cursor = child.walk();
                    let named: Vec<_> = child
                        .children(&mut inner_cursor)
                        .filter(|c| c.is_named())
                        .collect();
                    if !named.is_empty() {
                        let full = self.node_text(named[0], source);
                        if let Some((path, symbol)) = full.rsplit_once("::") {
                            exports.push(ExportInfo {
                                symbol_name: symbol.to_string(),
                                module_path: path.to_string(),
                                visibility,
                            });
                        }
                    }
                }
                "identifier" => {
                    let name = self.node_text(child, source);
                    exports.push(ExportInfo {
                        symbol_name: name.clone(),
                        module_path: name,
                        visibility,
                    });
                }
                _ => {}
            }
        }
    }

    // =========================================================================
    // Utility helpers
    // =========================================================================

    /// Get the UTF-8 text of a tree-sitter node.
    fn node_text(&self, node: tree_sitter::Node<'_>, source: &[u8]) -> String {
        node.utf8_text(source).unwrap_or("").to_string()
    }
}

impl Default for RustDependencyExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify AST node kinds for Rust use declarations to validate grammar assumptions.
    #[test]
    fn verify_ast_structure() {
        let source = "use std::collections::HashMap;";
        let tree = RustDependencyExtractor::parse_source(source).unwrap();
        let root = tree.root_node();
        assert_eq!(root.kind(), "source_file");
        let use_decl = root.child(0).unwrap();
        assert_eq!(use_decl.kind(), "use_declaration");
    }
}
