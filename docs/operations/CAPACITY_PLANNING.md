# Capacity Planning Guide

**Version**: 1.0.0
**Last Updated**: 2026-01-28
**Status**: Production Ready

---

## Overview

This guide provides comprehensive capacity planning guidance for Thread deployments across CLI and Edge environments. It covers resource requirements, scaling thresholds, cost optimization, and capacity monitoring strategies.

### Purpose

- **Right-sizing**: Determine appropriate resources for workload requirements
- **Cost Optimization**: Balance performance needs with infrastructure costs
- **Scalability Planning**: Plan for growth and traffic spikes
- **Performance Assurance**: Maintain SLO compliance under varying loads

### Integration Points

- **Day 15 Performance Foundation**: Blake3 fingerprinting, content-addressed caching
- **Day 20 Monitoring**: Prometheus metrics, Grafana dashboards, SLO tracking
- **Day 23 Optimization**: Load testing framework, performance benchmarks

---

## Resource Requirements by Project Size

### Small Projects (< 100 files, < 10 MB codebase)

#### CLI Deployment

**Compute**:
- CPU: 2 cores minimum
- Memory: 512 MB - 1 GB
- Storage: 5 GB (including cache)

**Performance Characteristics**:
- Fingerprint time: ~4.5 µs (100 files × 45 ns)
- Full analysis: < 100 ms
- Cache hit rate: 85-90% (after warmup)
- Throughput: 430-672 MiB/s

**Cost Model** (AWS EC2 t3.small equivalent):
- Instance: $0.0208/hour (~$15/month)
- Storage (EBS gp3): $0.08/GB/month (~$0.40/month)
- **Total**: ~$15.40/month

#### Edge Deployment

**Cloudflare Workers Limits**:
- CPU time: < 10 ms per request
- Memory: 128 MB
- Request size: < 10 MB
- Concurrent requests: Unlimited (auto-scaled)

**Performance Characteristics**:
- Cold start: 10-20 ms (WASM initialization)
- Warm request: < 5 ms
- Geographic latency: < 50 ms p95 (CDN edge)

**Cost Model** (Cloudflare Workers):
- Free tier: 100,000 requests/day
- Paid: $5/month + $0.50/million requests
- **Small project**: Free tier sufficient

### Medium Projects (100-1,000 files, 10-100 MB codebase)

#### CLI Deployment

**Compute**:
- CPU: 4-8 cores (for parallel processing)
- Memory: 2-4 GB
- Storage: 20 GB (including cache and historical data)

**Performance Characteristics**:
- Fingerprint time: ~425 µs (1,000 files × 425 ns)
- Full analysis: 500 ms - 2 seconds
- Cache hit rate: 90-95% (steady state)
- Throughput: 430-672 MiB/s (parallel)
- Parallel speedup: 2-4x with Rayon

**Database Requirements**:
- **Postgres (Local)**:
  - Storage: 1-5 GB (cache + metadata)
  - Connections: 10-50 concurrent
  - Query latency: < 10 ms p95

**Cost Model** (AWS EC2 t3.medium + RDS):
- Compute: $0.0416/hour (~$30/month)
- Storage (EBS gp3 20GB): $1.60/month
- RDS Postgres (db.t3.micro): $15/month
- **Total**: ~$46.60/month

#### Edge Deployment

**Cloudflare Workers + D1**:
- CPU time: 10-50 ms per request (with D1 queries)
- Memory: 128 MB (WASM + query results)
- D1 storage: 1-5 GB
- Geographic replication: Automatic

**Performance Characteristics**:
- Request latency: < 50 ms p95 (including D1)
- D1 query latency: < 20 ms p95
- Cache hit rate: 95%+ (edge caching)

**Cost Model**:
- Workers: $5/month base
- D1: $5/month (5 GB included)
- Requests: $0.50/million over 10M/month
- **Medium project**: ~$10-15/month

### Large Projects (1,000-10,000 files, 100 MB - 1 GB codebase)

#### CLI Deployment

**Compute**:
- CPU: 8-16 cores (full Rayon parallelism)
- Memory: 8-16 GB
- Storage: 100 GB (extensive cache, history, vectors)

**Performance Characteristics**:
- Fingerprint time: ~4.25 ms (10,000 files × 425 ns)
- Full analysis: 5-15 seconds (parallel)
- Cache hit rate: 95-99% (mature workload)
- Throughput: 430-672 MiB/s sustained
- Parallel efficiency: 70-80% (8+ cores)

