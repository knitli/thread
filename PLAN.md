<!--
SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-License-Identifier: MIT OR Apache-2.0
-->

thread/
├── crates/
│   ├── thread-core/        # Main engine + petgraph + error types
│   ├── thread-parse/       # ast-grep integration
│   ├── thread-store/       # Content store + memory mapping
│   ├── thread-diff/        # Vendored difftastic diff algorithms
│   ├── thread-cli/         # Command line interface
│   └── thread-wasm/        # WASM bindings
├── examples/               # Usage examples
├── docs/                   # Documentation
└── scripts/               # Build scripts

You're absolutely right to feel confused - those multiple implementation plans created a tangle of overlapping ideas. Let me cut through the complexity and show you a **single, coherent architecture** that actually makes sense.

## The Core Problem: Too Many Representations

Your instinct is spot-on: creating multiple data structures for the same data is wasteful and complex. The solution is to pick **one primary representation** and build everything around it.

## Simplified Architecture: One Graph to Rule Them All

Here's how the pieces actually fit together:

```
File → ast-grep (parsing) → petgraph (analysis) → Content store (dedup) → API
                      ↓
                   ropey (editing) → incremental updates
```

**That's it.** No type-sitter, no tree-sitter-graph, no redundant representations.

## What Each Component Actually Does

Let me explain each piece in plain terms:

### ast-grep: Your Parsing Orchestrator

- **What it does**: Detects file types, loads appropriate tree-sitter parsers, gives you clean AST access
- **Why you need it**: Handles the messy tree-sitter setup and gives you a jQuery-like API
- **You don't build competing representations** - you extract data from ast-grep and put it into petgraph

### petgraph: Your Single Source of Truth

- **What it does**: Stores your code structure as nodes (functions, classes) and edges (calls, imports)
- **Why you need it**: Fast queries, graph algorithms, memory-efficient storage
- **This is your primary data structure** - everything else feeds into or reads from this

### ropey: Your Text Editor

- **What it does**: Efficient text editing with line/column tracking
- **Why you need it**: When code changes, you can update specific parts without reparsing everything
- **How it fits**: Updates trigger incremental petgraph updates

### fmmap: Your Large File Handler

- **What it does**: Memory-maps huge files so you don't load them entirely into RAM
- **Why you need it**: Parse 100MB files without using 100MB of memory
- **How it fits**: Feeds chunks to ast-grep for parsing

### Content-Addressable Storage: Your Deduplication Layer

- **What it does**: Uses hashes to avoid storing duplicate content
- **Why you need it**: If 10 files import the same function, store the function once
- **rapidhash vs blake3**: rapidhash is faster, blake3 is more standard. Pick rapidhash for speed.

## Concrete Data Flow Example

Let's say you're analyzing this Rust file:

```rust
// main.rs
use std::collections::HashMap;

fn process_data(input: &str) -> HashMap<String, i32> {
    let mut result = HashMap::new();
    // ... processing logic
    result
}

fn main() {
    let data = process_data("hello");
    println!("{:?}", data);
}
```

Here's exactly what happens:

### Step 1: Parse with ast-grep

```rust
// In your parser crate
let ast = ast_grep_core::AstGrep::new(content, Language::Rust);
let root = ast.root();

// Find functions
let functions = root.find_all("fn $NAME($PARAMS) -> $RETURN { $BODY }");
```

### Step 2: Extract to petgraph

```rust
// In your core crate
let mut graph = petgraph::Graph::new();

for func in functions {
    let func_node = graph.add_node(CodeNode {
        id: hash_content(func.text()),
        kind: NodeKind::Function,
        name: func.field("NAME").text().to_string(),
        line: func.start_position().row,
        text: func.text().to_string(),
    });

    // Find calls within this function
    let calls = func.find_all("$FUNC($ARGS)");
    for call in calls {
        let call_node = graph.add_node(CodeNode {
            kind: NodeKind::FunctionCall,
            name: call.field("FUNC").text().to_string(),
            // ...
        });
        graph.add_edge(func_node, call_node, EdgeKind::Calls);
    }
}
```

### Step 3: Store with content addressing

```rust
// In your buffer crate
let content_hash = rapidhash::hash(content.as_bytes());
if !content_store.contains(&content_hash) {
    content_store.insert(content_hash, content);
}
```

**Result**: One graph containing all your code structure, with efficient storage and fast queries.

## Simplified Repo Structure

Based on this understanding, here's a cleaner crate structure:

