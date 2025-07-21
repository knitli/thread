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
pub mod parsers;

#[cfg(feature = "bash")]
mod bash;
#[cfg(feature = "cpp")]
mod cpp;
#[cfg(feature = "csharp")]
mod csharp;
#[cfg(feature = "css")]
mod css;
#[cfg(feature = "elixir")]
mod elixir;
#[cfg(feature = "go")]
mod go;
#[cfg(feature = "haskell")]
mod haskell;
#[cfg(feature = "html")]
mod html;
#[cfg(feature = "json")]
mod json;
#[cfg(feature = "kotlin")]
mod kotlin;
#[cfg(feature = "lua")]
mod lua;
#[cfg(feature = "php")]
mod php;
#[cfg(feature = "python")]
mod python;
#[cfg(feature = "ruby")]
mod ruby;
#[cfg(feature = "rust")]
mod rust;
#[cfg(feature = "scala")]
mod scala;
#[cfg(feature = "swift")]
mod swift;
#[cfg(feature = "yaml")]
mod yaml;
#[cfg(feature = "html")]
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
use thread_ast_engine::tree_sitter::TSLanguage;
#[cfg(feature = "matching")]
use thread_ast_engine::tree_sitter::{StrDoc, TSRange};
#[cfg(feature = "matching")]
use thread_utils::RapidMap;

pub use thread_ast_engine::language::Language;
pub use thread_ast_engine::tree_sitter::LanguageExt;

