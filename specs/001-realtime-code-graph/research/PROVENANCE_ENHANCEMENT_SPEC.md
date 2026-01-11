<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: AGPL-3.0-or-later
-->

# Specification: Enhanced Provenance Tracking for Code Graph

**Based on**: PROVENANCE_RESEARCH_REPORT.md
**Scope**: Detailed implementation specification for expanded T079
**Status**: Ready for implementation planning

---

## 1. Data Model Enhancements

### 1.1 New Types for Provenance Module

**Location**: `crates/thread-graph/src/provenance.rs`

```rust
// ============================================================================
// PROVENANCE MODULE: Tracking data lineage and analysis history
// ============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the version of source code being analyzed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceVersion {
    /// Type of source (LocalFiles, Git, S3, GitHub, GitLab, Bitbucket)
    pub source_type: String,

    /// Version-specific identifier
    /// - Git: commit hash (e.g., "abc123def456")
    /// - S3: ETag or version ID
    /// - Local: absolute file path + modification time
    /// - GitHub/GitLab: commit hash or branch with timestamp
    pub version_identifier: String,

    /// When this version existed/was accessed
    /// - Git: commit timestamp
    /// - S3: object version timestamp
    /// - Local: file modification time
    pub version_timestamp: DateTime<Utc>,

    /// Additional context (branch name, tag, storage class, etc.)
    pub metadata: HashMap<String, String>,
}

/// Represents a single step in the analysis pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageRecord {
    /// Operation identifier (e.g., "thread_parse_v0.26.3")
    pub operation_id: String,

    /// Type of operation
    pub operation_type: OperationType,

    /// Content-addressed hash of input data
    pub input_hash: String,

    /// Content-addressed hash of output data
    pub output_hash: String,

    /// When this operation executed
    pub executed_at: DateTime<Utc>,

    /// How long the operation took (milliseconds)
    pub duration_ms: u64,

    /// Whether operation succeeded
    pub success: bool,

    /// Optional error message if failed
    pub error: Option<String>,

    /// Whether output came from cache
    pub cache_hit: bool,

    /// Operation-specific metadata
    pub metadata: HashMap<String, String>,
}

/// Types of operations in the analysis pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    /// Parsing source code to AST
    Parse {
        language: String,
        parser_version: String,
    },

    /// Extracting symbols (functions, classes, etc.)
    ExtractSymbols {
        extractor_version: String,
    },

    /// Matching against rules (pattern matching, linting)
    RuleMatch {
        rules_version: String,
        rule_count: usize,
    },

    /// Extracting relationships (calls, imports, etc.)
    ExtractRelationships {
        extractor_version: String,
    },

    /// Conflict detection at specific tier
    ConflictDetection {
        tier: u8,  // 1, 2, or 3
        detector_version: String,
    },

    /// Building graph structure
    BuildGraph {
        graph_version: String,
    },

    /// Storing to persistent backend
    Store {
        backend_type: String,
        table: String,
    },
}

/// How an edge (relationship) was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeCreationMethod {
    /// Direct AST analysis (e.g., function calls)
    ASTAnalysis {
        confidence: f32,
        analysis_type: String,  // "direct_call", "import", etc.
    },

    /// Semantic analysis (e.g., type inference)
    SemanticAnalysis {
        confidence: f32,
        analysis_type: String,
    },

    /// Inferred from graph structure
    GraphInference {
        confidence: f32,
        inference_rule: String,
    },

    /// Manually annotated
    ExplicitAnnotation {
        annotated_by: String,
        annotated_at: DateTime<Utc>,
    },

    /// From codebase annotations (doc comments, attributes)
    CodeAnnotation {
        annotation_type: String,
    },
}

/// Complete provenance information for a node or edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    /// Which repository this came from
    pub repository_id: String,

    /// Version of source code
    pub source_version: SourceVersion,

    /// When source was accessed
    pub source_access_time: DateTime<Utc>,

    /// Content hash of source file
    pub source_content_hash: String,

    /// Complete pipeline execution trace
    pub analysis_lineage: Vec<LineageRecord>,

    /// Hashes of all upstream data that contributed
    pub upstream_hashes: Vec<String>,

    /// IDs of all sources that contributed
    pub upstream_source_ids: Vec<String>,

    /// Whether any part of analysis came from cache
    pub has_cached_components: bool,

    /// If from cache, when was it cached
    pub cache_timestamp: Option<DateTime<Utc>>,

    /// Overall confidence in this data
    pub confidence: f32,
}

impl Provenance {
    /// Check if analysis is potentially stale
    pub fn is_potentially_stale(&self, max_age: chrono::Duration) -> bool {
        let now = Utc::now();
        (now - self.source_access_time) > max_age
    }

    /// Get the most recent timestamp in the lineage
    pub fn latest_timestamp(&self) -> DateTime<Utc> {
        self.analysis_lineage
            .iter()
            .map(|r| r.executed_at)
            .max()
            .unwrap_or(self.source_access_time)
    }

    /// Count how many pipeline stages contributed to this data
    pub fn pipeline_depth(&self) -> usize {
        self.analysis_lineage.len()
    }

    /// Check if any cache miss occurred
    pub fn has_cache_miss(&self) -> bool {
        self.analysis_lineage.iter().any(|r| !r.cache_hit)
    }
}
```

