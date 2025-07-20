// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

//! Tree-sitter parser initialization and caching for all supported languages.
//!
//! Provides cached, zero-cost access to tree-sitter language parsers. Each parser is initialized once and cached using
//! [`std::sync::OnceLock`] for thread-safe, lazy initialization.
//!
//! ## Feature Flags
//!
//! ### `builtin-parser`
//! When enabled (default), imports all tree-sitter parser crates and provides
//! full parser functionality. Disable for WebAssembly builds where tree-sitter
//! cannot be compiled.
//!
//! ### `napi-lang`
//! Enables NAPI-compatible parsers (CSS, HTML, JavaScript, TypeScript) for
//! Node.js environments.
//!
//! ## Parser Functions
//!
//! Each language has a corresponding `language_*()` function that returns a
//! cached [`TSLanguage`] instance:
//!
//! ```rust
//! use thread_language::parsers::{language_rust, language_javascript};
//!
//! let rust_lang = language_rust();
//! let js_lang = language_javascript();
//! ```
//!
//! ## Caching Strategy
//!
//! Parsers use [`std::sync::OnceLock`] for optimal performance:
//! - First call initializes the parser
//! - Subsequent calls return the cached instance
//! - Thread-safe with no synchronization overhead after initialization

#[cfg(feature = "builtin-parser")]
macro_rules! into_lang {
    ($lang: ident, $field: ident) => {
        $lang::$field.into()
    };
    ($lang: ident) => {
        into_lang!($lang, LANGUAGE)
    };
}

#[cfg(not(feature = "builtin-parser"))]
macro_rules! into_lang {
    ($lang: ident, $field: ident) => {
        unimplemented!(
            "tree-sitter parser is not implemented when feature flag [builtin-parser] is off."
        )
    };
    ($lang: ident) => {
        into_lang!($lang, LANGUAGE)
    };
}

#[cfg(any(feature = "builtin-parser", feature = "napi-lang"))]
macro_rules! into_napi_lang {
    ($lang: path) => {
        $lang.into()
    };
}
#[cfg(not(any(feature = "builtin-parser", feature = "napi-lang")))]
macro_rules! into_napi_lang {
    ($lang: path) => {
        unimplemented!(
            "tree-sitter parser is not implemented when feature flag [builtin-parser] is off."
        )
    };
}

use std::sync::OnceLock;
use thread_ast_engine::tree_sitter::TSLanguage;

// Cached language instances for zero-cost repeated access
#[cfg(feature = "bash")]
static BASH_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(feature = "c")]
static C_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(feature = "cpp")]
static CPP_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(feature = "csharp")]
static CSHARP_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(feature = "css")]
static CSS_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(feature = "elixir")]
static ELIXIR_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(feature = "go")]
static GO_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(feature = "haskell")]
static HASKELL_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(feature = "html")]
static HTML_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(feature = "java")]
static JAVA_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(feature = "javascript")]
static JAVASCRIPT_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(feature = "json")]
static JSON_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(feature = "kotlin")]
static KOTLIN_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(feature = "lua")]
static LUA_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(feature = "php")]
static PHP_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(feature = "python")]
static PYTHON_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(feature = "ruby")]
static RUBY_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(feature = "rust")]
static RUST_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(feature = "scala")]
static SCALA_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(feature = "swift")]
static SWIFT_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(feature = "tsx")]
static TSX_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(feature = "typescript")]
static TYPESCRIPT_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(feature = "yaml")]
static YAML_LANG: OnceLock<TSLanguage> = OnceLock::new();

