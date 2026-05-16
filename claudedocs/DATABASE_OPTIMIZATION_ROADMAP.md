# Database Optimization Roadmap

## Overview

Systematic approach to achieving constitutional compliance for database performance in Thread.

---

## Constitutional Requirements

| Requirement | Target | Priority | Status |
|------------|--------|----------|--------|
| Content-addressed cache hit rate | >90% | CRITICAL | ❌ Not measured |
| D1 p95 latency | <50ms | CRITICAL | ❌ Not measured |
| Postgres p95 latency | <10ms | HIGH | ⚠️ N/A (not using yet) |
| Incremental updates | Affected components only | HIGH | ⚠️ Partial |

---

## Phase 1: Performance Instrumentation ✅ COMPLETE

**Status**: ✅ Complete (2026-01-28)
**Task**: #55

### Accomplishments
- Added `PerformanceMetrics` to `D1ExportContext`
- Instrumented all D1 query operations
- Updated all test fixtures
- 96/96 tests passing

### Metrics Now Available
- Query latency (per operation)
- Success/failure rates
- Query counts
- Error tracking

---

## Phase 2: Measurement & Validation 🔄 IN PROGRESS

**Status**: 🔄 In Progress
**Tasks**: #58 (benchmarks), #60 (compliance)

### Task #58: D1 Query Profiling Benchmarks

**Objective**: Measure actual D1 query performance

**Steps**:
1. Create benchmark suite (`crates/flow/benches/d1_query_bench.rs`)
2. Test scenarios:
   - Single query latency
   - Batch operation performance
   - Concurrent query handling
   - Cache hit/miss patterns
3. Generate percentile reports (p50, p95, p99)
4. Compare against constitutional requirements

**Deliverables**:
- Benchmark code with criterion
- Performance report with latency distribution
- Recommendations for optimization

**Estimated Time**: 4-6 hours

### Task #60: Constitutional Compliance Validation

**Objective**: Validate all database requirements