**Database Requirements**:
- **Postgres (Production)**:
  - Storage: 10-50 GB (cache + vectors + history)
  - Connections: 50-200 concurrent
  - Query latency: < 10 ms p95
  - Read replicas: 1-2 (for scale-out)

- **Qdrant (Vector Search - Optional)**:
  - Storage: 5-20 GB (vector embeddings)
  - Memory: 4-8 GB (in-memory indexes)
  - Query latency: < 100 ms p95

**Cost Model** (AWS EC2 c5.2xlarge + RDS + Qdrant):
- Compute: $0.34/hour (~$245/month)
- Storage (EBS gp3 100GB): $8/month
- RDS Postgres (db.m5.large): $140/month
- Qdrant (self-hosted on t3.large): $60/month
- **Total**: ~$453/month

#### Edge Deployment

**Cloudflare Workers + D1 + Durable Objects**:
- CPU time: 50-200 ms per complex request
- Memory: 128 MB (WASM runtime limit)
- D1 storage: 10-50 GB
- Durable Objects: For session state

**Performance Characteristics**:
- Request latency: < 100 ms p95 (complex analysis)
- D1 query latency: < 50 ms p95 (larger datasets)
- Cache hit rate: 99%+ (content-addressed caching)
- Geographic failover: Automatic

**Cost Model**:
- Workers: $5/month base
- D1: $5/month + $1/GB over 5GB (~$50/month for 50GB)
- Durable Objects: $5/month + $0.15/million requests
- Requests: $0.50/million over 10M/month
- **Large project**: ~$100-150/month

### Enterprise Projects (> 10,000 files, > 1 GB codebase)

#### CLI Deployment (Cluster)

**Multi-Node Architecture**:
- **Coordinator Node**: 4 cores, 8 GB memory
- **Worker Nodes**: 3-5 × (16 cores, 32 GB memory)
- **Database Cluster**: Postgres with replication + Qdrant cluster
- **Storage**: 500 GB - 1 TB (distributed cache)

**Performance Characteristics**:
- Fingerprint time: ~42.5 ms (100,000 files × 425 ns, batched)
- Full analysis: 30-120 seconds (distributed)
- Cache hit rate: 99%+ (mature enterprise workload)
- Throughput: 1-2 GiB/s (cluster aggregate)
- Horizontal scaling: Linear up to 10 nodes

**Database Requirements**:
- **Postgres Cluster**:
  - Primary + 2 replicas
  - Storage: 100-500 GB per node
  - Connections: 200-500 concurrent
  - Query latency: < 10 ms p95

- **Qdrant Cluster**:
  - 3 nodes (distributed)
  - Storage: 50-200 GB (vectors + metadata)
  - Memory: 16-32 GB per node
  - Query latency: < 100 ms p95

**Cost Model** (AWS EKS + RDS Multi-AZ):
- EKS control plane: $73/month
- Worker nodes (5 × c5.4xlarge): $1,224/month
- RDS Postgres Multi-AZ (db.r5.2xlarge): $840/month
- Qdrant cluster (3 × r5.xlarge): $540/month
- Storage (EBS gp3 1TB): $80/month
- Load balancer: $25/month
- **Total**: ~$2,782/month

#### Edge Deployment (Global CDN)

**Cloudflare Enterprise**:
- Workers: Unlimited CPU time (Enterprise plan)
- Memory: 128 MB per isolate
- D1: Multi-region replication
- Durable Objects: Global coordination

**Performance Characteristics**:
- Request latency: < 50 ms p95 (global edge)
- D1 query latency: < 50 ms p95 (regional reads)
- Cache hit rate: 99.5%+ (global cache)
- Geographic distribution: 200+ data centers

**Cost Model** (Cloudflare Enterprise):
- Enterprise plan: $200/month base
- D1 storage (500GB): $100/month
- Durable Objects: $50/month
- Bandwidth: Included (unlimited)
- **Enterprise project**: ~$350-500/month

---

## Scaling Thresholds and Decision Points

### When to Scale Up (Vertical Scaling)

**CPU Saturation Indicators**:
- Average CPU utilization > 70% sustained
- p95 request latency > 2× baseline
- Queue depth increasing
- Rayon thread pool exhaustion

**Action**: Increase CPU cores (2× current)

**Memory Pressure Indicators**:
- Memory utilization > 80%
- Swap usage increasing
- OOM events in logs
- Cache eviction rate > 20%

