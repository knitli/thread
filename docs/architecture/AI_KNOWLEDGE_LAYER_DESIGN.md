<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: AGPL-3.0-or-later
-->

# AI-Native Knowledge Layer: Architectural Design Report

**Version**: 0.1.0 (Draft)
**Date**: 2026-02-21
**Status**: Brainstorm / Options Analysis
**Relates To**: `specs/001-realtime-code-graph/`, Thread Constitution v2.0.0

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Problem Statement](#problem-statement)
3. [State of the Art](#state-of-the-art)
4. [Thread's Current Position](#threads-current-position)
5. [Architectural Options](#architectural-options)
   - [Option A: Graph-Enhanced File Model](#option-a-graph-enhanced-file-model-evolutionary)
   - [Option B: Content-Addressed Definition Store](#option-b-content-addressed-definition-store-unison-inspired)
   - [Option C: Multi-Resolution Knowledge Layer](#option-c-multi-resolution-knowledge-layer-recommended)
6. [Recommendation](#recommendation)
7. [Key Design Decisions](#key-design-decisions)
8. [Implementation Strategy](#implementation-strategy)
9. [Risk Analysis](#risk-analysis)
10. [Open Questions](#open-questions)

---

## Executive Summary

This document explores architectural options for an **AI-native knowledge layer** — a new abstraction that replaces the file as the primary unit of code intelligence. The goal is to design a system where AI agents interact with code through semantically rich, graph-structured representations rather than flat text files, while preserving full human oversight through familiar file-based workflows.

Three architectural options are analyzed, spanning from evolutionary (building on the existing 001-realtime-code-graph spec) to transformative (Unison-inspired content-addressed definition stores). The recommended approach is **Option C: Multi-Resolution Knowledge Layer** — a layered architecture where code is simultaneously represented at multiple levels of abstraction (files, symbols, relationships, architecture, intent), each content-addressed and incrementally updated, with AI agents selecting the appropriate resolution for each task.

### Key Thesis

> The file is a human-oriented container that obscures the actual semantic units of code. AI agents waste the majority of their context windows re-discovering structure that a persistent semantic graph would provide instantly. The knowledge layer inverts the relationship: **the graph is the source of working truth; files are one of many projections generated on demand.**

---

## Problem Statement

### Why Files Limit AI Agents

AI coding agents face systematic friction from the file abstraction:

1. **Context Window Saturation**: Even with 1M-token windows, large codebases don't fit. Agents must choose which files to read, and wrong choices cascade. "Context engineering" — carefully managing what enters the context — has become its own discipline, burning cycles that should go to actual reasoning.

2. **Files Hide Relationships**: Reading `auth.rs` gives an agent no automatic knowledge that `middleware.rs` depends on it, or that changing a function signature will break `tests/test_auth.rs`. The agent must grep, follow imports, and build this dependency map from scratch on every invocation.

3. **Redundant Re-Parsing**: Every agent invocation re-reads, re-parses, and re-analyzes files. There is no persistent semantic state between invocations. Claude Code, Cursor, Augment, Copilot — all of them rebuild understanding from raw text each time.

4. **Multi-File Coordination Explosion**: Refactoring a type used in 50 files requires O(n) search + O(n) reads + O(n) edits. With a graph, it's one traversal + targeted edits. The agent spends more time navigating than reasoning.

5. **Human-Centric Tooling Mismatch**: Programming languages, compilers, and debuggers abstract away internal states for human convenience. AI agents need fine-grained, structured access to these internal states — exactly what a semantic graph provides.

### The Opportunity

Every major AI coding tool is independently building ad-hoc solutions to this problem:

| Tool | Approach | Limitation |
|------|----------|-----------|
| **Augment Code** | Persistent semantic embeddings + dependency graphs + commit history lineage | Proprietary, tightly coupled to their product |
| **Cursor** | Embeddings + AST graphs + contextual cross-references | Per-session, not persistent across invocations |
| **Sourcegraph** | SCIP Code Graph + Zoekt search + Context Engine | File-centric, requires per-language indexers |
| **Claude Code** | bash + grep + git (lowest common denominator) | Rebuilds understanding every time |

Thread's opportunity is to provide this layer as **open infrastructure** — a content-addressed semantic graph that any AI agent can query, eliminating the file-parsing-context-engineering pipeline that every tool reinvents.

### What Success Looks Like

- AI agents query the knowledge layer directly: "what calls this function?", "what types does this module export?", "what changed since this hash?" — without reading files
- Humans continue editing text files normally; the knowledge layer updates incrementally in the background
- Context compression: deliver the semantically relevant subgraph for a task in 1/100th the tokens of reading equivalent files
- Cross-session persistence: the agent picks up where it left off with full architectural understanding
- Bidirectional sync: changes made through the knowledge layer (by AI) are projected back to files correctly

---

## State of the Art

### Content-Addressed Code (Unison)

**Unison** (1.0 released November 2025) stores code as content-addressed, immutable definitions in an append-only SQLite database:

- Each definition identified by a 512-bit SHA3 hash of its AST
- Named arguments replaced by positional references; dependencies replaced by their hashes
- Names are metadata pointers to hash addresses, stored separately — renaming never changes the hash
- Code is parsed once, stored as AST, and pretty-printed on demand
- Result: no builds (perfectly incremental compilation), no dependency conflicts (different versions coexist by hash), no broken states (codebase is always consistent)

**Key lesson for Thread**: You can maintain text as the human editing interface while using a structured database as the source of truth. The "scratch file → parse → store" pipeline is directly applicable.

### Semantic Code Graphs (Kythe, SCIP, Glean)

**Google Kythe**: Hub-and-spoke model where language-specific extractors produce cross-reference graphs, merged into a global graph. Processed billions of lines daily. Now in maintenance mode (team laid off April 2024).

**Sourcegraph SCIP**: Protobuf schema with human-readable symbol IDs. 10x faster indexing and 4-5x smaller indexes than LSIF. Powers Sourcegraph's Code Graph capturing inheritance, service dependencies, and API interactions.

**Meta Glean**: General-purpose fact collection using RocksDB and declarative Angle queries. Indexes diffs as "diff sketches" for semantic search over commits. Hundreds-of-microseconds query latency at massive scale.

### Graph-Integrated LLM Inference (CGM, NeurIPS 2025)

The **Code Graph Model** integrates graph structure directly into LLM attention masks. Uses 7 node types (repo → attribute) and 5 edge types (contains, calls, imports, extends). Achieved 44% resolution on SWE-bench Lite — first among open-weight models. **Critical insight: structural integration via attention masking compresses context by 512x** while preserving cross-file relationships.

### Projectional Editing (MPS, Intentional Software, Darklang)

**JetBrains MPS**: Stores code as an Abstract Syntax Graph with per-node UUIDs. Multiple notations (text, tables, diagrams) project the same AST. Smart diff/merge distinguishes moves from delete/create pairs.

**Intentional Software** (Simonyi, acquired by Microsoft 2017): Separated computational intent from implementation detail and presentation. Conflicts committed as first-class objects.

**Darklang**: Language + structured editor + infrastructure tightly coupled. AST stored in production database, saved within 50ms of each keystroke.

**Key lesson**: Technically superior representations repeatedly lose to text files because of ecosystem lock-in. The winning strategy is **structured database as source of truth, text files as one projection, with bidirectional sync**.

### AI Agent Ecosystem

The emerging consensus: **specifications, not code, may become the primary artifact**. Tools converge on bash + grep as the LCD interface, which is wasteful. 85% of developers use AI tools but "fully delegate" only 0-20% of tasks — the "Collaboration Paradox." The gap is a structured, persistent, semantic layer between raw files and the LLM context window.

---

## Thread's Current Position

### Existing Capabilities

Thread already has foundational pieces for a knowledge layer:

| Capability | Status | Component |
|-----------|--------|-----------|
| Multi-language AST parsing | Production | `thread-ast-engine` (tree-sitter, 20+ languages) |
| Content-addressed caching | Production | `thread-flow` (Blake3, 99.7% cost reduction) |
| Symbol extraction | Production | `thread-flow` functions (parse, extract_symbols, extract_imports, extract_calls) |
| Dual deployment | Production | CLI (Rayon) + Edge (tokio/WASM) |
| Pattern matching | Production | `thread-ast-engine` meta-variables ($VAR, $$$ITEMS) |
| Storage backends | Production | D1 (edge), Postgres (planned), Qdrant (planned) |
| Dataflow orchestration | Production | ReCoco integration via `thread-flow` |
| Incremental updates | Designed | 001 spec (Blake3 fingerprint → cache lookup → selective reparse) |

### 001-Realtime-Code-Graph Foundation

The existing 001 spec provides a direct foundation:

- **Overlay Graph Architecture** (FR-017): Base Layer (immutable, at Git commit) + Delta Layer (ephemeral, uncommitted changes) + Unified View (merged at query time)
- **Content-Addressed Storage**: All entities use content-addressed IDs (SHA-256 hashes)
- **Graph Entities**: GraphNode (symbol-level), GraphEdge (7 relationship types), with petgraph for in-memory algorithms
- **Multi-Tier Conflict Detection**: Tier 1 AST diff (<100ms) → Tier 2 Semantic (<1s) → Tier 3 Graph Impact (<5s)
- **Pluggable Engines**: Trait-based abstraction for parsers, graph builders, conflict detectors
- **Performance Targets**: <1s queries on 100k files, >90% cache hit rate, <10% incremental update time

### Gaps the Knowledge Layer Must Fill

| Gap | Current State | Required State |
|-----|--------------|----------------|
| **Atomic unit** | File-centric (FileId → GraphNodes) | Definition-centric (definitions as first-class entities) |
| **AI query interface** | None (agents read files) | Graph query API (MCP/RPC) for semantic traversal |
| **Cross-session persistence** | Per-invocation (lost between runs) | Persistent semantic memory across agent sessions |
| **Context compression** | Agent reads full files | System delivers minimal relevant subgraph |
| **Bidirectional sync** | One-way (files → graph) | Two-way (files ↔ graph, edits in either direction) |
| **Architectural patterns** | Not captured | Module clusters, API surfaces, dependency patterns |
| **Intent/specification** | Not captured | Natural language descriptions, behavioral contracts |
| **Projection engine** | Files only | Files, docs, diagrams, AI-optimized formats as views |

---

## Architectural Options

### Option A: Graph-Enhanced File Model (Evolutionary)

**Philosophy**: Keep files as source of truth. Add a persistent semantic graph as a queryable index layer on top. AI agents query the graph to locate relevant code, then read/edit files directly.

**Architecture**:
```
┌─────────────────────────────────────┐
│          AI Agent Interface          │
│   (MCP tools for graph queries)     │
└──────────────┬──────────────────────┘
               │ query
               ▼
┌─────────────────────────────────────┐
│        Semantic Graph Index          │
│  (Persistent, content-addressed)    │
│  Nodes: symbols, types, modules     │
│  Edges: calls, imports, extends     │
│  Storage: Postgres/D1               │
└──────────────┬──────────────────────┘
               │ derived from
               ▼
┌─────────────────────────────────────┐
│           Source Files               │
│  (Git = source of truth)            │
│  File edits → incremental reindex   │
└─────────────────────────────────────┘
```

**How It Works**:
1. Files are parsed via tree-sitter, symbols extracted, graph built — exactly as 001 spec describes
2. AI agents receive MCP tools like `query_callers(symbol)`, `query_dependencies(file)`, `find_similar(function)`
3. Graph queries return symbol locations; agent then reads/edits the specific files
4. File edits trigger incremental graph updates via Blake3 change detection

**Advantages**:
- **Minimal disruption**: Builds directly on 001 spec and existing thread-flow pipeline
- **Git compatibility**: Files remain the source of truth; standard tooling works unchanged
- **Adoption**: Zero learning curve for humans; AI agents get better context with no human-facing changes
- **Incremental delivery**: Each graph feature adds value independently
- **Constitutional alignment**: Fully compatible with all six principles

**Disadvantages**:
- **Still file-centric**: The fundamental abstraction hasn't changed — agents still read/edit files
- **Incomplete context compression**: Agent still needs file content for the "last mile" of understanding
- **No bidirectional graph edits**: Can't make structural changes (rename, move, refactor) through the graph
- **No definition-level caching**: Cache granularity remains at file level, not definition level
- **Limited AI-native operations**: Agents can query but not manipulate the graph directly

**Effort**: Low-Medium. Primarily extends 001 spec with query API layer.

---

### Option B: Content-Addressed Definition Store (Unison-Inspired)

**Philosophy**: Definitions (functions, types, modules) are the atomic unit, identified by AST hash. The database is the source of truth. Files are generated projections with bidirectional sync.

**Architecture**:
```
┌─────────────────────────────────────┐
│         Human Interface              │
│  (Text files as projections)        │
│  Edit → parse → store new hash      │
└──────────┬────────────▲─────────────┘
           │ ingest     │ project
           ▼            │
┌─────────────────────────────────────┐
│    Content-Addressed Definition DB   │
│  (Source of truth)                   │
│  Key: SHA3(AST of definition)       │
│  Value: typed AST + metadata        │
│  Names: separate table of pointers  │
│  Storage: Postgres/D1               │
└──────────┬────────────▲─────────────┘
           │ query      │ mutate
           ▼            │
┌─────────────────────────────────────┐
│        AI Agent Interface            │
│  (Direct graph manipulation)        │
│  Rename, move, refactor via API     │
│  Structural edits, not text edits   │
└─────────────────────────────────────┘
```

**How It Works**:
1. Each definition (function, type, module, trait) is hashed by its AST structure. `fn process(x: i32) -> bool { ... }` gets a unique hash regardless of whitespace, comments, or variable names.
2. Names are metadata — `process` is a pointer to hash `abc123`. Renaming is O(1) metadata update.
3. The graph connects definitions by hash. `fn caller()` calling `fn callee()` creates an edge between their hashes.
4. Humans edit "scratch files" (like Unison's `.u` files). On save, definitions are parsed, hashed, and stored. Changed definitions get new hashes; unchanged definitions keep their hashes — enabling perfect incremental caching.
5. AI agents work with definitions directly: "rename this definition", "move this function to that module", "add a parameter to this type". These are graph operations, not text operations.
6. Files are generated from the definition store on demand, formatted consistently.

**Advantages**:
- **True AI-native**: Agents manipulate semantic objects, not text strings
- **Perfect incremental caching**: Definition-level granularity means changing one function doesn't invalidate caching for the rest of the file
- **No merge conflicts on renames**: Renaming is a metadata operation that can't conflict with code changes
- **Structural refactoring**: Move function, extract module, change signature — all are graph operations with automatic downstream updates
- **512x context compression potential**: Following CGM's approach, graph structure can be encoded in attention masks
- **Deduplication**: Identical definitions across files/repos share a single hash

**Disadvantages**:
- **Massive implementation effort**: Requires building a full bidirectional sync engine, definition extraction for 20+ languages, and a projection/formatting pipeline
- **Ecosystem disruption**: Git diff, grep, blame, IDE integrations — all assume files. Every tool in the chain needs adaptation or a compatibility layer
- **Cross-language complexity**: Hashing definitions works differently for Rust (items), Python (indentation-scoped), JavaScript (hoisted declarations). Each language needs custom extraction logic.
- **Partial definitions**: Not all code fits neatly into "definitions" — top-level statements, module-level side effects, conditional compilation, macros
- **Adoption barrier**: Developers must trust a database they can't directly inspect with `cat` and `grep`
- **Constitution tension**: Constitution says "Git is the only SoT" (spec Assumption 16). This option makes the DB the SoT, requiring a constitutional amendment.

**Effort**: Very High. Multi-year effort to achieve production quality across languages.

---

### Option C: Multi-Resolution Knowledge Layer (Recommended)

**Philosophy**: Code is simultaneously represented at multiple levels of abstraction. Each level is content-addressed and incrementally updated. AI agents choose the appropriate resolution for each task. Files remain the editing interface; the knowledge layer is the intelligence interface.

**Architecture**:
```
┌──────────────────────────────────────────────────────────────┐
│                    PROJECTION ENGINE                          │
│  Files │ API Docs │ Dependency Diagrams │ AI Context Packs   │
└────────────────────────────▲─────────────────────────────────┘
                             │ render
┌──────────────────────────────────────────────────────────────┐
│                    KNOWLEDGE LAYER                            │
│                                                              │
│  Level 4: Intent/Contracts                                   │
│    Natural language descriptions, behavioral specs,          │
│    invariants, test contracts                                │
│    ┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈                  │
│  Level 3: Architectural Patterns                             │
│    Module clusters, API surfaces, dependency chains,         │
│    ownership boundaries, design patterns                     │
│    ┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈                  │
│  Level 2: Semantic Graph                                     │
│    Symbols + relationships (calls, imports, extends,         │
│    implements, uses, contains) — the 001 graph model         │
│    ┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈                  │
│  Level 1: Parsed Definitions                                 │
│    Content-addressed AST fragments per definition            │
│    (function, type, module, trait, impl block)               │
│    ┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈                  │
│  Level 0: File Index                                         │
│    Content-addressed file entries (Blake3 hash)              │
│    — the existing thread-flow cache layer                    │
│                                                              │
│  Each level: content-addressed, incrementally updated,       │
│  queryable independently, with upward/downward references    │
└──────────────────────────────▲───────────────────────────────┘
                               │ build/update
┌──────────────────────────────────────────────────────────────┐
│                    INGESTION PIPELINE                         │
│  File watcher → Blake3 diff → tree-sitter parse →           │
│  definition extraction → graph construction →                │
│  pattern detection → intent inference                        │
│  (ReCoco dataflow, incremental)                              │
└──────────────────────────────▲───────────────────────────────┘
                               │ watch
┌──────────────────────────────────────────────────────────────┐
│                    SOURCE FILES (Git)                         │
│  Human editing interface. Git remains source of record.      │
│  Files are authoritative for content; knowledge layer is     │
│  authoritative for relationships and meaning.                │
└──────────────────────────────────────────────────────────────┘
```

**How It Works**:

**Ingestion (files → knowledge layer)**:
1. File watcher detects changes (existing `thread-flow` Blake3 fingerprinting)
2. Changed files are parsed via tree-sitter (existing `thread-ast-engine`)
3. **Definition extraction** (new): AST is segmented into individual definitions (functions, types, modules, traits, impl blocks). Each definition gets its own content-addressed hash (Blake3 of its AST subtree).
4. **Graph construction** (from 001 spec): Definitions become nodes; relationships (calls, imports, extends, etc.) become edges
5. **Pattern detection** (new): Higher-level analysis identifies module clusters, API surfaces, architectural patterns
6. **Intent inference** (new, optional): LLM-assisted or rule-based extraction of behavioral contracts and natural language descriptions

**Querying (knowledge layer → AI agents)**:
- **Level 0**: "What files changed?" → Blake3 diff
- **Level 1**: "What definitions are in this file?" → Definition index lookup
- **Level 2**: "What calls this function? What depends on this type?" → Graph traversal
- **Level 3**: "What module does authentication? What's the API surface of the payments subsystem?" → Pattern query
- **Level 4**: "What is this function supposed to do? What invariants must hold?" → Intent/contract lookup

**Projection (knowledge layer → human-readable artifacts)**:
- **Files**: Standard source files (primary projection, bidirectional)
- **API docs**: Generated from Level 1-2 (definitions + relationships)
- **Dependency diagrams**: Generated from Level 2-3 (graph + patterns)
- **AI Context Packs**: Minimal subgraph for a specific task, optimized for LLM consumption (Level 1-3)

**AI Agent Operations**:
- **Read**: Query any level directly via MCP/RPC tools
- **Navigate**: Follow graph edges to discover related code without file searching
- **Understand**: Get architectural context (Level 3) before diving into details (Level 1)
- **Edit**: Edit files normally (through existing tools), knowledge layer updates incrementally
- **Refactor** (future): Structural operations (rename, move, extract) through Level 2 graph operations, projected back to file edits

**Advantages**:
- **Right level of abstraction for each task**: Bug fix? Level 1-2 (definitions + callers). Architecture review? Level 3. Understanding intent? Level 4. The agent doesn't over-fetch or under-fetch.
- **Incremental sophistication**: Level 0 exists today. Level 1-2 are straightforward extensions of 001. Level 3-4 can be added later without changing the architecture.
- **Git compatible**: Files remain the human interface and Git-tracked artifacts. Knowledge layer is a derived, persistent index — like a sophisticated `.git/index`.
- **Constitutional compliance**: Git is SoT for content. Knowledge layer is authoritative for *meaning*. This is an additive capability, not a replacement.
- **512x compression potential**: CGM-style attention masking can be built on Level 2-3 graph structure. An "AI Context Pack" delivers the relevant subgraph in ~1/100th the tokens of file reads.
- **Definition-level caching**: Level 1 gives per-definition content addressing. Changing one function doesn't invalidate the cache for sibling functions in the same file.
- **Cross-session persistence**: The knowledge layer persists across agent invocations. Agent can resume with full context by querying Levels 2-3.
- **Dual deployment**: Each level works in both CLI (Postgres + petgraph) and Edge (D1 + streaming iterators) following Thread's existing patterns.
- **MCP-native**: Each level maps cleanly to MCP tool definitions, enabling any AI agent (not just Thread-specific ones) to query the knowledge layer.

**Disadvantages**:
- **Complexity budget**: Five abstraction levels means five things to maintain, test, and keep consistent. Each level adds implementation and cognitive overhead.
- **Level 3-4 accuracy**: Architectural pattern detection and intent inference are inherently fuzzy. Wrong patterns or incorrect intent descriptions could mislead AI agents more than raw files would.
- **Definition extraction challenge**: Segmenting AST into "definitions" varies significantly across languages. Rust has clear items; Python has indentation-scoped blocks; JavaScript has hoisted declarations and IIFE patterns.
- **Update propagation latency**: Changes must flow through all levels. A file edit must update L0 → L1 → L2 → L3 → L4. Each level adds latency to the "time to consistent knowledge" metric.
- **Storage overhead**: Five levels of content-addressed data for every definition. For a 100k-file codebase, this could mean millions of Level 1 entries, tens of millions of Level 2 edges.
- **Query routing**: The agent (or a routing layer) must decide which level to query. Wrong level selection wastes tokens or returns irrelevant results.

**Effort**: Medium-High. Level 0-2 are achievable within the 001 spec timeline. Level 3-4 are follow-on work.

---

## Recommendation

### Option C: Multi-Resolution Knowledge Layer

Option C is recommended because it balances transformation with pragmatism:

1. **It builds on existing work**: Level 0 (file cache) exists in `thread-flow`. Level 2 (semantic graph) is the 001 spec. The knowledge layer adds Levels 1, 3, and 4 as new capabilities that extend rather than replace the existing architecture.

2. **It respects Git**: The Constitution and 001 spec both establish Git as the source of truth. Option C maintains this — files are authoritative for content, the knowledge layer is authoritative for meaning. This is an additive pattern, not a replacement.

3. **It delivers incrementally**: Levels can be built and shipped independently. Level 1 (definition extraction) alone provides significant value for AI agents by enabling per-definition caching and targeted context delivery. Level 2 (graph) enables navigation. Levels 3-4 are optional enhancements.

4. **It enables the future**: If Thread later wants to move toward Option B (full Unison model), the Level 1 definition store is the foundation. The multi-resolution architecture doesn't foreclose more radical evolution.

5. **It's what the market needs**: Every AI coding tool is independently building pieces of this (Augment's Context Engine, Cursor's embeddings, Sourcegraph's Code Graph). Thread can provide it as open infrastructure.

### How This Differs From 001

The 001-realtime-code-graph spec already designs Levels 0 and 2 (file index and semantic graph). The knowledge layer extends 001 with:

| Addition | 001 Spec | Knowledge Layer |
|----------|----------|----------------|
| Definition extraction (Level 1) | GraphNode is per-symbol but within file context | Definitions are independently content-addressed AST fragments |
| Architectural patterns (Level 3) | Not addressed | Module clusters, API surfaces, ownership boundaries |
| Intent/contracts (Level 4) | Not addressed | Natural language descriptions, behavioral specs |
| Projection engine | Files only | Multiple output formats (docs, diagrams, AI context packs) |
| AI query interface | RPC API for graph queries | Multi-level MCP tools with resolution selection |
| Context compression | Not addressed | AI Context Packs — minimal subgraphs for specific tasks |
| Cross-session persistence | Implied by overlay architecture | Explicit agent memory via Level 3-4 queries |

### Phased Delivery

| Phase | Levels | Deliverable | Value |
|-------|--------|-------------|-------|
| **Phase 1** (001 execution) | L0 + L2 | File index + semantic graph with overlay architecture | Graph queries, conflict detection, dependency tracking |
| **Phase 2** (Knowledge Layer v1) | L1 | Definition extraction + per-definition content addressing | Definition-level caching, targeted context for AI, 10-50x context compression |
| **Phase 3** (Knowledge Layer v2) | L2.5 | AI Context Pack generation + MCP tool suite | AI agents query the knowledge layer directly, eliminating file-read overhead |
| **Phase 4** (Knowledge Layer v3) | L3 | Architectural pattern detection | Module-level understanding, ownership boundaries, design pattern recognition |
| **Phase 5** (Knowledge Layer v4) | L4 | Intent inference + behavioral contracts | Specification-level understanding, automated contract verification |

---

## Key Design Decisions

### 1. Atomic Unit: Definition, not File

**Decision**: The primary unit of content-addressing and caching is the **definition** (function, type, module, trait, impl block), not the file.

**Rationale**: A single file often contains 10-50 definitions. When one definition changes, file-level caching invalidates the cache for all 50. Definition-level caching invalidates only the changed definition. For AI agents, this means:
- Requesting "the callers of function X" returns the definition of X and its callers — not the entire files containing them
- Context packs are 10-50x smaller than equivalent file reads
- Incremental updates are more granular — changing a comment in function A doesn't re-analyze function B in the same file

**Implementation**: Tree-sitter parse → walk AST → identify top-level items (functions, types, modules, impls, traits) → hash each item's AST subtree independently. Store with metadata (file provenance, byte range, line range) for projection back to files.

**Language-Specific Considerations**:
- **Rust**: Clean item boundaries (`fn`, `struct`, `enum`, `trait`, `impl`, `mod`). Easiest language.
- **TypeScript/JavaScript**: Export statements, class declarations, function declarations, const assignments. Hoisting requires careful boundary detection.
- **Python**: Class and function definitions at module level. Indentation-based scoping. Top-level statements treated as a "module-init" definition.
- **Go**: Package-level functions, types, interfaces. Methods are top-level but associated with types via receiver.

### 2. Source of Truth: Dual Authority

**Decision**: Git/files are authoritative for **content**. The knowledge layer is authoritative for **meaning** (relationships, patterns, intent).

**Rationale**: This avoids the constitutional conflict of replacing Git as SoT while acknowledging that the knowledge layer contains information that doesn't exist in files (architectural patterns, cross-file relationships, behavioral contracts). The knowledge layer is a **derived but persistent** index — like a search index that survives restarts.

**Implications**:
- File edits are always valid and always trigger knowledge layer updates
- Knowledge layer queries are always consistent with the latest file state (within propagation latency)
- If the knowledge layer is corrupted or lost, it can be fully rebuilt from files (recovery guarantee)
- The knowledge layer may contain information not derivable from files alone (Level 4 intent annotations) — these are treated as supplementary metadata, not authoritative content

### 3. AI Interface: MCP Tools with Level Selection

**Decision**: AI agents interact with the knowledge layer via MCP (Model Context Protocol) tools, with explicit level selection.

**Rationale**: MCP is the emerging standard for AI tool integration. It's supported by Claude, and increasingly by other agents. By exposing the knowledge layer as MCP tools, Thread becomes a universal intelligence backend for any MCP-compatible agent.

**Proposed MCP Tool Suite**:

```
# Level 0: File operations (existing)
thread_files_changed(since: hash) -> [FileChange]
thread_file_content(path: string) -> string

# Level 1: Definition operations
thread_definitions(file: string) -> [Definition]
thread_definition_by_hash(hash: string) -> Definition
thread_definition_search(query: string, language?: string) -> [Definition]

# Level 2: Graph operations
thread_callers(symbol: string) -> [CallerInfo]
thread_callees(symbol: string) -> [CalleeInfo]
thread_dependencies(symbol: string, depth?: int) -> SubGraph
thread_dependents(symbol: string, depth?: int) -> SubGraph
thread_imports(file: string) -> [ImportInfo]
thread_type_hierarchy(type: string) -> TypeTree

# Level 3: Architecture operations
thread_module_surface(module: string) -> APISurface
thread_ownership_boundary(path: string) -> OwnershipInfo
thread_similar_patterns(symbol: string) -> [PatternMatch]
thread_affected_by_change(symbol: string) -> ImpactAnalysis

# Level 4: Intent operations
thread_intent(symbol: string) -> IntentDescription
thread_contracts(symbol: string) -> [Contract]
thread_invariants(module: string) -> [Invariant]

# Meta: Context packs
thread_context_pack(task: string, scope?: string) -> ContextPack
```

### 4. Context Pack Format

**Decision**: The knowledge layer generates "AI Context Packs" — pre-assembled, token-optimized subgraphs for specific tasks.

**Rationale**: Rather than making the AI agent issue 10-20 individual queries to build context, the knowledge layer assembles the relevant subgraph proactively. This is the key to context compression.

**Format**:
```
ContextPack {
  // Task description that generated this pack
  task: string,

  // Definitions relevant to the task, ordered by relevance
  definitions: [Definition],

  // Relationships between included definitions
  edges: [Edge],

  // Architectural context (which modules, what patterns)
  architecture: ArchitecturalContext,

  // Token budget used / remaining
  token_estimate: u32,

  // What was excluded and why (for agent to request more if needed)
  excluded: [ExclusionReason],
}
```

**Example**: Agent asks `thread_context_pack(task: "add rate limiting to the /api/payments endpoint")`:
- Level 3 identifies the payments module and its API surface
- Level 2 finds the endpoint handler, its dependencies, middleware chain, and auth flow
- Level 1 includes the definitions of each relevant function
- Result: ~2,000 tokens instead of ~50,000 tokens from reading all relevant files

### 5. Storage Architecture

**Decision**: Each level uses the appropriate storage backend, following Thread's existing multi-backend pattern.

| Level | CLI (Postgres) | Edge (D1) | Vector (Qdrant) |
|-------|---------------|-----------|-----------------|
| L0: File Index | `files` table | `files` table | — |
| L1: Definitions | `definitions` table (AST as JSONB) | `definitions` table (AST as JSON) | Definition embeddings |
| L2: Graph | `nodes` + `edges` tables (petgraph in-memory) | `nodes` + `edges` + `reachability` tables (streaming) | — |
| L3: Patterns | `patterns` table | `patterns` table | Pattern embeddings |
| L4: Intent | `intents` table | `intents` table | Intent embeddings |

### 6. Update Propagation Model

**Decision**: Updates flow bottom-up (L0 → L1 → L2 → L3 → L4) with level-specific latency targets.

| Transition | Trigger | Target Latency | Mechanism |
|-----------|---------|---------------|-----------|
| File → L0 | File system event | <10ms | Blake3 hash comparison |
| L0 → L1 | File hash changed | <100ms | Tree-sitter incremental parse + definition extraction |
| L1 → L2 | Definition hash changed | <500ms | Graph edge update (affected edges only) |
| L2 → L3 | Graph topology changed | <5s | Pattern re-detection (batch, debounced) |
| L3 → L4 | Pattern changed | <30s | Intent re-inference (async, background) |

Lower levels update synchronously (blocking); higher levels update asynchronously (eventually consistent). AI agents see L0-L2 changes in near-real-time; L3-L4 changes are best-effort.

---

## Implementation Strategy

### Alignment with 001 Spec

The knowledge layer is designed as an **extension** of the 001-realtime-code-graph spec, not a replacement:

```
001 Spec Scope          Knowledge Layer Extension
─────────────────       ─────────────────────────
Level 0: File Index     ← Exists in thread-flow
Level 1: Definitions    ← NEW (definition extraction)
Level 2: Semantic Graph ← Exists in 001 (GraphNode, GraphEdge)
Level 3: Patterns       ← NEW (architectural detection)
Level 4: Intent         ← NEW (intent inference)
Projection Engine       ← NEW (multi-format output)
MCP Tool Suite          ← NEW (AI agent interface)
```

### Proposed Crate Organization

Building on the 001 spec's proposed crate structure:

```
crates/
├── thread-ast-engine/     # EXISTING: AST parsing (tree-sitter)
├── thread-language/       # EXISTING: Language definitions
├── thread-rule-engine/    # EXISTING: Rule-based scanning
├── thread-flow/           # EXISTING: Dataflow orchestration (ReCoco)
├── thread-utils/          # EXISTING: Shared utilities
├── thread-wasm/           # EXISTING: WASM bindings
│
├── thread-graph/          # FROM 001: Core graph structures + algorithms
├── thread-indexer/        # FROM 001: Multi-source code indexing
├── thread-conflict/       # FROM 001: Conflict detection engine
├── thread-storage/        # FROM 001: Multi-backend persistence
├── thread-api/            # FROM 001: RPC/API layer
│
├── thread-definitions/    # NEW (L1): Definition extraction + content-addressing
│   ├── src/
│   │   ├── extract.rs     # Language-specific definition boundary detection
│   │   ├── hash.rs        # AST subtree content-addressing
│   │   ├── store.rs       # Definition CRUD with storage backends
│   │   └── languages/     # Per-language extraction rules
│   └── tests/
├── thread-knowledge/      # NEW (L3-L4): Architectural patterns + intent
│   ├── src/
│   │   ├── patterns.rs    # Module cluster detection, API surface extraction
│   │   ├── intent.rs      # Behavioral contract extraction
│   │   └── inference.rs   # LLM-assisted intent inference (optional)
│   └── tests/
├── thread-projection/     # NEW: Multi-format output generation
│   ├── src/
│   │   ├── files.rs       # Source file projection (bidirectional sync)
│   │   ├── context.rs     # AI Context Pack generation
│   │   ├── docs.rs        # API documentation generation
│   │   └── diagrams.rs    # Dependency diagram generation
│   └── tests/
└── thread-mcp/            # NEW: MCP tool server
    ├── src/
    │   ├── server.rs      # MCP server implementation
    │   ├── tools/         # Tool handlers per level
    │   └── routing.rs     # Query routing + level selection
    └── tests/
```

### Dependency Graph

```
thread-mcp
  ├── thread-projection
  │     ├── thread-knowledge (L3-L4)
  │     │     ├── thread-graph (L2)
  │     │     │     ├── thread-definitions (L1)
  │     │     │     │     ├── thread-ast-engine
  │     │     │     │     └── thread-language
  │     │     │     └── thread-storage
  │     │     └── thread-graph
  │     └── thread-definitions
  └── thread-api (RPC layer)
```

All arrows flow downward — no circular dependencies (Constitution Principle IV).

---

## Risk Analysis

### Technical Risks

| Risk | Severity | Likelihood | Mitigation |
|------|----------|-----------|-----------|
| **Definition extraction accuracy varies by language** | High | High | Start with Rust (cleanest boundaries), expand incrementally. Accept 90% accuracy for dynamic languages initially. |
| **Update propagation latency exceeds targets** | Medium | Medium | Debounce higher levels (L3-L4). Make latency targets per-level, not end-to-end. L0-L2 are the critical path. |
| **Storage overhead exceeds 1.5x target** | Medium | Low | Definition deduplication across files. Lazy computation of L3-L4 (only on query, not on every update). Configurable level depth. |
| **Context Pack quality varies** | Medium | High | Start with rule-based pack assembly (Level 2 graph traversal). Add ML-based relevance ranking in Phase 5. Always include "excluded" list so agent can request more. |
| **MCP protocol limitations** | Low | Medium | MCP is still evolving. Design tool interfaces to be protocol-agnostic internally; MCP is just one transport. |

### Adoption Risks

| Risk | Severity | Likelihood | Mitigation |
|------|----------|-----------|-----------|
| **AI agents don't use the knowledge layer** | Critical | Low | MCP is the emerging standard. Context packs provide immediate, measurable value (10-50x compression). |
| **Developers don't trust non-file representations** | Medium | Medium | Files remain the editing interface. Knowledge layer is invisible to developers unless they choose to query it. |
| **Level 3-4 produce misleading results** | High | Medium | Mark L3-L4 results with confidence scores. Make them optional. Never present inferred intent as authoritative. |

### Constitutional Risks

| Risk | Mitigation |
|------|-----------|
| Principle I (Service-Library): Knowledge layer must serve both library API and service deployment | Each level is a library crate; service layer orchestrates updates and serves queries |
| Principle III (TDD): Complex definition extraction logic needs comprehensive tests | Property-based testing for extraction; golden-file tests per language; round-trip tests (extract → project → extract = identity) |
| Principle VI (Service): Must meet storage and cache performance targets | Definition-level caching improves cache hit rates (more granular = fewer invalidations). Storage schema designed for query patterns. |

---

## Open Questions

1. **Definition boundary heuristics**: How do we handle language constructs that don't fit cleanly into "definition" boundaries?
   - Rust macros that generate definitions
   - Python's module-level imperative code
   - JavaScript's IIFE patterns and dynamic exports
   - C/C++ preprocessor directives

2. **Level 3 pattern detection**: What patterns are most valuable for AI agents?
   - Module ownership boundaries
   - Error handling patterns
   - State management patterns
   - API versioning conventions
   - Should this be configurable per-project?

3. **Level 4 intent source**: Where does intent come from?
   - Existing docstrings/comments
   - Test names and assertions
   - Commit messages
   - LLM inference
   - Manual annotation
   - Some combination?

4. **Context Pack assembly algorithm**: How does the system decide what to include?
   - Fixed-depth graph traversal from focal point?
   - Relevance-weighted selection within token budget?
   - Task-type-specific heuristics (bug fix vs. feature vs. refactor)?
   - Should the agent specify the budget, or should the system optimize?

5. **Edge deployment scope**: Which levels are available on edge (WASM)?
   - L0-L2 seem feasible within 128MB memory constraints
   - L3 pattern detection may require full graph access (CLI only?)
   - L4 intent inference requires LLM access (API call from edge?)

6. **Bidirectional sync complexity**: When an AI agent makes a structural change through the graph API (future Phase), how do we project it back to files?
   - Simple cases (rename, move) are mechanical
   - Complex cases (extract function, change signature with callers) require formatting decisions
   - Should this be deferred entirely until Option B becomes viable?

7. **Versioning**: How does the knowledge layer handle branches?
   - One knowledge layer instance per branch?
   - Shared base with branch-specific overlays (extending 001's overlay architecture)?
   - How does branch switching work — full rebuild or incremental diff?

---

## Appendix A: Comparison Matrix

| Dimension | Option A (Evolutionary) | Option B (Unison) | Option C (Multi-Resolution) |
|-----------|------------------------|-------------------|---------------------------|
| **Atomic unit** | File | Definition | Definition (L1) + File (L0) |
| **Source of truth** | Git/files | Database | Git for content, KL for meaning |
| **AI interface** | Graph query → file read | Direct graph manipulation | Multi-level MCP tools |
| **Human interface** | Unchanged | Scratch files | Unchanged |
| **Context compression** | 2-5x (targeted file reads) | 50-500x (definition-level) | 10-100x (context packs) |
| **Git compatibility** | Full | Requires adaptation | Full |
| **Implementation effort** | Low-Medium | Very High | Medium-High |
| **Definition-level caching** | No | Yes | Yes (Level 1) |
| **Architectural awareness** | No | No | Yes (Level 3) |
| **Intent/contracts** | No | No | Yes (Level 4) |
| **Bidirectional sync** | N/A (files are truth) | Required (complex) | Optional (future) |
| **Constitutional compliance** | Full | Requires amendment | L0-L2: Full; L3-L4: Partial (TDD tension) |
| **Incremental delivery** | Yes | No (big bang) | Yes (level by level) |
| **Risk profile** | Low | Very High | Medium |

## Appendix B: Landscape Research Sources

### Semantic Code Graphs
- Google Kythe (kythe.io) — language-agnostic cross-reference graphs
- Sourcegraph SCIP — Protobuf-based code intelligence protocol
- Meta Glean — general-purpose fact collection with Angle query language
- Code-Graph-RAG — tree-sitter + knowledge graphs + UniXcoder embeddings
- CodeGraph Analyzer — Neo4j "digital twin" of codebases
- GraphGen4Code (IBM) — RDF knowledge graphs for code

### Content-Addressed Code
- Unison Language (unison-lang.org) — 512-bit SHA3 hashed definitions in SQLite
- Git object model — content-addressed blobs/trees (file-level, not definition-level)
- Nix/Guix — content-addressed package management

### AI-Integrated Code Models
- Code Graph Model (CGM, NeurIPS 2025) — graph structure in LLM attention masks, 512x compression
- Knowledge Graph-Based Repository-Level Code Generation (arXiv 2505.14394)

### Projectional Editors
- JetBrains MPS — AST Graph with per-node UUIDs, multiple notations
- Intentional Software (Simonyi) — intent/implementation/presentation separation
- Darklang — language + structured editor + infrastructure
- Pharo/Smalltalk — image-based development with Iceberg Git export

### AI Agent Platforms
- Claude Code — bash + grep (LCD interface)
- Augment Code — persistent semantic embeddings + dependency graphs + commit lineage
- Cursor — embeddings + AST graphs + contextual cross-references
- Sourcegraph Amp — Code Graph + Context Engine for AI agents

### Code Intelligence Platforms
- CodeQL (GitHub) — relational database from compiler instrumentation
- Semgrep — tree-sitter CST → Generic AST → IL pipeline
- Sourcegraph — SCIP + Zoekt + Code Graph

---

## Appendix C: Specialist Review Findings

Three independent specialist reviews were conducted to identify planning gaps and architectural challenges. Their findings are synthesized below, organized by theme, with specific revisions to the original design.

### Review Panel

1. **Rust Systems Architect** — focused on implementation feasibility, content-addressing mechanics, storage overhead, edge deployment constraints
2. **AI Agent Integration Specialist** — focused on MCP tool design, context pack quality, agent workflows, failure modes
3. **Product/Architecture Strategist** — focused on strategic positioning, adoption risks, phasing realism, complexity budget

---

### Finding 1: Storage Overhead Is Dramatically Underestimated

**Source**: Rust Systems Architect

The report's 1.5x storage target is not achievable. Quantitative analysis for a 100k-file Rust codebase (~10M LOC, ~400MB raw):

| Level | Storage Estimate | Notes |
|-------|-----------------|-------|
| L0 (File Index) | ~20MB | 100k entries x 200 bytes |
| L1 (Definitions) | 750MB-3.75GB | 1.5M definitions; 750MB metadata-only, 3.75GB with stored ASTs |
| L2 (Graph) | 1.2-2GB | 1.5M nodes + 7.5-15M edges |
| L3 (Patterns) | <100MB | Thousands of patterns |
| **Total** | **2-6GB** | **5-15x raw code, not 1.5x** |

**Required Design Revision**: Do NOT store ASTs per definition. Store only `(content_hash, byte_range, file_id, name, kind)` metadata. Reconstruct ASTs on demand by re-parsing the containing file and extracting the byte range. Use an LRU cache for hot definitions. This reduces L1 to ~750MB (~1.9x raw). **Revised storage target: 3-5x raw code size for L0-L2 combined.**

For D1 (edge), the full L2 graph cannot be loaded into memory under 128MB. L1 must also be queried from D1, not held in memory. An LRU cache of ~10MB (covering ~20k definitions) handles most query working sets.

---

### Finding 2: Context Pack Assembly Needs a Two-Phase Protocol

**Source**: AI Agent Integration Specialist

The one-shot `thread_context_pack(task: string)` design has a fundamental flaw: relevance cannot be determined outside the agent's reasoning loop. If the system guesses wrong about what's relevant, the agent wastes its entire context window on irrelevant definitions.

**Required Design Revision**: Replace single context pack with two-phase protocol:

1. **`thread_context_plan(focal_point, depth, budget)`** — returns a *manifest* of what would be included (definition names, token costs, relevance scores). Approximately 200-400 tokens.
2. **`thread_context_fetch(selections: [hash])`** — returns the actual content of selected definitions.

Additionally, provide two modes:
- **Structural mode** (reliable): `thread_context_plan(focal_point: "process_payment", depth: 2)` — returns 2-hop graph neighborhood. No relevance ranking needed.
- **Task mode** (best-effort): `thread_context_plan(task: "add rate limiting", scope: "src/payments/")` — attempts relevance ranking within agent-specified scope.

The structural mode should be the default. Task mode is explicitly marked as best-effort with accuracy caveats.

**Revised compression estimate**: Realistic compression is **5-15x** for targeted queries, degrading to **2-5x** for exploratory work. The 100x claim is achievable only for L3 architectural overview queries (narrow use case).

---

### Finding 3: MCP Tool Suite Should Be 3 Tiers, Not 5 Levels

**Source**: AI Agent Integration Specialist, Product Strategist

20 tools across 5 levels creates a two-step routing decision (which level, then which tool) that compounds LLM reasoning errors. In practice, agents will gravitate toward `thread_context_pack` for 90% of interactions.

**Required Design Revision**: Restructure into 3 tiers:

| Tier | Tools | Purpose |
|------|-------|---------|
| **Primary** (4 tools) | `thread_context_plan`, `thread_context_fetch`, `thread_search`, `thread_locate` | 90% of interactions |
| **Navigation** (5 tools) | `thread_callers`, `thread_callees`, `thread_dependencies`, `thread_dependents`, `thread_type_hierarchy` | Targeted graph traversal when context plan is insufficient |
| **Introspection** (3 tools) | `thread_status`, `thread_definitions_changed`, `thread_affected_by_change` | Health checks, session management, impact analysis |

**Total: 12 tools** (down from ~20). L4 intent/contract tools folded into context plan metadata, not exposed as separate tools.

**Critical missing tools identified**:
- **`thread_status()`**: Returns indexing progress, level availability, staleness — required for graceful degradation
- **`thread_definitions_changed(since: hash)`**: Required for cross-session persistence claim
- **`thread_locate(hash)`**: Bridges knowledge layer representation back to file path + line number for editing
- **`thread_context_from_diagnostic(error: string)`**: Handles the highest-frequency agent workflow (starting from an error message)
- **Pagination**: All tools returning lists need `cursor` parameter and `thread_more(cursor)` continuation

---

### Finding 4: L3-L4 Are Research Projects, Not Engineering Phases

**Source**: All three reviewers (unanimous)

L3 (Architectural Pattern Detection) and L4 (Intent Inference) are poorly defined, non-deterministic, and not amenable to TDD (Constitution Principle III).

- **L3**: "Detect that these 15 files form an authentication module" has no objectively correct answer. This is an open research question in software architecture recovery (studied since the 1990s: Koschke 2009, Garcia et al. 2013).
- **L4**: Intent inference from docstrings is trivial text extraction; from LLM inference it's non-deterministic. Neither meets TDD requirements.
- **Constitutional violation**: Principle III mandates TDD for all development. L3-L4 cannot be meaningfully TDD'd with red-green-refactor cycles.

**Required Design Revision**:
- Relabel Phases 4-5 as **"Research & Exploration"** with explicit success/failure criteria
- Define 2-week spikes that produce prototypes. If prototype demonstrates measurable value (e.g., "AI agent with L3 context resolves 20% more issues"), proceed. If not, cut.
- L3 should be computed **lazily** (on query, not on every update) to avoid cascade problems
- When implemented, L3 pattern detection should use existing graph algorithms (community detection, fan-in/fan-out analysis) composed on the L2 graph via petgraph, not a separate inference engine

---

### Finding 5: No Graceful Degradation Path

**Source**: AI Agent Integration Specialist

The architecture assumes the knowledge layer is always available and correct. Five failure scenarios are unaddressed:

1. **First use**: Repository never indexed. All L1+ tools return empty.
2. **Indexing in progress**: Initial parse takes minutes for 100k files. L1-L2 data is partial.
3. **Storage failure**: Postgres/D1 down.
4. **Stale after offline period**: Graph based on week-old commit.
5. **Corruption**: Incorrect graph edges from language edge cases (macros, dynamic dispatch).

**Required Design Revision**: Design "hybrid tools" with transparent fallback:
- `thread_search(name)` → check graph (L2) → fall back to tree-sitter scan → fall back to grep
- All results annotated with `source: "graph" | "parse" | "text_search"` for confidence signaling
- `thread_status()` returns per-level availability so agents make informed fallback decisions
- For dynamic languages (Python, JavaScript, Ruby), graph results carry `dynamic_dispatch_risk: bool` flag

This makes the knowledge layer an **accelerator** rather than a **requirement**. The system always works; it just works faster with the knowledge layer populated.

---

### Finding 6: Definition Extraction Should Use tree-sitter Queries, Not Custom Extractors

**Source**: Rust Systems Architect, Product Strategist

Building custom per-language definition extractors for 20+ languages is a multi-engineer-year effort. Tree-sitter already solves this.

**Options**:

| Approach | Effort | Coverage | Depth |
|----------|--------|----------|-------|
| Custom extractors | Multi-year | 20+ languages | Full control |
| tree-sitter `tags.scm` | Weeks | All tree-sitter languages | Definitions + basic types |
| SCIP indexers | Months | 5-6 languages | Full semantics + cross-references |

**Required Design Revision**: Use tree-sitter `tags.scm` queries for L1 definition extraction. These ship with every major grammar and identify definition boundaries (functions, classes, methods) out of the box. Build custom extractors only for features not covered by `tags.scm`.

Additionally, tier definition extraction quality:
- **Tier A** (Full extraction): Rust, Go — clean item boundaries, custom extractors justified
- **Tier B** (Major definitions): Python, TypeScript, Java — functions + classes via `tags.scm`
- **Tier C** (File-level only): C/C++, Bash, CSS, HTML, JSON, YAML — treat entire file as one definition

Tier C languages still benefit from L0 caching and L2 import-level graph edges.

---

### Finding 7: Complexity Budget Is Exceeded — Aggressive Merging Required

**Source**: Product Strategist

The proposal adds 4 new crates + 001 spec's 6 new crates = **10 new crates** for a project with 7 existing crates. This more than doubles the codebase surface area.

**Required Design Revision**:

| Proposed | Revised | Rationale |
|----------|---------|-----------|
| `thread-definitions` | **Keep** | Core new capability, clear library crate |
| `thread-knowledge` | **Cut** | Research project. Pattern detection is graph algorithms on L2 — belongs in `thread-graph` |
| `thread-projection` | **Merge into `thread-api`** | Context packs are a query response format, not a separate concern |
| `thread-mcp` | **Merge into `thread-api`** | MCP is a transport protocol alongside RPC, not a separate layer |

**Net result: 1 new crate** (`thread-definitions`) beyond what the 001 spec already proposes. The MCP server, context pack generation, and projection logic live as modules within `thread-api`.

---

### Finding 8: Cross-Session Persistence Claim Is Overstated

**Source**: AI Agent Integration Specialist

The knowledge layer persists *codebase state* (graph, patterns) but not *agent state* (current task, working hypothesis, exploration path, decisions). "Pick up where you left off" actually means "start over with a better index," which is a real improvement but not the claimed full-context restoration.

**Required Design Revision**:
- Revise claim from "full context restoration" to "persistent codebase understanding that eliminates re-navigation overhead"
- Note that true cross-session persistence would require an "agent session" entity storing task description, focal points, explored definitions, and hypotheses — this is a future capability (a potential L5), not part of the initial design
- The `thread_definitions_changed(since: hash)` tool partially addresses this by letting agents efficiently detect what changed since their last interaction

---

### Finding 9: Multi-Agent Coordination Needs Optimistic Concurrency

**Source**: AI Agent Integration Specialist

AI agents create fundamentally different concurrency patterns than human developers: edits every few seconds (not minutes), 40+ files at once (not 1-3), and graph-level coordination needs (not file-level).

**Required Design Revision**: Add optimistic concurrency control:
1. Context plans include `graph_version: u64` (monotonically increasing)
2. When the agent's edit triggers a knowledge layer update, the system checks whether the graph has advanced past the version the agent was working with
3. If the affected subgraph has changed, the system flags the potential inconsistency in `thread_status()` or returns a warning on the next `thread_context_plan` call

This is analogous to ETags in HTTP or `expectedVersion` in event sourcing.

---

### Finding 10: First Consumer Must Be Thread Itself

**Source**: Product Strategist

The "open infrastructure for all AI agents" positioning historically loses (Kythe, SCIP, Glean all failed at external adoption). The first consumer must be Thread's own CLI.

**Required Design Revision — Adoption Path**:

1. **Month 1-3**: Thread CLI uses L0-L2 internally. `thread analyze` queries the graph instead of re-parsing.
2. **Month 3-5**: Ship MCP server as `thread serve --mcp`. Claude Code users add Thread as an MCP server.
3. **Month 5-8**: Run validation benchmark (20-30 coding tasks, 5+ repos). Measure: tokens consumed, task completion rate, correctness. Target: 50%+ token reduction AND equal or better correctness.
4. **Month 8+**: If benchmarks pass, pitch to AI agent framework developers.

**Minimum Viable Knowledge Layer (MVKL)**:
1. Content-addressed file index (L0) — exists
2. Definition extraction with per-definition hashing (L1) — via tree-sitter `tags.scm`
3. Caller/callee graph (L2, subset) — from 001 spec
4. 5 MCP tools: `thread_context_plan`, `thread_context_fetch`, `thread_search`, `thread_callers`, `thread_status`
5. 4 languages: Rust, TypeScript, Python, Go
6. 1 storage backend: Postgres (CLI only)

Ship the MVKL, validate with benchmarks, then expand.

---

### Finding 11: OSS/Commercial Boundary Is Undefined

**Source**: Product Strategist

**Required Design Revision — Recommended Boundary**:

| Component | License | Rationale |
|-----------|---------|-----------|
| L0-L2 (File Index + Definitions + Graph) | OSS (AGPL) | Adoption engine |
| MCP Server (L0-L2 tools) | OSS (AGPL) | Drives MCP ecosystem adoption |
| L3 (Patterns) — when built | Commercial | Differentiated intelligence |
| L4 (Intent) — when built | Commercial | Requires LLM integration, high value |
| Advanced Context Packs (with L3-L4) | Commercial | Premium feature |
| Full Edge Deployment | Commercial | Per 001 spec |

This follows the open-core model: knowledge layer drives adoption (OSS), intelligence on top drives monetization (commercial).

---

### Revised Risk Matrix (Post-Review)

| Risk | Original Severity | Revised Severity | Key Mitigation |
|------|-------------------|------------------|----------------|
| Definition extraction across languages | Medium-High | **Medium** (mitigated by `tags.scm`) | Use tree-sitter queries, not custom extractors |
| Storage overhead | Medium | **High** (was underestimated 4-10x) | Metadata-only L1, no stored ASTs |
| Context pack quality | Medium | **High** (central value prop) | Two-phase protocol (manifest → fetch) |
| L3-L4 feasibility | Medium | **Critical** (research, not engineering) | Relabel as research spikes with go/no-go |
| Knowledge layer unavailability | Not assessed | **High** | Hybrid tools with transparent fallback |
| Adoption without validation | Not assessed | **Critical** | MVKL in 8 weeks + benchmark suite |
| Complexity exceeds team capacity | Not assessed | **High** | 1 new crate, not 4; defer L3-L4 |
| Competitive timing | Not assessed | **High** | Differentiate on OSS + self-hostable + edge |

---

### Revised Phasing (Post-Review)

| Phase | Scope | Timeline | Deliverable |
|-------|-------|----------|-------------|
| **Phase 1** | 001 execution (L0 + L2) | Per 001 plan | File index + semantic graph + overlay architecture |
| **Phase 2** (parallel with P1) | L1 + MVKL | 6-8 weeks | Definition extraction via `tags.scm` + 5 MCP tools + Postgres |
| **Phase 3** | Validation | 4 weeks | Benchmark suite: 20-30 tasks, 5+ repos, measure token reduction + correctness |
| **Phase 4** | Expansion | Based on P3 results | Additional languages, D1 backend, advanced context packs |
| **Phase R1** | Research spike: L3 | 2 weeks | Prototype pattern detection. Go/no-go decision. |
| **Phase R2** | Research spike: L4 | 2 weeks | Prototype intent inference. Go/no-go decision. |

Key changes from original phasing:
- Phase 1 and Phase 2 run **in parallel** (they are independent)
- Phase 3 is **validation**, not more building — must prove value before expanding
- Phases 4-5 split into expansion (engineering, conditional on P3) and research spikes (go/no-go)

---

**Document Status**: Draft with specialist review incorporated
**Next Steps**: Decision on proceeding to feature specification (speckit workflow) for the MVKL scope
