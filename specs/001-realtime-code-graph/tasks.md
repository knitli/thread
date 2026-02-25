<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: AGPL-3.0-or-later
-->

# Tasks: Real-Time Code Graph Intelligence

**Feature**: `001-realtime-code-graph`
**Status**: Planning
**Generated**: 2026-01-11

## Foundation Status (Updated 2026-02-24)

⚠️ **The `thread-flow` crate was built before these tasks were executed.** It provides the
foundational infrastructure for many Phase 1-2 tasks. Tasks below have been annotated to reflect
actual status. **Do not duplicate thread-flow's existing implementations.**

See `crates/flow/` for the existing foundation.

### Phase 0.5 — Semantic Classification Crate (`thread-definitions`)
*Prerequisite for T011. Can run in parallel with Phase 1 scaffolding.*

- [ ] **T-C01**: Create `thread-definitions` crate skeleton
  - `Cargo.toml` with dependencies: serde, serde_json, thiserror, toml, thread-language (SupportLang only)
  - Add to workspace Cargo.toml members
  - Add `workspace.dependencies` entry
  - **Constraint**: No tree-sitter dependency — classification works on node type name strings only

- [ ] **T-C02**: Implement `types.rs`
  - `SemanticClass` enum (22 variants across 5 importance tiers — replaces GraphNode.node_type)
  - `ImportanceRank` enum (5 variants, #[repr(u8)])
  - `TokenPurpose` enum (6 variants: Operator, Keyword, Literal, Punctuation, Comment, Identifier)
  - `NodeKind` enum (Token, Composite)
  - `Confidence` enum (High, Medium, Low)
  - `ClassificationMethod` enum (8 variants per pipeline)
  - `Classification` struct with class, rank, confidence, method fields
  - All serde derives with rename_all = "snake_case"

- [ ] **T-C08a**: Write failing classification accuracy test suite *(TDD: write tests before implementation)*
  - Load all 27 language JSON fixture files from `tests/fixtures/`
  - Assert ≥99% accuracy across all classified items — these tests MUST FAIL until T-C06 is complete
  - Assert ≥80% accuracy on simulated "new language" (universal rules only)
  - Add snapshot test stubs for representative node types (using insta) — stubs only, values TBD
  - Property test stubs: every SemanticClass has a rank, every class has scores — stubs only
  - **Gate**: Do NOT implement classifier (T-C06) until these tests are written and confirmed failing

- [ ] **T-C03**: Implement `error.rs`
  - `ClassifierError` enum with thiserror
  - Variants: DataLoadFailed, InvalidJson, InvalidToml, UnknownSemanticClass

- [ ] **T-C04**: Implement `rules.rs`
  - Deserialize `universal_rules.json` → `HashMap<Box<str>, SemanticClass>` (exact) + majority maps
  - Deserialize `categories.json` → category map
  - Deserialize TOML overrides per SupportLang language
  - String-keyed fallback map for non-SupportLang languages (best-effort coverage)
  - Use `include_str!` for embedded data in default build

- [ ] **T-C05**: Implement `scoring.rs`
  - `ImportanceScores` struct (5 f32 fields: discovery, comprehension, modification, debugging, documentation)
  - `AgentTask` enum (9 variants: Debug, Implement, Refactor, Review, Search, Test, Document, LocalEdit, Default)
  - `ContextualAdjustments` struct (depth_penalty_per_level: f32, size_bonus_threshold: u32, size_bonus_factor: f32 — all zero-default)
  - Load from `scoring.json` (includes per-task contextual_adjustments as optional fields)
  - `ImportanceScores::for_task()` with ContextualAdjustments parameter
  - **Note**: Current scoring.json values are starting points; design for easy empirical refinement via config

- [ ] **T-C06**: Implement `classifier.rs`
  - `Classifier` struct with all rule maps and scoring data
  - `Classifier::new()` (embedded data) and `from_directory()` (custom path, for testing)
  - 8-stage classification pipeline: Override → FileDetection → TokenPurpose → UniversalExact → UniversalMajority → Category → NameHeuristic → Unclassified
  - `classify()` with full parameter set
  - `classify_simple()` convenience method
  - `importance_scores()`, `task_score()` scoring methods
  - `ClassifierStats` introspection struct

- [ ] **T-C07**: Migrate data files from `classifications/` and `codeweaver_semantic_package/` to `crates/definitions/data/`
  - `_universal_rules.json` → `data/universal_rules.json` (strip description/count fields)
  - `_categories.json` → `data/categories.json` (extract mapping field only)
  - `_scoring.json` → `data/scoring.json` (copy as-is, add contextual_adjustments stubs)
  - `overrides/*.toml` → `data/overrides/*.toml` (copy as-is)
  - Language JSON files (e.g., rust.json) → `tests/fixtures/` (ground truth for accuracy tests only, not shipped)
  - `codeweaver_semantic_package/file_extensions.py` → split into:
    - `data/file_extensions.json` (source of truth, used by build.rs to generate hardcoded phf::Map):
      - `code_extensions`: { ".rs": "rust", ".py": "python", ... } — ~200 extension → language name (compiled to zero-cost lookup, hot path)
    - `data/file_categories.json` (embedded JSON, parsed once at startup):
      - `excluded_dirs`: [".git", "node_modules", "target", ...] — directories to skip
      - `excluded_extensions`: [".7z", ".exe", ".png", ...] — binary/media to exclude
      - `doc_extensions`: [...] — documentation file extensions
      - `data_extensions`: [...] — data file extensions
    - `data/special_files.toml` (new, from curation + CodeWeaver's lists):
      - `[llm_tooling]` — CLAUDE.md, .cursorrules, AGENTS.md, .claude/, GEMINI.md, etc. (high-priority AI context files)
      - `[dev_tools]` — Makefile, Dockerfile, .gitignore, .editorconfig, etc.
      - `[build_roots]` — Cargo.toml, package.json, pyproject.toml, go.mod, etc. (project root indicators)
    - `data/repo_heuristics.toml` (new — formalizes project-type detection):
      - Maps indicator filenames to project language/type
      - Replaces CodeWeaver's non-operational repo identification scheme
  - **Note**: All source data already in repo — this is a move/transform operation. file_extensions.py
    has ~200 code languages, ~50 excluded dirs, ~60 excluded extensions.

- [ ] **T-C08b**: Verify and complete classification accuracy tests *(TDD: make failing tests pass)*
  - Confirm all T-C08a tests now pass with classifier implementation from T-C06
  - Fill in snapshot test expected values (insta review)
  - Confirm property tests pass
  - Add any edge cases discovered during T-C06 implementation

- [ ] **T-C09**: Update `GraphNode` to use `semantic_class` + `node_kind`
  - Remove `node_type: NodeType` enum field
  - Add `semantic_class: SemanticClass` (from thread-definitions)
  - Add `node_kind: Option<Box<str>>` for raw tree-sitter node type name
  - Update T011 implementation to populate both fields
  - Update any existing code referencing node_type

- [ ] **T-C09b**: Harden bridge.rs stubs — replace all `// TODO` stub returns with explicit `Err(ThreadFlowError::NotImplemented { method: "method_name" })`. Add `NotImplemented` variant to `ThreadFlowError`. Commit independently so stub behavior is auditable before classify operator work begins.
  - **Constraint**: Zero silent empty returns permitted after this task. All stub calls must surface as errors in logs/tests.

- [ ] **T-C10**: Add `classify_node_types` operator to thread-flow pipeline
  - ⚠️ **PREREQUISITE**: Before modifying bridge.rs, all existing stub methods MUST be updated to return `Err(ThreadFlowError::NotImplemented { method: "<name>" })` instead of silent empty results. Silent empty stubs mask missing data as successful empty queries. This change MUST be committed and reviewed before any T-C10 work begins. (See T-C09b.)
  - New ReCoco operator: takes parsed AST nodes → emits classification metadata
  - Call `classifier.classify(node.kind(), lang, kind, purpose, is_root, categories)`
  - Positioned between parse and extract_symbols steps in ThreadFlowBuilder
  - Identifies definition boundaries (Rank 1-2 nodes) for L1 content addressing
  - Feature-gated behind `recoco-classify` feature in thread-flow/Cargo.toml

- [ ] **T-C11**: Implement AST fingerprinting for language identification fallback
  - Algorithm: parse with candidate grammar → classify all nodes → score = recognized/total
  - Threshold ~0.75 for confident identification; weight Rank 1–2 nodes more heavily
  - Integration: called when file extension is absent, ambiguous, or low-confidence
  - Lives in `thread-definitions` as `Classifier::fingerprint_language(ast_node_kinds: &[&str]) -> (f32, Option<Language>)`
  - Unit tests with known-language AST samples

- [ ] **T-C12**: Implement semantic query transform in `thread-flow`
  - `find_by_class(root: &AstNode, class: SemanticClass, classifier: &Classifier)` — language-agnostic node iterator
  - `find_by_rank(root: &AstNode, rank: ImportanceRank, classifier: &Classifier)` — filter by importance tier
  - `classify_tree(root: &AstNode, classifier: &Classifier) -> ClassifiedTree` — annotate full AST with SemanticClass
  - Depends on: T-C03 (classifier core) + thread-ast-engine AstNode API
  - These are the primary query interface for L1/L2 graph construction operators

## Phase 1: Setup
**Goal**: Initialize project structure and development environment.

- [ ] T001 Create `crates/thread-graph` — extend/wrap `crates/flow/src/incremental/graph.rs` (do NOT reimplement graph algorithms; expose as thread-graph public API)
- [ ] T002 Create `crates/thread-indexer` — build on `crates/flow/src/incremental/analyzer.rs` and `extractors/` (do NOT reimplement change detection or language extraction)
- [ ] T003 `[CF: boundary]` Create `crates/thread-conflict` stub crate with `lib.rs` and `Cargo.toml` — **OSS stub only**: crate exists to satisfy dependency declarations but contains no detection logic. All implementation is Commercial/Deferred (see Phase 4 notes). Detection logic (T029–T038) is not part of this task and must not be implemented here.
  > The shared types (`ConflictPrediction`, `ConflictType`, `Severity`, `DetectionTier`, `ConflictStatus`) are defined in `thread-api/src/types.rs`, not in `thread-conflict`. T003 creates an empty stub crate only — no type definitions needed here.
- [ ] T004 Create `crates/thread-storage` — re-export and extend `crates/flow/src/incremental/storage.rs` (StorageBackend trait and backends already implemented in thread-flow)
- [ ] T005 Create `crates/thread-api` with `lib.rs` and `Cargo.toml`
- [ ] T006 Create `crates/thread-realtime` with `lib.rs` and `Cargo.toml`
- [x] T007 ~~Update root `Cargo.toml`~~ — `thread-flow` is already a workspace member; add new crates as they are created
- [ ] T008 [P] Setup `xtask` for WASM build targeting `thread-wasm`
- [ ] T009 [P] Create `tests/contract` and `tests/integration` directories
- [ ] T010 [P] Create `tests/benchmarks` directory with scaffold files

## Phase 2: Foundational (Blocking Prerequisites)
**Goal**: Core data structures, traits, and storage implementations required by all user stories.

- [ ] T011 Implement rich `GraphNode` and `GraphEdge` structs with semantic metadata in `crates/thread-graph/` — NOTE: thread-flow has minimal `DependencyEdge`; the full semantic model (NodeType, SemanticMetadata, EdgeType enum with Calls/Inherits/etc.) still needs to be built
  - ⚠️ DEPENDS ON T-C09: GraphNode now uses `semantic_class: SemanticClass` + `node_kind: Option<Box<str>>` instead of `node_type: NodeType` enum. Complete T-C01 through T-C09 before implementing T011.
- [ ] **T011a**: Implement NodeId determinism property tests in `crates/thread-graph/tests/node_id_tests.rs`
  - Property: `NodeId::from_content(x) == NodeId::from_content(x)` always holds (same input = same output)
  - Property: `NodeId::from_content(x) != NodeId::from_content(y)` for all x ≠ y in test corpus
  - Fuzz test: normalized signature whitespace variations produce identical NodeId
  - Test: any change to normalization logic in `NodeId::from_content` fails at least one test
  - **Required before T012** — graph invariants depend on NodeId stability
- [ ] T012 Implement `Graph` container in `crates/thread-graph/` as a semantic graph layer — NOTE: `DependencyGraph` for incremental analysis already exists in thread-flow; this is the richer symbol-level graph
- [ ] **T012a**: Document `thread-graph` public API surface in `specs/001-realtime-code-graph/contracts/thread-graph-api.md`
  - Record all public traits, structs, and functions exposed by `crates/thread-graph/`
  - Include: `Graph`, `GraphNode`, `GraphEdge`, `OverlayView` trait, traversal functions, `NodeId`/`EdgeId` constructors
  - This document becomes the ground truth for contract tests (T027)
  - **Gate**: T027 (graph_storage integration test) MUST NOT be written until T012a exists
  - **Timing**: Written after T011/T012 TDD cycle stabilizes the API, before T027 begins
- [x] T013 ~~Implement `CasStorage` trait~~ — DONE: `StorageBackend` trait (async, full CRUD) implemented in `crates/flow/src/incremental/storage.rs`. Thread-storage crate should re-export this.
- [x] T014 ~~Implement `PostgresCas`~~ — DONE: `PostgresIncrementalBackend` implemented in `crates/flow/src/incremental/backends/postgres.rs`. Feature-gated behind `postgres-backend`.
- [x] T015 ~~Implement `D1Cas`~~ — DONE: `D1IncrementalBackend` implemented in `crates/flow/src/incremental/backends/d1.rs`. HTTP REST client for Cloudflare D1 API.
  > ⚠️ **Deprecation intent**: `D1IncrementalBackend` uses the D1 REST API and cannot meet the SC-STORE-001 <50ms p95 latency target due to the extra HTTP hop. It is retained for CI/CD tooling and external tooling use only. Production edge deployment MUST use `D1NativeBackend` (T016b, native `worker::D1Database` binding). `D1IncrementalBackend` will be deprecated once `D1NativeBackend` passes integration tests and benchmarks meet SC-STORE-001.
- [ ] T016 Implement `VectorizeStorage` for Cloudflare Vectorize API in `crates/thread-storage/src/vectorize.rs` — REPLACES QdrantStorage for edge deployment. NOTE: `recoco/target-qdrant` is currently disabled due to a CRC dependency conflict. Qdrant support (CLI-only) can be added later when the conflict is resolved. `[CF: OSS — follows D1 model; user-provided Cloudflare credentials]`
- [ ] **T016b**: Implement `D1NativeBackend` using `worker::D1Database` native binding in `crates/thread-storage/src/d1_native.rs` — required to meet SC-STORE-001 <50ms p95 target. Both `D1IncrementalBackend` and `D1NativeBackend` implement `StorageBackend` trait. `[CF: OSS]`
  - **Test strategy**: Use `miniflare` (local Cloudflare Workers runtime emulator) for unit and integration tests. CI integration tests require a Cloudflare staging environment via `wrangler dev`. Do not attempt to test `worker::D1Database` native bindings outside a Worker runtime context — they will not compile for the native target.
- [ ] T017 Define Protobuf message types (.proto) in `crates/thread-api/proto/v1/` and configure `prost` code generation. All `.proto` files use `package thread.v1;` namespace. Add `buf.yaml` and `buf.gen.yaml` for TypeScript client codegen targeting `@bufbuild/protobuf` (protobuf-es v2). Transport: plain HTTP POST — no Connect-RPC/gRPC framing. Document version bump policy in `crates/thread-api/proto/README.md`: field additions are backward-compatible; removing/renumbering fields or changing field types incompatibly requires a new `v2/` directory. Define conflict protocol types (`ConflictPrediction`, `ConflictType`, `Severity`, `DetectionTier`, `ConflictStatus`, `ResolutionStrategy`) in `crates/thread-api/src/types.rs` as Rust structs. These are shared between OSS `thread-api` and commercial `thread-conflict`.
- [x] T018 ~~Implement CocoIndex dataflow traits~~ — DONE: ReCoco integration implemented via bridge pattern:
  - `crates/flow/src/bridge.rs`: `CocoIndexAnalyzer` adapter
  - `crates/flow/src/flows/builder.rs`: `ThreadFlowBuilder` DSL
  - `crates/flow/src/functions/`: parse, symbols, imports, calls operators
  - Feature gating (`recoco-minimal`, `recoco-postgres`) prevents type leakage
- [ ] T019 Implement `RepoConfig` and `SourceType` in `crates/thread-indexer/src/config.rs`

## Phase 3: User Story 1 - Real-Time Code Analysis Query (P1)
**Goal**: Enable real-time dependency analysis and graph querying (<1s response).
**Independent Test**: Query a function's dependencies in a 50k file codebase and verify response < 1s.

- [ ] T020 [P] [US1] Create benchmark `tests/benchmarks/graph_queries.rs`
- [ ] T021 [US1] Implement AST to rich GraphNode conversion in `crates/thread-indexer/src/indexer.rs` — NOTE: Low-level dependency extraction for Rust/TS/Python/Go already exists in `crates/flow/src/incremental/extractors/`; this task builds the higher-level symbol→GraphNode mapping
- [ ] T022 [US1] Implement relationship extraction logic in `crates/thread-graph/src/algorithms.rs`
- [ ] T023 [US1] Implement `OverlayGraph` struct (merging Base + Delta) in `crates/thread-graph/src/overlay.rs`
- [ ] T024 [P] [US1] Implement `D1GraphIterator` for streaming access in `crates/thread-storage/src/d1.rs`
- [ ] T025 [US1] Expose graph traversal API in `crates/thread-graph/src/traversal.rs` — NOTE: BFS/topological sort/cycle detection already implemented in `crates/flow/src/incremental/graph.rs`; this wraps it in thread-graph's public API
- [ ] T026 [US1] Implement HTTP POST query handlers in `crates/thread-api/src/handlers.rs` using prost Protobuf encoding (plain HTTP POST, no Connect-RPC/gRPC framing)
- [ ] T026a [US1] Implement Circuit Breaker logic for data sources in `crates/thread-indexer/src/circuit_breaker.rs`
- [ ] T026b [US1] Implement Partial Graph Result Envelope in `crates/thread-api/src/response.rs`
- [ ] T027 [US1] Create integration test `tests/integration/graph_storage.rs` verifying graph persistence
- [ ] T028 [US1] Expose graph query API in `crates/thread-wasm/src/api_bindings.rs`

## Phase 4: User Story 2 - Conflict Prediction (P2) — ⚠️ DEFERRED: Commercial Scope

> **Note**: Conflict detection is a commercial differentiator. Tasks T029–T038 are deferred to a
> separate commercial design phase and will NOT be implemented in the OSS repository. The
> `thread-conflict` crate is classified as Commercial/TBD in spec.md. Tasks are preserved here
> for planning context and future commercial design input.
>
> The `ReachabilityIndex` infrastructure (T034) is OSS — the index lives in `thread-storage`.
> However, the conflict detection logic that consumes it (T029–T033, T036–T038) is commercial.

**Goal**: Detect merge conflicts before commit using multi-tier analysis.
**Independent Test**: Simulate concurrent changes to related files and verify conflict alert.

- [ ] T029 [P] [US2] [DEFERRED: commercial scope, separate design required] Create benchmark `tests/benchmarks/conflict_detection.rs`
- [ ] T030 [US2] [DEFERRED: commercial scope, separate design required] Implement `ConflictPrediction` struct in `crates/thread-conflict/src/types.rs`
- [ ] T030a [US2] [DEFERRED: commercial scope, separate design required] Implement `Delta` struct (representing local changes) in `crates/thread-graph/src/delta.rs`
- [ ] T031 [US2] [DEFERRED: commercial scope, separate design required] Implement Tier 1 AST diff detection in `crates/thread-conflict/src/tier1_ast.rs`
- [ ] T032 [US2] [DEFERRED: commercial scope, separate design required] Implement Tier 2 Structural analysis in `crates/thread-conflict/src/tier2_structural.rs`
- [ ] T033 [US2] [DEFERRED: commercial scope, separate design required] Implement Tier 3 Semantic analysis in `crates/thread-conflict/src/tier3_semantic.rs`
- [ ] T034 [US2] Implement `ReachabilityIndex` infrastructure in `crates/thread-storage/src/d1_reachability.rs` — OSS; k-hop bounded (k=3 default); tracks committed baseline. Conflict detection logic is commercial but the index itself is OSS.
- [ ] T035 [US2] Implement WebSocket/SSE notification logic in `crates/thread-realtime/src/websocket.rs` — OSS transport layer
- [ ] **T035b** `[BACKLOG]` Implement in-memory replay buffer for `thread-realtime` CLI WebSocket server
  - Configurable retention window (default: 5 minutes, max: 30 minutes)
  - Bounded by message count (max 500 messages) and memory (max 10MB)
  - Activated by `RequestMissedUpdates` client message
  - **Not blocked**: T035 (core WebSocket transport) proceeds without this
  - **Commercial equivalent**: Durable Objects replay (T-CF01 scope)
- [ ] T036 [US2] [DEFERRED: commercial scope, separate design required] Implement `ProgressiveConflictDetector` in `crates/thread-conflict/src/progressive.rs`
- [ ] T037 [US2] [DEFERRED: commercial scope, separate design required] Create integration test `tests/integration/realtime_conflict.rs`
- [ ] T038 [US2] [DEFERRED: commercial scope, separate design required] Expose conflict detection API in `crates/thread-wasm/src/realtime_bindings.rs`

## Phase 5: User Story 3 - Multi-Source Code Intelligence (P3)
**Goal**: Unified graph across multiple repositories and sources.
**Independent Test**: Index Git repo + local dir and verify cross-repo dependency link.

- [ ] T039 [US3] Implement `GitSource` in `crates/thread-indexer/src/sources/git.rs`
- [ ] T040 [US3] Implement `LocalSource` in `crates/thread-indexer/src/sources/local.rs`
- [ ] T041 [P] [US3] Implement `S3Source` in `crates/thread-indexer/src/sources/s3.rs`
- [ ] T042 [US3] Implement cross-repository dependency linking in `crates/thread-graph/src/linking.rs`
  > **Consistency**: Eventual consistency model. Cross-repo links are computed after each incremental update cycle. Stale links are flagged with `cross_repo_stale: true` in query responses. No distributed transaction or snapshot isolation required.
- [ ] T043 [US3] Update `ThreadBuildGraphFunction` to handle multiple sources
- [ ] T044 [US3] Create integration test `tests/integration/multi_source.rs`

## Phase 6: User Story 4 - AI-Assisted Conflict Resolution (P4) — ⚠️ DEFERRED: Commercial Scope

> **Note**: AI-assisted conflict resolution depends on conflict detection (Phase 4, also deferred).
> Tasks T045–T049 are deferred to the commercial design phase along with thread-conflict.

**Goal**: Suggest resolution strategies for detected conflicts.
**Independent Test**: Create conflict and verify resolution suggestion output.

- [ ] T045 [US4] [DEFERRED: commercial scope, requires thread-conflict] Implement `ResolutionStrategy` types in `crates/thread-conflict/src/resolution.rs`
- [ ] T046 [US4] [DEFERRED: commercial scope, requires thread-conflict] Implement heuristic-based resolution suggestions in `crates/thread-conflict/src/heuristics.rs`
- [ ] T047 [US4] [DEFERRED: commercial scope, requires thread-conflict] Implement semantic compatibility checks in `crates/thread-conflict/src/compatibility.rs`
- [ ] T048 [US4] [DEFERRED: commercial scope, requires thread-conflict] Update `ConflictPrediction` to include resolution strategies
- [ ] T049 [US4] [DEFERRED: commercial scope, requires thread-conflict] Add resolution tests in `crates/thread-conflict/tests/resolution_tests.rs`

## Phase 7: Polish & Cross-Cutting
**Goal**: Performance tuning, documentation, and final verification.

- [ ] T050 [P] Run and optimize benchmarks in `tests/benchmarks/`
- [ ] T051 Ensure >90% cache hit rate via `tests/benchmarks/cache_hit_rate.rs`
- [ ] T052 Verify incremental update performance in `tests/benchmarks/incremental_updates.rs`
- [ ] T053 Update `README.md` with usage instructions for new features
- [ ] T054 Create API documentation for new RPC endpoints
- [ ] T055 Final `mise run lint` and `cargo nextest` run

## Dependencies
- thread-definitions: Provides SemanticClass, ImportanceScores, AgentTask. Prerequisite for T011 (GraphNode), T-C10 (classify operator). Data migrated from classifications/ at T-C07.
- US2 depends on US1 (Graph foundation)
- US3 depends on US1 (Indexer foundation)
- US4 depends on US2 (Conflict detection)

## Implementation Foundation
- thread-flow provides: StorageBackend (T013-T015 ✅), DependencyGraph (T012 partial ✅), IncrementalAnalyzer (T021 partial ✅), ReCoco integration (T018 ✅), Language extractors (T021 partial ✅)
- Remaining: Semantic graph model (T011, T012), conflict detection (T029-T038), API layer (T017, T026), real-time (T035, T038), multi-source (T039-T043), Vectorize (T016)

## Parallel Execution Examples
- **Setup**: One dev creates crates (T001-T006) while another sets up CI/Tests (T008-T010).
- **Foundational**: Storage implementations (Postgres ✅ DONE, D1 ✅ DONE) and semantic graph model (T011) can proceed. Vectorize (T016) is a new item.
- **US1**: Indexer logic (T021) and Graph storage (T024) can proceed concurrently.

## Implementation Strategy
1. **MVP (US1)**: Focus on local CLI with Postgres and basic graph queries.
2. **Edge Enablement**: Port to WASM/D1 after core logic is stable.
3. **Real-time (US2)**: Add conflict detection once graph is reliable.
4. **Expansion (US3/4)**: Add multi-source and AI features last.

---

## Commercial Deployment Tasks (Private Crate)

> These tasks produce artifacts for the private commercial deployment crate (`crates/cloudflare` or equivalent private repo). They depend on Thread public crates but are not part of the Thread OSS repository. Listed here for planning visibility.
>
> **Dependency rule**: The private crate imports Thread public crates. Thread public crates never import or depend on the private crate.

- [ ] **T-CF01**: Implement `DurableObjectsBackend` for `thread-realtime::RealtimeBackend` trait
  - Cloudflare Durable Objects stateful actor implementation
  - WebSocket connection management via DO sessions
  - Depends on: T035 (OSS RealtimeBackend trait + WebSocket implementation)

- [ ] **T-CF02**: Implement multi-worker Cloudflare Workers architecture (FR-010 full)
  - Worker-to-Worker service bindings for language partitioning
  - Orchestrator Worker routing logic
  - Depends on: T038 (OSS WASM bindings), T-CF01 (DO backend)
  - Validates: SC-EDGE-004, SC-EDGE-005, SC-EDGE-006

- [ ] **T-CF03**: Wrangler configuration for 001 feature bindings
  - D1 database bindings for graph storage
  - Vectorize index bindings for semantic search
  - DO namespace bindings for real-time sessions
  - KV namespace bindings for caching
  - R2 bucket bindings for large payload offload (FR-022)
  - Add path dependencies for new 001 crates: `thread-graph`, `thread-indexer`, `thread-conflict`, `thread-api`, `thread-definitions`

- [ ] **T-CF04**: R2 large payload offload implementation (FR-022)
  - R2 client integration for conflict diffs and large AST payloads
  - Dead Letter Queue (DLQ) fallback pattern
  - Depends on: T022 (conflict detection), T-CF03 (R2 binding configuration)

- [ ] **T-CF05**: SC-EDGE-004 through SC-EDGE-006 validation in commercial deployment
  - Global latency validation (<50ms p95 from nearest POP)
  - Throughput validation (10k req/s per region)
  - Geographic coverage validation (<100ms from major cities worldwide)
  - Requires: T-CF02 deployed to production or staging
