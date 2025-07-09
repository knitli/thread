// SPDX-FileCopyrightText: 2025 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: MIT


use thread_languages::SupportedLanguage;
use crate::{
    #[cfg(feature = "language")] Language,
    #[cfg(feature = "language")] LanguageExt,
    #[cfg(feature = "dynamic-language")] CustomLang,
    #[cfg(feature = "dynamic-language")] DynamicLang
};
#[cfg(feature = "matcher")]
use crate::{Pattern, PatternBuilder, PatternError};
#[cfg(feature = "tree-sitter")]
use crate::{StrDoc, TSLanguage, TSRange, Node};
#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};
use ignore::types::Types;

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;
use std::str::FromStr;

#[cfg(feature = "ag-config")]
pub use injection::SerializableInjection;

#[cfg(feature = "ag-config")]
pub use lang_globs::*;

#[derive(Copy, Clone, PartialEq, Eq, #[cfg(feature = "serde")] Serialize, #[cfg(feature = "serde")] Deserialize, Hash)]
#[cfg_attr(feature = "serde", serde(untagged))]
/// Threadlang is a combined language type that can represent both built-in and custom languages.
pub enum ThreadLang {
    #[cfg(feature = "language")]
    BuiltIn(SupportedLanguage),
    #[cfg(feature = "dynamic-language")]
    Custom(CustomLang),
}

impl ThreadLang {
    /// Returns the file types associated with the language.
    pub fn file_types(&self) -> Vec<String> {
        match self {
            #[cfg(feature = "language")]
            ThreadLang::BuiltIn(lang) => lang.file_types(),
            #[cfg(feature = "dynamic-language")]
            ThreadLang::Custom(lang) => lang.file_types(),
        }
    }

    /// Returns all available languages, both built-in and custom, as a list of strings.
    pub fn all_languages() -> Vec<Self> {
        #[cfg(feature = "language")]
        let builtin = SupportedLanguage::all_languages().iter().copied().map(ThreadLang::BuiltIn);
        #[cfg(all(feature = "dynamic-language"), feature = "language")]
        let customs = CustomLang::all_languages().into_iter().map(ThreadLang::Custom);
        builtin.chain(customs).collect()
    }

    #[cfg(all(feature = "language", feature = "ag-config"))]
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
        let iter = languages.iter().filter_map(|s| if let Some(lang) = SgLang::from_str(s).ok() {
            if self.all_languages().contains(&lang) {
                // Only return languages that are supported by the current context
                Some(lang)
            }
        });
        Some(iter)
    }

    #[cfg(feature = "ag-config")]
    /// Returns the augmented file type for the current language, which includes the file types of injectable languages.
    pub fn augmented_file_type(&self) -> Types {
        let self_type = self.file_types();
        let injector = Self::all_languages().into_iter().filter_map(|lang| {
            lang
            .injectable_languages()?
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
            #[cfg(feature = "language")]
            ThreadLang::BuiltIn(lang) => write!(f, "{}", lang),
            #[cfg(feature = "dynamic-language")]
            ThreadLang::Custom(lang) => write!(f, "{}", lang.name()),
        }
    }
}

impl Debug for ThreadLang {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "language")]
            ThreadLang::BuiltIn(lang) => write!(f, "{:?}", lang),
            #[cfg(feature = "dynamic-language")]
            ThreadLang::Custom(lang) => write!(f, "{:?}", lang.name()),
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
        #[cfg(feature = "language")]
        if let Ok(lang) = SupportedLanguage::from_str(s) {
            return Ok(ThreadLang::BuiltIn(lang));
        }
        #[cfg(feature = "dynamic-language")]
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

use ThreadLang::*
impl Language for ThreadLang {
    fn pre_process_pattern<'q>(&self, query: &'q str) -> Cow<'q, str> {
        match self {
            #[cfg(feature = "language")]
            ThreadLang::BuiltIn(lang) => lang.pre_process_pattern(query),
            #[cfg(feature = "dynamic-language")]
            ThreadLang::Custom(lang) => lang.pre_process_pattern(query),
        }
    }

    #[inline]
    #[cfg(all(feature = "language", feature = "meta-var"))]
    fn meta_var_char(&self) -> char {
        match self {
            #[cfg(feature = "language")]
            ThreadLang::BuiltIn(lang) => lang.meta_var_char(),
            #[cfg(feature = "dynamic-language")]
            ThreadLang::Custom(lang) => lang.meta_var_char(),
        }
    }

    #[inline]
    #[cfg(feature = "language")]
    fn expando_char(&self) -> char {
        match self {
            #[cfg(feature = "language")]
            ThreadLang::BuiltIn(lang) => lang.expando_char(),
            #[cfg(feature = "dynamic-language")]
            ThreadLang::Custom(lang) => lang.expando_char(),
        }
    }

    fn kind_to_id(&self, kind: &str) -> u16 {
        match self {
            #[cfg(feature = "language")]
            ThreadLang::BuiltIn(lang) => lang.kind_to_id(kind),
            #[cfg(feature = "dynamic-language")]
            ThreadLang::Custom(lang) => lang.kind_to_id(kind),
        }
    }

    fn field_to_id(&self, field: &str) -> Option<u16> {
        match self {
            #[cfg(feature = "language")]
            ThreadLang::BuiltIn(lang) => lang.field_to_id(field),
            #[cfg(feature = "dynamic-language")]
            ThreadLang::Custom(lang) => lang.field_to_id(field),
        }
    }

    fn from_path<P: AsRef<Path>>(&self, path: P) -> Option<Self> {
        match self {
            #[cfg(feature = "language")]
            ThreadLang::BuiltIn(lang) => lang.from_path(path).map(ThreadLang::BuiltIn),
            #[cfg(feature = "dynamic-language")]
            ThreadLang::Custom(lang) => lang.from_path(path).map(ThreadLang::Custom),
        }
    }

    #[cfg(feature = "matcher", any(feature = "language", feature = "dynamic-language"))]
    fn build_pattern(&self, builder: &PatternBuilder) -> Result<Pattern, PatternError> {
        match self {
            #[cfg(feature = "language")]
            ThreadLang::BuiltIn(lang) => lang.build_pattern(builder),
            #[cfg(feature = "dynamic-language")]
            ThreadLang::Custom(lang) => lang.build_pattern(builder),
        }
    }
}

impl LanguageExt for ThreadLang {
    #[cfg(feature = "tree-sitter", any(feature = "language", feature = "dynamic-language"))]
    fn get_ts_language(&self) -> TSLanguage {
        match self {
            #[cfg(feature = "language")]
            ThreadLang::BuiltIn(lang) => lang.get_ts_language(),
            #[cfg(feature = "dynamic-language")]
            ThreadLang::Custom(lang) => lang.get_ts_language(),
        }
    }

    #[cfg(all(feature = "tree-sitter", any(feature = "language", feature = "dynamic-language"), feature = "ag-config"))]
    fn injectable_languages(&self) -> Option<&'static [&'static str]> {
        injection::injectable_languages(*self)
    }

    #[cfg(all(feature = "tree-sitter", feature = "ag-config"))]
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

    #[cfg(feature = "dynamic-language")]
    #[test]
    fn test_threadlang_size() {
        // Ensure that ThreadLang is the same size as SupportedLanguage or CustomLang
        assert_eq!(size_of::<ThreadLang>(), size_of::<DynamicLang>());
    }
}
