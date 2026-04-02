<!--
SPDX-FileCopyrightText: 2026 Knitli Inc. <knitli@knit.li>
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# thread

Unified entry point for the Thread code analysis ecosystem.

[![Crate](https://img.shields.io/crates/v/thread.svg)](https://crates.io/crates/thread)
[![Documentation](https://docs.rs/thread/badge.svg)](https://docs.rs/thread)
[![License](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue.svg)](../../LICENSE.md)

## Overview

`thread` is the top-level crate that re-exports the core components of the Thread ecosystem
under a single, coherent API. Instead of depending on several sub-crates individually, most
users should start here.

```toml
[dependencies]
thread = "0.1"
```

## Quick Start

```rust
use thread::language::{SupportLang, LanguageExt};

// Parse code with any supported language
let ast = SupportLang::Rust.ast_grep("fn hello() -> &'static str { \"world\" }");
let root = ast.root();

// Pattern match with meta-variables
let matches = root.find_all("fn $NAME($$$PARAMS) -> $RET { $$$BODY }");
for m in matches {
    let name = m.get_env().get_match("NAME").unwrap().text();
    println!("Function: {name}");
}
```

## Modules

| Module | Enabled by | Description |
|--------|-----------|-------------|
| `thread::ast` | `ast` (default) | Core AST parsing, matching, and transformation |
| `thread::language` | `language` (default) | 26-language tree-sitter parsers and `SupportLang` |
| `thread::rule` | `rule` (default) | YAML-configurable rule-based scanning |
| `thread::services` | `services` (default) | High-level service interfaces and abstractions |
| `thread::flow` | `flow` | ReCoco-based dataflow orchestration and caching |
| `thread::utils` | `utils` | Hashing and SIMD utilities |

## Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `ast` | Core AST engine (parsing + matching) | ✅ |
| `language` | All 26 language parsers | ✅ |
| `rule` | YAML rule engine | ✅ |
| `services` | Service layer abstractions | ✅ |
| `mimalloc` | High-performance allocator | ✅ |
| `flow` | ReCoco dataflow + Postgres backend + parallelism | — |
| `utils` | Shared SIMD/hash utilities | — |
| `full` | Every feature including embedded HTML injection | — |
| `worker` | Edge/WASM build (no filesystem, single-threaded) | — |

### Library-only Usage (minimal)

```toml
[dependencies]
thread = { version = "0.1", default-features = false, features = ["ast", "language"] }
```

### With Dataflow Pipelines

```toml
[dependencies]
thread = { version = "0.1", features = ["flow"] }
```

```rust
use thread::flow::ThreadFlowBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let flow = ThreadFlowBuilder::new("analyze_project")
        .source_local("src/", &["**/*.rs"], &["target/**"])
        .parse()
        .extract_symbols()
        .target_postgres("symbols", &["content_hash"])
        .build()
        .await?;

    flow.execute().await?;
    Ok(())
}
```

## Top-Level Re-exports

The following items are available at the crate root without module qualification:

```rust
use thread::{AstGrep, Language, Node, Root};   // from thread-ast-engine
use thread::SupportLang;                        // from thread-language
use thread::{CodeAnalyzer, CodeParser, ParsedDocument, ServiceError, ServiceResult};
```

## WASM / Edge Deployment

Use the `worker` feature for Cloudflare Workers:

```toml
[dependencies]
thread = { version = "0.1", default-features = false, features = ["worker"] }
```

See [`thread-wasm`](../wasm) for the WASM build pipeline and deployment tooling.

## Related Crates

- [`thread-ast-engine`](../ast-engine) — Core AST engine (use directly for low-level control)
- [`thread-language`](../language) — Language parsers
- [`thread-rule-engine`](../rule-engine) — YAML rule engine
- [`thread-flow`](../flow) — Dataflow orchestration with ReCoco
- [`thread-services`](../services) — Service layer interfaces
- [`thread-utilities`](../utils) — Shared utilities

## License

AGPL-3.0-or-later
