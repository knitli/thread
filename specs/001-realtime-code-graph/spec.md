# Feature Specification: Real-Time Code Graph Intelligence

**Feature Branch**: `001-realtime-code-graph`
**Created**: 2026-01-10
**Status**: Draft
**Input**: User description: "Build an application that can provide performant, real-time, code-base-wide graph intelligence with semantic/ast awareness. I want it to be able to interface with any data source, database target, work locally and in the cloud, and plug and change out underlying engines. It needs to be fast and cloudflare deployable. This will server as the foundational intelligence layer for the future of work -- enabling real time and asynchronous human-ai teaming with intelligent conflict prediction and resolution"

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

1. **Given** two developers editing different files, **When** their changes affect the same function call chain, **Then** system detects potential conflict and notifies both developers within 5 seconds of the conflicting change
2. **Given** a developer modifying a widely-used API, **When** the change would break 15 downstream callers, **Then** system lists all affected callers with severity ratings before commit
3. **Given** asynchronous work across timezones, **When** developer A's changes conflict with developer B's 8-hour-old WIP branch, **Then** system provides merge preview showing exactly what will conflict

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
3. **Given** breaking API change conflict, **When** system analyzes impact, **Then** it suggests adapter pattern or migration path with code examples

---

### Edge Cases

- What happens when indexing a codebase larger than available memory (1M+ files)?
- How does the system handle circular dependencies in the code graph?
- What occurs when two data sources contain the same file with different versions?
- How does conflict prediction work when one developer is offline for extended periods?
- What happens if the underlying analysis engine crashes mid-query?
- How does the system handle generated code files that change frequently?
- What occurs when database connection is lost during real-time updates?
- How does the system manage version drift between local and cloud deployments?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST parse and analyze source code to build AST (Abstract Syntax Tree) representations for all supported languages
- **FR-002**: System MUST construct a graph representation of codebase relationships including: function calls, type dependencies, data flows, and import/export chains
- **FR-003**: System MUST index code from configurable data sources including: local file systems, Git repositories (GitHub, GitLab, Bitbucket), and cloud storage (S3-compatible)
- **FR-004**: System MUST store analysis results in specialized database backends with deployment-specific primaries: Postgres (CLI deployment primary for full graph with ACID guarantees), D1 (edge deployment primary for distributed graph storage), and Qdrant (semantic search backend for vector embeddings, used across both deployments)
- **FR-005**: System MUST support real-time graph queries responding within 1 second for codebases up to 100k files
- **FR-006**: System MUST detect when concurrent code changes create potential conflicts in: shared function call chains, modified API signatures, or overlapping data structures. Detection uses multi-tier progressive strategy: fast AST diff for initial detection (<100ms), semantic analysis for accuracy refinement (<1s), graph impact analysis for comprehensive validation (<5s). Results update progressively as each tier completes.
- **FR-007**: System MUST provide conflict predictions with specific details: file locations, conflicting symbols, impact severity ratings, and confidence scores. Initial predictions (AST-based) deliver within 100ms, refined predictions (semantic-validated) within 1 second, comprehensive predictions (graph-validated) within 5 seconds.
- **FR-008**: System MUST support incremental updates where only changed files and affected dependencies are re-analyzed
- **FR-009**: System MUST allow pluggable analysis engines where the underlying AST parser, graph builder, or conflict detector can be swapped without rewriting application code
- **FR-010**: System MUST deploy to Cloudflare Workers as a WASM binary for edge computing scenarios. **OSS Boundary**: OSS library includes simple/limited WASM worker with core query capabilities. **Constraint**: Edge deployment MUST NOT load full graph into memory. Must use streaming/iterator access patterns and D1 Reachability Index.
- **FR-011**: System MUST run as a local CLI application for developer workstation use (available in OSS)
- **FR-012**: System MUST use content-addressed caching to avoid re-analyzing identical code sections across updates
- **FR-013**: System MUST propagate code changes to all connected clients within 100ms of detection for real-time collaboration
- **FR-014**: System MUST track analysis provenance showing which data source, version, and timestamp each graph node originated from
- **FR-015**: System MUST support semantic search across the codebase to find similar functions, related types, and usage patterns
- **FR-016**: System MUST provide graph traversal APIs via Custom RPC over HTTP protocol for: dependency walking, reverse lookups (who calls this), and path finding between symbols. This provides unified interface for CLI and edge deployments with built-in streaming and type safety.
- **FR-017**: System MUST maintain graph consistency when code is added, modified, or deleted during active queries
- **FR-018**: System MUST log all conflict predictions and resolutions for audit and learning purposes
- **FR-019**: System MUST handle authentication and authorization for multi-user scenarios when deployed as a service
- **FR-020**: System MUST expose metrics for: query performance, cache hit rates, indexing throughput, and storage utilization
- **FR-021**: System MUST utilize batched database operations (D1 Batch API) and strictly govern memory usage (<80MB active set) on Edge via CocoIndex adaptive controls to prevent OOM errors.