const BASH_EXTENSION_PATTERN: [&str; 19] = [
    "bash", "bats", "sh", ".bashrc", "bash_aliases", "bats", "cgi", "command", "env",
    "fcgi", "ksh", "tmux", "tool", "zsh", "bash_logout", "bash_profile", "profile",
    "login", "logout"
];

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
#[allow(unused_macros)]
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
macro_rules! impl_aliases {
  ($($lang:ident, $feature:literal => $as:expr),* $(,)?) => {
    $(#[cfg(feature = $feature)]
      impl_alias!($lang => $as);
    )*
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
#[cfg(feature = "c")]
impl_lang_expando!(C, language_c, 'µ');
#[cfg(feature = "cpp")]
impl_lang_expando!(Cpp, language_cpp, 'µ');

// https://docs.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/lexical-structure#643-identifiers
// all letter number is accepted
// https://www.compart.com/en/unicode/category/Nl
#[cfg(feature = "csharp")]
impl_lang_expando!(CSharp, language_c_sharp, 'µ');

// https://www.w3.org/TR/CSS21/grammar.html#scanner
#[cfg(feature = "css")]
impl_lang_expando!(Css, language_css, '_');

// https://github.com/elixir-lang/tree-sitter-elixir/blob/a2861e88a730287a60c11ea9299c033c7d076e30/grammar.js#L245
#[cfg(feature = "elixir")]
impl_lang_expando!(Elixir, language_elixir, 'µ');

// we can use any Unicode code point categorized as "Letter"
// https://go.dev/ref/spec#letter
#[cfg(feature = "go")]
impl_lang_expando!(Go, language_go, 'µ');

// GHC supports Unicode syntax per
// https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/unicode_syntax.html
// and the tree-sitter-haskell grammar parses it too.
#[cfg(feature = "haskell")]
impl_lang_expando!(Haskell, language_haskell, 'µ');

// https://github.com/fwcd/tree-sitter-kotlin/pull/93
#[cfg(feature = "kotlin")]
impl_lang_expando!(Kotlin, language_kotlin, 'µ');

// PHP accepts unicode to be used as some name not var name though
#[cfg(feature = "php")]
impl_lang_expando!(Php, language_php, 'µ');

// we can use any char in unicode range [:XID_Start:]
// https://docs.python.org/3/reference/lexical_analysis.html#identifiers
// see also [PEP 3131](https://peps.python.org/pep-3131/) for further details.
#[cfg(feature = "python")]
impl_lang_expando!(Python, language_python, 'µ');

// https://github.com/tree-sitter/tree-sitter-ruby/blob/f257f3f57833d584050336921773738a3fd8ca22/grammar.js#L30C26-L30C78
#[cfg(feature = "ruby")]
impl_lang_expando!(Ruby, language_ruby, 'µ');

// we can use any char in unicode range [:XID_Start:]
// https://doc.rust-lang.org/reference/identifiers.html
#[cfg(feature = "rust")]
impl_lang_expando!(Rust, language_rust, 'µ');

//https://docs.swift.org/swift-book/documentation/the-swift-programming-language/lexicalstructure/#Identifiers
#[cfg(feature = "swift")]
impl_lang_expando!(Swift, language_swift, 'µ');

// Stub Language without preprocessing
// Language Name, tree-sitter-name, alias, extension
#[cfg(feature = "bash")]
impl_lang!(Bash, language_bash);
#[cfg(feature = "java")]
impl_lang!(Java, language_java);
#[cfg(feature = "javascript")]
impl_lang!(JavaScript, language_javascript);
#[cfg(feature = "json")]
impl_lang!(Json, language_json);
#[cfg(feature = "lua")]
impl_lang!(Lua, language_lua);
#[cfg(feature = "scala")]
impl_lang!(Scala, language_scala);
#[cfg(feature = "tsx")]
impl_lang!(Tsx, language_tsx);
#[cfg(feature = "typescript")]
impl_lang!(TypeScript, language_typescript);
#[cfg(feature = "yaml")]
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
    #[cfg(feature = "bash")]
    Bash,
    #[cfg(feature = "c")]
    C,
    #[cfg(feature = "cpp")]
    Cpp,
    #[cfg(feature = "csharp")]
    CSharp,
    #[cfg(feature = "css")]
    Css,
    #[cfg(feature = "go")]
    Go,
    #[cfg(feature = "elixir")]
    Elixir,
    #[cfg(feature = "haskell")]
    Haskell,
    #[cfg(feature = "html")]
    Html,
    #[cfg(feature = "java")]
    Java,
    #[cfg(feature = "javascript")]
    JavaScript,
    #[cfg(feature = "json")]
    Json,
    #[cfg(feature = "kotlin")]
    Kotlin,
    #[cfg(feature = "lua")]
    Lua,
    #[cfg(feature = "php")]
    Php,
    #[cfg(feature = "python")]
    Python,
    #[cfg(feature = "ruby")]
    Ruby,
    #[cfg(feature = "rust")]
    Rust,
    #[cfg(feature = "scala")]
    Scala,
    #[cfg(feature = "swift")]
    Swift,
    #[cfg(feature = "tsx")]
    Tsx,
    #[cfg(feature = "typescript")]
    TypeScript,
    #[cfg(feature = "yaml")]
    Yaml,
}

impl SupportLang {
    pub const fn all_langs() -> &'static [SupportLang] {
        use SupportLang::*;
        &[
            #[cfg(feature = "bash")]
            Bash,
            #[cfg(feature = "c")]
            C,
            #[cfg(feature = "cpp")]
            Cpp,
            #[cfg(feature = "csharp")]
            CSharp,
            #[cfg(feature = "css")]
            Css,
            #[cfg(feature = "elixir")]
            Elixir,
            #[cfg(feature = "go")]
            Go,
            #[cfg(feature = "haskell")]
            Haskell,
            #[cfg(feature = "html")]
            Html,
            #[cfg(feature = "java")]
            Java,
            #[cfg(feature = "javascript")]
            JavaScript,
            #[cfg(feature = "json")]
            Json,
            #[cfg(feature = "kotlin")]
            Kotlin,
            #[cfg(feature = "lua")]
            Lua,
            #[cfg(feature = "php")]
            Php,
            #[cfg(feature = "python")]
            Python,
            #[cfg(feature = "ruby")]
            Ruby,
            #[cfg(feature = "rust")]
            Rust,
            #[cfg(feature = "scala")]
            Scala,
            #[cfg(feature = "swift")]
            Swift,
            #[cfg(feature = "tsx")]
            Tsx,
            #[cfg(feature = "typescript")]
            TypeScript,
            #[cfg(feature = "yaml")]
            Yaml,
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
    LanguageNotEnabled(String)
}

impl Display for SupportLangErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        use SupportLangErr::*;
        match self {
            LanguageNotSupported(lang) => write!(f, "{lang} is not supported!"),
            LanguageNotEnabled(lang) => write!(f, "{lang} is available but not enabled. You need to enable the feature flag for this language.")
        }
    }
}

