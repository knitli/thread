// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # Conversion Utilities
//!
//! Utilities for converting between service layer types and ast-engine types.
//! These functions bridge the ast-grep functionality with the service layer
//! abstractions while preserving all ast-grep power.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::types::{
    ParsedDocument, CodeMatch, DocumentMetadata, SymbolInfo, ImportInfo, ExportInfo,
    CallInfo, TypeInfo, SymbolKind, Visibility, ImportKind, ExportKind, TypeKind, Range
};
use crate::error::{ServiceResult, AnalysisError};

use thread_ast_engine::{Root, Node, NodeMatch, Position};
use thread_ast_engine::source::Doc;
use thread_language::SupportLang;

#[cfg(feature = "matching")]
use thread_ast_engine::matcher::MatcherExt;

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
    content_hash: u64,
) -> ParsedDocument<D> {
    ParsedDocument::new(ast_root, file_path, language, content_hash)
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
fn extract_functions<D: Doc>(root_node: &Node<D>) -> ServiceResult<HashMap<String, SymbolInfo>> {
    let mut functions = HashMap::new();

    // Try different function patterns based on common languages
    let patterns = [
        "fn $NAME($$$PARAMS) { $$$BODY }",  // Rust
        "function $NAME($$$PARAMS) { $$$BODY }", // JavaScript
        "def $NAME($$$PARAMS): $$$BODY",    // Python
        "func $NAME($$$PARAMS) { $$$BODY }", // Go
    ];

    for pattern in &patterns {
        if let Some(matches) = root_node.find_all(pattern) {
            for node_match in matches {
                if let Some(name_node) = node_match.get_env().get_match("NAME") {
                    let function_name = name_node.text().to_string();
                    let position = Position::new(
                        name_node.start_pos().row,
                        name_node.start_pos().column,
                        name_node.start_byte(),
                    );

                    let symbol_info = SymbolInfo {
                        name: function_name.clone(),
                        kind: SymbolKind::Function,
                        position,
                        scope: "global".to_string(), // Simplified for now
                        visibility: Visibility::Public, // Simplified for now
                    };

                    functions.insert(function_name, symbol_info);
                }
            }
        }
    }

    Ok(functions)
}

/// Extract import statements using language-specific patterns
#[cfg(feature = "matching")]
fn extract_imports<D: Doc>(
    root_node: &Node<D>,
    language: &SupportLang,
) -> ServiceResult<HashMap<String, ImportInfo>> {
    let mut imports = HashMap::new();

    let patterns = match language {
        SupportLang::Rust => vec![
            "use $PATH;",
            "use $PATH::$ITEM;",
            "use $PATH::{$$$ITEMS};",
        ],
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
        if let Some(matches) = root_node.find_all(pattern) {
            for node_match in matches {
                if let (Some(path_node), item_node) = (
                    node_match.get_env().get_match("PATH")
                        .or_else(|| node_match.get_env().get_match("MODULE")),
                    node_match.get_env().get_match("ITEM")
                        .or_else(|| node_match.get_env().get_match("PATH"))
                ) {
                    if let Some(item_node) = item_node {
                        let import_info = ImportInfo {
                            symbol_name: item_node.text().to_string(),
                            source_path: path_node.text().to_string(),
                            import_kind: ImportKind::Named, // Simplified
                            position: Position::new(
                                item_node.start_pos().row,
                                item_node.start_pos().column,
                                item_node.start_byte(),
                            ),
                        };

                        imports.insert(item_node.text().to_string(), import_info);
                    }
                }
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
        "$FUNC($$$ARGS)",  // Most languages
        "$OBJ.$METHOD($$$ARGS)", // Method calls
    ];

    for pattern in &patterns {
        if let Some(matches) = root_node.find_all(pattern) {
            for node_match in matches {
                if let Some(func_node) = node_match.get_env().get_match("FUNC")
                    .or_else(|| node_match.get_env().get_match("METHOD")) {
                    
                    let call_info = CallInfo {
                        function_name: func_node.text().to_string(),
                        position: Position::new(
                            func_node.start_pos().row,
                            func_node.start_pos().column,
                            func_node.start_byte(),
                        ),
                        arguments_count: count_arguments(&node_match),
                        is_resolved: false, // Would need cross-file analysis
                        target_file: None,  // Would need cross-file analysis
                    };

                    calls.push(call_info);
                }
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
        args_node.text().split(',').filter(|s| !s.trim().is_empty()).count()
    } else {
        0
    }
}

/// Convert ast-grep Position to service layer Range
pub fn position_to_range(start: Position, end: Position) -> Range {
    Range::from_ast_positions(start, end)
}

/// Helper for creating SymbolInfo with common defaults
pub fn create_symbol_info(
    name: String,
    kind: SymbolKind,
    position: Position,
) -> SymbolInfo {
    SymbolInfo {
        name,
        kind,
        position,
        scope: "unknown".to_string(),
        visibility: Visibility::Public,
    }
}

/// Extract content hash for deduplication
pub fn compute_content_hash(content: &str) -> u64 {
    use thread_utils::hash_help::rapid_hash;
    rapid_hash(content.as_bytes())
}

// Conversion functions for common patterns

/// Convert common node kinds to SymbolKind
pub fn node_kind_to_symbol_kind(node_kind: &str) -> SymbolKind {
    match node_kind {
        "function_declaration" | "function_definition" => SymbolKind::Function,
        "class_declaration" | "class_definition" => SymbolKind::Class,
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
    use std::path::PathBuf;

    #[test]
    fn test_compute_content_hash() {
        let content = "fn main() {}";
        let hash1 = compute_content_hash(content);
        let hash2 = compute_content_hash(content);
        assert_eq!(hash1, hash2);

        let different_content = "fn test() {}";
        let hash3 = compute_content_hash(different_content);
        assert_ne!(hash1, hash3);
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
        let info = create_symbol_info(
            "test_function".to_string(),
            SymbolKind::Function,
            pos
        );
        
        assert_eq!(info.name, "test_function");
        assert_eq!(info.kind, SymbolKind::Function);
        assert_eq!(info.position, pos);
    }
}