### 1.2 Updated GraphNode Structure

**Location**: `crates/thread-graph/src/node.rs`

```rust
use crate::provenance::{Provenance, SourceVersion, LineageRecord};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Content-addressed hash of symbol definition
    pub id: NodeId,

    /// Source file containing this symbol
    pub file_id: FileId,

    /// Type of node (function, class, variable, etc.)
    pub node_type: NodeType,

    /// Symbol name
    pub name: String,

    /// Fully qualified name (e.g., "module::Class::method")
    pub qualified_name: String,

    /// Source location (file, line, column)
    pub location: SourceLocation,

    /// Function/type signature
    pub signature: Option<String>,

    /// Language-specific metadata
    pub semantic_metadata: SemanticMetadata,

    // ======== NEW PROVENANCE TRACKING ========

    /// Which repository contains this symbol
    pub repository_id: String,

    /// Version of source code
    pub source_version: Option<SourceVersion>,

    /// Complete provenance information
    pub provenance: Option<Provenance>,

    /// When this node was created/analyzed
    pub analyzed_at: Option<DateTime<Utc>>,

    /// Confidence in this node's accuracy
    pub confidence: f32,
}

impl GraphNode {
    /// Get the full lineage for debugging
    pub fn get_lineage(&self) -> Option<&Vec<LineageRecord>> {
        self.provenance.as_ref().map(|p| &p.analysis_lineage)
    }

    /// Check if this node needs re-analysis
    pub fn should_reanalyze(&self, max_age: chrono::Duration) -> bool {
        self.provenance
            .as_ref()
            .map(|p| p.is_potentially_stale(max_age))
            .unwrap_or(true)  // Default to true if no provenance
    }
}
```

### 1.3 Updated GraphEdge Structure

**Location**: `crates/thread-graph/src/edge.rs`

```rust
use crate::provenance::{EdgeCreationMethod, LineageRecord};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source node ID
    pub source_id: NodeId,

    /// Target node ID
    pub target_id: NodeId,

    /// Type of relationship
    pub edge_type: EdgeType,

    /// Relationship strength (0.0-1.0)
    pub weight: f32,

    /// Optional context about the relationship
    pub context: Option<EdgeContext>,

    // ======== NEW PROVENANCE TRACKING ========

    /// Which repository has this relationship
    pub repository_id: String,

    /// How this edge was created (AST analysis, semantic, etc.)
    pub creation_method: Option<EdgeCreationMethod>,

    /// When this relationship was identified
    pub detected_at: Option<DateTime<Utc>>,

    /// Which conflict detection tier found this (if from conflict analysis)
    pub detected_by_tier: Option<u8>,

    /// Lineage of source node (how it was created)
    pub source_node_lineage: Option<Vec<LineageRecord>>,

    /// Lineage of target node (how it was created)
    pub target_node_lineage: Option<Vec<LineageRecord>>,

    /// Confidence in this relationship
    pub confidence: f32,
}

impl GraphEdge {
    /// Check if both nodes have full provenance
    pub fn has_complete_provenance(&self) -> bool {
        self.source_node_lineage.is_some() && self.target_node_lineage.is_some()
    }

    /// Get the most recent analysis time
    pub fn latest_analysis_time(&self) -> Option<DateTime<Utc>> {
        let source_time = self
            .source_node_lineage
            .as_ref()
            .and_then(|l| l.last())
            .map(|r| r.executed_at);

        let target_time = self
            .target_node_lineage
            .as_ref()
            .and_then(|l| l.last())
            .map(|r| r.executed_at);

        match (source_time, target_time) {
            (Some(s), Some(t)) => Some(s.max(t)),
            (Some(s), None) => Some(s),
            (None, Some(t)) => Some(t),
            (None, None) => self.detected_at,
        }
    }
}
```