### Key Entities

- **Code Repository**: Represents a source of code (Git repo, local directory, cloud storage). Attributes: source type, connection credentials, sync frequency, last sync timestamp
- **Code File**: Individual file in a repository. Attributes: file path, language, content hash, AST representation, last modified timestamp
- **Graph Node**: Represents a code symbol (function, class, variable, type). Attributes: symbol name, type, location (file + line), semantic metadata, relationships to other nodes
- **Graph Edge**: Represents a relationship between nodes. Attributes: relationship type (calls, imports, inherits, uses), direction, strength/confidence score
- **Conflict Prediction**: Represents a detected potential conflict. Attributes: affected files, conflicting developers, conflict type, severity, suggested resolution, timestamp
- **Analysis Session**: Represents a single analysis run. Attributes: start time, completion time, files analyzed, nodes/edges created, cache hit rate
- **Analysis Engine**: Represents a pluggable component. Attributes: engine type (parser, graph builder, conflict detector), version, configuration parameters

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Developers can query code dependencies and receive complete results in under 1 second for codebases up to 100,000 files
- **SC-002**: System detects 95% of potential merge conflicts before code is committed, with false positive rate below 10%
- **SC-003**: Incremental indexing completes in under 10% of full analysis time for typical code changes (affecting <5% of files)
- **SC-004**: System handles 1000 concurrent users querying simultaneously with <2 second p95 response time
- **SC-005**: Conflict resolution time reduces by 70% (from 30 minutes to under 10 minutes) when using AI-assisted suggestions
- **SC-006**: Cross-repository dependency tracking works across 5+ different code sources without manual configuration
- **SC-007**: Developer satisfaction score of 4.5/5 for "confidence in making code changes" after using conflict prediction
- **SC-008**: 90% of developers successfully integrate the system into their workflow within first week of adoption
- **SC-009**: Real-time collaboration features reduce integration delays from hours to minutes (75% improvement)
- **SC-010**: System operates with 99.9% uptime when deployed to Cloudflare edge network

### Service Architecture Success Criteria

**Deployment Targets**: Both CLI and Edge

#### Cache Performance

- **SC-CACHE-001**: Content-addressed cache achieves >90% hit rate for repeated analysis of unchanged code sections
- **SC-CACHE-002**: Cache invalidation occurs within 100ms of source code change detection
- **SC-CACHE-003**: Cache size remains under 500MB for 10k file repository, scaling linearly with codebase size
- **SC-CACHE-004**: Cache warmup completes in under 5 minutes for new deployment with existing persistent storage

#### Incremental Updates

- **SC-INCR-001**: Code changes trigger only affected component re-analysis, not full codebase scan
- **SC-INCR-002**: Incremental update completes in <10% of full analysis time for changes affecting <5% of files
- **SC-INCR-003**: Dependency graph updates propagate to all connected clients in <100ms
- **SC-INCR-004**: Change detection accurately identifies affected files with 99% precision (no missed dependencies)

#### Storage Performance

- **SC-STORE-001**: Database operations meet constitutional targets:
  - Postgres (CLI): <10ms p95 latency for graph traversal queries
  - D1 (Edge): <50ms p95 latency for distributed edge queries
  - Qdrant (vectors): <100ms p95 latency for semantic similarity search
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

## Assumptions

