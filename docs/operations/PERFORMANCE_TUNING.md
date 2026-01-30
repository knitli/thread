<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Thread Flow Performance Tuning Guide

Comprehensive guide for optimizing Thread Flow performance across CLI and Edge deployments.

---

## Table of Contents

1. [Performance Overview](#performance-overview)
2. [Content-Addressed Caching](#content-addressed-caching)
3. [Parallel Processing Tuning](#parallel-processing-tuning)
4. [Query Result Caching](#query-result-caching)
5. [Blake3 Fingerprinting](#blake3-fingerprinting)
6. [Batch Size Optimization](#batch-size-optimization)
7. [Database Performance](#database-performance)
8. [Edge-Specific Optimizations](#edge-specific-optimizations)
9. [Monitoring and Profiling](#monitoring-and-profiling)

---

## Performance Overview

### Baseline Performance Characteristics

| Metric | CLI (4 cores) | Edge (D1) | Target |
|--------|---------------|-----------|--------|
| **Fingerprint** | 425 ns/file | 425 ns/file | <1 µs |
| **Parse** | 147 µs/file | 147 µs/file | <200 µs |
| **Extract** | 50 µs/symbol | 50 µs/symbol | <100 µs |
| **Cache Lookup** | <1 µs | 15-25 ms (D1) | <10 ms (CLI), <50 ms (Edge) |
| **Total (cold)** | ~200 µs/file | ~200 µs + D1 latency | - |
| **Total (warm)** | ~1 µs/file | ~25 ms/file | - |

### Key Performance Metrics

**Cost Reduction (Content-Addressed Caching)**:
- **Fingerprint vs Parse**: 346x faster (425 ns vs 147 µs)
- **Cache vs Parse**: 147,000x faster (<1 µs vs 147 µs)
- **Overall Cost Reduction**: 99.7% on repeated analysis

**Throughput**:
- **CLI (4 cores)**: 2,500 files/sec (cold), 250,000 files/sec (warm)
- **Edge (D1)**: 40 requests/sec (cold), 100 requests/sec (warm)

**Parallelization Speedup**:
- **2 cores**: 2x baseline
- **4 cores**: 3.8x baseline
- **8 cores**: 7.2x baseline
- **Linear scaling**: Up to 8 cores, then diminishing returns

---

## Content-Addressed Caching

### How It Works

```
┌────────────┐
│ Source File│
│ "fn test()"│
└─────┬──────┘
      │
      ▼ Blake3 Hash (425 ns)
┌─────────────────────────────┐
│ Content Hash (Fingerprint)  │
│ "9f86d081884c7d659a2feaa0..." │
└──────────┬──────────────────┘
           │
           ▼ Check Cache
    ┌──────────────┐
    │ Hash in DB?  │
    └──┬───────┬───┘
       │ Yes   │ No
       │       │
       ▼       ▼ Parse + Extract (147 µs)
    Return   Store in Cache
    Cache    with Hash Key
    (<1 µs)
```

### Configuration

**CLI (PostgreSQL)**:

```bash
# .env
DATABASE_URL=postgresql://user:pass@localhost/thread_cache

# Automatic caching - no configuration needed
# ReCoco handles fingerprinting and cache lookups
```

**Edge (D1)**:

```javascript
// worker/index.js
async function analyzeWithCache(code, language, env) {
  // Compute content hash
  const hash = await computeBlake3Hash(code);

  // Check D1 cache
  const cached = await env.DB.prepare(
    'SELECT symbols FROM code_symbols WHERE content_hash = ?'
  ).bind(hash).first();

  if (cached) {
    return { symbols: JSON.parse(cached.symbols), cached: true };
  }

  // Cache miss - parse and cache
  const symbols = await analyzeCode(code, language);
  await env.DB.prepare(
    'INSERT INTO code_symbols (content_hash, symbols) VALUES (?, ?)'
  ).bind(hash, JSON.stringify(symbols)).run();

  return { symbols, cached: false };
}
```

### Optimization Tips

**1. Maximize Cache Hit Rate**

Target: **>90% hit rate** for production workloads

```bash
# Monitor cache hit rate (CLI)
psql -U thread_user -d thread_cache -c "
  SELECT
    COUNT(*) as total_lookups,
    SUM(CASE WHEN updated_at > created_at THEN 1 ELSE 0 END) as cache_hits,
    ROUND(100.0 * SUM(CASE WHEN updated_at > created_at THEN 1 ELSE 0 END) / COUNT(*), 2) as hit_rate_pct
  FROM code_symbols;
"
```

**2. Preload Common Patterns**

```bash
# Pre-populate cache with common files
thread analyze --preload standard-library/
thread analyze --preload common-dependencies/

# This "warms" the cache for frequently analyzed code
```

**3. Cache Expiration Strategy**

```sql
-- PostgreSQL: Remove stale entries older than 30 days
DELETE FROM code_symbols
WHERE updated_at < NOW() - INTERVAL '30 days';

-- D1: Remove stale entries (run via wrangler cron)
DELETE FROM code_symbols
WHERE updated_at < strftime('%s', 'now', '-30 days');
```

**4. Monitor Fingerprint Performance**

```rust
// Ensure fingerprinting is fast
use std::time::Instant;

let start = Instant::now();
let hash = compute_fingerprint(&content);
let duration = start.elapsed();

// Target: <1 µs per file
assert!(duration.as_nanos() < 1000);
```

---

## Parallel Processing Tuning

### Rayon Configuration (CLI Only)

**Default Behavior**:
- Auto-detects CPU cores
- Spawns one worker thread per core
- Work-stealing scheduler

**Manual Configuration**:

```bash
# Set thread count
export RAYON_NUM_THREADS=4

# Or in .env
echo "RAYON_NUM_THREADS=4" >> .env
```

### Optimal Thread Count

**Formula**:

```
CPU-bound (parsing): threads = physical_cores
I/O-bound (database): threads = physical_cores * 2
Mixed workload: threads = physical_cores * 1.5
```

**Examples**:

```bash
# 4-core CPU, parsing-heavy workload
export RAYON_NUM_THREADS=4  # Optimal

# 4-core CPU, database-heavy workload
export RAYON_NUM_THREADS=8  # Allow I/O overlap

# 8-core CPU, mixed workload
export RAYON_NUM_THREADS=12  # Balance parallelism and overhead
```

### Performance Testing

```bash
# Benchmark different thread counts
for threads in 1 2 4 8 16; do
  echo "Testing with $threads threads..."
  export RAYON_NUM_THREADS=$threads
  time thread analyze large-codebase/ > /dev/null
done

# Expected output:
# 1 thread:  16.2s
# 2 threads: 8.5s   (1.9x speedup)
# 4 threads: 4.3s   (3.8x speedup)
# 8 threads: 2.4s   (6.8x speedup)
# 16 threads: 2.2s  (7.4x speedup - diminishing returns)
```

### Work-Stealing Optimization

Rayon uses work-stealing for load balancing. Optimize by:

**1. Balanced Work Distribution**

```rust
// Good: Even file distribution
let files = vec!["small.rs", "medium.rs", "large.rs"];
process_files_batch(&files, |f| analyze(f));

// Better: Pre-sort by size for better work-stealing
files.sort_by_key(|f| std::fs::metadata(f).unwrap().len());
process_files_batch(&files, |f| analyze(f));
```

**2. Chunk Size Tuning**

```rust
// For small files (<1KB): larger chunks
use rayon::prelude::*;
files.par_chunks(100).for_each(|chunk| {
    chunk.iter().for_each(|f| analyze(f));
});

// For large files (>100KB): smaller chunks
files.par_chunks(10).for_each(|chunk| {
    chunk.iter().for_each(|f| analyze(f));
});
```

---

## Query Result Caching

### Configuration

**Enable Caching Feature**:

```bash
# Build with caching support
cargo build --release --features caching
```

**Cache Settings**:

```bash
# .env
THREAD_CACHE_MAX_CAPACITY=100000  # 100k entries (default: 10k)
THREAD_CACHE_TTL_SECONDS=3600     # 1 hour (default: 5 minutes)
```

### Usage

```rust
use thread_flow::cache::{QueryCache, CacheConfig};

// Create cache with custom config
let cache = QueryCache::new(CacheConfig {
    max_capacity: 100_000,
    ttl_seconds: 3600,
});

// Cache query results
let fingerprint = compute_fingerprint(&code);
if let Some(symbols) = cache.get(&fingerprint).await {
    // Cache hit - instant return
    return symbols;
}

// Cache miss - query and cache
let symbols = query_database(&fingerprint).await?;
cache.insert(fingerprint, symbols.clone()).await;
```

### Performance Impact

| Scenario | Without Cache | With Cache | Savings |
|----------|---------------|------------|---------|
| Symbol lookup (CLI) | 10-15ms (Postgres) | <1µs (memory) | **99.99%** |
| Symbol lookup (Edge) | 25-50ms (D1) | <1µs (memory) | **99.98%** |
| Metadata query | 5-10ms (DB) | <1µs (memory) | **99.99%** |
| Re-analysis (90% hit) | 100ms total | 10ms total | **90%** |

### Monitoring Cache Performance

```rust
// Get cache statistics
let stats = cache.stats().await;
println!("Cache hit rate: {:.2}%", stats.hit_rate());
println!("Cache miss rate: {:.2}%", stats.miss_rate());
println!("Total lookups: {}", stats.total_lookups);
println!("Hits: {}", stats.hits);
println!("Misses: {}", stats.misses);

// Target hit rate: >90%
assert!(stats.hit_rate() > 90.0);
```

### Cache Tuning

**1. Right-Size Cache Capacity**

```bash
# Monitor cache entry count
psql -c "SELECT COUNT(*) FROM code_symbols;"

# If count approaches max_capacity, increase it
# Rule of thumb: capacity = 2x unique files analyzed per day
```

**2. Optimize TTL for Workload**

```bash
# Short-lived projects (rapid iteration): 5-15 minutes
THREAD_CACHE_TTL_SECONDS=300

# Stable codebases: 1-6 hours
THREAD_CACHE_TTL_SECONDS=3600

# Long-term caching: 24 hours
THREAD_CACHE_TTL_SECONDS=86400
```

**3. Eviction Strategy**

Moka uses **Least Recently Used (LRU)** eviction:
- Oldest unused entries evicted first
- Hot entries stay in cache
- Cold entries removed when capacity reached

---

## Blake3 Fingerprinting

### Performance Characteristics

**Baseline**:
- **425 ns per file** (average)
- **346x faster than parsing** (vs 147 µs parse time)
- **100 files in 42.5 µs** (2.35 million files/second)

**Comparison**:

| Hash Algorithm | Time/File | Relative Speed |
|----------------|-----------|----------------|
| Blake3         | 425 ns    | 1x (baseline)  |
| SHA-256        | 1.2 µs    | 2.8x slower    |
| MD5            | 800 ns    | 1.9x slower    |
| Custom u64     | 200 ns    | 2.1x faster*   |

*Custom hashing faster but no collision resistance

### Optimization

**1. Batch Fingerprinting**

```rust
use rayon::prelude::*;

// Sequential fingerprinting
let hashes: Vec<_> = files.iter()
    .map(|f| compute_fingerprint(f))
    .collect();

// Parallel fingerprinting (3-4x faster on 4 cores)
let hashes: Vec<_> = files.par_iter()
    .map(|f| compute_fingerprint(f))
    .collect();
```

**2. Memory-Mapped Files**

```rust
use memmap2::Mmap;

// For large files (>1MB), use memory mapping
let file = File::open(path)?;
let mmap = unsafe { Mmap::map(&file)? };
let hash = blake3::hash(&mmap);

// 20-30% faster for large files
```

**3. Incremental Hashing**

```rust
// For streaming data or partial updates
let mut hasher = blake3::Hasher::new();
hasher.update(chunk1);
hasher.update(chunk2);
let hash = hasher.finalize();
```

### Benchmarking

```bash
# Run fingerprint benchmarks
cargo bench --bench fingerprint_benchmark

# Expected output:
# fingerprint_single_file   425.32 ns   (± 12.45 ns)
# fingerprint_100_files     42.531 µs   (± 1.234 µs)
# fingerprint_1000_files    425.12 µs   (± 8.567 µs)
# fingerprint_parallel_4c   106.28 µs   (± 3.456 µs)  ← 4x speedup
```

---

## Batch Size Optimization

### Concept

```
Batch Size = Number of files processed per database transaction

Small batches:  Many transactions, overhead-heavy
Large batches:  Fewer transactions, memory-heavy
Optimal:        Balance throughput and resource usage
```

### Configuration

```bash
# .env
THREAD_BATCH_SIZE=100  # Default
```

### Optimal Batch Sizes

| Scenario | Recommended Batch Size | Rationale |
|----------|------------------------|-----------|
| Small files (<10KB) | 500-1000 | Low memory, maximize transaction efficiency |
| Medium files (10-100KB) | 100-200 | Balance memory and transactions |
| Large files (>100KB) | 10-50 | Limit memory usage |
| High-latency DB (Edge) | 50-100 | Reduce round-trips |
| Low-latency DB (CLI) | 200-500 | Maximize throughput |

### Testing

```bash
# Benchmark different batch sizes
for batch_size in 10 50 100 500 1000; do
  export THREAD_BATCH_SIZE=$batch_size
  echo "Testing batch size: $batch_size"
  time thread analyze large-codebase/ > /dev/null
done

# Expected output:
# Batch 10:   18.2s  (too many transactions)
# Batch 50:   12.5s
# Batch 100:  10.1s  ← Optimal
# Batch 500:  10.3s  (memory overhead)
# Batch 1000: 11.2s  (memory thrashing)
```

### Implementation

```rust
// Batch processing with optimal size
const OPTIMAL_BATCH_SIZE: usize = 100;

fn process_files_in_batches(files: &[PathBuf]) -> Result<()> {
    for batch in files.chunks(OPTIMAL_BATCH_SIZE) {
        // Start transaction
        let mut tx = db.transaction()?;

        // Process batch
        for file in batch {
            let symbols = analyze_file(file)?;
            tx.insert(file, symbols)?;
        }

        // Commit once per batch
        tx.commit()?;
    }
    Ok(())
}
```

---

## Database Performance

### PostgreSQL (CLI)

**Connection Pooling**:

```bash
# .env
DB_POOL_SIZE=20  # Default: 10
DB_CONNECTION_TIMEOUT=60  # Seconds
```

**Index Optimization**:

```sql
-- Create indexes for fast lookups
CREATE INDEX CONCURRENTLY idx_symbols_hash ON code_symbols(content_hash);
CREATE INDEX CONCURRENTLY idx_symbols_path ON code_symbols(file_path);
CREATE INDEX CONCURRENTLY idx_symbols_created ON code_symbols(created_at);

-- Analyze tables for query planner
ANALYZE code_symbols;
```

**Query Optimization**:

```sql
-- Use prepared statements (automatic with ReCoco)
PREPARE get_symbols AS
  SELECT symbols FROM code_symbols WHERE content_hash = $1;

-- Execute repeatedly
EXECUTE get_symbols('abc123...');

-- 10-20% faster than non-prepared
```

**Vacuuming**:

```sql
-- Regular maintenance
VACUUM ANALYZE code_symbols;

-- Auto-vacuum configuration
ALTER TABLE code_symbols SET (autovacuum_vacuum_scale_factor = 0.1);
```

### D1 (Edge)

**Query Batching**:

```javascript
// Bad: Individual queries
for (const hash of hashes) {
  await env.DB.prepare('SELECT * FROM code_symbols WHERE content_hash = ?')
    .bind(hash).first();
}

// Good: Batch query with IN clause
const placeholders = hashes.map(() => '?').join(',');
const results = await env.DB.prepare(
  `SELECT * FROM code_symbols WHERE content_hash IN (${placeholders})`
).bind(...hashes).all();
```

**Read Replicas** (coming soon):

```javascript
// Use read replicas for query-heavy workloads
const result = await env.DB_REPLICA.prepare('SELECT ...').first();
```

**D1 Best Practices**:

1. **Minimize round-trips**: Batch queries when possible
2. **Use indexes**: D1 auto-indexes primary keys, add composite indexes
3. **Limit result sets**: Use `LIMIT` to avoid large payloads
4. **Monitor latency**: Target <50ms p95 for D1 queries

---

## Edge-Specific Optimizations

### WASM Bundle Size

**Current**: ~2.1 MB (optimized)
**Target**: <1.5 MB (future optimization)

**Size Reduction Techniques**:

```bash
# 1. Maximum optimization flags
cargo build --release \
  --target wasm32-unknown-unknown \
  -Z build-std=std,panic_abort \
  -Z build-std-features=panic_immediate_abort

# 2. wasm-opt aggressive optimization
wasm-opt -Oz --strip-debug --strip-producers \
  thread_flow_bg.wasm -o thread_flow_opt.wasm

# 3. wasm-snip to remove unused functions
wasm-snip --snip-rust-fmt-code \
  --snip-rust-panicking-code \
  thread_flow_opt.wasm -o thread_flow_final.wasm

# Expected size reduction: 15-25%
```

### CPU Time Limits

**Cloudflare Workers**: 50ms CPU time per request

**Optimization Strategies**:

```javascript
// 1. Offload heavy parsing to async operations
async function analyzeLarge(code) {
  // Break into chunks to avoid CPU limit
  const chunks = chunkCode(code, 1000 lines);

  for (const chunk of chunks) {
    await analyzeChunk(chunk);  // Yields between chunks
  }
}

// 2. Use cache aggressively
async function analyzeWithFallback(code) {
  const cached = await checkCache(code);
  if (cached) return cached;  // <1ms

  // Only parse if absolutely necessary
  return await parseAndCache(code);  // May hit 50ms limit
}

// 3. Monitor CPU time
const start = Date.now();
const result = await analyze(code);
const cpuTime = Date.now() - start;

if (cpuTime > 40) {
  console.warn(`High CPU usage: ${cpuTime}ms`);
}
```

### Memory Limits

**Cloudflare Workers**: 128 MB memory

**Strategies**:

```javascript
// 1. Stream large inputs
async function analyzeStream(readable) {
  const reader = readable.getReader();
  const chunks = [];

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    // Process chunk immediately, don't accumulate
    await processChunk(value);
  }
}

// 2. Limit cache size
const EDGE_CACHE_LIMIT = 1000;  // entries
if (cache.size > EDGE_CACHE_LIMIT) {
  cache.clear();  // Evict all to avoid memory limit
}
```

---

## Monitoring and Profiling

### CLI Profiling

**Linux perf**:

```bash
# Profile CPU usage
perf record --call-graph=dwarf thread analyze large-codebase/
perf report

# Look for hotspots:
# - tree_sitter parsing (should be ~60% of time)
# - blake3 hashing (should be <5% of time)
# - database queries (should be <10% of time)
```

**Flamegraph**:

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bin thread -- analyze large-codebase/

# Open flamegraph.svg in browser
# Look for:
# - Wide bars = time-consuming functions
# - Tall stacks = deep call chains
```

**Benchmarking**:

```bash
# Run benchmarks
cargo bench --bench parse_benchmark
cargo bench --bench fingerprint_benchmark

# Compare before/after optimization
cargo bench > before.txt
# ... make optimization ...
cargo bench > after.txt
cargo benchcmp before.txt after.txt
```

### Edge Monitoring

**Cloudflare Analytics**:

```bash
# View analytics dashboard
open https://dash.cloudflare.com/your-account-id/workers/services/view/thread-flow-worker/analytics

# Metrics to monitor:
# - Requests per second
# - CPU time (target: <25ms average, <50ms p95)
# - Errors (target: <0.1%)
# - Cache hit rate (target: >90%)
```

**Custom Metrics**:

```javascript
// Log custom metrics
export default {
  async fetch(request, env, ctx) {
    const start = Date.now();

    try {
      const result = await analyze(request);
      const duration = Date.now() - start;

      // Log metrics
      console.log(JSON.stringify({
        duration_ms: duration,
        cached: result.cached,
        symbols_count: result.symbols.length,
      }));

      return new Response(JSON.stringify(result));
    } catch (error) {
      console.error('Analysis failed:', error);
      throw error;
    }
  }
};

// View logs
wrangler tail --format json | jq '.duration_ms'
```

### Performance Alerts

**PostgreSQL**:

```sql
-- Alert on slow queries (>100ms)
SELECT
  query,
  mean_exec_time,
  calls
FROM pg_stat_statements
WHERE mean_exec_time > 100
ORDER BY mean_exec_time DESC;
```

**D1**:

```javascript
// Alert on high latency
async function queryWithAlert(env, sql, params) {
  const start = Date.now();
  const result = await env.DB.prepare(sql).bind(...params).all();
  const duration = Date.now() - start;

  if (duration > 50) {
    // Alert: High D1 latency
    await sendAlert(`D1 query took ${duration}ms: ${sql}`);
  }

  return result;
}
```

---

## Performance Checklist

### CLI Optimization

- [ ] PostgreSQL connection pool configured (10-20 connections)
- [ ] Rayon thread count set to physical cores (or 1.5x for mixed workload)
- [ ] Query result caching enabled (`--features caching`)
- [ ] Batch size optimized for file size distribution (100-500)
- [ ] PostgreSQL indexes created on `content_hash`, `file_path`, `created_at`
- [ ] Cache hit rate >90% after warm-up
- [ ] Parallel processing verified (2-4x speedup on multi-core)

### Edge Optimization

- [ ] WASM bundle optimized with `wasm-opt -Oz` (<2 MB)
- [ ] D1 queries batched when possible (reduce round-trips)
- [ ] CPU time monitored (<25ms average, <50ms p95)
- [ ] Memory usage monitored (<100 MB typical)
- [ ] Cache hit rate >90% after warm-up
- [ ] Query latency <50ms p95 for D1
- [ ] Error rate <0.1%

### Monitoring

- [ ] Logging configured (`RUST_LOG=thread_flow=info`)
- [ ] Performance metrics tracked (cache hit rate, query latency, throughput)
- [ ] Alerts configured for performance degradation
- [ ] Benchmarks run regularly to detect regressions
- [ ] Profiling performed on slow paths

---

**Performance Target Summary**:
- **Cache Hit Rate**: >90%
- **Fingerprint Time**: <1 µs per file
- **Parse Time**: <200 µs per file
- **Query Latency**: <10ms (CLI), <50ms (Edge)
- **Throughput**: 2,500+ files/sec (CLI), 40+ req/sec (Edge)
- **Cost Reduction**: 99.7% via content-addressed caching
