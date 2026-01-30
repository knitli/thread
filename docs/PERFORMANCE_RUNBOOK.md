<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Thread Performance Runbook

**Purpose**: Operational procedures for managing Thread performance in production
**Audience**: DevOps, SRE, Operations teams
**Last Updated**: 2026-01-28

---

## Quick Reference

### Emergency Response

| Symptom | Probable Cause | Quick Fix | Runbook Section |
|---------|----------------|-----------|-----------------|
| Cache hit rate <90% | Cache misconfiguration or evictions | Increase cache capacity | [Cache Issues](#cache-performance-issues) |
| Query latency >50ms p95 | Database overload or missing indexes | Check indexes, connection pool | [Database Issues](#database-performance-issues) |
| High CPU usage | Missing cache hits or regression | Check cache metrics, rollback | [CPU Issues](#cpu-performance-issues) |
| Memory leak | Cache not evicting or query accumulation | Restart service, check TTL | [Memory Issues](#memory-performance-issues) |
| Low throughput | Sequential processing or small batches | Enable parallel feature, tune batch size | [Throughput Issues](#throughput-issues) |

### SLO Targets

| Metric | Target | Alert Threshold | Critical Threshold |
|--------|--------|-----------------|-------------------|
| Cache hit rate | >90% | <85% for 5min | <80% for 2min |
| Fingerprint time | <1µs | >1µs for 1min | >2µs for 30sec |
| Postgres p95 latency | <10ms | >10ms for 2min | >20ms for 1min |
| D1 p95 latency | <50ms | >50ms for 2min | >100ms for 1min |
| Query error rate | <0.1% | >1% for 2min | >5% for 1min |
| Throughput | >5 MiB/s | <4 MiB/s for 5min | <2 MiB/s for 2min |

---

## Table of Contents

1. [Monitoring & Alerts](#monitoring--alerts)
2. [Performance Troubleshooting](#performance-troubleshooting)
3. [Configuration Management](#configuration-management)
4. [Capacity Planning](#capacity-planning)
5. [Incident Response](#incident-response)
6. [Maintenance Procedures](#maintenance-procedures)

---

## Monitoring & Alerts

### Dashboard Access

**Grafana Dashboard**: `thread-performance-monitoring`
- URL: `https://grafana.example.com/d/thread-performance`
- Panels: Constitutional compliance, performance metrics, throughput, cache ops, errors
- Refresh: 30 seconds

**Metrics Source**: Prometheus
- URL: `https://prometheus.example.com`
- Scrape interval: 15 seconds
- Retention: 30 days

### Key Metrics

#### Constitutional Compliance Metrics

```promql
# Cache hit rate (Constitutional: >90%)
thread_cache_hit_rate_percent

# Query latency p95 (Constitutional: Postgres <10ms, D1 <50ms)
thread_query_avg_duration_seconds * 1000

# Alert if cache hit rate <85% for 5 minutes
thread_cache_hit_rate_percent < 85
```

#### Performance Metrics

```promql
# Fingerprint computation time
thread_fingerprint_avg_duration_seconds * 1000000  # Convert to µs

# File processing rate
rate(thread_files_processed_total[5m])

# Data throughput
rate(thread_bytes_processed_total[5m]) / 1024 / 1024  # MB/sec

# Batch processing rate
rate(thread_batches_processed_total[5m])
```

#### Cache Metrics

```promql
# Cache hit rate over time
rate(thread_cache_hits_total[5m]) / (rate(thread_cache_hits_total[5m]) + rate(thread_cache_misses_total[5m]))

# Cache eviction rate
rate(thread_cache_evictions_total[5m])
```

#### Error Metrics

```promql
# Query error rate
thread_query_error_rate_percent

# Total errors per second
rate(thread_query_errors_total[5m])
```

### Alert Configuration

#### Critical Alerts (PagerDuty)

**Cache Hit Rate Critical**:
```yaml
alert: ThreadCacheHitRateCritical
expr: thread_cache_hit_rate_percent < 80
for: 2m
labels:
  severity: critical
  component: caching
annotations:
  summary: "Thread cache hit rate critically low"
  description: "Cache hit rate is {{ $value }}% (threshold: 80%)"
  runbook: "https://docs.example.com/runbooks/thread-performance#cache-performance-issues"
```

**Query Latency Critical**:
```yaml
alert: ThreadQueryLatencyCritical
expr: thread_query_avg_duration_seconds * 1000 > 100
for: 1m
labels:
  severity: critical
  component: database
annotations:
  summary: "Thread query latency critically high"
  description: "Query p95 latency is {{ $value }}ms (threshold: 100ms)"
  runbook: "https://docs.example.com/runbooks/thread-performance#database-performance-issues"
```

**Error Rate Critical**:
```yaml
alert: ThreadErrorRateCritical
expr: thread_query_error_rate_percent > 5
for: 1m
labels:
  severity: critical
  component: queries
annotations:
  summary: "Thread error rate critically high"
  description: "Error rate is {{ $value }}% (threshold: 5%)"
  runbook: "https://docs.example.com/runbooks/thread-performance#error-handling"
```

#### Warning Alerts (Slack)

**Cache Hit Rate Warning**:
```yaml
alert: ThreadCacheHitRateWarning
expr: thread_cache_hit_rate_percent < 85
for: 5m
labels:
  severity: warning
  component: caching
annotations:
  summary: "Thread cache hit rate low"
  description: "Cache hit rate is {{ $value }}% (threshold: 85%)"
```

**Query Latency Warning**:
```yaml
alert: ThreadQueryLatencyWarning
expr: (thread_query_avg_duration_seconds * 1000 > 50) and (thread_query_avg_duration_seconds * 1000 < 100)
for: 2m
labels:
  severity: warning
  component: database
annotations:
  summary: "Thread query latency elevated"
  description: "Query p95 latency is {{ $value }}ms (threshold: 50ms)"
```

**Throughput Warning**:
```yaml
alert: ThreadThroughputWarning
expr: rate(thread_bytes_processed_total[5m]) / 1024 / 1024 < 4
for: 5m
labels:
  severity: warning
  component: processing
annotations:
  summary: "Thread throughput low"
  description: "Throughput is {{ $value }} MB/s (threshold: 4 MB/s)"
```

---

## Performance Troubleshooting

### Cache Performance Issues

#### Symptom: Cache Hit Rate <90%

**Diagnosis Steps**:

1. **Check cache metrics**:
```bash
# Prometheus query
thread_cache_hit_rate_percent

# Expected: >90%
# If <90%: Investigate cache configuration
```

2. **Check cache capacity**:
```bash
# Environment variable
echo $THREAD_CACHE_MAX_CAPACITY

# Recommended: 100,000 for typical workloads
# If lower: Increase capacity
```

3. **Check cache evictions**:
```promql
rate(thread_cache_evictions_total[5m])

# High eviction rate indicates insufficient capacity
```

4. **Check TTL configuration**:
```bash
echo $THREAD_CACHE_TTL_SECONDS

# Recommended:
# - Rapid iteration: 300-900 (5-15 min)
# - Stable codebase: 3600-21600 (1-6 hours)
```

**Resolution**:

**Option 1: Increase Cache Capacity**
```bash
# Update environment variable
export THREAD_CACHE_MAX_CAPACITY=200000

# Restart service
systemctl restart thread-service
```

**Option 2: Increase TTL**
```bash
# Update environment variable
export THREAD_CACHE_TTL_SECONDS=7200  # 2 hours

# Restart service
systemctl restart thread-service
```

**Option 3: Pre-warm Cache**
```bash
# Pre-populate cache with common files
thread analyze --preload standard-library/
thread analyze --preload common-dependencies/
```

**Validation**:
```bash
# Monitor cache hit rate for 10 minutes
watch -n 10 'curl -s http://localhost:9090/api/v1/query?query=thread_cache_hit_rate_percent | jq ".data.result[0].value[1]"'

# Expected: Gradual increase to >90%
```

---

### Database Performance Issues

#### Symptom: Query Latency >50ms p95

**Diagnosis Steps**:

1. **Check database type and latency**:
```bash
# Postgres (CLI)
psql -U thread_user -d thread_cache -c "
SELECT
  query,
  mean_exec_time,
  calls
FROM pg_stat_statements
WHERE mean_exec_time > 50
ORDER BY mean_exec_time DESC
LIMIT 10;"

# Expected: <10ms for Postgres
# If >10ms: Investigate slow queries
```

```javascript
// D1 (Edge)
// Check Cloudflare Workers analytics dashboard
// Expected: <50ms for D1
// If >50ms: Investigate query optimization
```

2. **Check for missing indexes**:
```sql
-- Postgres: Verify indexes exist
SELECT indexname, tablename
FROM pg_indexes
WHERE tablename = 'code_symbols';

-- Expected indexes:
-- - idx_symbols_hash (content_hash)
-- - idx_symbols_path (file_path)
-- - idx_symbols_created (created_at)
```

3. **Check connection pool**:
```bash
# Environment variable
echo $DB_POOL_SIZE

# Recommended: 10-20 for CLI
# If lower or unset: Configure pool
```

4. **Check query patterns**:
```bash
# Look for N+1 query patterns in logs
grep "SELECT.*FROM code_symbols" /var/log/thread/queries.log | wc -l

# If excessive: Implement batching
```

**Resolution**:

**Option 1: Create Missing Indexes**
```sql
-- Postgres
CREATE INDEX CONCURRENTLY idx_symbols_hash ON code_symbols(content_hash);
CREATE INDEX CONCURRENTLY idx_symbols_path ON code_symbols(file_path);
CREATE INDEX CONCURRENTLY idx_symbols_created ON code_symbols(created_at);

-- Analyze table for query planner
ANALYZE code_symbols;
```

```sql
-- D1 (via wrangler)
CREATE INDEX idx_symbols_hash ON code_symbols(content_hash);
CREATE INDEX idx_symbols_path ON code_symbols(file_path);
```

**Option 2: Increase Connection Pool**
```bash
# Update environment variable
export DB_POOL_SIZE=20
export DB_CONNECTION_TIMEOUT=60

# Restart service
systemctl restart thread-service
```

**Option 3: Enable Query Batching**
```javascript
// D1: Batch queries with IN clause
const placeholders = hashes.map(() => '?').join(',');
const results = await env.DB.prepare(
  `SELECT * FROM code_symbols WHERE content_hash IN (${placeholders})`
).bind(...hashes).all();
```

**Option 4: Optimize Slow Queries**
```sql
-- Use prepared statements (automatic with ReCoco)
PREPARE get_symbols AS
  SELECT symbols FROM code_symbols WHERE content_hash = $1;

-- Execute repeatedly (10-20% faster)
EXECUTE get_symbols('abc123...');
```

**Validation**:
```bash
# Monitor query latency
watch -n 10 'curl -s http://localhost:9090/api/v1/query?query=thread_query_avg_duration_seconds | jq ".data.result[0].value[1]"'

# Expected: Gradual decrease to <0.05 (50ms) for D1, <0.01 (10ms) for Postgres
```

---

### CPU Performance Issues

#### Symptom: High CPU Usage

**Diagnosis Steps**:

1. **Check cache hit rate**:
```promql
thread_cache_hit_rate_percent

# Low hit rate causes excessive parsing (CPU-heavy)
```

2. **Check for performance regression**:
```bash
# Run benchmarks
cargo bench -p thread-flow --bench load_test

# Compare to baseline
cargo benchcmp baseline.txt current.txt

# If >10% regression: Investigate recent changes
```

3. **Profile CPU usage**:
```bash
# Generate flamegraph
./scripts/profile.sh flamegraph pattern_matching

# Look for unexpected hot paths
# Expected hot paths:
# - Pattern matching (~45%)
# - Tree-sitter parsing (~30%)
# - Meta-var processing (~15%)
```

4. **Check parallel processing**:
```bash
# Verify parallel feature is enabled (CLI only)
cargo build --release --features parallel

# Check thread count
echo $RAYON_NUM_THREADS

# Recommended: physical_cores (CPU-bound) or physical_cores * 1.5 (mixed)
```

**Resolution**:

**Option 1: Increase Cache Hit Rate**
(See [Cache Performance Issues](#cache-performance-issues))

**Option 2: Rollback Recent Changes**
```bash
# If regression detected
git log --oneline -10

# Rollback to last known good commit
git revert <commit-hash>

# Rebuild and restart
cargo build --release
systemctl restart thread-service
```

**Option 3: Optimize Thread Count**
```bash
# Set optimal thread count
export RAYON_NUM_THREADS=$(nproc)  # For CPU-bound

# Or for mixed workload
export RAYON_NUM_THREADS=$(($(nproc) * 3 / 2))

# Restart service
systemctl restart thread-service
```

**Option 4: Enable Lazy Parsing**
(If not already enabled in code)
```rust
// Skip parsing when file type doesn't match rules
if applicable_rules.is_empty() {
    return Ok(Vec::new());  // Skip parsing entirely
}
```

**Validation**:
```bash
# Monitor CPU usage
top -p $(pgrep thread-service)

# Expected: CPU usage proportional to workload
# If still high: Escalate to performance engineering team
```

---

### Memory Performance Issues

#### Symptom: Memory Leak or High Memory Usage

**Diagnosis Steps**:

1. **Check cache size**:
```bash
# Estimate cache memory usage
# Approximate: 1 KB per cached file

# Expected memory for 100k cache:
# 100,000 files * 1 KB = ~100 MB

# If much higher: Investigate leak
```

2. **Check for cache evictions**:
```promql
rate(thread_cache_evictions_total[5m])

# Low eviction rate with high memory suggests leak
```

3. **Profile memory allocation**:
```bash
# Memory profiling with valgrind
./scripts/profile.sh memory integration_tests

# Look for:
# - Memory leaks (unfreed allocations)
# - Excessive allocations (string cloning)
```

4. **Check query accumulation**:
```bash
# Look for unbounded query result accumulation
grep "query results" /var/log/thread/debug.log | wc -l

# If excessive: Check query cache TTL
```

**Resolution**:

**Option 1: Reduce Cache Capacity**
```bash
# Reduce cache size if memory-constrained
export THREAD_CACHE_MAX_CAPACITY=50000

# Restart service
systemctl restart thread-service
```

**Option 2: Enable Cache Eviction**
```bash
# Reduce TTL to force evictions
export THREAD_CACHE_TTL_SECONDS=1800  # 30 minutes

# Restart service
systemctl restart thread-service
```

**Option 3: Restart Service (Temporary Fix)**
```bash
# Emergency memory release
systemctl restart thread-service

# Monitor memory post-restart
watch -n 10 'ps aux | grep thread-service | awk "{print \$6}"'
```

**Option 4: Profile and Fix Leak** (If leak confirmed)
```bash
# Run heap profiler
./scripts/profile.sh heap integration_tests

# Analyze allocation patterns
# Report to development team for fix
```

**Validation**:
```bash
# Monitor memory usage over time
watch -n 60 'ps aux | grep thread-service | awk "{print \$6 / 1024} MB"'

# Expected: Stable memory usage over time
# If growing: Leak confirmed, escalate
```

---

### Throughput Issues

#### Symptom: Low Throughput (<5 MiB/s)

**Diagnosis Steps**:

1. **Check parallel processing**:
```bash
# Verify parallel feature enabled
cargo build --release --features parallel

# Check if actually parallel
ps aux | grep thread-service | grep rayon

# If missing: Not using parallel processing
```

2. **Check batch size**:
```bash
echo $THREAD_BATCH_SIZE

# Recommended:
# - Small files (<10KB): 500-1000
# - Medium files (10-100KB): 100-200
# - Large files (>100KB): 10-50
```

3. **Check cache hit rate**:
```promql
thread_cache_hit_rate_percent

# Low hit rate causes re-parsing (slow)
```

4. **Check for I/O bottleneck**:
```bash
# Monitor disk I/O
iostat -x 1 10

# Look for high %util on disk
# If >80%: I/O bottleneck
```

**Resolution**:

**Option 1: Enable Parallel Processing**
```bash
# Build with parallel feature
cargo build --release --features parallel

# Set thread count
export RAYON_NUM_THREADS=$(nproc)

# Restart service
systemctl restart thread-service
```

**Option 2: Optimize Batch Size**
```bash
# Test different batch sizes
for batch_size in 50 100 200 500; do
  export THREAD_BATCH_SIZE=$batch_size
  time thread analyze large-codebase/
done

# Use optimal batch size
export THREAD_BATCH_SIZE=<optimal>

# Update configuration
echo "THREAD_BATCH_SIZE=<optimal>" >> /etc/thread/config.env

# Restart service
systemctl restart thread-service
```

**Option 3: Increase Cache Hit Rate**
(See [Cache Performance Issues](#cache-performance-issues))

**Option 4: Address I/O Bottleneck**
```bash
# Use faster storage (SSD)
# Or: Add read cache
# Or: Batch file operations
```

**Validation**:
```bash
# Monitor throughput
watch -n 10 'curl -s http://localhost:9090/api/v1/query?query=rate(thread_bytes_processed_total[5m]) | jq ".data.result[0].value[1] | tonumber / 1024 / 1024"'

# Expected: >5 MB/s (cold), >100 MB/s (warm cache)
```

---

## Configuration Management

### Environment Variables

**Caching Configuration**:
```bash
# Cache capacity (number of entries)
THREAD_CACHE_MAX_CAPACITY=100000  # Default: 10,000

# Cache TTL (seconds)
THREAD_CACHE_TTL_SECONDS=3600  # Default: 300 (5 min)

# Feature flags
THREAD_FEATURES="parallel,caching"  # CLI deployment
THREAD_FEATURES="caching"  # Edge deployment (no parallel)
```

**Database Configuration**:
```bash
# Postgres (CLI)
DATABASE_URL=postgresql://user:pass@localhost/thread_cache
DB_POOL_SIZE=20  # Default: 10
DB_CONNECTION_TIMEOUT=60  # Seconds

# D1 (Edge) - configured in wrangler.toml
# No environment variables needed
```

**Processing Configuration**:
```bash
# Parallel processing (CLI only)
RAYON_NUM_THREADS=4  # Default: auto-detect cores

# Batch size
THREAD_BATCH_SIZE=100  # Default: 100

# Logging
RUST_LOG=thread_flow=info  # Levels: error, warn, info, debug, trace
```

### Configuration Files

**CLI Configuration** (`/etc/thread/config.env`):
```bash
# Caching
THREAD_CACHE_MAX_CAPACITY=200000
THREAD_CACHE_TTL_SECONDS=7200

# Database
DATABASE_URL=postgresql://thread:password@db.example.com:5432/thread_cache
DB_POOL_SIZE=20
DB_CONNECTION_TIMEOUT=60

# Processing
RAYON_NUM_THREADS=8
THREAD_BATCH_SIZE=200

# Logging
RUST_LOG=thread_flow=info,thread_services=info

# Features
THREAD_FEATURES=parallel,caching
```

**Edge Configuration** (`wrangler.toml`):
```toml
name = "thread-worker"
main = "src/index.js"
compatibility_date = "2024-01-01"

[vars]
THREAD_CACHE_MAX_CAPACITY = 50000
THREAD_CACHE_TTL_SECONDS = 3600
RUST_LOG = "thread_flow=info"
THREAD_FEATURES = "caching"

[[d1_databases]]
binding = "DB"
database_name = "thread-cache"
database_id = "your-d1-database-id"
```

### Configuration Validation

**Validate CLI Configuration**:
```bash
# Source configuration
source /etc/thread/config.env

# Validate environment variables
echo "Cache capacity: $THREAD_CACHE_MAX_CAPACITY"
echo "Cache TTL: $THREAD_CACHE_TTL_SECONDS"
echo "DB pool size: $DB_POOL_SIZE"
echo "Thread count: $RAYON_NUM_THREADS"
echo "Batch size: $THREAD_BATCH_SIZE"
echo "Features: $THREAD_FEATURES"

# Test database connection
psql $DATABASE_URL -c "SELECT 1;"

# Expected: Connection successful
```

**Validate Edge Configuration**:
```bash
# Validate wrangler.toml
wrangler validate

# Test D1 connection
wrangler d1 execute thread-cache --command "SELECT 1;"

# Deploy to preview
wrangler deploy --env preview

# Test preview deployment
curl https://thread-worker-preview.example.workers.dev/health

# Expected: 200 OK
```

---

## Capacity Planning

### Resource Requirements

**CLI Deployment** (per instance):

| Project Size | CPU Cores | Memory | Storage | Throughput |
|--------------|-----------|--------|---------|------------|
| Small (<100 files) | 2 | 2 GB | 1 GB | 50 files/sec |
| Medium (100-1000 files) | 4 | 4 GB | 5 GB | 200 files/sec |
| Large (1000-10000 files) | 8 | 8 GB | 20 GB | 500 files/sec |
| X-Large (>10000 files) | 16 | 16 GB | 50 GB | 1000 files/sec |

**Edge Deployment** (per Worker):

| Metric | Limit | Notes |
|--------|-------|-------|
| CPU Time | 50ms | Per request |
| Memory | 128 MB | Total |
| Bundle Size | 2.1 MB | Optimized WASM |
| Requests/sec | 100-200 | With 90% cache hit |
| Cold Start | <100ms | WASM initialization |

### Scaling Guidelines

**Horizontal Scaling** (CLI):
```bash
# Add instances behind load balancer
# Each instance processes independently

# Example: 3 instances, 8 cores each
# Capacity: 500 files/sec * 3 = 1500 files/sec

# Database: Increase connection pool
DB_POOL_SIZE=$((instances * cores * 2))
```

**Vertical Scaling** (CLI):
```bash
# Add cores for parallel processing
# Expected speedup: ~0.9 * cores (90% efficiency)

# Example: 4 → 8 cores
# Speedup: ~7.2x (from load test results)
```

**Edge Scaling** (Workers):
```bash
# Automatic horizontal scaling by Cloudflare
# No configuration needed

# Capacity planning:
# - Cache hit rate >90%: 100-200 req/sec per region
# - Cache hit rate <90%: 40-80 req/sec per region

# Global capacity: regions * req/sec
```

### Capacity Monitoring

**Dashboard**: `capacity-monitoring` (Grafana)

**Key Metrics**:
```promql
# Current throughput vs capacity
rate(thread_files_processed_total[5m]) / <instance_capacity>

# CPU utilization
100 - (avg by(instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)

# Memory utilization
(node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes) / node_memory_MemTotal_bytes * 100

# Storage utilization
(node_filesystem_size_bytes - node_filesystem_avail_bytes) / node_filesystem_size_bytes * 100
```

**Scaling Triggers**:
- CPU >80% for >10 min → Add instances or cores
- Memory >85% for >5 min → Add memory or instances
- Throughput >80% capacity for >10 min → Add instances
- Storage >90% → Add storage or increase cache eviction

---

## Incident Response

### Performance Degradation Incident

**Severity**: P2 (High)
**Response Time**: 15 minutes
**Resolution Target**: 2 hours

**Incident Response Procedure**:

1. **Acknowledge Incident**
```bash
# PagerDuty: Acknowledge alert
# Slack: Post incident in #incidents channel
# Subject: "Thread performance degradation - Cache hit rate <85%"
```

2. **Initial Assessment**
```bash
# Check Grafana dashboard
https://grafana.example.com/d/thread-performance

# Gather metrics
curl -s http://prometheus:9090/api/v1/query?query=thread_cache_hit_rate_percent
curl -s http://prometheus:9090/api/v1/query?query=thread_query_avg_duration_seconds

# Check logs
tail -n 100 /var/log/thread/error.log
```

3. **Quick Fixes**
```bash
# Option 1: Increase cache capacity
export THREAD_CACHE_MAX_CAPACITY=200000
systemctl restart thread-service

# Option 2: Clear cache and restart
rm -rf /var/cache/thread/*
systemctl restart thread-service

# Option 3: Rollback recent deploy
git checkout <previous-commit>
./deploy.sh
```

4. **Validation**
```bash
# Monitor metrics for 10 minutes
watch -n 30 'curl -s http://prometheus:9090/api/v1/query?query=thread_cache_hit_rate_percent'

# Expected: Gradual return to >90%
```

5. **Root Cause Analysis**
```bash
# Generate RCA report
./scripts/incident-report.sh

# Include:
# - Timeline of incident
# - Metrics snapshot
# - Actions taken
# - Root cause (if identified)
# - Prevention measures
```

6. **Post-Incident Review**
```bash
# Schedule PIR meeting
# Invite: On-call engineer, SRE lead, performance engineering

# Document:
# - What went wrong
# - What went right
# - Action items for prevention
```

---

## Maintenance Procedures

### Regular Maintenance

**Daily**:
```bash
# Monitor dashboard
# - Check Constitutional compliance metrics
# - Verify no active alerts

# Review error logs
tail -n 100 /var/log/thread/error.log | grep -E "ERROR|WARN"
```

**Weekly**:
```bash
# Review performance trends
# - Cache hit rate trend
# - Query latency trend
# - Throughput trend

# Check for performance regressions
cargo bench > weekly-benchmark.txt
cargo benchcmp baseline.txt weekly-benchmark.txt
```

**Monthly**:
```bash
# Vacuum database (Postgres)
psql $DATABASE_URL -c "VACUUM ANALYZE code_symbols;"

# Clean old cache entries (D1)
wrangler d1 execute thread-cache --command "
DELETE FROM code_symbols
WHERE updated_at < strftime('%s', 'now', '-30 days');"

# Review capacity planning
# - Check resource utilization trends
# - Plan for scaling if needed
```

**Quarterly**:
```bash
# Full performance audit
./scripts/comprehensive-profile.sh

# Review optimization roadmap
# - Evaluate completed optimizations
# - Prioritize next optimizations

# Update baselines
cargo bench > quarterly-baseline.txt
cp quarterly-baseline.txt baseline.txt
```

### Database Maintenance

**Postgres Vacuum** (Weekly):
```sql
-- Regular vacuum
VACUUM ANALYZE code_symbols;

-- Full vacuum (monthly, during maintenance window)
VACUUM FULL code_symbols;
```

**Index Maintenance** (Monthly):
```sql
-- Rebuild indexes
REINDEX TABLE code_symbols;

-- Update statistics
ANALYZE code_symbols;
```

**Cache Cleanup** (Monthly):
```sql
-- Remove stale entries (>30 days old)
DELETE FROM code_symbols
WHERE updated_at < NOW() - INTERVAL '30 days';
```

**D1 Maintenance** (Monthly):
```sql
-- Clean old entries
DELETE FROM code_symbols
WHERE updated_at < strftime('%s', 'now', '-30 days');

-- Optimize database
VACUUM;
```

### Cache Maintenance

**Cache Warming** (After deployment):
```bash
# Pre-populate cache with common files
thread analyze --preload standard-library/
thread analyze --preload common-dependencies/

# Verify cache population
curl -s http://prometheus:9090/api/v1/query?query=thread_cache_entries_total

# Expected: Gradual increase to 10k-100k
```

**Cache Invalidation** (When needed):
```bash
# Clear all cache entries
rm -rf /var/cache/thread/*

# Or: Clear specific entries via database
psql $DATABASE_URL -c "DELETE FROM code_symbols WHERE file_path LIKE '%old-library%';"

# Restart service
systemctl restart thread-service
```

---

## Appendix

### Useful Commands

**Performance Profiling**:
```bash
# Quick flamegraph
./scripts/profile.sh quick

# Comprehensive profiling
./scripts/profile.sh comprehensive

# Memory profiling
./scripts/profile.sh memory integration_tests

# Heap profiling
./scripts/profile.sh heap pattern_matching
```

**Load Testing**:
```bash
# Run all load tests
cargo bench -p thread-flow --bench load_test --all-features

# Run specific category
cargo bench -p thread-flow --bench load_test -- large_codebase

# Run with profiling
cargo flamegraph --bench load_test --all-features
```

**Benchmarking**:
```bash
# Run benchmarks
cargo bench -p thread-flow

# Save baseline
cargo bench > baseline.txt

# Compare
cargo bench > current.txt
cargo benchcmp baseline.txt current.txt
```

**Metrics Export**:
```bash
# Export Prometheus metrics
curl http://localhost:9090/metrics

# Query specific metric
curl -s 'http://prometheus:9090/api/v1/query?query=thread_cache_hit_rate_percent' | jq '.data.result[0].value[1]'
```

### Contact Information

**Escalation Path**:
1. On-call SRE: sre-oncall@example.com (PagerDuty)
2. Performance Engineering: perf-eng@example.com
3. Development Team: dev-team@example.com

**Resources**:
- Grafana: https://grafana.example.com
- Prometheus: https://prometheus.example.com
- Runbooks: https://docs.example.com/runbooks/
- Performance docs: https://docs.example.com/performance/

---

**Document Version**: 1.0
**Last Updated**: 2026-01-28
**Maintained By**: DevOps/SRE Team
**Review Frequency**: Monthly
