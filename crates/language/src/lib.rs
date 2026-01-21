// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

//! Language definitions and tree-sitter parsers for Thread AST analysis.
//!
//! Provides unified language support through consistent [`Language`] and [`LanguageExt`] traits
//! across 24+ programming languages. Each language can be feature-gated individually or included
//! in groups.
//!
//! ## Language Categories
//!
//! ### Standard Languages
//! Languages that accept `$` as a valid identifier character and use default pattern processing:
//! - [`Bash`], [`Java`], [`JavaScript`], [`Json`], [`Lua`], [`Scala`], [`TypeScript`], [`Tsx`], [`Yaml`]
//!
//! ### Custom Pattern Languages
//! Languages requiring special metavariable handling with custom expando characters:
//! - [`C`] (`µ`), [`Cpp`] (`µ`), [`CSharp`] (`µ`), [`Css`] (`_`), [`Elixir`] (`µ`)
//! - [`Go`] (`µ`), [`Haskell`] (`µ`), [`Html`] (`z`), [`Kotlin`] (`µ`), [`Php`] (`µ`)
//! - [`Python`] (`µ`), [`Ruby`] (`µ`), [`Rust`] (`µ`), [`Swift`] (`µ`)
//!
//! ## Usage
//!
//! ```rust
//! use thread_language::{SupportLang, Rust};
//! use thread_ast_engine::{Language, LanguageExt};
//!
//! // Runtime language selection
//! let lang = SupportLang::from_path("main.rs").unwrap();
//! let tree = lang.ast_grep("fn main() {}");
//!
//! // Compile-time language selection
//! let rust = Rust;
//! let tree = rust.ast_grep("fn main() {}");
//! ```
//!
//! ## Implementation Details
//!
//! Languages are implemented using two macros:
//! - [`impl_lang!`] - Standard languages accepting `$` in identifiers
//! - [`impl_lang_expando!`] - Languages requiring custom expando characters for metavariables

pub mod constants;
pub mod ext_iden;
#[cfg(any(
    feature = "all-parsers",
    feature = "napi-environment",
    feature = "napi-compatible",
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
))]
pub mod parsers;

#[cfg(any(feature = "bash", feature = "all-parsers"))]
mod bash;
#[cfg(any(feature = "cpp", feature = "all-parsers"))]
mod cpp;
#[cfg(any(feature = "csharp", feature = "all-parsers"))]
mod csharp;
#[cfg(any(
    feature = "css",
    feature = "all-parsers",
    feature = "css-napi",
    feature = "napi-compatible"
))]
mod css;
#[cfg(any(feature = "elixir", feature = "all-parsers"))]
mod elixir;
#[cfg(any(feature = "go", feature = "all-parsers"))]
mod go;
#[cfg(feature = "haskell")]
mod haskell;
#[cfg(any(
    feature = "html",
    feature = "all-parsers",
    feature = "html-napi",
    feature = "napi-compatible"
))]
mod html;
#[cfg(any(feature = "json", feature = "all-parsers"))]
mod json;
#[cfg(any(feature = "kotlin", feature = "all-parsers"))]
mod kotlin;
#[cfg(any(feature = "lua", feature = "all-parsers"))]
mod lua;
#[cfg(any(feature = "php", feature = "all-parsers"))]
mod php;
#[cfg(any(feature = "python", feature = "all-parsers"))]
mod python;
#[cfg(any(feature = "ruby", feature = "all-parsers"))]
mod ruby;
#[cfg(any(feature = "rust", feature = "all-parsers"))]
mod rust;
#[cfg(any(feature = "scala", feature = "all-parsers"))]
mod scala;
#[cfg(any(feature = "swift", feature = "all-parsers"))]
mod swift;
#[cfg(any(feature = "yaml", feature = "all-parsers"))]
mod yaml;
#[cfg(any(
    feature = "html",
    feature = "all-parsers",
    feature = "html-napi",
    feature = "napi-compatible"
))]
pub use html::Html;

#[cfg(feature = "matching")]
use thread_ast_engine::{Pattern, PatternBuilder, PatternError};
#[cfg(feature = "profiling")]
pub mod profiling;

use ignore::types::{Types, TypesBuilder};
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, de};
use std::borrow::Cow;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::str::FromStr;
#[cfg(feature = "matching")]
use thread_ast_engine::Node;
use thread_ast_engine::meta_var::MetaVariable;
#[cfg(feature = "matching")]
use thread_ast_engine::tree_sitter::{StrDoc, TSRange};
#[cfg(any(
    feature = "all-parsers",
    feature = "napi-compatible",
    feature = "css-napi",
    feature = "html-napi",
    feature = "javascript-napi",
    feature = "typescript-napi",
    feature = "tsx-napi",
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
))]
pub use thread_ast_engine::{
    language::Language,
    tree_sitter::{LanguageExt, TSLanguage},
};
#[cfg(feature = "matching")]
use thread_utils::RapidMap;

/// Implements standard [`Language`] and [`LanguageExt`] traits for languages that accept `$` in identifiers.
///
/// Used for languages like JavaScript, Python, and Rust where `$` can appear in variable names
/// and doesn't require special preprocessing for metavariables.
///
/// # Parameters
/// - `$lang` - The language struct name (e.g., `JavaScript`)
/// - `$func` - The parser function name from [`parsers`] module (e.g., `language_javascript`)
///
/// # Generated Implementation
/// Creates a zero-sized struct with [`Language`] and [`LanguageExt`] implementations that:
/// - Map node kinds and field names to tree-sitter IDs
/// - Build patterns using the language's parser
/// - Use default metavariable processing (no expando character substitution)
#[cfg(any(
    feature = "all-parsers",
    feature = "napi-compatible",
    feature = "bash",
    feature = "java",
    feature = "javascript",
    feature = "javascript-napi",
    feature = "json",
    feature = "lua",
    feature = "scala",
    feature = "tsx",
    feature = "tsx-napi",
    feature = "typescript",
    feature = "typescript-napi",
    feature = "yaml",
))]
macro_rules! impl_lang {
    ($lang: ident, $func: ident) => {
        #[derive(Clone, Copy, Debug)]
        pub struct $lang;
        impl Language for $lang {
            fn kind_to_id(&self, kind: &str) -> u16 {
                self.get_ts_language()
                    .id_for_node_kind(kind, /*named*/ true)
            }
            fn field_to_id(&self, field: &str) -> Option<u16> {
                self.get_ts_language()
                    .field_id_for_name(field)
                    .map(|f| f.get())
            }
            #[cfg(feature = "matching")]
            fn build_pattern(&self, builder: &PatternBuilder) -> Result<Pattern, PatternError> {
                builder.build(|src| StrDoc::try_new(src, self.clone()))
            }
        }
        impl LanguageExt for $lang {
            fn get_ts_language(&self) -> TSLanguage {
                parsers::$func().into()
            }
        }
    };
}

