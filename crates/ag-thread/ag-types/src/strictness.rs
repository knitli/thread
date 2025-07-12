#[derive(Clone, Default)]
pub enum MatchStrictness {
  Cst,       // all nodes are matched
  #[default]
  Smart,     // all nodes except source trivial nodes are matched.
  Ast,       // only ast nodes are matched
  Relaxed,   // ast-nodes excluding comments are matched
  Signature, // ast-nodes excluding comments, without text
}

pub enum MatchOneNode {
  MatchedBoth,
  SkipBoth,
  SkipGoal,
  SkipCandidate,
  NoMatch,
}