### 1.4 Conflict Provenance

**Location**: `crates/thread-conflict/src/provenance.rs`

```rust
use crate::ConflictPrediction;
use crate::provenance::{Provenance, LineageRecord};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictProvenance {
    /// Complete lineage of analysis that detected this conflict
    pub analysis_pipeline: Vec<LineageRecord>,

    /// Results from each detection tier
    pub tier_results: TierResults,

    /// Version of old code that was analyzed
    pub old_code_version: SourceVersion,

    /// Version of new code that was analyzed
    pub new_code_version: SourceVersion,

    /// When the conflict was detected
    pub detection_timestamp: DateTime<Utc>,

    /// Which upstream changes triggered this detection
    pub triggering_changes: Vec<UpstreamChange>,

    /// Whether analysis used cached results
    pub was_cached: bool,

    /// Which cache entries were affected
    pub affected_cache_entries: Vec<String>,

    /// Execution times for each tier
    pub tier_timings: TierTimings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierResults {
    pub tier1_ast: Option<Tier1Result>,
    pub tier2_semantic: Option<Tier2Result>,
    pub tier3_graph: Option<Tier3Result>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tier1Result {
    pub conflicts_found: usize,
    pub confidence: f32,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tier2Result {
    pub conflicts_found: usize,
    pub confidence: f32,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tier3Result {
    pub conflicts_found: usize,
    pub confidence: f32,
    pub execution_time_ms: u64,
    pub affected_nodes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierTimings {
    pub tier1: Option<u64>,
    pub tier2: Option<u64>,
    pub tier3: Option<u64>,
    pub total_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamChange {
    pub changed_node_id: String,
    pub change_type: ChangeType,
    pub previous_hash: String,
    pub new_hash: String,
    pub change_timestamp: DateTime<Utc>,
    pub source_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
}
```

---

## 2. Storage Schema Changes

### 2.1 PostgreSQL Migrations

**Location**: `migrations/postgres/003_add_provenance_tables.sql`

