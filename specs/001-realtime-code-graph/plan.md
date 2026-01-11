# Implementation Plan: Real-Time Code Graph Intelligence

**Branch**: `001-realtime-code-graph` | **Date**: 2026-01-11 | **Spec**: [spec.md](./spec.md)  
**Input**: Feature specification from `specs/001-realtime-code-graph/spec.md`

**Phase Status**:
- ✅ Phase 0: Research complete (8 research tasks documented in research.md)
- ✅ Phase 1: Design artifacts complete (data-model.md, contracts/, quickstart.md)
- ⏳ Phase 2: Task generation (run `/speckit.tasks` to generate tasks.md)

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Real-Time Code Graph Intelligence transforms Thread from a code analysis library into a persistent intelligence platform. The system provides performant, codebase-wide graph analysis with semantic/AST awareness, enabling real-time dependency tracking, conflict prediction, and collaborative development support.

**Primary Requirements**: 
- Build and maintain live code graph with <1s query response for 100k files
- Detect merge conflicts before commit with multi-tier progressive detection (100ms → 1s → 5s)
- Support dual deployment (CLI + Cloudflare Edge) from single codebase
- Achieve >90% cache hit rate via content-addressed storage
- Enable incremental updates affecting <10% of full analysis time

**Technical Approach** (pending Phase 0 research):
- Service-library dual architecture with CocoIndex dataflow orchestration
- Multi-backend storage (Postgres for CLI, D1 for edge, Qdrant for semantic search)
- Trait-based abstraction for CocoIndex integration (prevent type leakage)
- Custom RPC over HTTP unified API protocol (CLI + edge, pending WASM compatibility research)
- Progressive conflict detection (AST diff → semantic → graph impact)
- Rayon parallelism (CLI) + tokio async (edge) concurrency models

**Technical Context**

