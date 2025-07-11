pub use tree_sitter::{Language as TSLanguage, Point as TSPoint, Range as TSRange, InputEdit as TSInputEdit, Tree as TSTree, Node as TSNode, LanguageError as TSLanguageError, Parser as TSParser, TreeCursor as TSTreeCursor, Point as TSPoint, Range as TSRange};
pub use tree_sitter::LanguageExt;
/// Represents tree-sitter related error
#[derive(Debug, Error)]
pub enum TSParseError {
  #[error("incompatible `Language` is assigned to a `Parser`.")]
  Language(#[from] LanguageError),
  /// A general error when tree sitter fails to parse in time. It can be caused by
  /// the following reasons but tree-sitter does not provide error detail.
  /// * The timeout set with [Parser::set_timeout_micros] expired
  /// * The cancellation flag set with [Parser::set_cancellation_flag] was flipped
  /// * The parser has not yet had a language assigned with [Parser::set_language]
  #[error("general error when tree-sitter fails to parse.")]
  TreeUnavailable,
}

pub trait ContentExt: Content {
  fn accept_edit(&mut self, edit: &Edit<Self>) -> InputEdit;
}

pub struct DisplayContext<'r> {
  /// content for the matched node
  pub matched: Cow<'r, str>,
  /// content before the matched node
  pub leading: &'r str,
  /// content after the matched node
  pub trailing: &'r str,
  /// zero-based start line of the context
  pub start_line: usize,
}

/// tree-sitter specific language trait
pub trait LanguageExt: Language {
  /// Create an [`AstGrep`] instance for the language
  fn ast_grep<S: AsRef<str>>(&self, source: S) -> AstGrep<StrDoc<Self>> {
    AstGrep::new(source, self.clone())
  }

  /// tree sitter language to parse the source
  fn get_ts_language(&self) -> TSLanguage;

  fn injectable_languages(&self) -> Option<&'static [&'static str]> {
    None
  }

  /// get injected language regions in the root document. e.g. get JavaScripts in HTML
  /// it will return a list of tuples of (language, regions).
  /// The first item is the embedded region language, e.g. javascript
  /// The second item is a list of regions in tree_sitter.
  /// also see https://tree-sitter.github.io/tree-sitter/using-parsers#multi-language-documents
  fn extract_injections<L: LanguageExt>(
    &self,
    _root: crate::Node<StrDoc<L>>,
  ) -> FastMap<String, Vec<TSRange>> {
    FastMap::new()
  }
}

#[derive(Clone)]
pub struct StrDoc<L: LanguageExt> {
  pub src: String,
  pub lang: L,
  pub tree: Tree,
}