**Steps**:
1. Collect benchmark data from Task #58
2. Measure cache hit rates (after Task #57)
3. Document compliance status
4. Identify gaps and create remediation plans

**Deliverables**:
- Compliance report with evidence
- Gap analysis
- Remediation roadmap

**Estimated Time**: 2-3 hours

---

## Phase 3: Query Result Caching 📋 PLANNED

**Status**: 📋 Planned
**Task**: #57

**Objective**: Achieve >90% cache hit rate

### Implementation Plan

#### 3.1 Cache Integration Architecture

**Pattern**:
```rust
pub struct D1CachedContext {
    inner: D1ExportContext,
    query_cache: Arc<QueryCache<Fingerprint, QueryResult>>,
}

impl D1CachedContext {
    async fn query_symbols(&self, fingerprint: Fingerprint) -> Result<Vec<Symbol>> {
        self.query_cache.get_or_insert(fingerprint, || async {
            self.inner.execute_query(fingerprint).await
        }).await
    }
}
```

#### 3.2 Cache Configuration

**Settings**:
- Max capacity: 10,000 entries (tune based on workload)
- TTL: 3600 seconds (1 hour)
- Eviction: LRU policy
- Metrics: Hit rate, eviction rate, entry count

#### 3.3 Cache Warming Strategy

**Approaches**:
1. **On-demand**: Populate cache as queries arrive (lazy loading)
2. **Preload**: Warm cache with common queries at startup
3. **Background refresh**: Update cache before TTL expiration

**Recommendation**: Start with on-demand, add preloading for production

#### 3.4 Invalidation Strategy

**Triggers**:
- Content change detection (fingerprint mismatch)
- Manual cache clear (admin operation)
- TTL expiration (automatic)

**Pattern**:
```rust
// Invalidate on content change
if new_fingerprint != cached_fingerprint {
    query_cache.invalidate(cached_fingerprint).await;
}
```

### Success Metrics
- [ ] Cache hit rate >90% in production workload
- [ ] p99 cache lookup latency <1ms
- [ ] Memory usage within bounds (<500MB for cache)
- [ ] Zero cache-related query errors

**Estimated Time**: 8-10 hours

---

## Phase 4: Schema & Index Optimization 📋 PLANNED

**Status**: 📋 Planned
**Task**: #56

**Objective**: Optimize D1 schema for common query patterns

### Analysis Areas

#### 4.1 Current Schema Review

**File**: `crates/flow/src/targets/d1.rs`

**Methods to Analyze**:
- `D1SetupState::create_table_sql()` - Table creation
- `D1SetupState::create_indexes_sql()` - Index creation
- `build_upsert_stmt()` - Upsert query patterns
- `build_delete_stmt()` - Delete query patterns

#### 4.2 Index Optimization

**Common Patterns to Index**:
1. **Key lookups**: Primary key columns (likely already indexed)
2. **Foreign keys**: Reference columns in WHERE clauses
3. **Filter columns**: Frequently used in WHERE/ORDER BY
4. **Composite indexes**: Multi-column queries

**Analysis Pattern**:
```rust
// Identify slow queries from benchmarks
// Add covering indexes for common patterns
CREATE INDEX idx_table_key_value ON table(key_col, value_col);
```

#### 4.3 Query Plan Analysis

**Tools**:
- SQLite EXPLAIN QUERY PLAN
- Cloudflare D1 query insights (if available)

**Process**:
1. Capture slow queries from benchmarks
2. Run EXPLAIN QUERY PLAN
3. Identify table scans (⚠️ bad)
4. Add indexes to enable index scans (✅ good)

### Deliverables
- [ ] Schema review document
- [ ] Index recommendations
- [ ] Query plan improvements
- [ ] Before/after performance comparison

**Estimated Time**: 4-6 hours

---

## Phase 5: Connection Pooling 📋 PLANNED

**Status**: 📋 Planned
**Task**: #59

**Objective**: Optimize HTTP client for D1 API calls

### Current Configuration

**File**: `crates/flow/src/targets/d1.rs` line 134

```rust
let http_client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(30))
    .build()?;
```

### Optimized Configuration

```rust
let http_client = reqwest::Client::builder()
    // Connection pooling
    .pool_max_idle_per_host(10)  // Reuse up to 10 connections per host
    .pool_idle_timeout(Duration::from_secs(90))  // Keep idle connections for 90s

    // Timeouts
    .timeout(Duration::from_secs(30))  // Total request timeout
    .connect_timeout(Duration::from_secs(5))  // Connection establishment timeout

    // Performance
    .http2_prior_knowledge()  // Use HTTP/2 if available
    .tcp_nodelay(true)  // Disable Nagle's algorithm for lower latency

    .build()?;
```

### Tuning Parameters

**Considerations**:
- **Pool size**: Based on concurrency (start with 10, tune up if needed)
- **Idle timeout**: Balance between connection reuse and resource usage
- **Connect timeout**: Fast fail for unreachable hosts
- **HTTP/2**: Cloudflare supports HTTP/2, reduces overhead

### Monitoring

**Metrics to Track**:
- Connection reuse rate (should be >80%)
- Connection establishment time
- Pool saturation (should never hit max)
- Idle connection evictions

**Estimated Time**: 2-3 hours

---

## Phase 6: Incremental Update Optimization 📋 FUTURE

**Status**: 📋 Future work
**Priority**: HIGH (constitutional requirement)

**Objective**: Ensure incremental updates only re-analyze affected components

### Current State
- ✅ Content-addressed fingerprinting works (blake3)
- ⚠️ Triggering logic for affected component detection unclear
- ❌ No validation that incremental updates work as expected

### Investigation Needed

**Questions**:
1. How are file changes detected and fingerprinted?
2. How does Recoco determine which components to re-analyze?
3. Is there a dependency graph tracking component relationships?
4. What happens when a shared module is updated?

**Files to Review**:
- Recoco dataflow framework integration
- Fingerprint cache implementation
- Change detection logic

### Success Criteria
- [ ] File change → Only affected components re-analyzed
- [ ] Shared module change → Dependent components re-analyzed
- [ ] No change → Zero re-analysis (100% cache hit)
- [ ] Performance: <1% of full analysis time for typical updates

**Estimated Time**: 16-20 hours (requires deep Recoco understanding)

---

## Timeline Estimate

| Phase | Tasks | Estimated Time | Dependencies |
|-------|-------|----------------|--------------|
| Phase 1 | #55 | ✅ Complete | None |
| Phase 2 | #58, #60 | 6-9 hours | Phase 1 |
| Phase 3 | #57 | 8-10 hours | Phase 2 (validation) |
| Phase 4 | #56 | 4-6 hours | Phase 2 (query patterns) |
| Phase 5 | #59 | 2-3 hours | None (parallel) |
| Phase 6 | TBD | 16-20 hours | Phases 2-5 |

**Total**: 36-48 hours (5-6 working days)

**Critical Path**: Phase 1 → Phase 2 → Phase 3 → Constitutional compliance achieved

---

## Priority Ranking

### CRITICAL (Blocking constitutional compliance)
1. ✅ **Phase 1**: Performance instrumentation (DONE)
2. 🔄 **Phase 2**: Benchmarking and measurement (IN PROGRESS)
3. **Phase 3**: Query result caching (>90% hit rate requirement)

### HIGH (Performance optimization)
4. **Phase 4**: Schema and index optimization
5. **Phase 6**: Incremental update validation

### MEDIUM (Nice to have)
6. **Phase 5**: Connection pooling optimization

---

## Success Criteria

### Minimum Viable Compliance
- ✅ All queries instrumented with performance tracking
- [ ] D1 p95 latency <50ms (measured and validated)
- [ ] Cache hit rate >90% (measured and validated)
- [ ] Compliance report generated with evidence

### Production Ready
- [ ] All constitutional requirements met
- [ ] Performance baselines established
- [ ] Monitoring dashboards deployed
- [ ] Performance regression tests integrated
- [ ] Documentation complete

### Excellence
- [ ] p95 latency <25ms (2x better than requirement)
- [ ] Cache hit rate >95%
- [ ] Zero performance regressions in CI/CD
- [ ] Automated alerts for SLO violations

---

## Risk Mitigation

### Risk 1: D1 Latency Exceeds 50ms

**Likelihood**: HIGH (example assumes 75ms average)

**Mitigation**:
- Implement query result caching (99.9% latency reduction on hits)
- Optimize query patterns and indexes
- Consider regional query routing for edge deployment
- Batch operations where possible

**Contingency**:
- Request constitutional requirement adjustment (backed by data)
- Implement application-level query optimization
- Consider alternative storage backends for critical paths

### Risk 2: Cache Hit Rate Below 90%

**Likelihood**: MEDIUM

**Mitigation**:
- Content-addressed keys ensure deduplication
- Preload cache with common queries
- Increase cache capacity and TTL
- Analyze cache miss patterns

**Contingency**:
- Implement multi-tier caching (L1 in-memory, L2 distributed)
- Add cache warming strategies
- Optimize cache key design

### Risk 3: Incremental Updates Not Working

**Likelihood**: LOW-MEDIUM

**Mitigation**:
- Deep dive into Recoco dataflow framework
- Add comprehensive integration tests
- Implement dependency graph tracking
- Validate fingerprint-based change detection

**Contingency**:
- Manual dependency tracking
- Conservative re-analysis (re-analyze more than strictly necessary)
- Document known limitations

---

## Next Actions

### Immediate (This Week)
1. **Start Task #58**: Create D1 query profiling benchmarks
2. **Measure baseline**: Get actual p50/p95/p99 latencies
3. **Document findings**: Update compliance status

### Short Term (Next Week)
4. **Complete Task #57**: Implement query result caching
5. **Measure cache hit rate**: Validate >90% requirement
6. **Generate compliance report**: Task #60

### Medium Term (Following Week)
7. **Schema optimization**: Task #56
8. **Connection pooling**: Task #59
9. **Full compliance validation**: All requirements met

---

**Last Updated**: 2026-01-28
**Owner**: Database Optimization Team
**Next Review**: After Phase 2 completion
