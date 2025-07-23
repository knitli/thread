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
//! ### `all-parsers`
//! When enabled (default), imports all tree-sitter parser crates and provides
//! full parser functionality. Disable for WebAssembly builds where tree-sitter
//! cannot be compiled.
//!
//! ### Individual Language Features
//!
//! Each language has its own feature flag, so you can enable whichever ones you need.
//!
//! ### `napi-environment`
//! Disables tree-sitter parsing functionality because tree-sitter cannot build
//! in a NAPI environment. NAPI (Node.js API) is used for building WASM
//! modules for Node.js. (note: WASM builds for browser or other environments can support tree-sitter)
//!
//! #### Napi-compatible Languages
//! You can use these languages in a NAPI environment:
//! - You can use the `napi-compatible` flag as a shortcut to enable all napi-compatible languages.
//! - `css`
//! - `html`
//! - `javascript`
//! - `typescript`
//! - `tsx`
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
//! Parsers use [`std::sync::LazyLock`] for optimal performance:
//! - First call initializes the parser
//! - Subsequent calls return the cached instance
//! - Thread-safe with no synchronization overhead after initialization

#[cfg(all(
    any(
        feature = "all-parsers",
        feature = "bash",
        feature = "c",
        feature = "cpp",
        feature = "csharp",
        feature = "css",
        feature = "elixir",
        feature = "go",
        feature = "haskell",
        feature = "html",
        feature = "java",
        feature = "javascript",
        feature = "json",
        feature = "kotlin",
        feature = "lua",
        feature = "php",
        feature = "python",
        feature = "ruby",
        feature = "rust",
        feature = "scala",
        feature = "swift",
        feature = "tsx",
        feature = "typescript",
        feature = "yaml"
    ),
    not(any(
        feature = "napi-environment",
        feature = "napi-compatible",
        feature = "css-napi",
        feature = "html-napi",
        feature = "javascript-napi",
        feature = "typescript-napi",
        feature = "tsx-napi"
    ))
))]
macro_rules! into_lang {
    ($lang: ident, $field: ident) => {
        $lang::$field.into()
    };
    ($lang: ident) => {
        into_lang!($lang, LANGUAGE)
    };
}

// Napi-environment specific macro
#[cfg(all(
    feature = "napi-environment",
    any(
        feature = "napi-compatible",
        feature = "css-napi",
        feature = "html-napi",
        feature = "javascript-napi",
        feature = "typescript-napi",
        feature = "tsx-napi"
    )
))]
macro_rules! into_lang {
    ($lang: ident, $field: ident) => {
        unimplemented!(
            "tree-sitter parser is not implemented when feature flag [napi-environment] is on."
        )
    };
    ($lang: ident) => {
        into_lang!($lang, LANGUAGE)
    };
}

// With TS-enabled, we can always use the `into_napi_lang!` macro
// to convert the language into a NAPI-compatible type.
// We just can't do it... in NAPI.
#[cfg(
    all(any(
        feature = "all-parsers",
        feature = "bash",
        feature = "c",
        feature = "cpp",
        feature = "csharp",
        feature = "css",
        feature = "elixir",
        feature = "go",
        feature = "haskell",
        feature = "html",
        feature = "java",
        feature = "javascript",
        feature = "json",
        feature = "kotlin",
        feature = "lua",
        feature = "php",
        feature = "python",
        feature = "ruby",
        feature = "rust",
        feature = "scala",
        feature = "swift",
        feature = "tsx",
        feature = "typescript",
        feature = "yaml"
    ),
    not(feature = "napi-environment")
))]
macro_rules! into_napi_lang {
    ($lang: path) => {
        $lang.into()
    };
}

// Napi-environment specific macro
#[cfg(any(
    feature = "napi-environment",
    feature = "napi-compatible",
    feature = "css-napi",
    feature = "html-napi",
    feature = "javascript-napi",
    feature = "typescript-napi",
    feature = "tsx-napi"
))]
macro_rules! into_napi_lang {
    ($lang: path) => {
        $lang.into()
    };
}

