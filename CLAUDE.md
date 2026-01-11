<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: AGPL-3.0-or-later
-->

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Thread is a **service-library dual architecture** for safe, fast, flexible code analysis and parsing built in Rust. It operates as both:

1. **Reusable Library Ecosystem** - Modular crates (ast-engine, language, rule-engine) for AST-based pattern matching and transformation using tree-sitter parsers
2. **Persistent Service Platform** - Long-lived service with incremental intelligence, content-addressed caching, and real-time code analysis

The project is forked from ast-grep and enhanced with CocoIndex dataflow framework for production use as a code analysis engine for AI context generation.

**Key Differentiators**:
- ✅ **Content-Addressed Caching**: 50x+ performance gains on repeated analysis via automatic incremental updates
- ✅ **Dual Deployment**: Single codebase compiles to both CLI (Rayon parallelism) and Edge (tokio async, Cloudflare Workers)
- ✅ **Persistent Storage**: Native integration with Postgres (local), D1 (edge), Qdrant (vectors)
- ✅ **Dataflow Orchestration**: Declarative pipelines for ETL and dependency tracking via CocoIndex

## Architecture

Thread follows a **service-library dual architecture** (Constitution v2.0.0, Principle I) with six main crates plus service layer:

### Library Core (Reusable Components)

- **`thread-ast-engine`** - Core AST parsing, pattern matching, and transformation engine (forked from ast-grep-core)
- **`thread-language`** - Language definitions and tree-sitter parser integrations (supports 20+ languages)
- **`thread-rule-engine`** - Rule-based scanning and transformation system with YAML configuration support
- **`thread-utils`** - Shared utilities including SIMD optimizations and hash functions
- **`thread-wasm`** - WebAssembly bindings for browser and edge deployment

### Service Layer (Orchestration & Persistence)

- **`thread-services`** - High-level service interfaces, API abstractions, and CocoIndex integration
- **CocoIndex Dataflow** - Content-addressed caching, incremental ETL, and dependency tracking (foundational framework dependency per Constitution v2.0.0, Principle IV)
- **Storage Backends**:
  - **Postgres** (local CLI) - Persistent caching and analysis results
  - **D1** (Cloudflare Edge) - Distributed caching across CDN nodes
  - **Qdrant** (optional) - Vector similarity search for semantic analysis
- **Concurrency Models**:
  - **Rayon** (CLI) - CPU-bound parallelism for local multi-core utilization
  - **tokio** (Edge) - Async I/O for horizontal scaling and Cloudflare Workers

### Build System

- **`xtask`** - Custom build tasks, primarily for WASM compilation with optimization

## Development Commands

### Building

```bash
# Build everything (except WASM)
mise run build
# or: cargo build --workspace

# Build in release mode
mise run build-release
# or: cargo build --workspace --release --features inline

# Build WASM for development
mise run build-wasm
# or: cargo run -p xtask build-wasm

# Build WASM in release mode
mise run build-wasm-release
# or: cargo run -p xtask build-wasm --release
```

### Testing and Quality

```bash
# Run all tests
mise run test
# or: hk run test
# or: cargo nextest run --all-features --no-fail-fast -j 1

# Full linting
mise run lint
# or: hk run check

# Auto-fix formatting and linting issues
mise run fix
# or: hk fix

# Run CI pipeline locally
mise run ci
```

### Single Test Execution

```bash
# Run specific test
cargo nextest run --manifest-path Cargo.toml test_name --all-features

# Run tests for specific crate
cargo nextest run -p thread-ast-engine --all-features

# Run benchmarks
cargo bench -p thread-rule-engine
```

### Utility Commands

```bash
# Update dependencies
mise run update
# or: cargo update && cargo update --workspace

# Clean build artifacts
mise run clean

# Update license headers
mise run update-licenses
# or: ./scripts/update-licenses.py
```

## Key Language Support

The `thread-language` crate provides built-in support for major programming languages via tree-sitter:

**Tier 1 Languages** (primary focus):

- Rust, JavaScript/TypeScript, Python, Go, Java

**Tier 2 Languages** (full support):

- C/C++, C#, PHP, Ruby, Swift, Kotlin, Scala

**Tier 3 Languages** (basic support):

- Bash, CSS, HTML, JSON, YAML, Lua, Elixir, Haskell

## Pattern Matching System

Thread's core strength is AST-based pattern matching using meta-variables:

### Meta-Variable Syntax

- `$VAR` - Captures a single AST node
- `$$$ITEMS` - Captures multiple consecutive nodes (ellipsis)
- `$_` - Matches any node without capturing

### Example Usage

```rust
// Find function declarations
root.find("function $NAME($$$PARAMS) { $$$BODY }")

// Find variable assignments
root.find_all("let $VAR = $VALUE")

// Complex pattern matching
root.find("if ($COND) { $$$THEN } else { $$$ELSE }")
```

## Rule System

The `thread-rule-engine` supports YAML-based rule definitions for code analysis:

```yaml
id: no-var-declarations
message: "Use 'let' or 'const' instead of 'var'"
language: JavaScript
rule:
  pattern: "var $NAME = $VALUE"
fix: "let $NAME = $VALUE"
```