**Action**: Double memory allocation

**Storage Exhaustion Indicators**:
- Disk usage > 85%
- Cache eviction due to space
- Database write failures
- Slow query performance (I/O bound)

**Action**: Increase storage capacity 2×

### When to Scale Out (Horizontal Scaling)

**CLI Cluster Triggers**:
- Single-node CPU at capacity (>80%) for 1+ hour
- Request queue depth > 100 sustained
- Parallel efficiency < 50% (thread contention)
- Geographic distribution needed

**Action**: Add worker nodes (2-5× capacity)

**Edge Scaling** (Automatic):
- Cloudflare Workers auto-scale
- Monitor: Request latency and error rate
- Action: Optimize code, add D1 replicas if needed

### When to Scale Down

**Cost Optimization Triggers**:
- Average CPU < 20% for 7+ days
- Memory utilization < 40%
- Request volume decreased 50%+
- Cache hit rate > 99% (over-provisioned)

**Action**: Reduce instance size or node count

---

## Database Capacity Planning

### Postgres (Local CLI)

**Storage Growth Estimation**:
- **Cache entries**: ~1 KB per unique file fingerprint
- **Query results**: ~5 KB per cached query
- **Metadata**: ~100 bytes per file analyzed
- **Growth rate**: 10-50 MB/month (typical), 100-500 MB/month (heavy)

**Connection Pooling**:
- **Small projects**: 10-20 connections (single node)
- **Medium projects**: 50-100 connections (multi-threaded)
- **Large projects**: 100-200 connections (cluster)

**Maintenance**:
- **VACUUM**: Daily (automatic)
- **ANALYZE**: After bulk inserts
- **Reindex**: Monthly
- **Backup**: Daily incremental, weekly full

**Performance Tuning**:
```sql
-- Recommended settings for Thread workloads
shared_buffers = 256MB              -- 25% of system memory
effective_cache_size = 1GB          -- 50-75% of system memory
work_mem = 16MB                     -- For complex queries
maintenance_work_mem = 128MB        -- For VACUUM, CREATE INDEX
max_connections = 200               -- Based on workload
```

### D1 (Edge Deployment)

**Storage Limits**:
- Free tier: 5 GB per database
- Paid: 10 GB (soft limit, contact for more)
- **Planning**: Assume 5 GB per 1,000-5,000 files analyzed

**Query Limits**:
- 30-second query timeout (generous for edge)
- 1,000 rows per query result (pagination required)
- 100 MB response size limit

**Replication**:
- Multi-region replication (automatic)
- Read replicas in edge locations
- Write latency: < 100 ms (primary region)
- Read latency: < 20 ms (nearest edge)

**Cost Optimization**:
- Leverage content-addressed caching (99%+ hit rate)
- Minimize D1 writes (fingerprint changes only)
- Use edge caching for query results

### Qdrant (Vector Search)

**Memory Requirements**:
- **In-memory indexes**: 2-4× vector data size
- **1 million vectors (768D)**: ~3 GB in memory
- **Disk storage**: ~1 GB compressed

**Scaling**:
- **Vertical**: Increase memory for larger indexes
- **Horizontal**: Shard across nodes (3+ nodes)
- **Replication**: 2-3 replicas for HA

**Performance Tuning**:
```yaml
# Qdrant configuration for Thread workloads
storage:
  on_disk_payload: true           # Save memory
  hnsw_config:
    m: 16                         # Graph connectivity
    ef_construct: 100             # Build quality
    ef_search: 100                # Search quality

collection:
  replication_factor: 2           # HA
  shard_number: 3                 # Horizontal scaling
```

---

## Cost Optimization Strategies

### 1. Content-Addressed Caching (99.7% Cost Reduction)

**Strategy**: Fingerprint-based deduplication

**Impact**:
- Reduce redundant analysis by 99.7%
- Blake3 fingerprinting: 425 ns vs 147 µs parsing (346× faster)
- Cache hit rate: 90-99% (depending on workload maturity)

**Implementation**:
- Already implemented (Day 15)
- Monitor cache hit rate (target: >90%)
- Tune cache size based on working set

### 2. Parallel Processing Efficiency

**Strategy**: Use Rayon for CPU-bound workloads (CLI only)

**Impact**:
- 2-4× speedup on multi-core systems
- Reduce wall-clock time for large batches
- Better resource utilization

**Implementation**:
- Feature-gated (`parallel` feature)
- Optimal for 4+ cores
- Monitor parallel efficiency (target: >70%)

