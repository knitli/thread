use ag_service_types::{Matcher, Doc, Node, MetaVarEnv};

use bit_set::BitSet;
use regex::{Error as RegexError, Regex};
use thiserror::Error;

use std::borrow::Cow;

impl RegexMatcher {
  pub fn try_new(text: &str) -> Result<Self, RegexMatcherError> {
    Ok(RegexMatcher {
      regex: Regex::new(text)?,
    })
  }
}

impl Matcher for RegexMatcher {
  fn match_node_with_env<'tree, D: Doc>(
    &self,
    node: Node<'tree, D>,
    _env: &mut Cow<MetaVarEnv<'tree, D>>,
  ) -> Option<Node<'tree, D>> {
    self.regex.is_match(&node.text()).then_some(node)
  }

  fn potential_kinds(&self) -> Option<BitSet> {
    None
  }
}
