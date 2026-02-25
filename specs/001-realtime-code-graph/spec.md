<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: AGPL-3.0-or-later
-->

# Feature Specification: Real-Time Code Graph Intelligence

**Feature Branch**: `001-realtime-code-graph`
**Created**: 2026-01-10
**Status**: Draft
**Input**: User description: "Build an application that can provide performant, real-time, code-base-wide graph intelligence with semantic/ast awareness. I want it to be able to interface with any data source, database target, work locally and in the cloud, and plug and change out underlying engines. It needs to be fast and cloudflare deployable. This will server as the foundational intelligence layer for the future of work -- enabling real time and asynchronous human-ai teaming with intelligent conflict prediction and resolution"

## Related Documents

| Document | Location | Role |
|----------|----------|------|
| Semantic Classification Spec | [`docs/architecture/SEMANTIC_CLASSIFICATION_SPEC.md`](../../docs/architecture/SEMANTIC_CLASSIFICATION_SPEC.md) | Canonical implementation reference for `thread-definitions` — classifier internals, 8-stage lookup pipeline, data schemas, scoring model, language-agnostic query design |
| AI Knowledge Layer Design | [`docs/architecture/AI_KNOWLEDGE_LAYER_DESIGN.md`](../../docs/architecture/AI_KNOWLEDGE_LAYER_DESIGN.md) | Background architectural proposal for the multi-resolution knowledge layer (L0–L4); predates the classifier port proposal |
| Implementation Plan | [`specs/001-realtime-code-graph/plan.md`](./plan.md) | Phased implementation plan, crate breakdown, dependency graph |
| Tasks | [`specs/001-realtime-code-graph/tasks.md`](./tasks.md) | Ordered task list including Phase 0.5 (T-C01–T-C12) for `thread-definitions` |

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Real-Time Code Analysis Query (Priority: P1)

A developer working on a large codebase needs to understand the impact of a proposed change to a function. They query the graph intelligence system to see all dependencies, callers, and semantic relationships for that function in real-time.

**Why this priority**: This is the foundational use case that delivers immediate value. Without fast, accurate dependency analysis, developers cannot confidently make changes. This capability alone justifies the system's existence and enables all higher-level features.

**Independent Test**: Can be fully tested by querying a single function's relationships in a known codebase and verifying all dependencies are returned in under 1 second. Delivers value by reducing manual code navigation from minutes to seconds.

**Acceptance Scenarios**:

1. **Given** a codebase with 50,000 files indexed in the graph, **When** developer queries dependencies for function "processPayment", **Then** system returns complete dependency graph with all callers, callees, and data flows in under 1 second
2. **Given** developer is viewing a function, **When** they request semantic relationships, **Then** system highlights similar functions, related types, and usage patterns with confidence scores
3. **Given** multiple developers querying simultaneously, **When** 100 concurrent queries are issued, **Then** all queries complete within 2 seconds with <10% latency increase

---

### User Story 2 - Conflict Prediction for Team Collaboration (Priority: P2)

Two developers are working on different features that unknowingly modify overlapping parts of the codebase. The system detects the potential conflict before code is committed and alerts both developers with specific details about what will conflict and why.

**Why this priority**: Prevents integration failures and reduces rework. This builds on the graph analysis capability (P1) but adds proactive intelligence. High value but requires P1 foundation.

**Independent Test**: Can be tested by simulating two concurrent changes to related code sections and verifying the system predicts the conflict with specific file/line details before merge. Delivers value by preventing merge conflicts that typically take 30+ minutes to resolve.

**Acceptance Scenarios**:

1. **Given** two developers editing different files **and developer B's active working changes are visible to the system** (either committed to the shared baseline, or available via Delta-sharing within the same active session), **When** developer A saves a file (file system write event detected by the `thread-indexer` watcher) that affects a function call chain modified by developer B, **Then** system detects potential conflict and notifies both developers within 5 seconds of the save event
2. **Given** a developer modifying a widely-used API, **When** the change would break 15 downstream callers, **Then** system lists all affected callers with severity ratings before commit
3. **Given** asynchronous work across timezones, **When** developer A's changes conflict with developer B's 8-hour-old WIP branch, **Then** system provides merge preview showing exactly what will conflict

> **Trigger Note**: Conflict detection is triggered by file system watcher events (same mechanism as FR-013 real-time propagation). No explicit user action is required. Developers working in any editor that saves to disk automatically participate in real-time conflict detection. This applies to OSS CLI deployment; edge deployment conflict detection (commercial) uses the same watcher event as the source trigger, forwarded to the Container analysis service.

---

### User Story 3 - Multi-Source Code Intelligence (Priority: P3)

A team's codebase spans multiple repositories (monorepo + microservices) stored in different systems (GitHub, GitLab, local file systems). The graph intelligence system indexes and analyzes code from all sources, providing unified cross-repository dependency tracking.

**Why this priority**: Essential for modern distributed architectures but builds on core graph capabilities. Can be delivered later without blocking P1/P2 value.

**Independent Test**: Can be tested by indexing code from two different Git repositories and one local directory, then querying cross-repository dependencies. Delivers value by eliminating manual cross-repo dependency tracking.

**Acceptance Scenarios**:

1. **Given** three code repositories (GitHub, GitLab, local), **When** system indexes all three sources, **Then** unified graph shows dependencies across all sources within 10 minutes for 100k total files
2. **Given** a function in repo A calls an API in repo B, **When** developer queries the function, **Then** system shows the cross-repository dependency with source attribution
3. **Given** one repository updates its code, **When** incremental update runs, **Then** only affected cross-repository relationships are re-analyzed (not full re-index)

---

### User Story 4 - AI-Assisted Conflict Resolution (Priority: P4)

When a conflict is predicted, the system suggests resolution strategies based on semantic understanding of the code changes. It provides contextual recommendations like "Developer A's change improves performance, Developer B's adds security validation - both changes are compatible and can be merged in sequence."

**Why this priority**: High value but requires sophisticated AI integration and successful conflict prediction (P2). Can be delivered incrementally after core features are stable.

**Independent Test**: Can be tested by creating a known conflict scenario and verifying the system generates actionable resolution suggestions with reasoning. Delivers value by reducing conflict resolution time from 30 minutes to 5 minutes.

**Acceptance Scenarios**:

1. **Given** a detected conflict between two changes, **When** both changes are analyzed semantically, **Then** system provides resolution strategy with confidence score and reasoning
2. **Given** conflicting changes to the same function, **When** one change modifies logic and other adds logging, **Then** system recommends specific merge order and identifies safe integration points
3. **Given** breaking API change conflict, **When** system analyzes impact, **Then** it suggests adapter pattern or migration path with code examples formatted as fenced code blocks. Code examples are language-specific compilable snippets where the conflict context allows deterministic generation (known language, clear symbol signatures); structured pseudocode with inline comments otherwise.

> **Resolution Format Note**: AI-generated resolution suggestions use fenced code blocks throughout. The system targets language-specific compilable code for common patterns (adapter, migration, signature update) where the AST context provides sufficient precision. When the AI cannot determine a compilable form with high confidence, structured pseudocode with explanatory comments is used. The specific AI integration (Workers AI, external LLM, or local model) is a commercial implementation detail — this spec defines the output format contract only.

---

### Edge Cases

| Edge Case | Handled By |
|-----------|------------|
| Codebase larger than available memory (1M+ files) | FR-022 (memory governance, adaptive batching), FR-024 (partial graph results) |
| Circular dependencies in the code graph | FR-025 (cycle detection, depth-limiting) |
| Two data sources contain the same file with different versions | FR-004 (CAS — same content hash = same entry; different content = different entries, both retained) |
| Developer offline for extended periods (conflict prediction) | SC-002 deferred to commercial scope (thread-conflict); FR-017 Overlay Graph tracks committed baseline for offline comparison |
| Underlying analysis engine crashes mid-query | FR-023 (circuit breaker), FR-024 (partial results with allow_partial flag) |
| Generated code files that change frequently | FR-008 (incremental updates), FR-012 (CAS deduplication — identical generated output hits cache) |
| Database connection lost during real-time updates | FR-023 (circuit breaker for storage backends), FR-024 (partial results) |
| Version drift between local and cloud deployments | FR-017 (Overlay Graph — Base Layer is immutable committed state; Deltas are ephemeral) |

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST parse and analyze source code to build AST (Abstract Syntax Tree) representations for all supported languages
- **FR-002**: System MUST construct a graph representation of codebase relationships including: function calls, type dependencies, data flows, and import/export chains
- **FR-003**: System MUST index code from configurable data sources including: local file systems, Git repositories (GitHub, GitLab, Bitbucket), and cloud storage (S3-compatible)
- **FR-004**: System MUST store analysis results using a **Content-Addressed Storage (CAS)** model.
  - **Schema**: Data is stored as immutable "Graph Chunks" keyed by content hash (e.g., `hash(file_content) -> graph_data`).
  - **Backends**: Postgres (Local CAS), D1 (Cloud CAS).
  - **Consistency**: Consistency is achieved via immutability; a specific hash ALWAYS maps to the same graph data.
  - **Vectorize** (Cloudflare, Edge): Stores vector embeddings for semantic similarity search on the edge. **Qdrant** (optional, CLI-only): Self-hosted vector backend for local deployments. Note: ReCoco's Qdrant target is currently disabled due to a dependency conflict; Vectorize is the primary vector backend for edge deployment.