### 3. Edge Caching Layers

**Strategy**: Multi-tier caching (edge → D1 → origin)

**Impact**:
- 99%+ cache hit rate at edge (< 1 ms latency)
- Reduce D1 queries by 95%+
- Lower Cloudflare costs (fewer requests to origin)

**Implementation**:
- Cache-Control headers (1 hour for stable analysis)
- Content-addressed URLs (infinite cache TTL)
- Purge on file changes only

### 4. Right-Sizing and Auto-Scaling

**Strategy**: Match resources to actual workload

**Impact**:
- 30-50% cost reduction (typical over-provisioning)
- Pay only for needed capacity
- Scale down during off-hours

**Implementation**:
- Monitor utilization (CPU, memory, storage)
- Auto-scale based on queue depth and latency
- Use spot instances (AWS) for batch workloads

### 5. Database Query Optimization

**Strategy**: Optimize hot queries and indexes

**Impact**:
- 10× faster queries (typical)
- Reduce database instance size
- Lower read replica count

**Implementation**:
- Index on fingerprint columns (primary key)
- Partial indexes for recent data
- Query result caching (already implemented, Day 15)

---

## Capacity Monitoring and Alerting

### Key Metrics to Track

**Resource Utilization**:
- CPU: Average, p95, p99
- Memory: Used, available, swap
- Storage: Used, available, I/O wait
- Network: Bandwidth, packet loss

**Application Performance**:
- Fingerprint latency: Target < 1 µs
- Query latency: Target < 50 ms p95
- Cache hit rate: Target > 90%
- Throughput: 430-672 MiB/s (baseline)

**Scaling Indicators**:
- Request queue depth: Alert if > 100
- Parallel efficiency: Alert if < 50%
- Database connections: Alert if > 80% pool size
- Error rate: Alert if > 1%

### Prometheus Queries

**CPU Utilization**:
```promql
# Average CPU across all cores
100 - (avg by (instance) (irate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)

# Alert if sustained > 80%
avg_over_time((100 - avg(irate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)[15m:]) > 80
```

**Memory Pressure**:
```promql
# Memory utilization percentage
(node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes) / node_memory_MemTotal_bytes * 100

# Alert if > 85%
(node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes) / node_memory_MemTotal_bytes * 100 > 85
```

**Cache Hit Rate**:
```promql
# Cache hit rate from Day 20 metrics
thread_cache_hit_rate_percent

# Alert if < 90%
thread_cache_hit_rate_percent < 90
```

**Request Latency**:
```promql
# p95 query latency (from Day 20 metrics)
histogram_quantile(0.95, rate(thread_query_duration_seconds_bucket[5m]))

# Alert if > 50 ms
histogram_quantile(0.95, rate(thread_query_duration_seconds_bucket[5m])) > 0.050
```

### Grafana Dashboard Panels

**Panel 1: Resource Utilization Overview**
- CPU (gauge): Current, p95, p99
- Memory (gauge): Used/Total
- Storage (bar chart): Used by component
- Network (graph): Throughput over time

**Panel 2: Application Performance**
- Fingerprint latency (histogram): Distribution
- Cache hit rate (gauge): Current + 7-day trend
- Query latency (graph): p50, p95, p99 over time
- Throughput (graph): MiB/s sustained

**Panel 3: Scaling Indicators**
- Queue depth (graph): Current + threshold line
- Parallel efficiency (gauge): Percentage
- Database connections (gauge): Used/Pool size
- Error rate (graph): Percentage over time

**Panel 4: Cost Tracking**
- Estimated monthly cost (stat): Based on usage
- Resource cost breakdown (pie chart): Compute, storage, DB
- Cost trend (graph): Daily over 30 days
- Optimization opportunities (table): Recommendations

---

## Deployment Topology Decision Tree

### Decision Factors

**1. Project Size**
- Small (< 100 files): Single-node CLI OR Edge free tier
- Medium (100-1,000 files): Multi-core CLI OR Edge paid
- Large (1,000-10,000 files): High-memory CLI OR Edge with D1
- Enterprise (> 10,000 files): CLI cluster OR Edge Enterprise

**2. Latency Requirements**
- < 10 ms: Edge deployment (CDN proximity)
- < 50 ms: Single-node CLI (local) OR Edge
- < 500 ms: Multi-node CLI (local datacenter)
- > 500 ms: Batch processing acceptable

