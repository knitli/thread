pub mod error;

mod ast_grep;

/// Re-export Ast-Grep's core types and traits
#[cfg(feature = "language")]
pub use crate::ast_grep::language::{Language, LanguageExt, SupportLang, SupportLangErr};

#[cfg(feature = "tree-sitter")]
pub use crate::ast_grep::tree_sitter::{Tree, Node, NodeMatch, Position, Content, Doc, Edit, DisplayContext, TSPre, Visitor, TSLanguage, TSPoint, TSRange, StrDoc};

// general ast-grep types with no other dependencies
#[cfg(feature = "ast-grep")]
pub use crate::ast_grep::{MetaVarEnv, MetaVariable, MetaVariableID, Underlying};

#[cfg(feature = "matcher")]
pub use crate::ast_grep::matcher::{MatchStrictness, Matcher, Pattern, PatternError, NodeMatch, KindMatcher, KindMatcherError, MetaVarMatcher, PatternMatcher, PatternMatcherError, kind_utils, RegexMatcher, RegexMatcherError, PatternNode, MatchNone, MatchAll, And, All, Any, Or, Not, Op};

#[cfg(feature = "replacer")]
pub use crate::ast_grep::replacer::{Replacer, TemplateFix, TemplateFixError};
