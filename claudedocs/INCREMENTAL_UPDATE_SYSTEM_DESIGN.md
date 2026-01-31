# Thread Incremental Update System - Design Specification

**Design Date**: 2026-01-28
**Constitutional Requirement**: Principle VI - Service Architecture & Persistence
**Critical Compliance Gap**: Incremental updates NOT implemented (constitutional compliance report: ❌ Non-Compliant)

---

## Executive Summary

This design specifies the incremental update system for Thread, enabling **affected component detection** and **dependency-aware invalidation** to achieve constitutional compliance. The design leverages ReCoco's proven `FieldDefFingerprint` pattern while adapting it to Thread's AST analysis domain.

**Key Outcomes**:
- ✅ Only re-analyze affected components when source files change
- ✅ Avoid full repository re-scans (current 10-100x performance penalty)
- ✅ Maintain dependency graph for cascading invalidation
- ✅ Preserve content-addressed caching benefits (99.7% cost reduction)
- ✅ Support both CLI (Rayon) and Edge (tokio async) deployments

**Performance Impact**:
- **Before**: Edit `utils.rs` → full repository re-scan (10-100x slower)
- **After**: Edit `utils.rs` → re-analyze only files importing it (<2x slower)

---

## 1. Architectural Overview

### 1.1 System Components

Thread's incremental update system consists of four integrated subsystems:

```
┌─────────────────────────────────────────────────────────────────┐
│                  Incremental Update System                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────┐     ┌──────────────────┐                 │
│  │  Fingerprint    │────▶│  Dependency      │                 │
│  │  Tracker        │     │  Graph           │                 │
│  └─────────────────┘     └──────────────────┘                 │
│         │                         │                            │
│         │                         ▼                            │
│         │                ┌──────────────────┐                 │
│         │                │  Invalidation    │                 │
│         │                │  Detector        │                 │
│         │                └──────────────────┘                 │
│         │                         │                            │
│         ▼                         ▼                            │
│  ┌─────────────────────────────────────────┐                 │
│  │  Storage Backend (Postgres/D1)          │                 │
│  └─────────────────────────────────────────┘                 │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Component Responsibilities**:

1. **Fingerprint Tracker**: Tracks content-addressed fingerprints (Blake3) for each file and AST node
2. **Dependency Graph**: Maintains import/export relationships between files and symbols
3. **Invalidation Detector**: Identifies affected components based on fingerprint changes and dependencies
4. **Storage Backend**: Persists dependency graph and fingerprints for cross-session incremental updates

### 1.2 Core Data Structures

**Inspired by ReCoco's `FieldDefFingerprint` pattern** (analyzer.rs:69-84):

```rust
/// Tracks what affects the value of an analysis result
/// Pattern adapted from ReCoco's FieldDefFingerprint
#[derive(Debug, Clone)]
pub struct AnalysisDefFingerprint {
    /// Source files that contribute to this analysis result
    pub source_files: HashSet<PathBuf>,

    /// Content fingerprint of the analysis logic
    /// Combines: file content + parser version + rule configuration
    pub fingerprint: Fingerprint,

    /// Timestamp of last successful analysis
    pub last_analyzed: Option<i64>,
}

/// Dependency edge in the code graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    /// Source file path
    pub from: PathBuf,

    /// Target file path
    pub to: PathBuf,

    /// Dependency type (import, export, macro, etc.)
    pub dep_type: DependencyType,

    /// Symbol-level dependency info (optional)
    pub symbol: Option<SymbolDependency>,
}

