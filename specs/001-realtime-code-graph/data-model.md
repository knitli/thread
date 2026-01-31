<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: AGPL-3.0-or-later
-->

# Data Model: Real-Time Code Graph Intelligence

**Feature Branch**: `001-realtime-code-graph`  
**Phase**: Phase 1 - Design & Contracts  
**Last Updated**: 2026-01-11

## Overview

This document defines the core entities, relationships, and data structures for the Real-Time Code Graph Intelligence system. The data model supports both persistent storage (Postgres/D1) and in-memory operations (petgraph), with content-addressed caching via CocoIndex.

## Core Entities

### 1. Code Repository

**Purpose**: Represents a source of code (Git repo, local directory, cloud storage)

**Attributes**:
```rust
pub struct CodeRepository {
    pub id: RepositoryId,           // Content-addressed hash of repo metadata
    pub source_type: SourceType,    // Git, Local, S3, GitHub, GitLab
    pub connection: ConnectionConfig, // Credentials, URL, auth tokens
    pub sync_frequency: Duration,   // How often to poll for changes
    pub last_sync: DateTime<Utc>,   // Last successful sync timestamp
    pub branch: String,             // Primary branch to index (e.g., "main")
    pub file_patterns: Vec<String>, // Glob patterns for files to index
}

pub enum SourceType {
    Git { url: String, credentials: Option<GitCredentials> },
    Local { path: PathBuf },
    S3 { bucket: String, prefix: String, credentials: S3Credentials },
    GitHub { owner: String, repo: String, token: String },
    GitLab { project: String, token: String },
}
```

**Relationships**:
- One-to-many with `CodeFile` (repository contains many files)
- One-to-many with `AnalysisSession` (repository analyzed multiple times)

**Storage**: Postgres/D1 table `repositories`

---

### 2. Code File

**Purpose**: Individual file in a repository with AST representation

**Attributes**:
```rust
pub struct CodeFile {
    pub id: FileId,                 // Content-addressed hash of file content
    pub repository_id: RepositoryId, // Parent repository
    pub file_path: PathBuf,         // Relative path from repository root
    pub language: Language,         // Rust, TypeScript, Python, etc. (from thread-language)
    pub content_hash: ContentHash,  // SHA-256 hash of file content
    pub ast: Root,                  // AST from thread-ast-engine
    pub last_modified: DateTime<Utc>, // File modification timestamp
    pub size_bytes: u64,            // File size for indexing metrics
}

pub type FileId = String;           // Format: "sha256:{hash}"
pub type ContentHash = [u8; 32];    // SHA-256 hash
```

**Relationships**:
- Many-to-one with `CodeRepository` (file belongs to one repository)
- One-to-many with `GraphNode` (file contains multiple symbols)
- Many-to-many with `ConflictPrediction` (file can have multiple conflicts)

**Storage**: 
- Metadata: Postgres/D1 table `files`
- AST: Content-addressed cache (CocoIndex) with file hash as key
- Content: Not stored (re-fetched from source on demand)

---

### 3. Graph Node

**Purpose**: Represents a code symbol (function, class, variable, type) in the graph

**Attributes**:
```rust
pub struct GraphNode {
    pub id: NodeId,                 // Content-addressed hash of symbol definition
    pub file_id: FileId,            // Source file containing this symbol
    pub node_type: NodeType,        // FILE, CLASS, METHOD, FUNCTION, VARIABLE, etc.
    pub name: String,               // Symbol name (e.g., "processPayment")
    pub qualified_name: String,     // Fully qualified (e.g., "module::Class::method")
    pub location: SourceLocation,   // File path, line, column
    pub signature: Option<String>,  // Function signature, type definition
    pub semantic_metadata: SemanticMetadata, // Language-specific analysis
}

pub type NodeId = String;           // Format: "node:{content_hash}"

pub enum NodeType {
    File,
    Module,
    Class,
    Interface,
    Method,
    Function,
    Variable,
    Constant,
    Type,
    Import,
}

pub struct SourceLocation {
    pub file_path: PathBuf,
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

pub struct SemanticMetadata {
    pub visibility: Visibility,     // Public, Private, Protected
    pub mutability: Option<Mutability>, // Mutable, Immutable (Rust-specific)
    pub async_fn: bool,             // Is async function?
    pub generic_params: Vec<String>, // Generic type parameters
    pub attributes: HashMap<String, String>, // Language-specific attributes
}
```

