// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later


//! Rust-specific parser implementation using ast-grep
//!
//! This module provides functions to extract Rust code elements
//! using ast-grep's pattern matching capabilities.

use crate::LanguageProvider;
use ast_grep_core::{AstGrep, Pattern, NodeMatch};
use thread_core::{
    CodeElement, ElementId, ElementKind, ElementMetadata, FileParseResult,
    Import, Export, Parameter, SourceLocation, Visibility, LanguageParser, Result, ThreadError
};
use std::marker::PhantomData;

/// Rust parser that uses ast-grep for parsing
pub struct RustParser<P: LanguageProvider> {
    _provider: PhantomData<P>,
}

impl<P: LanguageProvider> RustParser<P> {
    pub fn new() -> Self {
        Self {
            _provider: PhantomData,
        }
    }

    /// Extract functions from Rust source code using ast-grep patterns
    pub fn extract_functions(&self, source: &str, file_path: &str) -> Result<Vec<CodeElement>> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let lang = P::get_language();
            let ast = lang.ast_grep(source);
            let root = ast.root();

            let mut functions = Vec::new();

            // Pattern for functions with return type
            if let Ok(pattern) = Pattern::new("fn $NAME($PARAMS) -> $RETURN { $$$BODY }", lang.clone()) {
                for m in pattern.find_all(&root) {
                    if let Ok(func) = self.extract_function_from_match(&m, file_path, true) {
                        functions.push(func);
                    }
                }
            }

            // Pattern for functions without return type
            if let Ok(pattern) = Pattern::new("fn $NAME($PARAMS) { $$$BODY }", lang.clone()) {
                for m in pattern.find_all(&root) {
                    if let Ok(func) = self.extract_function_from_match(&m, file_path, false) {
                        functions.push(func);
                    }
                }
            }