/// Symbol-level dependency tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolDependency {
    /// Symbol path in source file
    pub from_symbol: String,

    /// Symbol path in target file
    pub to_symbol: String,

    /// Dependency strength (strong vs weak)
    pub strength: DependencyStrength,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DependencyType {
    /// Direct import/require/use statement
    Import,

    /// Export declaration
    Export,

    /// Macro expansion dependency
    Macro,

    /// Type dependency (e.g., TypeScript interfaces)
    Type,

    /// Trait implementation dependency (Rust)
    Trait,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DependencyStrength {
    /// Hard dependency: change requires reanalysis
    Strong,

    /// Soft dependency: change may require reanalysis
    Weak,
}
```

**Design Rationale** (from ReCoco pattern):
- **Source tracking**: Enables precise invalidation scope determination
- **Fingerprint composition**: Detects both content AND logic changes (analyzer.rs:858-862)
- **Hierarchical structure**: Supports file-level and symbol-level dependency tracking

---

## 2. Dependency Graph Construction

### 2.1 Graph Building Strategy

**Pattern**: ReCoco's `analyze_field_path` approach (analyzer.rs:466-516)

Thread's dependency graph construction occurs during initial AST analysis:

```rust
impl DependencyGraphBuilder {
    /// Build dependency graph during AST traversal
    /// Pattern: Similar to ReCoco's DataScopeBuilder.analyze_field_path
    pub fn build_from_analysis(
        &mut self,
        file_path: &Path,
        root: &tree_sitter::Node,
        language: &Language,
    ) -> Result<()> {
        // 1. Extract imports/exports from AST
        let imports = self.extract_imports(root, language)?;
        let exports = self.extract_exports(root, language)?;

        // 2. Resolve import targets to actual file paths
        for import in imports {
            let target_path = self.resolve_import_path(
                file_path,
                &import.module_path,
            )?;

            // 3. Create dependency edge
            let edge = DependencyEdge {
                from: file_path.to_path_buf(),
                to: target_path,
                dep_type: DependencyType::Import,
                symbol: import.symbol.map(|s| SymbolDependency {
                    from_symbol: s.imported_name,
                    to_symbol: s.exported_name,
                    strength: DependencyStrength::Strong,
                }),
            };

            self.graph.add_edge(edge);
        }

        // 4. Index exports for reverse lookup
        for export in exports {
            self.export_index.insert(
                (file_path.to_path_buf(), export.symbol_name),
                export,
            );
        }

        Ok(())
    }

    /// Extract import statements from AST
    fn extract_imports(
        &self,
        root: &tree_sitter::Node,
        language: &Language,
    ) -> Result<Vec<ImportInfo>> {
        // Language-specific import extraction using tree-sitter queries
        let query = match language {
            Language::Rust => r#"
                (use_declaration
                  argument: (scoped_identifier) @import)
            "#,
            Language::TypeScript => r#"
                (import_statement
                  source: (string) @module)
            "#,
            Language::Python => r#"
                (import_statement
                  name: (dotted_name) @module)
                (import_from_statement
                  module_name: (dotted_name) @module)
            "#,
            _ => return Ok(vec![]),
        };

        // Execute tree-sitter query and extract import info
        // Implementation details omitted for brevity
        todo!()
    }
}
```

**Key Principles** (from ReCoco analyzer.rs):
1. **Hierarchical traversal**: Build graph during AST analysis pass (analyzer.rs:466-516)
2. **Fingerprint composition**: Track dependencies in fingerprint calculation (analyzer.rs:372-389)
3. **Incremental construction**: Support adding edges for new files without full rebuild

### 2.2 Storage Schema

**Pattern**: ReCoco's setup state persistence (exec_ctx.rs:38-52)

Dependency graph persists across sessions using Postgres (CLI) or D1 (Edge):

```sql
-- Dependency edges table (Postgres/D1)
CREATE TABLE dependency_edges (
    id SERIAL PRIMARY KEY,

    -- Source file
    from_file TEXT NOT NULL,
    from_symbol TEXT,

    -- Target file
    to_file TEXT NOT NULL,
    to_symbol TEXT,

    -- Dependency metadata
    dep_type TEXT NOT NULL, -- 'import', 'export', 'macro', 'type', 'trait'
    strength TEXT NOT NULL, -- 'strong', 'weak'

    -- Timestamps
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    -- Composite unique constraint
    UNIQUE(from_file, to_file, from_symbol, to_symbol, dep_type)
);

CREATE INDEX idx_dep_from ON dependency_edges(from_file);
CREATE INDEX idx_dep_to ON dependency_edges(to_file);
CREATE INDEX idx_dep_symbol ON dependency_edges(from_symbol, to_symbol);

-- Analysis fingerprints table
CREATE TABLE analysis_fingerprints (
    id SERIAL PRIMARY KEY,

    -- File identification
    file_path TEXT NOT NULL UNIQUE,

    -- Fingerprint tracking
    content_fingerprint BYTEA NOT NULL, -- Blake3 hash (16 bytes)
    analysis_fingerprint BYTEA NOT NULL, -- Combined logic + content hash

    -- Source tracking (ReCoco pattern: source_op_names)
    dependent_files TEXT[], -- Array of file paths this analysis depends on

    -- Timestamps
    last_analyzed BIGINT NOT NULL, -- Unix timestamp in microseconds
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_fingerprint_path ON analysis_fingerprints(file_path);
CREATE INDEX idx_fingerprint_content ON analysis_fingerprints(content_fingerprint);
CREATE INDEX idx_fingerprint_analyzed ON analysis_fingerprints(last_analyzed);
```

**Design Rationale**:
- **Separate tables**: Dependency graph vs. fingerprint tracking (ReCoco pattern: separate source/target states)
- **Array fields**: D1 supports JSON arrays; Postgres supports native arrays
- **Timestamps**: Track analysis freshness for invalidation decisions
- **Indexes**: Optimize graph traversal queries (from_file, to_file lookups)

### 2.3 Graph Traversal Algorithms

**Pattern**: ReCoco's scope traversal (analyzer.rs:656-668)

Thread implements bidirectional graph traversal for invalidation:

```rust
impl DependencyGraph {
    /// Find all files affected by changes to source files
    /// Pattern: Similar to ReCoco's is_op_scope_descendant traversal
    pub fn find_affected_files(
        &self,
        changed_files: &HashSet<PathBuf>,
    ) -> Result<HashSet<PathBuf>> {
        let mut affected = HashSet::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::from_iter(changed_files.iter().cloned());

        while let Some(file) = queue.pop_front() {
            if !visited.insert(file.clone()) {
                continue; // Already processed
            }

            affected.insert(file.clone());

            // Find all files that depend on this file
            let dependents = self.get_dependents(&file)?;

            for dependent in dependents {
                // Only traverse strong dependencies for invalidation
                if dependent.strength == DependencyStrength::Strong {
                    queue.push_back(dependent.from.clone());
                }
            }
        }

        Ok(affected)
    }

    /// Get all files that directly depend on the given file
    fn get_dependents(&self, file: &Path) -> Result<Vec<DependencyEdge>> {
        // Query storage backend for edges where `to_file = file`
        // Return edges sorted by dependency strength (Strong first)
        todo!()
    }

    /// Topological sort for ordered reanalysis
    /// Ensures dependencies are analyzed before dependents
    pub fn topological_sort(
        &self,
        files: &HashSet<PathBuf>,
    ) -> Result<Vec<PathBuf>> {
        let mut sorted = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_mark = HashSet::new();

        for file in files {
            if !visited.contains(file) {
                self.visit_node(
                    file,
                    &mut visited,
                    &mut temp_mark,
                    &mut sorted,
                )?;
            }
        }

        sorted.reverse(); // Return in dependency order
        Ok(sorted)
    }

    /// DFS visit for topological sort (detects cycles)
    fn visit_node(
        &self,
        file: &Path,
        visited: &mut HashSet<PathBuf>,
        temp_mark: &mut HashSet<PathBuf>,
        sorted: &mut Vec<PathBuf>,
    ) -> Result<()> {
        if temp_mark.contains(file) {
            return Err(Error::CyclicDependency(file.to_path_buf()));
        }

        if visited.contains(file) {
            return Ok(());
        }

        temp_mark.insert(file.to_path_buf());

        // Visit dependencies first
        let dependencies = self.get_dependencies(file)?;
        for dep in dependencies {
            self.visit_node(&dep.to, visited, temp_mark, sorted)?;
        }

        temp_mark.remove(file);
        visited.insert(file.to_path_buf());
        sorted.push(file.to_path_buf());

        Ok(())
    }
}
```

**Algorithm Complexity**:
- **find_affected_files**: O(V + E) where V = files, E = dependency edges (BFS)
- **topological_sort**: O(V + E) (DFS-based)
- **Cycle detection**: Built into topological sort (temp_mark tracking)

---

## 3. Fingerprint-Based Change Detection

### 3.1 Fingerprint Composition

**Pattern**: ReCoco's `FieldDefFingerprint` builder (analyzer.rs:359-389)

Thread composes fingerprints from multiple sources:

```rust
impl AnalysisDefFingerprint {
    /// Create fingerprint for analysis result
    /// Pattern: ReCoco's FieldDefFingerprintBuilder.add() composition
    pub fn new(
        file_content: &[u8],
        parser_version: &str,
        rule_config: &RuleConfig,
        dependencies: &HashSet<PathBuf>,
    ) -> Result<Self> {
        let mut fingerprinter = Fingerprinter::default();

        // 1. Hash file content (primary signal)
        fingerprinter = fingerprinter.with(file_content)?;

        // 2. Hash parser version (logic change detection)
        fingerprinter = fingerprinter.with(parser_version)?;

        // 3. Hash rule configuration (logic change detection)
        fingerprinter = fingerprinter.with(rule_config)?;

        // 4. Hash dependency fingerprints (cascading invalidation)
        let mut dep_fingerprints: Vec<_> = dependencies.iter().collect();
        dep_fingerprints.sort(); // Deterministic ordering

        for dep in dep_fingerprints {
            let dep_fp = Self::load_from_storage(dep)?;
            fingerprinter = fingerprinter.with(&dep_fp.fingerprint)?;
        }

        Ok(Self {
            source_files: dependencies.clone(),
            fingerprint: fingerprinter.into_fingerprint(),
            last_analyzed: Some(chrono::Utc::now().timestamp_micros()),
        })
    }

    /// Check if analysis is still valid
    /// Pattern: ReCoco's SourceLogicFingerprint.matches (indexing_status.rs:54-57)
    pub fn matches(&self, current_content: &[u8]) -> bool {
        // Quick check: content fingerprint only
        let content_fp = Fingerprinter::default()
            .with(current_content)
            .ok()
            .map(|fp| fp.into_fingerprint());

        content_fp
            .map(|fp| fp.as_slice() == self.fingerprint.as_slice())
            .unwrap_or(false)
    }

    /// Full validation including dependencies
    pub fn is_valid(
        &self,
        current_content: &[u8],
        current_deps: &HashSet<PathBuf>,
    ) -> Result<bool> {
        // 1. Check content fingerprint
        if !self.matches(current_content) {
            return Ok(false);
        }

        // 2. Check dependency set changes
        if &self.source_files != current_deps {
            return Ok(false);
        }

        // 3. Check dependency fingerprints (cascading invalidation)
        for dep in current_deps {
            let dep_fp = Self::load_from_storage(dep)?;
            let current_dep_content = std::fs::read(dep)?;

            if !dep_fp.matches(&current_dep_content) {
                return Ok(false); // Dependency changed
            }
        }

        Ok(true)
    }
}
```

**Fingerprint Invalidation Scenarios**:

| Scenario | Content Hash | Dependency Set | Dependency FP | Result |
|----------|--------------|----------------|---------------|--------|
| File edited | ❌ Changed | ✅ Same | ✅ Same | **Invalid** - Re-analyze |
| Import added | ✅ Same | ❌ Changed | N/A | **Invalid** - Re-analyze |
| Dependency edited | ✅ Same | ✅ Same | ❌ Changed | **Invalid** - Cascading invalidation |
| No changes | ✅ Same | ✅ Same | ✅ Same | **Valid** - Reuse cache |

### 3.2 Storage Integration

**Pattern**: ReCoco's database tracking (exec_ctx.rs:55-134)

Fingerprint persistence with transaction support:

```rust
impl AnalysisDefFingerprint {
    /// Persist fingerprint to storage backend
    /// Pattern: ReCoco's build_import_op_exec_ctx persistence
    pub async fn save_to_storage(
        &self,
        file_path: &Path,
        pool: &PgPool, // Or D1Context for edge
    ) -> Result<()> {
        let dependent_files: Vec<String> = self
            .source_files
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        sqlx::query(
            r#"
            INSERT INTO analysis_fingerprints
                (file_path, content_fingerprint, analysis_fingerprint,
                 dependent_files, last_analyzed)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (file_path) DO UPDATE SET
                content_fingerprint = EXCLUDED.content_fingerprint,
                analysis_fingerprint = EXCLUDED.analysis_fingerprint,
                dependent_files = EXCLUDED.dependent_files,
                last_analyzed = EXCLUDED.last_analyzed,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(file_path.to_string_lossy().as_ref())
        .bind(self.fingerprint.as_slice()) // Content FP (first 16 bytes)
        .bind(self.fingerprint.as_slice()) // Analysis FP (same for now)
        .bind(&dependent_files)
        .bind(self.last_analyzed)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Load fingerprint from storage
    pub async fn load_from_storage(
        file_path: &Path,
        pool: &PgPool,
    ) -> Result<Option<Self>> {
        let row = sqlx::query_as::<_, (Vec<u8>, Vec<String>, Option<i64>)>(
            r#"
            SELECT analysis_fingerprint, dependent_files, last_analyzed
            FROM analysis_fingerprints
            WHERE file_path = $1
            "#,
        )
        .bind(file_path.to_string_lossy().as_ref())
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|(fp_bytes, deps, timestamp)| {
            let mut fp_array = [0u8; 16];
            fp_array.copy_from_slice(&fp_bytes[..16]);

            Self {
                source_files: deps
                    .into_iter()
                    .map(PathBuf::from)
                    .collect(),
                fingerprint: Fingerprint(fp_array),
                last_analyzed: timestamp,
            }
        }))
    }
}
```

**Transaction Boundary**: All fingerprint updates within a single analysis run use a transaction to ensure consistency.

---

## 4. Invalidation and Reanalysis Strategy

### 4.1 Change Detection Algorithm

**Pattern**: ReCoco's refresh options and ordinal tracking (analyzer.rs:90-94, indexing_status.rs:78-119)

Thread's incremental update algorithm:

```rust
pub struct IncrementalAnalyzer {
    dependency_graph: DependencyGraph,
    storage_backend: Box<dyn StorageBackend>,
    cache: QueryCache,
}

impl IncrementalAnalyzer {
    /// Perform incremental analysis on changed files
    /// Pattern: Combines ReCoco's source indexing + invalidation detection
    pub async fn analyze_incremental(
        &mut self,
        workspace_root: &Path,
        changed_files: HashSet<PathBuf>,
    ) -> Result<AnalysisResult> {
        // 1. Detect all affected files (dependency traversal)
        let affected_files = self
            .dependency_graph
            .find_affected_files(&changed_files)?;

        info!(
            "Incremental update: {} changed files → {} affected files",
            changed_files.len(),
            affected_files.len()
        );

        // 2. Topological sort for ordered reanalysis
        let reanalysis_order = self
            .dependency_graph
            .topological_sort(&affected_files)?;

        // 3. Parallel analysis with dependency ordering
        let results = if cfg!(feature = "parallel") {
            // CLI: Use Rayon for parallel processing
            self.analyze_parallel_ordered(&reanalysis_order).await?
        } else {
            // Edge: Use tokio async for I/O-bound processing
            self.analyze_async_sequential(&reanalysis_order).await?
        };

        // 4. Update dependency graph with new edges
        for file in &reanalysis_order {
            self.update_dependency_edges(file).await?;
        }

        // 5. Persist updated fingerprints
        for (file, result) in &results {
            result.fingerprint
                .save_to_storage(file, &self.storage_backend)
                .await?;
        }

        Ok(AnalysisResult {
            analyzed_files: results.len(),
            cache_hits: self.cache.hit_count(),
            cache_misses: self.cache.miss_count(),
            total_time: Duration::default(), // Measured separately
        })
    }

    /// Parallel analysis with dependency ordering (CLI with Rayon)
    #[cfg(feature = "parallel")]
    async fn analyze_parallel_ordered(
        &self,
        files: &[PathBuf],
    ) -> Result<HashMap<PathBuf, FileAnalysisResult>> {
        use rayon::prelude::*;

        // Group files by dependency level for parallel processing
        let levels = self.partition_by_dependency_level(files)?;

        let mut all_results = HashMap::new();

        for level in levels {
            // Analyze files within same level in parallel
            let level_results: HashMap<_, _> = level
                .par_iter()
                .map(|file| {
                    let result = self.analyze_single_file(file)?;
                    Ok((file.clone(), result))
                })
                .collect::<Result<_>>()?;

            all_results.extend(level_results);
        }

        Ok(all_results)
    }

    /// Async sequential analysis (Edge with tokio)
    async fn analyze_async_sequential(
        &self,
        files: &[PathBuf],
    ) -> Result<HashMap<PathBuf, FileAnalysisResult>> {
        let mut results = HashMap::new();

        for file in files {
            let result = self.analyze_single_file(file)?;
            results.insert(file.clone(), result);
        }

        Ok(results)
    }

    /// Partition files into dependency levels for parallel processing
    fn partition_by_dependency_level(
        &self,
        files: &[PathBuf],
    ) -> Result<Vec<Vec<PathBuf>>> {
        // Kahn's algorithm for topological level assignment
        let mut in_degree: HashMap<PathBuf, usize> = HashMap::new();
        let mut adjacency: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();

        // Build in-degree and adjacency list
        for file in files {
            in_degree.entry(file.clone()).or_insert(0);

            let deps = self.dependency_graph.get_dependencies(file)?;
            for dep in deps {
                if files.contains(&dep.to) {
                    adjacency
                        .entry(dep.to.clone())
                        .or_default()
                        .push(file.clone());
                    *in_degree.entry(file.clone()).or_insert(0) += 1;
                }
            }
        }

        // Level assignment
        let mut levels = Vec::new();
        let mut current_level: Vec<_> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(file, _)| file.clone())
            .collect();

        while !current_level.is_empty() {
            levels.push(current_level.clone());

            let mut next_level = Vec::new();
            for file in &current_level {
                if let Some(neighbors) = adjacency.get(file) {
                    for neighbor in neighbors {
                        let deg = in_degree.get_mut(neighbor).unwrap();
                        *deg -= 1;
                        if *deg == 0 {
                            next_level.push(neighbor.clone());
                        }
                    }
                }
            }

            current_level = next_level;
        }

        Ok(levels)
    }
}
```

### 4.2 Cache Integration

**Pattern**: ReCoco's caching strategy (analyzer.rs:947-965)

Incremental updates preserve cache benefits:

```rust
impl IncrementalAnalyzer {
    /// Analyze single file with cache integration
    /// Pattern: ReCoco's enable_cache + behavior_version tracking
    fn analyze_single_file(
        &self,
        file: &Path,
    ) -> Result<FileAnalysisResult> {
        // 1. Load existing fingerprint
        let existing_fp = AnalysisDefFingerprint::load_from_storage(
            file,
            &self.storage_backend,
        )?;

        // 2. Read current file content
        let content = std::fs::read(file)?;

        // 3. Extract dependencies
        let dependencies = self.extract_file_dependencies(file, &content)?;

        // 4. Check if analysis is still valid
        if let Some(fp) = &existing_fp {
            if fp.is_valid(&content, &dependencies)? {
                // Cache hit: Reuse existing analysis
                let cached_result = self.cache
                    .get(file)
                    .ok_or_else(|| Error::CacheMiss)?;

                return Ok(FileAnalysisResult {
                    analysis: cached_result,
                    fingerprint: fp.clone(),
                    cache_hit: true,
                });
            }
        }

        // 5. Cache miss: Perform full analysis
        let analysis = self.perform_full_analysis(file, &content)?;

        // 6. Create new fingerprint
        let new_fp = AnalysisDefFingerprint::new(
            &content,
            &self.parser_version,
            &self.rule_config,
            &dependencies,
        )?;

        // 7. Update cache
        self.cache.insert(file.clone(), analysis.clone());

        Ok(FileAnalysisResult {
            analysis,
            fingerprint: new_fp,
            cache_hit: false,
        })
    }
}
```

**Cache Coherence**: Fingerprint validation ensures cache entries are invalidated when dependencies change, maintaining cache consistency.

---

## 5. Implementation Phases

### Phase 1: Core Infrastructure (Week 1-2)

**Deliverables**:
1. ✅ Data structures (`AnalysisDefFingerprint`, `DependencyEdge`, `DependencyGraph`)
2. ✅ Storage schema (Postgres + D1 migrations)
3. ✅ Fingerprint composition and validation logic
4. ✅ Graph traversal algorithms (BFS, topological sort)

**Success Criteria**:
- All data structures compile with zero warnings
- Storage schema migrations execute successfully on Postgres and D1
- Unit tests pass for fingerprint composition and validation (100% coverage)
- Graph traversal algorithms handle cycles and disconnected components

**Constitutional Alignment**: Service-library architecture (Principle I)

### Phase 2: Dependency Extraction (Week 2-3)

**Deliverables**:
1. ✅ Tree-sitter query-based import/export extraction
2. ✅ Language-specific import resolution (Rust, TypeScript, Python)
3. ✅ Symbol-level dependency tracking
4. ✅ Dependency graph builder integration

**Success Criteria**:
- Import extraction works for all Tier 1 languages (Rust, JS/TS, Python, Go, Java)
- Import resolution handles relative and absolute paths correctly
- Symbol-level tracking captures function/class dependencies
- Graph builder integrates with existing AST analysis pipeline

**Constitutional Alignment**: Test-first development (Principle III - NON-NEGOTIABLE)

### Phase 3: Incremental Analysis Engine (Week 3-4)

**Deliverables**:
1. ✅ `IncrementalAnalyzer` implementation
2. ✅ Change detection algorithm
3. ✅ Parallel reanalysis with dependency ordering (Rayon)
4. ✅ Async reanalysis (tokio for Edge)

**Success Criteria**:
- Incremental analysis correctly identifies affected files
- Parallel analysis respects dependency ordering (no race conditions)
- Edge deployment handles async analysis without blocking
- Performance regression tests pass (<10ms incremental update overhead)

**Constitutional Alignment**: Dual deployment architecture (CLI + Edge)

### Phase 4: Integration and Optimization (Week 4-5)

**Deliverables**:
1. ✅ Integration with existing cache system (`QueryCache`)
2. ✅ Performance benchmarks for incremental vs. full analysis
3. ✅ CLI commands for graph inspection (`thread deps`, `thread invalidate`)
4. ✅ Documentation and examples

**Success Criteria**:
- Cache integration maintains >90% hit rate requirement (Principle VI)
- Incremental analysis is 10-100x faster than full re-scan
- CLI commands provide actionable insights for developers
- All documentation examples execute successfully

**Constitutional Alignment**: Storage performance targets (<10ms Postgres, <50ms D1)

### Phase 5: Production Hardening (Week 5-6)

**Deliverables**:
1. ✅ Edge cases: cyclic dependencies, missing files, corrupted graph
2. ✅ Error recovery: fallback to full analysis on graph corruption
3. ✅ Monitoring: metrics for invalidation rate, graph size, analysis time
4. ✅ Load testing: 10k files, 100k dependency edges

**Success Criteria**:
- Graceful degradation when graph is corrupted (log warning, rebuild)
- Cyclic dependency detection with actionable error messages
- Prometheus metrics exported for monitoring
- Load tests complete without OOM or excessive latency

**Constitutional Alignment**: Production readiness and quality gates

---

## 6. Performance Targets

### 6.1 Incremental Update Latency

**Constitutional Requirement**: <10ms Postgres, <50ms D1 p95 latency

| Operation | Target Latency | Rationale |
|-----------|----------------|-----------|
| Fingerprint lookup | <1ms | Single table query with index |
| Dependency traversal (10 files) | <5ms | BFS with indexed edges |
| Topological sort (100 files) | <10ms | Linear algorithm O(V+E) |
| Full incremental update (1 file changed, 5 affected) | <50ms | Analysis + storage writes |

### 6.2 Cache Hit Rate

**Constitutional Requirement**: >90% cache hit rate

**Expected Distribution**:
- **Unchanged files**: 95% cache hit (fingerprint validation passes)
- **Changed files**: 0% cache hit (fingerprint invalidation triggers reanalysis)
- **Affected dependencies**: 30% cache hit (some dependencies unchanged at symbol level)

**Overall Hit Rate**: ~90-93% for typical development workflows (3-5% of files change per commit)

### 6.3 Storage Overhead

**Estimated Storage Requirements**:
- **Dependency graph**: ~50 bytes per edge × 10k edges = 500KB
- **Fingerprints**: ~100 bytes per file × 10k files = 1MB
- **Total overhead**: <2MB for 10k file codebase

**Acceptable Threshold**: <10MB for 100k file enterprise codebase

### 6.4 Scalability Limits

| Metric | Small Project | Medium Project | Large Project | Limit |
|--------|---------------|----------------|---------------|-------|
| Files | 100 | 1,000 | 10,000 | 100,000 |
| Dependency edges | 200 | 5,000 | 50,000 | 500,000 |
| Graph traversal time | <1ms | <10ms | <100ms | <1s |
| Memory overhead | <100KB | <1MB | <10MB | <100MB |

---

## 7. Edge Cases and Error Handling

### 7.1 Cyclic Dependencies

**Detection**: Topological sort with temp_mark tracking (implemented)

**Handling**:
```rust
// Error variant
pub enum Error {
    CyclicDependency(PathBuf),
    // ...
}

// User-facing error message
impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::CyclicDependency(file) => write!(
                f,
                "Cyclic dependency detected involving file: {}\n\
                 Hint: Use `thread deps --cycles` to visualize the cycle",
                file.display()
            ),
            // ...
        }
    }
}
```

**Fallback**: Break cycle at weakest dependency strength, proceed with warning.

### 7.2 Missing Dependencies

**Scenario**: File imports module that doesn't exist in codebase

**Handling**:
```rust
impl DependencyGraphBuilder {
    fn resolve_import_path(
        &self,
        source: &Path,
        import: &str,
    ) -> Result<PathBuf> {
        // Try resolution strategies
        let candidates = vec![
            self.resolve_relative(source, import),
            self.resolve_absolute(import),
            self.resolve_node_modules(source, import),
        ];

        for candidate in candidates {
            if candidate.exists() {
                return Ok(candidate);
            }
        }

        // Missing dependency: log warning, skip edge
        warn!(
            "Failed to resolve import '{}' from {}",
            import,
            source.display()
        );

        // Return synthetic path for tracking
        Ok(PathBuf::from(format!("__missing__/{}", import)))
    }
}
```

**Impact**: Missing dependencies are tracked separately; affected files are re-analyzed conservatively.

### 7.3 Graph Corruption

**Detection**: Integrity checks on graph load (validate edge count, dangling nodes)

**Recovery**:
```rust
impl DependencyGraph {
    pub async fn load_or_rebuild(
        storage: &impl StorageBackend,
        workspace: &Path,
    ) -> Result<Self> {
        match Self::load_from_storage(storage).await {
            Ok(graph) if graph.validate().is_ok() => {
                info!("Loaded dependency graph with {} edges", graph.edge_count());
                Ok(graph)
            }
            Ok(_) | Err(_) => {
                warn!("Dependency graph corrupted or missing, rebuilding...");
                Self::rebuild_from_scratch(workspace, storage).await
            }
        }
    }