impl std::error::Error for SupportLangErr {}

impl<'de> Deserialize<'de> for SupportLang {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(SupportLangVisitor)
    }
}

struct SupportLangVisitor;

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
}

/// Implements the language names and aliases.
impl FromStr for SupportLang {
    type Err = SupportLangErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Fast path: try exact matches first (most common case)
        match s {
            #[cfg(feature = "bash")]
            "bash" => return Ok(SupportLang::Bash),
            #[cfg(feature = "c")]
            "c" => return Ok(SupportLang::C),
            #[cfg(feature = "cpp")]
            "cpp" | "c++" => return Ok(SupportLang::Cpp),
            #[cfg(feature = "csharp")]
            "cs" | "csharp" => return Ok(SupportLang::CSharp),
            #[cfg(feature = "css")]
            "css" => return Ok(SupportLang::Css),
            #[cfg(feature = "elixir")]
            "elixir" | "ex" => return Ok(SupportLang::Elixir),
            #[cfg(feature = "go")]
            "go" | "golang" => return Ok(SupportLang::Go),
            #[cfg(feature = "haskell")]
            "haskell" | "hs" => return Ok(SupportLang::Haskell),
            #[cfg(feature = "html")]
            "html" => return Ok(SupportLang::Html),
            #[cfg(feature = "java")]
            "java" => return Ok(SupportLang::Java),
            #[cfg(feature = "javascript")]
            "javascript" | "js" => return Ok(SupportLang::JavaScript),
            #[cfg(feature = "json")]
            "json" => return Ok(SupportLang::Json),
            #[cfg(feature = "kotlin")]
            "kotlin" | "kt" => return Ok(SupportLang::Kotlin),
            #[cfg(feature = "lua")]
            "lua" => return Ok(SupportLang::Lua),
            #[cfg(feature = "php")]
            "php" => return Ok(SupportLang::Php),
            #[cfg(feature = "python")]
            "python" | "py" => return Ok(SupportLang::Python),
            #[cfg(feature = "ruby")]
            "ruby" | "rb" => return Ok(SupportLang::Ruby),
            #[cfg(feature = "rust")]
            "rust" | "rs" => return Ok(SupportLang::Rust),
            #[cfg(feature = "scala")]
            "scala" => return Ok(SupportLang::Scala),
            #[cfg(feature = "swift")]
            "swift" => return Ok(SupportLang::Swift),
            #[cfg(feature = "typescript")]
            "typescript" | "ts" => return Ok(SupportLang::TypeScript),
            #[cfg(feature = "tsx")]
            "tsx" => return Ok(SupportLang::Tsx),
            #[cfg(feature = "yaml")]
            "yaml" | "yml" => return Ok(SupportLang::Yaml),
            _ => {} // Fall through to case-insensitive search
        }

        // Slow path: case-insensitive search for less common aliases
        for &lang in Self::all_langs() {
            for moniker in alias(lang) {
                if s.eq_ignore_ascii_case(moniker) {
                    return Ok(lang);
                }
            }
        }
        Err(SupportLangErr::LanguageNotSupported(s.to_string()))
    }
}