**Relationships**:
- Many-to-one with `CodeFile` (node belongs to one file)
- Many-to-many with `GraphEdge` (node participates in many relationships)
- One-to-many with `ConflictPrediction` (node can be source of conflicts)

**Storage**:
- Metadata: Postgres/D1 table `nodes`
- In-memory: `petgraph` node for complex queries (CLI only)
- Edge Strategy: **Streaming/Iterator access only**. NEVER load full graph into memory. Use `D1GraphIterator` pattern.
- Cache: CocoIndex with node ID as key

---

### 4. Graph Edge

**Purpose**: Represents a relationship between code symbols

**Attributes**:
```rust
pub struct GraphEdge {
    pub source_id: NodeId,          // From node
    pub target_id: NodeId,          // To node
    pub edge_type: EdgeType,        // Relationship kind
    pub weight: f32,                // Relationship strength (1.0 default)
    pub context: EdgeContext,       // Additional context about relationship
}

pub enum EdgeType {
    Contains,       // FILE → CLASS, CLASS → METHOD (hierarchical)
    Calls,          // FUNCTION → FUNCTION (execution flow)
    Inherits,       // CLASS → CLASS (inheritance)
    Implements,     // CLASS → INTERFACE (interface implementation)
    Uses,           // METHOD → VARIABLE (data dependency)
    Imports,        // FILE → FILE (module dependency)
    TypeDependency, // TYPE → TYPE (type system dependency)
}

pub struct EdgeContext {
    pub call_site: Option<SourceLocation>, // Where relationship occurs
    pub conditional: bool,          // Relationship is conditional (e.g., if statement)
    pub async_context: bool,        // Relationship crosses async boundary
}
```

**Relationships**:
- Many-to-one with `GraphNode` (edge connects two nodes)
- Edges form the graph structure for traversal queries

**Storage**:
- Postgres/D1 table `edges` with composite primary key `(source_id, target_id, edge_type)`
- Indexed on `source_id` and `target_id` for fast traversal
- In-memory: `petgraph` edges (CLI only)

---

### Edge-Specific Optimizations (D1)

To overcome D1's single-threaded nature and Workers' memory limits, we utilize a **Reachability Index**.

**Reachability Table (Transitive Closure)**:
Stores pre-computed "impact" paths to allow O(1) lookups for conflict detection without recursion.

```rust
// Table: reachability
pub struct ReachabilityEntry {
    pub ancestor_id: NodeId,    // Upstream node (e.g., modified function)
    pub descendant_id: NodeId,  // Downstream node (e.g., affected API)
    pub hops: u32,              // Distance
    pub path_hash: u64,         // Hash of the path taken (for updates)
}
```

**Reachability Logic**:
- **Write Path**: `ThreadBuildGraphFunction` computes transitive closure for changed nodes and performs `BATCH INSERT` into D1.
- **Read Path**: Conflict detection runs `SELECT descendant_id FROM reachability WHERE ancestor_id = ?` (single fast query).
- **Maintenance**: Incremental updates only recalculate reachability for the changed subgraph.

---

### 5. Conflict Prediction

**Purpose**: Represents a detected potential conflict between concurrent code changes

