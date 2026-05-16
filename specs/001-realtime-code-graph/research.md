<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: AGPL-3.0-or-later
-->

# Research Findings: Real-Time Code Graph Intelligence

**Feature Branch**: `001-realtime-code-graph`
**Research Phase**: Phase 0
**Status**: Complete (revised 2026-02-24)
**Last Updated**: 2026-02-24

## Purpose

This document resolves all "NEEDS CLARIFICATION" and "PENDING RESEARCH" items identified during Technical Context and Constitution Check evaluation. Each research task investigates technical unknowns, evaluates alternatives, and makes architectural decisions with clear rationale.

## Research Tasks

### 1. Recoco Integration Architecture

**Status**: ✅ Complete — **SUPERSEDED AND IMPLEMENTED**

**Original Research Output** (2026-01-11): Recommended a trait abstraction layer because
"CocoIndex Has No Native Rust API Yet" (Issue #1372 was open).

**Current Reality** (2026-02-24): This is no longer accurate.

**Recoco** is Thread's own Rust-only public fork of CocoIndex, maintained as a separate
open-source crate. It IS the native Rust API. The integration is complete.

**Actual Implementation** (in `crates/flow/`):

1. **Bridge Pattern** (`src/bridge.rs`, 85 lines):
   - `CocoIndexAnalyzer` implements `thread-services::CodeAnalyzer` trait
   - Adapts Thread's imperative logic to Recoco's declarative dataflow

2. **ThreadFlowBuilder DSL** (`src/flows/builder.rs`, 150+ lines):
   - Chainable API: `.source_local() → .parse() → .extract_symbols() → .target_postgres()`
   - Composes Recoco operators declaratively

3. **Custom Operators** (`src/functions/`):
   - `thread_parse`: Source file → AST
   - `thread_symbols`: AST → symbol definitions
   - `thread_imports`: AST → import/dependency statements
   - `thread_calls`: AST → function call relationships

4. **Feature Gating** (type leakage prevention):
   ```toml
   recoco-minimal = ["recoco/source-local-file"]    # Default
   recoco-postgres = ["recoco-minimal", "recoco/target-postgres"]
   # Note: recoco/target-qdrant disabled due to CRC dependency conflict
   ```

5. **Development Control**: Since Recoco is Thread's own fork, any required changes
   can be implemented directly without waiting on external maintainers.

**Validation Criteria** (all met):

- ✅ Zero Recoco types in Thread public APIs
- ✅ All dataflow operations testable without external Python dependencies
- ✅ `cargo build --workspace` succeeds
- ✅ `thread-flow` compiles to WASM for edge deployment

**Original alternatives** (still accurately rejected):
- ❌ Direct Python Subprocess Integration: High overhead
- ❌ PyO3 Embed Python Interpreter: Massive binary size, edge incompatible
- ❌ Wait for CocoIndex Rust API: We built our own (Recoco)

---

### 2. Component Selection: ast-grep vs CodeWeaver

**Status**: ✅ Complete

**Research Output**:

**Decision**: Use Existing Thread Components (ast-grep-derived) with Potential CodeWeaver Integration for Semantic Layer

**Rationale**:
1. **Existing Thread Infrastructure**: Thread already has `thread-ast-engine`, `thread-language`, `thread-rule-engine` vendored from ast-grep, tested and integrated. These provide solid AST parsing foundation.

2. **CodeWeaver Evaluation**: CodeWeaver is sister project (currently Python) with sophisticated semantic characterization layer. Spec mentions it as "optional integration" pending Rust portability assessment.

3. **Pragmatic Approach**: Start with existing ast-grep components for MVP (proven, integrated), evaluate CodeWeaver semantic layer as Phase 2 enhancement for improved conflict detection accuracy.

4. **Alignment with Spec**: Spec Dependency 3 states "Existing Thread crates NOT guaranteed to be used" but provides "evaluation priority" guidance. CocoIndex evaluation comes first, then determine semantic layer needs.

**Alternatives Considered**:
- ✅ **Use Existing ast-grep Components**: Proven, integrated, supports 20+ languages (Tier 1-3 from CLAUDE.md), fast AST parsing
- ⚠️ **Port CodeWeaver to Rust**: High effort, unknown timeline, Python→Rust portability unproven, defer until semantic analysis requirements are clearer
- ❌ **Build Custom Semantic Layer**: Reinventing wheel, violates "don't rebuild what exists" principle

**Migration Plan**:

**Phase 1 (MVP)**: Existing ast-grep components
- Use `thread-ast-engine` for AST parsing
- Use `thread-language` for multi-language support
- Use `thread-rule-engine` for pattern-based conflict detection (Tier 1: AST diff)

**Phase 2 (Semantic Enhancement)**: Evaluate CodeWeaver integration
- Assess CodeWeaver's semantic characterization capabilities
- Determine Rust portability (Python→Rust)
- If viable, integrate for Tier 2 semantic analysis (conflict detection accuracy refinement)

**Phase 3 (Production Optimization)**: Refine based on metrics
- If CodeWeaver proves superior for semantic analysis, expand integration
- If ast-grep components sufficient, optimize existing implementation
- Decision driven by conflict detection accuracy metrics (95% target, <10% false positive from SC-002)

---

### 3. API Protocol: gRPC vs HTTP REST

**Status**: ✅ Complete

**Research Output**:

**Decision**: Hybrid Protocol Strategy - Custom RPC over HTTP/WebSockets (not gRPC)

**Rationale**:

gRPC via tonic is NOT viable for Cloudflare Workers due to fundamental platform limitations:

1. **HTTP/2 Streaming Unsupported**: Cloudflare Workers runtime does not support HTTP/2 streaming semantics required by gRPC (confirmed via GitHub workerd#4534)
2. **WASM Compilation Blocker**: tonic server relies on tokio runtime features incompatible with `wasm32-unknown-unknown` target
3. **Bundle Size Concerns**: tonic + dependencies would yield 5-10MB uncompressed, approaching the 10MB Worker limit before adding application logic

Instead, leverage Cloudflare Workers' actual capabilities:
- **HTTP Fetch API**: Request/response via workers-rs
- **WebSockets**: Real-time bidirectional streaming (supported natively)
- **Shared Rust Types**: Compile-time type safety without gRPC overhead

**Alternatives Considered**:
- ❌ **tonic (gRPC)**: Does NOT compile to WASM server-side, Workers platform incompatible, 5-10MB bundle size
- ❌ **grpc-web**: Client-side only (tonic-web-wasm-client), still requires HTTP/2 backend, doesn't solve server-side WASM problem
- ⚠️ **tarpc / Cap'n Proto**: No confirmed WASM compatibility, unclear Workers support, unproven for this use case
- ⚠️ **ultimo-rs**: Requires tokio "full" features (incompatible with wasm32-unknown-unknown), works for CLI only
- ✅ **Custom RPC over HTTP + WebSockets (RECOMMENDED)**: Full WASM compatibility via workers-rs, type safety through shared Rust types, binary efficiency via serde + postcard (~40% size reduction), real-time streaming via WebSockets, ~3-4MB optimized bundle size
- ✅ **HTTP REST (Fallback)**: Simplest implementation, minimal dependencies, JSON debugging, but no streaming and larger payloads

**WASM Compatibility**:

**Cloudflare Workers Platform Constraints:**
- **Target**: `wasm32-unknown-unknown` (NOT `wasm32-wasi`)
- **Runtime**: V8 isolates, no TCP sockets, Fetch API only
- **Bundle Limits**: Free tier 1MB compressed, Paid tier 10MB compressed
- **HTTP**: No HTTP/2 streaming, no raw socket access
- **Concurrency**: Single-threaded (no `tokio::spawn` for multi-threading)

**Confirmed Working Pattern**:
```rust
use worker::*;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .post_async("/rpc", |mut req, _ctx| async move {
            let input: MyInput = req.json().await?;
            let output = handle_rpc(input).await?;
            Response::from_json(&output)
        })
        .run(req, env).await
}

// WebSockets for streaming
app.get("/ws", |req, ctx| async move {
    let pair = WebSocketPair::new()?;
    pair.server.accept()?;
    Response::ok("")?.websocket(pair.client)
})
```

**Bundle Size Analysis (Edge Deployment)**:
- workers-rs runtime: 800KB → 250KB compressed
- serde + postcard: 200KB → 60KB compressed
- thread-ast-engine (minimal): 1.5MB → 500KB compressed
- thread-rule-engine: 800KB → 280KB compressed
- Application logic: 500KB → 180KB compressed
- **Total: ~3.8MB uncompressed → ~1.3MB compressed** (with wasm-opt -Oz: ~900KB)

**Performance Characteristics**:
- Cold Start: <50ms (Workers V8 isolate initialization)
- RPC Latency: Local (same edge) <10ms, Cross-region 50-100ms
- Serialization: postcard ~0.5ms, JSON ~1.2ms (2.4x slower)
- WebSocket Message Propagation: <50ms globally

**Fallback Strategy**:

If Custom RPC Development Proves Complex:
1. **Phase 1**: Simple HTTP REST with JSON (fastest to implement, ~2MB optimized)
2. **Phase 2**: Add binary serialization (switch to postcard for 40% size reduction)
3. **Phase 3**: Add WebSocket streaming (real-time updates, polling fallback)

For CLI Deployment (No WASM Constraints):
- Can freely use tonic/gRPC if desired
- Or use same HTTP-based protocol for consistency
- Shared trait ensures behavioral equivalence

---

### 4. Graph Database Layer Design

**Status**: ✅ Complete

**Research Output**:

**Decision**: Hybrid Relational Architecture with In-Memory Acceleration

Use Postgres/D1 for persistent graph storage with adjacency list schema, combined with in-memory graph representation for complex queries and content-addressed caching via Recoco.

**Rationale**:

Why NOT Dedicated Graph Databases:
- **Memgraph/Neo4j**: Require separate infrastructure incompatible with Thread's dual deployment model (Postgres CLI + D1 Edge). Memgraph is 100x+ faster than Neo4j but only works as standalone system.
- **SurrealDB**: Emerging technology, mixed performance reports, doesn't support both backends.
- **Infrastructure Complexity**: Adding separate graph DB violates Thread's service-library architecture (Constitution Principle I).

Why Hybrid Relational Works:
1. **Dual Backend Support**: Single schema works across Postgres (CLI) and D1 (Edge) with no architectural changes.
2. **Content-Addressed Caching**: Achieves >90% cache hit rate requirement (Constitution Principle VI) through Recoco integration.
3. **Performance Tiering**: Simple queries (1-2 hops) use indexed SQL; complex queries (3+ hops) load subgraphs into in-memory structures for traversal.
4. **Incremental Updates**: Recoco dataflow triggers only affected subgraph re-analysis on code changes (Constitution Principle IV).

**Alternatives Considered**:
- ❌ **Pure Postgres Recursive CTEs**: Performance degrades exponentially with depth and fan-out, string-based path tracking inefficient, D1's SQLite foundation limits concurrent writes
- ❌ **Materialized Paths**: Good for hierarchical queries but inefficient for non-hierarchical graphs (code has circular dependencies), update overhead
- ❌ **Neo4j/Memgraph**: Performance superior (Memgraph 114-132x faster than Neo4j, 400ms for 100k nodes) but cannot support dual Postgres/D1 deployment, requires separate infrastructure
- ❌ **Apache AGE**: Postgres-only solution (not available for D1/SQLite), doesn't work for edge deployment
- ❌ **petgraph**: Evaluated for in-memory complex queries but not used — custom implementation chosen for tighter integration with the incremental analysis pipeline and to avoid the dependency overhead

**Query Patterns**:

**Schema Design**:
```sql
CREATE TABLE nodes (
    id TEXT PRIMARY KEY,           -- Content-addressed hash
    type TEXT NOT NULL,            -- FILE, CLASS, METHOD, FUNCTION, VARIABLE
    name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    signature TEXT,
    properties JSONB              -- Language-specific metadata
);

CREATE TABLE edges (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    edge_type TEXT NOT NULL,       -- CONTAINS, CALLS, INHERITS, USES, IMPORTS
    weight REAL DEFAULT 1.0,
    PRIMARY KEY (source_id, target_id, edge_type),
    FOREIGN KEY (source_id) REFERENCES nodes(id),
    FOREIGN KEY (target_id) REFERENCES nodes(id)
);

-- Indexes for graph traversal
CREATE INDEX idx_edges_source ON edges(source_id, edge_type);
CREATE INDEX idx_edges_target ON edges(target_id, edge_type);
CREATE INDEX idx_nodes_type_name ON nodes(type, name);
```

**Query Routing Strategy**:
- **1-2 Hop Queries**: Direct SQL with indexed lookups (<10ms Postgres, <50ms D1)
- **3+ Hop Queries**: Load subgraph into custom `DependencyGraph`, execute in-memory algorithms, cache result
- **Reverse Dependencies**: Materialized views for "who depends on me" hot queries

**Scalability Analysis**:

**Storage Requirements (10M nodes, 50M edges)**:
- Postgres: Nodes 5GB + Edges 5GB + Indexes 5GB = ~15GB total (fits comfortably)
- D1: Same schema, distributed across CDN nodes, Recoco caching reduces query load by >90%

**Performance Projections**:
- **Postgres (CLI)**: 1-hop <2ms p95, 2-hop <10ms p95 ✅, 3+ hop <50ms p95 (10ms load + 1ms traversal)
- **D1 (Edge)**: Cached queries <5ms p95, 1-hop <20ms p95, 2-hop <50ms p95 ✅
- **Content-Addressed Cache Hit Rate**: >90% projected ✅ (constitutional requirement)

**Implementation Notes**:
- Custom BFS/topological sort implementation (see `crates/flow/src/incremental/graph.rs`, 1,099 lines) — petgraph evaluated but custom implementation chosen for better integration with incremental update semantics
- Implement incremental graph updates via Recoco diff tracking
- Composite indexes on `(source_id, edge_type)` and `(target_id, edge_type)`
- Materialized views for hot reverse dependency queries

**Actual Implementation** (2026-02-24): `crates/flow/src/incremental/graph.rs` provides:
- Custom `DependencyGraph` with bidirectional adjacency lists
- BFS affected-file detection (O(V+E))
- Topological sort for correct reanalysis order
- DFS cycle detection with temp markers
- **petgraph was NOT used** — custom implementation chosen for tighter integration
  with the incremental analysis pipeline and to avoid the dependency overhead.

---

### 5. Real-Time Protocol Selection

**Status**: ✅ Complete

**Research Output**:

**Decision**: WebSocket Primary, Server-Sent Events (SSE) Fallback, Polling Last Resort

**Rationale**:

1. **gRPC Streaming Not Viable**: Research from Task 3 (API Protocol) confirms HTTP/2 streaming unsupported in Cloudflare Workers. gRPC server-side streaming eliminated as option.

2. **WebSocket Native Support**: Cloudflare Workers natively support WebSockets via `WebSocketPair`. Provides bidirectional streaming ideal for real-time updates and progressive conflict detection results.

3. **SSE for One-Way Streaming**: Server-Sent Events work over HTTP/1.1, compatible with restrictive networks. Sufficient for server→client updates (code changes, conflict alerts). Simpler than WebSocket for one-directional use cases.

4. **Polling Graceful Degradation**: Long-polling fallback for networks that block WebSocket and SSE. Higher latency but ensures universal compatibility.

**Alternatives Considered**:
- ❌ **gRPC Server-Side Streaming**: Not supported by Cloudflare Workers runtime (confirmed in API Protocol research)
- ✅ **WebSocket (Primary)**: Native Workers support, bidirectional, <50ms global latency, works for progressive conflict detection
- ✅ **Server-Sent Events (Fallback)**: HTTP/1.1 compatible, restrictive network friendly, one-way sufficient for many use cases
- ✅ **Long-Polling (Last Resort)**: Universal compatibility, higher latency (100-500ms), acceptable for degraded mode

**Durable Objects Usage**:

Cloudflare Durable Objects enable stateful edge operations:
- **Connection Management**: Track active WebSocket connections per user/project
- **Session State**: Maintain user analysis sessions across requests
- **Collaborative State**: Coordinate multi-user conflict detection and resolution
- **Real-Time Coordination**: Propagate code changes to all connected clients within 100ms

**Implementation Pattern**:
```rust
// Durable Object for session management
#[durable_object]
pub struct AnalysisSession {
    state: State,
    env: Env,
}

#[durable_object]
impl DurableObject for AnalysisSession {
    async fn fetch(&mut self, req: Request) -> Result<Response> {
        // Handle WebSocket upgrade
        if req.headers().get("Upgrade")?.map(|v| v == "websocket").unwrap_or(false) {
            let pair = WebSocketPair::new()?;
            // Accept WebSocket and manage real-time updates
            pair.server.accept()?;
            wasm_bindgen_futures::spawn_local(self.handle_websocket(pair.server));
            Response::ok("")?.websocket(pair.client)
        } else {
            // Handle SSE or polling requests
            self.handle_http(req).await
        }
    }
}
```

**Progressive Conflict Detection Streaming**:

Multi-tier results update clients in real-time:
1. **Tier 1 (AST diff)**: <100ms → WebSocket message → Client shows initial conflict prediction
2. **Tier 2 (Semantic)**: <1s → WebSocket update → Client refines conflict details with accuracy score
3. **Tier 3 (Graph impact)**: <5s → WebSocket final update → Client shows comprehensive analysis with severity ratings

**Fallback Strategy**:

```rust
// Client-side protocol selection
pub enum RealtimeProtocol {
    WebSocket,  // Try first: bidirectional, lowest latency
    SSE,        // Fallback: one-way, restrictive network compatible
    LongPolling, // Last resort: universal compatibility
}

pub async fn connect_realtime(server: &str) -> Result<RealtimeClient> {
    // Try WebSocket
    if let Ok(ws) = connect_websocket(server).await {
        return Ok(RealtimeClient::WebSocket(ws));
    }
    
    // Fallback to SSE
    if let Ok(sse) = connect_sse(server).await {
        return Ok(RealtimeClient::SSE(sse));
    }
    
    // Last resort: polling
    Ok(RealtimeClient::LongPolling(connect_polling(server).await?))
}
```

**Performance Characteristics**:
- WebSocket: <50ms global propagation, <10ms same-edge
- SSE: <100ms propagation, <20ms same-edge
- Long-Polling: 100-500ms latency (poll interval configurable)

---

### 6. Crate Organization Strategy

**Status**: ✅ Complete

**Research Output**:

**Decision**: Extend Existing Thread Workspace with New Graph-Focused Crates

**Rationale**:

1. **Single Workspace Coherence**: Thread already has established workspace with `thread-ast-engine`, `thread-language`, `thread-rule-engine`, `thread-services`, `thread-utils`, `thread-wasm`. Adding new crates to existing workspace maintains build system coherence and dependency management.

2. **Library-Service Boundary Preservation**: New crates clearly split library (reusable graph algorithms) vs service (persistent storage, caching, real-time). Aligns with Constitution Principle I (Service-Library Dual Architecture).

3. **Recoco Integration Point**: `thread-services` becomes integration point for Recoco traits (via `thread-flow`). No type leakage into library crates.

4. **Acyclic Dependency Flow**: Clear dependency hierarchy prevents circular dependencies (Constitution Principle IV requirement).

**Crate Responsibilities**:

**NEW Library Crates** (reusable, WASM-compatible):
- `thread-graph`: Core graph data structures, traversal algorithms, pathfinding (depends on: thread-utils)
- `thread-indexer`: Multi-source code indexing, file watching, change detection (depends on: thread-ast-engine, thread-language)
- `thread-conflict`: Conflict detection engine (multi-tier: AST diff, semantic, graph) (depends on: thread-graph, thread-ast-engine)

**NEW Service Crates** (persistence, orchestration):
- `thread-storage`: Multi-backend storage abstraction (Postgres/D1/Qdrant traits) (depends on: thread-graph)
- `thread-api`: RPC protocol (HTTP+WebSocket), request/response types (depends on: thread-graph, thread-conflict)
- `thread-realtime`: Real-time update propagation, WebSocket/SSE handling, Durable Objects integration (depends on: thread-api)

**EXISTING Crates** (extended/reused):

- `thread-services`: **EXTENDED** - Add Recoco dataflow traits, registry, YAML spec parser (depends on: all new crates)
- `thread-ast-engine`: **REUSED** - AST parsing foundation (no changes)
- `thread-language`: **REUSED** - Language support (no changes)
- `thread-rule-engine`: **EXTENDED** - Add pattern-based conflict detection rules (depends on: thread-conflict)
- `thread-utils`: **REUSED** - SIMD, hashing utilities (no changes)
- `thread-wasm`: **EXTENDED** - Add edge deployment features for new crates (depends on: thread-api, thread-realtime)

**Dependency Graph**:
```
                    ┌──────────────────┐
                    │ thread-services  │ (Service orchestration, Recoco)
                    └────────┬─────────┘
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
   ┌────▼─────┐     ┌───────▼──────┐    ┌───────▼────────┐
   │ thread-  │     │ thread-      │    │ thread-        │
   │ storage  │     │ realtime     │    │ api            │
   └────┬─────┘     └───────┬──────┘    └───────┬────────┘
        │                   │                    │
        │                   │            ┌───────▼────────┐
        │                   │            │ thread-        │
        │                   │            │ conflict       │
        │                   │            └───────┬────────┘
        │                   │                    │
        │           ┌───────▼──────────┐         │
        │           │ thread-indexer   │         │
        │           └───────┬──────────┘         │
        │                   │                    │
   ┌────▼─────────────────────▼─────────────────▼──┐
   │ thread-graph (Core graph data structures)     │
   └────┬───────────────────────────────────────────┘
        │
   ┌────▼──────────────┬──────────────────────┬─────────┐
   │ thread-ast-engine │ thread-language      │ thread- │
   │                   │                      │ utils   │
   └───────────────────┴──────────────────────┴─────────┘
```

**Library-Service Split**:

**Library Crates** (embeddable, reusable):
- thread-graph
- thread-indexer
- thread-conflict
- thread-ast-engine (existing)
- thread-language (existing)
- thread-utils (existing)

**Service Crates** (deployment-specific):
- thread-services (orchestration)
- thread-storage (persistence)
- thread-api (network protocol)
- thread-realtime (WebSocket/Durable Objects)
- thread-wasm (edge deployment)

---

### 7. Multi-Tier Conflict Detection Implementation

**Status**: ✅ Complete

**Research Output**:

**Decision**: Progressive Detection Pipeline with Intelligent Tier Routing

**Rationale**:

1. **Spec Requirement (FR-006)**: "Multi-tier progressive strategy using all available detection methods (AST diff, semantic analysis, graph impact analysis) with intelligent routing. Fast methods provide immediate feedback, slower methods refine accuracy."

2. **Performance Targets**: <100ms (AST), <1s (semantic), <5s (graph impact). Results update progressively as each tier completes.

3. **Accuracy vs Speed Trade-off**: Tier 1 (fast but approximate) → Tier 2 (slower but accurate) → Tier 3 (comprehensive but expensive). Users get immediate feedback that improves over time.

**Tier 1 (AST Diff)**: <100ms for initial detection

**Algorithm**: Git-style tree diff on AST structure
```rust
pub fn ast_diff(old_ast: &Root, new_ast: &Root) -> Vec<ASTConflict> {
    let old_symbols = extract_symbols(old_ast); // Functions, classes, etc.
    let new_symbols = extract_symbols(new_ast);
    
    let mut conflicts = Vec::new();
    for (name, old_node) in old_symbols {
        if let Some(new_node) = new_symbols.get(&name) {
            if old_node.signature != new_node.signature {
                conflicts.push(ASTConflict {
                    symbol: name,
                    kind: ConflictKind::SignatureChange,
                    confidence: 0.6, // Low confidence, needs semantic validation
                    old: old_node,
                    new: new_node,
                });
            }
        } else {
            conflicts.push(ASTConflict {
                symbol: name,
                kind: ConflictKind::Deleted,
                confidence: 0.9, // High confidence
                old: old_node,
                new: None,
            });
        }
    }
    conflicts
}
```

**Data Structures**: 
- Hash-based symbol tables for O(n) diff
- Structural hashing for subtree comparison (similar to Git's tree objects)
- Content-addressed AST nodes for efficient comparison

**Tier 2 (Semantic Analysis)**: <1s for accuracy refinement

**Techniques**:
1. **Type Inference**: Resolve type signatures to detect breaking changes
   - Example: `fn process(x)` → `fn process(x: i32)` may or may not break callers
   - Infer types of call sites to determine if change is compatible
   
2. **Control Flow Analysis**: Detect behavioral changes
   - Example: Adding early return changes execution paths
   - Compare control flow graphs (CFG) to identify semantic shifts

3. **Data Flow Analysis**: Track variable dependencies
   - Example: Changing variable assignment order may affect results
   - Use reaching definitions and use-def chains

**Integration Point**: 
- If using CodeWeaver (from Research Task 2), leverage its semantic characterization layer
- Otherwise, implement minimal semantic analysis using thread-ast-engine metadata

**Tier 3 (Graph Impact Analysis)**: <5s for comprehensive validation

**Algorithm**: Graph reachability and impact propagation
```rust
pub async fn graph_impact_analysis(
    changed_nodes: &[NodeId],
    graph: &CodeGraph,
) -> ImpactReport {
    let mut impact = ImpactReport::new();
    
    for node in changed_nodes {
        // Find all downstream dependencies (who uses this?)
        let dependents = graph.reverse_dependencies(node, max_depth=10);
        
        // Classify severity based on dependency count and criticality
        let severity = classify_severity(dependents.len(), node.criticality);
        
        // Find alternative paths if this breaks
        let alternatives = graph.find_alternative_paths(dependents);
        
        impact.add_conflict(GraphConflict {
            symbol: node,
            affected_count: dependents.len(),
            severity,
            suggested_fixes: alternatives,
            confidence: 0.95, // High confidence from comprehensive analysis
        });
    }
    
    impact
}
```

**Graph Operations** (using custom `DependencyGraph` from `crates/flow/src/incremental/graph.rs`; petgraph was evaluated but not used — see Research Task 4):
- Reverse dependency traversal (BFS from changed nodes)
- Strongly connected components (detect circular dependencies affected by change)
- Shortest path alternative detection (suggest refactoring paths)

**Progressive Streaming**: How results update clients in real-time

**WebSocket Protocol** (from Research Task 5):
```rust
pub enum ConflictUpdate {
    TierOneComplete { conflicts: Vec<ASTConflict>, timestamp: DateTime },
    TierTwoRefinement { updated: Vec<SemanticConflict>, timestamp: DateTime },
    TierThreeComplete { final_report: ImpactReport, timestamp: DateTime },
}

pub async fn stream_conflict_detection(
    old_code: &str,
    new_code: &str,
    ws: WebSocket,
) -> Result<()> {
    // Tier 1: AST diff (fast)
    let tier1 = ast_diff(parse(old_code), parse(new_code));
    ws.send(ConflictUpdate::TierOneComplete { 
        conflicts: tier1.clone(),
        timestamp: now(),
    }).await?;
    
    // Tier 2: Semantic analysis (medium)
    let tier2 = semantic_analysis(tier1, parse(old_code), parse(new_code)).await;
    ws.send(ConflictUpdate::TierTwoRefinement {
        updated: tier2.clone(),
        timestamp: now(),
    }).await?;
    
    // Tier 3: Graph impact (comprehensive)
    let tier3 = graph_impact_analysis(&tier2, &load_graph()).await;
    ws.send(ConflictUpdate::TierThreeComplete {
        final_report: tier3,
        timestamp: now(),
    }).await?;
    
    Ok(())
}
```

**Client Experience**:
1. **Immediate Feedback (100ms)**: "Potential conflict detected in function signature" (low confidence)
2. **Refined Accuracy (1s)**: "Breaking change confirmed - 15 callers affected" (medium confidence)
3. **Comprehensive Analysis (5s)**: "High severity - critical path affected, 3 alternative refactoring strategies suggested" (high confidence)

**Intelligent Tier Routing**:

Not all conflicts need all three tiers. Route based on confidence:
```rust
pub fn should_run_tier2(tier1_result: &[ASTConflict]) -> bool {
    // Skip semantic analysis if Tier 1 has high confidence
    tier1_result.iter().any(|c| c.confidence < 0.8)
}

pub fn should_run_tier3(tier2_result: &[SemanticConflict]) -> bool {
    // Only run graph analysis for breaking changes or low confidence
    tier2_result.iter().any(|c| c.is_breaking || c.confidence < 0.9)
}
```

**Performance Optimization**:
- Parallel tier execution where possible (Tier 2 and 3 can start before Tier 1 completes if working on different symbols)
- Cache intermediate results in Recoco (content-addressed AST nodes reused across tiers)
- Early termination if high-confidence result achieved before final tier

---

### 8. Storage Backend Abstraction Pattern

**Status**: ✅ Complete

**Research Output**:

**IMPLEMENTED** (2026-02-24): The storage backend abstraction described below has been
implemented in `crates/flow/src/incremental/`:

- `storage.rs`: `StorageBackend` async trait (CRUD for fingerprints, edges, full graph)
- `backends/postgres.rs`: `PostgresIncrementalBackend` (feature-gated: `postgres-backend`)
- `backends/d1.rs`: `D1IncrementalBackend` (HTTP REST client, 200+ lines)
- `backends/mod.rs`: Factory pattern (`create_backend(BackendType, BackendConfig)`)
- `InMemoryBackend`: Always-available test backend

**Vector Storage (Qdrant → Vectorize)**:
The `QdrantStorage` in the original plan is **blocked** — `recoco/target-qdrant` is
disabled in Cargo.toml due to a CRC version conflict with other dependencies.

**Resolution**: Implement `VectorizeStorage` using Cloudflare Vectorize API for edge
vector search. Qdrant support (CLI-only) can be added later when the dependency
conflict is resolved.

---

**Decision**: Trait-Based Multi-Backend Abstraction with Backend-Specific Optimizations

**Rationale**:

1. **Constitutional Requirement**: Service Architecture & Persistence (Principle VI) requires support for Postgres (CLI), D1 (Edge), and Qdrant (vectors) from single codebase.

2. **Performance Preservation**: Trait abstraction must not sacrifice performance. Backend-specific optimizations (Postgres CTEs, D1 PRAGMA, Qdrant vector indexing) implemented via trait methods.

3. **Migration Support**: Schema versioning and rollback scripts essential for production service (SC-STORE-001 requires <10ms Postgres, <50ms D1, <100ms Vectorize p95 latency (Qdrant blocked — dependency conflict; superseded by Vectorize for edge)).

**Trait Definition**:

```rust
// thread-storage/src/traits.rs

#[async_trait::async_trait]
pub trait GraphStorage: Send + Sync {
    /// Store graph nodes (symbols)
    async fn store_nodes(&self, nodes: &[GraphNode]) -> Result<()>;
    
    /// Store graph edges (relationships)
    async fn store_edges(&self, edges: &[GraphEdge]) -> Result<()>;
    
    /// Query nodes by ID
    async fn get_nodes(&self, ids: &[NodeId]) -> Result<Vec<GraphNode>>;
    
    /// Query edges by source/target
    async fn get_edges(&self, source: NodeId, edge_type: EdgeType) -> Result<Vec<GraphEdge>>;
    
    /// Graph traversal (1-2 hops, optimized per backend)
    async fn traverse(&self, start: NodeId, depth: u32, edge_types: &[EdgeType]) 
        -> Result<TraversalResult>;
    
    /// Reverse dependencies (who calls/uses this?)
    async fn reverse_deps(&self, target: NodeId, edge_types: &[EdgeType])
        -> Result<Vec<GraphNode>>;
    
    /// Backend-specific optimization hook
    async fn optimize_for_query(&self, query: &GraphQuery) -> Result<QueryPlan>;
}

#[async_trait::async_trait]
pub trait VectorStorage: Send + Sync {
    /// Store vector embeddings for semantic search
    async fn store_vectors(&self, embeddings: &[(NodeId, Vec<f32>)]) -> Result<()>;
    
    /// Similarity search (k-nearest neighbors)
    async fn search_similar(&self, query: &[f32], k: usize) -> Result<Vec<(NodeId, f32)>>;
}

#[async_trait::async_trait]
pub trait StorageMigration: Send + Sync {
    /// Apply schema migration
    async fn migrate_up(&self, version: u32) -> Result<()>;
    
    /// Rollback schema migration
    async fn migrate_down(&self, version: u32) -> Result<()>;
    
    /// Get current schema version
    async fn current_version(&self) -> Result<u32>;
}
```

**Backend-Specific Optimizations**:

**Postgres Implementation**:
```rust
pub struct PostgresStorage {
    pool: PgPool,
}

#[async_trait::async_trait]
impl GraphStorage for PostgresStorage {
    async fn traverse(&self, start: NodeId, depth: u32, edge_types: &[EdgeType]) 
        -> Result<TraversalResult> {
        // Use recursive CTE for multi-hop queries
        let query = sqlx::query(r#"
            WITH RECURSIVE traversal AS (
                SELECT id, name, type, 0 as depth
                FROM nodes WHERE id = $1
                UNION ALL
                SELECT n.id, n.name, n.type, t.depth + 1
                FROM nodes n
                JOIN edges e ON e.target_id = n.id
                JOIN traversal t ON e.source_id = t.id
                WHERE t.depth < $2 AND e.edge_type = ANY($3)
            )
            SELECT * FROM traversal
        "#)
        .bind(&start)
        .bind(depth as i32)
        .bind(&edge_types)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(TraversalResult::from_rows(query))
    }
    
    async fn optimize_for_query(&self, query: &GraphQuery) -> Result<QueryPlan> {
        // PostgreSQL-specific: EXPLAIN ANALYZE for query planning
        Ok(QueryPlan::UseIndex("idx_edges_source"))
    }
}
```

**D1 Implementation** (Cloudflare Edge):
```rust
pub struct D1Storage {
    db: D1Database,
}

#[async_trait::async_trait]
impl GraphStorage for D1Storage {
    async fn traverse(&self, start: NodeId, depth: u32, edge_types: &[EdgeType])
        -> Result<TraversalResult> {
        // D1/SQLite: Use PRAGMA for performance
        self.db.exec("PRAGMA journal_mode=WAL").await?;
        self.db.exec("PRAGMA synchronous=NORMAL").await?;
        
        // Same recursive CTE as Postgres (SQLite compatible)
        let query = self.db.prepare(r#"
            WITH RECURSIVE traversal AS (
                SELECT id, name, type, 0 as depth FROM nodes WHERE id = ?1
                UNION ALL
                SELECT n.id, n.name, n.type, t.depth + 1
                FROM nodes n
                JOIN edges e ON e.target_id = n.id
                JOIN traversal t ON e.source_id = t.id
                WHERE t.depth < ?2 AND e.edge_type IN (?3)
            )
            SELECT * FROM traversal
        "#)
        .bind(start)?
        .bind(depth)?
        .bind(edge_types)?
        .all()
        .await?;
        
        Ok(TraversalResult::from_d1_rows(query))
    }
    
    async fn optimize_for_query(&self, query: &GraphQuery) -> Result<QueryPlan> {
        // D1-specific: Leverage edge CDN caching
        Ok(QueryPlan::CacheHint { ttl: Duration::from_secs(300) })
    }
}
```

**Qdrant Implementation** (Vector Search):
```rust
pub struct QdrantStorage {
    client: QdrantClient,
    collection: String,
}

#[async_trait::async_trait]
impl VectorStorage for QdrantStorage {
    async fn store_vectors(&self, embeddings: &[(NodeId, Vec<f32>)]) -> Result<()> {
        let points: Vec<_> = embeddings.iter()
            .enumerate()
            .map(|(i, (id, vec))| {
                PointStruct::new(i as u64, vec.clone(), Payload::new())
                    .with_payload(payload!({ "node_id": id.to_string() }))
            })
            .collect();
        
        self.client
            .upsert_points(&self.collection, points, None)
            .await?;
        Ok(())
    }
    
    async fn search_similar(&self, query: &[f32], k: usize) -> Result<Vec<(NodeId, f32)>> {
        let results = self.client
            .search_points(&self.collection, query.to_vec(), k as u64, None, None, None)
            .await?;
        
        Ok(results.result.into_iter()
            .map(|p| (NodeId::from(p.payload["node_id"].as_str().unwrap()), p.score))
            .collect())
    }
}
```

**Migration Strategy**:

**Schema Versioning**:
```sql
-- migrations/001_initial_schema.sql
CREATE TABLE schema_version (version INTEGER PRIMARY KEY);
INSERT INTO schema_version VALUES (1);

CREATE TABLE nodes (
    id TEXT PRIMARY KEY,
    type TEXT NOT NULL,
    name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    signature TEXT,
    properties JSONB
);

CREATE TABLE edges (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    edge_type TEXT NOT NULL,
    weight REAL DEFAULT 1.0,
    PRIMARY KEY (source_id, target_id, edge_type),
    FOREIGN KEY (source_id) REFERENCES nodes(id),
    FOREIGN KEY (target_id) REFERENCES nodes(id)
);

-- migrations/001_rollback.sql
DROP TABLE edges;
DROP TABLE nodes;
DELETE FROM schema_version WHERE version = 1;
```

**Migration Execution**:
```rust
impl StorageMigration for PostgresStorage {
    async fn migrate_up(&self, version: u32) -> Result<()> {
        let migration = load_migration(version)?;
        
        // Execute in transaction
        let mut tx = self.pool.begin().await?;
        sqlx::query(&migration.up_sql).execute(&mut *tx).await?;
        sqlx::query("UPDATE schema_version SET version = $1")
            .bind(version as i32)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        
        Ok(())
    }
    
    async fn migrate_down(&self, version: u32) -> Result<()> {
        let migration = load_migration(version)?;
        
        let mut tx = self.pool.begin().await?;
        sqlx::query(&migration.down_sql).execute(&mut *tx).await?;
        sqlx::query("UPDATE schema_version SET version = $1")
            .bind((version - 1) as i32)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        
        Ok(())
    }
}
```

**Resilience Patterns**:

**Connection Pooling**:
```rust
pub struct PostgresStorage {
    pool: PgPool,  // sqlx connection pool
}

impl PostgresStorage {
    pub async fn new(url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(20)
            .min_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(url)
            .await?;
        Ok(Self { pool })
    }
}
```

**Retry Logic** (exponential backoff):
```rust
pub async fn with_retry<F, T>(operation: F) -> Result<T>
where
    F: Fn() -> BoxFuture<'static, Result<T>>,
{
    let mut backoff = Duration::from_millis(100);
    for attempt in 0..5 {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if is_transient_error(&e) => {
                tokio::time::sleep(backoff).await;
                backoff *= 2; // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }
    Err(Error::MaxRetriesExceeded)
}
```

**Circuit Breaker**:
```rust
pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_threshold: usize,
    timeout: Duration,
}

enum CircuitState {
    Closed,
    Open { since: Instant },
    HalfOpen,
}

impl CircuitBreaker {
    pub async fn call<F, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> BoxFuture<'static, Result<T>>,
    {
        let state = self.state.lock().await.clone();
        match state {
            CircuitState::Open { since } if since.elapsed() < self.timeout => {
                Err(Error::CircuitBreakerOpen)
            }
            CircuitState::Open { .. } => {
                // Try half-open
                *self.state.lock().await = CircuitState::HalfOpen;
                self.execute_and_update(operation).await
            }
            _ => self.execute_and_update(operation).await,
        }
    }
}

---

## Best Practices Research

### Rust WebAssembly for Cloudflare Workers

**Status**: ✅ Decided elsewhere

**Questions**:
- What are current best practices for Rust WASM on Cloudflare Workers (2026)?
- How to achieve <10MB compressed bundle size?
- What crates are WASM-compatible vs problematic?
- How to handle async I/O in WASM context?

**Research Output**: Decisions documented in spec.md FR-010, SC-EDGE-001 through SC-EDGE-003. WASM compilation via xtask; multi-threading WASM for browser; single-threaded for Workers.

---

### Content-Addressed Caching Patterns

**Status**: ✅ Decided elsewhere

**Questions**:
- What are proven patterns for >90% cache hit rates?
- How to implement incremental invalidation efficiently?
- What content-addressing schemes (SHA-256, blake3) balance speed and collision resistance?
- How to handle cache warmup and cold-start scenarios?

**Research Output**: Decisions documented in plan.md (Recoco/thread-flow integration) and spec.md FR-004. Blake3 fingerprinting via Recoco; PostgresIncrementalBackend and D1IncrementalBackend implemented.

---

### Real-Time Collaboration Architecture

**Status**: ✅ Decided elsewhere

**Questions**:
- What are architectural patterns for real-time collaborative systems at scale?
- How to handle consistency across distributed edge nodes?
- What conflict resolution strategies work for code intelligence systems?
- How to balance latency vs consistency trade-offs?

**Research Output**: Decisions documented in spec.md FR-005, FR-006, FR-012. WebSocket primary, SSE fallback; RealtimeBackend trait in thread-realtime; Durable Objects implementation in private commercial crate.

---

### Research Task 9: Semantic Classification System Analysis ✅ COMPLETE

**Objective**: Evaluate CodeWeaver's semantic classification system as a candidate for Thread's AST node type classification needs, particularly for AI-context importance ranking and L1 definition extraction.

**Findings**:

**The CodeWeaver Python system** (7,200 lines, 11 modules) classifies tree-sitter AST node types into 22 semantic categories with multi-dimensional importance scoring. Key characteristics:
- 7-method classification pipeline with evidence tracking and confidence scoring
- Grammar model loaded at runtime from node-types.json (complex, not needed for Thread)
- Pickle cache for performance (replaced by embedded JSON/TOML in Rust port)

**The simplified Rust port** (`SEMANTIC_CLASSIFICATION_SPEC.md`) achieves equivalent accuracy via:
- Pre-baked data: 2,444 exact + 21 majority cross-language rules in universal_rules.json
- 8-stage lookup pipeline (no grammar model needed at runtime)
- Per-language TOML overrides (~10-50 lines) for 100% accuracy

**Key quantitative findings from the data** (`classifications/` directory, already in repo):
- 5,899 total items classified across 27 languages → 5,819 classified (98.6% coverage)
- Override TOML files already exist in correct format for the Rust spec
- Method distribution: token_purpose (57.9%), specific_thing/universal (30%), category (8.5%), connection_inference (3.5%)
- 80%+ baseline accuracy achievable on any tree-sitter grammar with universal rules alone
- ~10-50 TOML lines achieves 100% per language (CodeWeaver has 27 validated; ~166 total possible)

**Decision**: Proceed with Rust port as `thread-definitions` crate. SemanticClass (22 variants) replaces GraphNode.node_type. The `classify_node_types` operator in thread-flow replaces the tags.scm-based L1 definition extraction proposed in AI_KNOWLEDGE_LAYER_DESIGN.md.

**Data location**: `classifications/` directory (workspace root) — migrate to `crates/definitions/data/` at T-C07.

---

## Research Completion Criteria

Research phase is complete when:
- [x] All 9 research tasks have Decision, Rationale, and Alternatives documented (Task 9 added 2026-02-24)
- [x] All best practices research areas have findings (integrated into tasks)
- [x] Technical Context in plan.md updated with concrete values
- [x] Constitution Check re-evaluated with research findings
- [x] Crate organization finalized and documented in plan.md Project Structure
- [x] Ready to proceed to Phase 1 (Design & Contracts)
- [x] Phase 1 infrastructure implemented in thread-flow crate (2026-02-24)

**Status**: Research tasks complete, proceeding to plan.md updates

---

## Next Steps After Research

After completing research.md:
1. Update plan.md Technical Context with concrete decisions (remove "PENDING RESEARCH")
2. Re-evaluate Constitution Check Principle IV (Modular Design) with finalized crate organization
3. Proceed to Phase 1: Generate data-model.md, contracts/, quickstart.md
4. Update agent context via `.specify/scripts/bash/update-agent-context.sh claude`
