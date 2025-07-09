// SPDX-FileCopyrightText: 2025 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: MIT

//! This module defines the supported programming languages for Thread.
//!
//! This module was originally forked from the `ast-grep-language` crate as of ast-grep 0.3.8.
//! Ast-grep doesn't publicly export the languages and we wanted to feature gate each language, add languages and have finer control.
//!
//! **We have kept the original licensing of the `ast-grep-language` crate, which is licensed under the MIT license.** The rest of Thread is licensed under the AGPL 3.0, but we wanted to stay true to the author's intent.
//!
//! It provides a set of customized languages with expando_char / pre_process_pattern,
//! and a set of stub languages without preprocessing.
//!
//! A rule of thumb for adding languages:
//!   - You need to use `impl_lang_expando!` macro and a standalone file for testing **if your language doesn't accept identifiers like `$VAR`.
//!
//! The full list of supported languages is available in the [`SupportedLanguage` enum][`SupportedLanguage`] below.

#[cfg(feature = "bash")]
mod bash;
#[cfg(feature = "cpp")]
mod cpp;
#[cfg(feature = "c-sharp")]
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

pub mod parsers;

#[cfg(feature = "html")]
pub use html::Html;

// While the threadlang module *can* run with any of these features, it is only fully functional when they are *all* enabled.
#[cfg(feature = "threadlang")]
#[cfg((all(any(feature = "ag-language", feature = "ag-dynamic-language", feature = "ag-config")), feature = "threadlang"))]
pub mod threadlang;

use thread_ag::{
    MetaVariable, Node, Pattern, PatternBuilder, PatternError, StrDoc, TSLanguage, TSRange,
};
use thread_core::fastmap::FastMap;

use ignore::types::{Types, TypesBuilder};
#[cfg(any(feature = "serde-derive", feature = "serde-no-derive"))]
use serde::de::Visitor;
#[cfg(feature = "serde-derive")]
use serde::{Deserialize, Deserializer, Serialize, de};
use std::borrow::Cow;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::iter::repeat;
use std::path::Path;
use std::str::FromStr;

pub use thread_core::{Language, LanguageExt};

/// this macro implements bare-bone methods for a language
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

fn pre_process_pattern(expando: char, query: &str) -> std::borrow::Cow<str> {
    let mut ret = Vec::with_capacity(query.len());
    let mut dollar_count = 0;
    for c in query.chars() {
        if c == '$' {
            dollar_count += 1;
            continue;
        }
        let need_replace = matches!(c, 'A'..='Z' | '_') // $A or $$A or $$$A
      || dollar_count == 3; // anonymous multiple
        let sigil = if need_replace { expando } else { '$' };
        ret.extend(repeat(sigil).take(dollar_count));
        dollar_count = 0;
        ret.push(c);
    }
    // trailing anonymous multiple
    let sigil = if dollar_count == 3 { expando } else { '$' };
    ret.extend(repeat(sigil).take(dollar_count));
    std::borrow::Cow::Owned(ret.into_iter().collect())
}

/// this macro will implement expando_char and pre_process_pattern
/// use this if your language does not accept $ as valid identifier char
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

        #[cfg_attr(
            not(no_diagnostic_namespace),
            diagnostic::on_unimplemented(note = "You probably need to enable the 'serde' feature")
        )]
        #[cfg(feature = "serde-derive")]
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

        impl From<$lang> for SupportedLanguage {
            fn from(_: $lang) -> Self {
                Self::$lang
            }
        }
    };
}
/// Generates as convenience conversions between the lang types
/// and `SupportedType`.
macro_rules! impl_aliases {
  ($($lang:ident => $as:expr),* $(,)?) => {
    $(impl_alias!($lang => $as);)*
    const fn alias(lang: SupportedLanguage) -> &'static [&'static str] {
      match lang {
        $(SupportedLanguage::$lang => $lang::ALIAS),*
      }
    }
  };
}

/* Customized Language with expando_char / pre_process_pattern */
// https://en.cppreference.com/w/cpp/language/identifiers
// Due to some issues in the tree-sitter parser, it is not possible to use
// unicode literals in identifiers for C/C++ parsers
#[cfg(feature = "c")]
impl_lang_expando!(C, language_c, '_');
#[cfg(feature = "cpp")]
impl_lang_expando!(Cpp, language_cpp, '_');
// https://docs.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/lexical-structure#643-identifiers
// all letter number is accepted
// https://www.compart.com/en/unicode/category/Nl
#[cfg(feature = "c-sharp")]
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

/// Represents all built-in languages.
#[derive(Clone, Copy, Debug, PartialEq, Eq, #[cfg_attr(
  not(no_diagnostic_namespace),
  diagnostic::on_unimplemented(note = "You probably need to enable the 'serde' feature"))
] #[cfg(feature = "serde-derive")] Serialize, Hash)]
pub enum SupportedLanguage {
    #[cfg(feature = "bash")]
    Bash,
    #[cfg(feature = "c")]
    C,
    #[cfg(feature = "cpp")]
    Cpp,
    #[cfg(feature = "c-sharp")]
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
}