**3. Geographic Distribution**
- Single region: CLI deployment
- Multi-region: Edge deployment (automatic)
- Global: Edge Enterprise (200+ locations)

**4. Cost Sensitivity**
- Budget < $50/month: Edge free tier OR small CLI
- Budget $50-500/month: Edge paid OR medium CLI
- Budget $500-3,000/month: Large CLI OR Edge Enterprise
- Budget > $3,000/month: CLI cluster with HA

**5. Data Privacy and Compliance**
- On-premises required: CLI only (no cloud)
- Regional data residency: CLI in specific region OR Edge with region lock
- Global deployment OK: Edge (optimal)

### Recommended Topologies

**Topology 1: Development / Small Projects**
```
┌─────────────────────────────────────┐
│  Single-Node CLI                    │
│  ├─ 2 cores, 1 GB memory           │
│  ├─ Postgres (local)                │
│  └─ Cost: ~$15/month                │
└─────────────────────────────────────┘

OR

┌─────────────────────────────────────┐
│  Cloudflare Workers (Free Tier)     │
│  ├─ Auto-scaling                    │
│  ├─ D1 (5 GB included)              │
│  └─ Cost: Free (< 100K req/day)     │
└─────────────────────────────────────┘
```

**Topology 2: Production / Medium Projects**
```
┌─────────────────────────────────────────────┐
│  Multi-Core CLI                             │
│  ├─ 8 cores, 8 GB memory                   │
│  ├─ Rayon parallel processing              │
│  ├─ Postgres (db.t3.micro)                 │
│  └─ Cost: ~$46/month                        │
└─────────────────────────────────────────────┘

OR

┌─────────────────────────────────────────────┐
│  Cloudflare Workers + D1                    │
│  ├─ Global edge distribution               │
│  ├─ Content-addressed caching (99%+ hit)   │
│  ├─ D1 storage (10 GB)                     │
│  └─ Cost: ~$10-15/month                     │
└─────────────────────────────────────────────┘
```

**Topology 3: Enterprise / Large Projects**
```
┌────────────────────────────────────────────────────────┐
│  CLI Cluster (Kubernetes)                              │
│  ├─ Coordinator: 4 cores, 8 GB                        │
│  ├─ Workers: 5 × (16 cores, 32 GB)                    │
│  ├─ Postgres Multi-AZ (HA)                            │
│  ├─ Qdrant cluster (3 nodes)                          │
│  ├─ Load balancer                                     │
│  └─ Cost: ~$2,782/month                                │
└────────────────────────────────────────────────────────┘

OR

┌────────────────────────────────────────────────────────┐
│  Cloudflare Edge Enterprise                            │
│  ├─ Global CDN (200+ locations)                       │
│  ├─ D1 multi-region (500 GB)                          │
│  ├─ Durable Objects (state)                           │
│  ├─ Unlimited CPU time                                │
│  └─ Cost: ~$350-500/month                              │
└────────────────────────────────────────────────────────┘
```

**Topology 4: Hybrid (Best of Both)**
```
┌─────────────────────────────────────────────────────────────┐
│  Hybrid Deployment                                          │
│  ├─ Edge (Primary): Fast global reads                      │
│  │   └─ Cloudflare Workers + D1 cache                      │
│  ├─ CLI (Analysis): Heavy computation                      │
│  │   └─ Multi-node cluster + Postgres                      │
│  ├─ Sync: Fingerprint-based invalidation                   │
│  └─ Cost: ~$400-800/month (optimized)                      │
└─────────────────────────────────────────────────────────────┘

Benefits:
- Global low-latency reads (edge cache)
- Powerful analysis capabilities (CLI cluster)
- Cost-effective (cache hit rate 99%+)
- Best performance for both reads and writes
```

---

## Capacity Planning Workflow

### Phase 1: Baseline Assessment

**Step 1: Current Workload Analysis**
```bash
# Run load tests from Day 23
cargo bench -p thread-flow --bench load_test --all-features

# Capture baseline metrics
./scripts/profile.sh comprehensive

# Check current resource usage
docker stats  # or top/htop on CLI
```

**Step 2: Growth Projection**
- Estimate file count growth: +X% per month
- Estimate request volume growth: +Y% per month
- Estimate storage growth: Z MB per month
- Calculate resource needs in 6-12 months

**Step 3: Cost Modeling**
- Current cost: Calculate from resource usage
- Projected cost (6 months): Linear growth
- Projected cost (12 months): With optimizations
- Budget constraints: Maximum acceptable cost

