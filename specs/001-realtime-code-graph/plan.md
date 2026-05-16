<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: AGPL-3.0-or-later
-->

# Implementation Plan: Real-Time Code Graph Intelligence

**Branch**: `001-realtime-code-graph` | **Date**: 2026-01-11 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `specs/001-realtime-code-graph/spec.md`

**Phase Status**:
- ✅ Phase 0: Research complete (8 research tasks documented in research.md)
- ✅ Phase 1: Design artifacts complete (data-model.md, contracts/, quickstart.md)
- ✅ Phase 2: Task generation complete (tasks.md — Phase 0.5 through Phase 7, 55+ tasks)

## Summary

Real-Time Code Graph Intelligence transforms Thread from a code analysis library into a persistent intelligence platform. The system provides performant, codebase-wide graph analysis with semantic/AST awareness, enabling real-time dependency tracking, conflict prediction, and collaborative development support.

**Primary Requirements**:
- Build and maintain live code graph with <1s query response for 100k files
- Detect merge conflicts before commit with multi-tier progressive detection (100ms → 1s → 5s)
- Support dual deployment (CLI + Cloudflare Edge) from single codebase
- Achieve >90% cache hit rate via content-addressed storage
- Enable incremental updates affecting <10% of full analysis time

**Technical Approach**:

- Service-library dual architecture with Recoco dataflow orchestration
- Multi-backend storage (Postgres for CLI, D1 for edge, Vectorize for edge vector search)
- Trait-based abstraction for Recoco integration (prevent type leakage)
- **API Protocol**: prost + plain HTTP POST for external API (no Connect-RPC/gRPC framing); postcard for internal Rust-to-Rust (Worker→Container, CLI); JSON-RPC 2.0 for MCP server (future, separate adapter). All proto files use `package thread.v1;` namespace. Directory: `crates/thread-api/proto/v1/`. Add `buf.gen.yaml` for TypeScript codegen targeting `@bufbuild/protobuf` (protobuf-es v2). Version bump policy: field additions are backward-compatible; breaking changes require `v2/` directory.
- Conflict detection deferred to commercial `thread-conflict` crate (Phase 4)
- Rayon parallelism (CLI) + tokio async (edge) concurrency models

**Technical Context**