/// Preprocesses pattern strings by replacing `$` with the language's expando character.
///
/// Languages that don't accept `$` in identifiers need metavariables like `$VAR` converted
/// to use a different character. This function efficiently replaces `$` symbols that precede
/// uppercase letters, underscores, or appear in triple sequences (`$$$`).
///
/// # Parameters
/// - `expando` - The character to replace `$` with (e.g., `µ` for most languages, `_` for CSS)
/// - `query` - The pattern string containing `$` metavariables
///
/// # Returns
/// - `Cow::Borrowed` if no replacement is needed (fast path)
/// - `Cow::Owned` if replacement occurred
///
/// # Examples
/// ```rust
/// # use thread_language::pre_process_pattern;
/// // Python doesn't accept $ in identifiers, so use µ
/// let result = pre_process_pattern('µ', "def $FUNC($ARG): pass");
/// assert_eq!(result, "def µFUNC(µARG): pass");
///
/// // No change needed
/// let result = pre_process_pattern('µ', "def hello(): pass");
/// assert_eq!(result, "def hello(): pass");
/// ```
fn pre_process_pattern(expando: char, query: &str) -> std::borrow::Cow<'_, str> {
    // Fast path: check if any processing is needed
    let has_dollar = query.as_bytes().contains(&b'$');
    if !has_dollar {
        return std::borrow::Cow::Borrowed(query);
    }

    // Count exact size needed to avoid reallocations
    let mut size_needed = 0;
    let mut needs_processing = false;
    let mut dollar_count = 0;

    for c in query.chars() {
        if c == '$' {
            dollar_count += 1;
        } else {
            let need_replace = matches!(c, 'A'..='Z' | '_') || dollar_count == 3;
            if need_replace && dollar_count > 0 {
                needs_processing = true;
            }
            size_needed += dollar_count + 1;
            dollar_count = 0;
        }
    }
    size_needed += dollar_count;

    // If no replacement needed, return borrowed
    if !needs_processing {
        return std::borrow::Cow::Borrowed(query);
    }

    // Pre-allocate exact size and process in-place
    let mut ret = String::with_capacity(size_needed);
    dollar_count = 0;

    for c in query.chars() {
        if c == '$' {
            dollar_count += 1;
            continue;
        }
        let need_replace = matches!(c, 'A'..='Z' | '_') || dollar_count == 3;
        let sigil = if need_replace { expando } else { '$' };

        // Push dollars directly without iterator allocation
        for _ in 0..dollar_count {
            ret.push(sigil);
        }
        dollar_count = 0;
        ret.push(c);
    }

    // Handle trailing dollars
    let sigil = if dollar_count == 3 { expando } else { '$' };
    for _ in 0..dollar_count {
        ret.push(sigil);
    }

    std::borrow::Cow::Owned(ret)
}

/// Implements [`Language`] and [`LanguageExt`] traits for languages requiring custom expando characters.
///
/// Used for languages that don't accept `$` in identifiers and need metavariables like `$VAR`
/// converted to use a different character (e.g., `µVAR`, `_VAR`).
///
/// # Parameters
/// - `$lang` - The language struct name (e.g., `Python`)
/// - `$func` - The parser function name from [`parsers`] module (e.g., `language_python`)
/// - `$char` - The expando character to use instead of `$` (e.g., `'µ'`)
///
/// # Generated Implementation
/// Creates a zero-sized struct with [`Language`] and [`LanguageExt`] implementations that:
/// - Map node kinds and field names to tree-sitter IDs
/// - Build patterns using the language's parser
/// - Preprocess patterns by replacing `$` with the expando character
/// - Provide the expando character via [`Language::expando_char`]
///
/// # Examples
/// ```rust
/// # use thread_language::Python;
/// # use thread_ast_engine::Language;
/// let python = Python;
/// assert_eq!(python.expando_char(), 'µ');
///
/// // Pattern gets automatically preprocessed
/// let pattern = "def $FUNC($ARG): pass";
/// let processed = python.pre_process_pattern(pattern);
/// assert_eq!(processed, "def µFUNC(µARG): pass");
/// ```
#[cfg(any(
    feature = "all-parsers",
    feature = "napi-compatible",
    feature = "c",
    feature = "cpp",
    feature = "csharp",
    feature = "css",
    feature = "css-napi",
    feature = "elixir",
    feature = "go",
    feature = "haskell",
    feature = "html",
    feature = "html-napi",
    feature = "kotlin",
    feature = "php",
    feature = "python",
    feature = "ruby",
    feature = "rust",
))]
macro_rules! impl_lang_expando {
    ($lang: ident, $func: ident, $char: expr) => {
        #[derive(Clone, Copy, Debug)]
        pub struct $lang;
        impl Language for $lang {
            fn kind_to_id(&self, kind: &str) -> u16 {
                self.get_ts_language()
                    .id_for_node_kind(kind, /*named*/ true)
            }
            fn field_to_id(&self, field: &str) -> Option<u16> {
                self.get_ts_language()
                    .field_id_for_name(field)
                    .map(|f| f.get())
            }
            fn expando_char(&self) -> char {
                $char
            }
            fn pre_process_pattern<'q>(&self, query: &'q str) -> std::borrow::Cow<'q, str> {
                pre_process_pattern(self.expando_char(), query)
            }
            #[cfg(feature = "matching")]
            fn build_pattern(&self, builder: &PatternBuilder) -> Result<Pattern, PatternError> {
                builder.build(|src| StrDoc::try_new(src, self.clone()))
            }
        }
        impl LanguageExt for $lang {
            fn get_ts_language(&self) -> TSLanguage {
                $crate::parsers::$func().into()
            }
        }
    };
}

pub trait Alias: Display {
    const ALIAS: &'static [&'static str];
}

