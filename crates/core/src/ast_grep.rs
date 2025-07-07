// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

/// Re-export Ast-Grep's core types and traits
//* ======================  Language  ======================
#[cfg(feature = "language")]
pub mod language {

    pub use ast_grep_language::{Language, LanguageExt, SupportLang, SupportLangErr};
}

//* ======================  Ast-Grep's Parsed Types  ======================
#[cfg(feature = "meta-var")]
pub use ast_grep_core::meta_var::{MetaVarEnv, MetaVariable, MetaVariableID, Underlying};

// strictly speaking, you don't need tree-sitter for *most* of these types, but it significantly improves their versatility
// TODO: evaluate usage of tree-sitter dependent features after initial release
#[cfg(feature = "tree-sitter")]
pub mod tree_sitter {
    /// Ast-Grep's core types and traits for AstG-Sitter parsing and tree manipulation and traversal.

    /// Ast-Grep's core type for the parsed AST. The 'D' is a Node type that implements the `ast_grep_core::source::Doc` trait.
    pub type AstG<D> = ast_grep_core::AstGrep<D>;

    pub use ast_grep_core::{Node, Position};

    pub use ast_grep_core::source::{Content, Doc, Edit};

    //* ======================  AstG-Sitter Transversal  ======================

    pub use ast_grep_core::tree_sitter::{
        DisplayContext, StrDoc, TSLanguage, TSPoint, TSRange, Visitor,
    };
}

//* ======================  Matcher Types  ======================

#[cfg(feature = "matcher")]
pub mod matcher {
    /// Ast-Grep's core types and traits for pattern matching and searching in the AST.
    pub use ast_grep_core::{
        MatchStrictness, Matcher, Pattern, PatternError,
        matcher::{
            KindMatcher, KindMatcherError, MatchAll, MatchNone, MatcherExt, NodeMatch,
            PatternBuilder, PatternNode, RegexMatcher, RegexMatcherError, kind_utils,
        },
    };

    pub use ast_grep_core::ops::{All, And, Any, Not, Op, Or};
}

//* ======================  Replacer Types  ======================
#[cfg(feature = "replacer")]
pub mod replacer {
    /// Ast-Grep's core types and traits for replacing patterns in the AST.
    pub use ast_grep_core::replacer::{Replacer, TemplateFix, TemplateFixError};
}
