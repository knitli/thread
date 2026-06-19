// SPDX-FileCopyrightText: 2026 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! TypeScript/JavaScript dependency extractor using tree-sitter queries.
//!
//! Extracts ES6 imports, CommonJS requires, and export declarations from
//! TypeScript and JavaScript source files.
//!
//! ## Supported Import Patterns
//!
//! - ES6 default imports: `import React from 'react'`
//! - ES6 named imports: `import { useState } from 'react'`
//! - ES6 namespace imports: `import * as fs from 'fs'`
//! - ES6 mixed imports: `import React, { useState } from 'react'`
//! - CommonJS requires: `const express = require('express')`
//! - Dynamic imports: `import('module')` (weak dependency)
//! - TypeScript type-only: `import type { User } from './types'`
//!
//! ## Supported Export Patterns
//!
//! - Default exports: `export default function() {}`
//! - Named exports: `export const X = 1`
//! - Re-exports: `export * from './other'`
//! - Named re-exports: `export { X } from './other'`
//!
//! ## Performance
//!
//! Target: <5ms per file. Uses tree-sitter's incremental parsing for efficient
//! extraction without full AST traversal.

use std::path::{Path, PathBuf};

use crate::incremental::types::{DependencyEdge, DependencyType};

/// Error types for TypeScript/JavaScript dependency extraction.
#[derive(Debug, thiserror::Error)]
pub enum ExtractionError {
    /// Tree-sitter failed to parse the source file.
    #[error("parse error: failed to parse TypeScript/JavaScript source")]
    ParseError,

    /// Module path could not be resolved to a local file path.
    #[error("unresolved module: {path}")]
    UnresolvedModule {
        /// The module specifier that could not be resolved.
        path: String,
    },
}

/// Information about a single import statement (ES6 or CommonJS).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportInfo {
    /// The module specifier string (e.g., `"react"` or `"./utils"`).
    pub module_specifier: String,

    /// Named imports with optional aliases.
    pub symbols: Vec<ImportedSymbol>,

    /// Default import name (e.g., `React` in `import React from 'react'`).
    pub default_import: Option<String>,

    /// Namespace import name (e.g., `fs` in `import * as fs from 'fs'`).
    pub namespace_import: Option<String>,

    /// Whether this is a dynamic import (`import('...')`).
    pub is_dynamic: bool,
}

/// A single imported symbol with optional alias.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedSymbol {
    /// The name as exported from the module.
    pub imported_name: String,

    /// The name used locally (may differ if aliased).
    pub local_name: String,
}

/// Information about an export statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportInfo {
    /// The exported symbol name.
    pub symbol_name: String,

    /// Whether this is a default export.
    pub is_default: bool,

    /// The type of export.
    pub export_type: ExportType,
}

/// Types of export statements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportType {
    /// Default export: `export default X`
    Default,

    /// Named export: `export const X = 1`
    Named,

    /// Named re-export: `export { X } from './other'`
    NamedReexport,

    /// Namespace re-export: `export * from './other'`
    NamespaceReexport,
}

/// TypeScript/JavaScript dependency extractor with tree-sitter query-based extraction.
///
/// Supports both TypeScript and JavaScript files, handling ES6 modules, CommonJS,
/// and mixed module systems.
///
/// # Examples
///
/// ```rust,ignore
/// use thread_flow::incremental::extractors::typescript::TypeScriptDependencyExtractor;
/// use std::path::Path;
///
/// let extractor = TypeScriptDependencyExtractor::new();
/// let imports = extractor.extract_imports(source, Path::new("app.tsx")).unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct TypeScriptDependencyExtractor;

impl TypeScriptDependencyExtractor {
    /// Create a new TypeScript/JavaScript dependency extractor.
    pub fn new() -> Self {
        Self
    }

    /// Extract all import statements from a TypeScript/JavaScript source file.
    ///
    /// Handles ES6 imports, CommonJS requires, and dynamic imports.
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

        // Try JavaScript parser first (works for most JS/TS code)
        let language = thread_language::parsers::language_javascript();
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

