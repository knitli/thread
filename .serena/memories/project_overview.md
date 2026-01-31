<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: AGPL-3.0-or-later
-->

# Thread Project Overview

## Purpose
Thread is a safe, fast, flexible code analysis and parsing library built in Rust. It provides powerful AST-based pattern matching and transformation capabilities using tree-sitter parsers. The project is forked from ast-grep and enhanced for production use as a code analysis engine for AI context generation.

## Key Features
- AST-based pattern matching with meta-variables ($VAR, $$$ITEMS, $_)
- YAML-based rule system for code analysis
- Support for 20+ programming languages via tree-sitter
- WebAssembly compilation for browser and edge deployment
- SIMD optimizations for performance
- Parallel processing capabilities with rayon

## Tech Stack
- **Language**: Rust (edition 2024, rust-version 1.85)
- **Parser**: tree-sitter (v0.26.3)
- **Build System**: Cargo workspace (resolver = "3")
- **WASM Build**: Custom xtask with wasm-pack
- **Key Dependencies**:
  - tree-sitter: AST parsing foundation
  - regex: Pattern matching support
  - serde/serde_yaml: Configuration serialization
  - rayon: Parallel processing
  - bit-set: Efficient set operations
  - rapidhash: Fast non-cryptographic hashing
  - memchr: SIMD string searching

## License
- Main codebase: AGPL-3.0-or-later
- Forked ast-grep components: AGPL-3.0-or-later AND MIT
- All files require REUSE-compliant license headers
