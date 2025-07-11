use crate::ast::{Doc, Content, Node};
use thread_utils::FastMap;

pub type MetaVariableID = String;

pub type Underlying<D> = Vec<<<D as Doc>::Source as Content>::Underlying>;

/// a dictionary that stores metavariable instantiation
/// const a = 123 matched with const a = $A will produce env: $A => 123
#[derive(Clone)]
pub struct MetaVarEnv<'tree, D: Doc> {
  single_matched: FastMap<MetaVariableID, Node<'tree, D>>,
  multi_matched: FastMap<MetaVariableID, Vec<Node<'tree, D>>>,
  transformed_var: FastMap<MetaVariableID, Underlying<D>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetaVariable {
  /// $A for captured meta var
  Capture(MetaVariableID, bool),
  /// $_ for non-captured meta var
  Dropped(bool),
  /// $$$ for non-captured multi var
  Multiple,
  /// $$$A for captured ellipsis
  MultiCapture(MetaVariableID),
}


pub enum MetaVarExtract {
  /// $A for captured meta var
  Single(MetaVariableID),
  /// $$$A for captured ellipsis
  Multiple(MetaVariableID),
  Transformed(MetaVariableID),
}
