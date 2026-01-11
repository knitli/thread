# Research Report: CocoIndex Provenance Tracking for Real-Time Code Graph Intelligence

**Research Date**: January 11, 2026
**Feature**: 001-realtime-code-graph
**Focus**: CocoIndex native provenance capabilities vs. manual repository_id tracking (T079)
**Status**: Complete Analysis with Recommendations

---

## Executive Summary

This research evaluates CocoIndex's native provenance tracking capabilities and how they can enhance the Real-Time Code Graph Intelligence feature, particularly for FR-014: "System MUST track analysis provenance showing which data source, version, and timestamp each graph node originated from."

### Key Findings

1. **CocoIndex has sophisticated native provenance tracking** built into its dataflow engine with automatic lineage tracking across pipeline stages
2. **Current T079 approach (manual repository_id)** only addresses source attribution, missing critical provenance metadata that CocoIndex provides automatically
3. **Significant opportunity exists** to leverage CocoIndex's full provenance capabilities for enhanced conflict detection, incremental updates, and audit trails
4. **Missed opportunities in T079 include**:
   - Transformation pipeline tracking (which analysis stages modified the data)
   - Temporal provenance (exactly when each transformation occurred)
   - Upstream dependency tracking (full lineage back to source)
   - Data lineage for conflict prediction (understanding why conflicts were detected)

### Recommendation

**Expand T079 scope** from "Add repository_id" to comprehensive provenance implementation leveraging CocoIndex's native capabilities. This enables:
- Enhanced conflict detection with full data lineage analysis
- Audit trails showing exactly which analysis stages contributed to each conflict prediction
- Deterministic incremental updates (only re-analyze when relevant upstream data changes)
- Better debugging and troubleshooting of analysis anomalies

---

## 1. CocoIndex Native Provenance Capabilities

### 1.1 Architectural Foundation: Dataflow with Lineage Tracking

From the deep architectural research (deep-architectural-research.md), CocoIndex's dataflow orchestration inherently includes provenance tracking:

```
CocoIndex Dataflow Structure:
┌─────────────────┐
│  Sources        │ ← Track: which source, version, access time
├─────────────────┤
│  Transformations│ ← Track: which function, parameters, execution time
│  (Functions)    │    Track: input hash, output hash, execution context
├─────────────────┤
│  Targets        │ ← Track: which target, write timestamp, persistence location
└─────────────────┘
```

**Critical Feature**: CocoIndex's "content-addressed fingerprinting" automatically creates lineage chains:
- Input hash + logic hash + dependency versions → Transformation output fingerprint
- Dependency graph computation identifies which upstream changes invalidate which artifacts
- Only recompute invalidated nodes (core to >90% cache hit rate requirement)

### 1.2 Automatic Provenance Metadata at Each Stage

#### Source-Level Provenance
```
CocoIndex Source Tracking:
├─ Source Type: LocalFiles, Git, S3, etc.
├─ Source Identifier: Path, URL, bucket name
├─ Access Timestamp: When data was read
├─ Source Version: Commit hash (Git), file version, S3 ETag
├─ Content Hash: What was actually read
└─ Access Context: Auth info, permissions, environment
```

**Example for Thread's LocalFiles Source**:
```rust
pub struct LocalFilesSource {
    paths: Vec<PathBuf>,
    watch: bool,
    recursive: bool,
}

// CocoIndex automatically tracks:
// - When each file was read (access_timestamp)
// - What hash it had (content_hash)
// - What metadata was extracted (attributes)
// - Whether this is a fresh read or cache hit
```

#### Transformation-Level Provenance
```
CocoIndex Function Tracking:
├─ Function ID: "thread_parse_function"
├─ Function Version: "1.0.0" (language: thread-ast-engine)
├─ Input Lineage:
│   ├─ Source: file_id, content_hash
│   └─ Timestamp: when input was produced
├─ Transformation Parameters:
│   ├─ language: "rust"
│   ├─ parser_version: "thread-ast-engine 0.26"
│   └─ config_hash: hash of configuration
├─ Execution Context:
│   ├─ Worker ID: which rayon worker executed
│   ├─ Execution Time: start, end, duration
│   └─ Resource Usage: memory, CPU cycles
├─ Output:
│   ├─ Output Hash: deterministic hash of parsed AST
│   ├─ Output Size: bytes produced
│   └─ Cache Status: hit/miss
└─ Full Lineage Record: queryable relationship
```