### Phase 2: Topology Selection

**Decision Matrix**:
| Factor | CLI | Edge | Hybrid |
|--------|-----|------|--------|
| Small project | ✅ Best | ✅ Best (free) | ❌ Overkill |
| Medium project | ✅ Good | ✅ Best | ⚠️ Optional |
| Large project | ✅ Best | ⚠️ Expensive | ✅ Best |
| Enterprise | ✅ Best | ✅ Good | ✅ Best |
| Low latency | ⚠️ Regional | ✅ Best | ✅ Best |
| On-premises | ✅ Only option | ❌ Cloud only | ⚠️ CLI only |
| Budget < $50 | ✅ Good | ✅ Best | ❌ Too costly |
| Budget > $500 | ✅ Best | ✅ Good | ✅ Best |

### Phase 3: Implementation and Validation

**Step 1: Deploy Pilot**
- Start with smaller scale (50% of projected need)
- Monitor for 2-4 weeks
- Adjust based on actual usage patterns

**Step 2: Load Testing**
```bash
# Test at 150% projected load
cargo bench -p thread-flow --bench load_test -- --test-threads 8

# Stress test to failure point
./scripts/load-test.sh --requests 100000 --concurrency 100
```

**Step 3: Capacity Validation**
- Verify SLO compliance under load
- Check scaling triggers activate correctly
- Validate cost projections against actual usage

### Phase 4: Continuous Optimization

**Monthly Review**:
- Analyze cost trends (compare to budget)
- Review capacity utilization (find waste)
- Update projections based on actual growth
- Optimize configuration for efficiency

**Quarterly Planning**:
- Re-run capacity analysis
- Adjust topology if needed (scale up/down)
- Review SLO compliance (adjust targets if needed)
- Update cost models with new pricing

---

## Best Practices

### 1. Plan for Peak Load, Not Average

**Antipattern**: Size for average load (leads to SLO violations during peaks)

**Best Practice**: Size for p95 load + 20-30% headroom

**Example**:
- Average load: 1,000 requests/minute
- p95 load: 5,000 requests/minute
- Capacity target: 6,500 requests/minute (30% headroom)

### 2. Monitor Leading Indicators

**Antipattern**: React to failures (CPU 100%, OOM crashes)

**Best Practice**: Alert on trends before capacity exhaustion

**Example**:
- Alert at 70% CPU (not 90%+)
- Alert on cache hit rate decline (trend, not absolute)
- Alert on request queue growth (leading indicator)

### 3. Test Failure Scenarios

**Antipattern**: Assume infrastructure always works

**Best Practice**: Chaos engineering and failover testing

**Example**:
- Kill random worker nodes (test load balancing)
- Simulate database outage (test fallback caching)
- Network partition tests (test eventual consistency)

### 4. Optimize for Cost Efficiency

**Antipattern**: Always choose latest/largest instances

**Best Practice**: Right-size and use cost-effective options

**Example**:
- Use spot instances for batch workloads (70% cost reduction)
- Leverage edge caching to reduce origin load (99%+ hit rate)
- Auto-scale down during off-hours (50% cost reduction)

### 5. Document Capacity Decisions

**Antipattern**: Tribal knowledge, no written rationale

**Best Practice**: Document assumptions, calculations, trade-offs

**Example**:
- Why 8 cores? "Load tests showed 4-core saturation at 1,500 req/min"
- Why Postgres not DynamoDB? "Relational queries + cost ($140 vs $280/mo)"
- Why hybrid topology? "Edge for reads (99% traffic), CLI for writes"

---

## Troubleshooting Common Capacity Issues

### Issue 1: High CPU but Low Throughput

**Symptoms**:
- CPU at 80%+ sustained
- Request latency high (> 500 ms p95)
- Throughput below baseline (< 200 MiB/s)

**Root Causes**:
1. **Thread contention**: Too many threads for available cores
2. **I/O blocking**: CPU waiting on disk or network
3. **Inefficient algorithms**: O(n²) complexity in hot path

**Diagnosis**:
```bash
# Check thread contention
./scripts/profile.sh perf benchmark_name

# Look for:
# - High idle time (I/O bound)
# - Lock contention (std::sync patterns)
# - Excessive syscalls (read/write)
```

**Resolution**:
- Reduce thread count (match CPU cores)
- Optimize I/O (batch operations, async)
- Profile hot path (flamegraph) and optimize algorithms

