<!--
SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# thread-services

Service layer interfaces and API abstractions for the Thread code analysis platform.

## Overview

`thread-services` provides the high-level service interfaces that bridge Thread's AST engine with
codebase-level relational intelligence. It abstracts over different execution environments—CLI,
cloud workers, and WASM—while preserving the full power of the underlying AST capabilities.

## Architecture

The service layer acts as **abstraction glue** between Thread's components:

- **Preserves Power**: All ast-grep capabilities (pattern matching, replacement, position tracking) remain accessible
- **Bridges Levels**: Connects file-level AST operations to codebase-level relational intelligence
- **Enables Execution**: Abstracts over different execution environments (Rayon, tokio, cloud workers)
- **Commercial Boundaries**: Clear separation for open-source and commercial extensions

## Key Components

- **`types`** — Language-agnostic types that wrap ast-grep functionality, including `ParsedDocument` and `SymbolTable`
- **`traits`** — Service interfaces for parsing, analysis, and storage (`CodeAnalyzer`, `StorageProvider`)
- **`error`** — Comprehensive error handling with recovery strategies and contextual messages
- **`facade`** — High-level facade for common use cases without boilerplate

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
thread-services = "0.1"
```

### Basic Example

```rust,no_run
use thread_services::types::ParsedDocument;
use thread_services::traits::CodeAnalyzer;

async fn analyze_file(document: &ParsedDocument<impl thread_ast_engine::source::Doc>) {
    // Access underlying AST functionality directly
    let root = document.ast_grep_root();
    let matches = root.root().find_all("fn $NAME($$$PARAMS) { $$$BODY }");

    // Plus codebase-level metadata
    let symbols = document.metadata().defined_symbols.keys();
    println!("Found functions: {:?}", symbols.collect::<Vec<_>>());
}
```

## Related Crates

- [`thread-ast-engine`](../ast-engine) — Core AST parsing and pattern matching
- [`thread-language`](../language) — Language definitions and tree-sitter parsers
- [`thread-flow`](../flow) — Dataflow orchestration with ReCoco integration
- [`thread-rule-engine`](../rule-engine) — YAML-configurable rule-based scanning

## License

AGPL-3.0-or-later