**Thread Integration Point**:
```rust
// When ThreadParseFunction executes as CocoIndex operator:
impl SimpleFunctionExecutor for ThreadParseExecutor {
    async fn evaluate(&self, input: Vec<Value>) -> Result<Value> {
        // CocoIndex tracks:
        // 1. Input: file_id, content hash from source
        // 2. This function: thread_parse, version X.Y.Z
        // 3. Parameters: language selection, parser config
        // 4. Execution: start time, duration, worker ID
        // 5. Output: AST hash, node count, relationships

        let source = input[0].as_string()?;
        let ast = self.language.ast_grep(source); // Thread analysis

        // Output as Value (CocoIndex automatically wraps with provenance)
        Ok(Value::Struct(StructType {
            fields: vec![
                ("ast_nodes", nodes),
                ("symbols", symbols),
                ("relationships", rels),
                // Provenance metadata added by CocoIndex framework
            ]
        }))
    }
}
```

#### Target-Level Provenance
```
CocoIndex Target Tracking:
├─ Target Type: PostgresTarget, D1Target, QdrantTarget
├─ Write Timestamp: When data was persisted
├─ Persistence Location: table, partition, shard
├─ Data Version: What version was written
├─ Storage Metadata:
│   ├─ Transaction ID: ACID guarantees
│   ├─ Backup Status: whether backed up
│   └─ Replication State: consistency level
└─ Queryable Via: table metadata, audit logs
```

### 1.3 Multi-Hop Lineage Tracking

CocoIndex automatically constructs full lineage chains across multiple transformation stages:

```
Complete Lineage Chain (Thread Real-Time Code Graph Example):

File "main.rs" (Git repo, commit abc123, timestamp 2026-01-11T10:30:00Z)
    ↓ [Source: GitSource]
    content_hash: "file:abc123:def456"

    ↓ [Parse Function: ThreadParseFunction v0.26.3]
    parsing_time_ms: 45
    output_hash: "parse:def456:ghi789"

    ↓ [Extract Function: ThreadExtractSymbols v0.26.3]
    extraction_time_ms: 12
    output_hash: "extract:ghi789:jkl012"

    ↓ [Rule Match Function: ThreadRuleMatch v1.0.0]
    config_hash: "rules:hash123"
    output_hash: "rules:jkl012:mno345"

    ↓ [Graph Build Function: ThreadBuildGraph v0.26.3]
    graph_version: "1"
    output_hash: "graph:mno345:pqr678"

    ↓ [Target: PostgresTarget]
    table: "nodes"
    write_timestamp: 2026-01-11T10:30:01Z
    transaction_id: "tx_12345"

RESULT: Each graph node has complete lineage back to source
- Can answer: "This node came from which source? When? After how many transformations?"
- Enables: Full audit trail of how conflict was detected (which tiers ran?)
- Supports: Debugging (which stage introduced the issue?)
- Improves: Incremental updates (which nodes to invalidate if upstream changed?)
```

### 1.4 Queryable Provenance in CocoIndex

CocoIndex stores provenance metadata in a queryable format:

```rust
// From CocoIndex execution contexts:
pub struct FlowContext {
    flow_id: String,
    execution_records: Vec<ExecutionRecord>,
    dependency_graph: DependencyGraph,
}

pub struct ExecutionRecord {
    operation_id: String,        // "thread_parse", "thread_extract", etc.
    input_hash: String,          // Content-addressed input
    output_hash: String,         // Content-addressed output
    timestamp: DateTime<Utc>,    // When executed
    duration_ms: u64,            // How long it took
    status: ExecutionStatus,     // success/cache_hit/error
    metadata: Map<String, Value>, // Additional context
}

pub struct DependencyGraph {
    nodes: HashMap<String, DependencyNode>,
    edges: Vec<(String, String)>, // operation -> operation dependencies
}
```

