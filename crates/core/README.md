<!--
SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: AGPL-3.0-or-later
-->

# Thread Core

This crate provides the core types and traits for Thread. There are a lot of feature flags available to customize its behavior; this lets you pick and choose exactly what you need for your application or use case. We designed thread to be modular and extensible, so you can use it for any environment.

We'll mostly discuss these features so you can understand how to use them.

## Features

### `Ast-Grep Re-Exports`

Thread's built on top of [Ast-Grep](https://github.com/ast-grep/ast-grep), and uses its core language and parsing features. Thread diverges from Ast-Grep after information is parsed, but it was clearly the best choice for providing a solid foundation for Thread with a very flexible and powerful API. It brings AstG-Sitter into the Rust fold, it's battle tested, and it has a great community behind it.

Thread Core re-exports the major Ast-Grep types and traits for the rest of the Thread library to use. While Ast-Grep provides a foundation for all of Thread, you'll only see it imported here in Thread Core. We've divided these types and traits into multiple feature flags (we'll cover the major types; not every type is listed here):

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
  - `SupportLang` - An Enum that provides a list of supported languages, and methods to identify them (i.e. extensions).

- `matcher`: Provides the `Matcher` and `Pattern` traits. Ast-Grep provides robust pattern matching capabilities that let you query the AST using a combination of node types, regex patterns, and contextual relationships.

- `replacer`: Provides the `Replacer` trait, which allows you to replace nodes in the AST with new content. This is useful for transforming the AST or modifying the document's content.

- `all-ast-grep`: This feature flag enables all of the above features, providing a complete Ast-Grep experience. It is enabled by default.

### Utility Features

Thread Core provides several utility-oriented features that provide types that improve performance in certain settings, or provide additional functionality that is not strictly necessary for the core functionality of Thread. These features are:

- `fastmap`: Provides a `FastMap` type that either provides a `DashMap`  (and `DashSet`) for blazing concurrent access, or a std `HashMap` (and `HashSet`) for single-threaded access. This is useful for caching and storing data that needs to be accessed quickly and concurrently. For it to provide a `Dashmap`, the `dashmap` feature must *also* be enabled. Both are enabled by default.
  - Note: Regardless of flags you pass, the `FastMap` will always revert to a `HashMap` and `HashSet` if the `wasm-single-thread` feature is enabled. This saves on the size of the binary, and is primarily intended for Cloudflare Workers and similar cloud WASM environments that do not support multi-threading. You *can* use `DashMap` in other WASM environments like browsers or `WASI`.
- `inline` and `dash-inline`: Enables more aggressive inlining of functions and methods, which can improve performance in some cases at the expense of longer compile times. The `dash-inline` feature enables inlining for `DashMap` and `DashSet` methods, while `inline` enables it for both DashMap and `string-interner`. `inline` is not default, but we do use it in our prebuilt release binaries (github releases).
