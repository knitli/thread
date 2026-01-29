# Thread Performance Optimization Roadmap

**Based on**: Performance Profiling Report (2026-01-28)
**Status**: Ready for implementation
**Priority Levels**: ⭐⭐⭐ Critical | ⭐⭐ High | ⭐ Medium

---

## Quick Wins (Week 1-2)

### 1. String Interning ⭐⭐⭐

**Impact**: 20-30% allocation reduction
**Effort**: 2-3 days
**File**: `crates/ast-engine/src/meta_var.rs`, `crates/rule-engine/src/rule_config.rs`

```rust
// Before:
pub struct MetaVarEnv {
    map: RapidMap<String, String>,
}

// After:
use lasso::{ThreadedRodeo, Spur};

pub struct MetaVarEnv {
    interner: Arc<ThreadedRodeo>,
    map: RapidMap<Spur, Spur>,
}
```

**Implementation Steps**:
1. Add `lasso = "0.7.3"` to workspace dependencies
2. Create global thread-safe string interner
3. Replace `String` with `Spur` for meta-variable names
4. Update `MetaVarEnv::from()` to use interner

**Success Metrics**:
- Allocation count reduction: -20-30%
- Meta-var conversion time: -10-15%
- Memory footprint: -15-25%

---

### 2. Pattern Compilation Cache ⭐⭐⭐

**Impact**: Eliminate repeated compilation overhead (~100µs per pattern)
**Effort**: 1-2 days
**File**: `crates/ast-engine/src/pattern.rs`

```rust
use moka::sync::Cache;
use std::sync::Arc;

lazy_static! {
    static ref PATTERN_CACHE: Cache<String, Arc<Pattern>> =
        Cache::builder()
            .max_capacity(10_000)
            .time_to_live(Duration::from_secs(3600))
            .build();
}

impl Pattern {
    pub fn new(pattern: &str, lang: &SupportLang) -> Self {
        let key = format!("{}::{}", lang.get_ts_language().name(), pattern);
        PATTERN_CACHE.get_or_insert_with(&key, || {
            Arc::new(Self::compile_internal(pattern, lang))
        }).as_ref().clone()
    }
}
```

**Implementation Steps**:
1. Add `moka = "0.12"` to ast-engine dependencies
2. Create static pattern cache with LRU eviction
3. Implement cache key: `language::pattern_string`
4. Wrap Pattern in `Arc` for cheap cloning

**Success Metrics**:
- Cache hit rate: >80% in typical workloads
- Pattern compilation time (cache hit): ~1µs (100x faster)
- Memory overhead: <10MB for 10K cached patterns

---

### 3. Lazy Parsing ⭐⭐

**Impact**: Skip parsing when file type doesn't match rules
**Effort**: 1 day
**File**: `crates/rule-engine/src/scanner.rs`

```rust
impl Scanner {
    pub fn scan_file(&self, path: &Path, rules: &[Rule]) -> Result<Vec<Match>> {
        // Fast path: Check file extension before parsing
        let ext = path.extension().and_then(|s| s.to_str());
        let applicable_rules: Vec<_> = rules.iter()
            .filter(|rule| rule.matches_file_extension(ext))
            .collect();

        if applicable_rules.is_empty() {
            return Ok(Vec::new()); // Skip parsing entirely
        }

        // Only parse if at least one rule might match
        let content = fs::read_to_string(path)?;
        let root = Root::str(&content, lang);
        // ... continue with matching
    }
}
```

**Implementation Steps**:
1. Add `matches_file_extension()` to Rule trait
2. Pre-filter rules before parsing
3. Add metrics for skipped parses

**Success Metrics**:
- Files skipped: 50-80% in multi-language repos
- Overall throughput: +30-50% on large codebases

---

## High-Value Optimizations (Month 1)

### 4. Arc<str> for Immutable Strings ⭐⭐⭐

**Impact**: Eliminate String clones in read-only contexts
**Effort**: 1 week (refactoring effort)
**Files**: Multiple across ast-engine, rule-engine

```rust
// Before:
pub struct Node {
    text: String,
}

// After:
pub struct Node {
    text: Arc<str>,
}

impl Node {
    pub fn text(&self) -> &str {
        &self.text  // Cheap: just deref Arc
    }

    pub fn clone_text(&self) -> Arc<str> {
        Arc::clone(&self.text)  // Cheap: just pointer clone
    }
}
```

