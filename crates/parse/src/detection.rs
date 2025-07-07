// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

// crates/thread-lang/src/detection.rs
use ast_grep_core::{AstGrep, Language};
use std::path::Path;
use std::collections::HashMap;
use anyhow::Result;

pub struct LanguageDetector {
    extensions: HashMap<String, SupportedLanguage>,
    custom_languages: HashMap<String, CustomLanguage>,
}

#[derive(Debug, Clone)]
pub enum SupportedLanguage {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct CustomLanguage {
    pub name: String,
    pub library_path: String,
    pub extensions: Vec<String>,
    pub expando_char: char,
}

impl LanguageDetector {
    pub fn new() -> Self {
        let mut extensions = HashMap::new();
        extensions.insert("rs".to_string(), SupportedLanguage::Rust);
        extensions.insert("js".to_string(), SupportedLanguage::JavaScript);
        extensions.insert("mjs".to_string(), SupportedLanguage::JavaScript);
        extensions.insert("ts".to_string(), SupportedLanguage::TypeScript);
        extensions.insert("tsx".to_string(), SupportedLanguage::TypeScript);
        extensions.insert("py".to_string(), SupportedLanguage::Python);

        Self {
            extensions,
            custom_languages: HashMap::new(),
        }
    }

    pub fn detect_language<P: AsRef<Path>>(&self, file_path: P) -> Option<SupportedLanguage> {
        let path = file_path.as_ref();
        let extension = path.extension()?.to_str()?;

        self.extensions.get(extension).cloned()
    }

    pub fn register_custom_language(&mut self, lang: CustomLanguage) -> Result<()> {
        // Register with ast-grep
        ast_grep_core::register_dynamic_language(
            &lang.name,
            &lang.library_path,
            &lang.extensions.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            lang.expando_char,
        )?;

        // Update extension mapping
        for ext in &lang.extensions {
            self.extensions.insert(ext.clone(), SupportedLanguage::Custom(lang.name.clone()));
        }

        self.custom_languages.insert(lang.name.clone(), lang);
        Ok(())
    }

    pub fn analyze_with_ast_grep(&self, content: &str, language: &SupportedLanguage) -> Result<Vec<AstGrepMatch>> {
        let ast_grep_lang = match language {
            SupportedLanguage::Rust => ast_grep_core::Language::Rust,
            SupportedLanguage::JavaScript => ast_grep_core::Language::JavaScript,
            SupportedLanguage::TypeScript => ast_grep_core::Language::TypeScript,
            SupportedLanguage::Python => ast_grep_core::Language::Python,
            SupportedLanguage::Custom(name) => {
                // Use custom language
                return Ok(vec![]); // Placeholder
            }
        };

        let ast = AstGrep::new(content, ast_grep_lang);
        let root = ast.root();

        // Example: Find all function definitions
        let matches = root.find_all("function $NAME($PARAMS) { $BODY }");

        Ok(matches.into_iter().map(|node| AstGrepMatch {
            text: node.text().to_string(),
            start_line: node.start_position().row,
            start_column: node.start_position().column,
            end_line: node.end_position().row,
            end_column: node.end_position().column,
        }).collect())
    }
}

#[derive(Debug, Clone)]
pub struct AstGrepMatch {
    pub text: String,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}