/// Implements the `ALIAS` associated constant for the given lang, which is
/// then used to define the `alias` const fn and a `Deserialize` impl.
#[cfg(all(
    any(
        feature = "all-parsers",
        feature = "napi-compatible",
        feature = "css-napi",
        feature = "html-napi",
        feature = "javascript-napi",
        feature = "typescript-napi",
        feature = "tsx-napi",
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
    not(feature = "no-enabled-langs")
))]
macro_rules! impl_alias {
    ($lang:ident => $as:expr) => {
        impl Alias for $lang {
            const ALIAS: &'static [&'static str] = $as;
        }

        impl fmt::Display for $lang {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:?}", self)
            }
        }

        impl<'de> Deserialize<'de> for $lang {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let vis = AliasVisitor {
                    aliases: Self::ALIAS,
                };
                deserializer.deserialize_str(vis)?;
                Ok($lang)
            }
        }

        impl From<$lang> for SupportLang {
            fn from(_: $lang) -> Self {
                Self::$lang
            }
        }
    };
}
/// Generates as convenience conversions between the lang types
/// and `SupportedType`.
#[cfg(all(
    any(
        feature = "all-parsers",
        feature = "napi-compatible",
        feature = "css-napi",
        feature = "html-napi",
        feature = "javascript-napi",
        feature = "typescript-napi",
        feature = "tsx-napi",
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
    not(feature = "no-enabled-langs")
))]
macro_rules! impl_aliases {
  ($($lang:ident, $feature:literal => $as:expr),* $(,)?) => {
    $(#[cfg(feature = $feature)]
      impl_alias!($lang => $as);
    )*
    #[allow(dead_code)]
    const fn alias(lang: SupportLang) -> &'static [&'static str] {
      match lang {
        $(
          #[cfg(feature = $feature)]
          SupportLang::$lang => $lang::ALIAS,
        )*
      }
    }
  };
}
/* Customized Language with expando_char / pre_process_pattern */

// https://en.cppreference.com/w/cpp/language/identifiers
// Due to some issues in the tree-sitter parser, it is not possible to use
// unicode literals in identifiers for C/C++ parsers
#[cfg(any(feature = "c", feature = "all-parsers"))]
impl_lang_expando!(C, language_c, 'µ');
#[cfg(any(feature = "cpp", feature = "all-parsers"))]
impl_lang_expando!(Cpp, language_cpp, 'µ');

// https://docs.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/lexical-structure#643-identifiers
// all letter number is accepted
// https://www.compart.com/en/unicode/category/Nl
#[cfg(any(feature = "csharp", feature = "all-parsers"))]
impl_lang_expando!(CSharp, language_c_sharp, 'µ');

// https://www.w3.org/TR/CSS21/grammar.html#scanner
#[cfg(any(
    feature = "css",
    feature = "all-parsers",
    feature = "css-napi",
    feature = "napi-compatible"
))]
impl_lang_expando!(Css, language_css, '_');

// https://github.com/elixir-lang/tree-sitter-elixir/blob/a2861e88a730287a60c11ea9299c033c7d076e30/grammar.js#L245
#[cfg(any(feature = "elixir", feature = "all-parsers"))]
impl_lang_expando!(Elixir, language_elixir, 'µ');

// we can use any Unicode code point categorized as "Letter"
// https://go.dev/ref/spec#letter
#[cfg(any(feature = "go", feature = "all-parsers"))]
impl_lang_expando!(Go, language_go, 'µ');

// GHC supports Unicode syntax per
// https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/unicode_syntax.html
// and the tree-sitter-haskell grammar parses it too.
#[cfg(feature = "haskell")]
impl_lang_expando!(Haskell, language_haskell, 'µ');

// https://github.com/fwcd/tree-sitter-kotlin/pull/93
#[cfg(any(feature = "kotlin", feature = "all-parsers"))]
impl_lang_expando!(Kotlin, language_kotlin, 'µ');

// PHP accepts unicode to be used as some name not var name though
#[cfg(any(feature = "php", feature = "all-parsers"))]
impl_lang_expando!(Php, language_php, 'µ');

// we can use any char in unicode range [:XID_Start:]
// https://docs.python.org/3/reference/lexical_analysis.html#identifiers
// see also [PEP 3131](https://peps.python.org/pep-3131/) for further details.
#[cfg(any(feature = "python", feature = "all-parsers"))]
impl_lang_expando!(Python, language_python, 'µ');

// https://github.com/tree-sitter/tree-sitter-ruby/blob/f257f3f57833d584050336921773738a3fd8ca22/grammar.js#L30C26-L30C78
#[cfg(any(feature = "ruby", feature = "all-parsers"))]
impl_lang_expando!(Ruby, language_ruby, 'µ');

// we can use any char in unicode range [:XID_Start:]
// https://doc.rust-lang.org/reference/identifiers.html
#[cfg(any(feature = "rust", feature = "all-parsers"))]
impl_lang_expando!(Rust, language_rust, 'µ');

//https://docs.swift.org/swift-book/documentation/the-swift-programming-language/lexicalstructure/#Identifiers
#[cfg(any(feature = "swift", feature = "all-parsers"))]
impl_lang_expando!(Swift, language_swift, 'µ');

// Stub Language without preprocessing
// Language Name, tree-sitter-name, alias, extension
#[cfg(any(feature = "bash", feature = "all-parsers"))]
impl_lang!(Bash, language_bash);
#[cfg(any(feature = "java", feature = "all-parsers"))]
impl_lang!(Java, language_java);
#[cfg(any(
    feature = "javascript",
    feature = "all-parsers",
    feature = "javascript-napi",
    feature = "napi-compatible"
))]
impl_lang!(JavaScript, language_javascript);
#[cfg(any(feature = "json", feature = "all-parsers"))]
impl_lang!(Json, language_json);
#[cfg(any(feature = "lua", feature = "all-parsers"))]
impl_lang!(Lua, language_lua);
#[cfg(any(feature = "scala", feature = "all-parsers"))]
impl_lang!(Scala, language_scala);
#[cfg(any(
    feature = "tsx",
    feature = "all-parsers",
    feature = "tsx-napi",
    feature = "napi-compatible"
))]
impl_lang!(Tsx, language_tsx);
#[cfg(any(
    feature = "typescript",
    feature = "all-parsers",
    feature = "typescript-napi",
    feature = "napi-compatible"
))]
impl_lang!(TypeScript, language_typescript);
#[cfg(any(feature = "yaml", feature = "all-parsers"))]
impl_lang!(Yaml, language_yaml);