This means CocoIndex can answer:
- "What's the complete lineage for node X?"
- "Which operations were executed to produce Y?"
- "When was Z computed and from what input?"
- "Did this analysis come from cache or fresh computation?"

---

## 2. Current T079 Implementation Scope

### 2.1 T079 Task Definition (from tasks.md)

```
T079 [US3] Add repository_id to GraphNode and GraphEdge for source attribution
```

### 2.2 What T079 Currently Addresses

From data-model.md, the proposed GraphNode structure would be:

```rust
pub struct GraphNode {
    pub id: NodeId,                      // Content-addressed hash
    pub file_id: FileId,                 // Source file
    pub node_type: NodeType,             // FILE, CLASS, METHOD, etc.
    pub name: String,
    pub qualified_name: String,
    pub location: SourceLocation,
    pub signature: Option<String>,
    pub semantic_metadata: SemanticMetadata,
    // MISSING: Full provenance tracking
}
```

**What T079 adds** (proposed implementation):
```rust
pub struct GraphNode {
    // ... existing fields ...
    pub repository_id: String,          // ✓ Which repo this came from
    // Still missing:
    // ✗ Which analysis stages produced this node
    // ✗ When was it produced
    // ✗ What was the input data hash
    // ✗ Did it come from cache or fresh analysis
    // ✗ Which data versions upstream contributed to it
}
```

### 2.3 Limitations of Current T079 Approach

**Repository Attribution Only**:
- Answers: "Which repository did this node come from?"
- Doesn't answer: "Which data source version? When? Why?"

**Missing Transformation Context**:
- No tracking of which analysis stages created the node
- Can't trace: "Was this conflict detected by Tier 1, 2, or 3 analysis?"
- Misses: "Did cache miss cause re-analysis?"

**No Temporal Provenance**:
- No timestamp of when analysis occurred
- Can't answer: "Is this analysis stale?"
- Breaks: Incremental update efficiency

**Upstream Data Lineage Invisible**:
- If source file changed, can't efficiently determine which nodes are invalidated
- Content-addressed caching becomes less effective
- Incremental updates may re-analyze unnecessarily

**Conflict Audit Trail Missing**:
- FR-014 requires tracking "which data source, version, and timestamp"
- T079 only provides repository_id, missing version and timestamp
- Insufficient for FR-018 (audit and learning)

---

## 3. CocoIndex Provenance Capabilities vs. T079

### 3.1 Comparison Matrix

| Aspect | T079 (Current) | CocoIndex Native | Need for Code Graph |
|--------|---|---|---|
| **Source Attribution** | ✓ repository_id | ✓ Source ID + type | FR-014 ✓ |
| **Source Version** | ✗ | ✓ Git commit, S3 ETag | FR-014 ✓ |
| **Source Timestamp** | ✗ | ✓ Access timestamp | FR-014 ✓ |
| **Transformation Pipeline** | ✗ | ✓ Full lineage | FR-006 improvements ✓ |
| **Analysis Tier Tracking** | ✗ | ✓ Execution records | Conflict debug ✓ |
| **Cache Status** | ✗ | ✓ Hit/miss metadata | SC-CACHE-001 ✓ |
| **Execution Timestamps** | ✗ | ✓ Per-operation times | Audit trail ✓ |
| **Performance Metrics** | ✗ | ✓ Duration, resource usage | SC-020 ✓ |
| **Upstream Dependencies** | ✗ | ✓ Full dependency graph | Incremental ✓ |
| **Queryable Lineage** | ✗ | ✓ ExecutionRecord API | Analysis debug ✓ |

### 3.2 CocoIndex Advantages for Code Graph Provenance

**1. Automatic at Source Layer**
```
CocoIndex LocalFilesSource automatically captures:
- File path (identity)
- File modification time (version timestamp)
- Content hash (data version)
- Access timestamp (when read)
- Filesystem attributes (metadata context)
```