    fn validate(&self) -> Result<()> {
        // Check for dangling nodes
        for edge in &self.edges {
            if !self.nodes.contains(&edge.from) || !self.nodes.contains(&edge.to) {
                return Err(Error::CorruptedGraph(
                    "Dangling edge detected".into()
                ));
            }
        }

        Ok(())
    }
}
```

**Fallback**: Rebuild graph from scratch (one-time O(n) cost).

---

## 8. Monitoring and Observability

### 8.1 Prometheus Metrics

**Pattern**: ReCoco's metrics tracking (exec_ctx.rs, indexing_status.rs)

```rust
use prometheus::{IntCounter, IntGauge, Histogram, register_*};

lazy_static! {
    // Invalidation metrics
    static ref INVALIDATION_TOTAL: IntCounter = register_int_counter!(
        "thread_invalidation_total",
        "Total number of file invalidations"
    ).unwrap();

    static ref AFFECTED_FILES: Histogram = register_histogram!(
        "thread_affected_files",
        "Number of files affected per change",
        vec![1.0, 5.0, 10.0, 50.0, 100.0, 500.0]
    ).unwrap();

    // Graph metrics
    static ref GRAPH_NODES: IntGauge = register_int_gauge!(
        "thread_dependency_graph_nodes",
        "Number of nodes in dependency graph"
    ).unwrap();

    static ref GRAPH_EDGES: IntGauge = register_int_gauge!(
        "thread_dependency_graph_edges",
        "Number of edges in dependency graph"
    ).unwrap();

    // Performance metrics
    static ref INCREMENTAL_DURATION: Histogram = register_histogram!(
        "thread_incremental_update_duration_seconds",
        "Duration of incremental update",
        vec![0.01, 0.05, 0.1, 0.5, 1.0, 5.0]
    ).unwrap();
}
```

### 8.2 Logging Strategy

**Pattern**: ReCoco's structured logging with context

```rust
use tracing::{info, warn, error, debug, span, Level};

