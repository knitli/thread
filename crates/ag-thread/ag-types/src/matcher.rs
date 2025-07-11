use crate::ast::{Doc, Node}
use bit_set::BitSet;

/// `Matcher` defines whether a tree-sitter node matches certain pattern,
/// and update the matched meta-variable values in `MetaVarEnv`.
/// N.B. At least one positive term is required for matching
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

/// MatcherExt provides additional utility methods for `Matcher`.
/// It is implemented for all types that implement `Matcher`.
/// N.B. This trait is not intended to be implemented by users.
pub trait MatcherExt: Matcher {
  fn match_node<'tree, D: Doc>(&self, node: Node<'tree, D>) -> Option<NodeMatch<'tree, D>> {
    // in future we might need to customize initial MetaVarEnv
    let mut env = Cow::Owned(MetaVarEnv::new());
    let node = self.match_node_with_env(node, &mut env)?;
    Some(NodeMatch::new(node, env.into_owned()))
  }

  fn find_node<'tree, D: Doc>(&self, node: Node<'tree, D>) -> Option<NodeMatch<'tree, D>> {
    for n in node.dfs() {
      if let Some(ret) = self.match_node(n.clone()) {
        return Some(ret);
      }
    }
    None
  }
}

pub struct MatchAll;
pub struct MatchNone;

#[derive(Debug, Error)]
pub enum KindMatcherError {
  #[error("Kind `{0}` is invalid.")]
  InvalidKindName(String),
}

#[derive(Clone)]
pub struct KindMatcher {
  kind: KindId,
}

/// Represents the matched node with populated MetaVarEnv.
/// It derefs to the Node so you can use it as a Node.
/// To access the underlying MetaVarEnv, call `get_env` method.
#[derive(Clone)]
pub struct NodeMatch<'t, D: Doc>(Node<'t, D>, MetaVarEnv<'t, D>);

#[derive(Clone)]
pub struct Pattern {
  pub node: PatternNode,
  root_kind: Option<u16>,
  pub strictness: MatchStrictness,
}

pub struct PatternBuilder<'a> {
  selector: Option<&'a str>,
  src: Cow<'a, str>,
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
  #[error("Fails to create Contextual pattern: selector `{selector}` matches no node in the context `{context}`.")]
  NoSelectorInContext { context: String, selector: String },
}

#[derive(Debug, Error)]
pub enum RegexMatcherError {
  #[error("Parsing text matcher fails.")]
  Regex(#[from] RegexError),
}

#[derive(Clone)]
pub struct RegexMatcher {
  regex: Regex,
}