```sql
-- Provenance tracking tables for audit and debugging

-- Source versions (what code versions were analyzed)
CREATE TABLE source_versions (
    id TEXT PRIMARY KEY,
    source_type TEXT NOT NULL,              -- LocalFiles, Git, S3, etc.
    version_identifier TEXT NOT NULL,       -- Commit hash, ETag, path
    version_timestamp TIMESTAMP NOT NULL,   -- When this version existed
    metadata JSONB,                         -- Additional context
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(source_type, version_identifier, version_timestamp)
);

-- Analysis pipeline execution records
CREATE TABLE lineage_records (
    id BIGSERIAL PRIMARY KEY,
    operation_id TEXT NOT NULL,             -- thread_parse_v0.26.3
    operation_type TEXT NOT NULL,           -- Parse, Extract, etc.
    input_hash TEXT NOT NULL,               -- Content-addressed input
    output_hash TEXT NOT NULL,              -- Content-addressed output
    executed_at TIMESTAMP NOT NULL,
    duration_ms BIGINT NOT NULL,
    success BOOLEAN NOT NULL,
    error TEXT,
    cache_hit BOOLEAN NOT NULL,
    metadata JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    INDEX idx_lineage_output_hash (output_hash)
);

-- Node-to-provenance mapping
CREATE TABLE node_provenance (
    node_id TEXT PRIMARY KEY,
    repository_id TEXT NOT NULL,
    source_version_id TEXT NOT NULL REFERENCES source_versions(id),
    source_access_time TIMESTAMP NOT NULL,
    source_content_hash TEXT NOT NULL,
    analysis_pipeline JSONB NOT NULL,      -- Array of lineage_record IDs
    upstream_hashes TEXT[],                 -- Dependencies
    upstream_source_ids TEXT[],
    has_cached_components BOOLEAN,
    cache_timestamp TIMESTAMP,
    confidence FLOAT NOT NULL,
    analyzed_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    FOREIGN KEY (node_id) REFERENCES nodes(id),
    INDEX idx_node_prov_repo (repository_id),
    INDEX idx_node_prov_analyzed (analyzed_at)
);

-- Edge-to-provenance mapping
CREATE TABLE edge_provenance (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    edge_type TEXT NOT NULL,
    repository_id TEXT NOT NULL,
    creation_method JSONB,                  -- AST/Semantic/Graph/Explicit
    detected_at TIMESTAMP,
    detected_by_tier SMALLINT,              -- 1, 2, or 3
    source_node_lineage JSONB,              -- Array of lineage records
    target_node_lineage JSONB,              -- Array of lineage records
    confidence FLOAT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    PRIMARY KEY (source_id, target_id, edge_type),
    FOREIGN KEY (source_id) REFERENCES nodes(id),
    FOREIGN KEY (target_id) REFERENCES nodes(id),
    INDEX idx_edge_prov_created (detected_at)
);

-- Conflict detection provenance
CREATE TABLE conflict_provenance (
    conflict_id TEXT PRIMARY KEY,
    analysis_pipeline JSONB NOT NULL,      -- Complete execution trace
    tier_results JSONB NOT NULL,           -- Tier 1/2/3 results
    old_code_version_id TEXT REFERENCES source_versions(id),
    new_code_version_id TEXT REFERENCES source_versions(id),
    detection_timestamp TIMESTAMP NOT NULL,
    triggering_changes JSONB NOT NULL,     -- Array of upstream changes
    was_cached BOOLEAN,
    affected_cache_entries TEXT[],
    tier_timings JSONB NOT NULL,           -- Execution times
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    FOREIGN KEY (conflict_id) REFERENCES conflicts(id),
    INDEX idx_conflict_prov_detection (detection_timestamp)
);

-- Analysis session provenance
CREATE TABLE session_provenance (
    session_id TEXT PRIMARY KEY,
    execution_records JSONB NOT NULL,      -- All lineage records
    cache_statistics JSONB,                 -- Hit/miss counts
    performance_metrics JSONB,              -- Duration, throughput
    errors_encountered JSONB,               -- Error logs
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    FOREIGN KEY (session_id) REFERENCES analysis_sessions(id)
);
```

**Location**: `migrations/postgres/003_rollback.sql`

```sql
DROP TABLE session_provenance;
DROP TABLE conflict_provenance;
DROP TABLE edge_provenance;
DROP TABLE node_provenance;
DROP TABLE lineage_records;
DROP TABLE source_versions;
```

### 2.2 D1 Schema (Cloudflare Workers)

**Location**: `migrations/d1/003_add_provenance_tables.sql`

```sql
-- Same schema as PostgreSQL, adapted for SQLite/D1 constraints
-- (D1 uses SQLite which has slightly different type system)

CREATE TABLE source_versions (
    id TEXT PRIMARY KEY,
    source_type TEXT NOT NULL,
    version_identifier TEXT NOT NULL,
    version_timestamp TEXT NOT NULL,       -- ISO 8601 string
    metadata TEXT,                          -- JSON as TEXT
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(source_type, version_identifier, version_timestamp)
);

-- Similar tables, all JSON stored as TEXT
-- ... (rest follows same pattern)
```

---

## 3. API Additions

### 3.1 ProvenanceQuery API

**Location**: `crates/thread-api/src/provenance_api.rs`

