// SPDX-FileCopyrightText: 2025 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: MIT


//! This mod maintains a list of tree-sitter parsers crate.
//! When feature flag `all` is on, this mod will import all dependent crates.

#[cfg(any(feature = "bash", feature = "c", feature = "cpp", feature = "c-sharp", feature = "elixir", feature = "go", feature = "haskell", feature = "java", feature = "json", feature = "kotlin", feature = "lua", feature = "php", feature = "python", feature = "ruby", feature = "rust", feature = "scala", feature = "swift", feature = "yaml"))]
macro_rules! into_lang {
    ($lang: ident, $field: ident) => {
        $lang::$field.into()
    };
    ($lang: ident) => {
        into_lang!($lang, LANGUAGE)
    };
}

#[cfg(not(any(feature = "bash", feature = "c", feature = "cpp", feature = "c-sharp", feature = "elixir", feature = "go", feature = "haskell", feature = "java", feature = "json", feature = "kotlin", feature = "lua", feature = "php", feature = "python", feature = "ruby", feature = "rust", feature = "scala", feature = "swift", feature = "yaml")))]
macro_rules! into_lang {
    ($lang: ident, $field: ident) => {
        unimplemented!("tree-sitter parser is not implemented when there are no non-web feature flags enabled")
    };
    ($lang: ident) => {
        into_lang!($lang, LANGUAGE)
    };
}

#[cfg(any(feature = "javascript", feature = "typescript", feature = "tsx", feature = "css", feature = "html"))]
macro_rules! into_napi_lang {
    ($lang: path) => {
        $lang.into()
    };
}
#[cfg(not(any(feature = "javascript", feature = "typescript", feature = "tsx", feature = "css", feature = "html")))]
macro_rules! into_napi_lang {
    ($lang: path) => {
        unimplemented!("tree-sitter parser is not implemented when there are no non-web feature flags enabled.")
    };
}

use ag_service_core::TSLanguage;

#[cfg(feature = "bash")]
pub fn language_bash() -> TSLanguage {
    into_lang!(tree_sitter_bash)
}
#[cfg(feature = "c")]
pub fn language_c() -> TSLanguage {
    into_lang!(tree_sitter_c)
}
#[cfg(feature = "cpp")]
pub fn language_cpp() -> TSLanguage {
    into_lang!(tree_sitter_cpp)
}
#[cfg(feature = "c-sharp")]
pub fn language_c_sharp() -> TSLanguage {
    into_lang!(tree_sitter_c_sharp)
}
#[cfg(feature = "css")]
pub fn language_css() -> TSLanguage {
    into_napi_lang!(tree_sitter_css::LANGUAGE)
}
#[cfg(feature = "elixir")]
pub fn language_elixir() -> TSLanguage {
    into_lang!(tree_sitter_elixir)
}
#[cfg(feature = "go")]
pub fn language_go() -> TSLanguage {
    into_lang!(tree_sitter_go)
}
#[cfg(feature = "haskell")]
pub fn language_haskell() -> TSLanguage {
    into_lang!(tree_sitter_haskell)
}
#[cfg(feature = "html")]
pub fn language_html() -> TSLanguage {
    into_napi_lang!(tree_sitter_html::LANGUAGE)
}
#[cfg(feature = "java")]
pub fn language_java() -> TSLanguage {
    into_lang!(tree_sitter_java)
}
#[cfg(any(feature = "javascript", feature = "web"))]
pub fn language_javascript() -> TSLanguage {
    into_napi_lang!(tree_sitter_javascript::LANGUAGE)
}
#[cfg(any(feature = "json"))]
pub fn language_json() -> TSLanguage {
    into_lang!(tree_sitter_json)
}
#[cfg(any(feature = "kotlin"))]
pub fn language_kotlin() -> TSLanguage {
    into_lang!(tree_sitter_kotlin)
}
#[cfg(feature = "lua")]
pub fn language_lua() -> TSLanguage {
    into_lang!(tree_sitter_lua)
}
#[cfg(feature = "php")]
pub fn language_php() -> TSLanguage {
    into_lang!(tree_sitter_php, LANGUAGE_PHP_ONLY)
}
#[cfg(feature = "python")]
pub fn language_python() -> TSLanguage {
    into_lang!(tree_sitter_python)
}
#[cfg(feature = "ruby")]
pub fn language_ruby() -> TSLanguage {
    into_lang!(tree_sitter_ruby)
}
#[cfg(feature = "rust")]
pub fn language_rust() -> TSLanguage {
    into_lang!(tree_sitter_rust)
}
#[cfg(feature = "scala")]
pub fn language_scala() -> TSLanguage {
    into_lang!(tree_sitter_scala)
}
#[cfg(feature = "swift")]
pub fn language_swift() -> TSLanguage {
    into_lang!(tree_sitter_swift)
}
#[cfg(feature = "tsx")]
pub fn language_tsx() -> TSLanguage {
    into_napi_lang!(tree_sitter_typescript::LANGUAGE_TSX)
}
#[cfg(feature = "typescript")]
pub fn language_typescript() -> TSLanguage {
    into_napi_lang!(tree_sitter_typescript::LANGUAGE_TYPESCRIPT)
}
#[cfg(feature = "yaml")]
pub fn language_yaml() -> TSLanguage {
    into_lang!(tree_sitter_yaml)
}