1. **Primary Languages**: Initial support focuses on Rust, TypeScript/JavaScript, Python, Go (Tier 1 languages from CLAUDE.md)
2. **Data Source Priority**: Git-based repositories are primary data source, with local file system and cloud storage as secondary
3. **Conflict Types**: Focus on code merge conflicts, API breaking changes, and concurrent edit detection - not runtime conflicts or logic bugs
4. **Authentication**: Multi-user deployments use standard OAuth2/OIDC for authentication, delegating to existing identity providers
5. **Real-Time Protocol**: Custom RPC over HTTP streaming for real-time updates (unified with query API), with WebSocket/SSE as fallback options. RPC server-side streaming provides efficient real-time propagation for both CLI and edge deployments. Cloudflare Durable Objects expected for edge stateful operations (connection management, session state). Polling fallback for restrictive networks.
6. **Graph Granularity**: Multi-level graph representation (file → class/module → function/method → symbol) for flexibility
7. **Conflict Detection Strategy**: Multi-tier progressive approach using all available detection methods (AST diff, semantic analysis, graph impact analysis) with intelligent routing. Fast methods provide immediate feedback, slower methods refine accuracy. Results update in real-time as better information becomes available, balancing speed with precision.
8. **Conflict Resolution**: System provides predictions and suggestions only - final resolution decisions remain with developers
9. **Performance Baseline**: "Real-time" defined as <1 second query response for typical developer workflow interactions
10. **Scalability Target**: Initial target is codebases up to 500k files, 10M nodes - can scale higher with infrastructure investment
11. **Engine Architecture**: Engines are swappable via well-defined interfaces, not runtime plugin loading (compile-time composition)
12. **Storage Strategy**: Multi-backend architecture with specialized purposes: Postgres (CLI primary, full ACID graph), D1 (edge primary, distributed graph), Qdrant (semantic search, both deployments). Content-addressed storage via CocoIndex dataflow framework (per Constitution v2.0.0, Principle IV). CocoIndex integration follows trait boundary pattern: Thread defines storage and dataflow interfaces, CocoIndex provides implementations. This allows swapping CocoIndex components or vendoring parts as needed.
13. **Deployment Model**: Single binary for both CLI and WASM with conditional compilation, not separate codebases. **Commercial Boundaries**: OSS includes core library with simple/limited WASM worker. Full cloud deployment (comprehensive edge, managed service, advanced features) is commercial/paid. Architecture enables feature-flag-driven separation.
14. **Vendoring Strategy**: CocoIndex components may be vendored (copied into Thread codebase) if cloud deployment requires customization or upstream changes conflict with Thread's stability requirements. Trait boundaries enable selective vendoring without architectural disruption.
15. **Component Selection Strategy**: Do NOT assume existing Thread crates will be used. Evaluate CocoIndex capabilities first, identify gaps, then decide whether to use existing components (ast-engine, language, rule-engine), adapt CodeWeaver semantic layer, or build new components. Prioritize best-fit over code reuse.

## Dependencies

1. **Constitutional Requirements**: Must comply with Thread Constitution v2.0.0, particularly:
   - Principle I: Service-Library Dual Architecture
   - Principle III: Test-First Development (TDD mandatory)
   - Principle VI: Service Architecture & Persistence
2. **CocoIndex Framework**: Foundational dependency for content-addressed caching, dataflow orchestration, and incremental ETL. **Integration Strategy**: CocoIndex must be wrapped behind Thread-owned traits (following the ast-grep integration pattern) to maintain architectural flexibility, enable component swapping, and support potential vendoring for cloud deployment. CocoIndex types must not leak into Thread's public APIs. **Evaluation Priority**: Assess CocoIndex capabilities first, then determine what additional components are needed.
3. **AST & Semantic Analysis Components**: Existing Thread crates (`thread-ast-engine`, `thread-language`, `thread-rule-engine`) are vendored from ast-grep and NOT guaranteed to be used. Alternative options include CodeWeaver's semantic characterization layer (currently Python, portable to Rust) which may provide superior semantic analysis. Component selection deferred pending CocoIndex capability assessment.
4. **Storage Backends**: Integration with Postgres (local), D1 (edge), Qdrant (vectors) as defined in CLAUDE.md architecture
5. **Tree-sitter**: Underlying parser infrastructure for AST generation across multiple languages
6. **Concurrency Models**: Rayon for CLI parallelism, tokio for edge async I/O
7. **WASM Toolchain**: `xtask` build system for WASM compilation to Cloudflare Workers target
8. **gRPC Framework**: Primary API protocol dependency (likely tonic for Rust). Provides unified interface for queries and real-time updates across CLI and edge deployments with type safety and streaming. Must compile to WASM for Cloudflare Workers deployment.
9. **Network Protocol**: Cloudflare Durable Objects required for edge stateful operations (connection management, session persistence, collaborative state). HTTP REST fallback if RPC proves infeasible.
10. **CodeWeaver Integration** (Optional): CodeWeaver's semantic characterization layer (sister project, currently Python) provides sophisticated code analysis capabilities. May port to Rust if superior to ast-grep-derived components. Evaluation pending CocoIndex capability assessment.
11. **Graph Database**: Requires graph query capabilities - may need additional graph storage layer beyond relational DBs
12. **Semantic Analysis**: May require ML/embedding models for semantic similarity search (e.g., code2vec, CodeBERT). CodeWeaver may provide this capability.

