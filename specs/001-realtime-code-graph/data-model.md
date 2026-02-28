<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: AGPL-3.0-or-later
-->

# Data Model: Real-Time Code Graph Intelligence

**Feature Branch**: `001-realtime-code-graph`
**Phase**: Phase 1 - Design & Contracts
**Last Updated**: 2026-02-24

## Overview

This document defines the core entities, relationships, and data structures for the Real-Time Code Graph Intelligence system. The data model supports both persistent storage (Postgres/D1) and in-memory operations, with content-addressed caching via ReCoco.

## Implementation Status (as of 2026-02-24)

The `thread-flow` crate provides a foundational (minimal) data model. The rich semantic
model defined below is the target state — not all entities are implemented yet.

| Entity / Field | Status | Notes |
|---|---|---|
| `CodeRepository` | Not implemented | Planned for multi-source indexing (US3) |
| `CodeFile` | Not implemented | thread-flow tracks file paths + fingerprints only |
| `GraphNode` (rich) | Not implemented | thread-flow has `DependencyEdge` (minimal) |
| `GraphNode.node_type` | Replaced | Retired — replaced by `semantic_class` (see thread-definitions) |
| `GraphNode.semantic_class` | Not implemented | Planned (thread-definitions prerequisite, T-C02) |
| `GraphNode.node_kind` | Not implemented | Planned (populated during AST walk in thread-flow classify operator, T-C10) |
| `GraphEdge` (rich) | Not implemented | thread-flow has `DependencyEdge` with Import/Export/Macro/Type/Trait types |
| `ConflictPrediction` | Not implemented | Planned for conflict detection (US2) |
| `AnalysisSession` | Not implemented | Planned for observability layer |
| `PluginEngine` | Not implemented | Architecture planned but not built |

### Implemented Foundation (thread-flow)

```rust
// Already in crates/flow/src/incremental/types.rs
pub struct AnalysisDefFingerprint {
    pub source_files: HashSet<PathBuf>,  // Set of source files contributing to this fingerprint
    pub fingerprint: Fingerprint,        // Blake3 hash
    pub last_analyzed: Option<i64>,      // Unix timestamp of last analysis
}

pub struct DependencyEdge {
    pub from_file: PathBuf,
    pub to_file: PathBuf,
    pub dep_type: DependencyType,
    pub symbol_dependency: Option<SymbolDependency>,
    pub strength: DependencyStrength,
}

pub enum DependencyType { Import, Export, Macro, Type, Trait }
pub enum DependencyStrength { Strong, Weak }
```

## Core Entities

### 1. Code Repository

**Purpose**: Represents a source of code (Git repo, local directory, cloud storage)

