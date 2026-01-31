# Thread Hot Paths Quick Reference

**Purpose**: Quick lookup guide for developers working on performance-critical code
**Last Updated**: 2026-01-28
**Based On**: Performance Profiling Report v1.0

---

## CPU Hot Spots

### 🔥 Critical Path #1: Pattern Matching (~45% CPU)

**Location**: `crates/ast-engine/src/pattern.rs`, `crates/ast-engine/src/matcher.rs`

**Current Performance**: 101.65 µs per operation

**Hot Functions**:
1. `Pattern::new()` - Pattern string parsing
2. `Node::find_all()` - AST traversal
3. `Matcher::match_node_non_recursive()` - Core matching logic

**Optimization Targets**:
- ⭐⭐⭐ Add pattern compilation cache (100x speedup on cache hit)
- ⭐⭐⭐ String interning for meta-variable names
- ⭐⭐ Replace String with Arc<str> for immutable data

**Quick Fix Example**:
```rust
// Add this to pattern.rs
use moka::sync::Cache;

lazy_static! {
    static ref PATTERN_CACHE: Cache<String, Arc<Pattern>> =
        Cache::builder().max_capacity(10_000).build();
}
```

---

### 🔥 Critical Path #2: Meta-Variable Processing (~15% CPU)

**Location**: `crates/ast-engine/src/meta_var.rs`

**Current Performance**: 22.696 µs per conversion (⚠️ 11.7% regression detected)

**Hot Functions**:
1. `MetaVarEnv::from()` - Environment construction
2. `RapidMap<String, String>` allocations

**Optimization Targets**:
- ⭐⭐⭐ String interning (replace String with Spur)
- ⭐⭐ Copy-on-write environments for backtracking
- ⭐ Use Arc<str> instead of String

**Quick Fix Example**:
```rust
use lasso::{ThreadedRodeo, Spur};

pub struct MetaVarEnv {
    interner: Arc<ThreadedRodeo>,
    map: RapidMap<Spur, Spur>, // Much cheaper than <String, String>
}
```

---

### 🔥 Critical Path #3: Pattern Children Collection (~10% CPU)

**Location**: `crates/ast-engine/src/pattern.rs`

**Current Performance**: 52.692 µs (⚠️ 10.5% regression detected)

**Hot Functions**:
1. Ellipsis pattern matching (`$$$ITEMS`)
2. Child node collection

**Optimization Targets**:
- ⭐⭐ Reduce intermediate allocations
- ⭐ Arena allocators for temporary vectors

---

### 🔥 Critical Path #4: Tree-Sitter Parsing (~30% CPU)

**Location**: `crates/language/src/lib.rs` (external dependency)

**Current Performance**: 500µs - 500ms (depends on file size)

**Optimization Strategy**:
- Cannot optimize directly (external library)
- ⭐⭐⭐ Cache parse results (content-addressed)
- ⭐⭐⭐ Incremental parsing for edits
- ⭐⭐ Lazy parsing (skip when not needed)

---

## Memory Hot Spots

### 💾 Hot Spot #1: String Allocations (~40% of allocations)

**Locations**: Throughout codebase

**Current Impact**: Largest allocation source

**Optimization**:
```rust
// Before
let name: String = node.text().to_string();

// After (string interning)
let name: Spur = interner.get_or_intern(node.text());

// Or (immutable sharing)
let name: Arc<str> = Arc::from(node.text());
```

**Expected Impact**: -20-30% total allocations

---

### 💾 Hot Spot #2: MetaVar Environment Cloning (~25% of allocations)

**Location**: `crates/ast-engine/src/meta_var.rs`

**Current Impact**: Expensive during backtracking

**Optimization**:
```rust
// Before
let env_copy = env.clone(); // Full HashMap clone

// After (COW)
let env_copy = Rc::clone(&env); // Cheap pointer clone
```

**Expected Impact**: -60-80% environment-related allocations

---

### 💾 Hot Spot #3: AST Node Wrappers (~20% of allocations)

**Location**: `crates/ast-engine/src/node.rs`

**Optimization**: Arena allocation for short-lived traversals
```rust
use bumpalo::Bump;

fn traverse_ast<'arena>(arena: &'arena Bump, root: Node) {
    let temp_vec = bumpalo::vec![in arena; /* items */];
    // Arena auto-freed on drop
}
```

---

## I/O Hot Spots

### 💿 Hot Spot #1: Database Queries (Unmetered)

**Location**: `crates/flow/src/targets/d1.rs`, `crates/flow/src/targets/postgres.rs`

**Constitutional Requirements**:
- Postgres: <10ms p95 latency
- D1 (edge): <50ms p95 latency