// See ripgrep for extensions
// https://github.com/BurntSushi/ripgrep/blob/master/crates/ignore/src/default_types.rs

/// Runtime language selection enum supporting all built-in languages.
///
/// Provides a unified interface for working with any supported language at runtime.
/// Each variant corresponds to a specific programming language implementation.
///
/// # Language Detection
/// ```rust,ignore
/// use thread_language::SupportLang;
/// use std::path::Path;
///
/// // Detect from file extension
/// let lang = SupportLang::from_path("main.rs").unwrap();
/// assert_eq!(lang, SupportLang::Rust);
///
/// // Parse from string
/// let lang: SupportLang = "javascript".parse().unwrap();
/// assert_eq!(lang, SupportLang::JavaScript);
/// ```
///
/// # Usage with AST Analysis
/// ```rust,ignore
/// use thread_language::SupportLang;
/// use thread_ast_engine::{Language, LanguageExt};
///
/// let lang = SupportLang::Rust;
/// let tree = lang.ast_grep("fn main() {}");
/// let pattern = lang.build_pattern(&pattern_builder).unwrap();
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Hash)]
pub enum SupportLang {
    #[cfg(any(feature = "bash", feature = "all-parsers"))]
    Bash,
    #[cfg(any(feature = "c", feature = "all-parsers"))]
    C,
    #[cfg(any(feature = "cpp", feature = "all-parsers"))]
    Cpp,
    #[cfg(any(feature = "csharp", feature = "all-parsers"))]
    CSharp,
    #[cfg(any(
        feature = "css",
        feature = "all-parsers",
        feature = "css-napi",
        feature = "napi-compatible"
    ))]
    Css,
    #[cfg(any(feature = "go", feature = "all-parsers"))]
    Go,
    #[cfg(any(feature = "elixir", feature = "all-parsers"))]
    Elixir,
    #[cfg(feature = "haskell")]
    Haskell,
    #[cfg(any(
        feature = "html",
        feature = "all-parsers",
        feature = "html-napi",
        feature = "napi-compatible"
    ))]
    Html,
    #[cfg(any(feature = "java", feature = "all-parsers"))]
    Java,
    #[cfg(any(
        feature = "javascript",
        feature = "all-parsers",
        feature = "javascript-napi",
        feature = "napi-compatible"
    ))]
    JavaScript,
    #[cfg(any(feature = "json", feature = "all-parsers"))]
    Json,
    #[cfg(any(feature = "kotlin", feature = "all-parsers"))]
    Kotlin,
    #[cfg(any(feature = "lua", feature = "all-parsers"))]
    Lua,
    #[cfg(any(feature = "php", feature = "all-parsers"))]
    Php,
    #[cfg(any(feature = "python", feature = "all-parsers"))]
    Python,
    #[cfg(any(feature = "ruby", feature = "all-parsers"))]
    Ruby,
    #[cfg(any(feature = "rust", feature = "all-parsers"))]
    Rust,
    #[cfg(any(feature = "scala", feature = "all-parsers"))]
    Scala,
    #[cfg(any(feature = "swift", feature = "all-parsers"))]
    Swift,
    #[cfg(any(
        feature = "tsx",
        feature = "all-parsers",
        feature = "tsx-napi",
        feature = "napi-compatible"
    ))]
    Tsx,
    #[cfg(any(
        feature = "typescript",
        feature = "all-parsers",
        feature = "typescript-napi",
        feature = "napi-compatible"
    ))]
    TypeScript,
    #[cfg(any(feature = "yaml", feature = "all-parsers"))]
    Yaml,
    #[cfg(not(any(
        feature = "all-parsers",
        feature = "napi-compatible",
        feature = "css-napi",
        feature = "html-napi",
        feature = "javascript-napi",
        feature = "typescript-napi",
        feature = "tsx-napi",
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
    )))]
    NoEnabledLangs,
}

impl SupportLang {
    pub const fn all_langs() -> &'static [SupportLang] {
        use SupportLang::*;
        &[
            #[cfg(any(feature = "bash", feature = "all-parsers"))]
            Bash,
            #[cfg(any(feature = "c", feature = "all-parsers"))]
            C,
            #[cfg(any(feature = "cpp", feature = "all-parsers"))]
            Cpp,
            #[cfg(any(feature = "csharp", feature = "all-parsers"))]
            CSharp,
            #[cfg(any(
                feature = "css",
                feature = "all-parsers",
                feature = "css-napi",
                feature = "napi-compatible"
            ))]
            Css,
            #[cfg(any(feature = "elixir", feature = "all-parsers"))]
            Elixir,
            #[cfg(any(feature = "go", feature = "all-parsers"))]
            Go,
            #[cfg(feature = "haskell")]
            Haskell,
            #[cfg(any(
                feature = "html",
                feature = "all-parsers",
                feature = "html-napi",
                feature = "napi-compatible"
            ))]
            Html,
            #[cfg(any(feature = "java", feature = "all-parsers"))]
            Java,
            #[cfg(any(
                feature = "javascript",
                feature = "all-parsers",
                feature = "javascript-napi",
                feature = "napi-compatible"
            ))]
            JavaScript,
            #[cfg(any(feature = "json", feature = "all-parsers"))]
            Json,
            #[cfg(any(feature = "kotlin", feature = "all-parsers"))]
            Kotlin,
            #[cfg(any(feature = "lua", feature = "all-parsers"))]
            Lua,
            #[cfg(any(feature = "php", feature = "all-parsers"))]
            Php,
            #[cfg(any(feature = "python", feature = "all-parsers"))]
            Python,
            #[cfg(any(feature = "ruby", feature = "all-parsers"))]
            Ruby,
            #[cfg(any(feature = "rust", feature = "all-parsers"))]
            Rust,
            #[cfg(any(feature = "scala", feature = "all-parsers"))]
            Scala,
            #[cfg(any(feature = "swift", feature = "all-parsers"))]
            Swift,
            #[cfg(any(
                feature = "tsx",
                feature = "all-parsers",
                feature = "tsx-napi",
                feature = "napi-compatible"
            ))]
            Tsx,
            #[cfg(any(
                feature = "typescript",
                feature = "all-parsers",
                feature = "typescript-napi",
                feature = "napi-compatible"
            ))]
            TypeScript,
            #[cfg(any(feature = "yaml", feature = "all-parsers"))]
            Yaml,
            #[cfg(not(any(
                feature = "all-parsers",
                feature = "napi-compatible",
                feature = "css-napi",
                feature = "html-napi",
                feature = "javascript-napi",
                feature = "typescript-napi",
                feature = "tsx-napi",
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
            )))]
            NoEnabledLangs,
        ]
    }

    pub fn file_types(&self) -> Types {
        file_types(*self)
    }
}