**Attributes**:
```rust
pub struct CodeRepository {
    pub id: RepositoryId,           // Content-addressed hash of repo metadata
    pub source_type: SourceType,    // Git, Local, S3, GitHub, GitLab
    pub connection_ref: CredentialRef, // Reference to credentials in secrets store (never the credential itself)
    pub sync_frequency: Duration,   // How often to poll for changes
    pub last_sync: DateTime<Utc>,   // Last successful sync timestamp
    pub branch: String,             // Primary branch to index (e.g., "main")
    pub file_patterns: Vec<String>, // Glob patterns for files to index
}

/// Describes WHERE and HOW to access a code source.
/// Credentials are never embedded here — they are always resolved at runtime
/// via `CodeRepository.connection_ref` (a `CredentialRef` lookup into the secrets store).
pub enum SourceType {
    Git { url: String },                           // Credentials: connection_ref → GitCredentials
    Local { path: PathBuf },                       // No credentials required
    S3 { bucket: String, prefix: String },         // Credentials: connection_ref → S3Credentials
    GitHub { owner: String, repo: String },        // Token: connection_ref → GitHub PAT or App token
    GitLab { project: String },                    // Token: connection_ref → GitLab PAT
}

/// Opaque reference to connection credentials stored in an external secrets manager.
/// The credentials themselves are never persisted with the entity.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CredentialRef {
    pub store: CredentialStore,  // Which secrets backend holds this credential
    pub key: Box<str>,           // Lookup key within that store
}

pub enum CredentialStore {
    EnvVar,           // CLI: environment variable name
    SystemKeychain,   // CLI: OS keychain reference
    CloudflareSecret, // Edge: Cloudflare Workers secret binding name
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
    pub content_hash: ContentHash,  // Blake3 hash of file content
    pub last_modified: DateTime<Utc>, // File modification timestamp
    pub size_bytes: u64,            // File size for indexing metrics
    // NOTE: `ast: Root` is intentionally ABSENT. tree-sitter's `Tree`/`Root` is an opaque
    // C struct — not serializable, not persistable, and not Send. AST is obtained on demand
    // via `tree_sitter_parse(source_bytes, language)`. For frequently-accessed files, use a
    // separate AstCache (e.g., LRU thread_utils::RapidMap<ContentHash, Root>) owned by the analysis session,
    // never stored in CodeFile itself.
}

/// Newtype wrapper for file identifiers. Prevents accidental substitution of
/// other string-typed IDs. Format: `"blake3:{hash}"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct FileId(Box<str>);
impl FileId {
    pub fn as_str(&self) -> &str { &self.0 }
}
pub type ContentHash = [u8; 32];    // Blake3 hash
```

**Relationships**:

- Many-to-one with `CodeRepository` (file belongs to one repository)
- One-to-many with `GraphNode` (file contains multiple symbols)
- Many-to-many with `ConflictPrediction` (file can have multiple conflicts)

**Storage**:

- Metadata: Postgres/D1 table `files`
- AST: Content-addressed cache (ReCoco) with file hash as key
- Content: Not stored (re-fetched from source on demand)

---

### 3. Graph Node

**Purpose**: Represents a code symbol (function, class, variable, type) in the graph

> **SemanticClass** is defined in the `thread-definitions` crate (`crates/definitions/`). See
> `docs/architecture/SEMANTIC_CLASSIFICATION_SPEC.md` for the complete enum, importance tier
> definitions, and scoring system.

**Attributes**:
```rust
pub struct GraphNode {
    pub id: NodeId,                 // Content-addressed hash of symbol definition
    pub file_id: FileId,            // Source file containing this symbol
    pub semantic_class: SemanticClass, // Language-agnostic 22-category classification
                                    // (from thread-definitions crate). Replaces the
                                    // former node_type enum. Carries importance tier
                                    // and scoring. Examples:
                                    //   DefinitionCallable  — functions, methods, constructors
                                    //   DefinitionType      — classes, structs, enums, traits
                                    //   BoundaryModule      — imports, exports, module declarations
    pub node_kind: Option<Box<str>>, // Raw tree-sitter node type name for finer structural
                                    // distinctions when SemanticClass loses relevant detail.
                                    // Examples:
                                    //   "function_item" vs "closure_expression"
                                    //       (both DefinitionCallable in Rust)
                                    //   "impl_item"
                                    //       (DefinitionType container for Rust impl blocks)
                                    //   "enum_variant"
                                    //       (DefinitionCallable in Rust, per rust.toml override)
                                    // Populated from tree-sitter parse; None for nodes derived
                                    // from higher-level analysis.
    pub name: String,               // Symbol name (e.g., "processPayment")
    pub qualified_name: String,     // Fully qualified (e.g., "module::Class::method")
    pub location: SourceLocation,   // File path, line, column
    pub signature: Option<String>,  // Function signature, type definition
    pub semantic_metadata: SemanticMetadata, // Language-specific analysis
}

