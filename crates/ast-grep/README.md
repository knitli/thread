<!--
SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Thread Ast-Grep (thread-ast-grep)

This is a convenience crate that re-exports core types and traits from Ast-Grep, and provides narrower feature gates to these features for the rest of Thread.

We'll mostly discuss these features so you can understand how to use them.

## Features

### `Ast-Grep Re-Exports`

Thread's built on top of [Ast-Grep](https://github.com/ast-grep/ast-grep), and uses its core language and parsing features. Thread diverges from Ast-Grep after information is parsed, but it was clearly the best choice for providing a solid foundation for Thread with a very flexible and powerful API. It brings AstG-Sitter into the Rust fold, it's battle tested, and it has a great community behind it.

Thread Ast-Grep re-exports the major Ast-Grep types and traits for the rest of the Thread library to use. While Ast-Grep provides a foundation for all of Thread, you'll only see it imported here in Thread Ast-Grep. We've divided these types and traits into multiple feature flags (we'll cover the major types; not every type is listed here):

- `tree-sitter`: So-named because it requires the `tree-sitter` dependency, these types and traits provide *most* of the functionality Ast-Grep provides. Enabled by default. It includes:

  - `AstG` - Our re-export of Ast-Grep's [`AstGrep`](https://docs.rs/ast-grep-core/latest/ast_grep_core/type.AstGrep.html) core type. `AstG` is a wrapper around a `StrDoc` type, which is where the real work happens.
  - `StrDoc` - Our re-export of Ast-Grep's [`StrDoc`](https://docs.rs/ast-grep-core/latest/ast_grep_core/tree_sitter/struct.StrDoc.html) type. This is a wrapper around a source code string, the typed and parsed language of the document, and the parsed AST tree.

    ```rust
    #[derive(Clone)]
    pub struct StrDoc<L: LanguageExt> {
        pub src: String, // The raw source code of the document.
        pub lang: L,  // The language of the document, implementing the `LanguageExt` trait. (the type is the language, e.g. `JavaScript`, `Python`, etc.)
        pub tree: Tree, // The tree-sitter parsed AST of the document.
    }
    ```

      Functionally, Ast-Grep identifies the language of a document, uses the appropriate language parser to parse it, using a tree-sitter language parser implementing [`LanguageExt`](https://docs.rs/ast-grep-core/latest/ast_grep_core/tree_sitter/trait.LanguageExt.html), and the parsed language instance calls its own `ast_grep()` method to return an `AstGrep` instance.

  - `Node` - Is a typed language-agnostic AST node type that wraps a tree-sitter `Node` type. It provides methods to access the node's properties. Functionally, this is mostly what you would query from a parsed AST. They can be arbitrarily nested. A node:
    - Represents a single node in the tree-sitter AST of a file.
    - Knows its position, kind, text, and has methods for traversal, matching, and manipulation.
      - Its start and end positions are represented as `Position` types, which are structs with `line`, `byte_column`, and `byte_offset` fields.
    - Is "typed" in the sense that it is associated with the language and the underlying document (via `StrDoc<L>`).
  - `Content`, `Doc`, and `Edit` - are the core traits that give StrDoc its functionality for manipulating a document, abstracting away the underlying encoding and parsing details. They provide methods for:
    - Getting and setting the document's content.
    - Editing the document's content.
    - Querying the document's AST.

- `language`: This flag provides language-specific types and traits for Thread. It includes:

  - `LanguageExt` - A trait that provides methods for parsing a document in a specific language, returning an `AstG` instance. It also may implement `extract_injections()` to extract language-specific injections from the document (like JavaScript in html `<script>` tags). You would need to handle the output and parse it into its own set of `StrDoc` instances.
  - `Language` - A trait that provides methods for working with a specific language, such as getting the language's name, version, and parser.
  - `SupportedLanguage` - An Enum that provides a list of supported languages, and methods to identify them (i.e. extensions).

- `matcher`: Provides the `Matcher` and `Pattern` traits. Ast-Grep provides robust pattern matching capabilities that let you query the AST using a combination of node types, regex patterns, and contextual relationships.

- `replacer`: Provides the `Replacer` trait, which allows you to replace nodes in the AST with new content. This is useful for transforming the AST or modifying the document's content.

- `all-core`: This feature flag enables `tree-sitter`, `matcher`, `replacer`, and `meta-var`, providing a complete Ast-Grep core experience. It is enabled by default.
