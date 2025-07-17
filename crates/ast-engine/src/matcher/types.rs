//! Types for Pattern and Pattern matching.
//!
//! Definitions for the globally important pattern matching types.
//! Allows their use outside the pattern matching feature flags (unimplemented).

use super::kind::{KindMatcherError};
use thiserror::Error;
use std::borrow::Cow;
use crate::meta_var::MetaVariable;

#[derive(Clone)]
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
    InvalidKind(#[from] KindMatcherError),
    #[error(
        "Fails to create Contextual pattern: selector `{selector}` matches no node in the context `{context}`."
    )]
    NoSelectorInContext { context: String, selector: String },
}