```
thread/
├── crates/
│   ├── thread-core/        # Main analysis engine + petgraph
│   ├── thread-parse/       # ast-grep integration
│   ├── thread-store/       # Content-addressable storage + fmmap
│   ├── thread-edit/        # ropey integration for live updates
│   ├── thread-diff/        # difftastic algorithms (vendored)
│   ├── thread-cli/         # Command line interface
│   └── thread-wasm/        # WASM bindings
```

**That's 7 crates instead of 10**, and each has a clear, single responsibility.

## Core Integration Example

Here's how the main engine actually works:## How the Parsing Layer Works## Content Store with Memory Mapping## What to Skip for MVP

You're right to question some of the complexity. Here's what to **skip** for your MVP:

### ❌ Skip These (Add Complexity Without Clear Value)

- **type-sitter**: Adds build complexity and compile-time dependency management for marginal benefit
- **tree-sitter-graph**: Complicates WASM builds and memory management; petgraph is more flexible
- **difftastic parsing**: Only vendor their diff algorithms, use ast-grep for parsing
- **ropey for now**: Start with simple string replacement, add incremental editing later

### ✅ Keep These (Core Value)

- **ast-grep**: Mature parsing with great language detection
- **petgraph**: Your single source of truth for code structure
- **content-addressable storage**: Essential for deduplication
- **memory mapping**: Critical for large repos

## Recommended Repo Structure

Based on this simplified architecture, here's your actual repo structure:

```
thread/
├── crates/
│   ├── thread-core/        # Main engine + petgraph + error types
│   ├── thread-parse/       # ast-grep integration
│   ├── thread-store/       # Content store + memory mapping
│   ├── thread-diff/        # Vendored difftastic diff algorithms
│   ├── thread-cli/         # Command line interface
│   └── thread-wasm/        # WASM bindings
├── examples/               # Usage examples
├── docs/                   # Documentation
└── scripts/               # Build scripts
```

**That's 6 crates total** - much simpler than your current 10.

## Concrete Next StepsHere's your concrete action plan

### Step 1: Reorganize Your Crates (1 day)

**Consolidate to 6 crates:**

```bash
# Delete unnecessary crates
rm -rf crates/ast crates/derive crates/query crates/vcs

# Rename existing crates
mv crates/core crates/thread-core
mv crates/parser crates/thread-parse
mv crates/buffer crates/thread-store
mv crates/diff crates/thread-diff
mv crates/cli crates/thread-cli
mv crates/wasm crates/thread-wasm

# Merge language detection into parser
cp crates/lang/src/* crates/thread-parse/src/
rm -rf crates/lang
```

### Step 2: Build Your First Working Example (2-3 days)

Start with this **minimal working example** in `thread-core`:

```rust
// crates/thread-core/src/lib.rs
use thread_parse::Parser;
use petgraph::Graph;

pub fn analyze_rust_file(content: &str) -> Result<Analysis, Error> {
    let mut parser = Parser::new();
    let elements = parser.parse(content, SupportedLanguage::Rust)?;

    let mut graph = Graph::new();
    for element in elements {
        graph.add_node(element);
    }

    Ok(Analysis { graph, node_count: graph.node_count() })
}
```

**Test it on your own codebase:**

```bash
cargo run -- analyze crates/thread-core/src/lib.rs
```

### Step 3: Add Content-Addressable Storage (1-2 days)

Implement the content store and test deduplication:

```rust
let mut store = ContentStore::new();
let hash1 = store.intern("fn main() {}");
let hash2 = store.intern("fn main() {}"); // Same hash!
assert_eq!(hash1, hash2);
```

### Step 4: Connect to AI Context Goal

**This is how your architecture serves AI assistants:**

When an AI asks: *"How does the `parse` function work in Thread?"*

Your system can:

1. **Find the function**: Query graph for nodes where `name == "parse"`
2. **Find dependencies**: Traverse edges to see what `parse` calls
3. **Find usage**: Traverse reverse edges to see what calls `parse`
4. **Build context**: Return just the relevant functions with exact line numbers

Instead of dumping entire files, you give the AI **exactly what it needs**.

### Step 5: Test End-to-End (1 day)

Build a simple CLI command:

```bash
thread context --function "parse" --file "src/main.rs"
```

Output:

```markdown
## Function: parse (line 42-67)
```rust
fn parse(&self, content: &str) -> Result<Vec<Element>, Error> {
    // ... exact function code
}
```

## Called by

- analyze_file (line 15)
- update_file (line 89)

## Calls

- detect_language (line 45)
- extract_functions (line 52)

```

**That's your MVP.** A working system that can provide intelligent context to AI assistants.

