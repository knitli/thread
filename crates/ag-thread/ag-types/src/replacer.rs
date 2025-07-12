use crate::ast::{Content, Doc};
use crate::meta_var::Underlying;
use crate::matcher::{Matcher, NodeMatch};

/// Replace meta variable in the replacer string
pub trait Replacer<D: Doc> {
  fn generate_replacement(&self, nm: &NodeMatch<'_, D>) -> Underlying<D>;
  fn get_replaced_range(&self, nm: &NodeMatch<'_, D>, matcher: impl Matcher) -> Range<usize> {
    let range = nm.range();
    if let Some(len) = matcher.get_match_len(nm.get_node().clone()) {
      range.start..range.start + len
    } else {
      range
    }
  }
}

/// Represents how we de-indent matched meta var.
pub enum DeindentedExtract<'a, C: Content> {
  /// If meta-var is only one line, no need to de-indent/re-indent
  SingleLine(&'a [C::Underlying]),
  /// meta-var's has multiple lines, may need re-indent
  MultiLine(&'a [C::Underlying], usize),
}

pub enum TemplateFix {
  // no meta_var, pure text
  Textual(String),
  WithMetaVar(Template),
}

#[derive(Debug, Error)]
pub enum TemplateFixError {}