```rust
use crate::GraphNode;
use crate::provenance::Provenance;
use chrono::DateTime;
use chrono::Utc;

#[async_trait::async_trait]
pub trait ProvenanceQuery {
    /// Get complete lineage for a node
    async fn get_node_lineage(&self, node_id: &str) -> Result<Option<Provenance>>;

    /// Get all nodes created by a specific analysis operation
    async fn get_nodes_by_operation(
        &self,
        operation_id: &str,
    ) -> Result<Vec<(String, Provenance)>>;

    /// Find all nodes that depend on a specific source version
    async fn get_nodes_from_source_version(
        &self,
        source_version_id: &str,
    ) -> Result<Vec<GraphNode>>;

    /// Trace which nodes were invalidated by a source change
    async fn find_affected_nodes(
        &self,
        old_hash: &str,
        new_hash: &str,
    ) -> Result<Vec<String>>;

    /// Get analysis history for a node
    async fn get_analysis_timeline(
        &self,
        node_id: &str,
    ) -> Result<Vec<(DateTime<Utc>, String)>>;  // (time, event)

    /// Check cache effectiveness
    async fn get_cache_statistics(
        &self,
        session_id: &str,
    ) -> Result<CacheStatistics>;

    /// Get conflict detection provenance
    async fn get_conflict_analysis_trace(
        &self,
        conflict_id: &str,
    ) -> Result<Option<ConflictProvenance>>;

    /// Find nodes that haven't been re-analyzed recently
    async fn find_stale_nodes(
        &self,
        max_age: chrono::Duration,
    ) -> Result<Vec<String>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatistics {
    pub total_operations: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub hit_rate: f32,
    pub avg_cache_age: Option<chrono::Duration>,
}
```

### 3.2 RPC Type Extensions

**Update**: `crates/thread-api/src/types.rs`

Add new message types for provenance queries:

```rust
/// Request to get node lineage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLineageRequest {
    pub node_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLineageResponse {
    pub lineage: Option<Provenance>,
    pub query_time_ms: u64,
}

/// Request to trace conflict detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceConflictRequest {
    pub conflict_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceConflictResponse {
    pub trace: Option<ConflictProvenance>,
    pub analysis_stages: Vec<String>,  // Which stages ran
    pub query_time_ms: u64,
}
```

---

## 4. Implementation Tasks (Updated T079)

### 4.1 Core Tasks

**T079.1: Provenance Module Creation**
- File: `crates/thread-graph/src/provenance.rs`
- Define: `SourceVersion`, `LineageRecord`, `OperationType`, `EdgeCreationMethod`, `Provenance`
- Tests: Unit tests for provenance type conversions
- Time estimate: 3-4 hours

**T079.2: GraphNode/GraphEdge Updates**
- File: `crates/thread-graph/src/node.rs` and `edge.rs`
- Add provenance fields (with `Option<T>` for backward compat)
- Implement helper methods (`get_lineage`, `should_reanalyze`, etc.)
- Tests: Serialization tests, schema validation
- Time estimate: 2-3 hours

**T079.3: Conflict Provenance Module**
- File: `crates/thread-conflict/src/provenance.rs`
- Define: `ConflictProvenance`, `TierResults`, `UpstreamChange`
- Link to conflict detection results
- Time estimate: 2-3 hours

**T079.4: Database Schema & Migrations**
- Files: `migrations/postgres/003_*.sql` and `migrations/d1/003_*.sql`
- Create: All provenance tables
- Implement: Migration runner logic
- Tests: Schema validation
- Time estimate: 3-4 hours

**T079.5: Storage Implementation**
- Files: `crates/thread-storage/src/{postgres,d1}.rs`
- Implement: `ProvenanceStore` trait (new file: `src/provenance.rs`)
- Add: Node/edge persistence with provenance
- Add: Lineage record insertion
- Tests: Integration tests with real database
- Time estimate: 4-5 hours

**T079.6: Provenance Query API**
- File: `crates/thread-api/src/provenance_api.rs` (new file)
- Implement: `ProvenanceQuery` trait methods
- Add: Query handler implementations
- Tests: Query correctness, performance
- Time estimate: 5-6 hours

**T079.7: CocoIndex Integration**
- File: `crates/thread-services/src/dataflow/provenance_collector.rs` (new)
- Create: `ProvenanceCollector` that extracts ExecutionRecords
- Wire: Collection during flow execution
- Tests: End-to-end provenance flow
- Time estimate: 5-6 hours

**T079.8: Documentation & Examples**
- Update: `crates/thread-graph/src/lib.rs` documentation
- Add: Examples of provenance queries
- Create: Debugging guide ("How to trace why a conflict was detected?")
- Time estimate: 2-3 hours

### 4.2 Total Effort Estimate