**Language/Version**: Rust (edition 2024, aligning with Thread's existing codebase)
**Primary Dependencies**:

- Recoco framework v0.2.1 (content-addressed caching, dataflow orchestration) - **INTEGRATED** in thread-flow crate via bridge pattern + ThreadFlowBuilder DSL
- tree-sitter (AST parsing foundation, existing Thread dependency)
- workers-rs (Cloudflare Workers runtime for edge deployment)
- prost (Protobuf encoding for external API, no_std/WASM-compatible); prost-build (host-only code gen, never in WASM binary)
- serde + postcard (internal Rust-to-Rust binary serialization: Worker→Container service bindings, CLI internal calls)
- rayon (CPU-bound parallelism for CLI, existing)
- tokio (async I/O for edge deployment, existing)
- tokio-postgres + deadpool-postgres (Postgres client for CLI storage, used in thread-flow)
- cloudflare-workers-rs SDK (D1 client for edge storage, WebSocket support)
- cloudflare-vectorize (edge vector search - replaces Qdrant for edge deployment)
- ~~petgraph~~ - NOT USED: thread-flow implements custom BFS/topological sort in incremental/graph.rs (1,099 lines)

**Edge Deployment Architecture**:

- **Cloudflare Containers (Recoco/thread-flow)**: Heavy computation — indexing, graph construction, incremental analysis — runs in Cloudflare Containers (beta). Full tokio/async support; no WASM constraints. Resolves WASM incompatibility with Recoco (D2).
- **Workers (thin WASM layer)**: Handles request routing, D1 native queries, Vectorize semantic search, and result serialization. OSS: single Worker (Rust/Python/TypeScript). Commercial: Router Worker + per-language Language Workers via service bindings.
- **Memory Wall (Workers only)**: Strict 128MB limit. **NO** loading full graph into Worker memory. Use streaming/iterator patterns (`D1GraphIterator`).
- **Database-First**: Primary graph state lives in D1. In-memory structs are ephemeral (batch processing only).
- **Reachability Index**: k-hop bounded (k=3 default) — NOT a full transitive closure (full closure for 10M nodes ≈ 800GB, exceeds D1 10GB limit). Tracks live session state (Container/DO memory) + committed baseline (D1). On-demand BFS beyond k hops.
- **Throughput Governance**: Use Recoco adaptive controls (max_inflight_bytes) (<80MB) and `Adaptive Batching` to manage resource pressure.

**Storage**: Multi-backend architecture with deployment-specific primaries:
- Postgres (CLI deployment primary - full graph with ACID guarantees)
- D1 (edge deployment primary - distributed graph storage + **Reachability Index**). Two implementations: `D1IncrementalBackend` (existing, REST API — for external tooling/CI); `D1NativeBackend` (planned, `worker::D1Database` native binding for in-Worker use — zero extra HTTP hop, enables SC-STORE-001 <50ms p95 target). Both implement `StorageBackend` trait.
- Vectorize (edge vector search), Qdrant (CLI-only, optional — currently blocked by Recoco dependency conflict)

**Testing**: cargo nextest (constitutional requirement, all tests executed via nextest)

**Target Platform**: Dual deployment targets:
- Native binary (Linux, macOS, Windows) for CLI
- WASM (Cloudflare Workers) for edge deployment

**Project Type**: Service-library dual architecture (both library crates AND persistent service components)

**Performance Goals**:
- Query response <1s for codebases up to 100k files (FR-005, SC-001)
- Conflict detection latency: <100ms (initial AST diff), <1s (semantic analysis), <5s (comprehensive graph analysis) (FR-006)
- Real-time update propagation: <100ms from code change detection to client notification (FR-013)
- Cache hit rate: >90% for repeated analysis of unchanged code (SC-CACHE-001)
- Incremental update: <10% of full analysis time for changes affecting <5% of files (SC-INCR-002)

**Constraints**:
- WASM bundle size: <10MB compressed for fast cold-start (SC-EDGE-003)
- Storage latency targets (p95): Postgres <10ms, D1 <50ms, Vectorize (edge vectors) <100ms (SC-STORE-001)
- Edge deployment global latency: <50ms p95 from any major city (commercial) (SC-EDGE-004)
- Memory: Sublinear storage growth through deduplication, max 1.5x raw code size (SC-STORE-004)

**Scale/Scope**:
- Initial target: 500k files, 10M graph nodes (expandable with infrastructure)
- Concurrent users: 1000 simultaneous queries with <2s p95 response (SC-004)
- Edge throughput: 10k requests/sec per geographic region (commercial) (SC-EDGE-005)
- Graph capacity: 10M nodes, 50M edges per deployment instance (SC-STORE-002)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Service-Library Architecture ✅

- [x] **Library Core**: Feature includes reusable library crates for graph analysis, indexing, conflict detection
- [x] **Service Layer**: Feature includes persistent service with Recoco orchestration, caching, and real-time updates
- [x] **Dual Consideration**: Design explicitly addresses both library API (for embedding) and service deployment (CLI + edge)

**Justification if violated**: N/A - Feature is fundamentally a service-library dual architecture system. Graph analysis logic is library-reusable, persistence/caching/real-time are service-specific.

### II. Performance & Safety ✅

- [x] **Unsafe Code**: No unsafe blocks planned initially. If needed for SIMD optimizations, will be explicitly justified with safety invariants
- [x] **Benchmarks**: Performance-critical paths (graph traversal, conflict detection, caching) include benchmark suite (SC-001 through SC-010 define targets)
- [x] **Memory Efficiency**: Sublinear storage growth enforced (max 1.5x raw code size). Content-addressed caching minimizes redundant allocations

**Justification if violated**: N/A - Performance is constitutional requirement. All critical paths benchmarked against success criteria.

### III. Test-First Development (NON-NEGOTIABLE) ✅

- [x] **TDD Workflow**: Tests written → Approved → Fail → Implement (mandatory red-green-refactor cycle)
- [x] **Integration Tests**: Crate boundaries covered (graph ↔ storage, indexer ↔ parser, API ↔ service)
- [x] **Contract Tests**: Public API behavior guaranteed (RPC contracts, library API stability)

**This gate CANNOT be violated. No justification accepted.** All development follows strict TDD discipline per Constitution Principle III.

### IV. Modular Design ✅

- [x] **Single Responsibility**: Each crate has singular purpose:
  - Library crates: thread-graph (core algorithms), thread-indexer (multi-source), thread-conflict (detection)
  - Service crates: thread-storage (persistence), thread-api (RPC), thread-realtime (WebSocket)
- [x] **No Circular Dependencies**: Acyclic dependency graph (see Project Structure for flow diagram)
- [x] **Recoco Integration**: Follows declarative dataflow patterns with trait-based abstraction in thread-services (research complete)

**Justification if violated**: N/A - Fully compliant. Research Task 6 defined clear crate organization with library-service split and acyclic dependencies

### V. Open Source Compliance ✅

- [x] **AGPL-3.0**: All new code properly licensed under AGPL-3.0-or-later (Thread standard)
- [x] **REUSE Spec**: License headers or .license files present (enforced via `mise run lint`)
- [x] **Attribution**: Recoco integration properly attributed, any vendored code documented

**Justification if violated**: N/A - Standard Thread licensing applies. Commercial features use feature flags, not separate licensing.

**Dependency direction**: Private commercial crates depend on Thread public crates. Thread public crates NEVER depend on commercial crates. `crates/cloudflare/` is a local development convenience (gitignored); it is a genuinely separate project in a private repo. No path dependencies are part of the design — they are local dev shortcuts only.

### VI. Service Architecture & Persistence ✅

- [x] **Deployment Target**: Both CLI and Edge (dual deployment architecture)
- [x] **Storage Backend**: Postgres (CLI primary), D1 (Edge primary), Vectorize (edge vectors), Qdrant (CLI optional)
- [x] **Caching Strategy**: Content-addressed caching via Recoco framework (IMPLEMENTED in thread-flow) (>90% hit rate target)
- [x] **Concurrency Model**: Rayon (CLI parallel processing), tokio (Edge async I/O)

**Deployment Target**: Both (CLI + Edge with single codebase, conditional compilation)
**Storage Backend**: Multi-backend (Postgres for CLI, D1 for edge, Vectorize for edge vector search)
**Justification if N/A**: N/A - Feature is fundamentally service-oriented with persistent intelligence layer

### Quality Standards (Service-Specific) ✅

- [x] **Storage Benchmarks**: Performance targets defined in SC-STORE-001
  - Postgres: <10ms p95 latency for graph traversal queries
  - D1: <50ms p95 latency for distributed edge queries
  - Vectorize (edge vectors): <100ms p95 latency for semantic similarity search
- [x] **Cache Performance**: >90% hit rate targeted (SC-CACHE-001) via content-addressed storage
- [x] **Incremental Updates**: Incremental re-analysis implemented (SC-INCR-001 through SC-INCR-004)
  - Only affected components re-analyzed, not full codebase
  - <10% of full analysis time for changes affecting <5% of files
- [x] **Edge Deployment**: WASM target required, `mise run build-wasm-release` must pass
  - OSS: Basic/limited WASM worker with core query capabilities
  - Commercial: Full edge deployment with advanced features

**Justification if N/A**: N/A - All service quality gates apply. Feature is service-first architecture.

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

#### Existing Foundation: thread-flow

The `thread-flow` crate (already in workspace) provides foundational infrastructure for
several planned crates. New crates should build on — not duplicate — this foundation:

| thread-flow component | Provides for planned crate |
|---|---|
| `incremental/graph.rs` (1,099 lines) | Core of `thread-graph` — BFS, topological sort, cycle detection |
| `incremental/analyzer.rs` (636 lines) | Core of `thread-indexer` — incremental analysis coordinator |
| `incremental/storage.rs` + backends | Core of `thread-storage` — StorageBackend trait, Postgres, D1 |
| `bridge.rs` + `flows/builder.rs` | Recoco integration (thread-services extension) |
| Language extractors (Rust/TS/Python/Go) | Dependency extraction for `thread-indexer` |

**Implication**: Phase 1 crate creation should wrap/extend thread-flow rather than reimplementing this infrastructure.

#### Language Coverage Strategy

The `thread-definitions` classification engine enables broad language coverage without per-language engineering work:

| Coverage | Mechanism | Languages |
|----------|-----------|----------|
| 80%+ baseline | token_purpose + universal_exact rules (2,444 cross-language patterns) | Any tree-sitter grammar |
| ~100% full | + TOML overrides (~10-50 lines/language) | All 27 currently validated |
| Potential | tree-sitter-language-pack grammars + TOML | ~166 languages |

This reframes FR-010 (multi-language support on Cloudflare): broad coverage is automatic via the classifier; per-language refinement is a community-contribution concern, not an engineering bottleneck.

**File-extension language identification**: CodeWeaver has ~200 language extension mappings. Porting these to `data/file_extensions.json` in thread-definitions would provide language detection for the full tree-sitter-language-pack without expanding the SupportLang enum.

> **Decision D-API-GRAPH**: The `thread-graph` crate's public API surface (all `pub use` exports, public trait signatures, and stable function signatures) MUST be documented in `specs/001-realtime-code-graph/contracts/` as `thread-graph-api.md` BEFORE contract tests (`T009`, `T027`) are written. This is an explicit gate: contract tests have nothing to verify against until the API surface is declared. The API surface document is the ground truth for contract tests. Emerges from TDD — the API is not pre-designed top-down but MUST be formally recorded once it stabilizes from test-driven discovery.

```text
crates/
├── thread-graph/          # PARTIAL: Extend thread-flow/src/incremental/graph.rs — do NOT reimplement
│   ├── src/
│   │   ├── lib.rs
│   │   ├── node.rs        # GraphNode, NodeId (semantic_class: SemanticClass, node_kind: Option<Box<str>>)
│   │   ├── edge.rs        # GraphEdge, EdgeType, relationship types
│   │   ├── graph.rs       # Graph container, adjacency lists
│   │   └── algorithms.rs  # Traversal, pathfinding (custom BFS/topo-sort from thread-flow)
│   └── tests/
├── thread-indexer/        # PARTIAL: Extend thread-flow/src/incremental/analyzer.rs and extractors/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── sources/       # Git, local file, S3 sources
│   │   ├── watcher.rs     # File change detection
│   │   └── indexer.rs     # Code → AST → graph nodes
│   └── tests/
├── thread-conflict/       # NEW: Multi-tier conflict detection engine (not started)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── tier1_ast.rs   # AST diff algorithm (<100ms)
│   │   ├── tier2_semantic.rs # Semantic analysis (<1s)
│   │   ├── tier3_graph.rs # Graph impact analysis (<5s)
│   │   └── progressive.rs # Progressive result streaming
│   └── tests/
├── thread-storage/        # PARTIAL: Extend thread-flow/src/incremental/storage.rs and backends/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── traits.rs      # GraphStorage, VectorStorage, StorageMigration
│   │   ├── postgres.rs    # PostgresStorage implementation
│   │   ├── d1.rs          # D1Storage implementation (Cloudflare)
│   │   └── vectorize.rs   # VectorizeStorage implementation (edge vector search)
│   └── tests/
├── thread-api/            # NEW: RPC protocol (HTTP+WebSocket) (not started)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── rpc.rs         # Custom RPC over HTTP (workers-rs + postcard)
│   │   ├── types.rs       # Request/response types, shared across CLI/edge
│   │   └── errors.rs      # Error types, status codes
│   └── tests/
├── thread-realtime/       # NEW: Real-time update propagation (not started)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── websocket.rs   # WebSocket handling
│   │   ├── sse.rs         # Server-Sent Events fallback
│   │   ├── polling.rs     # Long-polling last resort
│   │   └── traits.rs      # RealtimeBackend trait (OSS — DO implementation is NOT here)
│   │   # [CF: private] — Durable Objects backend implements RealtimeBackend trait;
│   │   # lives in private commercial crate, not in thread-realtime OSS crate
│   └── tests/
├── thread-definitions/   # FROM SEMANTIC_CLASSIFICATION_SPEC: Language-agnostic
│   │                       semantic classification engine for AST node types.
│   │                       Provides SemanticClass (22 variants across 5 importance
│   │                       tiers), ImportanceScores, AgentTask scoring with
│   │                       ContextualAdjustments. Pure lookup table over pre-baked
│   │                       JSON/TOML data — no tree-sitter dependency. Enables L1
│   │                       definition extraction in thread-flow operators (replaces
│   │                       tags.scm approach). Pre-baked data in
│   │                       crates/definitions/data/ migrated from classifications/.
│   │                       **Key emergent capability**: language-agnostic semantic
│   │                       queries — callers search by `SemanticClass` (e.g.,
│   │                       `DefinitionCallable`) rather than language-specific node
│   │                       type strings, enabling uniform AST traversal across all
│   │                       166+ supported languages. The query adapter transform
│   │                       lives in `thread-flow`, keeping both `thread-definitions`
│   │                       and `thread-ast-engine` dep-free of each other.
│   │                       NEW crate.
│   ├── src/
│   │   └── lib.rs
│   └── tests/
├── thread-services/       # EXISTING → EXTENDED: Recoco integration
│   ├── src/
│   │   ├── lib.rs
│   │   ├── dataflow/      # NEW: Recoco trait abstractions
│   │   │   ├── traits.rs  # DataSource, DataFunction, DataTarget
│   │   │   ├── registry.rs # Factory registry pattern
│   │   │   └── spec.rs    # YAML dataflow specification parser
│   │   └── existing...    # Previous service interfaces
│   └── tests/
├── thread-flow/           # EXISTING: Recoco integration layer (FOUNDATIONAL - already implemented)
│   │                       Also provides the semantic query transform — the adapter
│   │                       that bridges thread-ast-engine AST traversal with
│   │                       thread-definitions classification to enable
│   │                       language-agnostic queries via SemanticClass.
│   ├── src/
│   │   ├── bridge.rs      # CocoIndexAnalyzer adapter (CodeAnalyzer trait)
│   │   ├── flows/builder.rs # ThreadFlowBuilder DSL
│   │   ├── incremental/
│   │   │   ├── graph.rs   # Custom BFS/topological sort (1,099 lines)
│   │   │   ├── analyzer.rs # IncrementalAnalyzer (636 lines)
│   │   │   ├── storage.rs # StorageBackend trait
│   │   │   └── backends/
│   │   │       ├── postgres.rs # PostgresIncrementalBackend
│   │   │       └── d1.rs      # D1IncrementalBackend (800+ lines)
│   │   └── functions/     # parse, symbols, imports, calls operators
│   └── tests/
# Note: file tree above shows key files; actual codebase includes additional files (batch.rs, cache.rs, conversion.rs, monitoring/, registry.rs, runtime.rs, incremental/extractors/, etc.)
├── thread-ast-engine/     # EXISTING → REUSED: AST parsing foundation
├── thread-language/       # EXISTING → REUSED: Language support (Tier 1-3 languages)
├── thread-rule-engine/    # EXISTING → EXTENDED: Pattern-based conflict rules
│   └── src/
│       └── conflict_rules/ # NEW: Conflict detection rule definitions
├── thread-utils/          # EXISTING → REUSED: SIMD, hashing utilities
└── thread-wasm/           # EXISTING → EXTENDED: Edge deployment features
    ├── src/
    │   ├── api_bindings.rs # NEW: WASM bindings for thread-api
    │   └── realtime_bindings.rs # NEW: WebSocket for WASM
    └── tests/

specs/001-realtime-code-graph/
├── spec.md              # Feature specification (existing)
├── plan.md              # This file (implementation plan)
├── research.md          # Phase 0: Research findings and decisions (complete)
├── data-model.md        # Phase 1: Entity definitions and relationships
├── quickstart.md        # Phase 1: Getting started guide
└── contracts/           # Phase 1: API protocol definitions
    ├── rpc-types.rs     # Shared RPC types for CLI and edge
    └── websocket-protocol.md # WebSocket message format specification

tests/
├── contract/            # API contract tests (RPC behavior, WebSocket protocol)
├── integration/         # Cross-crate integration tests
│   ├── graph_storage.rs # thread-graph ↔ thread-storage
│   ├── indexer_api.rs   # thread-indexer ↔ thread-api
│   └── realtime_conflict.rs # thread-realtime ↔ thread-conflict
└── benchmarks/          # Performance regression tests
    ├── graph_queries.rs # <1s for 100k files (SC-001)
    ├── conflict_detection.rs # <100ms, <1s, <5s tiers (FR-006)
    ├── incremental_updates.rs # <10% of full analysis (SC-INCR-002)
    └── cache_hit_rate.rs # >90% (SC-CACHE-001)
```

**Dependency Graph** (acyclic, library-service separated):
```
Service Layer (orchestration, persistence):
    thread-services (Recoco traits)
       ├─> thread-storage (Postgres/D1/Vectorize)
       ├─> thread-realtime (WebSocket/SSE)
       └─> thread-api (Custom RPC over HTTP)
              # NOTE: thread-conflict (commercial) → thread-api (not the reverse)
              # Commercial crate imports conflict protocol types from thread-api; thread-api never depends on thread-conflict

    thread-flow (Recoco integration layer - FOUNDATIONAL)
       ├─> recoco v0.2.1 (public crate)
       ├─> thread-ast-engine
       ├─> thread-definitions (for classify_node_types operator)
       └─> [Postgres | D1 | Qdrant (CLI optional)]

Library Layer (reusable, embeddable):
    thread-conflict
       └─> thread-graph (core data structures)
       └─> thread-ast-engine (AST parsing)

    thread-indexer
       └─> thread-ast-engine
       └─> thread-language
       └─> thread-graph

    thread-graph
       └─> thread-definitions (semantic classification)
       └─> thread-utils (SIMD, hashing)

    thread-definitions
       └─> thread-language (SupportLang)

    thread-ast-engine, thread-language, thread-utils (existing, no changes)

Edge Deployment:
    thread-wasm (WASM bindings)
       └─> thread-api
       └─> thread-realtime
```

**Structure Decision**:
- **Single Workspace Extension**: New graph-focused crates added to existing Thread workspace
- **Library-Service Boundary**: Clear separation (graph/indexer are library-reusable; storage/api/realtime are service-specific; thread-conflict is commercial/deferred)
- **Recoco Integration**: SCAFFOLDED via bridge.rs + ThreadFlowBuilder DSL in thread-flow (bridge.rs = stubs only, must be implemented before T-C10)
- **Acyclic Dependencies**: Top-down flow from services → libraries, no circular references
- **Component Selection**: Existing ast-grep components (ast-engine, language) reused, CodeWeaver evaluation deferred to Phase 2 (Research Task 2)

**Crate Ownership Boundary (D3)**:

- `thread-services` = engine-agnostic orchestration traits ONLY (`DataSource`, `DataFunction`, `DataTarget`). Recoco types NEVER appear in `thread-services` public API.
- `thread-flow` = the Recoco implementation. Owns `bridge.rs`, `ThreadFlowBuilder`, storage backends, and the semantic query transform (bridge between `thread-ast-engine` and `thread-definitions`).
- All new crates (`thread-graph`, `thread-indexer`, etc.) depend on `thread-services` traits, NOT `thread-flow` directly. This prevents circular dependencies (thread-flow depends on these crates while also being their implementation) and preserves engine swappability.
- `thread-conflict` is Commercial/Deferred — Phase 4 tasks are out of OSS scope (see D4 decision).
- `thread-api/types.rs` owns shared conflict protocol types (`ConflictPrediction`, `ConflictType`, `Severity`, `DetectionTier`, `ConflictStatus`, `ResolutionStrategy`). `thread-conflict` (commercial) imports these from `thread-api` — it does not define them.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |

**Phase 0.5 — Semantic Classification** (parallel workstream, prerequisite for T011)
```
thread-definitions crate (parallel, prerequisite for T011):
  T-C01: Create thread-definitions crate skeleton
  T-C02: Implement types.rs (SemanticClass, ImportanceRank, TokenPurpose, etc.)
  T-C03: Implement error.rs
  T-C04: Implement rules.rs (data loading)
  T-C05: Implement scoring.rs (ImportanceScores + ContextualAdjustments)
  T-C06: Implement classifier.rs (8-stage lookup pipeline)
  T-C07: Migrate data files from classifications/ to crates/definitions/data/
  T-C08: Port holdout evaluation to Rust accuracy tests
  T-C09: Extend GraphNode to use semantic_class + node_kind (update T011)
  T-C10: Add classify_node_types operator to thread-flow pipeline
  T-C11: AST fingerprinting for language identification fallback (`Classifier::fingerprint_language`)
  T-C12: Semantic query transforms in `thread-flow` (`find_by_class`, `find_by_rank`, `classify_tree`)
```

**Phase 1: Core Integration** (3 weeks, conditional on Phase 0 pass)
```
Goal: Implement full Thread operator suite and storage backends

Tasks:
✓ Implement all Thread custom functions:
  - ThreadParseFunction → DONE (thread_parse operator in thread-flow/src/functions/parse.rs)
  - ThreadExtractSymbolsFunction → DONE (thread_symbols in thread-flow/src/functions/symbols.rs)
  - ⏳ ThreadRuleMatchFunction (planned, not built)
  - ⏳ ThreadExtractRelationshipsFunction (planned, not built)
  - ⏳ ThreadBuildGraphFunction (planned, not built)
✓ Implement storage targets:
  - PostgresTarget → DONE (PostgresIncrementalBackend in thread-flow/src/incremental/backends/postgres.rs)
  - D1Target → DONE (D1IncrementalBackend in thread-flow/src/incremental/backends/d1.rs) + **Reachability Index Logic**
  - ⏳ VectorizeTarget (planned, not built — edge vector search via Cloudflare Vectorize API — replaces QdrantTarget)
✓ Implement **Batching Strategy**:
  - D1 `BATCH INSERT` optimization
  - Streaming iterator for graph traversal
✓ Build service trait wrappers (external API) → PARTIAL (bridge.rs implements CodeAnalyzer trait)
✓ Comprehensive integration tests

Success Criteria:
⏳ All Thread capabilities functional through Recoco (blocked: ThreadRuleMatchFunction, ThreadExtractRelationshipsFunction, ThreadBuildGraphFunction not yet built)
⏳ Service trait API stable and tested (partial — bridge.rs complete, remaining operators pending)
⏳ Performance targets met (<1s query, <100ms Tier 1 conflict)
⏳ >90% cache hit rate on real-world codebases
✅ D1 writes handled via batches, avoiding lock contention
```