## Why This Architecture Works

1. **Single source of truth**: Everything goes into one petgraph
2. **No duplication**: Content store handles dedup automatically
3. **Fast queries**: Graph traversal is O(edges), not O(files)
4. **Incremental**: Can add features without rebuilding everything
5. **WASM-friendly**: ast-grep compiles to WASM, petgraph is pure Rust

## Start Simple, Add Complexity Later

You can add these features incrementally:
- **Week 2**: Memory mapping for large files
- **Week 3**: Incremental updates with ropey
- **Week 4**: WASM compilation
- **Week 5**: Difftastic integration for change tracking

The beauty of this architecture is that each piece is independent. Get the core working first, then add the bells and whistles.

**Your next commit should be**: A working `analyze_rust_file()` function that parses a simple Rust file and builds a petgraph. Everything else can wait.


# Thread: 30-Day Implementation Plan

*Building a Rust code analysis engine for AI context generation*

## Overview & Success Metrics

**Goal**: By day 30, you'll have a working tool that can analyze Rust codebases and generate perfect context for AI assistants.

**Success Criteria**:
- ✅ Parses Rust files into a queryable graph
- ✅ Provides AI-friendly context with exact line numbers
- ✅ Handles large codebases efficiently
- ✅ Compiles to WASM for edge deployment
- ✅ CLI tool that works on real projects

---

## Week 1: Foundation (Days 1-7)
*Theme: Get something working quickly*

### Day 1: Project Cleanup & Setup
**Goal**: Clean repo structure, working build

**Tasks**:
```bash
# Reorganize crates (30 min)
rm -rf crates/{ast,derive,query,vcs}
mv crates/core crates/thread-core
mv crates/parser crates/thread-parse
mv crates/buffer crates/thread-store
mv crates/cli crates/thread-cli
mv crates/wasm crates/thread-wasm

# Update root Cargo.toml (30 min)
# Add workspace dependencies
```

**Implementation Changes**:

- While setting up the workspace, I realized that any filesystem bound operations (reading/writing files) need to be separated from all other operations. This is because the filesystem is not available in WASM, and we want to keep the core logic portable. I added the `thread-fs` crate to handle filesystem operations separately. We'll still use ast-grep's implementation most likely, so it will be a very tiny crate.
- At some point we may need an interface to the filesystem that allows us to read/write files in a way that is compatible with both Rust and WASM (i.e. communicate with a server implementing a trait or a filesystem handler implementing the same trait -- abstract the 'get and save stuff' ops from IO/environment). For now, we can just use the `thread-fs` crate for reading/writing files in Rust.

**Deliverable**: `cargo build` works without errors

### Day 2: Basic ast-grep Integration <!-- We are here on 6 July -->

**Goal**: Parse a simple Rust file

**Tasks**:

- Add ast-grep dependency to `thread-parse`
- Implement basic language detection
- Write function to extract Rust functions
- Test with a simple example

**Deliverable**:

```rust
let functions = parser.extract_functions("fn main() { println!(\"hello\"); }")?;
assert_eq!(functions.len(), 1);
assert_eq!(functions[0].name, "main");
```

### Day 3: Petgraph Integration

**Goal**: Build your first code graph

**Tasks**:

- Add petgraph to `thread-core`
- Define `CodeNode` and `CodeEdge` structs
- Implement graph building from parsed functions
- Test graph creation and basic queries

**Deliverable**:

```rust
let graph = analyzer.build_graph(functions)?;
assert_eq!(graph.node_count(), 1);
```

### Day 4: End-to-End MVP

**Goal**: Working file analysis

**Tasks**:

- Connect parser → graph builder → analysis result
- Implement `analyze_file()` in thread-core
- Test on a real Rust file (like `src/main.rs`)
- Add basic error handling

**Deliverable**: Analyze your own `main.rs` and print function names

### Day 5: Content-Addressable Storage

**Goal**: Deduplication foundation

**Tasks**:

- Implement basic ContentStore in `thread-store`
- Add rapidhash for content hashing
- Test deduplication with duplicate content
- Integrate with graph builder

**Deliverable**: Same content gets same hash, storage deduplicates automatically

### Day 6: Basic CLI Interface

**Goal**: Usable command-line tool

**Tasks**:

- Create `thread analyze <file>` command
- Pretty-print analysis results
- Add `--format json` option
- Test on multiple Rust files

**Deliverable**:

```bash
thread analyze src/main.rs
# Output: Functions: main (line 1), Dependencies: 0
```

