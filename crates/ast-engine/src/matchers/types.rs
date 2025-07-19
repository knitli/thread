//! Types for Pattern and Pattern matching.
//!
//! Definitions for the globally important pattern matching types.
//! Allows their use outside the pattern matching feature flags (unimplemented).

use thiserror::Error;
use bit_set::BitSet;
use crate::Doc;
use crate::MetaVarEnv;
use crate::node::Node;
use std::borrow::Cow;
use crate::meta_var::MetaVariable;

<<<<<<< Updated upstream
||||||| Stash base
#[derive(Clone)]
=======
pub trait Matcher {
  /// Returns the node why the input is matched or None if not matched.
  /// The return value is usually input node itself, but it can be different node.
  /// For example `Has` matcher can return the child or descendant node.
  fn match_node_with_env<'tree, D: Doc>(
    &self,
    _node: Node<'tree, D>,
    _env: &mut Cow<MetaVarEnv<'tree, D>>,
  ) -> Option<Node<'tree, D>>;

  /// Returns a bitset for all possible target node kind ids.
  /// Returns None if the matcher needs to try against all node kind.
  fn potential_kinds(&self) -> Option<BitSet> {
    None
  }

  /// get_match_len will skip trailing anonymous child node to exclude punctuation.
  // This is not included in NodeMatch since it is only used in replace
  fn get_match_len<D: Doc>(&self, _node: Node<'_, D>) -> Option<usize> {
    None
  }
}

>>>>>>> Stashed changes
#[derive(Clone, Debug)]
pub enum MatchStrictness {
  Cst,       // all nodes are matched
  Smart,     // all nodes except source trivial nodes are matched.
  Ast,       // only ast nodes are matched
  Relaxed,   // ast-nodes excluding comments are matched
  Signature, // ast-nodes excluding comments, without text
}

#[derive(Clone)]
pub struct Pattern {
    pub node: PatternNode,
    pub(crate) root_kind: Option<u16>,
    pub strictness: MatchStrictness,
}

#[derive(Clone, Debug)]
pub struct PatternBuilder<'a> {
    pub(crate) selector: Option<&'a str>,
    pub(crate) src: Cow<'a, str>,
}

#[derive(Clone)]
pub enum PatternNode {
    MetaVar {
        meta_var: MetaVariable,
    },
    /// Node without children.
    Terminal {
        text: String,
        is_named: bool,
        kind_id: u16,
    },
    /// Non-Terminal Syntax Nodes are called Internal
    Internal {
        kind_id: u16,
        children: Vec<PatternNode>,
    },
}

#[derive(Debug, Error)]
pub enum PatternError {
    #[error("Fails to parse the pattern query: `{0}`")]
    Parse(String),
    #[error("No AST root is detected. Please check the pattern source `{0}`.")]
    NoContent(String),
    #[error("Multiple AST nodes are detected. Please check the pattern source `{0}`.")]
    MultipleNode(String),
    #[error(transparent)]
    #[cfg(feature = "matching")]
    InvalidKind(#[from] super::kind::KindMatcherError),
    #[error(
        "Fails to create Contextual pattern: selector `{selector}` matches no node in the context `{context}`."
    )]
    NoSelectorInContext { context: String, selector: String },
}