use std::sync::OnceLock;
use thread_ast_engine::tree_sitter::TSLanguage;

// Cached language instances for zero-cost repeated access
#[cfg(any(feature = "bash", feature = "all-parsers"))]
static BASH_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(any(feature = "c", feature = "all-parsers"))]
static C_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(any(feature = "cpp", feature = "all-parsers"))]
static CPP_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(any(feature = "csharp", feature = "all-parsers"))]
static CSHARP_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(any(feature = "css", feature = "all-parsers", feature = "css-napi", feature = "napi-compatible"))]
static CSS_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(any(feature = "elixir", feature = "all-parsers"))]
static ELIXIR_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(any(feature = "go", feature = "all-parsers"))]
static GO_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(feature = "haskell")]
static HASKELL_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(any(
    feature = "html",
    feature = "all-parsers",
    feature = "html-napi",
    feature = "napi-compatible"
))]
static HTML_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(any(feature = "java", feature = "all-parsers"))]
static JAVA_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(any(
    feature = "javascript",
    feature = "all-parsers",
    feature = "javascript-napi",
    feature = "napi-compatible"
))]
static JAVASCRIPT_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(any(feature = "json", feature = "all-parsers"))]
static JSON_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(any(feature = "kotlin", feature = "all-parsers"))]
static KOTLIN_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(any(feature = "lua", feature = "all-parsers"))]
static LUA_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(any(feature = "php", feature = "all-parsers"))]
static PHP_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(any(feature = "python", feature = "all-parsers"))]
static PYTHON_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(any(feature = "ruby", feature = "all-parsers"))]
static RUBY_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(any(feature = "rust", feature = "all-parsers"))]
static RUST_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(any(feature = "scala", feature = "all-parsers"))]
static SCALA_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(any(feature = "swift", feature = "all-parsers"))]
static SWIFT_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(any(feature = "tsx", feature = "all-parsers", feature = "tsx-napi", feature = "napi-compatible"))]
static TSX_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(any(
    feature = "typescript",
    feature = "all-parsers",
    feature = "typescript-napi",
    feature = "napi-compatible"
))]
static TYPESCRIPT_LANG: OnceLock<TSLanguage> = OnceLock::new();
#[cfg(any(feature = "yaml", feature = "all-parsers"))]
static YAML_LANG: OnceLock<TSLanguage> = OnceLock::new();

#[cfg(any(feature = "bash", feature = "all-parsers"))]
pub fn language_bash() -> TSLanguage {
    BASH_LANG
        .get_or_init(|| into_lang!(tree_sitter_bash))
        .clone()
}
#[cfg(any(feature = "c", feature = "all-parsers"))]
pub fn language_c() -> TSLanguage {
    C_LANG.get_or_init(|| into_lang!(tree_sitter_c)).clone()
}
#[cfg(any(feature = "cpp", feature = "all-parsers"))]
pub fn language_cpp() -> TSLanguage {
    CPP_LANG.get_or_init(|| into_lang!(tree_sitter_cpp)).clone()
}
#[cfg(any(feature = "csharp", feature = "all-parsers"))]
pub fn language_c_sharp() -> TSLanguage {
    CSHARP_LANG
        .get_or_init(|| into_lang!(tree_sitter_c_sharp))
        .clone()
}
#[cfg(all(any(feature = "css", feature = "all-parsers", feature = "css-napi", feature = "napi-compatible")))]
pub fn language_css() -> TSLanguage {
    CSS_LANG
        .get_or_init(|| into_napi_lang!(tree_sitter_css::LANGUAGE))
        .clone()
}
#[cfg(any(feature = "elixir", feature = "all-parsers"))]
pub fn language_elixir() -> TSLanguage {
    ELIXIR_LANG
        .get_or_init(|| into_lang!(tree_sitter_elixir))
        .clone()
}
#[cfg(any(feature = "go", feature = "all-parsers"))]
pub fn language_go() -> TSLanguage {
    GO_LANG.get_or_init(|| into_lang!(tree_sitter_go)).clone()
}