impl IncrementalAnalyzer {
    pub async fn analyze_incremental(
        &mut self,
        workspace_root: &Path,
        changed_files: HashSet<PathBuf>,
    ) -> Result<AnalysisResult> {
        let span = span!(
            Level::INFO,
            "incremental_update",
            workspace = %workspace_root.display(),
            changed_files = changed_files.len()
        );
        let _enter = span.enter();

        info!("Starting incremental update");

        let affected_files = self
            .dependency_graph
            .find_affected_files(&changed_files)?;

        info!(
            affected_files = affected_files.len(),
            "Computed affected files"
        );

        // Record metrics
        AFFECTED_FILES.observe(affected_files.len() as f64);

        // ...rest of implementation
    }
}
```

---

## 9. CLI Integration

### 9.1 Developer Commands

```bash
# Inspect dependency graph
thread deps <file>                    # Show dependencies of a file
thread deps --reverse <file>          # Show dependents of a file
thread deps --cycles                  # Detect and visualize cycles
thread deps --stats                   # Graph statistics

# Invalidation analysis
thread invalidate <file>              # Show what would be invalidated
thread invalidate --simulate <file>   # Dry-run incremental update

# Graph maintenance
thread graph rebuild                  # Rebuild dependency graph
thread graph validate                 # Check graph integrity
thread graph export --format dot      # Export to Graphviz
```

### 9.2 Configuration

**Pattern**: ReCoco's execution options

```yaml
# .thread/config.yml
incremental:
  # Enable incremental updates
  enabled: true

  # Graph storage backend
  storage:
    type: postgres  # or 'd1' for edge
    connection: postgresql://localhost/thread

  # Dependency tracking
  dependencies:
    # Track symbol-level dependencies
    symbol_level: true

    # Dependency types to track
    types:
      - import
      - export
      - macro
      - type

    # Dependency strength threshold
    strength: strong  # 'strong' or 'weak'

  # Performance tuning
  performance:
    # Max files to analyze in parallel (CLI only)
    parallel_limit: 8

    # Graph rebuild threshold (edges)
    rebuild_threshold: 100000

    # Cache TTL for fingerprints (seconds)
    fingerprint_ttl: 3600