            Ok(functions)
        }

        #[cfg(target_arch = "wasm32")]
        {
            // For WASM, return empty for now - would need custom implementation
            Ok(vec![])
        }
    }

    /// Extract imports from Rust source code
    pub fn extract_imports(&self, source: &str, file_path: &str) -> Result<Vec<Import>> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let lang = P::get_language();
            let ast = lang.ast_grep(source);
            let root = ast.root();

            let mut imports = Vec::new();

            // Pattern for simple use statements: use path;
            if let Ok(pattern) = Pattern::new("use $PATH;", lang.clone()) {
                for m in pattern.find_all(&root) {
                    if let Ok(import) = self.extract_simple_import(&m, file_path) {
                        imports.push(import);
                    }
                }
            }

            // Pattern for use statements with items: use path::{items};
            if let Ok(pattern) = Pattern::new("use $PATH::{$$$ITEMS};", lang.clone()) {
                for m in pattern.find_all(&root) {
                    if let Ok(import) = self.extract_multi_import(&m, file_path) {
                        imports.push(import);
                    }
                }
            }

            Ok(imports)
        }

        #[cfg(target_arch = "wasm32")]
        {
            Ok(vec![])
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn extract_function_from_match(&self, m: &NodeMatch<ast_grep_core::tree_sitter::StrDoc<P::Language>>, file_path: &str, has_return_type: bool) -> Result<CodeElement> {
        let env = m.get_env();
        let node = m.get_node();

        let name = env.get_match("NAME")
            .ok_or_else(|| ThreadError::ParseError("Function name not found".to_string()))?
            .text()
            .to_string();

        let params_text = env.get_match("PARAMS")
            .map(|n| n.text().to_string())
            .unwrap_or_default();

        let return_type = if has_return_type {
            env.get_match("RETURN").map(|n| n.text().to_string())
        } else {
            None
        };

        let range = node.range();
        let start_pos = node.start_pos();
        let end_pos = node.end_pos();

        let location = SourceLocation::new(
            file_path.to_string(),
            start_pos.line() + 1, // ast-grep uses 0-based, we want 1-based
            end_pos.line() + 1,
            0, // We'll need to calculate actual column positions
            0,
            range.start,
            range.end,
        );

        let signature = if let Some(ret) = &return_type {
            format!("fn {}({}) -> {}", name, params_text, ret)
        } else {
            format!("fn {}({})", name, params_text)
        };

        Ok(CodeElement {
            id: ElementId(format!("{}:{}", file_path, name)),
            kind: ElementKind::Function,
            name: name.clone(),
            signature,
            location,
            dependencies: Vec::new(), // Will be filled in by relationship analysis
            metadata: ElementMetadata {
                visibility: Some(Visibility::Public), // TODO: detect actual visibility
                is_async: false, // TODO: detect async
                is_generic: false, // TODO: detect generics
                docstring: None, // TODO: extract doc comments
                annotations: Vec::new(),
                return_type,
                parameters: parse_parameters(&params_text),
                extra: std::collections::HashMap::new(),
            },
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn extract_simple_import(&self, m: &NodeMatch<ast_grep_core::tree_sitter::StrDoc<P::Language>>, file_path: &str) -> Result<Import> {
        let env = m.get_env();
        let node = m.get_node();

        let path = env.get_match("PATH")
            .ok_or_else(|| ThreadError::ParseError("Import path not found".to_string()))?
            .text()
            .to_string();

        let range = node.range();
        let start_pos = node.start_pos();
        let end_pos = node.end_pos();

        let location = SourceLocation::new(
            file_path.to_string(),
            start_pos.line() + 1,
            end_pos.line() + 1,
            0,
            0,
            range.start,
            range.end,
        );

        Ok(Import {
            module: path,
            items: vec![], // Simple import doesn't specify items
            alias: None,
            location,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn extract_multi_import(&self, m: &NodeMatch<ast_grep_core::tree_sitter::StrDoc<P::Language>>, file_path: &str) -> Result<Import> {
        let env = m.get_env();
        let node = m.get_node();

        let path = env.get_match("PATH")
            .ok_or_else(|| ThreadError::ParseError("Import path not found".to_string()))?
            .text()
            .to_string();

        // TODO: Parse the ITEMS list properly
        let items = vec![]; // Simplified for now

        let range = node.range();
        let start_pos = node.start_pos();
        let end_pos = node.end_pos();

        let location = SourceLocation::new(
            file_path.to_string(),
            start_pos.line() + 1,
            end_pos.line() + 1,
            0,
            0,
            range.start,
            range.end,
        );

        Ok(Import {
            module: path,
            items,
            alias: None,
            location,
        })
    }
}

impl<P: LanguageProvider> LanguageParser for RustParser<P> {
    fn language_id(&self) -> &'static str {
        P::language_name()
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        P::file_extensions()
    }

    fn parse_content(&self, content: &str, file_path: &str) -> Result<FileParseResult> {
        let start_time = std::time::Instant::now();

        let functions = self.extract_functions(content, file_path)?;
        let imports = self.extract_imports(content, file_path)?;

        let parse_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(FileParseResult {
            file_path: file_path.to_string(),
            language: self.language_id().to_string(),
            elements: functions,
            imports,
            exports: vec![], // TODO: implement export detection
            parse_time_ms,
        })
    }

    fn extract_dependencies(&self, content: &str) -> Result<Vec<String>> {
        // Extract module dependencies from imports
        let imports = self.extract_imports(content, "")?;
        Ok(imports.into_iter().map(|i| i.module).collect())
    }
}

/// Parse function parameters from parameter string
/// This is a simplified parser - a full implementation would use ast-grep patterns
fn parse_parameters(params_text: &str) -> Vec<Parameter> {
    if params_text.trim().is_empty() {
        return vec![];
    }

    // Very basic parameter parsing - in reality we'd want to use ast-grep for this too
    params_text
        .split(',')
        .filter_map(|param| {
            let param = param.trim();
            if param.is_empty() {
                return None;
            }

            // Handle "name: type" pattern
            if let Some((name, type_part)) = param.split_once(':') {
                Some(Parameter {
                    name: name.trim().to_string(),
                    type_annotation: Some(type_part.trim().to_string()),
                    default_value: None,
                    is_optional: false,
                })
            } else {
                // Just a name without type annotation
                Some(Parameter {
                    name: param.to_string(),
                    type_annotation: None,
                    default_value: None,
                    is_optional: false,
                })
            }
        })
        .collect()
}

impl<P: LanguageProvider> Default for RustParser<P> {
    fn default() -> Self {
        Self::new()
    }
}
