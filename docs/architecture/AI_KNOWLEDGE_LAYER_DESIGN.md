<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: AGPL-3.0-or-later
-->

# AI-Native Knowledge Layer: Architectural Design Report

**Version**: 0.2.0 (Post-Review Draft)
**Date**: 2026-02-21
**Status**: Options Analysis — Revised per specialist review and Sourcery feedback
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
10. [Open Questions (MVKL Scope)](#open-questions-mvkl-scope)
11. [Future Work / Research](#future-work--research)

---

## Executive Summary

This document explores architectural options for an **AI-native knowledge layer** — a new abstraction that replaces the file as the primary unit of code intelligence. The goal is to design a system where AI agents interact with code through semantically rich, graph-structured representations rather than flat text files, while preserving full human oversight through familiar file-based workflows.

Three architectural options are analyzed, spanning from evolutionary (building on the existing 001-realtime-code-graph spec) to transformative (Unison-inspired content-addressed definition stores). The recommended approach is **Option C: Multi-Resolution Knowledge Layer** — a layered architecture where code is simultaneously represented at multiple levels of abstraction, each content-addressed and incrementally updated, with AI agents selecting the appropriate resolution for each task. The **Minimum Viable Knowledge Layer (MVKL)** focuses on Levels 0-2 (file index, parsed definitions, semantic graph) with a 3-tier MCP tool suite, validated by benchmarks before expansion. Higher levels (L3 architectural patterns, L4 intent) are deferred to time-boxed research spikes.

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
- Context compression: deliver the semantically relevant subgraph for a task in 5-15x fewer tokens than reading equivalent files (see [Appendix C, Finding 2](#finding-2-context-pack-assembly-needs-a-two-phase-protocol) for validation of this estimate)
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

**Meta Glean**: General-purpose fact collection using RocksDB and declarative Angle queries. Indexes diffs as "diff sketches" for semantic search over commits. Hundreds of microseconds query latency at massive scale.

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
- **512x compression potential via attention masking** (research): Following CGM's approach, graph structure can be encoded in LLM attention masks — a fundamentally different technique from context packs
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
│                    QUERY / PROJECTION LAYER                   │
│  MCP Tools │ AI Context Packs │ Files │ API Docs             │
└────────────────────────────▲─────────────────────────────────┘
                             │ render / query
┌──────────────────────────────────────────────────────────────┐
│                    KNOWLEDGE LAYER                            │
│                                                              │
│  ┄┄┄┄┄┄ Future Research (see Future Work section) ┄┄┄┄┄┄   │
│  Level 4: Intent/Contracts (research spike)                  │
│  Level 3: Architectural Patterns (research spike)            │
│  ┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄  │
│                                                              │
│  Level 2: Semantic Graph ◄── MVKL scope                      │
│    Symbols + relationships (calls, imports, extends,         │
│    implements, uses, contains) — the 001 graph model         │
│    ┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈                  │
│  Level 1: Parsed Definitions ◄── MVKL scope                  │
│    Content-addressed metadata per definition                 │
│    (function, type, module, trait, impl block)               │
│    ┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈                  │
│  Level 0: File Index ◄── exists                               │
│    Content-addressed file entries (Blake3 hash)              │
│    — the existing thread-flow cache layer                    │
│                                                              │
│  L0-L2: content-addressed, incrementally updated,            │
│  queryable independently, with upward/downward references    │
└──────────────────────────────▲───────────────────────────────┘
                               │ build/update
┌──────────────────────────────────────────────────────────────┐
│                    INGESTION PIPELINE                         │
│  File watcher → Blake3 diff → tree-sitter parse →           │
│  definition extraction (tags.scm) → graph construction       │
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
3. **Definition extraction** (new): tree-sitter `tags.scm` queries identify definition boundaries (functions, types, modules, traits, impl blocks). Each definition gets a content-addressed hash (Blake3). Metadata stored; ASTs reconstructed on demand.
4. **Graph construction** (from 001 spec): Definitions become nodes; relationships (calls, imports, extends, etc.) become edges

**Querying (knowledge layer → AI agents)**:
- **Level 0**: "What files changed?" → Blake3 diff
- **Level 1**: "What definitions are in this file?" → Definition index lookup
- **Level 2**: "What calls this function? What depends on this type?" → Graph traversal

**Projection (knowledge layer → human-readable artifacts)**:
- **Files**: Standard source files (primary projection, bidirectional)
- **API docs**: Generated from Level 1-2 (definitions + relationships)
- **Dependency diagrams**: Generated from Level 2-3 (graph + patterns)
- **AI Context Packs**: Minimal subgraph for a specific task, optimized for LLM consumption (Level 1-3)

**AI Agent Operations**:
- **Read**: Query L0-L2 directly via MCP tools (3-tier tool suite)
- **Navigate**: Follow graph edges to discover related code without file searching
- **Understand**: Two-phase context protocol — manifest first, then selective fetch
- **Edit**: Edit files normally (through existing tools), knowledge layer updates incrementally
- **Refactor** (future): Structural operations (rename, move, extract) through Level 2 graph operations, projected back to file edits

**Advantages**:
- **Right level of abstraction for each task**: Bug fix? Level 1-2 (definitions + callers). Impact analysis? Level 2 (dependents traversal). The agent doesn't over-fetch or under-fetch.
- **Incremental sophistication**: Level 0 exists today. Level 1-2 are straightforward extensions of 001. Higher levels (L3-L4) can be explored as research spikes without changing the core architecture (see [Future Work](#future-work--research)).
- **Git compatible**: Files remain the human interface and Git-tracked artifacts. Knowledge layer is a derived, persistent index — like a sophisticated `.git/index`.
- **Constitutional compliance**: Git is SoT for content. Knowledge layer is authoritative for *meaning*. This is an additive capability, not a replacement.
- **5-15x context compression**: AI Context Packs deliver the relevant subgraph for targeted queries in 5-15x fewer tokens than equivalent file reads, degrading to 2-5x for exploratory work. (CGM-style attention masking on Level 2-3 graph structure could achieve higher compression ratios as a separate research direction.)
- **Definition-level caching**: Level 1 gives per-definition content addressing. Changing one function doesn't invalidate the cache for sibling functions in the same file.
- **Cross-session persistence**: The knowledge layer persists codebase understanding across agent invocations, eliminating re-navigation overhead (note: persists codebase state, not agent task state — see Finding 8).
- **Dual deployment**: Each level works in both CLI (Postgres + petgraph) and Edge (D1 + streaming iterators) following Thread's existing patterns.
- **MCP-native**: The 3-tier tool suite maps cleanly to MCP tool definitions, enabling any MCP-compatible AI agent to query the knowledge layer.

**Disadvantages**:
- **Complexity budget**: Three core levels (L0-L2) are well-defined; higher levels (L3-L4) are research projects with uncertain outcomes (see [Future Work](#future-work--research)).
- **Definition extraction challenge**: Segmenting AST into "definitions" varies significantly across languages. Mitigated by tiered extraction using tree-sitter `tags.scm` (see Finding 6).
- **Update propagation latency**: Changes must flow L0 → L1 → L2. Each level adds latency to the "time to consistent knowledge" metric.
- **Storage overhead**: Revised to 3-5x raw code size for L0-L2 (see Finding 1). Metadata-only L1 is critical to keeping this manageable.

**Effort**: Medium. MVKL (L0-L2 + MCP tools) is achievable in 6-8 weeks parallel with 001 spec execution. Higher levels are deferred research spikes.

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

| Addition | 001 Spec | Knowledge Layer (MVKL) |
|----------|----------|----------------|
| Definition extraction (Level 1) | GraphNode is per-symbol but within file context | Definitions are independently content-addressed via tree-sitter `tags.scm` |
| AI query interface | RPC API for graph queries | 3-tier MCP tool suite (12 tools) |
| Context compression | Not addressed | Two-phase context packs (manifest → fetch), 5-15x compression |
| Cross-session persistence | Implied by overlay architecture | Persistent codebase understanding via `thread_definitions_changed` |
| Graceful degradation | Not addressed | Hybrid tools with transparent fallback (graph → parse → grep) |

### Phased Delivery

The MVKL (Minimum Viable Knowledge Layer) focuses on L0-L2 with MCP tools, validated by benchmarks before expansion. L3-L4 are deferred to research spikes (see [Future Work](#future-work--research)).

| Phase | Scope | Timeline | Deliverable |
|-------|-------|----------|-------------|
| **Phase 1** | 001 execution (L0 + L2) | Per 001 plan | File index + semantic graph + overlay architecture |
| **Phase 2** (parallel with P1) | L1 + MVKL | 6-8 weeks | Definition extraction via `tags.scm` + 5 MCP tools + Postgres |
| **Phase 3** | Validation | 4 weeks | Benchmark suite: 20-30 coding tasks, 5+ repos, measure token reduction + correctness |
| **Phase 4** | Expansion (conditional on P3) | Based on P3 results | Additional languages, D1 backend, advanced context packs |

**MVKL success criteria** (Phase 3 validation):
- 50%+ token reduction for targeted queries vs. file-based approach
- Equal or better task correctness (agent produces correct edits)
- <500ms query latency for L2 graph traversals on 100k-file codebases

---

## Key Design Decisions

### 1. Atomic Unit: Definition, not File

**Decision**: The primary unit of content-addressing and caching is the **definition** (function, type, module, trait, impl block), not the file.

**Rationale**: A single file often contains 10-50 definitions. When one definition changes, file-level caching invalidates the cache for all 50. Definition-level caching invalidates only the changed definition. For AI agents, this means:
- Requesting "the callers of function X" returns the definition of X and its callers — not the entire files containing them
- Context packs are 5-15x smaller than equivalent file reads for targeted queries
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
- The knowledge layer may eventually contain information not derivable from files alone (e.g., intent annotations — see [Future Work](#future-work--research)) — these would be treated as supplementary metadata, not authoritative content

### 3. AI Interface: MCP Tools with Level Selection

**Decision**: AI agents interact with the knowledge layer via MCP (Model Context Protocol) tools, with explicit level selection.

**Rationale**: MCP is the emerging standard for AI tool integration. It's supported by Claude, and increasingly by other agents. By exposing the knowledge layer as MCP tools, Thread becomes a universal intelligence backend for any MCP-compatible agent.

**MCP Tool Suite (3-Tier Design)**:

Specialist review (Finding 3) identified that 20 tools across 5 levels creates prohibitive routing overhead for LLM agents. The tool suite is organized into 3 tiers based on usage frequency:

```
# Tier 1: Primary (90% of agent interactions)
thread_context_plan(focal_point?: string, task?: string,
                    depth?: int, budget?: int) -> ContextManifest
thread_context_fetch(selections: [hash]) -> [Definition]
thread_search(query: string, language?: string) -> [SearchResult]
thread_locate(hash: string) -> FileLocation

# Tier 2: Navigation (targeted graph traversal)
thread_callers(symbol: string) -> [CallerInfo]
thread_callees(symbol: string) -> [CalleeInfo]
thread_dependencies(symbol: string, depth?: int) -> SubGraph
thread_dependents(symbol: string, depth?: int) -> SubGraph
thread_type_hierarchy(type: string) -> TypeTree

# Tier 3: Introspection (health checks, session management)
thread_status() -> SystemStatus
thread_definitions_changed(since: hash) -> [DefinitionChange]
thread_affected_by_change(symbol: string) -> ImpactAnalysis
```

**Design principles**:
- All tools returning lists support `cursor` parameter for pagination
- All results include `source: "graph" | "parse" | "text_search"` to signal confidence
- Tools fall back transparently when the knowledge layer is partially available (see Finding 5)
- Context plan uses two-phase protocol: manifest first (200-400 tokens), then selective fetch (see Finding 2)

### 4. Context Pack Format (Two-Phase Protocol)

**Decision**: Context delivery uses a two-phase protocol — **manifest first, then selective fetch** — rather than a single pre-assembled pack.

**Rationale**: Specialist review (Finding 2) identified that one-shot context assembly cannot reliably determine relevance outside the agent's reasoning loop. A two-phase approach gives the agent control over what enters its context window while still leveraging the knowledge layer's graph traversal.

**Phase 1 — Manifest** (`thread_context_plan`):
```
ContextManifest {
  // Graph version for optimistic concurrency (see Finding 9)
  graph_version: u64,

  // Available definitions, ordered by relevance
  entries: [{
    hash: string,
    name: string,
    kind: "function" | "type" | "trait" | "impl" | "module",
    file: string,
    token_estimate: u32,
    relevance_score: f32,
  }],

  // Relationships between entries
  edges: [{source: hash, target: hash, kind: string}],

  // What is not yet indexed (for agent fallback decisions)
  coverage: { indexed_files: u32, total_files: u32 },
}
```
Approximately 200-400 tokens. The agent reviews the manifest and selects which definitions to fetch.

**Phase 2 — Fetch** (`thread_context_fetch`):
```
[Definition {
  hash: string,
  content: string,         // Source code of the definition
  file: string,            // File path for editing
  line_range: (u32, u32),  // Line range for editing
  edges: [Edge],           // Relationships to other definitions
}]
```

**Two modes**:
- **Structural mode** (reliable, default): `thread_context_plan(focal_point: "process_payment", depth: 2)` — returns the 2-hop graph neighborhood. Deterministic, no relevance ranking needed.
- **Task mode** (best-effort): `thread_context_plan(task: "add rate limiting", scope: "src/payments/")` — attempts relevance ranking within an agent-specified scope. Explicitly marked as best-effort.

**Example**: Agent investigates `process_payment`:
1. `thread_context_plan(focal_point: "process_payment", depth: 1)` → manifest listing 12 definitions (~300 tokens)
2. Agent selects 6 relevant definitions from the manifest
3. `thread_context_fetch([hash1, hash2, ..., hash6])` → ~2,000-3,000 tokens of definition content
4. Total: ~2,500-3,500 tokens instead of ~30,000-50,000 tokens from reading all relevant files (5-15x compression)

### 5. Storage Architecture

**Decision**: Each level uses the appropriate storage backend, following Thread's existing multi-backend pattern.

| Level | CLI (Postgres) | Edge (D1) | Notes |
|-------|---------------|-----------|-------|
| L0: File Index | `files` table | `files` table | Exists in `thread-flow` |
| L1: Definitions | `definitions` table (metadata only: hash, byte_range, file_id, name, kind) | `definitions` table | ASTs reconstructed on demand, LRU cached |
| L2: Graph | `nodes` + `edges` tables (petgraph in-memory) | `nodes` + `edges` + `reachability` tables (streaming) | ~10MB LRU cache sufficient for D1 |

**Revised storage estimate**: 3-5x raw code size for L0-L2 combined (see Finding 1). Metadata-only L1 is critical — storing ASTs per definition would inflate to 5-15x.

### 6. Update Propagation Model

**Decision**: Updates flow bottom-up (L0 → L1 → L2) with level-specific latency targets.

| Transition | Trigger | Target Latency | Mechanism |
|-----------|---------|---------------|-----------|
| File → L0 | File system event | <10ms | Blake3 hash comparison |
| L0 → L1 | File hash changed | <100ms | Tree-sitter parse + `tags.scm` definition extraction |
| L1 → L2 | Definition hash changed | <500ms | Graph edge update (affected edges only) |

L0-L2 updates are synchronous. AI agents see changes in near-real-time (<500ms end-to-end for the critical path).

---

## Implementation Strategy

### Alignment with 001 Spec

The knowledge layer is designed as an **extension** of the 001-realtime-code-graph spec, not a replacement:

```
001 Spec Scope          Knowledge Layer Extension (MVKL)
─────────────────       ────────────────────────────────
Level 0: File Index     ← Exists in thread-flow
Level 1: Definitions    ← NEW (definition extraction via tags.scm)
Level 2: Semantic Graph ← Exists in 001 (GraphNode, GraphEdge)
MCP Tool Suite          ← NEW (3-tier, 12 tools)
Context Packs           ← NEW (two-phase protocol)
Level 3-4               ← Future research (see Future Work section)
```

### Proposed Crate Organization

Building on the 001 spec's proposed crate structure. Specialist review (Finding 7) reduced this from 4 new crates to 1. MCP server, context pack generation, and projection logic live as modules within `thread-api`.

```
crates/
├── thread-ast-engine/     # EXISTING: AST parsing (tree-sitter)
├── thread-language/       # EXISTING: Language definitions
├── thread-rule-engine/    # EXISTING: Rule-based scanning
├── thread-flow/           # EXISTING: Dataflow orchestration (ReCoco)
├── thread-utilities/          # EXISTING: Shared utilities
├── thread-wasm/           # EXISTING: WASM bindings
│
├── thread-graph/          # FROM 001: Core graph structures + algorithms
├── thread-indexer/        # FROM 001: Multi-source code indexing
├── thread-conflict/       # FROM 001: Conflict detection engine
├── thread-storage/        # FROM 001: Multi-backend persistence
├── thread-api/            # FROM 001: RPC/API layer (+ MCP server, context packs)
│   ├── src/
│   │   ├── ...            # Existing RPC/API code
│   │   ├── mcp/           # MCP tool server (3-tier tool suite)
│   │   └── context/       # Context pack generation (two-phase protocol)
│   └── tests/
│
└── thread-definitions/    # NEW (L1): Definition extraction + content-addressing
    ├── src/
    │   ├── extract.rs     # tree-sitter tags.scm-based definition boundary detection
    │   ├── hash.rs        # Content-addressing (Blake3 of definition metadata)
    │   ├── store.rs       # Definition CRUD with storage backends
    │   └── languages/     # Per-language extraction tiers (A/B/C)
    └── tests/
```

### Dependency Graph

```
thread-api (RPC + MCP + context packs)
  ├── thread-graph (L2)
  │     ├── thread-definitions (L1)
  │     │     ├── thread-ast-engine
  │     │     └── thread-language
  │     └── thread-storage
  └── thread-definitions
```

All arrows flow downward — no circular dependencies (Constitution Principle IV). **Net new crates: 1** (`thread-definitions`).

---

## Risk Analysis

Risk assessments reflect post-review findings. See Appendix C for the full revised risk matrix.

### Technical Risks (MVKL Scope)

| Risk | Severity | Likelihood | Mitigation |
|------|----------|-----------|-----------|
| **Definition extraction accuracy varies by language** | Medium | Medium | tree-sitter `tags.scm` for broad coverage; tiered quality (A/B/C); custom extractors only for Tier A languages |
| **Storage overhead** | High | High | Metadata-only L1 (no stored ASTs). Target: 3-5x raw code size for L0-L2. See Finding 1. |
| **Context pack quality** | High | High | Two-phase protocol (manifest → fetch) gives agent control. Structural mode is deterministic. See Finding 2. |
| **Knowledge layer unavailability** | High | Medium | Hybrid tools with transparent fallback (graph → parse → grep). `thread_status()` for health checks. See Finding 5. |
| **MCP protocol limitations** | Low | Medium | Tool interfaces are protocol-agnostic internally; MCP is one transport. |

### Adoption Risks

| Risk | Severity | Likelihood | Mitigation |
|------|----------|-----------|-----------|
| **Adoption without validation** | Critical | Medium | MVKL benchmark suite in Phase 3: 20-30 tasks, 5+ repos, measure token reduction + correctness. See Finding 10. |
| **AI agents don't use the knowledge layer** | Critical | Low | First consumer is Thread's own CLI. MCP server follows. See Finding 10. |
| **Complexity exceeds team capacity** | High | Medium | 1 new crate, not 4. L3-L4 deferred to research spikes. See Finding 7. |

### Constitutional Risks

| Risk | Mitigation |
|------|-----------|
| Principle I (Service-Library): Knowledge layer must serve both library API and service deployment | `thread-definitions` is a library crate; MCP/context packs are modules in `thread-api` |
| Principle III (TDD): Definition extraction logic needs comprehensive tests | Property-based testing for extraction; golden-file tests per language; round-trip tests |
| Principle VI (Service): Must meet storage and cache performance targets | Definition-level caching improves cache hit rates. Metadata-only storage. |

---

## Open Questions (MVKL Scope)

1. **Definition boundary edge cases**: How do we handle language constructs that don't fit cleanly into "definition" boundaries?
   - Rust macros that generate definitions
   - Python's module-level imperative code
   - JavaScript's IIFE patterns and dynamic exports
   - Mitigation: Tier C languages (C/C++, Bash, etc.) use file-level granularity

2. **Context manifest ranking**: How does `thread_context_plan` rank definitions in structural mode?
   - Fixed-depth graph traversal from focal point (baseline)
   - Fan-in/fan-out weighting (higher connectivity = higher relevance)
   - Agent-specified budget vs. system-optimized budget
   - Task mode ranking algorithm (best-effort, if implemented)

3. **Edge deployment scope**: Which L0-L2 operations are available on edge (WASM)?
   - L0-L1 metadata fits in D1 with ~10MB LRU cache
   - L2 graph: full graph exceeds 128MB for large codebases — streaming iterators or subgraph-only queries?
   - All MCP tools must work with partial data (graceful degradation)

4. **Branch handling**: How does the knowledge layer handle branches?
   - One knowledge layer instance per branch?
   - Shared base with branch-specific overlays (extending 001's overlay architecture)?
   - How does branch switching work — full rebuild or incremental diff?

5. **Optimistic concurrency semantics**: What exactly happens when `graph_version` conflict is detected?
   - Reject agent's follow-up queries until it re-fetches the manifest?
   - Return a diff of what changed, letting the agent decide?
   - Silent re-base (risky — may mask real conflicts)?

---

## Future Work / Research

The following capabilities are **not part of the MVKL scope**. They are documented here for completeness but require separate validation before investment. Each is framed as a time-boxed research spike with explicit go/no-go criteria.

### Research Spike R1: Architectural Pattern Detection (L3)

**Hypothesis**: Detecting module clusters, API surfaces, and ownership boundaries from the L2 graph would provide valuable architectural context for AI agents performing large-scale refactoring or onboarding tasks.

**Approach**: Use graph algorithms (community detection, fan-in/fan-out analysis, betweenness centrality) composed on the L2 petgraph, not a separate inference engine. Compute lazily on query, not eagerly on every graph update.

**Open questions**:
- What patterns are most valuable? Module boundaries, error handling conventions, state management patterns, API versioning?
- Should patterns be configurable per-project?
- How to handle the TDD tension — pattern detection has no objectively "correct" answer (Koschke 2009, Garcia et al. 2013)?

**Go/no-go criteria** (2-week spike):
- Prototype detects module boundaries on 3+ real-world Rust codebases
- AI agent with L3 context resolves 20%+ more architectural questions correctly vs. L2-only
- If no measurable improvement: cut L3 entirely

**If approved**: L3 pattern data lives in `thread-graph` as optional graph annotations, not in a separate crate. Exposed via the existing `thread_affected_by_change` introspection tool, not as separate L3-specific tools.

### Research Spike R2: Intent Inference (L4)

**Hypothesis**: Extracting behavioral contracts and natural language intent descriptions from code, docstrings, and tests would help AI agents write conformant new code and avoid violating implicit invariants.

**Approach**: Start with deterministic extraction (docstring parsing, test assertion extraction). LLM-assisted inference is a later option if deterministic extraction proves insufficient.

**Open questions**:
- What is the source of intent? Docstrings, test names, commit messages, LLM inference, manual annotation?
- How to validate correctness of inferred intent?
- Can intent be expressed as machine-checkable contracts, or is it inherently natural language?

**Go/no-go criteria** (2-week spike):
- Prototype extracts meaningful intent for 50%+ of public functions in a well-documented Rust codebase
- AI agent with L4 context produces code that violates fewer implicit conventions vs. L2-only
- If no measurable improvement: cut L4 entirely

**If approved**: Intent data lives as metadata on L1 definitions. Exposed via context pack metadata, not as separate tools.

### Future Capability: Bidirectional Graph Edits

Structural refactoring operations (rename, move, extract function, change signature) through the graph API, projected back to file edits. This is the path toward Option B (Unison model) and would require:
- A projection engine for graph → file sync
- Formatting-preserving code generation
- Multi-file atomic edit transactions

**Prerequisite**: MVKL must be validated and stable before investing in bidirectional sync.

### Future Capability: Agent Session Memory (L5)

True cross-session persistence would require storing agent-specific state: current task, working hypotheses, explored definitions, decisions made. This is a potential "L5" that lives above the codebase-level knowledge layer. See Finding 8 for analysis.

### Future Capability: Semantic Search

Embedding-based search via Qdrant for queries like "find code that handles retries" or "where is rate limiting implemented." Requires high-quality embeddings of every definition. Complements but does not replace the structural graph queries.

---

## Appendix A: Comparison Matrix

| Dimension | Option A (Evolutionary) | Option B (Unison) | Option C (Multi-Resolution) |
|-----------|------------------------|-------------------|---------------------------|
| **Atomic unit** | File | Definition | Definition (L1) + File (L0) |
| **Source of truth** | Git/files | Database | Git for content, KL for meaning |
| **AI interface** | Graph query → file read | Direct graph manipulation | Multi-level MCP tools |
| **Human interface** | Unchanged | Scratch files | Unchanged |
| **Context compression** | 2-5x (targeted file reads) | 50-500x (definition-level) | 5-15x targeted, 2-5x exploratory (context packs) |
| **Git compatibility** | Full | Requires adaptation | Full |
| **Implementation effort** | Low-Medium | Very High | Medium-High |
| **Definition-level caching** | No | Yes | Yes (Level 1) |
| **Architectural awareness** | No | No | Future research (Level 3) |
| **Intent/contracts** | No | No | Future research (Level 4) |
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

Three independent specialist reviews were conducted to identify planning gaps and architectural challenges. Their findings are synthesized below, organized by theme. **Key revisions have been incorporated into the main body** — this appendix preserves the detailed rationale and evidence behind each design change.

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

**Document Status**: Draft — revised per specialist review and Sourcery feedback. Main body reflects MVKL scope (L0-L2); L3-L4 detail consolidated into [Future Work / Research](#future-work--research) section.
**Next Steps**: Decision on proceeding to feature specification (speckit workflow) for the MVKL scope