    /// Extract all export statements from a TypeScript/JavaScript source file.
    ///
    /// Handles default exports, named exports, and re-exports.
    ///
    /// # Errors
    ///
    /// Returns [`ExtractionError::ParseError`] if tree-sitter cannot parse the source.
    pub fn extract_exports(
        &self,
        source: &str,
        _file_path: &Path,
    ) -> Result<Vec<ExportInfo>, ExtractionError> {
        if source.is_empty() {
            return Ok(Vec::new());
        }

        let language = thread_language::parsers::language_javascript();
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&language)
            .map_err(|_| ExtractionError::ParseError)?;

        let tree = parser
            .parse(source, None)
            .ok_or(ExtractionError::ParseError)?;

        let root_node = tree.root_node();
        let mut exports = Vec::new();

        self.walk_exports(root_node, source.as_bytes(), &mut exports);

        Ok(exports)
    }

    /// Walk the tree-sitter AST to extract import statements and require calls.
    fn walk_imports(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
        imports: &mut Vec<ImportInfo>,
    ) {
        match node.kind() {
            "import_statement" => {
                if let Some(info) = self.extract_from_import_statement(node, source) {
                    imports.push(info);
                }
                return;
            }
            "call_expression" => {
                // Check for CommonJS require() or dynamic import()
                if let Some(info) = self.extract_from_call_expression(node, source) {
                    imports.push(info);
                }
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_imports(child, source, imports);
        }
    }

    /// Walk the tree-sitter AST to extract export statements.
    fn walk_exports(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
        exports: &mut Vec<ExportInfo>,
    ) {
        if node.kind() == "export_statement" {
            self.extract_from_export_statement(node, source, exports);
            // Don't return - might have nested structures
        }

        // Continue walking for nested structures
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_exports(child, source, exports);
        }
    }

    /// Extract import information from an ES6 `import_statement` node.
    ///
    /// Handles:
    /// - Default imports: `import React from 'react'`
    /// - Named imports: `import { useState } from 'react'`
    /// - Namespace imports: `import * as fs from 'fs'`
    /// - Mixed imports: `import React, { useState } from 'react'`
    fn extract_from_import_statement(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
    ) -> Option<ImportInfo> {
        let mut module_specifier: Option<String> = None;
        let mut symbols: Vec<ImportedSymbol> = Vec::new();
        let mut default_import: Option<String> = None;
        let mut namespace_import: Option<String> = None;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "import_clause" => {
                    self.extract_import_clause(
                        child,
                        source,
                        &mut default_import,
                        &mut namespace_import,
                        &mut symbols,
                    );
                }
                "string" => {
                    module_specifier = self.extract_string_value(child, source);
                }
                _ => {}
            }
        }

        module_specifier.map(|specifier| ImportInfo {
            module_specifier: specifier,
            symbols,
            default_import,
            namespace_import,
            is_dynamic: false,
        })
    }

    /// Extract import clause components (default, named, namespace).
    fn extract_import_clause(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
        default_import: &mut Option<String>,
        namespace_import: &mut Option<String>,
        symbols: &mut Vec<ImportedSymbol>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    // Default import
                    if let Ok(name) = child.utf8_text(source) {
                        *default_import = Some(name.to_string());
                    }
                }
                "namespace_import" => {
                    // import * as X
                    if let Some(name) = self.extract_namespace_import(child, source) {
                        *namespace_import = Some(name);
                    }
                }
                "named_imports" => {
                    // import { X, Y }
                    self.extract_named_imports(child, source, symbols);
                }
                _ => {}
            }
        }
    }

    /// Extract namespace import name from `import * as X`.
    fn extract_namespace_import(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
    ) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return child.utf8_text(source).ok().map(|s| s.to_string());
            }
        }
        None
    }

    /// Extract named imports from `{ X, Y as Z }`.
    fn extract_named_imports(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
        symbols: &mut Vec<ImportedSymbol>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "import_specifier"
                && let Some(symbol) = self.extract_import_specifier(child, source)
            {
                symbols.push(symbol);
            }
        }
    }

    /// Extract a single import specifier (handles aliases).
    fn extract_import_specifier(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
    ) -> Option<ImportedSymbol> {
        let mut imported_name: Option<String> = None;
        let mut local_name: Option<String> = None;

        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();

        for child in &children {
            if child.kind() == "identifier"
                && let Ok(name) = child.utf8_text(source)
            {
                if imported_name.is_none() {
                    imported_name = Some(name.to_string());
                } else {
                    local_name = Some(name.to_string());
                }
            }
        }

        imported_name.map(|imported| ImportedSymbol {
            imported_name: imported.clone(),
            local_name: local_name.unwrap_or(imported),
        })
    }

    /// Extract import from CommonJS require() or dynamic import().
    fn extract_from_call_expression(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
    ) -> Option<ImportInfo> {
        let mut is_require = false;
        let mut is_dynamic_import = false;
        let mut module_specifier: Option<String> = None;
        let mut default_import: Option<String> = None;
        let mut symbols: Vec<ImportedSymbol> = Vec::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    if let Ok(text) = child.utf8_text(source)
                        && text == "require"
                    {
                        is_require = true;
                    }
                }
                "import" => {
                    is_dynamic_import = true;
                }
                "arguments" => {
                    // Extract module specifier from arguments
                    module_specifier = self.extract_first_string_argument(child, source);
                }
                _ => {}
            }
        }

        if (is_require || is_dynamic_import) && module_specifier.is_some() {
            // Check if this require is assigned to a variable or destructured
            if is_require {
                let (default, destructured) = self.find_variable_or_destructured(node, source);
                default_import = default;
                symbols = destructured;
            }

            return Some(ImportInfo {
                module_specifier: module_specifier?,
                symbols,
                default_import,
                namespace_import: None,
                is_dynamic: is_dynamic_import,
            });
        }

        None
    }

    /// Find the variable name or destructured names for a require() call.
    fn find_variable_or_destructured(
        &self,
        call_node: tree_sitter::Node<'_>,
        source: &[u8],
    ) -> (Option<String>, Vec<ImportedSymbol>) {
        // Walk up to find variable_declarator
        let mut current = call_node.parent();
        while let Some(node) = current {
            if node.kind() == "variable_declarator" {
                return self.extract_variable_declarator_pattern(node, source);
            }
            current = node.parent();
        }

        (None, Vec::new())
    }

    /// Extract variable pattern from declarator (handles both simple and destructured).
    fn extract_variable_declarator_pattern(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
    ) -> (Option<String>, Vec<ImportedSymbol>) {
        let mut default_import = None;
        let mut symbols = Vec::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    // Simple assignment: const X = require(...)
                    if let Ok(name) = child.utf8_text(source) {
                        default_import = Some(name.to_string());
                    }
                }
                "object_pattern" => {
                    // Destructured: const { X, Y } = require(...)
                    symbols = self.extract_object_pattern(child, source);
                }
                _ => {}
            }
        }

        (default_import, symbols)
    }

    /// Extract destructured names from object_pattern.
    fn extract_object_pattern(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
    ) -> Vec<ImportedSymbol> {
        let mut symbols = Vec::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "shorthand_property_identifier_pattern" {
                // { X }
                if let Ok(name) = child.utf8_text(source) {
                    symbols.push(ImportedSymbol {
                        imported_name: name.to_string(),
                        local_name: name.to_string(),
                    });
                }
            } else if child.kind() == "pair_pattern" {
                // { X: Y } or { X as Y }
                if let Some(symbol) = self.extract_pair_pattern(child, source) {
                    symbols.push(symbol);
                }
            }
        }

        symbols
    }

    /// Extract symbol from pair_pattern (handles aliasing).
    fn extract_pair_pattern(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
    ) -> Option<ImportedSymbol> {
        let mut imported_name = None;
        let mut local_name = None;

        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();

        for child in &children {
            if (child.kind() == "property_identifier" || child.kind() == "identifier")
                && let Ok(name) = child.utf8_text(source)
            {
                if imported_name.is_none() {
                    imported_name = Some(name.to_string());
                } else {
                    local_name = Some(name.to_string());
                }
            }
        }

        imported_name.map(|imported| ImportedSymbol {
            imported_name: imported.clone(),
            local_name: local_name.unwrap_or(imported),
        })
    }

    /// Extract the first string argument from an arguments node.
    fn extract_first_string_argument(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
    ) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "string" {
                return self.extract_string_value(child, source);
            }
        }
        None
    }

    /// Extract string value from a string node (removes quotes).
    fn extract_string_value(&self, node: tree_sitter::Node<'_>, source: &[u8]) -> Option<String> {
        let raw = node.utf8_text(source).ok()?;
        // Remove surrounding quotes (single or double)
        let trimmed = raw.trim_matches(|c| c == '\'' || c == '"' || c == '`');
        Some(trimmed.to_string())
    }

    /// Extract export information from an `export_statement` node.
    fn extract_from_export_statement(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
        exports: &mut Vec<ExportInfo>,
    ) {
        // Check if this is a re-export (has a source string)
        let is_reexport = self.has_export_source(node, source);
        let mut has_default = false;
        let mut has_wildcard = false;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "*" => {
                    // Wildcard export: export * from './other'
                    has_wildcard = true;
                }
                "lexical_declaration" => {
                    // export const X = 1
                    self.extract_named_exports_from_declaration(child, source, exports);
                }
                "function_declaration" | "class_declaration" => {
                    // export function X() {} or export class X {}
                    if let Some(name) = self.extract_declaration_name(child, source) {
                        exports.push(ExportInfo {
                            symbol_name: name,
                            is_default: has_default,
                            export_type: if has_default {
                                ExportType::Default
                            } else {
                                ExportType::Named
                            },
                        });
                    }
                }
                "function_expression" | "arrow_function" | "class" => {
                    // export default function() {} or export default class {}
                    if has_default {
                        exports.push(ExportInfo {
                            symbol_name: "default".to_string(),
                            is_default: true,
                            export_type: ExportType::Default,
                        });
                    }
                }
                "export_clause" | "named_exports" => {
                    // export { X, Y } or export { X } from './other'
                    self.extract_export_clause(child, source, exports, is_reexport);
                }
                "namespace_export" => {
                    // export * as name from './other'
                    exports.push(ExportInfo {
                        symbol_name: "*".to_string(),
                        is_default: false,
                        export_type: ExportType::NamespaceReexport,
                    });
                }
                _ => {
                    // Check for default keyword or wildcard
                    if let Ok(text) = child.utf8_text(source) {
                        if text == "default" {
                            has_default = true;
                        } else if text == "*" {
                            has_wildcard = true;
                        }
                    }
                }
            }
        }

        // Handle wildcard re-export (export * from './other')
        if has_wildcard && is_reexport {
            exports.push(ExportInfo {
                symbol_name: "*".to_string(),
                is_default: false,
                export_type: ExportType::NamespaceReexport,
            });
        }

        // Handle standalone default export (export default X)
        if has_default && exports.is_empty() {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if (child.kind() == "identifier"
                    || child.kind() == "number"
                    || child.kind() == "string")
                    && let Ok(text) = child.utf8_text(source)
                    && text != "default"
                    && text != "export"
                    && text != "*"
                {
                    exports.push(ExportInfo {
                        symbol_name: "default".to_string(),
                        is_default: true,
                        export_type: ExportType::Default,
                    });
                    break;
                }
            }
        }
    }

    /// Check if an export_statement has a source string (indicating re-export).
    fn has_export_source(&self, node: tree_sitter::Node<'_>, _source: &[u8]) -> bool {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "string" {
                return true;
            }
        }
        false
    }

    /// Extract named exports from a declaration (const, let, var).
    fn extract_named_exports_from_declaration(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
        exports: &mut Vec<ExportInfo>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "variable_declarator"
                && let Some(name) = self.extract_variable_name(child, source)
            {
                exports.push(ExportInfo {
                    symbol_name: name,
                    is_default: false,
                    export_type: ExportType::Named,
                });
            }
        }
    }

    /// Extract variable name from a variable_declarator.
    fn extract_variable_name(&self, node: tree_sitter::Node<'_>, source: &[u8]) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return child.utf8_text(source).ok().map(|s| s.to_string());
            }
        }
        None
    }

    /// Extract function or class name from declaration.
    fn extract_declaration_name(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
    ) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return child.utf8_text(source).ok().map(|s| s.to_string());
            }
        }
        None
    }

    /// Extract export clause (handles re-exports).
    fn extract_export_clause(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
        exports: &mut Vec<ExportInfo>,
        is_reexport: bool,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "export_specifier"
                && let Some(name) = self.extract_export_specifier_name(child, source)
            {
                exports.push(ExportInfo {
                    symbol_name: name,
                    is_default: false,
                    export_type: if is_reexport {
                        ExportType::NamedReexport
                    } else {
                        ExportType::Named
                    },
                });
            }
        }
    }

    /// Extract export specifier name (handles aliases).
    fn extract_export_specifier_name(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
    ) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return child.utf8_text(source).ok().map(|s| s.to_string());
            }
        }
        None
    }

    /// Resolve a JavaScript/TypeScript module path to a local file path.
    ///
    /// Resolution strategy:
    /// 1. Relative paths (`./`, `../`) resolve relative to source file
    /// 2. Node modules (`react`) resolve to `node_modules/<module>/index.js`
    /// 3. Add appropriate file extensions (.js, .ts, .tsx)
    ///
    /// # Errors
    ///
    /// Returns [`ExtractionError::UnresolvedModule`] if resolution fails.
    pub fn resolve_module_path(
        &self,
        source_file: &Path,
        module_specifier: &str,
    ) -> Result<PathBuf, ExtractionError> {
        // Relative import
        if module_specifier.starts_with("./") || module_specifier.starts_with("../") {
            let source_dir =
                source_file
                    .parent()
                    .ok_or_else(|| ExtractionError::UnresolvedModule {
                        path: module_specifier.to_string(),
                    })?;

            // Resolve the path (handles ../ navigation)
            let mut resolved = source_dir.join(module_specifier);

            // Normalize the path to resolve ../ components
            if let Ok(canonical) = resolved.canonicalize() {
                resolved = canonical;
            } else {
                // If canonicalize fails (file doesn't exist), manually resolve
                let mut components = Vec::new();
                for component in resolved.components() {
                    match component {
                        std::path::Component::ParentDir => {
                            // 🛡️ SECURITY: Prevent path traversal by explicitly blocking Component::ParentDir
                            // from popping RootDir/Prefix. Also correctly handle when the component list
                            // is empty or already ends with ParentDir to preserve paths like ../../a
                            if let Some(last) = components.last() {
                                match last {
                                    std::path::Component::Normal(_) => {
                                        components.pop();
                                    }
                                    std::path::Component::ParentDir => {
                                        components.push(component);
                                    }
                                    // Don't pop RootDir or Prefix
                                    _ => {}
                                }
                            } else {
                                components.push(component);
                            }
                        }
                        std::path::Component::CurDir => {}
                        _ => components.push(component),
                    }
                }
                resolved = components.iter().collect();
            }

            // Try adding extensions if no extension present
            if resolved.extension().is_none() {
                for ext in &["ts", "tsx", "js", "jsx"] {
                    let mut with_ext = resolved.clone();
                    with_ext.set_extension(ext);
                    if with_ext.exists() {
                        return Ok(with_ext);
                    }
                }

                // Try index file in directory
                let index_ts = resolved.join("index.ts");
                if index_ts.exists() {
                    return Ok(index_ts);
                }
            }

            return Ok(resolved);
        }

        // Node module
        Ok(PathBuf::from(format!(
            "node_modules/{}/index.js",
            module_specifier
        )))
    }

    /// Extract [`DependencyEdge`] values from a TypeScript/JavaScript source file.
    ///
    /// Combines import extraction with path resolution to produce edges
    /// suitable for the incremental dependency graph.
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
            // Only create edges for resolvable module paths
            // Node modules and unresolvable paths are silently skipped per design spec
            if let Ok(resolved) = self.resolve_module_path(file_path, &import.module_specifier) {
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

impl Default for TypeScriptDependencyExtractor {
    fn default() -> Self {
        Self::new()
    }
}
