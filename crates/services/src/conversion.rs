// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # Conversion Utilities
//!
//! Utilities for converting between service layer types and ast-engine types.
//! These functions bridge the ast-grep functionality with the service layer
//! abstractions while preserving all ast-grep power.

use crate::types::{CodeMatch, ParsedDocument, Range, SymbolInfo, SymbolKind, Visibility};
use std::path::PathBuf;

#[cfg(feature = "matching")]
use crate::error::ServiceResult;
#[cfg(feature = "matching")]
use crate::types::{CallInfo, DocumentMetadata, ImportInfo, ImportKind};
#[cfg(feature = "matching")]
use thread_utilities::RapidMap;

cfg_if::cfg_if!(
    if #[cfg(feature = "ast-grep-backend")] {
        use thread_ast_engine::{Doc, Root, Node, NodeMatch, Position};
        use thread_language::SupportLang;
    } else  {
        use crate::types::{Doc, Root, NodeMatch, Position, SupportLang};
    }
);

/// Convert ast-grep NodeMatch to service layer CodeMatch
///
/// This preserves all ast-grep functionality while adding service layer context.
pub fn node_match_to_code_match<'tree, D: Doc>(
    node_match: NodeMatch<'tree, D>,
) -> CodeMatch<'tree, D> {
    CodeMatch::new(node_match)
}

/// Create ParsedDocument from ast-grep Root
///
/// This is the core conversion that bridges file-level ast-grep to codebase-level intelligence.
pub fn root_to_parsed_document<D: Doc>(
    ast_root: Root<D>,
    file_path: PathBuf,
    language: SupportLang,
    content_fingerprint: recoco_utils::fingerprint::Fingerprint,
) -> ParsedDocument<D> {
    ParsedDocument::new(ast_root, file_path, language, content_fingerprint)
}

/// Extract basic metadata from a parsed document
///
/// This function demonstrates how to build codebase-level intelligence
/// on top of ast-grep's file-level analysis capabilities.
#[cfg(feature = "matching")]
pub fn extract_basic_metadata<D: Doc>(
    document: &ParsedDocument<D>,
) -> ServiceResult<DocumentMetadata> {
    let mut metadata = DocumentMetadata::default();
    let root = document.ast_grep_root();
    let root_node = root.root();

    // Extract function definitions
    if let Ok(function_matches) = extract_functions(&root_node) {
        for (name, info) in function_matches {
            metadata.defined_symbols.insert(name, info);
        }
    }

    // Extract class and struct definitions
    if let Ok(class_matches) = extract_classes(&root_node, &document.language) {
        for (name, info) in class_matches {
            metadata.defined_symbols.insert(name, info);
        }
    }

    // Extract import statements
    if let Ok(imports) = extract_imports(&root_node, &document.language) {
        for (name, info) in imports {
            metadata.imported_symbols.insert(name, info);
        }
    }

    // Extract function calls
    if let Ok(calls) = extract_function_calls(&root_node) {
        metadata.function_calls = calls;
    }

    Ok(metadata)
}

/// Extract function definitions using ast-grep patterns
#[cfg(feature = "matching")]
fn extract_functions<D: Doc>(root_node: &Node<D>) -> ServiceResult<RapidMap<String, SymbolInfo>> {
    let mut functions = thread_utilities::get_map();

    // Try different function patterns based on common languages
    let patterns = [
        "fn $NAME($$$PARAMS) { $$$BODY }",       // Rust
        "function $NAME($$$PARAMS) { $$$BODY }", // JavaScript
        "def $NAME($$$PARAMS): $$$BODY",         // Python
        "func $NAME($$$PARAMS) { $$$BODY }",     // Go
    ];

    for pattern in &patterns {
        for node_match in root_node.find_all(pattern) {
            if let Some(name_node) = node_match.get_env().get_match("NAME") {
                let function_name = name_node.text().to_string();
                let position = name_node.start_pos();

                let symbol_info = SymbolInfo {
                    name: function_name.clone(),
                    kind: SymbolKind::Function,
                    position,
                    scope: "global".to_string(),    // Simplified for now
                    visibility: Visibility::Public, // Simplified for now
                };

                functions.insert(function_name, symbol_info);
            }
        }
    }

    Ok(functions)
}