- **FR-005**: System MUST support real-time graph queries responding within 1 second for codebases up to 100k files
- **FR-006**: System MUST detect and classify concurrent code changes into a Three-Tier Conflict Taxonomy: **Tier 1 (Syntactic)** for parse/compile errors (detected via AST diff <100ms), **Tier 2 (Structural)** for valid syntax but broken linking/structure (detected via Symbol Graph <1s), and **Tier 3 (Semantic)** for valid structure but incompatible logic/behavior (detected via Dataflow/Semantic analysis <5s). Results update progressively as each tier completes.
- **FR-007**: System MUST provide conflict predictions with specific details: file locations, conflicting symbols, impact severity ratings, confidence scores, and conflict tier classification. Initial predictions (Tier 1) deliver within 100ms, refined predictions (Tier 2) within 1 second, comprehensive predictions (Tier 3) within 5 seconds. If a detection tier fails to complete (timeout or circuit breaker), the system MUST send a terminal `ConflictUpdate` with `status: Timeout, final: true` containing the last known tier result. Silence after a tier fires is NOT acceptable.
- **FR-008**: System MUST support incremental updates where only changed files and affected dependencies are re-analyzed
- **FR-009**: System MUST allow pluggable analysis engines where the underlying AST parser, graph builder, or conflict detector can be swapped via **compile-time composition only** — not runtime plugin loading, not hot-swappable without recompilation. Swapping an engine requires: (1) implementing the relevant trait (e.g., `CodeAnalyzer`, `GraphBuilder`), (2) registering the implementation in the `FactoryRegistry`, (3) updating the configuration file to activate it, and (4) recompiling. Zero changes to pipeline orchestration code in `thread-flow` or `thread-services` are required. Adding a new crate dependency and implementing a trait is the expected workflow — this is not "rewriting application code." This abstraction MUST support diverse type systems (e.g., CodeWeaver's "Things/Connections" model) alongside standard Tree-sitter nodes.
- **FR-010**: System MUST deploy to Cloudflare Workers using a **Multi-Worker Architecture** to support ~166 languages. The architecture consists of a central Router/Handler Worker that delegates to specialized Language Workers via Service Bindings. **OSS Boundary**: OSS distribution includes a simplified single-worker deployment bundling only core languages (Rust, Python, TypeScript) to minimize complexity. **Constraint**: Edge deployment MUST NOT load full graph into memory. Must use streaming/iterator access patterns and D1 Reachability Index.

  > **Implementation note**: The `thread-definitions` semantic classifier provides 80%+ accuracy on any tree-sitter grammar out of the box via universal rules (2,444 cross-language patterns). Full language support (~100%) requires only ~10–50 lines of TOML overrides per language. Target: all ~166 tree-sitter-language-pack languages. File-extension language identification for ~200 languages available from CodeWeaver as `data/file_extensions.json`.
- **FR-LANGDETECT**: Language identification SHALL use a two-tier strategy: (1) hardcoded extension lookup (primary, zero-cost), (2) AST fingerprinting fallback — parse with candidate grammar, classify node types, score = recognized/total; grammar with highest score (threshold ~0.75) is the probable language. Enables reliable detection for extensionless files and ambiguous cases.
- **FR-011**: System MUST run as a local CLI application for developer workstation use (available in OSS). **Local-Only Mode**: In this mode, Postgres serves as both the CAS store and the "Real-Time Service" (managing the Overlay/Deltas in memory), ensuring full functionality without cloud connectivity.
- **FR-012**: System MUST use content-addressed caching to avoid re-analyzing identical code sections across updates
- **FR-013**: System MUST propagate code changes to all connected clients within 100ms, measured from the moment the file system event is received by the `thread-indexer` watcher (or equivalent source event for non-filesystem sources), to the moment the first WebSocket/SSE message is sent to connected clients. This budget covers: event receipt → incremental analysis → graph delta computation → client notification. Applies to CLI deployment; edge deployment target is 200ms p95 due to additional Container→Worker hop.
- **FR-014**: System MUST track analysis provenance showing which data source, version, and timestamp each graph node originated from
- **FR-015**: System MUST support semantic search across the codebase to find similar functions, related types, and usage patterns. When the vector search backend (Vectorize/Qdrant) is unavailable, FR-015 MUST degrade gracefully to AST-based search using `SemanticClass` and importance scores from `thread-definitions`. AST-based search still provides rich structural results (function definitions, type relationships, call patterns) — it loses only vector similarity ranking. Responses in degraded mode include `"search_mode": "ast_semantic"` to distinguish from full vector search. This degraded mode is NOT 'keyword-only' — it leverages the full semantic classification layer.
- **FR-016**: System MUST provide graph traversal APIs using **prost**-generated Protobuf message encoding over plain **HTTP POST** transport (`Content-Type: application/x-protobuf`). There is NO Connect-RPC or gRPC framing — Cloudflare Workers do not support HTTP/2 trailers required by Connect-RPC/gRPC. TypeScript clients use `buf` CLI + `@bufbuild/protobuf` (protobuf-es v2) for type-safe code generation from `.proto` definitions. Internal Rust-to-Rust communication (Worker→Container service bindings, Container internal, CLI internal calls) uses **postcard** for compact binary serialization. MCP server integration (future) uses `serde_json`/JSON-RPC 2.0 as a separate transport adapter. API type definitions are centralized in the `thread-api` crate to ensure type safety across CLI (Rust), Edge (WASM), and Web (TypeScript) clients. All `.proto` files MUST use `package thread.v1;` versioning. Proto files are committed at `crates/thread-api/proto/v1/`. Field additions within a version are backward-compatible and do not require a version bump. Removing or renumbering fields, or changing field types incompatibly, requires a new package version (`thread.v2;`) and a corresponding new proto directory. The TypeScript client regenerates from the versioned proto directory via `buf generate`.
- **FR-017**: System MUST utilize an **Overlay Graph Architecture** to manage state and consistency.
  - **Base Layer (Immutable)**: Represents the graph at a specific Git commit, stored in D1 (Cloud) or Postgres (Local).
  - **Delta Layer (Ephemeral)**: Represents local uncommitted changes (dirty state), stored in memory or temporary local storage.
  - **Unified View**: The query engine merges Base + Delta at runtime to provide a real-time view without modifying the persistent Base storage.
  - **Conflict Detection**: Performed by comparing active Deltas from different users against the Base, rather than merging database states.
  - **Default Behavior**: The Unified View (Base + Delta merge) is the default query behavior. Callers receive their local uncommitted changes automatically reflected in all graph query results. To query the committed Base Layer only (excluding local deltas), callers pass `include_local_delta: false` in the query request. This opt-out is useful for: comparing local changes against the committed baseline, debugging conflict predictions, or generating reports from stable committed state.
- **FR-018**: System MUST maintain graph consistency when code is added, modified, or deleted during active queries
- **FR-019**: System MUST log all conflict predictions and resolutions for audit and learning purposes
- **FR-020**: System MUST handle authentication and authorization for multi-user scenarios when deployed as a service, utilizing standard **OAuth2/OIDC** protocols.
- **FR-021**: System MUST expose metrics for: query performance, cache hit rates, indexing throughput, and storage utilization. This covers the `/metrics` and `/health` HTTP endpoints. Log stream observability (structured per-operation logs across pipeline crates) is covered by FR-027.
- **FR-022**: System MUST utilize batched database operations (D1 Batch API) and strictly govern memory usage (<80MB active set) on Edge via ReCoco adaptive controls (limiting in-flight rows and bytes) to prevent OOM errors. Large payloads exceeding storage backend limits MUST be offloaded via a configurable **Large Payload Offload** strategy rather than failing the write. The offload strategy is backend-specific (e.g., R2 + Dead Letter Queue for Cloudflare edge deployment) and implemented in the deployment layer, not the OSS library. **Recommended thresholds** (informative, not normative for OSS): trigger offload when a single payload item exceeds 512KB or when a batch exceeds 20MB. Offloaded items MUST be processed asynchronously and retried until acknowledged; a maximum retry count (recommended: 5) after which items are logged and discarded. Offload queue depth MUST be observable via the metrics endpoint (FR-021).
- **FR-023**: System MUST implement a **Circuit Breaker** pattern for data sources. If a source fails >5 times in 30s, it moves to OPEN state. After 60s in OPEN state, it moves to HALF-OPEN to allow a single probe request to verify source health. Circuit breaker pattern applies to: configured Git/S3/GitHub/GitLab data sources, Postgres storage backend, D1 storage backend, and Vectorize/Qdrant vector search backends.
- **FR-024**: System MUST support **Partial Graph Results**. Query APIs must accept an `allow_partial=true` flag and return a "Graph Result Envelope" containing available subgraphs, a list of missing regions, and error details, rather than failing the entire query.
**FR-023/FR-024 Interaction**: When a circuit breaker is OPEN for a required data source and an incoming query has `allow_partial=false`:
1. **If request timeout budget allows**: Queue the request. When the circuit moves to HALF-OPEN and the probe succeeds, process the queued request. Return a `Retry-After` header indicating estimated wait time.
2. **If timeout budget is exceeded before HALF-OPEN**: Return an error response with `{"error": "CIRCUIT_OPEN", "source": "<source_id>", "retry_after_seconds": <n>, "partial_available": true}`. Include a hint that retrying with `allow_partial=true` would return available data immediately.

Queued requests are bounded: maximum 100 queued requests per circuit-broken source. Beyond this limit, immediately return the error response.

- **FR-025**: System MUST detect and handle circular dependencies via depth-limiting and cycle detection mechanisms to prevent infinite recursion during graph traversal.
- **FR-026**: System MUST expose a health check endpoint `GET /health` returning a JSON response within 50ms: `{"status": "ok"|"degraded"|"starting", "cache_hit_rate": <f32>, "lag_ms": <u64>, "storage_ok": <bool>}`. `"starting"` status indicates vector index warmup in progress; core graph queries remain available. `"degraded"` indicates a storage backend circuit breaker is OPEN. This endpoint requires no authentication.

- **FR-027**: System MUST emit structured logs in JSON format for all significant operations throughout the analysis pipeline. Minimum required fields per log entry: `timestamp` (ISO-8601), `level` (`error`/`warn`/`info`/`debug`), `component` (crate name), `operation` (pipeline stage or function name), `duration_ms` (for timed operations), and applicable entity IDs (`session_id`, `repository_id` where available). Errors MUST include `error_type` and `context` fields.

  **Deployment paths**:
  - **Edge**: `workers-rs` log macros → Cloudflare Workers Logs → automated OTEL export. No manual trace ID propagation is required in the `thread-api` protocol — Cloudflare handles span correlation across the Worker→Container hop automatically.
  - **CLI**: `tracing` crate with JSON subscriber (`tracing-subscriber` + `fmt` JSON format). Human-readable pretty-print format available via feature flag or environment variable for local development.

**FR-CLASSIFY**: The system MUST classify all extracted AST node types into one of 22 language-agnostic `SemanticClass` categories using the `thread-definitions` classifier, enabling AI-context importance ranking.

- Classification pipeline: Override → FileDetection → TokenPurpose → UniversalExact → UniversalMajority → Category → NameHeuristic → Unclassified
- Accuracy: ≥99% across 27 validated languages; ≥80% baseline on any tree-sitter grammar
- Scoring: Per-class `ImportanceScores` (5 dimensions) with optional per-`AgentTask` `ContextualAdjustments`
- Storage: `semantic_class` field on `GraphNode`; importance scores computed on-demand

**Success criteria:**
- All `GraphNode`s have a populated `semantic_class` field
- Context pack generation can rank definitions by `task_score(class, agent_task)`
- New language support achievable via TOML overrides without Rust code changes

**FR-LANGQUERY**: The system SHALL support language-agnostic semantic queries via `SemanticClass` — callers find `DefinitionCallable` nodes without knowing that Rust uses `function_item`, Python uses `function_definition`, or Go uses `function_declaration`. The query adapter lives in `thread-flow`; `thread-definitions` and `thread-ast-engine` remain mutually independent.

### Key Entities

- **Code Repository**: Represents a source of code (Git repo, local directory, cloud storage). Attributes: source type, connection credentials, sync frequency, last sync timestamp
- **Code File**: Individual file in a repository. Attributes: file path, language, content hash, AST representation, last modified timestamp
- **Graph Node**: Represents a code symbol (function, class, variable, type). Attributes: symbol name, location (file + line), semantic metadata, relationships to other nodes
  - `semantic_class: SemanticClass` — Language-agnostic 22-category classification. Carries importance tier (5 ranks) and AI-task scoring. Replaces the former node_type enum. Sourced from `thread-definitions` crate. Examples: `DefinitionCallable` (functions, methods, constructors), `DefinitionType` (classes, structs, traits), `BoundaryModule` (imports, exports, module declarations).
  - `node_kind: Option<Box<str>>` — Raw tree-sitter node type name for finer structural distinctions. Examples: `"function_item"` vs `"closure_expression"` (both `DefinitionCallable`), `"impl_item"` (Rust impl block, `DefinitionType` container). `None` for nodes derived from higher-level analysis.
- **Graph Edge**: Represents a relationship between nodes. Attributes: relationship type (calls, imports, inherits, uses), direction, strength/confidence score
- **Conflict Prediction**: Represents a detected potential conflict. Attributes: affected files, conflicting developers, conflict type, severity, suggested resolution, timestamp
- **Analysis Session**: Represents a single analysis run. Attributes: start time, completion time, files analyzed, nodes/edges created, cache hit rate
- **Analysis Engine**: Represents a pluggable component. Attributes: engine type (parser, graph builder, conflict detector), version, configuration parameters

## Success Criteria *(mandatory)*

### Technical Success Criteria

Measurable, automatable outcomes tied to functional requirements.

- **SC-001**: Developers can query code dependencies and receive complete results in under 1 second for codebases up to 100,000 files
- **SC-002**: System detects 95% of potential merge conflicts before code is committed, with false positive rate below 10%. False Positive defined as: A predicted conflict that is manually dismissed by the user or successfully merged without modification. *(Commercial scope — requires thread-conflict crate; deferred.)*
- **SC-003**: Incremental indexing completes in under 10% of full analysis time for typical code changes (affecting <5% of files)
- **SC-004**: System handles 1000 concurrent users querying simultaneously with <2 second p95 response time
- **SC-006**: Cross-repository dependency tracking works across 5+ different code sources without manual configuration
- **SC-007-OSS**: `ReachabilityIndex` returns correct k-hop ancestor/descendant sets for 100% of test cases in `tests/benchmarks/reachability_accuracy.rs`. Test corpus: 10,000-node synthetic graph with known ground-truth reachability up to k=3 hops (FR-017, T034). *(OSS proxy for SC-002, which is deferred to commercial scope with thread-conflict.)*
- **SC-035-OSS**: WebSocket transport delivers `CodeChangeDetected` and `GraphUpdate` messages to all connected test clients within 100ms in the integration test suite for `thread-realtime` (T035). Verified against a local mock repository watcher with 50 concurrent test connections.

### Product Goals *(tracked by product metrics, not automated tests)*

These express desired user outcomes. They are not directly verifiable by automated tests and are tracked
via usage analytics, user surveys, and adoption metrics. They require the commercial conflict detection
features (thread-conflict) to be meaningful.

- **SC-005**: Conflict resolution time reduces by 70% (from 30 minutes to under 10 minutes) when using AI-assisted suggestions *(requires thread-conflict — deferred to commercial)*
- **SC-007**: Developer satisfaction score of 4.5/5 for "confidence in making code changes" after using conflict prediction *(requires thread-conflict — deferred to commercial)*
- **SC-008**: 90% of developers successfully integrate the system into their workflow within first week of adoption
- **SC-009**: Real-time collaboration features reduce integration delays from hours to minutes (75% improvement) *(requires thread-conflict — deferred to commercial)*
- **SC-010**: System operates with 99.9% uptime when deployed to Cloudflare edge network *(SLA target — tracked operationally via uptime monitoring, not by automated test)*

### Service Architecture Success Criteria

**Deployment Targets**: Both CLI and Edge

#### Cache Performance

- **SC-CACHE-001**: Content-addressed cache achieves >90% hit rate for repeated analysis of unchanged code sections
- **SC-CACHE-002**: Cache invalidation occurs within 100ms of source code change detection
- **SC-CACHE-003**: Cache size remains under 500MB for 10k file repository, scaling linearly with codebase size
- **SC-CACHE-004**: Core AST graph analysis is available immediately on deployment — there is no warmup period for graph queries, dependency analysis, or semantic classification. Vector search (FR-015, Vectorize/Qdrant) may require index warmup; during this period, queries return results with `semantic_search_available: false` and fall back to AST-based search (see D7/FR-015 degraded mode). Cache warmup for previously-analyzed codebases (restoring from persistent storage) completes in under 5 minutes.
- **SC-HEALTH-001**: Health endpoint responds within 50ms under normal load and within 200ms during peak indexing. Returns `status: "starting"` during cold-start warmup and `status: "degraded"` when any circuit breaker is OPEN (FR-026).

#### Incremental Updates

- **SC-INCR-001**: Code changes trigger only affected component re-analysis, not full codebase scan
- **SC-INCR-002**: Incremental update completes in <10% of full analysis time for changes affecting <5% of files
- **SC-INCR-003**: Dependency graph updates propagate to all connected clients in <100ms (measured from watcher event receipt)
- **SC-INCR-004**: Change detection accurately identifies affected files with 99% precision (no missed dependencies)

#### Storage Performance

- **SC-STORE-001**: Database operations meet constitutional targets:
  - Postgres (CLI): <10ms p95 latency for graph traversal queries
  - D1 (Edge): <50ms p95 latency for distributed edge queries
  - Vectorize (edge vectors): <100ms p95 latency for semantic similarity search (edge deployment)
- **SC-STORE-002**: Graph schema handles up to 10 million nodes and 50 million edges per deployment
- **SC-STORE-003**: Database write throughput supports 1000 file updates per second during bulk re-indexing
- **SC-STORE-004**: Storage growth is sub-linear to codebase size through effective deduplication (1.5x raw code size maximum)

#### Edge Deployment

- **SC-EDGE-001**: WASM binary compiles successfully via `mise run build-wasm-release` with zero errors (OSS)
- **SC-EDGE-002**: OSS edge worker provides basic query capabilities with <200ms p95 latency for simple queries
- **SC-EDGE-003**: WASM bundle size under 10MB compressed for fast cold-start performance (OSS target)
- **SC-EDGE-004**: Commercial edge deployment serves requests with <50ms p95 latency globally from nearest Cloudflare POP
- **SC-EDGE-005**: Commercial edge workers handle 10k requests per second per geographic region without rate limiting
- **SC-EDGE-006**: Commercial global edge deployment achieves <100ms p95 latency from any major city worldwide

#### Provenance Tracking

- **SC-PROV-001**: Provenance query for any `GraphNode` returns source repository, commit ref, and ingestion timestamp within 100ms p95 (FR-014)

#### Semantic Search

- **SC-SEARCH-001**: Semantic similarity search achieves ≥70% precision and ≥70% recall on annotated benchmark set; top-10 results returned within 200ms p95 (FR-015)
- **SC-SEARCH-002**: When Vectorize/Qdrant is unavailable, semantic search MUST automatically degrade to AST-based search and return results within 200ms p95. Degraded responses include `"search_mode": "ast_semantic"`. No error is returned to the caller for this degraded mode.

#### Engine Pluggability

- **SC-ENGINE-001**: A new `CodeAnalyzer` implementation can be integrated by: (1) implementing the relevant trait, (2) registering in `FactoryRegistry`, (3) updating the configuration file, (4) recompiling — with zero modifications to `thread-flow` or `thread-services` orchestration code. Verified by integration test that adds a mock `CodeAnalyzer` implementation (FR-009).

#### Language-Agnostic Queries

- **SC-LANGQUERY-001**: `find_by_class(SemanticClass::DefinitionCallable)` returns semantically equivalent results across Rust (`function_item`), Python (`function_definition`), and Go (`function_declaration`) test fixtures with no language-specific query code at the call site. Verified by cross-language query integration tests in the T-C12 test suite (FR-LANGQUERY).

#### Audit Log

- **SC-AUDIT-001**: Conflict event log captures 100% of conflict predictions and status transitions; retained for ≥90 days; queryable by file, developer, and time range (FR-019) *(Commercial scope — deferred with thread-conflict)*

#### Observability

- **SC-OBS-001**: All pipeline crates (`thread-flow`, `thread-graph`, `thread-indexer`, `thread-api`, `thread-realtime`) emit structured log entries containing the required fields from FR-027 for: analysis start/completion, cache hits/misses, storage operation latency, and error conditions. Verified by integration test capturing log output and asserting schema compliance on a representative operation in each crate (FR-027).

#### Language Detection

- **SC-LANGDETECT-001**: Extensionless file language detection achieves ≥95% accuracy on a benchmark set of 500 representative files across major languages; false-language-assignment rate below 2% (FR-LANGDETECT)

#### Authentication

- **SC-AUTH-001**: CLI local-mode deployment MUST NOT require authentication (single-user, local network only). Multi-user service-mode deployment MUST authenticate via OAuth2 PKCE flow, support at minimum GitHub and Google as OIDC providers, issue tokens with configurable expiry (default 24 hours), and invalidate sessions on explicit logout. No unauthenticated requests accepted by the service-mode HTTP endpoints.

### FR Coverage Gaps *(documented for tracking)*

The following functional requirements currently have no associated success criterion.
SCs should be added during implementation planning when measurable targets can be defined:

- **FR-009** (pluggable engines) — SC-ENGINE-001 added below.
- **FR-020** (OAuth2/OIDC authentication): SC-AUTH-001 added below.
- **FR-LANGQUERY** (language-agnostic semantic queries via `SemanticClass`) — SC-LANGQUERY-001 added below.

Previously uncovered; SCs added in this review: FR-014 → SC-PROV-001, FR-015 → SC-SEARCH-001,
FR-019 → SC-AUDIT-001 *(deferred)*, FR-LANGDETECT → SC-LANGDETECT-001, FR-020 → SC-AUTH-001,
FR-009 → SC-ENGINE-001, FR-LANGQUERY → SC-LANGQUERY-001, FR-027 → SC-OBS-001.

## Assumptions

1. **Primary Languages**: Initial support focuses on Rust, TypeScript/JavaScript, Python, Go (Tier 1 languages from CLAUDE.md)
2. **Data Source Priority**: Git-based repositories are primary data source, with local file system and cloud storage as secondary
3. **Conflict Types**: Focus on code merge conflicts, API breaking changes, and concurrent edit detection - not runtime conflicts or logic bugs
4. **Authentication**: Multi-user deployments use standard OAuth2/OIDC for authentication, delegating to existing identity providers
5. **API & Real-Time Protocol**: Query API uses prost Protobuf over plain HTTP POST (no Connect-RPC/gRPC framing). Real-time update propagation uses WebSocket (CLI, full-duplex) and SSE (edge, server-push) transports via the `thread-realtime` crate. Cloudflare Durable Objects required for edge stateful operations (connection management, session state) — implemented in commercial crate. Polling fallback for restrictive networks.
6. **Graph Granularity**: Multi-level graph representation (file -> class/module -> function/method -> symbol) for flexibility
7. **Conflict Detection Strategy**: Multi-tier progressive approach using all available detection methods (AST diff, semantic analysis, graph impact analysis) with intelligent routing. Fast methods provide immediate feedback, slower methods refine accuracy. Results update in real-time as better information becomes available, balancing speed with precision.
8. **Conflict Resolution**: System provides predictions and suggestions only - final resolution decisions remain with developers
9. **Performance Baseline**: "Real-time" defined as <1 second query response for typical developer workflow interactions
10. **Scalability Target**: Initial target is codebases up to 500k files, 10M nodes - can scale higher with infrastructure investment
11. **Engine Pluggability**: Engines are swappable via compile-time composition only — not runtime plugin loading and not hot-swappable. The swap contract is: implement the relevant trait + register in `FactoryRegistry` + update config file + recompile. No orchestration code changes required. This is the intended workflow, not a limitation.
12. **Storage Strategy**: Multi-backend architecture with specialized purposes: Postgres (CLI primary, full ACID graph), D1 (edge primary, distributed graph), Vectorize (edge vector search), Qdrant (CLI-only vector search, optional). Content-addressed storage via ReCoco dataflow framework (per Constitution v2.0.0, Principle IV). ReCoco integration follows trait boundary pattern: Thread defines storage and dataflow interfaces, ReCoco provides implementations. This allows swapping ReCoco components or vendoring parts as needed.
13. **Deployment Model**: Single binary for both CLI and WASM with conditional compilation, not separate codebases. **Commercial Boundaries**: OSS includes core library with simple/limited WASM worker (Rust, Python, TypeScript). Full cloud deployment (comprehensive edge, managed service, advanced features) is commercial/paid. Architecture enables feature-flag-driven separation.
14. **Vendoring Strategy**: ReCoco components may be vendored (copied into Thread codebase) if cloud deployment requires customization or upstream changes conflict with Thread's stability requirements. Trait boundaries enable selective vendoring without architectural disruption. (Note: less critical now that ReCoco is Thread's own fork.)
15. **Component Selection Strategy**: Do NOT assume existing Thread components will be used. Evaluate ReCoco capabilities first, identify gaps, then decide whether to use existing components (ast-engine, language, rule-engine), adapt CodeWeaver semantic layer, or build new components. Prioritize best-fit over code reuse.
16. **Storage Consistency Model**: Replaced "Database Sync" with **Content-Addressed Storage (CAS)**.
    - **Source of Truth**: Git is the only SoT. DBs are derived indexes.
    - **Sync Strategy**: "Sync" is simply uploading/downloading immutable CAS chunks. No row-level merge logic required.
    - **Local-Only**: Postgres acts as the standalone CAS and State manager.
    - **Distributed**: D1 acts as the shared CAS; Real-Time Service manages ephemeral Deltas.
