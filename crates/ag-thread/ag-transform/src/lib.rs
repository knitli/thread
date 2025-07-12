/*!
This module contains the code transformation engine for ast-grep.

It provides APIs for replacing and modifying code, including:
- Code replacement logic
- Template-based transformations
- Structural replacements
- Indentation handling
*/

mod indent;
mod structural;
mod template;
use ag_service_types::{Doc, Replacer, MetaVariable, MetaVarExtract, MetaVariableID, Underlying};
use ag_service_pattern::NodeMatch;
pub use indent::{Indentation, IndentationError};
pub use structural::{StructuralReplacement, StructuralReplacementError};
pub use template::{TemplateFix, TemplateFixError};
// Re-export implemented Replacer
pub use Replacer;

impl<D: Doc> Replacer<D> for str {
  fn generate_replacement(&self, nm: &NodeMatch<'_, D>) -> Underlying<D> {
    template::gen_replacement(self, nm)
  }
}

impl<D: Doc> Replacer<D> for Root<D> {
  fn generate_replacement(&self, nm: &NodeMatch<'_, D>) -> Underlying<D> {
    structural::gen_replacement(self, nm)
  }
}

impl<D, T> Replacer<D> for &T
where
  D: Doc,
  T: Replacer<D> + ?Sized,
{
  fn generate_replacement(&self, nm: &NodeMatch<D>) -> Underlying<D> {
    (**self).generate_replacement(nm)
  }
}

impl<D: Doc> Replacer<D> for Node<'_, D> {
  fn generate_replacement(&self, _nm: &NodeMatch<'_, D>) -> Underlying<D> {
    let range = self.range();
    self.root.doc.get_source().get_range(range).to_vec()
  }
}

impl MetaVarExtract {
  fn used_var(&self) -> &str {
    match self {
      MetaVarExtract::Single(s) => s,
      MetaVarExtract::Multiple(s) => s,
      MetaVarExtract::Transformed(s) => s,
    }
  }
}

fn split_first_meta_var(
  src: &str,
  meta_char: char,
  transform: &[MetaVariableID],
) -> Option<(MetaVarExtract, usize)> {
  debug_assert!(src.starts_with(meta_char));
  let mut i = 0;
  let mut skipped = 0;
  let is_multi = loop {
    i += 1;
    skipped += meta_char.len_utf8();
    if i == 3 {
      break true;
    }
    if !src[skipped..].starts_with(meta_char) {
      break false;
    }
  };
  // no Anonymous meta var allowed, so _ is not allowed
  let i = src[skipped..]
    .find(|c: char| !is_valid_meta_var_char(c))
    .unwrap_or(src.len() - skipped);
  // no name found
  if i == 0 {
    return None;
  }
  let name = src[skipped..skipped + i].to_string();
  let var = if is_multi {
    MetaVarExtract::Multiple(name)
  } else if transform.contains(&name) {
    MetaVarExtract::Transformed(name)
  } else {
    MetaVarExtract::Single(name)
  };
  Some((var, skipped + i))
}
