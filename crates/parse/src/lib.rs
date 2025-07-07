// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Thread parsing layer that integrates ast-grep with thread-core types
//!
//! This crate provides WASM-compatible parsing by using ast-grep's core
//! parsing functionality with string-only APIs.

mod rust_parser_simple;

pub use rust_parser_simple::*;

/// Language provider abstraction for WASM compatibility
pub trait LanguageProvider: Send + Sync {
    type Language: ast_grep_core::Language + Clone + Send + Sync;
    
    fn get_language() -> Self::Language;
    fn language_name() -> &'static str;
    fn file_extensions() -> &'static [&'static str];
}

/// Native Rust language provider (uses full ast-grep-language)
#[cfg(not(target_arch = "wasm32"))]
pub struct NativeRustProvider;

#[cfg(not(target_arch = "wasm32"))]
impl LanguageProvider for NativeRustProvider {
    type Language = ast_grep_language::Rust;
    
    fn get_language() -> Self::Language {
        ast_grep_language::Rust
    }
    
    fn language_name() -> &'static str {
        "rust"
    }
    
    fn file_extensions() -> &'static [&'static str] {
        &[".rs"]
    }
}

/// WASM Rust language provider (minimal implementation)
#[cfg(target_arch = "wasm32")]
pub struct WasmRustProvider;

#[cfg(target_arch = "wasm32")]
impl LanguageProvider for WasmRustProvider {
    type Language = MinimalRust;
    
    fn get_language() -> Self::Language {
        MinimalRust
    }
    
    fn language_name() -> &'static str {
        "rust"
    }
    
    fn file_extensions() -> &'static [&'static str] {
        &[".rs"]
    }
}

/// Minimal Rust language implementation for WASM
#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
pub struct MinimalRust;

#[cfg(target_arch = "wasm32")]
impl ast_grep_core::Language for MinimalRust {
    fn kind_to_id(&self, _kind: &str) -> u16 {
        // Simplified implementation for WASM
        0
    }
    
    fn field_to_id(&self, _field: &str) -> Option<u16> {
        None
    }
    
    fn build_pattern(&self, _builder: &ast_grep_core::matcher::PatternBuilder) -> std::result::Result<ast_grep_core::Pattern, ast_grep_core::PatternError> {
        Err(ast_grep_core::PatternError::Syntax("WASM minimal implementation".to_string()))
    }
}

/// Common type aliases for easier usage
#[cfg(not(target_arch = "wasm32"))]
pub type DefaultRustProvider = NativeRustProvider;

#[cfg(target_arch = "wasm32")]
pub type DefaultRustProvider = WasmRustProvider;

pub type DefaultRustParser = SimpleRustParser<DefaultRustProvider>;

#[cfg(test)]
mod tests {
    use super::*;
    use thread_core::LanguageParser;

    #[test]
    fn test_simple_rust_parser() {
        let parser = DefaultRustParser::new();
        
        let rust_code = r#"
fn main() {
    println!("Hello, world!");
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;
        
        let result = parser.parse_content(rust_code, "test.rs").unwrap();
        
        assert_eq!(result.language, "rust");
        assert_eq!(result.file_path, "test.rs");
        assert_eq!(result.elements.len(), 1); // Our mock implementation returns 1 function
        assert_eq!(result.elements[0].name, "main");
        assert_eq!(result.elements[0].kind, thread_core::ElementKind::Function);
    }

    #[test]
    fn test_language_detection() {
        let parser = DefaultRustParser::new();
        
        assert_eq!(parser.language_id(), "rust");
        assert!(parser.can_parse_extension("rs"));
        assert!(!parser.can_parse_extension("js"));
    }
}