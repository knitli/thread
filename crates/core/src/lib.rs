// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod error;

mod ast_grep;

/// Re-export Ast-Grep's core types and traits
#[cfg(feature = "language")]
pub use crate::ast_grep::language::{Language, LanguageExt, SupportLang, SupportLangErr};

#[cfg(feature = "tree-sitter")]
pub use crate::ast_grep::tree_sitter::{Tree, Node, Position, Content, Doc, Edit, DisplayContext, Visitor, TSLanguage, TSPoint, TSRange, StrDoc};

// general ast-grep types with no other dependencies
#[cfg(feature = "ast-grep")]
pub use crate::ast_grep::{MetaVarEnv, MetaVariable, MetaVariableID, Underlying};

#[cfg(feature = "matcher")]
pub use crate::ast_grep::matcher::{MatchStrictness, Matcher, Pattern, PatternBuilder, PatternError, KindMatcher, KindMatcherError, MatcherExt, NodeMatch, kind_utils, RegexMatcher, RegexMatcherError, PatternNode, MatchNone, MatchAll, And, All, Any, Or, Not, Op};

#[cfg(feature = "replacer")]
pub use crate::ast_grep::replacer::{Replacer, TemplateFix, TemplateFixError};