/// `NodeId` is a content-addressed identifier for a code symbol.
///
/// **Hash composition** (Decision D6):
///   `blake3(file_path_bytes || qualified_name_bytes || normalized_signature_bytes)`
///
/// - `file_path_bytes`: ensures uniqueness across files for identical function names
/// - `qualified_name_bytes`: captures renames (fn process → fn handle_payment = new NodeId)
/// - `normalized_signature_bytes`: whitespace-stripped, formatting-invariant representation
///
/// **Invariant**: Provenance/analysis metadata (timestamps, sources, confidence scores,
/// branch refs) are NOT included in the hash. Same symbol content = same NodeId,
/// regardless of when or how it was analyzed.
///
/// **Property test required**: `hash(same_content) == hash(same_content)` always holds.
/// Any change to normalization logic MUST re-verify all 27 validated languages.
///
/// Newtype wrapper for node identifiers. Prevents accidental substitution of
/// other string-typed IDs. Format: `"node:{blake3_hex}"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct NodeId(Box<str>);
impl NodeId {
    pub fn as_str(&self) -> &str { &self.0 }
}
```

> **NodeType (retired)**: Previously defined as
> `FILE | CLASS | METHOD | FUNCTION | VARIABLE | IMPORT | EXPORT | TYPE | MODULE | INTERFACE | TRAIT | IMPL | ENUM | CONST | TEST`.
> Replaced by `SemanticClass` from the `thread-definitions` crate.
> See `docs/architecture/SEMANTIC_CLASSIFICATION_SPEC.md` for the 22-variant `SemanticClass` enum.

```rust
pub struct SourceLocation {
    pub file_path: PathBuf,
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

pub struct SemanticMetadata {
    pub visibility: Visibility,          // Public, Private, Protected (language-agnostic)
    pub generic_params: Vec<String>,     // Generic type parameters (language-agnostic)
    pub attributes: thread_utils::RapidMap<Box<str>, serde_json::Value>, // Language-specific metadata.
    // Documented attribute keys:
    //   "mutability"  → bool   — Rust: mutable binding or field
    //   "async"       → bool   — Rust/JS/Python/Go: async function
    //   "unsafe"      → bool   — Rust: unsafe fn or block
    //   "abstract"    → bool   — Java/C#/Python: abstract method
    //   "static"      → bool   — Java/C#/JS: static member
    //   "override"    → bool   — Java/C#/Kotlin: overriding method
    //   "throws"      → [str]  — Java: checked exception types
    //   "decorators"  → [str]  — Python/TS: decorator names
    // Keys follow snake_case convention. New keys may be added per language without schema migration.
}
```

**Relationships**:

- Many-to-one with `CodeFile` (node belongs to one file)
- Many-to-many with `GraphEdge` (node participates in many relationships)
- One-to-many with `ConflictPrediction` (node can be source of conflicts)

**Storage**:

- Metadata: Postgres/D1 table `nodes`
- In-memory: Custom DependencyGraph (crates/flow/src/incremental/graph.rs) for complex queries (CLI only) — petgraph was evaluated but custom implementation was chosen
- Edge Strategy: **Streaming/Iterator access only**. NEVER load full graph into memory. Use `D1GraphIterator` pattern.
- Cache: ReCoco with node ID as key
- Vector Embeddings: Cloudflare Vectorize (edge deployment), Qdrant (CLI-only, optional — currently blocked)

---

### 4. Graph Edge

**Purpose**: Represents a relationship between code symbols

> **Edge types vs. node classification**: `GraphEdge.edge_type` (Contains, Calls, Inherits,
> Implements, Uses, Imports, TypeDependency) describes structural/semantic *relationships*
> between nodes. These are distinct from and complementary to `GraphNode.semantic_class`,
> which describes what a node *is*. Edge types are determined by language-specific analysis;
> `semantic_class` is determined by the language-agnostic classifier.

**Attributes**:

```rust
pub struct GraphEdge {
    pub id: EdgeId,                 // Content-addressed edge identifier
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

/// Content-addressed edge identifier. Derived from source_id + target_id + edge_type.
/// Format: `"edge:{blake3_hex}"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct EdgeId(Box<str>);
impl EdgeId {
    pub fn as_str(&self) -> &str { &self.0 }
}
```

**Relationships**:

- Many-to-one with `GraphNode` (edge connects two nodes)
- Edges form the graph structure for traversal queries

**Storage**:

- Postgres/D1 table `edges` with `id` (`EdgeId`) as primary key; composite index `(source_id, target_id, edge_type)` for uniqueness
- Indexed on `source_id` and `target_id` for fast traversal
- In-memory: DependencyGraph adjacency lists (thread-flow) — custom BFS/topological sort
- `GraphUpdate` WebSocket messages reference `EdgeId` values in `added_edges` and `removed_edges` fields

---

### Edge-Specific Optimizations (D1)

To enable O(1) reachability lookups without recursive queries, we maintain a **Reachability Index**.

#### Overlay Graph Query Default (FR-017)

**Query default**: `include_local_delta` defaults to `true`. Queries without this parameter return the merged Base + Delta view. Pass `include_local_delta: false` to query the committed Base Layer only.

#### Reachability Index — Dual Model (Decision D5, D8)

The reachability index serves two complementary purposes:

1. **Live session state** — tracks active thread instances and their current in-progress analysis;
   stored in Container/Durable Object memory (ephemeral, authoritative for running sessions).
2. **Committed baseline** — tracks last committed graph state per branch/ref;
   stored in D1 (persistent, authoritative for offline and divergence analysis).

**Goal**: understand how feature branches are converging or diverging. Live data is the primary
source for real-time conflict queries; the committed baseline enables offline/divergence analysis
and comparison against a known good state.

**k-Hop Bounded** (k=3 default, Decision D8):
NOT a full transitive closure. Full closure for 10M nodes ≈ 800GB, which exceeds D1's 10GB limit.
Instead:

- Pre-compute reachability up to **k=3 hops** from each changed node (configurable)
- Beyond k hops: **on-demand BFS** (streaming, does not materialize the full closure)
- Conflict detection queries beyond k hops use streaming BFS from the Container

```rust
// Table: reachability (D1 committed baseline — k-hop bounded)
pub struct ReachabilityEntry {
    pub ancestor_id: NodeId,    // Upstream node (e.g., modified function)
    pub descendant_id: NodeId,  // Downstream node (e.g., affected API)
    pub hops: u32,              // Distance (≤ k, typically k=3)
    pub path_hash: u64,         // Hash of the path taken (for incremental updates)
    pub branch_ref: String,     // Git ref this baseline was computed from
    pub computed_at: i64,       // Unix timestamp of last computation (for staleness)
}
```

**Reachability Logic**:

- **Write Path**: `ThreadBuildGraphFunction` computes reachability up to k hops for changed nodes and performs `BATCH INSERT` into D1.
- **Read Path**: Queries run `SELECT descendant_id FROM reachability WHERE ancestor_id = ? AND hops <= ?` (O(1) index lookup within k-hop bound).
- **Beyond k hops**: Streaming BFS in Container; does NOT materialize the full transitive closure.
- **Maintenance**: Incremental updates only recalculate reachability for the changed subgraph (not the full graph).

> **Note**: Conflict detection itself (consuming this index) is deferred to the commercial
> `thread-conflict` crate (Phase 4, commercial scope). The reachability index infrastructure
> (T034) is OSS and lives in `thread-storage`.

---

### 5. Conflict Prediction

**Purpose**: Represents a detected potential conflict between concurrent code changes

> **Type ownership**: These types are defined in `thread-api/src/types.rs` (OSS), not in `thread-conflict`. `thread-conflict` (commercial) imports them from `thread-api`. This ensures `thread-api` compiles independently of the commercial crate.

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

/// Newtype wrapper for conflict identifiers. Prevents accidental substitution of
/// other string-typed IDs. Format: `"conflict:{hash}"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ConflictId(Box<str>);
impl ConflictId {
    pub fn as_str(&self) -> &str { &self.0 }
}
/// Newtype wrapper for user identifiers. Prevents accidental substitution of
/// other string-typed IDs. Value is the OAuth2/OIDC provider subject claim (`sub`).
/// Format: `"{provider}:{subject}"` (e.g., `"github:12345678"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct UserId(Box<str>);
impl UserId {
    pub fn as_str(&self) -> &str { &self.0 }
}

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
- Many-to-one with `AnalysisSession` (many conflicts can be detected during one analysis session)

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
    pub git_ref: Option<String>,    // Git ref (commit SHA or branch name) being analyzed; None for non-VCS sources
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

/// Newtype wrapper for analysis session identifiers. Prevents accidental substitution of
/// other string-typed IDs. Format: `"session:{timestamp}:{hash}"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SessionId(Box<str>);
impl SessionId {
    pub fn as_str(&self) -> &str { &self.0 }
}

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

/// Newtype wrapper for plugin engine identifiers. Prevents accidental substitution of
/// other string-typed IDs. Format: `"engine:{type}:{name}"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct EngineId(Box<str>);
impl EngineId {
    pub fn as_str(&self) -> &str { &self.0 }
}

pub enum EngineType {
    Parser { language: Language },  // AST parsing engine (thread-ast-engine)
    GraphBuilder,                   // Graph construction engine
    ConflictDetector { tier: u8 },  // Conflict detection engine (1, 2, or 3)
    SemanticAnalyzer,               // Semantic analysis engine (CodeWeaver?)
}

pub struct EngineConfig {
    pub params: thread_utils::RapidMap<String, serde_json::Value>, // Key-value configuration
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

### 8. Delta (Overlay Graph — Uncommitted Changes)

**Purpose**: Represents a developer's local uncommitted changes layered on top of the committed Base Layer (FR-017). The query engine merges Base + Delta at runtime to produce the Unified View without modifying persistent Base storage.

> **OSS/Commercial boundary**: In OSS CLI, Deltas are stored in-memory per process (single-user, process lifetime). Multi-developer Delta sharing — required for cross-developer conflict detection (US2) — is a commercial capability implemented via Durable Object storage.

**Attributes**:

```rust
// NOTE: Delta implementation scope —
//   OSS CLI:         In-memory only, single-user. Not shared across processes or connections.
//   Commercial edge: Durable Object memory, shared across developers in the same repository session.
pub struct Delta {
    pub user_id: UserId,
    pub repository_id: RepositoryId,
    pub session_id: SessionId,
    pub changed_nodes: thread_utils::RapidMap<NodeId, GraphNode>,  // Modified or added nodes (local state)
    pub removed_nodes: thread_utils::RapidSet<NodeId>,             // Nodes deleted in local working state
    pub added_edges: Vec<GraphEdge>,                // New relationships in local working state
    pub removed_edges: thread_utils::RapidSet<EdgeId>,             // Removed relationships in local working state
    pub base_ref: String,                           // Git ref this delta was forked from (e.g., "main@abc123")
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}
```

**Relationships**:

- Many-to-one with `CodeRepository` (Delta applies changes within one repository)
- Many-to-one with `AnalysisSession` (Delta is owned by one analysis session)

**Storage**:

- OSS CLI: In-memory only (process lifetime; discarded on exit)
- Commercial edge: Durable Object storage (session lifetime; shared across WebSocket connections from the same developer)
- NOT persisted to Postgres or D1 — Deltas are ephemeral by design (FR-017)

---

## Entity Relationships Diagram

```plaintext
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

**ReCoco Integration**:

- All entities use content-addressed IDs (Blake3 hashes)
- Content changes → new ID → automatic cache invalidation
- Incremental updates: diff old vs new IDs, update only changed nodes/edges
- Cache key format: `{entity_type}:{content_hash}`

**Current Implementation**: The `thread-flow` crate implements Blake3-based fingerprinting
(`AnalysisDefFingerprint`) for content addressing. The full entity-level CAS with `NodeId`
content hashes is the target state for `thread-graph`/`thread-storage`.

**Cache Hit Rate Target**: >90% (SC-CACHE-001)

**Example**:

```rust
// Function signature changes
let old_id = NodeId::from_content("fn process(x: i32)");  // "node:abc123..."
let new_id = NodeId::from_content("fn process(x: String)"); // "node:def456..." (different!)

// ReCoco detects change, invalidates cache for old_id
recoco.invalidate(&old_id)?;

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

1. **Content Hashing**: All IDs derived from content Blake3 hashes (deterministic)
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
3. Implement ReCoco content-addressing for all entities (foundation exists in thread-flow via Blake3 fingerprinting)
4. Write contract tests for entity invariants
5. Create database indexes for performance targets (SC-STORE-001)