## Performance Considerations

### Optimization Features

- SIMD optimizations in `thread-utils` for fast string operations
- Parallel processing capabilities with rayon
- Memory-efficient AST representation
- Content-addressable storage for deduplication

### Build Profiles

- **dev**: Fast compilation with basic optimizations
- **dev-debug**: Cranelift backend for faster debug builds
- **release**: Full LTO optimization
- **wasm-release**: Size-optimized for WebAssembly

## WASM Deployment

Thread compiles to WebAssembly for edge deployment:

```bash
# Basic WASM build (for Cloudflare Workers)
cargo run -p xtask build-wasm

# Multi-threading WASM (for browsers)
cargo run -p xtask build-wasm --multi-threading

# Optimized release build
cargo run -p xtask build-wasm --release
```

## Testing Infrastructure

### Test Organization

- Unit tests: In each crate's `src/` directory
- Integration tests: In `tests/` directories
- Benchmarks: In `benches/` directories
- Test data: In `test_data/` directories

### Quality Tooling

- **cargo-nextest**: Parallel test execution
- **hk**: Git hooks and linting orchestration
- **mise**: Development environment management
- **typos**: Spell checking
- **reuse**: License compliance

## Dependencies

### Core Dependencies

- `tree-sitter`: AST parsing foundation
- `regex`: Pattern matching support
- `serde`: Configuration serialization
- `bit-set`: Efficient set operations
- `rayon`: Parallel processing

### Performance Dependencies

- `rapidhash`: Fast non-cryptographic hashing
- `memchr`: SIMD string searching
- `simdeez`: SIMD abstractions

## Contributing Workflow

1. Run `mise run install-tools` to set up development environment
2. Make changes following existing patterns
3. Run `mise run fix` to apply formatting and linting
4. Run `mise run test` to verify functionality
5. Use `mise run ci` to run full CI pipeline locally

## Constitutional Compliance

**All development MUST adhere to the Thread Constitution v2.0.0** (`.specify/memory/constitution.md`)

### Core Governance Principles

1. **Service-Library Architecture** (Principle I)
   - Features MUST consider both library API design AND service deployment
   - Libraries remain self-contained; services leverage CocoIndex for orchestration
   - Dual architecture is non-negotiable—both aspects are first-class citizens

2. **Test-First Development** (Principle III - NON-NEGOTIABLE)
   - TDD mandatory: Tests → Approve → Fail → Implement
   - All tests execute via `cargo nextest`
   - No exceptions, no justifications accepted

3. **Service Architecture & Persistence** (Principle VI)
   - Content-addressed caching MUST achieve >90% hit rate
   - Storage targets: Postgres <10ms, D1 <50ms, Qdrant <100ms p95 latency
   - Incremental updates MUST trigger only affected component re-analysis

### Quality Gates (Constitutional Requirements)

Before any PR merge, verify:
- ✅ `mise run lint` passes (zero warnings)
- ✅ `cargo nextest run --all-features` passes (100% success)
- ✅ `mise run ci` completes successfully
- ✅ Public APIs have rustdoc documentation
- ✅ Performance-sensitive changes include benchmarks
- ✅ Service features meet storage/cache/incremental requirements

### Deployment Validation

- **CLI Target**: Test on Linux, macOS, Windows with Rayon parallelism
- **Edge Target**: `mise run build-wasm-release` succeeds for Cloudflare Workers
- **Storage**: Integration tests pass for all backends (Postgres, D1, Qdrant)

**Review Process**: All PRs MUST have constitution compliance verified by reviewers. Violations require explicit technical justification or rejection.

See `.specify/memory/constitution.md` for complete governance framework.

## License Structure

- Main codebase: AGPL-3.0-or-later
- Forked ast-grep components: AGPL-3.0-or-later AND MIT
- Documentation and config: MIT OR Apache-2.0
- See `VENDORED.md` files for specific attribution

---

## Tools for AI Assistants

The library provides multiple tools to help me AI assistants more efficient:

- MCP Tools:
  - You always have access to `sequential-thinking`. Use this to plan out tasks before executing and document things you learn along the way. Regularly refer back to it.
  - `context7` provides a library of up-to-date code examples and API documentation for almost any library.
- The `llm-edit.sh` script:
  - Script in `scripts/llm-edit.sh` gives you an easy interface for providing multiple file edits in one go.
  Full details on how to use it are in `scripts/README-llm-edit.md`

### Multi-File Output System (llm-edit)

- When the user mentions "multi-file output", "generate files as json", or similar requests for bundled file generation, use the multi-file output system
- Execute using: `./llm-edit.sh <json_file>`
- Provide output as a single JSON object following the schema in `./README-llm-edit.md`
- The JSON must include an array of files, each with file_name, file_type, and file_content fields
- For binary files, encode content as base64 and set file_type to "binary"
- NEVER include explanatory text or markdown outside the JSON structure

## Active Technologies
- Rust (edition 2024, aligning with Thread's existing codebase) (001-realtime-code-graph)
- Multi-backend architecture with deployment-specific primaries: (001-realtime-code-graph)

## Recent Changes
- 001-realtime-code-graph: Added Rust (edition 2024, aligning with Thread's existing codebase)