/// Extract class and struct definitions using language-specific ast-grep patterns
#[cfg(feature = "matching")]
fn extract_classes<D: Doc>(
    root_node: &Node<D>,
    language: &SupportLang,
) -> ServiceResult<RapidMap<String, SymbolInfo>> {
    let mut classes = thread_utilities::get_map();

    // Select language-specific patterns paired with their SymbolKind
    let patterns: Vec<(&str, SymbolKind)> = match language {
        SupportLang::Rust => vec![
            ("struct $NAME { $$$BODY }", SymbolKind::Struct),  // braced struct
            ("struct $NAME($$$FIELDS)", SymbolKind::Struct),   // tuple struct
            ("struct $NAME", SymbolKind::Struct),              // unit struct
        ],
        SupportLang::Go => vec![
            ("type $NAME struct { $$$BODY }", SymbolKind::Struct), // Go struct
        ],
        SupportLang::JavaScript | SupportLang::TypeScript => vec![
            ("class $NAME { $$$BODY }", SymbolKind::Class),         // JS/TS class
            ("interface $NAME { $$$BODY }", SymbolKind::Interface),  // TS interface
        ],
        SupportLang::Python => vec![
            ("class $NAME: $$$BODY", SymbolKind::Class),            // simple Python class
            ("class $NAME($$$PARAMS): $$$BODY", SymbolKind::Class), // Python class with bases
        ],
        _ => vec![
            // Generic fallbacks for C++, Java, C#, etc.
            ("struct $NAME { $$$BODY }", SymbolKind::Struct),
            ("class $NAME { $$$BODY }", SymbolKind::Class),
            ("interface $NAME { $$$BODY }", SymbolKind::Interface),
        ],
    };

    for (pattern, kind) in &patterns {
        for node_match in root_node.find_all(*pattern) {
            if let Some(name_node) = node_match.get_env().get_match("NAME") {
                let symbol_name = name_node.text().to_string();
                let position = name_node.start_pos();

                let symbol_info = SymbolInfo {
                    name: symbol_name.clone(),
                    kind: kind.clone(),
                    position,
                    scope: "global".to_string(),    // Simplified for now
                    visibility: Visibility::Public, // Simplified for now
                };

                classes.insert(symbol_name, symbol_info);
            }
        }
    }

    Ok(classes)
}

/// Extract import statements using language-specific patterns
#[cfg(feature = "matching")]
fn extract_imports<D: Doc>(
    root_node: &Node<D>,
    language: &SupportLang,
) -> ServiceResult<RapidMap<String, ImportInfo>> {
    let mut imports = thread_utilities::get_map();

    let patterns = match language {
        SupportLang::Rust => vec!["use $PATH;", "use $PATH::$ITEM;", "use $PATH::{$$$ITEMS};"],
        SupportLang::JavaScript | SupportLang::TypeScript => vec![
            "import $ITEM from '$PATH';",
            "import { $$$ITEMS } from '$PATH';",
            "import * as $ALIAS from '$PATH';",
        ],
        SupportLang::Python => vec![
            "import $MODULE",
            "from $MODULE import $ITEM",
            "from $MODULE import $$$ITEMS",
        ],
        _ => vec![], // Add more languages as needed
    };

    for pattern in patterns {
        for node_match in root_node.find_all(pattern) {
            if let (Some(path_node), Some(item_node)) = (
                node_match
                    .get_env()
                    .get_match("PATH")
                    .or_else(|| node_match.get_env().get_match("MODULE")),
                node_match
                    .get_env()
                    .get_match("ITEM")
                    .or_else(|| node_match.get_env().get_match("PATH")),
            ) {
                let import_info = ImportInfo {
                    symbol_name: item_node.text().to_string(),
                    source_path: path_node.text().to_string(),
                    import_kind: ImportKind::Named, // Simplified
                    position: item_node.start_pos(),
                };

                imports.insert(item_node.text().to_string(), import_info);
            }
        }
    }

    Ok(imports)
}

/// Extract function calls using ast-grep patterns
#[cfg(feature = "matching")]
fn extract_function_calls<D: Doc>(root_node: &Node<D>) -> ServiceResult<Vec<CallInfo>> {
    let mut calls = Vec::new();

    // Common function call patterns
    let patterns = [
        "$FUNC($$$ARGS)",        // Most languages
        "$OBJ.$METHOD($$$ARGS)", // Method calls
    ];

    for pattern in &patterns {
        for node_match in root_node.find_all(pattern) {
            if let Some(func_node) = node_match
                .get_env()
                .get_match("FUNC")
                .or_else(|| node_match.get_env().get_match("METHOD"))
            {
                let call_info = CallInfo {
                    function_name: func_node.text().to_string(),
                    position: func_node.start_pos(),
                    arguments_count: count_arguments(&node_match),
                    is_resolved: false, // Would need cross-file analysis
                    target_file: None,  // Would need cross-file analysis
                };

                calls.push(call_info);
            }
        }
    }

    Ok(calls)
}