macro_rules! execute_lang_method {
  ($me: path, $method: ident, $($pname:tt),*) => {
    use SupportLang as S;
    match $me {
      #[cfg(feature = "bash")]
      S::Bash => Bash.$method($($pname,)*),
      #[cfg(feature = "c")]
      S::C => C.$method($($pname,)*),
        #[cfg(feature = "cpp")]
      S::Cpp => Cpp.$method($($pname,)*),
        #[cfg(feature = "csharp")]
      S::CSharp => CSharp.$method($($pname,)*),
        #[cfg(feature = "css")]
      S::Css => Css.$method($($pname,)*),
        #[cfg(feature = "elixir")]
      S::Elixir => Elixir.$method($($pname,)*),
        #[cfg(feature = "go")]
      S::Go => Go.$method($($pname,)*),
        #[cfg(feature = "haskell")]
      S::Haskell => Haskell.$method($($pname,)*),
        #[cfg(feature = "html")]
      S::Html => Html.$method($($pname,)*),
        #[cfg(feature = "json")]
      S::Java => Java.$method($($pname,)*),
        #[cfg(feature = "javascript")]
      S::JavaScript => JavaScript.$method($($pname,)*),
        #[cfg(feature = "json")]
      S::Json => Json.$method($($pname,)*),
        #[cfg(feature = "kotlin")]
      S::Kotlin => Kotlin.$method($($pname,)*),
        #[cfg(feature = "lua")]
      S::Lua => Lua.$method($($pname,)*),
        #[cfg(feature = "php")]
      S::Php => Php.$method($($pname,)*),
        #[cfg(feature = "python")]
      S::Python => Python.$method($($pname,)*),
        #[cfg(feature = "ruby")]
      S::Ruby => Ruby.$method($($pname,)*),
        #[cfg(feature = "rust")]
      S::Rust => Rust.$method($($pname,)*),
        #[cfg(feature = "scala")]
      S::Scala => Scala.$method($($pname,)*),
        #[cfg(feature = "swift")]
      S::Swift => Swift.$method($($pname,)*),
        #[cfg(feature = "tsx")]
      S::Tsx => Tsx.$method($($pname,)*),
        #[cfg(feature = "typescript")]
      S::TypeScript => TypeScript.$method($($pname,)*),
        #[cfg(feature = "yaml")]
      S::Yaml => Yaml.$method($($pname,)*),
    }
  }
}

macro_rules! impl_lang_method {
  ($method: ident, ($($pname:tt: $ptype:ty),*) => $return_type: ty) => {
    #[inline]
    fn $method(&self, $($pname: $ptype),*) -> $return_type {
      execute_lang_method!{ self, $method, $($pname),* }
    }
  };
}
impl Language for SupportLang {
    impl_lang_method!(kind_to_id, (kind: &str) => u16);
    impl_lang_method!(field_to_id, (field: &str) => Option<u16>);
    impl_lang_method!(meta_var_char, () => char);
    impl_lang_method!(expando_char, () => char);
    impl_lang_method!(extract_meta_var, (source: &str) => Option<MetaVariable>);
    #[cfg(feature = "matching")]
    impl_lang_method!(build_pattern, (builder: &PatternBuilder) => Result<Pattern, PatternError>);
    fn pre_process_pattern<'q>(&self, query: &'q str) -> Cow<'q, str> {
        execute_lang_method! { self, pre_process_pattern, query }
    }
    fn from_path<P: AsRef<Path>>(path: P) -> Option<Self> {
        from_extension(path.as_ref())
    }
}

#[cfg(feature = "matching")]
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

