# Tasks: Real-Time Code Graph Intelligence

**Feature**: `001-realtime-code-graph`
**Status**: Planning
**Generated**: 2026-01-11

## Phase 1: Setup
**Goal**: Initialize project structure and development environment.

- [ ] T001 Create `crates/graph` with `lib.rs` and `Cargo.toml`
- [ ] T002 Create `crates/indexer` with `lib.rs` and `Cargo.toml`
- [ ] T003 Create `crates/conflict` with `lib.rs` and `Cargo.toml`
- [ ] T004 Create `crates/storage` with `lib.rs` and `Cargo.toml`
- [ ] T005 Create `crates/api` with `lib.rs` and `Cargo.toml`
- [ ] T006 Create `crates/realtime` with `lib.rs` and `Cargo.toml`
- [ ] T007 Update root `Cargo.toml` to include new workspace members
- [ ] T008 [P] Setup `xtask` for WASM build targeting `thread-wasm`
- [ ] T009 [P] Create `tests/contract` and `tests/integration` directories
- [ ] T010 [P] Create `tests/benchmarks` directory with scaffold files

## Phase 2: Foundational (Blocking Prerequisites)
**Goal**: Core data structures, traits, and storage implementations required by all user stories.

- [ ] T011 Implement `GraphNode` and `GraphEdge` structs in `crates/graph/src/node.rs` and `crates/graph/src/edge.rs` with full provenance fields (T079)
- [ ] T011b Implement `Provenance` types (`SourceVersion`, `LineageRecord`) and trait wrappers in `crates/graph/src/provenance.rs`
- [ ] T012 Implement `Graph` container and adjacency list in `crates/graph/src/graph.rs`
- [ ] T013 Implement `GraphStorage` trait in `crates/storage/src/traits.rs`
- [ ] T014 [P] Implement `PostgresStorage` for `GraphStorage` in `crates/storage/src/postgres.rs`
- [ ] T015 [P] Implement `D1Storage` for `GraphStorage` in `crates/storage/src/d1.rs`
- [ ] T016 [P] Implement `QdrantStorage` struct in `crates/storage/src/qdrant.rs`
- [ ] T017 Define shared RPC types in `crates/api/src/types.rs` based on `specs/001-realtime-code-graph/contracts/rpc-types.rs`
- [ ] T018 Implement CocoIndex dataflow traits in `crates/services/src/dataflow/traits.rs` covering provenance collection
- [ ] T019 Implement `RepoConfig` and `SourceType` in `crates/indexer/src/config.rs`

## Phase 3: User Story 1 - Real-Time Code Analysis Query (P1)
**Goal**: Enable real-time dependency analysis and graph querying (<1s response).
**Independent Test**: Query a function's dependencies in a 50k file codebase and verify response < 1s.

- [ ] T020 [P] [US1] Create benchmark `tests/benchmarks/graph_queries.rs`
- [ ] T021 [US1] Implement AST to Graph Node conversion in `crates/indexer/src/indexer.rs`
- [ ] T022 [US1] Implement relationship extraction logic in `crates/graph/src/algorithms.rs`
- [ ] T023 [US1] Implement `ThreadBuildGraphFunction` in `crates/services/src/functions/build_graph.rs` using CocoIndex traits
- [ ] T024 [P] [US1] Implement `D1GraphIterator` for streaming access in `crates/storage/src/d1.rs`
- [ ] T025 [US1] Implement graph traversal algorithms (BFS/DFS) in `crates/graph/src/traversal.rs`
- [ ] T026 [US1] Implement RPC query handlers in `crates/api/src/rpc.rs`
- [ ] T027 [US1] Create integration test `tests/integration/graph_storage.rs` verifying graph persistence
- [ ] T028 [US1] Expose graph query API in `crates/wasm/src/api_bindings.rs`

## Phase 4: User Story 2 - Conflict Prediction (P2)
**Goal**: Detect merge conflicts before commit using multi-tier analysis.
**Independent Test**: Simulate concurrent changes to related files and verify conflict alert.