## Clarifications

### Session 2026-01-11

- Q: What is CocoIndex's architectural role in the real-time code graph system? → A: CocoIndex provides both storage abstraction AND dataflow orchestration for the entire analysis pipeline, but must be integrated through strong trait boundaries (similar to ast-grep integration pattern) to enable swappability and potential vendoring for cloud deployment. CocoIndex serves as "pipes" infrastructure, not a tightly-coupled dependency.
- Q: How do the three storage backends (Postgres, D1, Qdrant) relate to each other architecturally? → A: Specialized backends with deployment-specific primaries - Postgres for CLI graph storage, D1 for edge deployment graph storage, Qdrant for semantic search across both deployments. Each serves a distinct purpose rather than being alternatives or replicas.
- Q: What protocol propagates real-time code changes to connected clients? → A: Deployment-specific protocols (SSE for edge stateless operations, WebSocket for CLI stateful operations) with expectation that Cloudflare Durable Objects will be required for some edge stateful functions. Protocol choice remains flexible (WebSocket, SSE, Custom RPC all candidates) pending implementation constraints.
- Q: How does the system detect potential merge conflicts between concurrent code changes? → A: Multi-tier progressive detection system using all available methods (AST diff, semantic analysis, graph impact analysis) with intelligent routing. Prioritizes speed (fast AST diff for initial detection) then falls back to slower methods for accuracy. Results update progressively as more accurate analysis completes, delivering fast feedback that improves over time.
- Q: What API interface do developers use to query the code graph? → A: Custom RPC over HTTP for unified protocol across CLI and edge deployments (single API surface, built-in streaming, type safety). If RPC proves infeasible, fallback to HTTP REST API for both deployments. Priority is maintaining single API surface rather than deployment-specific optimizations.
- Q: Should we assume existing Thread crates (ast-engine, language, rule-engine) will be used, or evaluate alternatives? → A: Do NOT assume existing Thread components will be used. These are vendored from ast-grep and may not be optimal. Approach: (1) Evaluate what capabilities CocoIndex provides, (2) Identify gaps, (3) Decide what to build/adapt. Consider CodeWeaver's semantic characterization layer (Python, portable to Rust) as alternative to existing semantic analysis.
- Q: How do we maintain commercial boundaries between open-source and paid cloud service? → A: Carefully defined boundaries: OSS library includes core graph analysis with simple/limited WASM worker for edge. Full cloud deployment (comprehensive edge, managed service, advanced features) is commercial/paid service. Architecture must enable this split through feature flags and deployment configurations.

## Open Questions

None - all critical items have been addressed with reasonable defaults documented in Assumptions section.

## Notes

- This feature represents a significant architectural addition to Thread, evolving it from a code analysis library to a real-time intelligence platform
- The service-library dual architecture aligns with Constitutional Principle I and requires careful API design for both library consumers and service deployments
- Content-addressed caching and incremental updates are constitutional requirements (Principle VI) and must achieve >90% cache hit rates and <10% incremental analysis time
- Conflict prediction is the highest-value differentiator and should be prioritized for early validation with real development teams
- Edge deployment to Cloudflare Workers enables global low-latency access but requires careful WASM optimization and may limit available crates/features
- Consider phased rollout: P1 (graph queries) → P2 (conflict prediction) → P3 (multi-source) → P4 (AI resolution) to validate core value proposition early
- **Commercial Architecture**: OSS/commercial boundaries must be designed from day one. OSS provides core library value (CLI + basic edge), commercial provides managed cloud service with advanced features. Architecture uses feature flags and conditional compilation to enable clean separation while maintaining single codebase.
- **Component Evaluation Strategy**: Do NOT assume existing Thread components will be reused. First evaluate CocoIndex capabilities comprehensively, then identify gaps, then decide on AST/semantic analysis components. CodeWeaver's semantic layer is a viable alternative to Thread's ast-grep-derived components.