17. **Cross-Repository Consistency Model**: Cross-repository dependency links between separately indexed repositories use eventual consistency. During concurrent indexing of multiple repositories, cross-repo links may be briefly stale (pointing to an older version of the linked symbol) until the next incremental update cycle completes. Queries against stale cross-repo links return results with `cross_repo_stale: true` in the response envelope. This is acceptable — cross-repo links are updated opportunistically, not transactionally. *Informative target (not normative for OSS): cross-repo links are refreshed within 5 minutes of an incremental update cycle completing for the source repository.*
18. **Observability Model**: Structured logging (FR-027) is the primary observability mechanism. Edge deployment relies on Cloudflare Workers Logs with automated OTEL export — no manual distributed trace header propagation is required in the `thread-api` protocol. CLI deployment uses the `tracing` crate ecosystem. Metrics endpoints (FR-021) complement log-based observability but are not a substitute for it.

## Dependencies

1. **Constitutional Requirements**: Must comply with Thread Constitution v2.0.0, particularly:
   - Principle I: Service-Library Dual Architecture
   - Principle III: Test-First Development (TDD mandatory)
   - Principle VI: Service Architecture & Persistence
2. **ReCoco Framework (Rust-only fork)**: Foundational dependency for content-addressed caching, dataflow orchestration, and incremental ETL. Already integrated as the `recoco` crate in `thread-flow` (`recoco = { version = "0.2.1" }`). **Integration Strategy**: ReCoco is wrapped behind Thread-owned traits (following the ast-grep integration pattern) to maintain architectural flexibility, enable component swapping, and support potential vendoring for cloud deployment. ReCoco types must not leak into Thread's public APIs.
3. **AST & Semantic Analysis Components**: Existing Thread crates (`thread-ast-engine`, `thread-language`, `thread-rule-engine`) are vendored from ast-grep and NOT guaranteed to be used. Alternative options include CodeWeaver's semantic characterization layer (currently Python, portable to Rust) which may provide superior semantic analysis. Component selection deferred pending ReCoco capability assessment.
4. **thread-definitions** (new crate): Semantic classification engine for AST node types. Provides `SemanticClass` (22 variants), `ImportanceRank` (5 tiers), `ImportanceScores`, `AgentTask` scoring with `ContextualAdjustments`. Pre-baked data from `classifications/` directory (27 languages validated, 5,899 items classified). Enables L1 definition extraction via `classify_node_types` operator in `thread-flow`. Zero overlap with `thread-ast-engine` (confirmed). **Canonical implementation spec**: [`docs/architecture/SEMANTIC_CLASSIFICATION_SPEC.md`](../../docs/architecture/SEMANTIC_CLASSIFICATION_SPEC.md) — authoritative reference for classifier internals, data schemas, lookup pipeline, and scoring model.
5. **Storage Backends**: Integration with Postgres (local), D1 (edge), Vectorize (edge vectors, primary), Qdrant (CLI-only, optional) as defined in CLAUDE.md architecture
6. **Tree-sitter**: Underlying parser infrastructure for AST generation across multiple languages
7. **Concurrency Models**: Rayon for CLI parallelism, tokio for edge async I/O
8. **WASM Toolchain**: `xtask` build system for WASM compilation to Cloudflare Workers target
9. **API Protocol**: `prost` runtime for Protobuf message encoding/decoding (no_std compatible, compiles to `wasm32-unknown-unknown`); `prost-build` as host-only code generation tool (never in WASM binary). TypeScript client codegen via `buf` CLI + `@bufbuild/protobuf` (protobuf-es v2). Transport: plain HTTP POST — no Connect-RPC or gRPC framing (Workers lack HTTP/2 trailer support). Internal Rust-to-Rust communication uses `postcard` (already in workspace). MCP server integration (future): `serde_json`/JSON-RPC 2.0 as separate transport adapter; prost types can optionally derive serde for MCP JSON output.
10. **Network Protocol**: Cloudflare Durable Objects required for edge stateful operations (connection management, session persistence, collaborative state). HTTP REST fallback if RPC proves infeasible.
11. **CodeWeaver Integration** (Optional): CodeWeaver's semantic characterization layer (sister project, currently Python) provides sophisticated code analysis capabilities. May port to Rust if superior to ast-grep-derived components. Evaluation pending ReCoco capability assessment.
12. **Graph Database**: Requires graph query capabilities - may need additional graph storage layer beyond relational DBs
13. **Semantic Analysis**: May require ML/embedding models for semantic similarity search (e.g., code2vec, CodeBERT). CodeWeaver may provide this capability.