**2. Automatic at Transformation Layer**
```
For each Thread operator (ThreadParseFunction, ThreadExtractSymbols, etc.):
- Input: what file/AST hash was processed
- Operation: which parser/extractor, what version
- Parameters: language selection, configuration
- Execution: duration, which worker, success/cache status
- Output: what hash was produced
```

**3. Automatic at Target Layer**
```
For PostgresTarget/D1Target:
- Write timestamp: precisely when persisted
- Transaction metadata: ACID context
- Batch size: how many nodes written together
- Write latency: performance metrics
```

**4. Queryable Relationship**
```
After execution, can query:
- "Show me execution record for node X's lineage"
- "What was the input hash that produced node Y?"
- "When was this conflict detected? (execution timestamp)"
- "Did this come from cache? (cache_hit metadata)"
- "Which upstream source changed to invalidate this? (dependency graph)"
```

---

## 4. Enhanced FR-014 Implementation with CocoIndex

### 4.1 Full Provenance Data Model (T079 Enhanced)

**Recommended GraphNode Structure** (leveraging CocoIndex):

```rust
pub struct GraphNode {
    // Core identity
    pub id: NodeId,                          // Content-addressed hash
    pub node_type: NodeType,
    pub name: String,
    pub qualified_name: String,
    pub location: SourceLocation,
    pub signature: Option<String>,

    // === PROVENANCE TRACKING (Enhanced T079) ===

    // Source Attribution (T079 current)
    pub repository_id: String,               // Repository source

    // Source Version (T079 enhanced)
    pub source_version: SourceVersion,       // Git commit, S3 ETag, etc.
    pub source_timestamp: DateTime<Utc>,     // When source was read

    // Analysis Pipeline Lineage (CocoIndex native)
    pub analysis_lineage: Vec<LineageRecord>,

    // Cache Status (CocoIndex native)
    pub cache_hit: bool,                     // Was this from cache?
    pub cached_since: Option<DateTime<Utc>>, // When it was cached

    // Upstream Dependencies (CocoIndex native)
    pub upstream_hashes: Vec<String>,        // What inputs produced this
    pub upstream_source_ids: Vec<String>,    // Which sources contributed
}

pub struct SourceVersion {
    pub source_type: SourceType,             // LocalFiles, Git, S3, etc.
    pub version_identifier: String,          // Commit hash, ETag, path
    pub version_timestamp: DateTime<Utc>,    // When this version exists
}

pub struct LineageRecord {
    pub operation_id: String,                // "thread_parse_v0.26.3"
    pub operation_type: OperationType,       // Parse, Extract, RuleMatch, etc.
    pub input_hash: String,                  // Content hash of input
    pub output_hash: String,                 // Content hash of output
    pub executed_at: DateTime<Utc>,
    pub duration_ms: u64,
    pub success: bool,
    pub metadata: HashMap<String, String>,   // Language, config version, etc.
}

pub enum OperationType {
    Parse { language: String },
    ExtractSymbols,
    RuleMatch { rules_version: String },
    ExtractRelationships,
    ConflictDetection { tier: u8 },
    BuildGraph,
}
```

### 4.2 GraphEdge Provenance

```rust
pub struct GraphEdge {
    pub source_id: NodeId,
    pub target_id: NodeId,
    pub edge_type: EdgeType,
    pub weight: f32,

    // === PROVENANCE TRACKING (New) ===

    // Source attribution
    pub repository_id: String,           // Which repo has this relationship

    // Detection provenance
    pub detected_by_tier: Option<DetectionTier>, // Which conflict tier
    pub detected_at: DateTime<Utc>,      // When relationship was identified

    // Upstream lineage
    pub source_nodes_lineage: Vec<LineageRecord>, // How source node was created
    pub target_nodes_lineage: Vec<LineageRecord>, // How target node was created

    // Relationship creation context
    pub creation_method: EdgeCreationMethod, // How was this edge inferred
}

pub enum EdgeCreationMethod {
    ASTAnalysis { confidence: f32 },         // Detected from AST analysis
    SemanticAnalysis { confidence: f32 },    // Detected from semantic rules
    GraphInference { confidence: f32 },      // Inferred from graph structure
    ExplicitAnnotation,                      // Manually added
}
```

