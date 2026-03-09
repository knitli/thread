// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT
//! # Language Abstraction for AST Parsing
//!
//! This module defines the [`Language`](crates/ast-engine/src/language.rs:16) trait, which abstracts over language-specific details for AST parsing and pattern matching.
//!
//! ## Purpose
//!
//! - **Meta-variable Handling:** Configure how meta-variables (e.g., `$A`) are recognized and processed for different languages.
//! - **Pattern Preprocessing:** Normalize pattern code before matching, adapting to language-specific quirks.
//! - **Tree-sitter Integration:** Map node kinds and fields to tree-sitter IDs for efficient AST traversal.
//! - **Extensibility:** Support custom language implementations (see [`Tsx`](crates/ast-engine/src/language.rs:63) for TypeScript/TSX).
//!
//! ## Key Components
//!
//! - [`Language`](crates/ast-engine/src/language.rs:16): Core trait for language-specific AST operations.
//! - [`Tsx`](crates/ast-engine/src/language.rs:63): Example implementation for TypeScript/TSX.
//! - Meta-variable extraction and normalization utilities.
//!
//! ## Example
//!
//! ```rust,no_run
//! use thread_ast_engine::language::Language;
//!
//! let lang = Tsx {};
//! let pattern = lang.pre_process_pattern("var $A = $B");
//! let meta_var = lang.extract_meta_var("$A");
//! ```
#[allow(unused_imports)]
#[cfg(feature = "matching")]
use super::{Pattern, PatternBuilder, PatternError};
use crate::meta_var::{MetaVariable, extract_meta_var};
use std::borrow::Cow;
use std::path::Path;

/// Trait to abstract ts-language usage in ast-grep, which includes:
/// * which character is used for meta variable.
/// * if we need to use other char in meta var for parser at runtime
/// * pre process the Pattern code.
pub trait Language: Clone + std::fmt::Debug + Send + Sync + 'static {
    /// normalize pattern code before matching
    /// e.g. remove `expression_statement`, or prefer parsing {} to object over block
    fn pre_process_pattern<'q>(&self, query: &'q str) -> Cow<'q, str> {
        Cow::Borrowed(query)
    }

    /// Configure meta variable special character
    /// By default $ is the metavar char, but in PHP it can be #
    #[inline]
    fn meta_var_char(&self) -> char {
        '$'
    }

    /// Some language does not accept $ as the leading char for identifiers.
    /// We need to change $ to other char at run-time to make parser happy, thus the name expando.
    /// By default this is the same as `meta_var` char so replacement is done at runtime.
    #[inline]
    fn expando_char(&self) -> char {
        self.meta_var_char()
    }

    /// extract `MetaVariable` from a given source string
    /// At runtime we need to use `expand_char`
    fn extract_meta_var(&self, source: &str) -> Option<MetaVariable> {
        extract_meta_var(source, self.expando_char())
    }
    /// Return the file language inferred from a filesystem path.
    ///
    /// The *default* implementation is not implemented and will panic if called.
    /// Implementors should override this method and return `Some(Self)` when the
    /// file type is supported and `None` when it is not.
    fn from_path<P: AsRef<Path>>(_path: P) -> Option<Self> {
        unimplemented!(
            "Language::from_path is not implemented for type `{}`. \
             Override Language::from_path for this type if path-based detection is required.",
            std::any::type_name::<Self>()
        )
    }

    fn kind_to_id(&self, kind: &str) -> u16;
    fn field_to_id(&self, field: &str) -> Option<u16>;
    #[cfg(feature = "matching")]
    fn build_pattern(&self, builder: &PatternBuilder) -> Result<Pattern, PatternError>;
}

#[cfg(test)]
pub use test::*;

#[cfg(test)]
mod test {
    use super::*;
    use crate::tree_sitter::{LanguageExt, StrDoc, TSLanguage};

    // Shared helpers for test Language impls backed by tree-sitter-typescript.
    fn tsx_kind_to_id(kind: &str) -> u16 {
        let ts_lang: TSLanguage = tree_sitter_typescript::LANGUAGE_TSX.into();
        ts_lang.id_for_node_kind(kind, /* named */ true)
    }

    fn tsx_field_to_id(field: &str) -> Option<u16> {
        let ts_lang: TSLanguage = tree_sitter_typescript::LANGUAGE_TSX.into();
        ts_lang.field_id_for_name(field).map(|f| f.get())
    }

    fn tsx_ts_language() -> TSLanguage {
        tree_sitter_typescript::LANGUAGE_TSX.into()
    }

    #[derive(Clone, Debug)]
    pub struct Tsx;
    impl Language for Tsx {
        fn kind_to_id(&self, kind: &str) -> u16 {
            tsx_kind_to_id(kind)
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
    impl LanguageExt for Tsx {
        fn get_ts_language(&self) -> TSLanguage {
            tsx_ts_language()
        }
    }

    /// A minimal `Language` impl that does *not* override `from_path`, used to
    /// verify that the default implementation panics.
    #[derive(Clone, Debug)]
    struct NoFromPath;
    impl Language for NoFromPath {
        fn kind_to_id(&self, kind: &str) -> u16 {
            tsx_kind_to_id(kind)
        }
        fn field_to_id(&self, field: &str) -> Option<u16> {
            tsx_field_to_id(field)
        }
        #[cfg(feature = "matching")]
        fn build_pattern(&self, builder: &PatternBuilder) -> Result<Pattern, PatternError> {
            builder.build(|src| StrDoc::try_new(src, self.clone()))
        }
    }
    impl LanguageExt for NoFromPath {
        fn get_ts_language(&self) -> TSLanguage {
            tsx_ts_language()
        }
    }

    /// A `Language` impl that *does* override `from_path`, used to verify that
    /// overriding the default works correctly.
    #[derive(Clone, Debug)]
    struct TsxWithFromPath;
    impl Language for TsxWithFromPath {
        fn kind_to_id(&self, kind: &str) -> u16 {
            tsx_kind_to_id(kind)
        }
        fn field_to_id(&self, field: &str) -> Option<u16> {
            tsx_field_to_id(field)
        }
        #[cfg(feature = "matching")]
        fn build_pattern(&self, builder: &PatternBuilder) -> Result<Pattern, PatternError> {
            builder.build(|src| StrDoc::try_new(src, self.clone()))
        }
        fn from_path<P: AsRef<Path>>(path: P) -> Option<Self> {
            path.as_ref()
                .extension()
                .and_then(|e| e.to_str())
                .filter(|&e| e == "tsx")
                .map(|_| Self)
        }
    }
    impl LanguageExt for TsxWithFromPath {
        fn get_ts_language(&self) -> TSLanguage {
            tsx_ts_language()
        }
    }

    #[test]
    #[should_panic(expected = "Language::from_path is not implemented for type")]
    fn default_from_path_panics() {
        let _ = NoFromPath::from_path("some/file.rs");
    }

    #[test]
    fn overridden_from_path_does_not_panic() {
        assert!(TsxWithFromPath::from_path("component.tsx").is_some());
        assert!(TsxWithFromPath::from_path("main.rs").is_none());
    }
}