**Language/Version**: Rust (edition 2024, aligning with Thread's existing codebase)  
**Primary Dependencies**: 
- CocoIndex framework (content-addressed caching, dataflow orchestration) - trait-based integration in thread-services
- tree-sitter (AST parsing foundation, existing Thread dependency)
- workers-rs (Cloudflare Workers runtime for edge deployment)
- serde + postcard (binary serialization for RPC, ~40% size reduction vs JSON)
- rayon (CPU-bound parallelism for CLI, existing)
- tokio (async I/O for edge deployment, existing)
- sqlx (Postgres client for CLI storage)
- cloudflare-workers-rs SDK (D1 client for edge storage, WebSocket support)
- qdrant-client (vector database for semantic search)
- petgraph (in-memory graph algorithms for complex queries - **CLI ONLY**)

**Edge Constraint Strategy**:
- **Memory Wall**: Strict 128MB limit. **NO** loading full graph into memory. Use streaming/iterator patterns (`D1GraphIterator`).
- **Database-First**: Primary graph state lives in D1. In-memory structs are ephemeral (batch processing only).
- **Reachability Index**: Maintain a pre-computed transitive closure table in D1 to enable O(1) conflict detection without recursive queries.
- **Throughput Governance**: Use CocoIndex `max_inflight_bytes` (<80MB) and `Adaptive Batching` to manage resource pressure.

**Storage**: Multi-backend architecture with deployment-specific primaries:
- Postgres (CLI deployment primary - full graph with ACID guarantees)
- D1 (edge deployment primary - distributed graph storage + **Reachability Index**)
- Qdrant (semantic search backend for vector embeddings, both deployments)

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
- Storage latency targets (p95): Postgres <10ms, D1 <50ms, Qdrant <100ms (SC-STORE-001)
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
- [x] **Service Layer**: Feature includes persistent service with CocoIndex orchestration, caching, and real-time updates
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
- [x] **CocoIndex Integration**: Follows declarative YAML dataflow patterns with trait-based abstraction in thread-services (research complete)

**Justification if violated**: N/A - Fully compliant. Research Task 6 defined clear crate organization with library-service split and acyclic dependencies

### V. Open Source Compliance ✅

- [x] **AGPL-3.0**: All new code properly licensed under AGPL-3.0-or-later (Thread standard)
- [x] **REUSE Spec**: License headers or .license files present (enforced via `mise run lint`)
- [x] **Attribution**: CocoIndex integration properly attributed, any vendored code documented

**Justification if violated**: N/A - Standard Thread licensing applies. Commercial features use feature flags, not separate licensing.

### VI. Service Architecture & Persistence ✅

- [x] **Deployment Target**: Both CLI and Edge (dual deployment architecture)
- [x] **Storage Backend**: Postgres (CLI primary), D1 (Edge primary), Qdrant (vectors, both deployments)
- [x] **Caching Strategy**: Content-addressed caching via CocoIndex framework (>90% hit rate target)
- [x] **Concurrency Model**: Rayon (CLI parallel processing), tokio (Edge async I/O)

**Deployment Target**: Both (CLI + Edge with single codebase, conditional compilation)
**Storage Backend**: Multi-backend (Postgres for CLI, D1 for edge, Qdrant for semantic search)
**Justification if N/A**: N/A - Feature is fundamentally service-oriented with persistent intelligence layer

### Quality Standards (Service-Specific) ✅

- [x] **Storage Benchmarks**: Performance targets defined in SC-STORE-001
  - Postgres: <10ms p95 latency for graph traversal queries
  - D1: <50ms p95 latency for distributed edge queries
  - Qdrant: <100ms p95 latency for semantic similarity search
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

```text
crates/
├── thread-graph/          # NEW: Core graph data structures, traversal algorithms, pathfinding
│   ├── src/
│   │   ├── lib.rs
│   │   ├── node.rs        # GraphNode, NodeId, NodeType
│   │   ├── edge.rs        # GraphEdge, EdgeType, relationship types
│   │   ├── graph.rs       # Graph container, adjacency lists
│   │   └── algorithms.rs  # Traversal, pathfinding (uses petgraph)
│   └── tests/
├── thread-indexer/        # NEW: Multi-source code indexing (Git, local, cloud)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── sources/       # Git, local file, S3 sources
│   │   ├── watcher.rs     # File change detection
│   │   └── indexer.rs     # Code → AST → graph nodes
│   └── tests/
├── thread-conflict/       # NEW: Multi-tier conflict detection engine
│   ├── src/
│   │   ├── lib.rs
│   │   ├── tier1_ast.rs   # AST diff algorithm (<100ms)
│   │   ├── tier2_semantic.rs # Semantic analysis (<1s)
│   │   ├── tier3_graph.rs # Graph impact analysis (<5s)
│   │   └── progressive.rs # Progressive result streaming
│   └── tests/
├── thread-storage/        # NEW: Multi-backend storage abstraction
│   ├── src/
│   │   ├── lib.rs
│   │   ├── traits.rs      # GraphStorage, VectorStorage, StorageMigration
│   │   ├── postgres.rs    # PostgresStorage implementation
│   │   ├── d1.rs          # D1Storage implementation (Cloudflare)
│   │   └── qdrant.rs      # QdrantStorage implementation (vectors)
│   └── tests/
├── thread-api/            # NEW: RPC protocol (HTTP+WebSocket)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── rpc.rs         # Custom RPC over HTTP (workers-rs + postcard)
│   │   ├── types.rs       # Request/response types, shared across CLI/edge
│   │   └── errors.rs      # Error types, status codes
│   └── tests/
├── thread-realtime/       # NEW: Real-time update propagation
│   ├── src/
│   │   ├── lib.rs
│   │   ├── websocket.rs   # WebSocket handling
│   │   ├── sse.rs         # Server-Sent Events fallback
│   │   ├── polling.rs     # Long-polling last resort
│   │   └── durable_objects.rs # Cloudflare Durable Objects integration
│   └── tests/
├── thread-services/       # EXISTING → EXTENDED: CocoIndex integration
│   ├── src/
│   │   ├── lib.rs
│   │   ├── dataflow/      # NEW: CocoIndex trait abstractions
│   │   │   ├── traits.rs  # DataSource, DataFunction, DataTarget
│   │   │   ├── registry.rs # Factory registry pattern
│   │   │   └── spec.rs    # YAML dataflow specification parser
│   │   └── existing...    # Previous service interfaces
│   └── tests/
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
    thread-services (CocoIndex traits)
       ├─> thread-storage (Postgres/D1/Qdrant)
       ├─> thread-realtime (WebSocket/SSE)
       └─> thread-api (Custom RPC over HTTP)
              └─> thread-conflict (multi-tier detection)

Library Layer (reusable, embeddable):
    thread-conflict
       └─> thread-graph (core data structures)
       └─> thread-ast-engine (AST parsing)
       
    thread-indexer
       └─> thread-ast-engine
       └─> thread-language
       └─> thread-graph
    
    thread-graph
       └─> thread-utils (SIMD, hashing)
    
    thread-ast-engine, thread-language, thread-utils (existing, no changes)

Edge Deployment:
    thread-wasm (WASM bindings)
       └─> thread-api
       └─> thread-realtime
```

**Structure Decision**: 
- **Single Workspace Extension**: New graph-focused crates added to existing Thread workspace
- **Library-Service Boundary**: Clear separation (graph/indexer/conflict are library-reusable, storage/api/realtime are service-specific)
- **CocoIndex Integration**: Trait abstractions in thread-services prevent type leakage (Research Task 1)
- **Acyclic Dependencies**: Top-down flow from services → libraries, no circular references
- **Component Selection**: Existing ast-grep components (ast-engine, language) reused, CodeWeaver evaluation deferred to Phase 2 (Research Task 2)

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |

**Phase 1: Core Integration** (3 weeks, conditional on Phase 0 pass)
```
Goal: Implement full Thread operator suite and storage backends

Tasks:
✓ Implement all Thread custom functions:
  - ThreadParseFunction
  - ThreadExtractSymbolsFunction
  - ThreadRuleMatchFunction
  - ThreadExtractRelationshipsFunction
  - ThreadBuildGraphFunction
✓ Implement storage targets:
  - PostgresTarget (CLI)
  - D1Target (Edge) + **Reachability Index Logic**
  - QdrantTarget (vectors)
✓ Implement **Batching Strategy**:
  - D1 `BATCH INSERT` optimization
  - Streaming iterator for graph traversal
✓ Build service trait wrappers (external API)
✓ Comprehensive integration tests

Success Criteria:
✅ All Thread capabilities functional through CocoIndex
✅ Service trait API stable and tested
✅ Performance targets met (<1s query, <100ms Tier 1 conflict)
✅ >90% cache hit rate on real-world codebases
✅ D1 writes handled via batches, avoiding lock contention
```