- [ ] T029 [P] [US2] Create benchmark `tests/benchmarks/conflict_detection.rs`
- [ ] T030 [US2] Implement `ConflictPrediction` struct in `crates/conflict/src/types.rs`
- [ ] T031 [US2] Implement Tier 1 AST diff detection in `crates/conflict/src/tier1_ast.rs`
- [ ] T032 [US2] Implement Tier 2 Semantic analysis in `crates/conflict/src/tier2_semantic.rs`
- [ ] T033 [US2] Implement Tier 3 Graph impact analysis in `crates/conflict/src/tier3_graph.rs`
- [ ] T034 [US2] Implement `ReachabilityIndex` logic for D1 in `crates/storage/src/d1_reachability.rs`
- [ ] T035 [US2] Implement WebSocket/SSE notification logic in `crates/realtime/src/websocket.rs`
- [ ] T036 [US2] Implement `ProgressiveConflictDetector` in `crates/conflict/src/progressive.rs`
- [ ] T037 [US2] Create integration test `tests/integration/realtime_conflict.rs`
- [ ] T038 [US2] Expose conflict detection API in `crates/wasm/src/realtime_bindings.rs`

## Phase 5: User Story 3 - Multi-Source Code Intelligence (P3)
**Goal**: Unified graph across multiple repositories and sources.
**Independent Test**: Index Git repo + local dir and verify cross-repo dependency link.

- [ ] T039 [US3] Implement `GitSource` in `crates/indexer/src/sources/git.rs`
- [ ] T040 [US3] Implement `LocalSource` in `crates/indexer/src/sources/local.rs`
- [ ] T041 [P] [US3] Implement `S3Source` in `crates/indexer/src/sources/s3.rs`
- [ ] T042 [US3] Implement cross-repository dependency linking in `crates/graph/src/linking.rs`
- [ ] T043 [US3] Update `ThreadBuildGraphFunction` to handle multiple sources
- [ ] T044 [US3] Create integration test `tests/integration/multi_source.rs`

## Phase 6: User Story 4 - AI-Assisted Conflict Resolution (P4)
**Goal**: Suggest resolution strategies for detected conflicts.
**Independent Test**: Create conflict and verify resolution suggestion output.

- [ ] T045 [US4] Implement `ResolutionStrategy` types in `crates/conflict/src/resolution.rs`
- [ ] T046 [US4] Implement heuristic-based resolution suggestions in `crates/conflict/src/heuristics.rs`
- [ ] T047 [US4] Implement semantic compatibility checks in `crates/conflict/src/compatibility.rs`
- [ ] T048 [US4] Update `ConflictPrediction` to include resolution strategies
- [ ] T049 [US4] Add resolution tests in `crates/conflict/tests/resolution_tests.rs`

## Phase 7: Polish & Cross-Cutting
**Goal**: Performance tuning, documentation, and final verification.

- [ ] T050 [P] Run and optimize benchmarks in `tests/benchmarks/`
- [ ] T051 Ensure >90% cache hit rate via `tests/benchmarks/cache_hit_rate.rs`
- [ ] T052 Verify incremental update performance in `tests/benchmarks/incremental_updates.rs`
- [ ] T053 Update `README.md` with usage instructions for new features
- [ ] T054 Create API documentation for new RPC endpoints
- [ ] T055 Final `mise run lint` and `cargo nextest` run

## Dependencies
- US2 depends on US1 (Graph foundation)
- US3 depends on US1 (Indexer foundation)
- US4 depends on US2 (Conflict detection)

## Parallel Execution Examples
- **Setup**: One dev creates crates (T001-T006) while another sets up CI/Tests (T008-T010).
- **Foundational**: Storage implementations (Postgres, D1, Qdrant) can be built in parallel.
- **US1**: Indexer logic (T021) and Graph storage (T024) can proceed concurrently.

## Implementation Strategy
1. **MVP (US1)**: Focus on local CLI with Postgres and basic graph queries.
2. **Edge Enablement**: Port to WASM/D1 after core logic is stable.
3. **Real-time (US2)**: Add conflict detection once graph is reliable.
4. **Expansion (US3/4)**: Add multi-source and AI features last.