### Day 7: Week 1 Demo & Testing

**Goal**: Working end-to-end system

**Tasks**:

- Run analysis on Thread's own codebase
- Write basic integration tests
- Document what works and what doesn't
- Plan Week 2 priorities

**Deliverable**: Demo video showing Thread analyzing itself

---

## Week 2: Core Features (Days 8-14)

#### Theme: Make it robust and useful

### Day 8: Multi-Language Support

**Goal**: Support JavaScript/TypeScript

**Tasks**:

- Add JavaScript patterns to ast-grep integration
- Implement `extract_js_functions()` and `extract_js_classes()`
- Test with real JS/TS files
- Update language detection

**Deliverable**: Parse both Rust and JavaScript files correctly

### Day 9: Relationship Detection

**Goal**: Find function calls and imports

**Tasks**:

- Extract function calls from parsed ASTs
- Build call graph edges in petgraph
- Implement import/export detection
- Test relationship accuracy

**Deliverable**: Graph shows "main calls println!" relationships

### Day 10: Memory Mapping for Large Files

**Goal**: Handle big repositories

**Tasks**:

- Add fmmap integration to thread-store
- Implement threshold-based memory mapping
- Test with files >1MB
- Benchmark memory usage

**Deliverable**: Parse large files without loading them entirely into memory

### Day 11: Graph Queries & Context Generation

**Goal**: AI-ready context extraction

**Tasks**:

- Implement `find_function()`, `get_dependencies()`, `get_callers()`
- Create context generation that includes related functions
- Format output for AI consumption (markdown with line numbers)
- Test context relevance

**Deliverable**: Given a function name, return all related code with line numbers

### Day 12: Incremental Updates Foundation

**Goal**: Detect what changed

**Tasks**:

- Add change detection when files are modified
- Implement basic graph node invalidation
- Test update performance vs full re-analysis
- Design API for incremental updates

**Deliverable**: Re-analyzing changed files is faster than full analysis

### Day 13: Error Handling & Fallbacks

**Goal**: Graceful failure handling

**Tasks**:

- Implement parse error recovery
- Add fallback strategies for unsupported files
- Timeout handling for large files
- Comprehensive error types

**Deliverable**: Tool handles malformed files and edge cases gracefully

### Day 14: Week 2 Demo & Optimization

**Goal**: Fast, reliable analysis

**Tasks**:

- Performance profiling and optimization
- Test on popular open-source Rust projects
- Memory usage analysis
- Document performance characteristics

**Deliverable**: Analyze a 100+ file project in under 10 seconds

---

## Week 3: Production Ready (Days 15-21)

*Theme: Real-world usage and polish*

### Day 15: Advanced CLI Features

**Goal**: Full-featured command-line interface

**Tasks**:

- Add `thread scan <directory>` for whole projects
- Implement `thread context --function <name>` for AI context
- Add filtering options (include/exclude patterns)
- Progress bars and better UX

**Deliverable**:

```bash
thread context --function "parse" --include-callers --include-dependencies
```

### Day 16: Configuration System

**Goal**: Customizable analysis

**Tasks**:

- Add `thread.toml` configuration file support
- Configurable language patterns
- Custom ignore patterns
- Analysis depth settings

**Deliverable**: Users can customize analysis behavior per project

### Day 17: Repository-Level Analysis

**Goal**: Understand project structure

**Tasks**:

- Implement project-wide dependency graphs
- Module/package relationship detection
- Cross-file reference resolution
- Export project summaries

**Deliverable**: Generate project-wide code maps with module relationships

### Day 18: Content Addressing Optimization

**Goal**: Efficient storage and retrieval

**Tasks**:

- Implement storage compression
- Add garbage collection for unused content
- Optimize hash table performance
- Cache analysis results

**Deliverable**: Analyzing the same project twice is nearly instant

### Day 19: Real-World Testing

**Goal**: Validate on popular projects

**Tasks**:

- Test on tokio, serde, clap, and other popular crates
- Benchmark against manual code reading
- Collect accuracy metrics
- Fix bugs found in testing

**Deliverable**: Successfully analyze 10+ popular Rust projects

### Day 20: Documentation & Examples

**Goal**: User-friendly documentation

**Tasks**:

- Write comprehensive README
- Create usage examples
- Document API for programmatic use
- Performance tuning guide

**Deliverable**: Someone else can use Thread effectively from the docs

### Day 21: Week 3 Demo & Integration Testing

**Goal**: Production-ready tool

**Tasks**:

- End-to-end integration tests
- CI/CD pipeline setup
- Release preparation
- Performance regression tests