impl fmt::Display for SupportLang {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug)]
pub enum SupportLangErr {
    LanguageNotSupported(String),
    LanguageNotEnabled(String),
}

impl Display for SupportLangErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        use SupportLangErr::*;
        match self {
            LanguageNotSupported(lang) => write!(f, "{lang} is not supported!"),
            LanguageNotEnabled(lang) => write!(
                f,
                "{lang} is available but not enabled. You need to enable the feature flag for this language."
            ),
        }
    }
}

impl std::error::Error for SupportLangErr {}

#[cfg(any(
    feature = "all-parsers",
    feature = "napi-compatible",
    feature = "css-napi",
    feature = "html-napi",
    feature = "javascript-napi",
    feature = "typescript-napi",
    feature = "tsx-napi",
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
))]
impl<'de> Deserialize<'de> for SupportLang {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(SupportLangVisitor)
    }
}

struct SupportLangVisitor;

#[cfg(any(
    feature = "all-parsers",
    feature = "napi-compatible",
    feature = "css-napi",
    feature = "html-napi",
    feature = "javascript-napi",
    feature = "typescript-napi",
    feature = "tsx-napi",
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
))]
impl Visitor<'_> for SupportLangVisitor {
    type Value = SupportLang;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("SupportLang")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        v.parse().map_err(de::Error::custom)
    }
}

struct AliasVisitor {
    aliases: &'static [&'static str],
}
#[cfg(any(
    feature = "all-parsers",
    feature = "napi-compatible",
    feature = "css-napi",
    feature = "html-napi",
    feature = "javascript-napi",
    feature = "typescript-napi",
    feature = "tsx-napi",
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
))]
impl Visitor<'_> for AliasVisitor {
    type Value = &'static str;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "one of {:?}", self.aliases)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.aliases
            .iter()
            .copied()
            .find(|&a| v.eq_ignore_ascii_case(a))
            .ok_or_else(|| de::Error::invalid_value(de::Unexpected::Str(v), &self))
    }
}
#[cfg(any(
    feature = "all-parsers",
    feature = "napi-compatible",
    feature = "css-napi",
    feature = "html-napi",
    feature = "javascript-napi",
    feature = "typescript-napi",
    feature = "tsx-napi",
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
))]
impl_aliases! {
  Bash, "bash" => &["bash"],
  C, "c" => &["c"],
  Cpp, "cpp" => &["cc", "c++", "cpp", "cxx"],
  CSharp, "csharp" => &["cs", "csharp"],
  Css, "css" => &["css"],
  Elixir, "elixir" => &["ex", "elixir"],
  Go, "go" => &["go", "golang"],
  Haskell, "haskell" => &["hs", "haskell"],
  Html, "html" => &["html"],
  Java, "java" => &["java"],
  JavaScript, "javascript" => &["javascript", "js", "jsx"],
  Json, "json" => &["json"],
  Kotlin, "kotlin" => &["kotlin", "kt"],
  Lua, "lua" => &["lua"],
  Php, "php" => &["php"],
  Python, "python" => &["py", "python"],
  Ruby, "ruby" => &["rb", "ruby"],
  Rust, "rust" => &["rs", "rust"],
  Scala, "scala" => &["scala"],
  Swift, "swift" => &["swift"],
  TypeScript, "typescript" => &["ts", "typescript"],
  Tsx, "tsx" => &["tsx"],
  Yaml, "yaml" => &["yaml", "yml"],
  NoEnabledLangs, "no-enabled-langs" => &["no-enabled-langs"],
}