const fn extensions(lang: SupportLang) -> &'static [&'static str] {
    use SupportLang::*;
    match lang {
        #[cfg(feature = "bash")]
        Bash => &BASH_EXTENSION_PATTERN,
        #[cfg(feature = "c")]
        C => &["c", "h"],
        #[cfg(feature = "cpp")]
        Cpp => &["cc", "hpp", "cpp", "c++", "hh", "cxx", "cu", "ino"],
        #[cfg(feature = "csharp")]
        CSharp => &["cs"],
        #[cfg(feature = "css")]
        Css => &["css", "scss"],
        #[cfg(feature = "elixir")]
        Elixir => &["ex", "exs"],
        #[cfg(feature = "go")]
        Go => &["go"],
        #[cfg(feature = "haskell")]
        Haskell => &["hs"],
        #[cfg(feature = "html")]
        Html => &["html", "htm", "xhtml"],
        #[cfg(feature = "java")]
        Java => &["java"],
        #[cfg(feature = "javascript")]
        JavaScript => &["cjs", "js", "mjs", "jsx"],
        #[cfg(feature = "json")]
        Json => &["json"],
        #[cfg(feature = "kotlin")]
        Kotlin => &["kt", "ktm", "kts"],
        #[cfg(feature = "lua")]
        Lua => &["lua"],
        #[cfg(feature = "php")]
        Php => &["php"],
        #[cfg(feature = "python")]
        Python => &["py", "py3", "pyi", "bzl"],
        #[cfg(feature = "ruby")]
        Ruby => &["rb", "rbw", "gemspec"],
        #[cfg(feature = "rust")]
        Rust => &["rs"],
        #[cfg(feature = "scala")]
        Scala => &["scala", "sc", "sbt"],
        #[cfg(feature = "swift")]
        Swift => &["swift"],
        #[cfg(feature = "typescript")]
        TypeScript => &["ts", "cts", "mts"],
        #[cfg(feature = "tsx")]
        Tsx => &["tsx"],
        #[cfg(feature = "yaml")]
        Yaml => &["yaml", "yml"],
    }
}

/// Guess which programming language a file is written in
/// Adapt from `<https://github.com/Wilfred/difftastic/blob/master/src/parse/guess_language.rs>`
/// N.B do not confuse it with `FromStr` trait. This function is to guess language from file extension.
fn from_extension(path: &Path) -> Option<SupportLang> {
    let ext = path.extension()?.to_str()?;
    #[cfg(feature = "bash")]
    if BASH_EXTENSION_PATTERN.contains(&ext) {
        return Some(SupportLang::Bash)
    }
    // Fast path: try most common extensions first
    match ext {
        #[cfg(feature = "c")]
        "c" | "h" => return Some(SupportLang::C),
        #[cfg(feature = "cpp")]
        "cpp" | "cc" | "cxx" => return Some(SupportLang::Cpp),
        #[cfg(feature = "css")]
        "css" => return Some(SupportLang::Css),
        #[cfg(feature = "go")]
        "go" => return Some(SupportLang::Go),
        #[cfg(feature = "html")]
        "html" | "htm" => return Some(SupportLang::Html),
        #[cfg(feature = "java")]
        "java" => return Some(SupportLang::Java),
        #[cfg(feature = "javascript")]
        "js" | "mjs" | "cjs" => return Some(SupportLang::JavaScript),
        #[cfg(feature = "json")]
        "json" => return Some(SupportLang::Json),
        #[cfg(feature = "python")]
        "py" | "py3" | "pyi" => return Some(SupportLang::Python),
        #[cfg(feature = "rust")]
        "rs" => return Some(SupportLang::Rust),
        #[cfg(feature = "typescript")]
        "ts" | "cts" | "mts" => return Some(SupportLang::TypeScript),
        #[cfg(feature = "tsx")]
        "tsx" => return Some(SupportLang::Tsx),
        #[cfg(feature = "yaml")]
        "yaml" | "yml" => return Some(SupportLang::Yaml),
        _ => {}
    }

    // Fallback: comprehensive search for less common extensions
    SupportLang::all_langs()
        .iter()
        .copied()
        .find(|&l| extensions(l).contains(&ext))
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

    // TODO: add test for file_types
}
