use crate::matcher::Matcher;
use bit_set::BitSet;

pub struct And<P1: Matcher, P2: Matcher> {
  pattern1: P1,
  pattern2: P2,
}

// we pre-compute and cache potential_kinds. So patterns should not be mutated.
// Box<[P]> is used here for immutability so that kinds will never be invalidated.
pub struct All<P: Matcher> {
  patterns: Box<[P]>,
  kinds: Option<BitSet>,
}

// Box<[P]> for immutability and potential_kinds cache correctness
pub struct Any<P> {
  patterns: Box<[P]>,
  kinds: Option<BitSet>,
}

pub struct Or<P1: Matcher, P2: Matcher> {
  pattern1: P1,
  pattern2: P2,
}

pub struct Not<M: Matcher> {
  not: M,
}

#[derive(Clone)]
pub struct Op<M: Matcher> {
  inner: M,
}

pub type NestedAnd<M, N, O> = And<And<M, N>, O>;
pub type NestedOr<M, N, O> = Or<Or<M, N>, O>;