**Implementation Steps**:
1. Identify String fields that are never mutated
2. Replace `String` with `Arc<str>`
3. Update function signatures to accept `&str` or `Arc<str>`
4. Benchmark allocation reduction

**Success Metrics**:
- Clone operations: -50-70% in AST traversal
- Memory usage: -20-30% for large ASTs
- Cache efficiency: Improved (smaller structures)

---

### 5. Copy-on-Write MetaVar Environments ⭐⭐

**Impact**: Reduce environment cloning during backtracking
**Effort**: 3-5 days
**File**: `crates/ast-engine/src/meta_var.rs`

```rust
use std::rc::Rc;
use std::cell::RefCell;

pub struct MetaVarEnv {
    inner: Rc<RefCell<MetaVarEnvInner>>,
}

impl MetaVarEnv {
    pub fn clone_for_backtrack(&self) -> Self {
        // Cheap: just clone Rc
        Self { inner: Rc::clone(&self.inner) }
    }

    pub fn insert(&mut self, key: String, value: String) {
        // COW: Clone only if shared
        if Rc::strong_count(&self.inner) > 1 {
            self.inner = Rc::new(RefCell::new(
                self.inner.borrow().clone()
            ));
        }
        self.inner.borrow_mut().insert(key, value);
    }
}
```

**Implementation Steps**:
1. Wrap MetaVarEnv in `Rc<RefCell<>>`
2. Implement COW semantics for mutations
3. Update matcher to use cheap clones
4. Benchmark backtracking performance

**Success Metrics**:
- Environment clones: -60-80% reduction
- Backtracking overhead: -30-50%
- Memory pressure: Significantly reduced

---

### 6. Query Result Caching ⭐⭐

**Impact**: Reduce database roundtrips
**Effort**: 2-3 days
**File**: `crates/flow/src/targets/d1.rs`, `crates/flow/src/cache.rs`

```rust
use moka::future::Cache;

pub struct CachedD1Target {
    client: D1Database,
    query_cache: Cache<String, Vec<Row>>,
}

impl CachedD1Target {
    pub async fn query(&self, sql: &str, params: &[Value]) -> Result<Vec<Row>> {
        let cache_key = format!("{}::{:?}", sql, params);

        self.query_cache.try_get_with(cache_key, async {
            self.client.prepare(sql)
                .bind(params)?
                .all()
                .await
        }).await
    }
}
```

**Implementation Steps**:
1. Add async LRU cache to D1/Postgres targets
2. Implement cache key generation (SQL + params hash)
3. Add cache metrics (hit rate, latency)
4. Configure TTL based on data volatility

**Success Metrics**:
- Cache hit rate: >70% for hot queries
- Query latency (cache hit): <1ms (vs 10-50ms)
- Database load: -50-80%

---

## Advanced Optimizations (Quarter 1)

### 7. Incremental Parsing ⭐⭐⭐

**Impact**: Only re-parse changed code regions
**Effort**: 2-3 weeks
**File**: `crates/ast-engine/src/root.rs`

```rust
use tree_sitter::InputEdit;

pub struct IncrementalRoot {
    tree: Tree,
    content: String,
}

impl IncrementalRoot {
    pub fn edit(&mut self, start_byte: usize, old_end_byte: usize,
                new_end_byte: usize, new_content: String) {
        // Apply edit to tree-sitter tree
        self.tree.edit(&InputEdit {
            start_byte,
            old_end_byte,
            new_end_byte,
            start_position: /* calculate */,
            old_end_position: /* calculate */,
            new_end_position: /* calculate */,
        });

        // Re-parse only changed region
        self.content = new_content;
        self.tree = parser.parse(&self.content, Some(&self.tree))?;
    }
}
```

**Implementation Steps**:
1. Integrate tree-sitter `InputEdit` API
2. Track file changes via LSP or file watcher
3. Implement incremental parse coordinator
4. Benchmark speedup on large files

**Success Metrics**:
- Incremental parse time: 10-100x faster than full parse
- Memory overhead: Minimal (keep old tree temporarily)
- Correctness: 100% (validated via tests)

---

### 8. SIMD Multi-Pattern Matching ⭐⭐