/// Count arguments in a function call
#[cfg(feature = "matching")]
fn count_arguments<D: Doc>(node_match: &NodeMatch<D>) -> usize {
    if let Some(args_node) = node_match.get_env().get_match("ARGS") {
        // This is a simplified count - would need language-specific parsing
        args_node
            .text()
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .count()
    } else {
        0
    }
}

/// Convert ast-grep Position to service layer Range
pub fn position_to_range(start: Position, end: Position) -> Range {
    Range::from_ast_positions(start, end)
}

/// Helper for creating SymbolInfo with common defaults
pub fn create_symbol_info(name: String, kind: SymbolKind, position: Position) -> SymbolInfo {
    SymbolInfo {
        name,
        kind,
        position,
        scope: "unknown".to_string(),
        visibility: Visibility::Public,
    }
}

/// Compute content fingerprint for deduplication using blake3
///
/// This uses ReCoco's Fingerprinter which provides:
/// - 10-100x faster hashing than SHA256 via blake3
/// - 16-byte compact fingerprint (vs 32-byte SHA256)
/// - Automatic integration with ReCoco's memoization system
/// - Type-safe content-addressed caching
pub fn compute_content_fingerprint(content: &str) -> recoco_utils::fingerprint::Fingerprint {
    let mut fp = recoco_utils::fingerprint::Fingerprinter::default();
    // Note: write() can fail for serialization, but with &str it won't fail
    fp.write(content)
        .expect("fingerprinting string should not fail");
    fp.into_fingerprint()
}

// Conversion functions for common patterns

/// Convert common node kinds to SymbolKind
pub fn node_kind_to_symbol_kind(node_kind: &str) -> SymbolKind {
    match node_kind {
        "function_declaration" | "function_definition" => SymbolKind::Function,
        "class_declaration" | "class_definition" => SymbolKind::Class,
        "struct_declaration" | "struct_definition" => SymbolKind::Struct,
        "interface_declaration" => SymbolKind::Interface,
        "variable_declaration" | "let_declaration" => SymbolKind::Variable,
        "const_declaration" | "constant" => SymbolKind::Constant,
        "type_declaration" | "type_definition" => SymbolKind::Type,
        "module_declaration" => SymbolKind::Module,
        "namespace_declaration" => SymbolKind::Namespace,
        "enum_declaration" => SymbolKind::Enum,
        "field_declaration" => SymbolKind::Field,
        "property_declaration" => SymbolKind::Property,
        "method_declaration" | "method_definition" => SymbolKind::Method,
        "constructor_declaration" => SymbolKind::Constructor,
        _ => SymbolKind::Other(node_kind.to_string()),
    }
}

/// Convert visibility modifiers to Visibility enum
pub fn modifier_to_visibility(modifier: &str) -> Visibility {
    match modifier {
        "pub" | "public" => Visibility::Public,
        "priv" | "private" => Visibility::Private,
        "protected" => Visibility::Protected,
        "internal" => Visibility::Internal,
        "package" => Visibility::Package,
        _ => Visibility::Other(modifier.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_content_fingerprint() {
        let content = "fn main() {}";
        let fp1 = compute_content_fingerprint(content);
        let fp2 = compute_content_fingerprint(content);
        assert_eq!(fp1, fp2, "Same content should produce same fingerprint");

        let different_content = "fn test() {}";
        let fp3 = compute_content_fingerprint(different_content);
        assert_ne!(
            fp1, fp3,
            "Different content should produce different fingerprint"
        );
    }

    #[test]
    fn test_node_kind_to_symbol_kind() {
        assert_eq!(
            node_kind_to_symbol_kind("function_declaration"),
            SymbolKind::Function
        );
        assert_eq!(
            node_kind_to_symbol_kind("class_declaration"),
            SymbolKind::Class
        );
        assert_eq!(
            node_kind_to_symbol_kind("unknown"),
            SymbolKind::Other("unknown".to_string())
        );
    }

    #[test]
    fn test_modifier_to_visibility() {
        assert_eq!(modifier_to_visibility("pub"), Visibility::Public);
        assert_eq!(modifier_to_visibility("private"), Visibility::Private);
        assert_eq!(modifier_to_visibility("protected"), Visibility::Protected);
    }

    #[test]
    fn test_create_symbol_info() {
        let pos = Position::new(1, 0, 10);
        let info = create_symbol_info("test_function".to_string(), SymbolKind::Function, pos);

        assert_eq!(info.name, "test_function");
        assert_eq!(info.kind, SymbolKind::Function);
        assert_eq!(info.position, pos);
    }
}