## Commercial Boundary

Thread follows a strict one-directional dependency rule: **commercial/private crates depend on Thread public crates; Thread public crates NEVER depend on commercial crates**. The `crates/cloudflare/` directory provides a local development convenience (gitignored, separate private repo) but represents a genuinely separate project — commercial deployment and implementation built atop Thread's public capabilities.

### Component Classification

| Component | Classification | Notes |
|-----------|---------------|-------|
| `thread-graph`, `thread-indexer` | OSS | Core graph intelligence crates, no deployment dependency |
| `thread-conflict` | **Commercial/TBD** | Conflict detection is a proprietary differentiator; deferred to dedicated commercial design phase. Phase 4 tasks (T029–T038) in tasks.md are out of OSS scope. |
| `thread-definitions` | OSS | Pure classification library, zero cloud dependency |
| `thread-storage` (Postgres + D1 + Vectorize backends) | OSS | All three follow the D1 model — library backends, user-provided credentials |
| `thread-api` (RPC types, Protobuf definitions) | OSS | Protocol definitions required for CLI and third-party clients |
| `thread-realtime` (WebSocket + SSE transports) | OSS | Standard transports; exposes `RealtimeBackend` trait |
| Durable Objects backend for `thread-realtime` | **Private** | Cloudflare-specific; implements `RealtimeBackend` in commercial crate |
| Multi-worker Cloudflare Workers architecture (FR-010 full) | **Private** | Commercial deployment; OSS distribution includes simplified single-worker only |
| OSS simplified WASM worker | OSS | Single-worker deployment (SC-EDGE-001 through SC-EDGE-003) |
| MCP server / AI tool interface | TBD | Requires dedicated design; likely OSS core with potential commercial enhancements |
| Wrangler configurations, Worker entry points | **Private** | Deployment machinery in private repo |
| R2 offload, Workers AI integrations | **Private** | Cloudflare proprietary services; commercial crate only |