**Attributes**:
```rust
pub struct ConflictPrediction {
    pub id: ConflictId,             // Unique conflict identifier
    pub detection_time: DateTime<Utc>, // When conflict was detected
    pub affected_files: Vec<FileId>, // Files involved in conflict
    pub conflicting_developers: Vec<UserId>, // Developers whose changes conflict
    pub conflict_type: ConflictType, // Kind of conflict
    pub severity: Severity,         // Impact severity rating
    pub confidence: f32,            // Detection confidence (0.0-1.0)
    pub tier: DetectionTier,        // Which tier detected it (AST/Semantic/Graph)
    pub suggested_resolution: Option<ResolutionStrategy>, // AI-suggested fix
    pub status: ConflictStatus,     // Unresolved, Acknowledged, Resolved
}

pub type ConflictId = String;       // Format: "conflict:{hash}"
pub type UserId = String;

pub enum ConflictType {
    SignatureChange,    // Function signature modified
    Deletion,           // Symbol deleted
    BreakingAPIChange,  // API contract broken
    ConcurrentEdit,     // Same symbol edited by multiple developers
    SemanticConflict,   // Different edits with semantic incompatibility
    DependencyConflict, // Conflicting dependency versions
}

pub enum Severity {
    Low,        // Minor issue, easy to resolve
    Medium,     // Requires attention, may block merge
    High,       // Critical issue, definitely blocks merge
    Critical,   // System-breaking change
}

pub enum DetectionTier {
    Tier1AST,       // Fast AST diff (<100ms)
    Tier2Semantic,  // Semantic analysis (<1s)
    Tier3GraphImpact, // Comprehensive graph analysis (<5s)
}

pub struct ResolutionStrategy {
    pub description: String,        // Human-readable explanation
    pub automated_fix: Option<CodePatch>, // Machine-applicable patch
    pub alternative_approaches: Vec<String>, // Other resolution options
    pub reasoning: String,          // Why this strategy is suggested
}

pub enum ConflictStatus {
    Unresolved,
    Acknowledged { by: UserId, at: DateTime<Utc> },
    Resolved { by: UserId, at: DateTime<Utc>, strategy: String },
}
```

**Relationships**:
- Many-to-many with `CodeFile` (conflict affects multiple files)
- Many-to-many with `GraphNode` (conflict involves multiple symbols)
- One-to-one with `AnalysisSession` (conflict detected during specific analysis)

**Storage**:
- Postgres/D1 table `conflicts`
- Audit log: Separate `conflict_history` table for learning

---

### 6. Analysis Session

**Purpose**: Represents a single analysis run (full or incremental)

**Attributes**:
```rust
pub struct AnalysisSession {
    pub id: SessionId,              // Unique session identifier
    pub repository_id: RepositoryId, // Repository being analyzed
    pub session_type: SessionType,  // Full, Incremental, OnDemand
    pub start_time: DateTime<Utc>,  // Session start
    pub completion_time: Option<DateTime<Utc>>, // Session end (None if running)
    pub files_analyzed: u32,        // Count of files processed
    pub nodes_created: u32,         // Graph nodes added
    pub edges_created: u32,         // Graph edges added
    pub conflicts_detected: u32,    // Conflicts found
    pub cache_hit_rate: f32,        // Percentage of cache hits (0.0-1.0)
    pub errors: Vec<AnalysisError>, // Errors encountered during analysis
    pub metrics: PerformanceMetrics, // Performance statistics
}

pub type SessionId = String;        // Format: "session:{timestamp}:{hash}"

pub enum SessionType {
    FullAnalysis,       // Complete repository scan
    IncrementalUpdate,  // Only changed files
    OnDemand,           // User-triggered analysis
}

pub struct PerformanceMetrics {
    pub parsing_time_ms: u64,       // Total AST parsing time
    pub indexing_time_ms: u64,      // Graph construction time
    pub storage_time_ms: u64,       // Database write time
    pub cache_lookups: u32,         // Cache query count
    pub cache_hits: u32,            // Cache hit count
}
```

**Relationships**:
- Many-to-one with `CodeRepository` (session analyzes one repository)
- One-to-many with `ConflictPrediction` (session detects multiple conflicts)

**Storage**:
- Postgres/D1 table `analysis_sessions`
- Metrics aggregated for dashboard/reporting

---

### 7. Plugin Engine

**Purpose**: Represents a pluggable analysis component (parser, graph builder, conflict detector)