/// Implements the language names and aliases.
impl FromStr for SupportLang {
    type Err = SupportLangErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut str_matcher = s.trim().to_string();
        str_matcher.make_ascii_lowercase();
        match str_matcher.as_str() {
            #[cfg(any(feature = "bash", feature = "all-parsers"))]
            "bash" => Ok(SupportLang::Bash),
            #[cfg(any(feature = "c", feature = "all-parsers"))]
            "c" => Ok(SupportLang::C),
            #[cfg(any(feature = "cpp", feature = "all-parsers"))]
            "cpp" | "c++" => Ok(SupportLang::Cpp),
            #[cfg(any(feature = "csharp", feature = "all-parsers"))]
            "cs" | "csharp" => Ok(SupportLang::CSharp),
            #[cfg(any(
                feature = "css",
                feature = "all-parsers",
                feature = "css-napi",
                feature = "napi-compatible"
            ))]
            "css" => Ok(SupportLang::Css),
            #[cfg(any(feature = "elixir", feature = "all-parsers"))]
            "elixir" | "ex" => Ok(SupportLang::Elixir),
            #[cfg(any(feature = "go", feature = "all-parsers"))]
            "go" | "golang" => Ok(SupportLang::Go),
            #[cfg(feature = "haskell")]
            "haskell" | "hs" => Ok(SupportLang::Haskell),
            #[cfg(any(
                feature = "html",
                feature = "all-parsers",
                feature = "html-napi",
                feature = "napi-compatible"
            ))]
            "html" => Ok(SupportLang::Html),
            #[cfg(any(feature = "java", feature = "all-parsers"))]
            "java" => Ok(SupportLang::Java),
            #[cfg(any(
                feature = "javascript",
                feature = "all-parsers",
                feature = "javascript-napi",
                feature = "napi-compatible"
            ))]
            "javascript" | "js" => Ok(SupportLang::JavaScript),
            #[cfg(any(feature = "json", feature = "all-parsers"))]
            "json" => Ok(SupportLang::Json),
            #[cfg(any(feature = "kotlin", feature = "all-parsers"))]
            "kotlin" | "kt" => Ok(SupportLang::Kotlin),
            #[cfg(any(feature = "lua", feature = "all-parsers"))]
            "lua" => Ok(SupportLang::Lua),
            #[cfg(any(feature = "php", feature = "all-parsers"))]
            "php" => Ok(SupportLang::Php),
            #[cfg(any(feature = "python", feature = "all-parsers"))]
            "python" | "py" => Ok(SupportLang::Python),
            #[cfg(any(feature = "ruby", feature = "all-parsers"))]
            "ruby" | "rb" => Ok(SupportLang::Ruby),
            #[cfg(any(feature = "rust", feature = "all-parsers"))]
            "rust" | "rs" => Ok(SupportLang::Rust),
            #[cfg(any(feature = "scala", feature = "all-parsers"))]
            "scala" => Ok(SupportLang::Scala),
            #[cfg(any(feature = "swift", feature = "all-parsers"))]
            "swift" => Ok(SupportLang::Swift),
            #[cfg(any(
                feature = "typescript",
                feature = "all-parsers",
                feature = "typescript-napi",
                feature = "napi-compatible"
            ))]
            "typescript" | "ts" => Ok(SupportLang::TypeScript),
            #[cfg(any(
                feature = "tsx",
                feature = "all-parsers",
                feature = "tsx-napi",
                feature = "napi-compatible"
            ))]
            "tsx" => Ok(SupportLang::Tsx),
            #[cfg(any(feature = "yaml", feature = "all-parsers"))]
            "yaml" | "yml" => Ok(SupportLang::Yaml),
            #[cfg(not(any(
                feature = "all-parsers",
                feature = "napi-compatible",
                feature = "css-napi",
                feature = "html-napi",
                feature = "javascript-napi",
                feature = "typescript-napi",
                feature = "tsx-napi",
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
            )))]
            "no-enabled-langs" => Ok(SupportLang::NoEnabledLangs),

            _ => {
                if constants::ALL_SUPPORTED_LANGS.contains(&str_matcher.as_str()) {
                    Err(SupportLangErr::LanguageNotEnabled(format!(
                        "language {} was detected, but it is not enabled by feature flags. If you want to parse this kind of file, enable the flag in `thread-language`",
                        &str_matcher
                    )))
                } else {
                    Err(SupportLangErr::LanguageNotSupported(format!(
                        "language {} is not supported",
                        &str_matcher
                    )))
                }
            }
        }
    }
}
#[cfg(any(
    feature = "all-parsers",
    feature = "napi-compatible",
    feature = "css-napi",
    feature = "html-napi",
    feature = "javascript-napi",
    feature = "typescript-napi",
    feature = "tsx-napi",
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
))]
macro_rules! execute_lang_method {
  ($me: path, $method: ident, $($pname:tt),*) => {
    use SupportLang as S;
    match $me {
        #[cfg(any(feature = "bash", feature = "all-parsers"))]
        S::Bash => Bash.$method($($pname,)*),
        #[cfg(any(feature = "c", feature = "all-parsers"))]
        S::C => C.$method($($pname,)*),
        #[cfg(any(feature = "cpp", feature = "all-parsers"))]
        S::Cpp => Cpp.$method($($pname,)*),
        #[cfg(any(feature = "csharp", feature = "all-parsers"))]
        S::CSharp => CSharp.$method($($pname,)*),
        #[cfg(any(feature = "css", feature = "all-parsers", feature = "css-napi", feature = "napi-compatible"))]
        S::Css => Css.$method($($pname,)*),
        #[cfg(any(feature = "elixir", feature = "all-parsers"))]
        S::Elixir => Elixir.$method($($pname,)*),
        #[cfg(any(feature = "go", feature = "all-parsers"))]
        S::Go => Go.$method($($pname,)*),
        #[cfg(feature = "haskell")]
        S::Haskell => Haskell.$method($($pname,)*),
        #[cfg(any(feature = "html", feature = "all-parsers", feature = "html-napi", feature = "napi-compatible"))]
        S::Html => Html.$method($($pname,)*),
        #[cfg(any(feature = "json", feature = "all-parsers"))]
        S::Java => Java.$method($($pname,)*),
        #[cfg(any(feature = "javascript", feature = "all-parsers", feature = "javascript-napi", feature = "napi-compatible"))]
        S::JavaScript => JavaScript.$method($($pname,)*),
        #[cfg(any(feature = "json", feature = "all-parsers"))]
        S::Json => Json.$method($($pname,)*),
        #[cfg(any(feature = "kotlin", feature = "all-parsers"))]
        S::Kotlin => Kotlin.$method($($pname,)*),
        #[cfg(any(feature = "lua", feature = "all-parsers"))]
        S::Lua => Lua.$method($($pname,)*),
        #[cfg(any(feature = "php", feature = "all-parsers"))]
        S::Php => Php.$method($($pname,)*),
        #[cfg(any(feature = "python", feature = "all-parsers"))]
        S::Python => Python.$method($($pname,)*),
        #[cfg(any(feature = "ruby", feature = "all-parsers"))]
        S::Ruby => Ruby.$method($($pname,)*),
        #[cfg(any(feature = "rust", feature = "all-parsers"))]
        S::Rust => Rust.$method($($pname,)*),
        #[cfg(any(feature = "scala", feature = "all-parsers"))]
        S::Scala => Scala.$method($($pname,)*),
        #[cfg(any(feature = "swift", feature = "all-parsers"))]
        S::Swift => Swift.$method($($pname,)*),
        #[cfg(any(feature = "tsx", feature = "all-parsers", feature = "tsx-napi", feature = "napi-compatible"))]
        S::Tsx => Tsx.$method($($pname,)*),
        #[cfg(any(feature = "typescript", feature = "all-parsers", feature = "typescript-napi", feature = "napi-compatible"))]
        S::TypeScript => TypeScript.$method($($pname,)*),
        #[cfg(any(feature = "yaml", feature = "all-parsers"))]
        S::Yaml => Yaml.$method($($pname,)*),
        #[cfg(not(any(
            feature = "all-parsers",
            feature = "napi-compatible",
            feature = "css-napi",
            feature = "html-napi",
            feature = "javascript-napi",
            feature = "typescript-napi",
            feature = "tsx-napi",
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
        )))]
        S::NoEnabledLangs => {
            return Err(SupportLangErr::LanguageNotEnabled(
                "no-enabled-langs".to_string(),
            ))
        }
    }
  }
}
#[cfg(any(
    feature = "all-parsers",
    feature = "napi-compatible",
    feature = "css-napi",
    feature = "javascript-napi",
    feature = "html-napi",
    feature = "typescript-napi",
    feature = "tsx-napi",
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
))]
macro_rules! impl_lang_method {
  ($method: ident, ($($pname:tt: $ptype:ty),*) => $return_type: ty) => {
    #[inline]
    fn $method(&self, $($pname: $ptype),*) -> $return_type {
      execute_lang_method!{ self, $method, $($pname),* }
    }
  };
}
#[cfg(all(
    feature = "matching",
    any(
        feature = "all-parsers",
        feature = "napi-environment",
        feature = "napi-compatible",
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
    )
))]
impl Language for SupportLang {
    impl_lang_method!(kind_to_id, (kind: &str) => u16);
    impl_lang_method!(field_to_id, (field: &str) => Option<u16>);
    impl_lang_method!(meta_var_char, () => char);
    impl_lang_method!(expando_char, () => char);
    impl_lang_method!(extract_meta_var, (source: &str) => Option<MetaVariable>);
    impl_lang_method!(build_pattern, (builder: &PatternBuilder) => Result<Pattern, PatternError>);
    fn pre_process_pattern<'q>(&self, query: &'q str) -> Cow<'q, str> {
        execute_lang_method! { self, pre_process_pattern, query }
    }
    fn from_path<P: AsRef<Path>>(path: P) -> Option<Self> {
        from_extension(path.as_ref())
    }
}