```

---

## 10. Testing Strategy

### 10.1 Unit Tests

**Pattern**: Test-first development (Principle III)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_composition() {
        // Test fingerprint creation and validation
        let fp = AnalysisDefFingerprint::new(
            b"test content",
            "parser-v1.0",
            &RuleConfig::default(),
            &HashSet::new(),
        ).unwrap();

        assert!(fp.matches(b"test content"));
        assert!(!fp.matches(b"different content"));
    }

    #[test]
    fn test_dependency_graph_traversal() {
        // Test BFS traversal
        let mut graph = DependencyGraph::new();

        // Build test graph: A → B → C, A → D
        graph.add_edge(DependencyEdge {
            from: PathBuf::from("A"),
            to: PathBuf::from("B"),
            dep_type: DependencyType::Import,
            symbol: None,
        });
        // ...

        let affected = graph
            .find_affected_files(&HashSet::from([PathBuf::from("C")]))
            .unwrap();

        assert!(affected.contains(&PathBuf::from("B")));
        assert!(affected.contains(&PathBuf::from("A")));
    }

    #[test]
    fn test_cyclic_dependency_detection() {
        // Test cycle detection
        let mut graph = DependencyGraph::new();

        // Build cycle: A → B → C → A
        // ...

        let result = graph.topological_sort(&HashSet::from([
            PathBuf::from("A"),
            PathBuf::from("B"),
            PathBuf::from("C"),
        ]));

        assert!(matches!(result, Err(Error::CyclicDependency(_))));
    }
}
```

