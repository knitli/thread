<!--
SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-License-Identifier: MIT OR Apache-2.0
-->

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Thread is a Rust code analysis engine designed to generate intelligent context for AI assistants. The core goal is to parse code into a queryable graph that can provide exactly the right context when an AI asks about specific functions, dependencies, or code relationships.

**Current Status**: Day 2 of 30-day implementation plan. Basic scaffolding exists, but most crates contain placeholder code from earlier architectural iterations.

## Simplified Architecture

Thread follows a single-representation approach:

```plaintext
File → ast-grep (parsing) → petgraph (analysis) → Content store (dedup) → API
```

### Core Components

- **ast-grep**: Parsing orchestrator with tree-sitter integration and language detection
- **petgraph**: Single source of truth for code structure (nodes = functions/classes, edges = calls/imports)
- **Content-addressable storage**: Deduplication using rapidhash
- **fmmap**: Memory mapping for large files
- **thread-fs**: Filesystem operations (separated for WASM compatibility)

## Idiomatic Crate Structure

The workspace follows Rust conventions with core types separated from implementations:

- `thread-core/` - Core traits, types, and error definitions only
- `thread-engine/` - Main analysis implementation using petgraph
- `thread-parse/` - ast-grep integration and language detection
- `thread-store/` - Content-addressable storage + memory mapping
- `thread-fs/` - Filesystem operations (WASM-compatible abstraction)
- `thread-diff/` - Vendored difftastic diff algorithms
- `thread-cli/` - Command line interface
- `thread-wasm/` - WebAssembly bindings
- `xtask/` - Build automation for WASM targets

### Design Rationale

This structure follows the pattern used by `serde` (core traits) vs `serde_json` (implementation):

- `thread-core` defines `LanguageParser` trait, `CodeElement` types, `Result` types
- `thread-engine` implements the actual analysis logic and graph building
- Other crates can depend on `thread-core` for types without pulling in the full engine

## Development Commands

### Build Commands

- `mise run build` or `mise run b` - Build all crates (except WASM)
- `mise run build-release` or `mise run br` - Release build
- `mise run build-wasm` or `mise run bw` - Build WASM for development (single-threaded)
- `mise run build-wasm-release` or `mise run bwr` - Build WASM for production

### WASM Build Options

- `cargo run -p xtask build-wasm` - Basic WASM build
- `cargo run -p xtask build-wasm --multi-threading` - Multi-threaded for browsers
- `cargo run -p xtask build-wasm --release` - Production optimized
- `cargo run -p xtask build-wasm --profiling` - With profiling enabled

### Testing and Quality

- `mise run test` or `mise run t` - Run tests with `cargo nextest`
- `mise run lint` or `mise run c` - Full linting via `hk run check`
- `mise run fix` or `mise run f` - Auto-fix formatting and linting
- `mise run ci` - Run all CI checks (build + lint + test)

### Development Setup

- `mise run install` - Install dev tools and git hooks
- `mise run update` - Update all dev tools
- `mise run clean` - Clean build artifacts and caches

## Implementation Plan Context

### Current Sprint (Week 1)

- **Day 1**: ✅ Project cleanup and setup
- **Day 2**: 🔄 Basic ast-grep integration (current focus)
- **Day 3**: Petgraph integration
- **Day 4**: End-to-end MVP
- **Day 5**: Content-addressable storage
- **Day 6**: Basic CLI interface
- **Day 7**: Week 1 demo and testing

### Near-term Goals

The immediate target is a working `analyze_rust_file()` function that:

1. Parses Rust code with ast-grep
2. Extracts functions, calls, and imports
3. Builds a petgraph representation
4. Provides basic graph queries

### MVP Definition

A CLI tool that can analyze Rust files and generate AI-friendly context showing:

- Function definitions with line numbers
- Call relationships (what calls what)
- Import dependencies
- Context-relevant code snippets for AI assistants

## Key Design Decisions

### What to Skip for MVP

- ❌ type-sitter (build complexity)
- ❌ tree-sitter-graph (memory management complexity)
- ❌ ropey (incremental editing - add later)
- ❌ Multi-language support initially (Rust first)

### What to Keep

- ✅ ast-grep (mature parsing with language detection)
- ✅ petgraph (single source of truth)
- ✅ Content-addressable storage (essential for deduplication)
- ✅ Memory mapping (critical for large repos)

## Testing Strategy

- Uses `cargo nextest` for parallel test execution
- Single-threaded execution (`-j 1`) to prevent race conditions
- `--no-fail-fast` for development, `--fail-fast` for CI
- Full backtraces enabled (`RUST_BACKTRACE=1`)

## WASM Considerations

- Default build is single-threaded for Cloudflare Workers
- Multi-threaded builds available for browser environments
- Core logic separated from filesystem operations for portability
- Uses `wasm-opt` for size and performance optimization

## Context Generation Goal

When an AI asks: "How does the `parse` function work in Thread?"

Thread should provide:

1. **Function location**: Exact file and line numbers
2. **Dependencies**: What functions `parse` calls
3. **Usage**: What functions call `parse`
4. **Context**: Related code snippets with line numbers

This enables AI assistants to get precisely the context they need without dumping entire files.