**Deliverable**: Automated testing ensures reliability

---

## Week 4: WASM & Deployment (Days 22-30)

#### Theme: Edge deployment and finishing touches

### Day 22: WASM Compilation Setup

**Goal**: Compile core functionality to WASM

**Tasks**:

- Configure WASM build for thread-core and thread-parse
- Handle ast-grep WASM compatibility issues
- Create JavaScript bindings
- Test basic WASM functionality

**Deliverable**: Core parsing functionality runs in browser/Cloudflare Workers

### Day 23: WASM API Design

**Goal**: Clean JavaScript interface

**Tasks**:

- Design ergonomic JS API for WASM module
- Handle memory management across JS/WASM boundary
- Implement async patterns for large operations
- Add TypeScript definitions

**Deliverable**:

```javascript
const thread = await ThreadWasm.new();
const result = await thread.analyzeCode(rustCode, 'rust');
```

### Day 24: Edge Deployment Testing

**Goal**: Validate Cloudflare Workers deployment

**Tasks**:

- Create minimal Cloudflare Worker using Thread WASM
- Test memory limits and performance
- Optimize bundle size
- Handle worker timeout constraints

**Deliverable**: Working demo on Cloudflare Workers

### Day 25: API Service Architecture

**Goal**: HTTP API for Focus by knitli service

**Tasks**:

- Design REST API for code analysis
- Implement rate limiting and auth placeholders
- Add webhook endpoints for GitHub integration
- Test API performance under load

**Deliverable**: HTTP API that provides code analysis as a service

### Day 26: GitHub Integration Preparation

**Goal**: Foundation for GitHub App

**Tasks**:

- Implement repository cloning and analysis
- Add webhook payload parsing
- Design data models for stored analyses
- Test with real GitHub repositories

**Deliverable**: Can analyze a GitHub repository from webhook payload

### Day 27: Performance Optimization & Caching

**Goal**: Production-scale performance

**Tasks**:

- Implement intelligent caching strategies
- Optimize for repeated analyses
- Memory usage profiling and optimization
- Benchmark against performance targets

**Deliverable**: Meets performance requirements for commercial service

### Day 28: Security & Reliability

**Goal**: Production security standards

**Tasks**:

- Input validation and sanitization
- Resource limit enforcement
- Error reporting and monitoring hooks
- Security audit of dependencies

**Deliverable**: Secure, reliable service foundation

### Day 29: Final Integration & Polish

**Goal**: Complete, polished system

**Tasks**:

- End-to-end testing of all components
- Bug fixes and edge case handling
- Final documentation updates
- Release candidate preparation

**Deliverable**: Feature-complete Thread v0.1.0

### Day 30: Launch Preparation & Demo

**Goal**: Ready for public use

**Tasks**:

- Create compelling demo showcasing Thread's capabilities
- Prepare launch blog post
- Set up issue tracking and contribution guidelines
- Submit to crates.io

**Deliverable**: 🚀 **Thread 0.1.0 released and ready for early adopters**

---

## Daily Reality Checks

**Every day, ask yourself**:

1. ✅ Does what I built today actually work when I test it?
2. ✅ Can I demo this feature to someone else and have it make sense?
3. ✅ Am I building toward the AI context generation goal?
4. ✅ What's the simplest thing I can cut if I'm behind schedule?

## Weekly Demos

**End of each week, record a short demo showing**:

- Week 1: "Thread can parse a Rust file and show me the functions"
- Week 2: "Thread can analyze my entire project and show dependencies"
- Week 3: "Thread generates perfect AI context for any function I ask about"
- Week 4: "Thread runs in Cloudflare Workers and serves analysis via API"

## Risk Mitigation

**If you fall behind**:

- **Week 1 behind?** Skip multi-language support, focus on Rust only
- **Week 2 behind?** Skip incremental updates, focus on full analysis
- **Week 3 behind?** Skip advanced CLI features, focus on core functionality
- **Week 4 behind?** Skip WASM optimization, get basic compilation working

**The MVP is**: A working CLI tool that can analyze Rust files and generate AI-friendly context. Everything else is enhancement.

## Success Celebration 🎉

**By Day 30, you'll have**:

- A working Rust crate published to crates.io
- WASM module running on Cloudflare Workers
- Foundation for Focus by knitli commercial service
- Proof of concept for Thread's full vision
- Something genuinely useful for developers and AI assistants

**This plan balances ambition with achievability** - each week builds on the last, with built-in flexibility to adapt as you learn and discover what works best.