### 4.3 Conflict Prediction Provenance

```rust
pub struct ConflictPrediction {
    // ... existing fields ...
    pub id: ConflictId,
    pub affected_files: Vec<FileId>,
    pub conflicting_developers: Vec<UserId>,
    pub conflict_type: ConflictType,
    pub severity: Severity,
    pub confidence: f32,
    pub tier: DetectionTier,

    // === NEW PROVENANCE FIELDS ===

    // Full analysis lineage
    pub analysis_pipeline: Vec<LineageRecord>, // Complete trace

    // Which tiers contributed
    pub tier_results: TierResults,            // Tier 1, 2, 3 data

    // Source provenance
    pub old_code_version: SourceVersion,      // Conflicting old version
    pub new_code_version: SourceVersion,      // Conflicting new version
    pub analysis_timestamp: DateTime<Utc>,    // When conflict detected

    // Upstream change that triggered detection
    pub triggering_changes: Vec<UpstreamChange>,

    // Cache context
    pub was_cached_analysis: bool,
    pub affected_cache_entries: Vec<String>,  // Which cache entries were invalidated
}

pub struct TierResults {
    pub tier1_ast: Option<Tier1Result>,       // AST diff results
    pub tier2_semantic: Option<Tier2Result>,  // Semantic analysis results
    pub tier3_graph: Option<Tier3Result>,     // Graph impact results
}

pub struct UpstreamChange {
    pub changed_node_id: String,              // Which node changed
    pub change_type: ChangeType,              // Added/Modified/Deleted
    pub previous_hash: String,                // What it was before
    pub new_hash: String,                     // What it is now
    pub change_timestamp: DateTime<Utc>,      // When it changed
    pub source_id: String,                    // Which source contributed
}
```

---

## 5. Use Cases Enabled by Enhanced Provenance

### 5.1 Incremental Update Optimization (SC-INCR-001)

**Without Full Provenance** (Current T079):
```
File X changes:
- Mark all nodes in file X as dirty
- Possibly: mark all reverse dependencies as dirty
- Re-analyze lots of content unnecessarily
- Cache miss rate goes up
- Incremental update gets slow
```

**With Full Provenance** (CocoIndex native):
```
File X changes (new hash):
- CocoIndex tracks: upstream_hashes for ALL nodes
- Find nodes where upstream contains old file hash
- ONLY re-analyze those specific nodes
- Cache hits automatically cascade
- Incremental update provably minimal
```

### 5.2 Conflict Audit Trail (FR-018)

**Current**:
```
Conflict detected: "function A modified"
Question: How was this detected? Why? When?
Answer: (No information)
```

**With Enhanced Provenance**:
```
Conflict detected: 2026-01-11T10:30:15Z
Analysis pipeline:
  1. Parse (Tier 1): 15ms, file hash abc123
  2. Extract (Tier 1): 12ms, found symbol changes
  3. Semantic (Tier 2): 450ms, checked type compatibility
  4. Graph (Tier 3): 1200ms, found 5 downstream impacts
Confidence: 0.95 (Tier 3 validated)

If investigation needed:
- "Why high confidence?" → See Tier 3 results
- "When was this detected?" → 10:30:15Z
- "What version of code?" → Git commit abc123def456
- "Was this fresh or cached?" → Fresh (cache miss due to upstream change)
```

### 5.3 Debugging Analysis Anomalies

**Scenario**: Conflict detector reports an issue that manual inspection disagrees with

**With Full Provenance**:
```
Question: "Why was this marked as a conflict?"

Answer (from lineage records):
1. Parse stage: File was read at 10:30:00Z, hash X
2. Extract stage: Found 3 symbol modifications
3. Semantic stage: Type inference showed incompatible changes
4. Graph stage: Found 12 downstream callers affected

Investigation path:
- Query: "Show me what the semantic stage found"
- See actual types that were considered
- See which callers were marked as affected
- Trace back to which symbols triggered this
- Find root cause of disagreement

=> Enables accurate tuning of conflict detection
```