#[cfg(all(
    feature = "matching",
    any(
        feature = "all-parsers",
        feature = "napi-compatible",
        feature = "css-napi",
        feature = "html-napi",
        feature = "javascript-napi",
        feature = "typescript-napi",
        feature = "tsx-napi",
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
    )
))]
impl LanguageExt for SupportLang {
    impl_lang_method!(get_ts_language, () => TSLanguage);
    impl_lang_method!(injectable_languages, () => Option<&'static [&'static str]>);
    fn extract_injections<L: LanguageExt>(
        &self,
        root: Node<StrDoc<L>>,
    ) -> RapidMap<String, Vec<TSRange>> {
        match self {
            #[cfg(feature = "html-embedded")]
            SupportLang::Html => Html.extract_injections(root),
            _ => RapidMap::default(),
        }
    }
}

pub const fn extensions(lang: SupportLang) -> &'static [&'static str] {
    use SupportLang::*;
    match lang {
        #[cfg(any(feature = "bash", feature = "all-parsers"))]
        Bash => &constants::BASH_EXTS,
        #[cfg(any(feature = "c", feature = "all-parsers"))]
        C => &constants::C_EXTS,
        #[cfg(any(feature = "cpp", feature = "all-parsers"))]
        Cpp => &constants::CPP_EXTS,
        #[cfg(any(feature = "csharp", feature = "all-parsers"))]
        CSharp => &constants::CSHARP_EXTS,
        #[cfg(any(
            feature = "css",
            feature = "all-parsers",
            feature = "css-napi",
            feature = "napi-compatible"
        ))]
        Css => &constants::CSS_EXTS,
        #[cfg(any(feature = "elixir", feature = "all-parsers"))]
        Elixir => &constants::ELIXIR_EXTS,
        #[cfg(any(feature = "go", feature = "all-parsers"))]
        Go => &constants::GO_EXTS,
        #[cfg(feature = "haskell")]
        Haskell => &constants::HASKELL_EXTS,
        #[cfg(any(
            feature = "html",
            feature = "all-parsers",
            feature = "html-napi",
            feature = "napi-compatible"
        ))]
        Html => &constants::HTML_EXTS,
        #[cfg(any(feature = "java", feature = "all-parsers"))]
        Java => &constants::JAVA_EXTS,
        #[cfg(any(
            feature = "javascript",
            feature = "all-parsers",
            feature = "javascript-napi",
            feature = "napi-compatible"
        ))]
        JavaScript => &constants::JAVASCRIPT_EXTS,
        #[cfg(any(feature = "json", feature = "all-parsers"))]
        Json => &constants::JSON_EXTS,
        #[cfg(any(feature = "kotlin", feature = "all-parsers"))]
        Kotlin => &constants::KOTLIN_EXTS,
        #[cfg(any(feature = "lua", feature = "all-parsers"))]
        Lua => &constants::LUA_EXTS,
        #[cfg(any(feature = "php", feature = "all-parsers"))]
        Php => &constants::PHP_EXTS,
        #[cfg(any(feature = "python", feature = "all-parsers"))]
        Python => &constants::PYTHON_EXTS,
        #[cfg(any(feature = "ruby", feature = "all-parsers"))]
        Ruby => &constants::RUBY_EXTS,
        #[cfg(any(feature = "rust", feature = "all-parsers"))]
        Rust => &constants::RUST_EXTS,
        #[cfg(any(feature = "scala", feature = "all-parsers"))]
        Scala => &constants::SCALA_EXTS,
        #[cfg(any(feature = "swift", feature = "all-parsers"))]
        Swift => &constants::SWIFT_EXTS,
        #[cfg(any(
            feature = "typescript",
            feature = "all-parsers",
            feature = "typescript-napi",
            feature = "napi-compatible"
        ))]
        TypeScript => &constants::TYPESCRIPT_EXTS,
        #[cfg(any(
            feature = "tsx",
            feature = "all-parsers",
            feature = "tsx-napi",
            feature = "napi-compatible"
        ))]
        Tsx => &constants::TSX_EXTS,
        #[cfg(any(feature = "yaml", feature = "all-parsers"))]
        Yaml => &constants::YAML_EXTS,
        #[cfg(not(any(
            feature = "all-parsers",
            feature = "napi-environment",
            feature = "napi-compatible",
            feature = "css-napi",
            feature = "html-napi",
            feature = "javascript-napi",
            feature = "typescript-napi",
            feature = "tsx-napi",
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
        )))]
        NoEnabledLangs => &[],
    }
}

/// Guess which programming language a file is written in
/// Adapt from `<https://github.com/Wilfred/difftastic/blob/master/src/parse/guess_language.rs>`
/// N.B do not confuse it with `FromStr` trait. This function is to guess language from file extension.
///
/// We check against the most common file types and extensions first.
/// These are hardcoded matches
#[inline]
pub fn from_extension(path: &Path) -> Option<SupportLang> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    from_extension_str(&ext)
}