### OSS → Commercial Upgrade Path

The OSS → commercial upgrade is zero-re-index. The D1 schema is forward-compatible:

1. Deploy the commercial Worker pointing at the existing OSS D1 database — no schema migration needed.
2. Add Durable Objects binding for `thread-realtime` (`RealtimeBackend` implementation).
3. Add Vectorize index binding (if upgrading from OSS without vector search).
4. Swap the OSS single Worker binary for the commercial Router Worker.
5. WebSocket connections are interrupted during the Worker swap (standard Cloudflare deployment behavior) but reconnect automatically.

**No re-indexing is required.** All previously analyzed graph data in D1 is immediately available to the commercial Worker. The OSS D1 schema is a strict subset of the commercial schema — commercial Wrangler migrations are additive only.

### Task Annotations

Tasks in tasks.md that produce artifacts destined for the private commercial crate are annotated `[CF: private]`. Tasks that touch the boundary (OSS component + private integration) are annotated `[CF: boundary]`.

### FR-010 Boundary Note

The OSS distribution implements SC-EDGE-001 through SC-EDGE-003 (single-worker, core languages). SC-EDGE-004 through SC-EDGE-006 (multi-worker, global distribution, 10k req/s) are commercial deployment targets implemented in the private crate using Thread public crates as dependencies.