**Optimization**:
```rust
// Add query result caching
use moka::future::Cache;

let query_cache: Cache<String, Vec<Row>> = Cache::builder()
    .max_capacity(1_000)
    .time_to_live(Duration::from_secs(300))
    .build();
```

**Priority**: 🚨 HIGH - Required for Constitutional compliance

---

### 💿 Hot Spot #2: Content-Addressed Cache Lookup

**Location**: `crates/flow/src/cache.rs`

**Current Performance**: 18.66 µs (cache hit), 22.04 µs (cache miss)

**Status**: ✅ Already optimized (Blake3 fingerprinting)

---

## Quick Optimization Checklist

### Before Making Changes

- [ ] Run baseline benchmarks: `cargo bench --bench <name> -- --save-baseline main`
- [ ] Profile with criterion: Results in `target/criterion/report/index.html`
- [ ] Check for regressions: `cargo bench -- --baseline main`

### String-Heavy Code

- [ ] Can you use `&str` instead of `String`?
- [ ] Can you use `Arc<str>` for shared immutable strings?
- [ ] Can you use string interning (`Spur`) for identifiers?
- [ ] Are you cloning strings unnecessarily?

### Allocation-Heavy Code

- [ ] Can you use `Rc` or `Arc` instead of cloning?
- [ ] Can you implement Copy-on-Write semantics?
- [ ] Can you use an arena allocator for short-lived data?
- [ ] Are intermediate collections necessary?

### Parsing/Matching Code

- [ ] Can you cache the result?
- [ ] Can you skip parsing when not needed (lazy evaluation)?
- [ ] Can you use incremental parsing for edits?
- [ ] Can you parallelize with Rayon?

---

## Profiling Commands

### CPU Profiling
```bash
# Run benchmarks
cargo bench --bench performance_improvements

# Generate flamegraph (requires native Linux)
./scripts/profile.sh flamegraph performance_improvements
```

### Memory Profiling
```bash
# Integration with existing monitoring
cargo test --release --features monitoring

# Check allocation counts
cargo bench --bench performance_improvements -- --profile-time=10
```

### I/O Profiling
```bash
# Run database benchmarks
cargo bench --bench d1_integration_test
cargo bench --bench postgres_integration_test
```

---

## Performance Regression Detection

### CI Integration
```yaml
# .github/workflows/performance.yml
- name: Benchmark Performance
  run: |
    cargo bench --bench performance_improvements -- --save-baseline main
    cargo bench --bench performance_improvements -- --baseline main
    # Fail if >10% regression
```

### Local Validation
```bash
# Before committing changes
./scripts/performance-regression-test.sh
```

---

## When to Profile

### Profile Before Optimizing If:
- [ ] You're optimizing without measurement
- [ ] You're not sure where the bottleneck is
- [ ] You're making "obvious" optimizations

### Profile After Optimizing To:
- [ ] Verify the optimization worked
- [ ] Check for unexpected regressions
- [ ] Quantify the improvement

### Profile Continuously:
- [ ] In CI for every PR
- [ ] In production with telemetry
- [ ] Monthly comprehensive profiling

---

## Red Flags 🚨

### Performance Anti-Patterns

❌ **String allocation in loops**
```rust
for item in items {
    let s = format!("prefix_{}", item); // Allocates every iteration
}
```

✅ **Pre-allocate or reuse**
```rust
let mut buf = String::with_capacity(100);
for item in items {
    buf.clear();
    write!(buf, "prefix_{}", item).unwrap();
}
```

---

❌ **Cloning when not necessary**
```rust
fn process(data: String) { /* ... */ }
process(data.clone()); // Unnecessary clone
```

✅ **Use references**
```rust
fn process(data: &str) { /* ... */ }
process(&data);
```

---

❌ **Repeated parsing**
```rust
for _ in 0..1000 {
    let pattern = Pattern::new("function $F() {}", &lang); // Re-parses 1000 times
}
```

✅ **Cache compiled patterns**
```rust
let pattern = Pattern::new("function $F() {}", &lang); // Parse once
for _ in 0..1000 {
    let matches = root.find_all(&pattern); // Reuse
}
```

---

## Useful Profiling Tools

- **cargo-flamegraph**: CPU flamegraphs
- **criterion**: Benchmarking with statistical analysis
- **perf**: Native Linux profiler
- **valgrind/massif**: Heap profiling
- **heaptrack**: Allocation profiling
- **dhat-rs**: Rust heap profiling crate

---

**Version**: 1.0
**Maintainer**: Performance Engineering Team
**Related Docs**:
- `PERFORMANCE_PROFILING_REPORT.md` - Full profiling results
- `OPTIMIZATION_ROADMAP.md` - Prioritized optimization plan
- `crates/flow/src/monitoring/performance.rs` - Runtime metrics