#[inline]
pub fn from_extension_str(ext: &str) -> Option<SupportLang> {
    let ext = ext.to_ascii_lowercase();
    // TODO: Add shebang check if no ext
    if ext.is_empty() {
        return None;
    }
    match ext.as_str() {
        #[cfg(any(feature = "python", feature = "all-parsers"))]
        "py" => Some(SupportLang::Python),
        #[cfg(any(
            feature = "javascript",
            feature = "all-parsers",
            feature = "javascript-napi",
            feature = "napi-compatible"
        ))]
        "js" => Some(SupportLang::JavaScript),
        #[cfg(any(
            feature = "typescript",
            feature = "all-parsers",
            feature = "typescript-napi",
            feature = "napi-compatible"
        ))]
        "ts" => Some(SupportLang::TypeScript),
        #[cfg(any(feature = "java", feature = "all-parsers"))]
        "java" => Some(SupportLang::Java),
        #[cfg(any(feature = "go", feature = "all-parsers"))]
        "go" => Some(SupportLang::Go),
        #[cfg(any(feature = "cpp", feature = "all-parsers"))]
        "cpp" => Some(SupportLang::Cpp),
        #[cfg(any(feature = "rust", feature = "all-parsers"))]
        "rs" => Some(SupportLang::Rust),
        #[cfg(any(feature = "c", feature = "all-parsers"))]
        "c" => Some(SupportLang::C),
        // json and yaml are the most common config formats
        #[cfg(any(feature = "json", feature = "all-parsers"))]
        "json" => Some(SupportLang::Json),
        #[cfg(any(feature = "yaml", feature = "all-parsers"))]
        "yaml" | "yml" => Some(SupportLang::Yaml),
        _ => ext_iden::match_by_aho_corasick(&ext),
    }
}

fn add_custom_file_type<'b>(
    builder: &'b mut TypesBuilder,
    file_type: &str,
    suffix_list: &[&str],
) -> &'b mut TypesBuilder {
    for suffix in suffix_list {
        let glob = format!("*.{suffix}");
        builder
            .add(file_type, &glob)
            .expect("file pattern must compile");
    }
    builder.select(file_type)
}

fn file_types(lang: SupportLang) -> Types {
    let mut builder = TypesBuilder::new();
    let exts = extensions(lang);
    let lang_name = lang.to_string();
    add_custom_file_type(&mut builder, &lang_name, exts);
    builder.build().expect("file type must be valid")
}

pub fn config_file_type() -> Types {
    let mut builder = TypesBuilder::new();
    let builder = add_custom_file_type(&mut builder, "yml", &["yml", "yaml"]);
    builder.build().expect("yaml type must be valid")
}

#[cfg(test)]
mod test {
    use super::*;
    use thread_ast_engine::{Pattern, matcher::MatcherExt};

    pub fn test_match_lang(query: &str, source: &str, lang: impl LanguageExt) {
        let cand = lang.ast_grep(source);
        let pattern = Pattern::new(query, &lang);
        assert!(
            pattern.find_node(cand.root()).is_some(),
            "goal: {pattern:?}, candidate: {}",
            cand.root().get_inner_node().to_sexp(),
        );
    }

    pub fn test_non_match_lang(query: &str, source: &str, lang: impl LanguageExt) {
        let cand = lang.ast_grep(source);
        let pattern = Pattern::new(query, &lang);
        assert!(
            pattern.find_node(cand.root()).is_none(),
            "goal: {pattern:?}, candidate: {}",
            cand.root().get_inner_node().to_sexp(),
        );
    }

    pub fn test_replace_lang(
        src: &str,
        pattern: &str,
        replacer: &str,
        lang: impl LanguageExt,
    ) -> String {
        let mut source = lang.ast_grep(src);
        assert!(
            source
                .replace(pattern, replacer)
                .expect("should parse successfully")
        );
        source.generate()
    }

    #[test]
    fn test_js_string() {
        test_match_lang("'a'", "'a'", JavaScript);
        test_match_lang("\"\"", "\"\"", JavaScript);
        test_match_lang("''", "''", JavaScript);
    }

    #[test]
    fn test_guess_by_extension() {
        let path = Path::new("foo.rs");
        assert_eq!(from_extension(path), Some(SupportLang::Rust));
    }

    #[test]
    fn test_optimized_extension_matching() {
        // Test that the optimized implementation produces the same results as the original
        let test_cases = [
            ("main.rs", Some(SupportLang::Rust)),
            ("app.js", Some(SupportLang::JavaScript)),
            ("index.html", Some(SupportLang::Html)),
            ("data.json", Some(SupportLang::Json)),
            ("script.py", Some(SupportLang::Python)),
            ("main.go", Some(SupportLang::Go)),
            ("style.css", Some(SupportLang::Css)),
            ("component.tsx", Some(SupportLang::Tsx)),
            ("build.gradle.kts", Some(SupportLang::Kotlin)),
            ("config.yml", Some(SupportLang::Yaml)),
            ("script.sh", Some(SupportLang::Bash)),
            ("app.swift", Some(SupportLang::Swift)),
            ("main.cpp", Some(SupportLang::Cpp)),
            ("header.hpp", Some(SupportLang::Cpp)),
            ("style.scss", Some(SupportLang::Css)),
            ("script.rb", Some(SupportLang::Ruby)),
            ("main.scala", Some(SupportLang::Scala)),
            ("app.kt", Some(SupportLang::Kotlin)),
            // Case insensitive tests
            ("Main.RS", Some(SupportLang::Rust)),
            ("App.JS", Some(SupportLang::JavaScript)),
            ("Config.YML", Some(SupportLang::Yaml)),
            // Non-existent extensions
            ("file.xyz", None),
            ("test.unknown", None),
        ];

        for (filename, expected) in test_cases {
            let path = Path::new(filename);
            let result = from_extension(path);
            assert_eq!(result, expected, "Failed for {}", filename);

            // Also test the direct extension matching
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let direct_result = ext_iden::match_by_aho_corasick(ext);
                assert_eq!(
                    direct_result, expected,
                    "Direct matching failed for {}",
                    ext
                );
            }
        }
    }

    // TODO: add test for file_types
}
