// SPDX-FileCopyrightText: 2025 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: MIT
pub mod config;
pub mod injection;
pub mod lang_globs;
use thread_languages::SupportedLanguage;

use ignore::types::Types;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "ag-dynamic-language")]
use ast_grep_dynamic::{CustomLang, DynamicLang};
use ag_service_tree_sitter::{Language, LanguageExt, TSLanguage, TSRange, StrDoc};
use ag_service_ast::{Doc, Node};
use ag_service_pattern::{Pattern, PatternBuilder, PatternError};

use thread_utils::FastMap;
use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;
use std::str::FromStr;


pub use injection::SerializableInjection;


pub use lang_globs::*;


pub use config::{
    AstGrepConfig, ProjectConfig, TestConfig, read_rule_file, with_rule_stats,
};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(untagged))]
/// Threadlang is a combined language type that can represent both built-in and custom languages.
pub enum ThreadLang {
    BuiltIn(SupportedLanguage),
    #[cfg(feature = "ag-dynamic-language")]
    Custom(CustomLang),
}

impl ThreadLang {
    /// Returns the file types associated with the language.
    pub fn file_types(&self) -> Vec<String> {
        match self {
            ThreadLang::BuiltIn(lang) => lang.file_types(),
            #[cfg(feature = "ag-dynamic-language")]
            ThreadLang::Custom(lang) => lang.file_types(),
            _ => Vec::new(),
        }
    }

    /// Returns all available languages, both built-in and custom, as a list of strings.
    pub fn all_languages() -> Vec<Self> {
        let mut languages = Vec::new();
        languages.extend(ThreadLang::BuiltIn.all_languages());
        #[cfg(feature = "ag-dynamic-language")]
        languages.extend(ThreadLang::Custom.all_languages());
        languages
    }


    /// Registers injectable languages -- these are languages that can be included in other languages, like CSS in HTML.
    pub fn register_injections(injections: Vec<SerializableInjection>) -> Result<()> {
        unsafe { injection::register_injectables(injections) }
    }


    /// Returns a list of injectable languages for the current language.
    fn injectable_languages(&self) -> Option<&'static [&'static str]> {
        injection::injectable_languages(*self)
    }


    /// Returns an iterator over injectable languages that are supported by the current context.
    pub fn injectable_languages(&self) -> Option<impl Iterator<Item = Self>> {
        let languages = self.injectable_languages()?;
        // e.g vue can inject scss which is not supported by sg
        // we should report an error here
        let iter = languages.iter().filter_map(|s| {
            if let Some(lang) = ThreadLang::from_str(s).ok() {
                if self.all_languages().contains(&lang) {
                    // Only return languages that are supported by the current context
                    Some(lang)
                }
            }
        });
        Some(iter)
    }


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


    /// Merges file types for a given iterator of languages, including injectable languages.
    pub fn file_types_for_languages(languages: impl Iterator<Item = Self>) -> Types {
        let types = languages.map(|lang| lang.augmented_file_type());
        lang_globs::merge_types(types)
    }
}

impl Display for ThreadLang {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
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
#[cfg(feature = "ag-dynamic-language")]
impl From<CustomLang> for ThreadLang {
    fn from(value: CustomLang) -> Self {
        Self::Custom(value)
    }
}

use ThreadLang::*;
#[cfg(feature = "ag-dynamic-language")]
impl Language for ThreadLang {
    fn pre_process_pattern<'q>(&self, query: &'q str) -> Cow<'q, str> {
        match self {
            ThreadLang::BuiltIn(lang) => lang.pre_process_pattern(query),
            #[cfg(feature = "ag-dynamic-language")]
            ThreadLang::Custom(lang) => lang.pre_process_pattern(query),
            _ => Cow::Borrowed(query),
        }
    }

    #[inline]
    fn meta_var_char(&self) -> char {
        match self {
            ThreadLang::BuiltIn(lang) => lang.meta_var_char(),
            #[cfg(feature = "ag-dynamic-language")]
            ThreadLang::Custom(lang) => lang.meta_var_char(),
        }
    }

    #[inline]
    fn expando_char(&self) -> char {
        match self {
            ThreadLang::BuiltIn(lang) => lang.expando_char(),
            #[cfg(feature = "ag-dynamic-language")]
            ThreadLang::Custom(lang) => lang.expando_char(),
        }
    }

    fn kind_to_id(&self, kind: &str) -> u16 {
        match self {
            ThreadLang::BuiltIn(lang) => lang.kind_to_id(kind),
            #[cfg(feature = "ag-dynamic-language")]
            ThreadLang::Custom(lang) => lang.kind_to_id(kind),
            _ => 0,
        }
    }

    fn field_to_id(&self, field: &str) -> Option<u16> {
        match self {
            ThreadLang::BuiltIn(lang) => lang.field_to_id(field),
            #[cfg(feature = "ag-dynamic-language")]
            ThreadLang::Custom(lang) => lang.field_to_id(field),
            _ => None,
        }
    }

    fn from_path<P: AsRef<Path>>(&self, path: P) -> Option<Self> {
        match self {
            ThreadLang::BuiltIn(lang) => lang.from_path(path).map(ThreadLang::BuiltIn),
            #[cfg(feature = "ag-dynamic-language")]
            ThreadLang::Custom(lang) => lang.from_path(path).map(ThreadLang::Custom),
            _ => None,
        }
    }

    fn build_pattern(&self, builder: &PatternBuilder) -> Result<Pattern, PatternError> {
        match self {
            ThreadLang::BuiltIn(lang) => lang.build_pattern(builder),
            #[cfg(feature = "ag-dynamic-language")]
            ThreadLang::Custom(lang) => lang.build_pattern(builder),
        }
    }
}

#[cfg(feature = "ag-dynamic-language")]
impl LanguageExt for ThreadLang {
    fn get_ts_language(&self) -> TSLanguage {
        match self {
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
    ) -> FastMap<String, Vec<TSRange>> {
        injection::extract_injections(self, root)
    }
}

#[cfg(feature = "ag-dynamic-language")]
#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_threadlang_size() {
        // Ensure that ThreadLang is the same size as SupportedLanguage or CustomLang
        assert_eq!(size_of::<ThreadLang>(), size_of::<DynamicLang>());
    }
}