impl SupportedLanguage {
    pub const fn all_languages() -> &'static [SupportedLanguage] {
        use SupportedLanguage::*;
        &[
            #[cfg(feature = "bash")]
            Bash,
            #[cfg(feature = "c")]
            C,
            #[cfg(feature = "cpp")]
            Cpp,
            #[cfg(feature = "c-sharp")]
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

impl fmt::Display for SupportedLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug)]
pub enum SupportedLanguageErr {
    LanguageNotSupported(String),
}

impl Display for SupportedLanguageErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        use SupportedLanguageErr::*;
        match self {
            LanguageNotSupported(lang) => write!(f, "{lang} is not supported!"),
        }
    }
}

impl std::error::Error for SupportedLanguageErr {}

#[cfg(feature = "serde-derive")]
impl<'de> Deserialize<'de> for SupportedLanguage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(SupportedLanguageVisitor)
    }
}

#[cfg(any(feature = "serde-derive", feature = "serde-no-derive"))]
struct SupportedLanguageVisitor;
#[cfg(any(feature = "serde-derive", feature = "serde-no-derive"))]
impl Visitor<'_> for SupportedLanguageVisitor {
    type Value = SupportedLanguage;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("SupportedLanguage")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        v.parse().map_err(de::Error::custom)
    }
}
#[cfg(any(feature = "serde-derive", feature = "serde-no-derive"))]
struct AliasVisitor {
    aliases: &'static [&'static str],
}
#[cfg(any(feature = "serde-derive", feature = "serde-no-derive"))]
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
  Bash => &["bash"],
  C => &["c"],
  Cpp => &["cc", "c++", "cpp", "cxx"],
  CSharp => &["cs", "csharp"],
  Css => &["css"],
  Elixir => &["ex", "elixir"],
  Go => &["go", "golang"],
  Haskell => &["hs", "haskell"],
  Html => &["html"],
  Java => &["java"],
  JavaScript => &["javascript", "js", "jsx"],
  Json => &["json"],
  Kotlin => &["kotlin", "kt"],
  Lua => &["lua"],
  Php => &["php"],
  Python => &["py", "python"],
  Ruby => &["rb", "ruby"],
  Rust => &["rs", "rust"],
  Scala => &["scala"],
  Swift => &["swift"],
  TypeScript => &["ts", "typescript"],
  Tsx => &["tsx"],
  Yaml => &["yaml", "yml"],
}

/// Implements the language names and aliases.
impl FromStr for SupportedLanguage {
    type Err = SupportedLanguageErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for &lang in Self::all_languages() {
            for moniker in alias(lang) {
                if s.eq_ignore_ascii_case(moniker) {
                    return Ok(lang);
                }
            }
        }
        Err(SupportedLanguageErr::LanguageNotSupported(s.to_string()))
    }
}

