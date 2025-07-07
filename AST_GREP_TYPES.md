<!--
SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: AGPL-3.0-or-later
-->

# AST-GREP Types and Tree-Sitter Integration

1. Does ast-grep do its own parsing or just use tree-sitter?
ast-grep relies on tree-sitter for parsing source code into abstract syntax trees (ASTs). It doesn’t implement its own parser, but instead builds tooling and abstractions on top of the tree-sitter ASTs for searching, linting, and rewriting code.

From the README:

ast-grep's core is an algorithm to search and replace code based on abstract syntax tree produced by tree-sitter.

2. What type represents tree-sitter data in ast-grep?
The fundamental type representing the tree-sitter AST in ast-grep is:

Rust
pub type AstGrep<D> = Root<D>;
Root<D> is a struct (defined in node.rs) that wraps a document type.
For tree-sitter-backed documents, the type StrDoc<L> (where L: LanguageExt) is used.
The parsing entry point is via the LanguageExt trait:

Rust
pub trait LanguageExt: Language {
  fn ast_grep<S: AsRef<str>>(&self, source: S) -> AstGrep<StrDoc<Self>>;
  fn get_ts_language(&self) -> TSLanguage;
  // ...
}
LanguageExt::ast_grep() takes source code as input and returns an AstGrep<StrDoc<Self>>, which is essentially a tree-sitter AST wrapped in ast-grep’s abstractions.
3. How is the tree-sitter AST transformed and used?
Here's the high-level flow:

Parsing:

The language definition (implementing LanguageExt) provides the actual tree-sitter grammar.
Code is parsed into a tree-sitter Tree and wrapped in a StrDoc.
This is further wrapped in Root<D>/AstGrep<D> for ast-grep operations.
Node Representation:

The Node<D> type is used to represent specific AST nodes, parameterized by the document type.
Traversal, searching, and rewriting operate over Node<D> (typically Node<StrDoc<L>>).
Traits and Operations:

Traits like Matcher, Pattern, etc., provide searching and matching logic on these nodes.
Traversals (pre-order, post-order, etc.) are done using tree-sitter’s cursor API, but abstracted via ast-grep's APIs.
Rule Application:

Rules (see SerializableRule) define patterns and logic for searching the AST (using patterns or node kinds).
These rules operate on the ast-grep node/document abstractions, not directly on tree-sitter’s raw nodes.
4. Key Types and Traits in core:
AstGrep<D> / Root<D>: Root of the AST for a given document.
StrDoc<L>: Document wrapper around tree-sitter’s parsed tree (where L: LanguageExt).
LanguageExt: Trait for a language, providing parsing and tree-sitter integration.
Node<D>: Represents AST nodes and provides traversal and inspection APIs.
Matcher, Pattern, etc.: Traits/structs for matching and searching nodes.
5. Summary Table
Concept ast-grep Type(s) Underlying tree-sitter? Notes
Language Definition L: LanguageExt Yes Provides grammar and parsing
Parsed AST AstGrep<StrDoc<L>> Yes Wraps parsed tree-sitter tree
AST Node Node<StrDoc<L>> Yes Wraps tree-sitter node, adds APIs
Traversal/Matching Matcher, Pattern, etc. Yes (via ast-grep) High-level search/replace logic
Rule Application SerializableRule Yes (via ast-grep) YAML-defined matching rules
In summary:
ast-grep uses tree-sitter for all parsing; its main type for representing tree-sitter ASTs is AstGrep<StrDoc<L>>. The data is transformed and operated on via the Node abstraction, with traits like LanguageExt managing language-specific parsing and integration. All searching, matching, and rewriting logic is built on these abstractions, not directly on tree-sitter’s raw types.

If you want details on a specific trait or type, let me know!


```rust
// ast-grep-core/src/lib.rs
// these are the core types and traits and mostly what we need
// We don't have an immediate need for the match_tree and matcher modules, but they may be useful later (ok, will be).
pub use language::Language;
pub use match_tree::MatchStrictness;
pub use matcher::{Matcher, NodeMatch, Pattern, PatternError};
pub use node::{Node, Position};
pub use source::Doc;

use node::Root;

pub type AstGrep<D> = Root<D>; // <-- this is the main abstraction for the AST, where D is the document type