### 5.4 Cache Effectiveness Analysis (SC-CACHE-001)

**With Provenance Tracking**:
```
Query: "Why did cache miss for this node?"

Answer:
1. Node was previously cached with hash Y
2. Upstream changed: source file hash X → X'
3. Dependent node's upstream hash changed
4. Cache entry invalidated automatically
5. Re-analysis triggered

This proves:
- Cache invalidation working correctly
- Incremental updates respecting dependencies
- No false cache hits
- System behaving as designed
```

### 5.5 Cross-Repository Dependency Transparency

**T079 Current**:
```
Node "process_payment"
repository_id: "stripe-integration-service"

Can answer: "Where does this come from?"
Cannot answer: "Is this fresh from latest code? When?"
```

**With Full Provenance**:
```
Node "process_payment"
repository_id: "stripe-integration-service"
source_version: SourceVersion {
    source_type: Git,
    version_identifier: "abc123def456",
    version_timestamp: 2026-01-11T08:00:00Z
}
analysis_lineage: [
    LineageRecord {
        operation: "thread_parse",
        input_hash: "file:abc123def456...",
        output_hash: "ast:xyz789...",
        executed_at: 2026-01-11T10:30:00Z
    }
]

Can answer:
- "When was this analyzed?" → 10:30:00Z
- "From which commit?" → abc123def456
- "How long ago?" → 2 hours ago
- "If latest commit is newer, is analysis stale?" → Yes
- "Need to re-analyze?" → Compare version timestamps
```

---

## 6. Implementation Recommendations

### 6.1 Revised T079 Scope

**Current**: "Add repository_id to GraphNode and GraphEdge for source attribution"

**Recommended Scope**: "Implement comprehensive provenance tracking leveraging CocoIndex native capabilities"

**Specific Tasks**:

1. **T079.1**: Create `Provenance` module in `thread-graph/src/provenance.rs`
   - Define `SourceVersion`, `LineageRecord`, `EdgeCreationMethod`
   - Integrate with GraphNode and GraphEdge

2. **T079.2**: Implement `ProvenanceCollector` in `thread-services/src/dataflow/provenance.rs`
   - Intercept CocoIndex ExecutionRecords at each pipeline stage
   - Build complete lineage chains
   - Store in queryable format

3. **T079.3**: Create `ProvenanceStore` trait in `thread-storage/src/provenance.rs`
   - Postgres backend: store lineage in `node_provenance` table
   - D1 backend: similar schema for edge deployment
   - Enable queries like "show me lineage for node X"

4. **T079.4**: Add provenance-aware graph persistence
   - Update `PostgresStorage::store_nodes()` to include provenance
   - Update `D1Storage::store_nodes()` for edge deployment
   - Create migrations: `003_add_provenance_tables.sql`

5. **T079.5**: Implement `ProvenanceQuery` API
   - `get_node_lineage(node_id)` → Full trace
   - `get_analysis_timeline(node_id)` → When was each stage
   - `find_cache_ancestors(node_id)` → What was cached
   - `trace_conflict_detection(conflict_id)` → Full conflict trace

### 6.2 CocoIndex Integration Points for Provenance

**During Dataflow Execution**:

```rust
// In thread-services/src/dataflow/execution.rs

pub async fn execute_code_analysis_flow(
    lib_ctx: &LibContext,
    repo: CodeRepository,
) -> Result<GraphBuildOutput> {
    let flow = build_thread_dataflow_pipeline(&lib_ctx)?;

    // Get execution context with provenance
    let exec_ctx = flow.get_execution_context().await?;

    // Execute with provenance collection
    let result = flow.execute().await?;

    // Extract execution records AFTER each stage
    let source_records = exec_ctx.get_execution_records("local_files_source")?;
    let parse_records = exec_ctx.get_execution_records("thread_parse")?;
    let extract_records = exec_ctx.get_execution_records("thread_extract_symbols")?;
    let graph_records = exec_ctx.get_execution_records("thread_build_graph")?;

    // Combine into lineage chains
    let provenance = build_provenance_from_records(
        source_records,
        parse_records,
        extract_records,
        graph_records,
    )?;

    // Store alongside graph data
    storage.store_nodes_with_provenance(&result.nodes, &provenance)?;

    Ok(result)
}
```