macro_rules! execute_lang_method {
  ($me: path, $method: ident, $($pname:tt),*) => {
    use SupportedLanguage as S;
    match $me {
      #[cfg(feature = "bash")]
      S::Bash => Bash.$method($($pname,)*),
      #[cfg(feature = "c")]
      S::C => C.$method($($pname,)*),
      #[cfg(feature = "cpp")]
      S::Cpp => Cpp.$method($($pname,)*),
      #[cfg(feature = "c-sharp")]
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
      #[cfg(feature = "java")]
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
      #[cfg(not(feature = "bash"))]
      S::Bash => unreachable!(),
      #[cfg(not(feature = "c"))]
      S::C => unreachable!(),
      #[cfg(not(feature = "cpp"))]
      S::Cpp => unreachable!(),
      #[cfg(not(feature = "c-sharp"))]
      S::CSharp => unreachable!(),
      #[cfg(not(feature = "css"))]
      S::Css => unreachable!(),
      #[cfg(not(feature = "elixir"))]
      S::Elixir => unreachable!(),
      #[cfg(not(feature = "go"))]
      S::Go => unreachable!(),
      #[cfg(not(feature = "haskell"))]
      S::Haskell => unreachable!(),
      #[cfg(not(feature = "html"))]
      S::Html => unreachable!(),
      #[cfg(not(feature = "java"))]
      S::Java => unreachable!(),
      #[cfg(not(feature = "javascript"))]
      S::JavaScript => unreachable!(),
      #[cfg(not(feature = "json"))]
      S::Json => unreachable!(),
      #[cfg(not(feature = "kotlin"))]
      S::Kotlin => unreachable!(),
      #[cfg(not(feature = "lua"))]
      S::Lua => unreachable!(),
      #[cfg(not(feature = "php"))]
      S::Php => unreachable!(),
      #[cfg(not(feature = "python"))]
      S::Python => unreachable!(),
      #[cfg(not(feature = "ruby"))]
      S::Ruby => unreachable!(),
      #[cfg(not(feature = "rust"))]
      S::Rust => unreachable!(),
      #[cfg(not(feature = "scala"))]
      S::Scala => unreachable!(),
      #[cfg(not(feature = "swift"))]
      S::Swift => unreachable!(),
      #[cfg(not(feature = "tsx"))]
      S::Tsx => unreachable!(),
      #[cfg(not(feature = "typescript"))]
      S::TypeScript => unreachable!(),
      #[cfg(not(feature = "yaml"))]
      S::Yaml => unreachable!(),
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
impl Language for SupportedLanguage {
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

impl LanguageExt for SupportedLanguage {
    impl_lang_method!(get_ts_language, () => TSLanguage);
    impl_lang_method!(injectable_languages, () => Option<&'static [&'static str]>);
    fn extract_injections<L: LanguageExt>(
        &self,
        root: Node<StrDoc<L>>,
    ) -> FastMap<String, Vec<TSRange>> {
        match self {
            SupportedLanguage::Html => Html.extract_injections(root),
            _ => FastMap::new(),
        }
    }
}

#[allow(rust_analyzer::non_snake_case)]
fn extensions(lang: SupportedLanguage) -> &'static [&'static str] {
    use SupportedLanguage::*;
    static BASH: &[&str] = &[
        "bash", "bats", "cgi", "command", "env", "fcgi", "ksh", "sh", "tmux", "tool", "zsh",
    ];
    static C_: &[&str] = &["c", "h"];
    static CPP: &[&str] = &["cc", "hpp", "cpp", "c++", "hh", "cxx", "cu", "ino"];
    static CSHARP: &[&str] = &["cs"];
    static CSS: &[&str] = &["css", "scss"];
    static ELIXIR: &[&str] = &["ex", "exs"];
    static GO: &[&str] = &["go"];
    static HASKELL: &[&str] = &["hs"];
    static HTML: &[&str] = &["html", "htm", "xhtml"];
    static JAVA: &[&str] = &["java"];
    static JAVASCRIPT: &[&str] = &["cjs", "js", "mjs", "jsx"];
    static JSON: &[&str] = &["json"];
    static KOTLIN: &[&str] = &["kt", "ktm", "kts"];
    static LUA: &[&str] = &["lua"];
    static PHP: &[&str] = &["php"];
    static PYTHON: &[&str] = &["py", "py3", "pyi", "bzl"];
    static RUBY: &[&str] = &["rb", "rbw", "gemspec"];
    static RUST: &[&str] = &["rs"];
    static SCALA: &[&str] = &["scala", "sc", "sbt"];
    static SWIFT: &[&str] = &["swift"];
    static TYPESCRIPT: &[&str] = &["ts", "cts", "mts"];
    static TSX: &[&str] = &["tsx"];
    static YAML: &[&str] = &["yaml", "yml"];
    match lang {
        Bash => BASH,
        C => C_,
        Cpp => CPP,
        CSharp => CSHARP,
        Css => CSS,
        Elixir => ELIXIR,
        Go => GO,
        Haskell => HASKELL,
        Html => HTML,
        Java => JAVA,
        JavaScript => JAVASCRIPT,
        Json => JSON,
        Kotlin => KOTLIN,
        Lua => LUA,
        Php => PHP,
        Python => PYTHON,
        Ruby => RUBY,
        Rust => RUST,
        Scala => SCALA,
        Swift => SWIFT,
        TypeScript => TYPESCRIPT,
        Tsx => TSX,
        Yaml => YAML,
    }
}

pub fn supported_extensions() -> Vec<&'static str> {
    SupportedLanguage::all_languages()
        .iter()
        .flat_map(|lang| extensions(*lang))
        .copied()
        .collect()
}

/// Guess which programming language a file is written in
/// Adapt from `<https://github.com/Wilfred/difftastic/blob/master/src/parse/guess_language.rs>`
/// N.B do not confuse it with `FromStr` trait. This function is to guess language from file extension.
fn from_extension(path: &Path) -> Option<SupportedLanguage> {
    let ext = path.extension()?.to_str()?;
    SupportedLanguage::all_languages()
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

fn file_types(lang: SupportedLanguage) -> Types {
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
    use thread_core::{MatcherExt, Pattern};

    pub fn test_match_lang(query: &str, source: &str, lang: impl LanguageExt) {
        let cand = lang.ast_grep(source);
        let pattern = Pattern::new(query, lang);
        assert!(
            pattern.find_node(cand.root()).is_some(),
            "goal: {pattern:?}, candidate: {}",
            cand.root().get_inner_node().to_sexp(),
        );
    }

    pub fn test_non_match_lang(query: &str, source: &str, lang: impl LanguageExt) {
        let cand = lang.ast_grep(source);
        let pattern = Pattern::new(query, lang);
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
        assert_eq!(from_extension(path), Some(SupportedLanguage::Rust));
    }

    // TODO: add test for file_types
}