## Clarifications

### Session 2026-01-11

- Q: What is ReCoco's architectural role in the real-time code graph system? -> A: ReCoco provides both storage abstraction AND dataflow orchestration for the entire analysis pipeline, but must be integrated through strong trait boundaries (similar to ast-grep integration pattern) to enable swappability and potential vendoring for cloud deployment. ReCoco serves as "pipes" infrastructure, not a tightly-coupled dependency.
- Q: How do the three storage backends (Postgres, D1, Qdrant) relate to each other architecturally? -> A: Specialized backends with deployment-specific primaries - Postgres for CLI graph storage, D1 for edge deployment graph storage, Vectorize for edge semantic search, Qdrant (CLI-only, optional) for local semantic search. Each serves a distinct purpose rather than being alternatives or replicas.
- Q: What protocol propagates real-time code changes to connected clients? -> A: Deployment-specific protocols (SSE for edge stateless operations, WebSocket for CLI stateful operations) with expectation that Cloudflare Durable Objects will be required for some edge stateful functions. Protocol choice remains flexible (WebSocket, SSE, Custom RPC all candidates) pending implementation constraints.
- Q: How does the system detect potential merge conflicts between concurrent code changes? -> A: Multi-tier progressive detection system using all available methods (AST diff, semantic analysis, graph impact analysis) with intelligent routing. Prioritizes speed (fast AST diff for initial detection) then falls back to slower methods for accuracy. Results update progressively as more accurate analysis completes, delivering fast feedback that improves over time.
- Q: What API interface do developers use to query the code graph? -> A: Custom RPC over HTTP for unified protocol across CLI and edge deployments (single API surface, built-in streaming, type safety). If RPC proves infeasible, fallback to HTTP REST API for both deployments. Priority is maintaining single API surface rather than deployment-specific optimizations.
- Q: Should we assume existing Thread crates (ast-engine, language, rule-engine) will be used, or evaluate alternatives? -> A: Do NOT assume existing Thread components will be used. These are vendored from ast-grep and may not be optimal. Approach: (1) Evaluate what capabilities ReCoco provides, (2) Identify gaps, (3) Decide what to build/adapt. Consider CodeWeaver's semantic characterization layer (Python, portable to Rust) as alternative to existing semantic analysis.
- Q: How do we maintain commercial boundaries between open-source and paid cloud service? -> A: Carefully defined boundaries: OSS library includes core graph analysis with simple/limited WASM worker for edge. Full cloud deployment (comprehensive edge, managed service, advanced features) is commercial/paid service. Architecture must enable this split through feature flags and deployment configurations.

