// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

//! This mod maintains a list of tree-sitter parsers crate.
//! When feature flag `builtin-parser` is on, this mod will import all dependent crates.
//! However, tree-sitter bs cannot be compiled by wasm-pack.
//! In this case, we can use a blank implementation by turning feature flag off.
//! And use other implementation.

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
static BASH_LANG: OnceLock<TSLanguage> = OnceLock::new();
static C_LANG: OnceLock<TSLanguage> = OnceLock::new();
static CPP_LANG: OnceLock<TSLanguage> = OnceLock::new();
static CSHARP_LANG: OnceLock<TSLanguage> = OnceLock::new();
static CSS_LANG: OnceLock<TSLanguage> = OnceLock::new();
static ELIXIR_LANG: OnceLock<TSLanguage> = OnceLock::new();
static GO_LANG: OnceLock<TSLanguage> = OnceLock::new();
static HASKELL_LANG: OnceLock<TSLanguage> = OnceLock::new();
static HTML_LANG: OnceLock<TSLanguage> = OnceLock::new();
static JAVA_LANG: OnceLock<TSLanguage> = OnceLock::new();
static JAVASCRIPT_LANG: OnceLock<TSLanguage> = OnceLock::new();
static JSON_LANG: OnceLock<TSLanguage> = OnceLock::new();
static KOTLIN_LANG: OnceLock<TSLanguage> = OnceLock::new();
static LUA_LANG: OnceLock<TSLanguage> = OnceLock::new();
static PHP_LANG: OnceLock<TSLanguage> = OnceLock::new();
static PYTHON_LANG: OnceLock<TSLanguage> = OnceLock::new();
static RUBY_LANG: OnceLock<TSLanguage> = OnceLock::new();
static RUST_LANG: OnceLock<TSLanguage> = OnceLock::new();
static SCALA_LANG: OnceLock<TSLanguage> = OnceLock::new();
static SWIFT_LANG: OnceLock<TSLanguage> = OnceLock::new();
static TSX_LANG: OnceLock<TSLanguage> = OnceLock::new();
static TYPESCRIPT_LANG: OnceLock<TSLanguage> = OnceLock::new();
static YAML_LANG: OnceLock<TSLanguage> = OnceLock::new();

pub fn language_bash() -> TSLanguage {
    BASH_LANG
        .get_or_init(|| into_lang!(tree_sitter_bash))
        .clone()
}

pub fn language_c() -> TSLanguage {
    C_LANG.get_or_init(|| into_lang!(tree_sitter_c)).clone()
}

pub fn language_cpp() -> TSLanguage {
    CPP_LANG.get_or_init(|| into_lang!(tree_sitter_cpp)).clone()
}

pub fn language_c_sharp() -> TSLanguage {
    CSHARP_LANG
        .get_or_init(|| into_lang!(tree_sitter_c_sharp))
        .clone()
}

pub fn language_css() -> TSLanguage {
    CSS_LANG
        .get_or_init(|| into_napi_lang!(tree_sitter_css::LANGUAGE))
        .clone()
}

pub fn language_elixir() -> TSLanguage {
    ELIXIR_LANG
        .get_or_init(|| into_lang!(tree_sitter_elixir))
        .clone()
}

pub fn language_go() -> TSLanguage {
    GO_LANG.get_or_init(|| into_lang!(tree_sitter_go)).clone()
}

pub fn language_haskell() -> TSLanguage {
    HASKELL_LANG
        .get_or_init(|| into_lang!(tree_sitter_haskell))
        .clone()
}

pub fn language_html() -> TSLanguage {
    HTML_LANG
        .get_or_init(|| into_napi_lang!(tree_sitter_html::LANGUAGE))
        .clone()
}

pub fn language_java() -> TSLanguage {
    JAVA_LANG
        .get_or_init(|| into_lang!(tree_sitter_java))
        .clone()
}

pub fn language_javascript() -> TSLanguage {
    JAVASCRIPT_LANG
        .get_or_init(|| into_napi_lang!(tree_sitter_javascript::LANGUAGE))
        .clone()
}

pub fn language_json() -> TSLanguage {
    JSON_LANG
        .get_or_init(|| into_lang!(tree_sitter_json))
        .clone()
}

pub fn language_kotlin() -> TSLanguage {
    KOTLIN_LANG
        .get_or_init(|| into_lang!(tree_sitter_kotlin))
        .clone()
}

pub fn language_lua() -> TSLanguage {
    LUA_LANG.get_or_init(|| into_lang!(tree_sitter_lua)).clone()
}

pub fn language_php() -> TSLanguage {
    PHP_LANG
        .get_or_init(|| into_lang!(tree_sitter_php, LANGUAGE_PHP_ONLY))
        .clone()
}

pub fn language_python() -> TSLanguage {
    PYTHON_LANG
        .get_or_init(|| into_lang!(tree_sitter_python))
        .clone()
}

pub fn language_ruby() -> TSLanguage {
    RUBY_LANG
        .get_or_init(|| into_lang!(tree_sitter_ruby))
        .clone()
}

pub fn language_rust() -> TSLanguage {
    RUST_LANG
        .get_or_init(|| into_lang!(tree_sitter_rust))
        .clone()
}

pub fn language_scala() -> TSLanguage {
    SCALA_LANG
        .get_or_init(|| into_lang!(tree_sitter_scala))
        .clone()
}

pub fn language_swift() -> TSLanguage {
    SWIFT_LANG
        .get_or_init(|| into_lang!(tree_sitter_swift))
        .clone()
}

pub fn language_tsx() -> TSLanguage {
    TSX_LANG
        .get_or_init(|| into_napi_lang!(tree_sitter_typescript::LANGUAGE_TSX))
        .clone()
}

pub fn language_typescript() -> TSLanguage {
    TYPESCRIPT_LANG
        .get_or_init(|| into_napi_lang!(tree_sitter_typescript::LANGUAGE_TYPESCRIPT))
        .clone()
}

pub fn language_yaml() -> TSLanguage {
    YAML_LANG
        .get_or_init(|| into_lang!(tree_sitter_yaml))
        .clone()
}
