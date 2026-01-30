<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Performance Optimization Guide

**Version**: 1.0
**Last Updated**: 2026-01-28

---

## Table of Contents

- [Overview](#overview)
- [Performance Profiling](#performance-profiling)
- [Load Testing](#load-testing)
- [Optimization Strategies](#optimization-strategies)
- [Monitoring & Metrics](#monitoring--metrics)
- [Capacity Planning](#capacity-planning)
- [Best Practices](#best-practices)

---

## Overview

Thread's performance optimization framework combines profiling tools, load testing, continuous monitoring, and systematic optimization strategies to achieve production-grade performance.

### Performance Philosophy

1. **Measure First**: Profile before optimizing
2. **Evidence-Based**: All optimizations backed by benchmarks
3. **Systematic**: Address hot paths systematically
4. **Continuous**: Monitor performance in production
5. **Practical**: Balance optimization effort with real-world impact

### Current Performance Baseline

| Metric | Value | Target |
|--------|-------|--------|
| **Fingerprint (Blake3)** | 425 ns | <1 µs |
| **Cache Hit Latency** | <1 µs | <10 µs |
| **Cache Miss Overhead** | 16 ns | <100 ns |
| **Content-Addressed Caching** | 99.7% reduction | >99% |
| **Parallel Speedup** | 2-4x (CLI) | >2x |
| **Query Latency (p95)** | <50 ms | <50 ms |

### Performance Improvements Timeline

**Day 15** (Foundation):
- Blake3 fingerprinting (346x faster than parsing)
- Content-addressed caching
- Query result caching
- Parallel batch processing

**Day 23** (Optimization):
- Advanced profiling tools
- Load testing framework
- Performance monitoring integration
- Comprehensive optimization documentation

---

## Performance Profiling

### Profiling Tools

Thread provides comprehensive profiling infrastructure via `scripts/profile.sh`:

```bash
# Quick flamegraph profiling
./scripts/profile.sh quick

# Full profiling suite
./scripts/profile.sh comprehensive

# Specific benchmark flamegraph
./scripts/profile.sh flamegraph fingerprint_benchmark

# Linux perf profiling
./scripts/profile.sh perf fingerprint_benchmark 30

# Memory profiling with valgrind
./scripts/profile.sh memory cache

# Heap profiling with heaptrack
./scripts/profile.sh heap fingerprint_benchmark
```

### Profiling Workflow

#### 1. Baseline Profiling

**Before any optimization**:

```bash
# Establish baseline with flamegraph
./scripts/profile.sh flamegraph fingerprint_benchmark

# Run benchmarks
cargo bench -p thread-flow

# Record baseline metrics
cat target/criterion/*/report/index.html
```

#### 2. Identify Hot Paths

**Analyze flamegraph**:
- Look for wide horizontal bars (time-intensive functions)
- Identify recursive patterns
- Find unexpected call stacks
- Locate allocation hot spots

**Key Questions**:
- What functions consume >10% of CPU time?
- Are there unnecessary allocations in hot paths?
- Can we avoid string conversions or clones?
- Are there O(n²) algorithms that could be O(n log n)?

#### 3. Profile-Guided Optimization

**Use perf for detailed analysis** (Linux):

```bash
# Record performance data
./scripts/profile.sh perf fingerprint_benchmark 60

# Analyze with perf report
perf report -i target/profiling/perf.data

# Look for:
# - Cache misses (perf stat -e cache-misses)
# - Branch mispredictions
# - TLB misses
# - CPU cycles per instruction
```

#### 4. Memory Profiling

**Identify memory issues**:

```bash
# Heap profiling
./scripts/profile.sh heap fingerprint_benchmark

# Memory leaks (valgrind)
./scripts/profile.sh memory cache

# Look for:
# - Unnecessary allocations
# - Large heap usage
# - Memory leaks
# - Allocation/deallocation patterns
```

### Manual Profiling

#### CPU Profiling

```rust
use std::time::Instant;

// Time-critical section
let start = Instant::now();
let result = compute_expensive_operation();
let duration = start.elapsed();

eprintln!("Operation took: {:?}", duration);
```

#### Allocation Profiling

```rust
// Count allocations
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

// Print stats
malloc_stats_print();
```

---

## Load Testing

### Load Test Benchmarks

Thread includes comprehensive load testing in `crates/flow/benches/load_test.rs`:

```bash
# Run all load tests
cargo bench -p thread-flow --bench load_test --all-features

# Run specific load test category
cargo bench -p thread-flow --bench load_test -- large_codebase

# Run with profiling
cargo flamegraph --bench load_test --all-features
```

### Load Test Categories

#### 1. Large Codebase Fingerprinting

**Tests**: 100, 500, 1000, 2000 files

```bash
cargo bench --bench load_test -- large_codebase_fingerprinting
```

**Metrics**:
- Throughput (files/sec, bytes/sec)
- Linear scaling verification
- Memory usage under load

#### 2. Concurrent Processing

**Tests**: Sequential vs Parallel vs Batch

```bash
cargo bench --bench load_test --features parallel -- concurrent_processing
```

**Metrics**:
- Parallel speedup factor
- CPU utilization
- Thread contention

#### 3. Cache Patterns

**Tests**: 0%, 25%, 50%, 75%, 95%, 100% hit rates

```bash
cargo bench --bench load_test --features caching -- cache_patterns
```

**Metrics**:
- Cache hit latency
- Cache miss latency
- Hit rate impact on throughput

#### 4. Incremental Updates

**Tests**: 1%, 5%, 10%, 25%, 50% file changes

```bash
cargo bench --bench load_test -- incremental_updates
```

**Metrics**:
- Incremental update efficiency
- Cache reuse effectiveness
- Change detection overhead

#### 5. Realistic Workloads

**Tests**: Small (50 files), Medium (500 files), Large (2000 files) projects

```bash
cargo bench --bench load_test -- realistic_workloads
```

**Metrics**:
- End-to-end latency
- Resource usage
- Real-world performance

### Custom Load Tests

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use thread_services::conversion::compute_content_fingerprint;

fn bench_custom_workload(c: &mut Criterion) {
    let files = generate_test_data();

    c.bench_function("custom_workload", |b| {
        b.iter(|| {
            for file in &files {
                black_box(compute_content_fingerprint(file));
            }
        });
    });
}

criterion_group!(benches, bench_custom_workload);
criterion_main!(benches);
```

---

## Optimization Strategies

### 1. Fingerprinting Optimization

**Current**: Blake3 hashing at 425 ns/file (346x faster than parsing)

**Further Optimizations**:

```rust
// Use SIMD for large files
#[cfg(target_feature = "avx2")]
use blake3::Hasher;

// Batch fingerprinting
fn batch_fingerprint(files: &[&str]) -> Vec<Fingerprint> {
    files.par_iter()
        .map(|content| compute_content_fingerprint(content))
        .collect()
}
```

**Strategies**:
- Incremental hashing for streaming
- SIMD acceleration (AVX2, NEON)
- Parallel batch processing
- Memory-mapped file reading

### 2. Caching Optimization

**Current**: Content-addressed cache with 99.7% cost reduction

**Query Result Caching**:

```rust
use thread_flow::cache::QueryCache;
use std::time::Duration;

// Create cache with capacity and TTL
let cache = QueryCache::new(10_000, Duration::from_secs(3600));

// Cache query results
if let Some(result) = cache.get(&query_key) {
    return result.clone();
}

let result = execute_expensive_query(&query);
cache.insert(query_key, result.clone());
result
```

**Strategies**:
- Adaptive TTL based on update frequency
- LRU eviction for memory efficiency
- Multi-tier caching (L1: memory, L2: disk/D1)
- Cache warming for predictable access patterns

### 3. Parallel Processing Optimization

**Current**: 2-4x speedup with Rayon (CLI only)

**Batch Processing**:

```rust
use thread_flow::batch::process_files_batch;

let results = process_files_batch(&file_paths, |path| {
    analyze_file(path)
});
```

**Strategies**:
- Work stealing for load balancing
- Chunk size tuning (avoid overhead)
- CPU affinity for cache locality
- Async I/O for Edge deployment

### 4. Memory Optimization

**Current**: <1KB overhead per cached file

**Strategies**:

```rust
// Use compact data structures
use bit_set::BitSet;  // vs HashSet<usize>
use tinyvec::TinyVec;  // vs Vec for small collections

// Avoid unnecessary allocations
fn process_str(s: &str) -> &str {  // vs String
    // Return slice, not owned String
    &s[..]
}

// Use Cow for conditional allocation
use std::borrow::Cow;

fn maybe_transform(s: &str) -> Cow<str> {
    if needs_transform(s) {
        Cow::Owned(transform(s))
    } else {
        Cow::Borrowed(s)
    }
}
```

**Memory Profiling**:

```bash
# Heap profiling
./scripts/profile.sh heap fingerprint_benchmark

# Memory usage over time
valgrind --tool=massif target/release/thread-flow

# Analyze massif output
ms_print massif.out.* > memory-report.txt
```

### 5. Database Query Optimization

**Postgres** (CLI):

```sql
-- Index fingerprints for fast lookups
CREATE INDEX idx_fingerprint ON code_analysis(fingerprint);

-- Batch inserts
INSERT INTO code_analysis (fingerprint, content, symbols)
VALUES
  ($1, $2, $3),
  ($4, $5, $6),
  ... -- batch of 100-1000
ON CONFLICT (fingerprint) DO NOTHING;

-- Use prepared statements
PREPARE insert_analysis (text, text, jsonb) AS
  INSERT INTO code_analysis VALUES ($1, $2, $3);
```

**D1** (Edge):

```typescript
// Batch operations
await env.DB.batch([
  env.DB.prepare("INSERT INTO ...").bind(...),
  env.DB.prepare("INSERT INTO ...").bind(...),
  // ... up to 100 statements
]);

// Use indexes
-- Create in schema.sql
CREATE INDEX idx_fingerprint ON code_analysis(fingerprint);
```

### 6. WASM Optimization

**Edge Deployment**:

```toml
[profile.wasm-release]
inherits = "release"
opt-level = "z"  # Optimize for size
lto = true
codegen-units = 1
panic = "abort"  # Smaller binary
strip = true
```

**Build Optimization**:

```bash
# Size-optimized WASM build
cargo run -p xtask build-wasm --release

# Analyze WASM binary size
wasm-opt -Oz -o optimized.wasm original.wasm
twiggy top optimized.wasm
```

---

## Monitoring & Metrics

### Performance Metrics Collection

```rust
use thread_flow::monitoring::performance::PerformanceMetrics;

let metrics = PerformanceMetrics::new();

// Record fingerprint computation
let timer = PerformanceTimer::start(&metrics, MetricType::Fingerprint);
compute_fingerprint(content);
timer.stop_success();

// Record cache hit/miss
metrics.record_cache_hit();
metrics.record_cache_miss();

// Record query execution
metrics.record_query(duration, success);

// Get statistics
let stats = metrics.fingerprint_stats();
println!("Avg fingerprint time: {}ns", stats.avg_duration_ns);

let cache_stats = metrics.cache_stats();
println!("Cache hit rate: {:.2}%", cache_stats.hit_rate_percent);
```

### Prometheus Integration

**Export Metrics**:

```rust
// HTTP endpoint for Prometheus scraping
async fn metrics_handler(metrics: Arc<PerformanceMetrics>) -> String {
    metrics.export_prometheus()
}
```

**Prometheus Queries**:

```promql
# Cache hit rate (target: >90%)
rate(thread_cache_hits_total[5m]) /
  (rate(thread_cache_hits_total[5m]) + rate(thread_cache_misses_total[5m]))

# Average fingerprint time
rate(thread_fingerprint_duration_seconds[5m]) /
  rate(thread_fingerprint_total[5m])

# Query latency p95
histogram_quantile(0.95, thread_query_duration_seconds)

# Throughput
rate(thread_files_processed_total[1m])
```

### Grafana Dashboards

**Key Metrics Panels**:

1. **Cache Performance**:
   - Hit rate gauge (target line at 90%)
   - Hit/miss counters
   - Eviction rate

2. **Latency Distribution**:
   - Fingerprint p50/p95/p99
   - Query p50/p95/p99
   - Parse time distribution

3. **Throughput**:
   - Files processed per second
   - Bytes processed per second
   - Batches per minute

4. **Error Tracking**:
   - Error rate percentage
   - Errors by type
   - Error count over time

---

## Capacity Planning

### Resource Requirements

#### CLI Deployment

**Small Projects** (<100 files):
- **CPU**: 1-2 cores
- **Memory**: 512 MB - 1 GB
- **Disk**: 1 GB
- **Expected Performance**: <1s total analysis

**Medium Projects** (100-1000 files):
- **CPU**: 2-4 cores
- **Memory**: 1-2 GB
- **Disk**: 5 GB
- **Expected Performance**: 1-10s total analysis

**Large Projects** (1000-10000 files):
- **CPU**: 4-8 cores
- **Memory**: 2-4 GB
- **Disk**: 20 GB
- **Expected Performance**: 10-60s total analysis

#### Edge Deployment (Cloudflare Workers)

**Per Request**:
- **CPU Time**: 10-50 ms
- **Memory**: 128 MB limit
- **Execution Time**: <50 ms (sub-request)
- **Concurrent Requests**: Auto-scaling

**Resource Limits**:
- CPU time: 50 ms (startup), 50 ms (per request)
- Memory: 128 MB
- Requests/min: 1000 (free), 10M (paid)

### Scaling Strategies

#### Vertical Scaling (CLI)

**CPU Scaling**:
```rust
// Configure parallel threads
use rayon::ThreadPoolBuilder;

ThreadPoolBuilder::new()
    .num_threads(num_cpus::get())
    .build_global()
    .unwrap();
```

**Memory Scaling**:
```rust
// Tune cache capacity
let cache = QueryCache::new(
    capacity: usize,  // based on available RAM
    ttl: Duration::from_secs(3600),
);
```

#### Horizontal Scaling (Edge)

**Request Distribution**:
- Cloudflare automatically distributes to nearest edge
- No configuration needed
- Geographic load balancing built-in

**Database Scaling**:
- D1 automatic replication
- Read replicas at each edge location
- Eventual consistency model

### Performance Testing Under Load

**Stress Testing**:

```bash
# Generate large test corpus
./scripts/generate-test-data.sh 10000  # 10K files

# Run benchmarks under memory pressure
cargo bench --bench load_test -- large_project_10000_files

# Monitor resource usage
htop  # or similar
```

**Capacity Validation**:

1. **Determine peak load**: Max files processed per minute
2. **Measure resource usage**: CPU, memory, I/O at peak
3. **Calculate headroom**: Target 50% max resource usage
4. **Plan scaling**: When to add resources

---

## Best Practices

### 1. Profile Before Optimizing

```bash
# Always establish baseline
cargo bench --bench fingerprint_benchmark > baseline.txt

# Make changes
# ...

# Verify improvement
cargo bench --bench fingerprint_benchmark > optimized.txt
diff baseline.txt optimized.txt
```

### 2. Optimize Hot Paths First

**Focus on**:
- Functions consuming >10% CPU time
- Tight loops (>1000 iterations)
- Allocations in hot paths
- String operations and conversions

**Ignore**:
- One-time initialization code
- Error handling paths
- Debug/logging code (unless excessive)

### 3. Use Feature Flags for Optimization

```toml
[features]
default = ["parallel"]
parallel = ["dep:rayon"]  # CLI optimization
caching = ["dep:moka"]    # Optional caching
simd = []                 # SIMD optimizations
```

```rust
#[cfg(feature = "parallel")]
fn process_parallel(files: &[&str]) {
    files.par_iter().for_each(|f| process(f));
}

#[cfg(not(feature = "parallel"))]
fn process_parallel(files: &[&str]) {
    files.iter().for_each(|f| process(f));
}
```

### 4. Benchmark Regression Testing

**CI Integration**:

```yaml
# .github/workflows/ci.yml
- name: Run benchmarks
  run: cargo bench --workspace -- --save-baseline main

- name: Compare with baseline
  run: |
    cargo bench --workspace -- --baseline main
    # Fail if regression >10%
```

### 5. Monitor in Production

**Essential Metrics**:
- Cache hit rate (>90% target)
- Query latency p95 (<50ms)
- Throughput (files/sec)
- Error rate (<1%)

**Alerts**:

```yaml
# Prometheus alerting rules
groups:
  - name: performance
    rules:
      - alert: LowCacheHitRate
        expr: thread_cache_hit_rate < 90
        for: 5m

      - alert: HighQueryLatency
        expr: thread_query_latency_p95 > 50
        for: 5m
```

### 6. Document Optimization Decisions

**Performance Notes**:

```rust
/// Compute fingerprint using Blake3
///
/// # Performance
/// - Average: 425 ns per file
/// - Throughput: 430-672 MiB/s
/// - 346x faster than parsing
///
/// # Optimization History
/// - v1.0: Custom u64 hash (slower)
/// - v1.1: Switched to Blake3 (current)
/// - Future: Consider xxHash for non-crypto use
pub fn compute_content_fingerprint(content: &str) -> Fingerprint {
    // ...
}
```

---

## Performance Checklist

### Development

- [ ] Profile before optimizing
- [ ] Write benchmarks for hot paths
- [ ] Use criterion for microbenchmarks
- [ ] Test with realistic data sizes
- [ ] Verify improvements with flamegraphs

### Pre-Release

- [ ] Run full benchmark suite
- [ ] Compare with baseline performance
- [ ] Verify no regressions (>10% slowdown)
- [ ] Update performance documentation
- [ ] Test under load (stress testing)

### Production

- [ ] Enable performance monitoring
- [ ] Set up Prometheus scraping
- [ ] Configure Grafana dashboards
- [ ] Define performance SLOs
- [ ] Set up performance alerts
- [ ] Document capacity planning

---

## Troubleshooting

### Performance Degradation

**Symptoms**: Slower than expected

**Diagnosis**:

```bash
# Profile to find hot paths
./scripts/profile.sh comprehensive

# Check for regressions
cargo bench --workspace -- --baseline production

# Memory issues?
./scripts/profile.sh memory cache
```

**Common Causes**:
- Disabled caching (check features)
- Sequential processing on multi-core (check `parallel` feature)
- Cache thrashing (increase capacity)
- Database connection issues (check pool)

### High Memory Usage

**Symptoms**: OOM errors or high RSS

**Diagnosis**:

```bash
# Heap profiling
./scripts/profile.sh heap fingerprint_benchmark

# Check for leaks
valgrind --leak-check=full ./target/release/thread-flow
```

**Common Causes**:
- Large cache capacity
- Unbounded vector growth
- String cloning in hot paths
- Leaked connections

### Low Cache Hit Rate

**Symptoms**: <90% cache hit rate

**Diagnosis**:

```rust
let stats = metrics.cache_stats();
println!("Hits: {}, Misses: {}, Rate: {:.2}%",
    stats.hits, stats.misses, stats.hit_rate_percent);
```

**Common Causes**:
- Cache capacity too small
- TTL too aggressive
- High eviction rate
- Changing file content frequently

---

## Resources

### Tools

- **cargo-flamegraph**: CPU profiling
- **criterion**: Benchmarking
- **perf**: Linux profiling
- **valgrind/massif**: Memory profiling
- **heaptrack**: Heap profiling
- **cargo-bloat**: Binary size analysis

### Documentation

- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Criterion Documentation](https://bheisler.github.io/criterion.rs/)
- [Blake3 Performance](https://github.com/BLAKE3-team/BLAKE3)
- [Rayon Documentation](https://docs.rs/rayon/)

### References

- [Thread Constitution v2.0.0](../../.specify/memory/constitution.md)
- [Day 15 Performance Analysis](../../.phase0-planning/DAY15_PERFORMANCE_ANALYSIS.md)
- [Monitoring Guide](./MONITORING.md)

---

**Last Updated**: 2026-01-28
**Review Cycle**: Monthly
**Next Review**: 2026-02-28
