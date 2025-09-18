<!--
SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: MIT OR Apache-2.0
-->
# thread-ast-engine

**Core AST engine for Thread: parsing, matching, and transforming code using AST patterns.**

## Overview

`thread-ast-engine` provides powerful tools for working with Abstract Syntax Trees (ASTs). Forked from [`ast-grep-core`](https://github.com/ast-grep/ast-grep/), it offers language-agnostic APIs for code analysis and transformation.

### What You Can Do

- **Parse** source code into ASTs using [tree-sitter](https://tree-sitter.github.io/tree-sitter/)
- **Search** for code patterns using flexible meta-variables (like `$VAR`)
- **Transform** code by replacing matched patterns with new code
- **Navigate** AST nodes with intuitive tree traversal methods

Perfect for building code linters, refactoring tools, and automated code modification systems.

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
thread-ast-engine = { version = "0.1.0", features = ["parsing", "matching", "replacing"] }
```

### Basic Example: Find and Replace Variables

```rust
use thread_ast_engine::Language;
use thread_ast_engine::tree_sitter::LanguageExt;

// Parse JavaScript/TypeScript code
let mut ast = Language::Tsx.ast_grep("var a = 1; var b = 2;");

// Replace all 'var' declarations with 'let'
ast.replace("var $NAME = $VALUE", "let $NAME = $VALUE")?;

// Get the transformed code
println!("{}", ast.generate());
// Output: "let a = 1; let b = 2;"
```

### Finding Code Patterns

```rust
use thread_ast_engine::matcher::MatcherExt;

let ast = Language::Tsx.ast_grep("function add(a, b) { return a + b; }");
let root = ast.root();

// Find all function declarations
if let Some(func) = root.find("function $NAME($$$PARAMS) { $$$BODY }") {
    println!("Function name: {}", func.get_env().get_match("NAME").unwrap().text());
}

// Find all return statements
for ret_stmt in root.find_all("return $EXPR") {
    println!("Returns: {}", ret_stmt.get_env().get_match("EXPR").unwrap().text());
}
```

### Working with Meta-Variables

Meta-variables capture parts of the matched code:

- `$VAR` - Captures a single AST node
- `$$$ITEMS` - Captures multiple consecutive nodes (ellipsis)
- `$_` - Matches any node but doesn't capture it

```rust
let ast = Language::Tsx.ast_grep("console.log('Hello', 'World', 123)");
let root = ast.root();

if let Some(call) = root.find("console.log($$$ARGS)") {
    let args = call.get_env().get_multiple_matches("ARGS");
    println!("Found {} arguments", args.len()); // Output: Found 3 arguments
}
```

## Core Components

### [`Node`](src/node.rs) - AST Navigation

Navigate and inspect AST nodes with methods like `children()`, `parent()`, and `find()`.

### [`Pattern`](src/matchers/pattern.rs) - Code Matching

Match code structures using tree-sitter patterns with meta-variables.

### [`MetaVarEnv`](src/meta_var.rs) - Variable Capture

Store and retrieve captured meta-variables from pattern matches.

### [`Replacer`](src/replacer.rs) - Code Transformation

Replace matched code with new content, supporting template-based replacement.

### [`Language`](src/language.rs) - Language Support

Abstract interface for different programming languages via tree-sitter grammars.

## Feature Flags

- **`parsing`** - Enables tree-sitter parsing (includes tree-sitter dependency)
- **`matching`** - Enables pattern matching and node transformation engine.

Use `default-features = false` to opt out of all features and enable only what you need:

```toml
[dependencies]
thread-ast-engine = { version = "0.1.0", default-features = false, features = ["matching"] }
```

## Advanced Examples

### Custom Pattern Matching

```rust
use thread_ast_engine::ops::Op;

// Combine multiple patterns with logical operators
let pattern = Op::either("let $VAR = $VALUE")
    .or("const $VAR = $VALUE")
    .or("var $VAR = $VALUE");

let ast = Language::Tsx.ast_grep("const x = 42;");
let root = ast.root();

if let Some(match_) = root.find(pattern) {
    println!("Found variable declaration");
}
```

### Tree Traversal

```rust
let ast = Language::Tsx.ast_grep("if (condition) { doSomething(); } else { doOther(); }");
let root = ast.root();

// Traverse all descendants
for node in root.dfs() {
    if node.kind() == "identifier" {
        println!("Identifier: {}", node.text());
    }
}

// Check relationships between nodes
if let Some(if_stmt) = root.find("if ($COND) { $$$THEN }") {
    println!("If statement condition: {}",
        if_stmt.get_env().get_match("COND").unwrap().text());
}
```

## License

Original ast-grep code is licensed under the [MIT license](./LICENSE-MIT). All changes introduced in this project are licensed under [AGPL-3.0-or-later](./LICENSE-AGPL-3.0-or-later).

See [`VENDORED.md`](VENDORED.md) for details about our fork, changes, and licensing.
