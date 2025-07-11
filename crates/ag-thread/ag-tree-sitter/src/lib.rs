/*!
This module contains the tree-sitter parsing backend for ast-grep.

It provides APIs for tree-sitter integration, including:
- Tree-sitter specific implementations
- Language extensions for tree-sitter
- Document parsing and editing
- Traversal optimizations
*/

mod tree_sitter;
pub use tree_sitter::Ts::{AstGrep, ContentExt, StrDoc, Level, Post, Pre, Visit, Visitor, TsPre, TSLanguage, TSPoint, TSRange, TSTree, TSNode, TSInputEdit, TSLanguageError, TSParseError, TSParser, LanguageExt}
