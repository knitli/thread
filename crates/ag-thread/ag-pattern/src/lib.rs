/*!
This module contains the pattern matching engine for ast-grep.

It provides APIs for pattern matching against AST nodes, including:
- Pattern matching implementations
- Tree matching algorithms
- Meta-variable handling
- Boolean operations support
*/

pub mod match_tree;
mod matcher;
mod meta_var;

pub use match_tree::{MatchStrictness, does_node_match_exactly, match_node_non_recursive, match_end_non_recursive};
pub use matcher::matchers::{KindMatcher, KindMatcherError, Matcher, MatchAll, MatchNone, MatcherExt, NodeMatch, Pattern, PatternError, PatternBuilder, PatternNode, RegexMatcher, RegexMatcherError, kind_utils};
pub use meta_var::metavar::{MetaVarEnv, MetaVariable, MetaVariableID};
