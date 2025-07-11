#[derive(Clone)]
pub enum MatchStrictness {
  Cst,       // all nodes are matched
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