## Open Questions

None - all critical items have been addressed with reasonable defaults documented in Assumptions section.

## Notes

- This feature represents a significant architectural addition to Thread, evolving it from a code analysis library to a real-time intelligence platform
- The service-library dual architecture aligns with Constitutional Principle I and requires careful API design for both library consumers and service deployments
- Content-addressed caching and incremental updates are constitutional requirements (Principle VI) and must achieve >90% cache hit rates and <10% incremental analysis time
- Conflict prediction is the highest-value differentiator and should be prioritized for early validation with real development teams
- Edge deployment to Cloudflare Workers enables global low-latency access but requires careful WASM optimization and may limit available crates/features
- Consider phased rollout: P1 (graph queries) -> P2 (conflict prediction) -> P3 (multi-source) -> P4 (AI resolution) to validate core value proposition early
- **Commercial Architecture**: OSS/commercial boundaries must be designed from day one. OSS provides core library value (CLI + basic edge), commercial provides managed cloud service with advanced features. Architecture uses feature flags and conditional compilation to enable clean separation while maintaining single codebase.
- **Component Evaluation Strategy**: Do NOT assume existing Thread components will be reused. First evaluate ReCoco capabilities comprehensively, then identify gaps, then decide on AST/semantic analysis components. CodeWeaver's semantic layer is a viable alternative to Thread's ast-grep-derived components.
- **MCP Server**: Will be implemented as a separate `thread-mcp` crate using `rmcp-actix-web` (<https://gitlab.com/lx-industries/rmcp-actix-web>). Tool design, tier structure, and OSS/commercial scope require dedicated design work. **Structural requirement**: All types crossing the `thread-api` → MCP boundary must derive `serde::Serialize`/`serde::Deserialize`. `prost`-generated types can satisfy this via serde feature flags or manual derives — verify compatibility before T017 finalizes proto definitions. MCP will NOT be hand-rolled; `thread-mcp` depends on `thread-api` but `thread-api` has no dependency on `thread-mcp`.

## Implementation Status

**As of 2026-02-24**, the following components have been implemented in the `thread-flow` crate:

### Implemented ✅

- **Content-Addressed Storage** (FR-004, FR-012): Blake3 fingerprinting via ReCoco, StorageBackend trait with Postgres and D1 backends
- **Incremental Updates** (FR-008): IncrementalAnalyzer, DependencyGraph with BFS invalidation and topological sort
- **Language Extractors**: Rust, TypeScript, Python, Go dependency extraction
- **ReCoco Integration (Scaffolded ⚠️)**: bridge.rs contains stub implementations only — all methods return empty results with TODO comments. ThreadFlowBuilder DSL and CocoIndex operator structure are in place. bridge.rs must be fully implemented before T-C10 (classify operator integration).
- **CLI Deployment** (FR-011): Postgres backend fully operational
- **Edge Storage** (FR-010 partial): D1 backend implemented

### Not Yet Started ❌

- **Semantic classification** (FR-CLASSIFY): `SemanticClass`, `ImportanceScores` — `thread-definitions` crate (T-C01 through T-C10)
- **Semantic Graph** (FR-002 partial): Rich GraphNode/GraphEdge model not built; current implementation has minimal DependencyEdge
- **Real-Time Queries** (FR-005): No query API layer yet
- **Conflict Detection** (FR-006, FR-007): Three-tier conflict detection system not started
- **Multi-Source Indexing** (FR-003): Git, S3 sources not implemented
- **Graph API** (FR-016): No API layer yet — prost-generated Protobuf types and HTTP POST handlers not implemented
- **Overlay Graph** (FR-017): Not implemented
- **Semantic/Vector Search** (FR-015): Vectorize integration pending; Qdrant blocked by dependency conflict

### Blocked ⚠️

- **Qdrant vector search**: `recoco/target-qdrant` disabled due to CRC version conflict. **Resolution**: Use Cloudflare Vectorize for edge deployment.