### 10.2 Integration Tests

```rust
#[tokio::test]
async fn test_incremental_update_end_to_end() {
    // Setup test workspace
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path();

    // Create test files
    create_test_file(workspace, "a.rs", "fn foo() {}");
    create_test_file(workspace, "b.rs", "use crate::a::foo;");

    // Initial analysis
    let mut analyzer = IncrementalAnalyzer::new(workspace).await.unwrap();
    let initial_result = analyzer
        .analyze_full(workspace)
        .await
        .unwrap();

    assert_eq!(initial_result.analyzed_files, 2);

    // Modify a.rs
    modify_test_file(workspace, "a.rs", "fn foo() {} fn bar() {}");

    // Incremental update
    let incremental_result = analyzer
        .analyze_incremental(
            workspace,
            HashSet::from([workspace.join("a.rs")]),
        )
        .await
        .unwrap();

    // Should re-analyze both a.rs and b.rs
    assert_eq!(incremental_result.analyzed_files, 2);
    assert!(incremental_result.cache_hits > 0); // Some cache reuse
}
```

### 10.3 Performance Regression Tests

**Pattern**: Load test report (LOAD_TEST_REPORT.md)

```rust
#[test]
fn test_incremental_update_latency() {
    // Ensure incremental updates meet constitutional targets
    let workspace = setup_large_test_workspace(10_000); // 10k files

    let start = Instant::now();
    let result = analyze_incremental(
        &workspace,
        HashSet::from([workspace.join("changed.rs")]),
    );
    let duration = start.elapsed();

    assert!(result.is_ok());
    assert!(duration < Duration::from_millis(100)); // <100ms for 1 file change
}
```