### Issue 2: Cache Hit Rate Below Target

**Symptoms**:
- Cache hit rate < 90%
- High database load
- Increased latency (cache misses expensive)

**Root Causes**:
1. **Cache size too small**: Evicting working set
2. **Cache TTL too short**: Premature eviction
3. **Workload changed**: New access patterns

**Diagnosis**:
```bash
# Check cache metrics from Day 20
curl http://localhost:9090/api/v1/query?query=thread_cache_hit_rate_percent

# Check eviction rate
curl http://localhost:9090/api/v1/query?query=rate(thread_cache_evictions_total[5m])
```

**Resolution**:
- Increase cache size (2× current)
- Increase TTL (e.g., 1 hour → 24 hours for stable data)
- Add cache warming for common queries

### Issue 3: Database Connection Pool Exhaustion

**Symptoms**:
- "Too many connections" errors
- High connection acquisition time
- Request timeouts

**Root Causes**:
1. **Connection leaks**: Not releasing connections
2. **Pool too small**: Insufficient for workload
3. **Long-running queries**: Holding connections

**Diagnosis**:
```sql
-- Check current connections (Postgres)
SELECT count(*) FROM pg_stat_activity;

-- Check connection age
SELECT client_addr, state, now() - query_start as duration
FROM pg_stat_activity
ORDER BY duration DESC;
```

**Resolution**:
- Fix connection leaks (ensure Drop/close)
- Increase pool size (current × 1.5)
- Add query timeout (30 seconds max)
- Optimize long-running queries

### Issue 4: Storage Exhaustion

**Symptoms**:
- Disk usage > 90%
- Write failures
- Database degradation

**Root Causes**:
1. **Cache growth unbounded**: No eviction policy
2. **Log accumulation**: Not rotating/pruning
3. **Database growth**: No VACUUM or archival

**Diagnosis**:
```bash
# Check disk usage by directory
du -sh /var/lib/postgresql/data/*

# Check largest tables (Postgres)
SELECT schemaname, tablename, pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename))
FROM pg_tables
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC
LIMIT 10;
```

**Resolution**:
- Implement cache eviction (LRU with size limit)
- Configure log rotation (daily, 7-day retention)
- Run VACUUM FULL on large tables
- Archive old data (> 90 days) to cold storage

---

## Appendix A: Capacity Planning Calculator

### CLI Deployment Calculator

```python
# Thread CLI Capacity Calculator

def calculate_cli_capacity(
    file_count: int,
    avg_file_size_kb: int,
    requests_per_minute: int,
    cache_hit_rate: float = 0.9,
    parallel_cores: int = 4
) -> dict:
    """
    Calculate required CLI capacity

    Returns:
        {
            'cpu_cores': int,
            'memory_gb': int,
            'storage_gb': int,
            'estimated_cost_usd': float
        }
    """
    # Fingerprint time: 425 ns per file
    fingerprint_time_ms = (file_count * 0.000425)

    # Cache miss analysis time: 147 µs per file (worst case)
    cache_miss_time_ms = (file_count * 0.147 * (1 - cache_hit_rate))

    # Total time per request
    total_time_ms = fingerprint_time_ms + cache_miss_time_ms

    # Required capacity (requests per minute)
    capacity_rps = requests_per_minute / 60.0

    # CPU cores needed (with 30% headroom)
    cpu_utilization = (capacity_rps * total_time_ms) / 1000.0
    cpu_cores = max(2, int(cpu_utilization * 1.3 / 0.7))  # 70% target utilization

    # Memory (rule of thumb: 2 GB base + 2 MB per file)
    memory_gb = max(2, int(2 + (file_count * 0.002)))

    # Storage (cache: 1 KB/file, history: 10%, overhead: 2×)
    storage_gb = max(5, int((file_count * 0.001 * 1.1 * 2)))

    # Cost estimation (AWS EC2 + RDS rough estimate)
    if cpu_cores <= 2 and memory_gb <= 2:
        instance_cost = 15  # t3.small
    elif cpu_cores <= 4 and memory_gb <= 8:
        instance_cost = 46  # t3.medium + db.t3.micro
    elif cpu_cores <= 8 and memory_gb <= 16:
        instance_cost = 120  # c5.2xlarge + db.m5.large
    else:
        instance_cost = 450  # c5.4xlarge + db.m5.2xlarge

    storage_cost = storage_gb * 0.08  # EBS gp3

    return {
        'cpu_cores': cpu_cores,
        'memory_gb': memory_gb,
        'storage_gb': storage_gb,
        'estimated_cost_usd': instance_cost + storage_cost,
        'expected_latency_ms': total_time_ms / parallel_cores if cpu_cores >= parallel_cores else total_time_ms
    }

# Example usage
print(calculate_cli_capacity(
    file_count=5000,
    avg_file_size_kb=50,
    requests_per_minute=60,
    cache_hit_rate=0.95,
    parallel_cores=8
))
```