**Impact**: 2-4x throughput for large rule sets
**Effort**: 1-2 weeks
**File**: `crates/rule-engine/src/scanner.rs`

```rust
use aho_corasick::AhoCorasick;

pub struct SimdScanner {
    // Pre-compiled SIMD matcher for all patterns
    pattern_matcher: AhoCorasick,
    rule_map: Vec<Rule>,
}

impl SimdScanner {
    pub fn scan(&self, content: &str) -> Vec<Match> {
        // SIMD-accelerated multi-pattern search
        let matches = self.pattern_matcher.find_overlapping_iter(content);

        matches.map(|mat| {
            let rule = &self.rule_map[mat.pattern()];
            // Full AST matching only on SIMD-identified candidates
            self.verify_ast_match(content, rule, mat.start())
        }).collect()
    }
}
```

**Implementation Steps**:
1. Add `aho-corasick` with SIMD features
2. Extract literal patterns from rules
3. Use SIMD for initial filtering, AST for verification
4. Benchmark on large rule sets (100+ rules)

**Success Metrics**:
- Throughput: 2-4x on 100+ rule sets
- False positive rate: <10% (acceptable for pre-filter)
- Latency: Sub-millisecond for large files

---

### 9. Arena Allocators ⭐⭐

**Impact**: Reduce allocation overhead in short-lived operations
**Effort**: 2-3 weeks
**File**: `crates/ast-engine/src/pattern.rs`, `crates/ast-engine/src/matcher.rs`

```rust
use bumpalo::Bump;

pub struct ArenaMatcher<'arena> {
    arena: &'arena Bump,
    matcher: PatternMatcher<'arena>,
}

impl<'arena> ArenaMatcher<'arena> {
    pub fn match_node(&self, node: Node) -> Vec<&'arena Match> {
        // All temporary allocations use arena
        let temp_vec = bumpalo::vec![in self.arena; /* items */];

        // Arena automatically freed when dropped
        temp_vec
    }
}
```

**Implementation Steps**:
1. Add `bumpalo` for arena allocation
2. Refactor matcher to use arena lifetimes
3. Benchmark allocation count reduction
4. Measure performance impact (may be neutral/negative)

**Success Metrics**:
- Allocation count: -40-60% for short-lived operations
- Deallocation overhead: Eliminated (bulk free)
- Performance: Neutral to +10% (depends on workload)

---

## Long-Term Experiments (Quarter 2+)

### 10. Zero-Copy Pattern Matching ⭐

**Impact**: Eliminate intermediate allocations
**Effort**: 4-6 weeks
**File**: Refactor across entire ast-engine

**Concept**: Use `&str` slices throughout, eliminate `String` allocations.

**Challenges**:
- Lifetime management complexity
- API surface changes (breaking change)
- Incremental migration path required

---

### 11. Custom Global Allocator ⭐

**Impact**: 10-20% overall speedup (estimated)
**Effort**: 1-2 weeks (experimentation)

```rust
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
```

**Implementation**:
1. Benchmark with `mimalloc`, `jemalloc`, `snmalloc`
2. Measure allocation-heavy workloads
3. Choose best performer for Thread's patterns

---

## Measurement & Validation

### Performance Regression Tests

Add to CI pipeline:

```bash
# Benchmark baseline
cargo bench --bench performance_improvements -- --save-baseline main

# After changes
cargo bench --bench performance_improvements -- --baseline main

# Fail if >10% regression
```

### Profiling Dashboard

Integrate with existing `crates/flow/src/monitoring/performance.rs`:

- Prometheus metrics export
- Grafana dashboard (use existing `grafana/` directory)
- Real-time performance tracking

---

## Success Criteria

### Short-Term (Month 1)

- [ ] String interning: -20% allocations
- [ ] Pattern cache: >80% hit rate
- [ ] Lazy parsing: +30% throughput

### Medium-Term (Quarter 1)

- [ ] Memory usage: -30% overall
- [ ] Incremental parsing: 10-100x on edits
- [ ] Database queries: <10ms p95 (Postgres), <50ms p95 (D1)

### Long-Term (Quarter 2+)

- [ ] Zero-copy architecture: -50% allocations
- [ ] SIMD matching: 2-4x throughput
- [ ] Cache hit rate: >90% in production

---

**Version**: 1.0
**Date**: 2026-01-28
**Maintained By**: Performance Engineering Team