---

## 11. Migration Plan

### 11.1 Backward Compatibility

**Strategy**: Gradual rollout with feature flag

```rust
// Feature gate for incremental updates
#[cfg(feature = "incremental")]
pub mod incremental;

// Fallback to full analysis when feature is disabled
pub async fn analyze(workspace: &Path) -> Result<AnalysisResult> {
    #[cfg(feature = "incremental")]
    {
        if is_incremental_enabled() {
            return analyze_incremental(workspace).await;
        }
    }

    analyze_full(workspace).await
}
```

### 11.2 Migration Steps

**Phase 1**: Deploy with feature flag disabled (default: full analysis)
**Phase 2**: Enable for internal testing (10% of users)
**Phase 3**: Gradual rollout (25% → 50% → 100%)
**Phase 4**: Make incremental the default, keep full analysis as fallback

### 11.3 Rollback Plan

**Trigger**: Incremental analysis shows >5% error rate or >2x latency

**Action**:
1. Disable `incremental` feature flag via configuration
2. Clear corrupted dependency graphs from storage
3. Revert to full analysis mode
4. Investigate root cause offline

---

## 12. Future Enhancements

### 12.1 Cross-Repo Dependency Tracking

**Use Case**: Monorepo with multiple crates/packages

**Approach**: Extend dependency graph to track cross-crate imports, invalidate across boundaries

### 12.2 Symbol-Level Granularity

**Use Case**: Large files with multiple exports; only re-analyze affected symbols

**Approach**:
- Track symbol-level fingerprints in addition to file-level
- Invalidate only specific symbols and their dependents
- Requires AST-level diffing (complex)

### 12.3 Distributed Dependency Graph

**Use Case**: Team collaboration with shared dependency graph

**Approach**:
- Store dependency graph in shared storage (e.g., S3, GitHub repo)
- CRDTs for conflict-free graph merging
- Requires careful synchronization

### 12.4 Machine Learning-Based Prediction

**Use Case**: Predict likely affected files before running full traversal

**Approach**:
- Train model on historical invalidation patterns
- Use predictions to pre-warm cache or parallelize analysis
- Experimental; requires data collection

---

## 13. Success Metrics

### 13.1 Constitutional Compliance

| Requirement | Target | Measurement |
|-------------|--------|-------------|
| Incremental updates | Affected components only | ✅ BFS traversal validates |
| Postgres latency | <10ms p95 | Measure with Criterion benchmarks |
| D1 latency | <50ms p95 | Measure in Cloudflare Workers |
| Cache hit rate | >90% | Track via Prometheus metrics |

### 13.2 Developer Experience

| Metric | Target | Measurement |
|--------|--------|-------------|
| Incremental update time (1 file changed) | <1s | End-to-end CLI benchmark |
| Incremental update time (5 files changed) | <5s | End-to-end CLI benchmark |
| Graph rebuild time (10k files) | <30s | One-time rebuild benchmark |
| CLI command responsiveness | <100ms | `thread deps` latency |

### 13.3 Production Readiness

| Criterion | Target | Status |
|-----------|--------|--------|
| Test coverage | >90% | TDD ensures high coverage |
| Error recovery | Graceful degradation | Fallback to full analysis |
| Monitoring | Prometheus metrics | All key metrics instrumented |
| Documentation | Complete | CLI help, examples, architecture docs |

---

## 14. References

### 14.1 ReCoco Patterns Referenced

- **FieldDefFingerprint** (analyzer.rs:69-84): Fingerprint composition with source tracking
- **FieldDefFingerprintBuilder** (analyzer.rs:359-389): Incremental fingerprint construction
- **analyze_field_path** (analyzer.rs:466-516): Hierarchical dependency traversal
- **is_op_scope_descendant** (analyzer.rs:660-668): Ancestor chain traversal
- **SourceLogicFingerprint** (indexing_status.rs:20-58): Logic fingerprint matching
- **build_import_op_exec_ctx** (exec_ctx.rs:55-134): Setup state persistence
- **evaluate_with_cell** (evaluator.rs:25-26): Caching strategy with invalidation

### 14.2 Thread Constitution

- **Principle I**: Service-library architecture (dual deployment CLI + Edge)
- **Principle III**: Test-first development (TDD mandatory)
- **Principle VI**: Service architecture & persistence (incremental updates, storage targets, cache hit rate)

### 14.3 External References

- Tree-sitter documentation: https://tree-sitter.github.io/tree-sitter/
- Blake3 specification: https://github.com/BLAKE3-team/BLAKE3-specs
- Postgres JSONB indexing: https://www.postgresql.org/docs/current/datatype-json.html
- Cloudflare D1: https://developers.cloudflare.com/d1/

---

## Appendix A: Schema Definitions

### A.1 Complete Postgres Schema

