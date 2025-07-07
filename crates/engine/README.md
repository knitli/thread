<!--
SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: AGPL-3.0-or-later
-->

# thread-engine

Analysis engine implementation for Thread code analysis.

This crate provides the main analysis engine that orchestrates parsing, graph building, and query operations.

## Architecture

- **ThreadEngine**: Main engine that manages the code graph
- **Analyzer**: High-level orchestration of analysis pipeline
- **GraphBuilder**: Constructs petgraph from parsed code elements
- **QueryEngine**: Extracts context and relationships from the graph

## Usage

```rust
use thread_engine::Analyzer;

let mut analyzer = Analyzer::new();
let result = analyzer.analyze_rust_file(content, file_path)?;
println!("Found {} elements", result.elements_found);
```

## Day 2 Target

The immediate goal is implementing `analyze_rust_file()` to:
1. Parse Rust code with thread-parse
2. Extract functions and basic relationships
3. Build a queryable graph representation
4. Return analysis results