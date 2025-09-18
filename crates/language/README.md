<!--
SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# thread-language

Language definitions and tree-sitter parsers for the Thread AST analysis toolkit.

## Overview

`thread-language` provides unified language support for AST-based code analysis and transformation. Built on tree-sitter grammars, it implements consistent Rust traits ([`Language`](src/lib.rs) and [`LanguageExt`](src/lib.rs)) across 24+ programming languages.

This crate is a fork of `ast-grep-language`, enhanced with improved performance, better feature organization, and streamlined language detection.

## Supported Languages

The crate supports two categories of languages:

### Languages with Custom Pattern Processing

These languages require special handling for metavariables because they don't accept `$` as a valid identifier character:

- **C/C++** - Uses `µ` as expando character
- **C#** - Uses `µ` as expando character
- **CSS** - Uses `_` as expando character
- **Elixir** - Uses `µ` as expando character
- **Go** - Uses `µ` as expando character
- **Haskell** - Uses `µ` as expando character
- **HTML** - Uses `z` as expando character with injection support
- **Kotlin** - Uses `µ` as expando character
- **PHP** - Uses `µ` as expando character
- **Python** - Uses `µ` as expando character
- **Ruby** - Uses `µ` as expando character
- **Rust** - Uses `µ` as expando character
- **Swift** - Uses `µ` as expando character

### Standard Languages

These languages accept `$` in identifiers and use standard pattern processing:

- **Bash**
- **Java**
- **JavaScript**
- **JSON**
- **Lua**
- **Scala**
- **TypeScript**
- **TSX**
- **YAML**

## Features

### Core Features

- **Unified API** - All languages implement the same [`Language`](src/lib.rs) and [`LanguageExt`](src/lib.rs) traits
- **Pattern Matching** - Advanced AST pattern matching with metavariable support.
  - Requires `matching` feature flag (enabled by default).
- **Language Detection** - Automatic language detection from file extensions
- **Fast Parser Access** - Cached tree-sitter parsers for zero-cost repeated access
- **Injection Support** - Extract embedded languages (JavaScript in HTML, CSS in HTML)
  - Requires `html-embedded` flag.

### Feature Flags

- **`matching`** - Enables advanced AST pattern matching and replacement with metavariable support.

#### Parser Groups

- **`all-parsers`** (default) - Includes all language parsers
- **`napi-environment`** - Includes only NAPI-compatible (WASM for Node.js environments) parsers (CSS, HTML, JavaScript, TypeScript)

#### Individual Languages

Each language can be enabled individually:

```toml
[dependencies]
thread-language = { version = "0.1", default-features = false, features = ["rust", "javascript"] }
```

Available language features:

- `bash`, `c`, `cpp`, `csharp`, `css`, `elixir`, `go`, `haskell`, `html`, `html-embedded`
- `java`, `javascript`, `json`, `kotlin`, `lua`, `php`, `python`
- `ruby`, `rust`, `scala`, `swift`, `typescript`, `tsx`, `yaml`

## Usage

### Basic Language Detection

```rust
use thread_language::SupportLang;
use std::path::Path;

// Detect language from file extension
let lang = SupportLang::from_path("main.rs").unwrap();
assert_eq!(lang, SupportLang::Rust);

// Parse from string
let lang: SupportLang = "javascript".parse().unwrap();
```

### Pattern Matching

```rust
use thread_language::{Rust, SupportLang};
use thread_ast_engine::{Language, LanguageExt};

// Using specific language type
let rust = Rust;
let source = "fn main() { println!('Hello'); }";
let tree = rust.ast_grep(source);

// Using enum for runtime language selection
let lang = SupportLang::Rust;
let tree = lang.ast_grep(source);
```

### Working with Metavariables

For languages that don't support `$` in identifiers, the crate automatically handles pattern preprocessing:

```rust
use thread_language::Python;

let python = Python;
// Pattern uses $ for metavariables
let pattern = "def $FUNC($ARGS): $BODY";
// Automatically converted to use µ internally
let processed = python.pre_process_pattern(pattern);
```

### HTML with Embedded Languages

```rust
use thread_language::Html;

let html = Html;
let source = r#"
<script>console.log('hello');</script>
<style>.class { color: red; }</style>
"#;

let tree = html.ast_grep(source);
let injections = html.extract_injections(tree.root());
// injections contains JavaScript and CSS code ranges
```

## Architecture

### Core Modules

- [`lib.rs`](src/lib.rs) - Main module with language definitions and [`SupportLang`](src/lib.rs) enum
- [`parsers.rs`](src/parsers.rs) - Tree-sitter parser initialization and caching
- [`html.rs`](src/html.rs) - Special HTML implementation with language injection support

### Language Implementation Patterns

The crate uses two macros to implement languages:

1. **`impl_lang!`** - For standard languages that accept `$` in identifiers
2. **`impl_lang_expando!`** - For languages requiring custom expando characters

Both macros generate the same [`Language`](src/lib.rs) and [`LanguageExt`](src/lib.rs) trait implementations but with different pattern preprocessing behavior.

## Performance

- **Cached Parsers** - Tree-sitter languages are initialized once and cached using [`OnceLock`](src/parsers.rs)
- **Fast Path Optimizations** - Common file extensions and language names use fast-path matching
- **Zero-Cost Abstractions** - Language traits compile to direct function calls

## Examples

### File Type Detection

```rust
use thread_language::SupportLang;

// Get file types for a language
let types = SupportLang::Rust.file_types();
// Use with ignore crate for file filtering
```

### Pattern Building

```rust
use thread_language::JavaScript;
use thread_ast_engine::{Language, PatternBuilder};

let js = JavaScript;
let builder = PatternBuilder::new("console.log($MSG)");
let pattern = js.build_pattern(&builder).unwrap();
```

## Contributing

When adding a new language:

1. Add the tree-sitter dependency to `Cargo.toml`
2. Add the parser function to [`parsers.rs`](src/parsers.rs)
3. Choose the appropriate macro (`impl_lang!` or `impl_lang_expando!`) in [`lib.rs`](src/lib.rs)
4. Add the language to [`SupportLang`](src/lib.rs) enum and related functions
5. Add tests in a separate module file

## License

Licensed under AGPL-3.0-or-later AND MIT. See license files for details.
