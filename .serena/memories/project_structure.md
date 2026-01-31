<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: AGPL-3.0-or-later
-->

# Thread Project Structure

## Workspace Crates

### Core Library Crates
1. **thread-ast-engine** (`crates/ast-engine/`)
   - Core AST parsing, pattern matching, and transformation engine
   - Forked from ast-grep-core
   - Handles meta-variable matching and AST traversal

2. **thread-rule-engine** (`crates/rule-engine/`)
   - Rule-based scanning and transformation system
   - YAML configuration support
   - Code analysis rules and fixes

3. **thread-language** (`crates/language/`)
   - Language definitions and tree-sitter parser integrations
   - Supports 20+ languages: Rust, JS/TS, Python, Go, Java, C/C++, etc.
   - Tier-based language support (Tier 1: primary, Tier 2: full, Tier 3: basic)

4. **thread-utils** (`crates/utils/`)
   - Shared utilities across crates
   - SIMD optimizations for string operations
   - Hash functions (rapidhash)
   - Common helper functions

5. **thread-services** (`crates/services/`)
   - High-level service interfaces
   - API abstractions
   - Integration layer

6. **thread-wasm** (`crates/wasm/`)
   - WebAssembly bindings
   - Browser and edge deployment support
   - JavaScript interop

### Build System
- **xtask** (`xtask/`)
  - Custom build tasks
  - WASM compilation with optimization
  - Build orchestration

## Directory Layout
```
thread/
├── crates/          # All library crates
├── scripts/         # Development scripts (license updates, etc.)
├── xtask/           # Build automation
├── LICENSES/        # License texts
├── .github/         # CI/CD workflows
├── info/            # Documentation
├── target/          # Build artifacts
├── Cargo.toml       # Workspace configuration
├── mise.toml        # Tool management
└── hk.pkl          # Git hooks and linting
```