#[cfg(feature = "bash")]
pub fn language_bash() -> TSLanguage {
    BASH_LANG
        .get_or_init(|| into_lang!(tree_sitter_bash))
        .clone()
}
#[cfg(feature = "c")]
pub fn language_c() -> TSLanguage {
    C_LANG.get_or_init(|| into_lang!(tree_sitter_c)).clone()
}
#[cfg(feature = "cpp")]
pub fn language_cpp() -> TSLanguage {
    CPP_LANG.get_or_init(|| into_lang!(tree_sitter_cpp)).clone()
}
#[cfg(feature = "csharp")]
pub fn language_c_sharp() -> TSLanguage {
    CSHARP_LANG
        .get_or_init(|| into_lang!(tree_sitter_c_sharp))
        .clone()
}
#[cfg(feature = "css")]
pub fn language_css() -> TSLanguage {
    CSS_LANG
        .get_or_init(|| into_napi_lang!(tree_sitter_css::LANGUAGE))
        .clone()
}
#[cfg(feature = "elixir")]
pub fn language_elixir() -> TSLanguage {
    ELIXIR_LANG
        .get_or_init(|| into_lang!(tree_sitter_elixir))
        .clone()
}
#[cfg(feature = "go")]
pub fn language_go() -> TSLanguage {
    GO_LANG.get_or_init(|| into_lang!(tree_sitter_go)).clone()
}

#[cfg(feature = "haskell")]
pub fn language_haskell() -> TSLanguage {
    HASKELL_LANG
        .get_or_init(|| into_lang!(tree_sitter_haskell))
        .clone()
}
#[cfg(feature = "html")]
pub fn language_html() -> TSLanguage {
    HTML_LANG
        .get_or_init(|| into_napi_lang!(tree_sitter_html::LANGUAGE))
        .clone()
}

#[cfg(feature = "java")]
pub fn language_java() -> TSLanguage {
    JAVA_LANG
        .get_or_init(|| into_lang!(tree_sitter_java))
        .clone()
}
#[cfg(feature = "javascript")]
pub fn language_javascript() -> TSLanguage {
    JAVASCRIPT_LANG
        .get_or_init(|| into_napi_lang!(tree_sitter_javascript::LANGUAGE))
        .clone()
}
#[cfg(feature = "json")]
pub fn language_json() -> TSLanguage {
    JSON_LANG
        .get_or_init(|| into_lang!(tree_sitter_json))
        .clone()
}
#[cfg(feature = "kotlin")]
pub fn language_kotlin() -> TSLanguage {
    KOTLIN_LANG
        .get_or_init(|| into_lang!(tree_sitter_kotlin))
        .clone()
}

#[cfg(feature = "lua")]
pub fn language_lua() -> TSLanguage {
    LUA_LANG.get_or_init(|| into_lang!(tree_sitter_lua)).clone()
}
#[cfg(feature = "php")]
pub fn language_php() -> TSLanguage {
    PHP_LANG
        .get_or_init(|| into_lang!(tree_sitter_php, LANGUAGE_PHP_ONLY))
        .clone()
}
#[cfg(feature = "python")]
pub fn language_python() -> TSLanguage {
    PYTHON_LANG
        .get_or_init(|| into_lang!(tree_sitter_python))
        .clone()
}
#[cfg(feature = "ruby")]
pub fn language_ruby() -> TSLanguage {
    RUBY_LANG
        .get_or_init(|| into_lang!(tree_sitter_ruby))
        .clone()
}
#[cfg(feature = "rust")]
pub fn language_rust() -> TSLanguage {
    RUST_LANG
        .get_or_init(|| into_lang!(tree_sitter_rust))
        .clone()
}
#[cfg(feature = "scala")]
pub fn language_scala() -> TSLanguage {
    SCALA_LANG
        .get_or_init(|| into_lang!(tree_sitter_scala))
        .clone()
}
#[cfg(feature = "swift")]
pub fn language_swift() -> TSLanguage {
    SWIFT_LANG
        .get_or_init(|| into_lang!(tree_sitter_swift))
        .clone()
}
#[cfg(feature = "tsx")]
pub fn language_tsx() -> TSLanguage {
    TSX_LANG
        .get_or_init(|| into_napi_lang!(tree_sitter_typescript::LANGUAGE_TSX))
        .clone()
}
#[cfg(feature = "typescript")]
pub fn language_typescript() -> TSLanguage {
    TYPESCRIPT_LANG
        .get_or_init(|| into_napi_lang!(tree_sitter_typescript::LANGUAGE_TYPESCRIPT))
        .clone()
}
#[cfg(feature = "yaml")]
pub fn language_yaml() -> TSLanguage {
    YAML_LANG
        .get_or_init(|| into_lang!(tree_sitter_yaml))
        .clone()
}
