// SPDX-FileCopyrightText: 2025 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: MIT


pub mod error_context;

//* ======================  DynamicLanguage  ======================

#[cfg(all(feature="dynamic-language", not(target_family="wasm")))]
pub use thread_ast_grep::{DynamicLang, DynamicLangError, Registration, LibraryPath, CustomLang};

//* ======================  Language  ======================
#[cfg(all(feature = "language", not(feature="tree-sitter")))]
pub use ast_grep_language::{Language, LanguageExt};

//* =======================================================================
//* ======================  Ast-Grep's Parsed Types  ======================
//* =======================================================================
#[cfg(feature = "meta-var")]
pub use ast_grep_core::meta_var::{MetaVarEnv, MetaVariable, MetaVariableID, Underlying};

// strictly speaking, you don't need tree-sitter for *most* of these types, but it significantly improves their versatility
// TODO: evaluate usage of tree-sitter dependent features after initial release
/// Ast-Grep's core types and traits for AstG-Sitter parsing and tree manipulation and traversal.
///
/// Ast-Grep's core type for the parsed AST. The 'D' is a Node type that implements the `ast_grep_core::source::Doc` trait.
#[cfg(feature = "tree-sitter")]
pub type AstG<D> = ast_grep_core::AstGrep<D>;

#[cfg(feature = "tree-sitter")]
pub use ast_grep_core::{language::Language, {Node, Position, source::{Content, Doc, Edit}}};

//* ======================  Tree-Sitter Transversal  ======================
#[cfg(feature = "tree-sitter")]
pub use ast_grep_core::tree_sitter::{
    DisplayContext, LanguageExt, StrDoc, TSLanguage, TSPoint, TSRange, Visitor,
};

//* ======================  Matcher Types  ======================

#[cfg(feature = "matcher")]
/// Ast-Grep's core types and traits for pattern matching and searching in the AST.
pub use ast_grep_core::{
    MatchStrictness, Matcher, Pattern, PatternError,
    matcher::{
        KindMatcher, KindMatcherError, MatchAll, MatchNone, MatcherExt, NodeMatch,
        PatternBuilder, PatternNode, RegexMatcher, RegexMatcherError, kind_utils
    },
    ops::{
        All, And, Any, Not, Op, Or
    },
};

//* ======================  Replacer Types  ======================
#[cfg(feature = "replacer")]

/// Ast-Grep's core types and traits for replacing patterns in the AST.
pub use ast_grep_core::replacer::{Replacer, TemplateFix, TemplateFixError};


//* ======================  DynamicLanguage  ======================
#[cfg(feature = "ag-config")]
pub use ast_grep_config::{
    CombinedScan, Fixer, Label, LabelStyle, GlobalRules, DeserializeEnv, Rule, RuleSerializeError, SerializableRule, RuleCollection, Severity, RuleConfig, RuleConfigError, SerializableRuleConfig, Metadata,RuleCore, RuleCoreError, SerializableRuleCore, Transformation, from_str, from_yaml_string,
};