### 6.3 Backward Compatibility

**Concern**: Adding provenance to existing nodes

**Solution**:
- Mark provenance fields as `Option<T>` initially
- Provide migration for existing nodes (backfill with minimal provenance)
- New analyses automatically get full provenance
- Gradually deprecate nodes without provenance

```rust
pub struct GraphNode {
    // ... existing fields ...
    pub repository_id: String,           // Required (from T079)

    // Provenance (initially optional for backward compat)
    pub source_version: Option<SourceVersion>,
    pub source_timestamp: Option<DateTime<Utc>>,
    pub analysis_lineage: Option<Vec<LineageRecord>>,
    pub upstream_hashes: Option<Vec<String>>,
}
```

---

## 7. Missed Opportunities Summary

### 7.1 What T079 Misses

| Missing Feature | CocoIndex Capability | Value |
|---|---|---|
| **Source Version Tracking** | Native SourceVersion tracking | FR-014 completeness |
| **Timestamp Precision** | Per-operation execution times | Audit trail quality |
| **Analysis Pipeline Transparency** | Complete lineage records | Debugging conflicts |
| **Cache Status** | Automatic hit/miss tracking | Cache validation |
| **Incremental Update Efficiency** | Upstream dependency graph | SC-INCR-001/002 |
| **Conflict Detection Audit** | Tier execution records | FR-018 compliance |
| **Stale Analysis Detection** | Version timestamp comparison | Data quality |

### 7.2 Downstream Impact of Current T079

If T079 implemented as-is (repository_id only):

**Problems**:
1. ✗ Can't prove cache is working correctly (missing cache metadata)
2. ✗ Can't audit why conflict was detected (missing tier execution records)
3. ✗ Can't efficiently invalidate caches on upstream change (missing upstream lineage)
4. ✗ Can't determine if analysis is stale (missing source versions)
5. ✗ Doesn't fully satisfy FR-014 (missing version and timestamp)

**Rework Required Later**:
- Phase 1: Implement repository_id (T079 as-is)
- Phase 2: Add source versioning (more work, schema changes)
- Phase 3: Add lineage tracking (significant refactor)
- Phase 4: Add upstream dependencies (impacts incremental update implementation)

**Better Approach**: Implement full provenance once in T079 (slightly more effort now, no rework)

---

## 8. Recommended Implementation Order

### 8.1 Phased Approach to Minimize Risk

**Phase 1: Foundation (Week 1)**
- Implement basic `SourceVersion` struct (Git commit, S3 ETag, local timestamp)
- Add `source_version` and `source_timestamp` fields to GraphNode
- Update T079 scope document

**Phase 2: CocoIndex Integration (Week 2-3)**
- Build `ProvenanceCollector` that extracts ExecutionRecords
- Implement `LineageRecord` structure
- Wire CocoIndex execution data into node storage

**Phase 3: Queryable Provenance (Week 4)**
- Implement `ProvenanceQuery` API
- Add provenance table migrations
- Build debugging tools (show lineage, trace conflicts)

**Phase 4: Validation (Week 5)**
- Verify incremental updates work correctly
- Confirm cache invalidation matches lineage
- Validate conflict audit trail completeness

### 8.2 Parallel Work Streams

**T079.1 + T079.2**: Can happen in parallel
- T079.1: Graph structure changes (module organization)
- T079.2: CocoIndex integration (different crate)

**T079.3**: Depends on T079.1 + T079.2
- Needs provenance data to store

**T079.4**: Depends on T079.3
- Needs schema for persistence

**T079.5**: Depends on all above
- Needs all pieces in place to query

---

## 9. Architecture Diagram: Enhanced Provenance