**Attributes**:
```rust
pub struct PluginEngine {
    pub id: EngineId,               // Unique engine identifier
    pub engine_type: EngineType,    // Parser, GraphBuilder, ConflictDetector
    pub name: String,               // Human-readable name
    pub version: String,            // Semantic version (e.g., "1.0.0")
    pub configuration: EngineConfig, // Engine-specific parameters
    pub enabled: bool,              // Is this engine active?
}

pub type EngineId = String;         // Format: "engine:{type}:{name}"

pub enum EngineType {
    Parser { language: Language },  // AST parsing engine (thread-ast-engine)
    GraphBuilder,                   // Graph construction engine
    ConflictDetector { tier: u8 },  // Conflict detection engine (1, 2, or 3)
    SemanticAnalyzer,               // Semantic analysis engine (CodeWeaver?)
}

pub struct EngineConfig {
    pub params: HashMap<String, serde_json::Value>, // Key-value configuration
    pub enabled_languages: Vec<Language>, // Languages this engine supports
    pub performance_tuning: PerformanceTuning, // Resource limits
}

pub struct PerformanceTuning {
    pub max_file_size_mb: u32,      // Skip files larger than this
    pub timeout_seconds: u32,       // Timeout per-file analysis
    pub parallel_workers: u32,      // Parallelism level
}
```

**Relationships**:
- Many-to-many with `AnalysisSession` (session uses multiple engines)
- Engines are swappable via trait boundaries (Constitution Principle IV)

**Storage**:
- Postgres/D1 table `plugin_engines`
- Configuration managed via admin API or config files

---

## Entity Relationships Diagram

```
CodeRepository (1) ────< (many) CodeFile
      │                           │
      │                           └──> (many) GraphNode ──┐
      │                                       │            │
      ▼                                       ▼            ▼
AnalysisSession ───> ConflictPrediction    GraphEdge ────┘
      │                   │
      └───> PluginEngine  └───> (many) CodeFile
```

## Content-Addressed Storage Strategy

**CocoIndex Integration**:
- All entities use content-addressed IDs (SHA-256 hashes)
- Content changes → new ID → automatic cache invalidation
- Incremental updates: diff old vs new IDs, update only changed nodes/edges
- Cache key format: `{entity_type}:{content_hash}`

**Cache Hit Rate Target**: >90% (SC-CACHE-001)

**Example**:
```rust
// Function signature changes
let old_id = NodeId::from_content("fn process(x: i32)");  // "node:abc123..."
let new_id = NodeId::from_content("fn process(x: String)"); // "node:def456..." (different!)

// CocoIndex detects change, invalidates cache for old_id
cocoindex.invalidate(&old_id)?;

// Only new_id node and affected edges need re-analysis
db.update_node(&new_id)?;
db.update_edges_referencing(&old_id, &new_id)?;
```

## Schema Migrations

**Version 1** (Initial Schema):
- Tables: `repositories`, `files`, `nodes`, `edges`, `conflicts`, `analysis_sessions`, `plugin_engines`
- Indexes: `idx_edges_source`, `idx_edges_target`, `idx_nodes_type_name`, `idx_nodes_file`
- Schema version tracked in `schema_version` table

**Future Migrations**:
- Version 2: Add materialized views for reverse dependencies
- Version 3: Add partitioning for large-scale deployments (>10M nodes)
- Version 4: Add audit logging for conflict resolutions

---

## Validation Rules

1. **Content Hashing**: All IDs derived from content SHA-256 hashes (deterministic)
2. **Graph Consistency**: Edges must reference existing nodes (foreign key constraints)
3. **File Uniqueness**: One file per (repository_id, file_path) pair
4. **Node Location**: Node source location must exist in parent file AST
5. **Conflict Status**: Conflicts can only move Unresolved → Acknowledged → Resolved (state machine)
6. **Cache Coherence**: Content change invalidates all downstream caches

---

## Next Steps (Phase 2 - tasks.md)

Based on this data model:
1. Implement Rust struct definitions in appropriate crates
2. Generate database migration SQL for Postgres and D1
3. Implement CocoIndex content-addressing for all entities
4. Write contract tests for entity invariants
5. Create database indexes for performance targets (SC-STORE-001)