- **Low estimate**: 25 hours (1 week, 3 days implementation)
- **High estimate**: 35 hours (1 week, 4 days full completion with tests)
- **Recommended**: Schedule for Sprint 2-3 (after T001-T032 foundation)

### 4.3 Dependency Graph

```
T079.1 (Provenance types)
    ↓
T079.2 (GraphNode/Edge updates) ← Depends on T079.1
    ↓
T079.3 (Conflict provenance)    ← Can parallel with T079.2
    ↓
T079.4 (Migrations)             ← Depends on T079.2
    ↓
T079.5 (Storage)                ← Depends on T079.4
    ↓
T079.6 (Query API)              ← Depends on T079.5
    ↓
T079.7 (CocoIndex integration)  ← Depends on T001-T032 AND T079.5
    ↓
T079.8 (Documentation)          ← Depends on all above
```

---

## 5. Backward Compatibility Strategy

### 5.1 Phased Rollout

**Phase 1: Optional Provenance**
- All provenance fields are `Option<T>`
- Existing nodes continue to work
- New analyses automatically include provenance
- No schema change required immediately

**Phase 2: Migration**
- Backfill historical nodes (lazy evaluation)
- Run migration script: `scripts/backfill_provenance.sql`
- Generates minimal provenance for existing nodes

**Phase 3: Required Provenance**
- After Phase 2, make provenance required
- All queries validate provenance present
- Better audit trail and debugging

### 5.2 Migration Script

**Location**: `scripts/backfill_provenance.sql`

```sql
-- For each existing node without provenance:
-- 1. Assume it came from initial analysis
-- 2. Create minimal source_version record
-- 3. Create minimal lineage (single "legacy_analysis" record)
-- 4. Link via node_provenance

INSERT INTO source_versions (
    id, source_type, version_identifier, version_timestamp
)
SELECT
    'legacy:' || n.file_id,
    'unknown',
    n.file_id,
    n.created_at
FROM nodes n
WHERE NOT EXISTS (
    SELECT 1 FROM node_provenance WHERE node_id = n.id
);

-- ... rest of migration
```

---

## 6. Success Validation

### 6.1 Metrics to Track

- **Completeness**: % of nodes with full provenance (target: 100% for new analyses)
- **Query Performance**: Latency of `get_node_lineage()` (target: <10ms)
- **Cache Effectiveness**: Hit rate improvement from detailed upstream tracking (target: >90%)
- **Debugging Utility**: Developer satisfaction with provenance queries (qualitative)

### 6.2 Test Scenarios

**Scenario 1: Basic Provenance**
- Parse a file
- Store node with provenance
- Query: Retrieve complete lineage
- Verify: All stages present, timestamps match

**Scenario 2: Conflict Audit**
- Detect a conflict
- Store with conflict provenance
- Query: Get analysis trace for conflict
- Verify: All tiers documented, timing correct

**Scenario 3: Incremental Update**
- Change one source file
- Use provenance to identify affected nodes
- Re-analyze only affected nodes
- Verify: Cache hits for unaffected nodes

**Scenario 4: Cross-Repository**
- Index two repositories
- Query provenance for cross-repo dependency
- Verify: Both source versions tracked

---

## 7. Recommended Rollout Timeline

**Week 1**:
- T079.1-T079.3: Define all provenance types (parallel)
- Code review and approval

**Week 2**:
- T079.4-T079.5: Database and storage (sequential)
- Integration testing

**Week 3**:
- T079.6: Query API (depends on storage completion)
- API testing

**Week 4**:
- T079.7: CocoIndex integration (depends on foundation complete)
- End-to-end testing

**Week 5**:
- T079.8: Documentation and cleanup
- QA and validation

---

## 8. Risk Mitigation

**Risk**: Schema changes impact existing deployments
**Mitigation**: Use optional fields + lazy migration approach

**Risk**: Performance impact of storing/querying provenance
**Mitigation**: Proper indexing, async operations, caching

**Risk**: CocoIndex execution record API changes
**Mitigation**: Abstract collection layer, handle API differences

**Risk**: Feature creep (too much provenance data)
**Mitigation**: Track only essential metadata, keep payloads compact

---

**Status**: Ready for implementation
**Next Step**: Schedule T079 expansion in project planning
**Contact**: Reference PROVENANCE_RESEARCH_REPORT.md for background
