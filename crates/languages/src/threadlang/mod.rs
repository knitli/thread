// SPDX-FileCopyrightText: 2025 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: MIT

use crate::SupportedLanguage;

use ignore::types::Types;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "ag-dynamic-language")]
use thread_ast_grep::{CustomLang, DynamicLang};
#[cfg(feature = "ag-language")]
use thread_ast_grep::{Language, LanguageExt};
#[cfg(feature = "ag-tree-sitter")]
use thread_ast_grep::{Node, StrDoc, TSLanguage, TSRange};
#[cfg(feature = "ag-matcher")]
use thread_ast_grep::{Pattern, PatternBuilder, PatternError};

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;
use std::str::FromStr;

#[cfg(feature = "ag-config")]
pub use injection::SerializableInjection;

#[cfg(feature = "ag-config")]
pub use lang_globs::*;

#[cfg(feature = "ag-config")]
pub use crate::config::{AstGrepConfig, ProjectConfig, with_rule_stats, read_rule_file, TestConfig};

#[derive(Copy, Clone, PartialEq, Eq, #[cfg(feature = "serde")] Serialize, #[cfg(feature = "serde")] Deserialize, Hash)]
#[cfg_attr(feature = "serde", serde(untagged))]
/// Threadlang is a combined language type that can represent both built-in and custom languages.
pub enum ThreadLang {
    #[cfg(feature = "ag-language")]
    BuiltIn(SupportedLanguage),
    #[cfg(feature = "ag-dynamic-language")]
    Custom(CustomLang),
}

impl ThreadLang {
    /// Returns the file types associated with the language.
    pub fn file_types(&self) -> Vec<String> {
        match self {
            #[cfg(feature = "ag-language")]
            ThreadLang::BuiltIn(lang) => lang.file_types(),
            #[cfg(feature = "ag-dynamic-language")]
            ThreadLang::Custom(lang) => lang.file_types(),
            _ => Vec::new(),
        }
    }

    /// Returns all available languages, both built-in and custom, as a list of strings.
    pub fn all_languages() -> Vec<Self> {
        #[cfg(feature = "ag-language")]
        let builtin = SupportedLanguage::all_languages()
            .iter()
            .copied()
            .map(ThreadLang::BuiltIn);
        #[cfg(all(feature = "ag-dynamic-language"), feature = "ag-language")]
        let customs = CustomLang::all_languages()
            .into_iter()
            .map(ThreadLang::Custom);
        builtin.chain(customs).collect()
    }

    #[cfg(all(feature = "ag-language", feature = "ag-config"))]
    /// Registers injectable languages -- these are languages that can be included in other languages, like CSS in HTML.
    pub fn register_injections(injections: Vec<SerializableInjection>) -> Result<()> {
        unsafe { injection::register_injectables(injections) }
    }

    #[cfg(feature = "ag-config")]
    /// Returns a list of injectable languages for the current language.
    fn injectable_languages(&self) -> Option<&'static [&'static str]> {
        injection::injectable_languages(*self)
    }

    #[cfg(feature = "ag-config")]
    /// Returns an iterator over injectable languages that are supported by the current context.
    pub fn injectable_languages(&self) -> Option<impl Iterator<Item = Self>> {
        let languages = self.injectable_languages()?;
        // e.g vue can inject scss which is not supported by sg
        // we should report an error here
        let iter = languages.iter().filter_map(|s| {
            if let Some(lang) = SgLang::from_str(s).ok() {
                if self.all_languages().contains(&lang) {
                    // Only return languages that are supported by the current context
                    Some(lang)
                }
            }
        });
        Some(iter)
    }

    #[cfg(feature = "ag-config")]
    /// Returns the augmented file type for the current language, which includes the file types of injectable languages.
    pub fn augmented_file_type(&self) -> Types {
        let self_type = self.file_types();
        let injector = Self::all_languages().into_iter().filter_map(|lang| {
            lang.injectable_languages()?
                .any(|l| l == *self)
                .then_some(lang)
        });
        let injector_types = injector.map(|lang| lang.file_types());
        let all_types = std::iter::once(self_type).chain(injector_types);
        lang_globs::merge_types(all_types)
    }

    #[cfg(feature = "ag-config")]
    /// Merges file types for a given iterator of languages, including injectable languages.
    pub fn file_types_for_languages(languages: impl Iterator<Item = Self>) -> Types {
        let types = languages.map(|lang| lang.augmented_file_type());
        lang_globs::merge_types(types)
    }
}

impl Display for ThreadLang {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "ag-language")]
            ThreadLang::BuiltIn(lang) => write!(f, "{}", lang),
            #[cfg(feature = "ag-dynamic-language")]
            ThreadLang::Custom(lang) => write!(f, "{}", lang.name()),
            _ => write!(f, "null"),
        }
    }
}

impl Debug for ThreadLang {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "ag-language")]
            ThreadLang::BuiltIn(lang) => write!(f, "{:?}", lang),
            #[cfg(feature = "ag-dynamic-language")]
            ThreadLang::Custom(lang) => write!(f, "{:?}", lang.name()),
            _ => write!(f, "NO VALUE -- NO LANGUAGE FEATURES ENABLED"),
        }
    }
}