#[cfg(feature = "haskell")]
pub fn language_haskell() -> TSLanguage {
    HASKELL_LANG
        .get_or_init(|| into_lang!(tree_sitter_haskell))
        .clone()
}
#[cfg(any(
    feature = "html",
    feature = "all-parsers",
    feature = "html-napi",
    feature = "napi-compatible"
))]
pub fn language_html() -> TSLanguage {
    HTML_LANG
        .get_or_init(|| into_napi_lang!(tree_sitter_html::LANGUAGE))
        .clone()
}

#[cfg(any(feature = "java", feature = "all-parsers"))]
pub fn language_java() -> TSLanguage {
    JAVA_LANG
        .get_or_init(|| into_lang!(tree_sitter_java))
        .clone()
}
#[cfg(any(
    feature = "javascript",
    feature = "all-parsers",
    feature = "javascript-napi",
    feature = "napi-compatible"
))]
pub fn language_javascript() -> TSLanguage {
    JAVASCRIPT_LANG
        .get_or_init(|| into_napi_lang!(tree_sitter_javascript::LANGUAGE))
        .clone()
}
#[cfg(any(feature = "json", feature = "all-parsers"))]
pub fn language_json() -> TSLanguage {
    JSON_LANG
        .get_or_init(|| into_lang!(tree_sitter_json))
        .clone()
}
#[cfg(any(feature = "kotlin", feature = "all-parsers"))]
pub fn language_kotlin() -> TSLanguage {
    KOTLIN_LANG
        .get_or_init(|| into_lang!(tree_sitter_kotlin))
        .clone()
}

#[cfg(any(feature = "lua", feature = "all-parsers"))]
pub fn language_lua() -> TSLanguage {
    LUA_LANG.get_or_init(|| into_lang!(tree_sitter_lua)).clone()
}
#[cfg(any(feature = "php", feature = "all-parsers"))]
pub fn language_php() -> TSLanguage {
    PHP_LANG
        .get_or_init(|| into_lang!(tree_sitter_php, LANGUAGE_PHP_ONLY))
        .clone()
}
#[cfg(any(feature = "python", feature = "all-parsers"))]
pub fn language_python() -> TSLanguage {
    PYTHON_LANG
        .get_or_init(|| into_lang!(tree_sitter_python))
        .clone()
}
#[cfg(any(feature = "ruby", feature = "all-parsers"))]
pub fn language_ruby() -> TSLanguage {
    RUBY_LANG
        .get_or_init(|| into_lang!(tree_sitter_ruby))
        .clone()
}
#[cfg(any(feature = "rust", feature = "all-parsers"))]
pub fn language_rust() -> TSLanguage {
    RUST_LANG
        .get_or_init(|| into_lang!(tree_sitter_rust))
        .clone()
}
#[cfg(any(feature = "scala", feature = "all-parsers"))]
pub fn language_scala() -> TSLanguage {
    SCALA_LANG
        .get_or_init(|| into_lang!(tree_sitter_scala))
        .clone()
}
#[cfg(any(feature = "swift", feature = "all-parsers"))]
pub fn language_swift() -> TSLanguage {
    SWIFT_LANG
        .get_or_init(|| into_lang!(tree_sitter_swift))
        .clone()
}
#[cfg(any(feature = "tsx", feature = "all-parsers", feature = "tsx-napi", feature = "napi-compatible"))]
pub fn language_tsx() -> TSLanguage {
    TSX_LANG
        .get_or_init(|| into_napi_lang!(tree_sitter_typescript::LANGUAGE_TSX))
        .clone()
}
#[cfg(any(
    feature = "typescript",
    feature = "all-parsers",
    feature = "typescript-napi",
    feature = "napi-compatible"
))]
pub fn language_typescript() -> TSLanguage {
    TYPESCRIPT_LANG
        .get_or_init(|| into_napi_lang!(tree_sitter_typescript::LANGUAGE_TYPESCRIPT))
        .clone()
}
#[cfg(any(feature = "yaml", feature = "all-parsers"))]
pub fn language_yaml() -> TSLanguage {
    YAML_LANG
        .get_or_init(|| into_lang!(tree_sitter_yaml))
        .clone()
}