```
File System / Git / Cloud Source
    │
    ├─ Source: LocalFiles, Git, S3
    │  Provenance: source_type, version_id, timestamp, content_hash
    │
    ▼
CocoIndex Source Executor
    │
    ├─ Tracks: access_time, version, content_hash
    │
    ▼
ThreadParseFunction (CocoIndex SimpleFunctionExecutor)
    │
    ├─ Input: file_id, content_hash (from source)
    │ Output: AST, node_count
    │ Tracks: operation_id, input_hash, output_hash, duration, execution_time
    │
    ▼
ThreadExtractSymbolsFunction
    │
    ├─ Input: AST (from parse)
    │ Output: symbol list
    │ Tracks: input_hash→parse_output, extraction params, duration
    │
    ▼
ThreadRuleMatchFunction
    │
    ├─ Input: AST, symbols
    │ Output: matched rules, conflicts
    │ Tracks: rule_version, matches, confidence scores
    │
    ▼
ThreadBuildGraphFunction
    │
    ├─ Input: symbols, rules
    │ Output: nodes, edges
    │ Tracks: graph_version, node_count, edge_count
    │
    ▼
PostgresTarget / D1Target
    │
    ├─ Write: nodes with full lineage
    │         edges with creation_method
    │ Tracks: write_timestamp, transaction_id, persistence_location
    │
    ▼
Database: nodes, edges, provenance tables
    │
    └─ Query: "Show lineage for node X"
         Answer: Complete trace from source → final node

```

---

## 10. Conclusion and Next Steps

### 10.1 Key Recommendations

1. **Expand T079 Scope** from "repository_id only" to "comprehensive provenance"
   - Still achievable in same timeframe with CocoIndex data
   - Prevents rework and schema changes later
   - Enables full compliance with FR-014

2. **Leverage CocoIndex Native Capabilities**
   - No extra implementation burden (CocoIndex provides automatically)
   - Simpler than building custom lineage tracking
   - Better quality (audited, battle-tested)

3. **Build ProvenanceQuery API Early**
   - Enables debugging and validation
   - Supports incremental update optimization
   - Provides tools for developers and operators

4. **Integrate with Conflict Detection (FR-006, FR-007)**
   - Store tier execution records with conflicts
   - Enable "why was this conflict detected?" questions
   - Build audit trail for FR-018

### 10.2 Impact on Other Features

**Helps**:
- SC-INCR-001/002: Incremental updates can be more precise
- SC-CACHE-001: Cache effectiveness becomes provable
- FR-018: Audit trail and learning from past conflicts
- FR-014: Full compliance (not just repository_id)

**Independent Of**:
- Real-time performance (FR-005, FR-013)
- Conflict prediction accuracy (SC-002)
- Multi-source support (US3)
- Edge deployment (FR-010)

### 10.3 Risk Assessment

**Risk**: Expanding scope increases implementation complexity
**Mitigation**:
- CocoIndex provides most of the data automatically
- Phased approach (foundation → integration → validation)
- Backward compatible with optional fields initially

**Risk**: CocoIndex API changes
**Mitigation**:
- ExecutionRecords API is stable (core dataflow concept)
- Even if API changes, basic capability preserved
- Worst case: store less detailed provenance

**Overall**: Low risk, high value

---

## 11. Research Sources and References

### 11.1 CocoIndex Documentation
- deep-architectural-research.md: Complete CocoIndex architecture analysis
- research.md Task 1: CocoIndex Integration Architecture
- research.md Task 8: Storage Backend Abstraction Pattern

### 11.2 Thread Real-Time Code Graph
- spec.md: FR-014 provenance requirement
- data-model.md: GraphNode, GraphEdge structures
- tasks.md: T079 current scope
- contracts/rpc-types.rs: API definitions

### 11.3 Key Architectural Documents
- CLAUDE.md: Project architecture and CocoIndex integration
- Constitution v2.0.0: Service-library architecture principles

---

**Report Status**: Complete
**Recommendations**: Implement enhanced provenance (T079 expanded) leveraging CocoIndex native capabilities
**Next Step**: Update T079 task scope and create detailed implementation plan