### Edge Deployment Calculator

```python
# Thread Edge Capacity Calculator

def calculate_edge_capacity(
    file_count: int,
    requests_per_day: int,
    cache_hit_rate: float = 0.99,
    d1_storage_gb: int = 10
) -> dict:
    """
    Calculate Cloudflare Workers + D1 costs

    Returns:
        {
            'worker_requests': int,
            'd1_storage_gb': int,
            'estimated_cost_usd': float
        }
    """
    # Workers pricing
    base_cost = 5.0  # $5/month base

    # Requests beyond 10M/month
    included_requests = 10_000_000
    additional_requests = max(0, requests_per_day * 30 - included_requests)
    request_cost = (additional_requests / 1_000_000) * 0.50  # $0.50/million

    # D1 storage
    included_storage = 5  # 5 GB included
    additional_storage = max(0, d1_storage_gb - included_storage)
    storage_cost = 5.0 if additional_storage > 0 else 0  # $5/month for up to 10GB
    storage_cost += max(0, additional_storage - 5) * 1.0  # $1/GB beyond 10GB

    # Durable Objects (if needed for large projects)
    durable_objects_cost = 0
    if file_count > 10000:
        durable_objects_cost = 5.0 + (requests_per_day * 30 / 1_000_000) * 0.15

    return {
        'worker_requests': requests_per_day * 30,
        'd1_storage_gb': d1_storage_gb,
        'cache_hit_rate': cache_hit_rate,
        'estimated_cost_usd': base_cost + request_cost + storage_cost + durable_objects_cost,
        'expected_latency_ms': 50 if cache_hit_rate > 0.95 else 100  # p95 estimate
    }

# Example usage
print(calculate_edge_capacity(
    file_count=5000,
    requests_per_day=100_000,
    cache_hit_rate=0.99,
    d1_storage_gb=15
))
```

---

## Appendix B: Capacity Planning Checklist

### Pre-Deployment

- [ ] Workload analysis complete (file count, request volume, growth rate)
- [ ] Topology selected (CLI, Edge, or Hybrid)
- [ ] Resource requirements calculated (CPU, memory, storage)
- [ ] Database capacity planned (Postgres, D1, Qdrant)
- [ ] Cost model validated (within budget constraints)
- [ ] SLO targets defined (latency, throughput, availability)
- [ ] Monitoring configured (Prometheus metrics, Grafana dashboards)
- [ ] Load testing completed (Day 23 benchmarks)
- [ ] Scaling thresholds configured (CPU, memory, queue depth)
- [ ] Documentation updated (topology diagram, capacity plan)

### Post-Deployment

- [ ] Baseline metrics captured (CPU, memory, latency, cache hit rate)
- [ ] Monitoring alerts configured (capacity warnings before exhaustion)
- [ ] Auto-scaling tested (scale-up and scale-down verified)
- [ ] Failover tested (database, worker node failures)
- [ ] Cost tracking enabled (actual vs projected)
- [ ] Capacity review scheduled (monthly)
- [ ] Growth projections updated (based on actual trends)
- [ ] Optimization opportunities identified (efficiency gains)
- [ ] Incident runbooks created (capacity exhaustion, scaling failures)
- [ ] Capacity plan documented (for future reference)

### Monthly Review

- [ ] Review actual vs projected growth (file count, requests, cost)
- [ ] Check resource utilization trends (identify waste or constraints)
- [ ] Validate SLO compliance (latency, availability, cache hit rate)
- [ ] Update capacity projections (6-month, 12-month forecast)
- [ ] Identify optimization opportunities (cost reduction, efficiency)
- [ ] Adjust scaling thresholds if needed (based on actual behavior)
- [ ] Review incident history (capacity-related outages)
- [ ] Update capacity plan documentation

---

**Document Version**: 1.0.0
**Last Updated**: 2026-01-28
**Next Review**: 2026-02-28
**Owner**: Thread Operations Team