```sql
-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Dependency edges table
CREATE TABLE dependency_edges (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Source file
    from_file TEXT NOT NULL,
    from_symbol TEXT,

    -- Target file
    to_file TEXT NOT NULL,
    to_symbol TEXT,

    -- Dependency metadata
    dep_type TEXT NOT NULL CHECK (dep_type IN ('import', 'export', 'macro', 'type', 'trait')),
    strength TEXT NOT NULL CHECK (strength IN ('strong', 'weak')),

    -- Timestamps
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    -- Composite unique constraint
    UNIQUE(from_file, to_file, from_symbol, to_symbol, dep_type)
);

-- Indexes for fast lookups
CREATE INDEX idx_dep_from ON dependency_edges(from_file);
CREATE INDEX idx_dep_to ON dependency_edges(to_file);
CREATE INDEX idx_dep_symbol ON dependency_edges(from_symbol, to_symbol) WHERE from_symbol IS NOT NULL;
CREATE INDEX idx_dep_type ON dependency_edges(dep_type);

-- Analysis fingerprints table
CREATE TABLE analysis_fingerprints (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- File identification
    file_path TEXT NOT NULL UNIQUE,

    -- Fingerprint tracking
    content_fingerprint BYTEA NOT NULL CHECK (length(content_fingerprint) = 16),
    analysis_fingerprint BYTEA NOT NULL CHECK (length(analysis_fingerprint) = 16),

    -- Source tracking (ReCoco pattern: source_op_names)
    dependent_files TEXT[] NOT NULL DEFAULT '{}',

    -- Timestamps
    last_analyzed BIGINT NOT NULL, -- Unix timestamp in microseconds
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for fingerprint lookups
CREATE INDEX idx_fingerprint_path ON analysis_fingerprints(file_path);
CREATE INDEX idx_fingerprint_content ON analysis_fingerprints(content_fingerprint);
CREATE INDEX idx_fingerprint_analysis ON analysis_fingerprints(analysis_fingerprint);
CREATE INDEX idx_fingerprint_analyzed ON analysis_fingerprints(last_analyzed);

-- Trigger for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_dependency_edges_updated_at
    BEFORE UPDATE ON dependency_edges
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_analysis_fingerprints_updated_at
    BEFORE UPDATE ON analysis_fingerprints
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

### A.2 D1 Schema Adaptations

```sql
-- D1 schema (similar to Postgres but with D1-specific adaptations)
-- Note: D1 doesn't support BYTEA, use BLOB instead
-- Note: D1 doesn't support arrays natively, use JSON

CREATE TABLE dependency_edges (
    id TEXT PRIMARY KEY, -- UUID stored as text

    from_file TEXT NOT NULL,
    from_symbol TEXT,

    to_file TEXT NOT NULL,
    to_symbol TEXT,

    dep_type TEXT NOT NULL CHECK (dep_type IN ('import', 'export', 'macro', 'type', 'trait')),
    strength TEXT NOT NULL CHECK (strength IN ('strong', 'weak')),

    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER DEFAULT (strftime('%s', 'now')),

    UNIQUE(from_file, to_file, from_symbol, to_symbol, dep_type)
);

CREATE INDEX idx_dep_from ON dependency_edges(from_file);
CREATE INDEX idx_dep_to ON dependency_edges(to_file);

CREATE TABLE analysis_fingerprints (
    id TEXT PRIMARY KEY,

    file_path TEXT NOT NULL UNIQUE,

    content_fingerprint BLOB NOT NULL, -- 16 bytes
    analysis_fingerprint BLOB NOT NULL, -- 16 bytes

    dependent_files TEXT NOT NULL DEFAULT '[]', -- JSON array

    last_analyzed INTEGER NOT NULL,
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX idx_fingerprint_path ON analysis_fingerprints(file_path);
CREATE INDEX idx_fingerprint_analyzed ON analysis_fingerprints(last_analyzed);
```

---

## Appendix B: Example Workflows

### B.1 Developer Workflow: Edit Single File

```bash
# 1. Developer edits utils.rs
$ vim src/utils.rs

# 2. Thread detects change (filesystem watch or explicit trigger)
$ thread analyze --incremental

# Output:
# Incremental update: 1 changed file → 5 affected files
# Analyzing: src/utils.rs
# Analyzing: src/main.rs (depends on utils.rs)
# Analyzing: src/lib.rs (depends on utils.rs)
# Analyzing: tests/integration.rs (depends on utils.rs)
# Analyzing: tests/unit.rs (depends on utils.rs)
#
# Analysis complete: 5 files analyzed in 1.2s
# Cache hits: 95 files (95% hit rate)
#
# Constitutional compliance: ✅ Incremental updates working

# 3. Inspect dependency impact
$ thread deps src/utils.rs --reverse

# Output:
# Files depending on src/utils.rs:
#   - src/main.rs (strong import)
#   - src/lib.rs (strong import)
#   - tests/integration.rs (weak import)
#   - tests/unit.rs (weak import)
```

### B.2 CI/CD Workflow: Pull Request Analysis

```yaml
# .github/workflows/thread-analysis.yml
name: Thread Incremental Analysis

on: [pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Full git history for base comparison

      - name: Install Thread
        run: cargo install thread-cli

      - name: Setup Postgres (for incremental updates)
        uses: ikalnytskyi/action-setup-postgres@v6

      - name: Run incremental analysis
        run: |
          # Get changed files
          CHANGED_FILES=$(git diff --name-only origin/main...HEAD)

          # Run Thread with incremental updates
          thread analyze --incremental --changed "$CHANGED_FILES"

      - name: Check constitutional compliance
        run: thread validate --constitutional-compliance
```

### B.3 Debugging Workflow: Graph Corruption

```bash
# Symptom: Incremental updates failing
$ thread analyze --incremental
# Error: Dependency graph corrupted (dangling edge detected)

# Step 1: Validate graph
$ thread graph validate
# Output:
# Graph validation FAILED:
#   - 3 dangling edges detected
#   - Edge (src/deleted.rs → src/main.rs) points to missing file
#   - Edge (src/renamed.rs → src/lib.rs) points to renamed file
#   - Edge (src/moved.rs → tests/unit.rs) points to moved file

# Step 2: Rebuild graph
$ thread graph rebuild
# Output:
# Rebuilding dependency graph from scratch...
# Scanning 1,234 files...
# Extracted 5,678 dependencies...
# Graph rebuilt successfully in 12.3s

# Step 3: Verify incremental updates
$ thread analyze --incremental
# Output:
# Incremental update: 1 changed file → 3 affected files
# Analysis complete: 3 files analyzed in 0.8s
# Cache hits: 1,231 files (99.8% hit rate)
```

---

**End of Design Specification**