#[derive(Debug)]
pub enum ThreadLangError {
    LanguageNotSupported(String),
}

impl Display for ThreadLangError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ThreadLangError::LanguageNotSupported(lang) => write!(f, "{lang} is not supported!"),
        }
    }
}

impl std::error::Error for ThreadLangError {}

impl FromStr for ThreadLang {
    type Err = ThreadLangError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        #[cfg(feature = "ag-language")]
        if let Ok(lang) = SupportedLanguage::from_str(s) {
            return Ok(ThreadLang::BuiltIn(lang));
        }
        #[cfg(feature = "ag-dynamic-language")]
        if let Ok(lang) = CustomLang::from_str(s) {
            return Ok(ThreadLang::Custom(lang));
        }
        Err(ThreadLangError::LanguageNotSupported(s.to_string()))
    }
}

impl From<SupportedLanguage> for ThreadLang {
    fn from(value: SupportedLanguage) -> Self {
        Self::BuiltIn(value)
    }
}

impl From<CustomLang> for ThreadLang {
    fn from(value: CustomLang) -> Self {
        Self::Custom(value)
    }
}

use ThreadLang::*;
#[cfg(
    all(any(feature = "ag-language", feature = "ag-dynamic-language")),
    feature = "meta-var",
    feature = "ag-matcher"
)]
impl Language for ThreadLang {
    fn pre_process_pattern<'q>(&self, query: &'q str) -> Cow<'q, str> {
        match self {
            #[cfg(feature = "ag-language")]
            ThreadLang::BuiltIn(lang) => lang.pre_process_pattern(query),
            #[cfg(feature = "ag-dynamic-language")]
            ThreadLang::Custom(lang) => lang.pre_process_pattern(query),
            _ => Cow::Borrowed(query),
        }
    }

    #[inline]
    fn meta_var_char(&self) -> char {
        match self {
            #[cfg(feature = "ag-language")]
            ThreadLang::BuiltIn(lang) => lang.meta_var_char(),
            #[cfg(feature = "ag-dynamic-language")]
            ThreadLang::Custom(lang) => lang.meta_var_char(),
        }
    }

    #[inline]
    fn expando_char(&self) -> char {
        match self {
            #[cfg(feature = "ag-language")]
            ThreadLang::BuiltIn(lang) => lang.expando_char(),
            #[cfg(feature = "ag-dynamic-language")]
            ThreadLang::Custom(lang) => lang.expando_char(),
        }
    }

    fn kind_to_id(&self, kind: &str) -> u16 {
        match self {
            #[cfg(feature = "ag-language")]
            ThreadLang::BuiltIn(lang) => lang.kind_to_id(kind),
            #[cfg(feature = "ag-dynamic-language")]
            ThreadLang::Custom(lang) => lang.kind_to_id(kind),
            _ => 0,
        }
    }

    fn field_to_id(&self, field: &str) -> Option<u16> {
        match self {
            #[cfg(feature = "ag-language")]
            ThreadLang::BuiltIn(lang) => lang.field_to_id(field),
            #[cfg(feature = "ag-dynamic-language")]
            ThreadLang::Custom(lang) => lang.field_to_id(field),
            _ => None,
        }
    }

    fn from_path<P: AsRef<Path>>(&self, path: P) -> Option<Self> {
        match self {
            #[cfg(feature = "ag-language")]
            ThreadLang::BuiltIn(lang) => lang.from_path(path).map(ThreadLang::BuiltIn),
            #[cfg(feature = "ag-dynamic-language")]
            ThreadLang::Custom(lang) => lang.from_path(path).map(ThreadLang::Custom),
            _ => None,
        }
    }

    fn build_pattern(&self, builder: &PatternBuilder) -> Result<Pattern, PatternError> {
        match self {
            #[cfg(feature = "ag-language")]
            ThreadLang::BuiltIn(lang) => lang.build_pattern(builder),
            #[cfg(feature = "ag-dynamic-language")]
            ThreadLang::Custom(lang) => lang.build_pattern(builder),
        }
    }
}

#[cfg(
    all(feature = "ag-tree-sitter"),
    any(feature = "ag-language", feature = "ag-dynamic-language")
)]
impl LanguageExt for ThreadLang {
    fn get_ts_language(&self) -> TSLanguage {
        match self {
            #[cfg(feature = "ag-language")]
            ThreadLang::BuiltIn(lang) => lang.get_ts_language(),
            #[cfg(feature = "ag-dynamic-language")]
            ThreadLang::Custom(lang) => lang.get_ts_language(),
        }
    }

    fn injectable_languages(&self) -> Option<&'static [&'static str]> {
        injection::injectable_languages(*self)
    }

    fn extract_injections<L: LanguageExt>(
        &self,
        root: Node<StrDoc<L>>,
    ) -> HashMap<String, Vec<TSRange>> {
        injection::extract_injections(self, root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[cfg(feature = "ag-dynamic-language")]
    #[test]
    fn test_threadlang_size() {
        // Ensure that ThreadLang is the same size as SupportedLanguage or CustomLang
        assert_eq!(size_of::<ThreadLang>(), size_of::<DynamicLang>());
    }
}
