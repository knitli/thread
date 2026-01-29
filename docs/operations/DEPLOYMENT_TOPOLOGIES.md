# Deployment Topologies

**Version**: 1.0.0
**Last Updated**: 2026-01-28
**Status**: Production Ready

---

## Overview

This document describes deployment architecture patterns for Thread across CLI and Edge environments. It covers topology design, database placement, geographic distribution, and hybrid architectures.

### Purpose

- **Architecture Guidance**: Choose appropriate topology for requirements
- **Scalability Planning**: Design for growth and geographic expansion
- **High Availability**: Ensure service continuity through redundancy
- **Cost Optimization**: Balance performance needs with infrastructure costs

### Integration Points

- **Day 15 Performance**: Content-addressed caching, parallel processing
- **Day 20 Monitoring**: Health checks, metrics collection, observability
- **Day 23 Optimization**: Load testing, performance benchmarks
- **Day 24 Capacity Planning**: Resource requirements, scaling strategies
- **Day 24 Load Balancing**: Request routing, failover mechanisms

---

## Topology Decision Framework

### Decision Factors

**1. Project Size and Complexity**
- Small (< 100 files): Single-node or Edge free tier
- Medium (100-1,000 files): Multi-core CLI or Edge paid
- Large (1,000-10,000 files): High-memory CLI or Edge with D1
- Enterprise (> 10,000 files): CLI cluster or Edge Enterprise

**2. Performance Requirements**
- Latency SLO < 10 ms: Edge deployment (CDN proximity)
- Latency SLO < 50 ms: Multi-core CLI (local) or Edge
- Latency SLO < 500 ms: Single-node CLI or standard deployment
- Batch processing: CLI with high parallelism

**3. Geographic Distribution**
- Single region: CLI deployment in target region
- Multi-region: Edge deployment (automatic routing)
- Global: Edge Enterprise (200+ locations worldwide)
- Specific regions: Hybrid (CLI per region + Edge global)

**4. Data Privacy and Compliance**
- On-premises required: CLI-only (no cloud services)
- Regional data residency: CLI in specific region with isolation
- GDPR/Privacy Shield: Edge with region lock or CLI in EU
- General cloud: Edge (optimal cost and performance)

**5. Budget Constraints**
- < $50/month: Edge free tier or small CLI
- $50-500/month: Edge paid or medium CLI
- $500-3,000/month: Large CLI or Edge Enterprise
- > $3,000/month: CLI cluster with HA

**6. Operational Expertise**
- Self-managed infrastructure: CLI deployment (full control)
- Managed services preferred: Edge deployment (Cloudflare managed)
- Kubernetes expertise: CLI on K8s (containerized)
- Minimal ops: Edge (zero infrastructure management)

### Decision Matrix

| Factor | Single-Node CLI | Multi-Node CLI | Edge Standard | Edge Enterprise | Hybrid |
|--------|-----------------|----------------|---------------|-----------------|--------|
| **Cost** | Low ($15-50) | Medium ($50-500) | Low-Medium ($0-150) | High ($350-500) | High ($400-800) |
| **Latency** | Regional (50-100ms) | Regional (10-50ms) | Global (<50ms) | Global (<20ms) | Global (<20ms) |
| **Scale** | 100-1K files | 1K-10K files | 1K-10K files | 10K-100K files | 10K-100K files |
| **HA** | Single point of failure | Active-active HA | Automatic HA | Automatic HA | Maximum HA |
| **Ops Complexity** | Low | Medium-High | Minimal | Minimal | High |
| **Geographic** | Single region | Single/multi-region | Global (auto) | Global (200+ PoPs) | Global + regional |
| **On-Premises** | ✅ Yes | ✅ Yes | ❌ Cloud only | ❌ Cloud only | ⚠️ Partial (CLI) |

---

## Topology Patterns

### Pattern 1: Single-Node CLI (Development/Small Projects)

**Architecture**:
```
┌────────────────────────────────────────┐
│  Thread CLI                            │
│  ├─ 2-4 CPU cores                     │
│  ├─ 1-4 GB memory                     │
│  ├─ Rayon thread pool                 │
│  └─ Local Postgres (embedded)         │
└────────────────────────────────────────┘
         │
         ├─ Analysis requests (local API)
         └─ File fingerprinting (local storage)
```

**Characteristics**:
- **Deployment**: Single VM/bare metal server
- **Database**: Postgres (single instance, local)
- **Caching**: In-process cache (no external cache)
- **Parallelism**: Rayon (multi-core within process)
- **Geographic**: Single datacenter

**Resource Requirements**:
- CPU: 2-4 cores (Intel Xeon or AMD EPYC)
- Memory: 1-4 GB
- Storage: 10-50 GB (SSD recommended)
- Network: 100 Mbps minimum

**Cost Estimate**:
- AWS EC2 t3.small: ~$15/month
- DigitalOcean Droplet (2 CPU, 2GB): ~$12/month
- Self-hosted (amortized): ~$5-10/month

**Use Cases**:
- Local development and testing
- Small projects (< 100 files)
- Single-user workflows
- Prototyping and POC

**Scaling Limitations**:
- Single point of failure (no HA)
- Limited to single-node resources
- Manual vertical scaling only
- Not suitable for > 1,000 files

**Deployment Steps**:
```bash
# 1. Install dependencies
sudo apt update && sudo apt install -y postgresql-14

# 2. Configure Postgres
sudo -u postgres createdb thread
sudo -u postgres psql -c "CREATE USER thread WITH PASSWORD 'secure_password';"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE thread TO thread;"

# 3. Build Thread
cd thread
cargo build --release --features parallel

# 4. Configure environment
cat > .env <<EOF
DATABASE_URL=postgresql://thread:secure_password@localhost/thread
RUST_LOG=info
RAYON_NUM_THREADS=4
EOF

# 5. Run migrations
diesel migration run

# 6. Start Thread
./target/release/thread-cli serve --port 8080
```

---

### Pattern 2: Multi-Node CLI Cluster (Production/Medium-Large Projects)

**Architecture**:
```
                     ┌─────────────────────┐
                     │   Load Balancer     │
                     │   (HAProxy/Nginx)   │
                     └──────────┬──────────┘
                                │
            ┌───────────────────┼───────────────────┐
            │                   │                   │
    ┌───────▼────────┐  ┌──────▼────────┐  ┌──────▼────────┐
    │ Thread Worker 1│  │ Thread Worker 2│  │ Thread Worker 3│
    │ 16 cores, 32GB │  │ 16 cores, 32GB │  │ 16 cores, 32GB │
    │ Rayon parallel │  │ Rayon parallel │  │ Rayon parallel │
    └───────┬────────┘  └───────┬───────┘  └───────┬───────┘
            │                   │                   │
            └───────────────────┼───────────────────┘
                                │
                     ┌──────────▼──────────┐
                     │  Postgres Cluster   │
                     │  (Primary + Replica)│
                     │  + Qdrant (Vector)  │
                     └─────────────────────┘
```

**Characteristics**:
- **Deployment**: 3-10 worker nodes behind load balancer
- **Database**: Postgres Multi-AZ with read replicas
- **Caching**: Distributed cache (Redis) + content-addressed caching
- **Parallelism**: Rayon within each worker + horizontal scaling
- **Geographic**: Single or multi-region

**Resource Requirements per Worker**:
- CPU: 8-16 cores (Intel Xeon Gold or AMD EPYC)
- Memory: 16-32 GB
- Storage: 100-500 GB (NVMe SSD)
- Network: 10 Gbps

**Cost Estimate**:
- 5 × AWS EC2 c5.4xlarge: ~$1,224/month (workers)
- RDS Postgres Multi-AZ (db.r5.2xlarge): ~$840/month
- Qdrant cluster (3 × r5.xlarge): ~$540/month
- Load balancer + storage: ~$100/month
- **Total**: ~$2,700/month

**Use Cases**:
- Production workloads (1,000-10,000 files)
- High-availability requirements (99.9%+ uptime)
- Multi-tenant SaaS platforms
- Enterprise deployments

**Scaling Capabilities**:
- Horizontal scaling (add/remove workers)
- Automatic failover (load balancer health checks)
- Database read scaling (add replicas)
- Geographic distribution (multi-region workers)

**Deployment Steps** (Kubernetes):
```yaml
# thread-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: thread-worker
  namespace: thread
spec:
  replicas: 5
  selector:
    matchLabels:
      app: thread-worker
  template:
    metadata:
      labels:
        app: thread-worker
    spec:
      containers:
      - name: thread
        image: thread:latest
        resources:
          requests:
            cpu: "8000m"
            memory: "16Gi"
          limits:
            cpu: "16000m"
            memory: "32Gi"
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: thread-secrets
              key: database-url
        - name: RAYON_NUM_THREADS
          value: "16"
        ports:
        - containerPort: 8080
        livenessProbe:
          httpGet:
            path: /health/live
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
```

---

### Pattern 3: Edge Deployment (Cloudflare Workers + D1)

**Architecture**:
```
User Request (Global)
       │
       ├─ Americas → CF Edge (New York/LA/São Paulo)
       ├─ Europe → CF Edge (London/Frankfurt/Paris)
       └─ Asia → CF Edge (Singapore/Tokyo/Sydney)
              │
        ┌─────▼──────────────────────────┐
        │   Cloudflare Worker (Isolate)  │
        │   ├─ WASM runtime (128 MB)     │
        │   ├─ Content-addressed cache   │
        │   └─ Async processing (tokio)  │
        └────────┬───────────────────────┘
                 │
        ┌────────▼────────────────────────┐
        │   D1 Database (SQLite on edge)  │
        │   ├─ Primary (write region)     │
        │   └─ Replicas (global reads)    │
        └─────────────────────────────────┘
```

**Characteristics**:
- **Deployment**: Cloudflare CDN (200+ global locations)
- **Database**: D1 (distributed SQLite with multi-region replication)
- **Caching**: Edge KV + content-addressed fingerprints
- **Parallelism**: Automatic horizontal scaling (isolates)
- **Geographic**: Global (automatic edge routing)

**Resource Limits** (Cloudflare Workers):
- CPU time: 50 ms per request (free), unlimited (Enterprise)
- Memory: 128 MB per isolate
- Request size: 100 MB
- D1 storage: 5 GB (free), 10+ GB (paid)

**Cost Estimate**:
- Workers base: $5/month
- D1 storage (10 GB): $5/month
- Requests (10M/month): Included in base
- Additional requests: $0.50/million
- **Small project**: ~$10/month
- **Medium project**: ~$50/month
- **Large project**: ~$150/month

**Use Cases**:
- Global user base (low latency worldwide)
- Cost-sensitive deployments
- Variable traffic patterns (auto-scaling)
- Minimal operational overhead

**Scaling Capabilities**:
- Automatic horizontal scaling (unlimited concurrency)
- Geographic distribution (200+ PoPs)
- Zero configuration (Cloudflare managed)
- Pay-per-request pricing (cost scales with usage)

**Deployment Steps**:
```bash
# 1. Install Wrangler CLI
npm install -g wrangler

# 2. Authenticate
wrangler login

# 3. Create D1 database
wrangler d1 create thread-db

# 4. Update wrangler.toml
cat > wrangler.toml <<EOF
name = "thread-worker"
main = "src/index.ts"
compatibility_date = "2024-01-01"

[[d1_databases]]
binding = "DB"
database_name = "thread-db"
database_id = "<your-database-id>"

[[kv_namespaces]]
binding = "CACHE"
id = "<your-kv-namespace-id>"
EOF

# 5. Build WASM
cargo run -p xtask build-wasm --release

# 6. Deploy
wrangler deploy
```

---

### Pattern 4: Edge Enterprise (Global Low-Latency)

**Architecture**:
```
                Global User Base
                       │
         ┌─────────────┼─────────────┐
         │             │             │
    Americas        Europe          Asia
         │             │             │
    ┌────▼────┐   ┌────▼────┐   ┌───▼─────┐
    │ CF Edge │   │ CF Edge │   │ CF Edge │
    │ (5 PoPs)│   │ (10 PoPs)│  │ (8 PoPs)│
    └────┬────┘   └────┬────┘   └────┬────┘
         │             │              │
         └─────────────┼──────────────┘
                       │
            ┌──────────▼───────────┐
            │ Durable Objects      │
            │ (Stateful Workers)   │
            │ + D1 Multi-Region    │
            └──────────────────────┘
```

**Characteristics**:
- **Deployment**: Cloudflare Enterprise (200+ global PoPs)
- **Database**: D1 multi-region + Durable Objects (state)
- **Caching**: Multi-tier (edge KV + D1 + Durable Objects)
- **Parallelism**: Massive horizontal scaling (enterprise limits)
- **Geographic**: Global with regional failover

**Resource Limits** (Enterprise):
- CPU time: Unlimited
- Memory: 128 MB per isolate (can request increase)
- Request size: 500 MB
- D1 storage: Custom (100+ GB)
- Durable Objects: Unlimited

**Cost Estimate**:
- Enterprise plan: $200/month base
- D1 storage (500 GB): $100/month
- Durable Objects: $50/month
- Requests (100M/month): Included
- **Total**: ~$350-500/month

**Use Cases**:
- Global enterprise deployments
- Extreme low-latency requirements (< 20 ms p95)
- Very large projects (10,000+ files)
- Mission-critical applications (99.99% SLO)

**Scaling Capabilities**:
- Unlimited horizontal scaling
- 200+ global PoPs (automatic routing)
- Custom enterprise limits
- Dedicated support and SLA

---

### Pattern 5: Hybrid (Edge + CLI)

**Architecture**:
```
       Global Users
            │
      ┌─────▼─────────────┐
      │ Cloudflare Edge   │  ← Read path (99%+ cache hit)
      │ (Workers + D1)    │     • Fingerprint cache
      │ Cache-first reads │     • Query result cache
      └────────┬──────────┘     • < 20 ms p95 latency
               │
    Cache MISS │ (< 1%)
               │
      ┌────────▼──────────┐
      │ CLI Cluster       │  ← Write path + complex analysis
      │ (Multi-node)      │     • Full analysis engine
      │ Heavy computation │     • Postgres + Qdrant
      └───────────────────┘     • 100-500 ms analysis time
```

**Characteristics**:
- **Read Path**: Edge (global, low-latency, cached reads)
- **Write Path**: CLI cluster (powerful analysis, persistent storage)
- **Sync**: Fingerprint-based invalidation (edge cache purge on write)
- **Database**: CLI Postgres (primary) + Edge D1 (cache/replica)
- **Geographic**: Global (edge reads) + Regional (CLI writes)

**Resource Requirements**:
- **Edge**: Standard Cloudflare Workers + D1
- **CLI Cluster**: 3-5 worker nodes + Postgres
- **Sync Infrastructure**: Message queue (SQS/Redis) for cache invalidation

**Cost Estimate**:
- Edge (Workers + D1): ~$50-100/month
- CLI cluster (3 workers + DB): ~$300-500/month
- Sync infrastructure: ~$20/month
- **Total**: ~$370-620/month

**Use Cases**:
- Best of both worlds (global reads + powerful writes)
- Large user base with heavy analysis needs
- Cost optimization (cache offloads expensive CLI)
- Geographic distribution with centralized intelligence

**Scaling Capabilities**:
- Independent scaling of read and write paths
- Edge auto-scales for reads (no limit)
- CLI scales for write throughput
- 99%+ cache hit rate reduces CLI load dramatically

**Deployment Architecture**:
```typescript
// Edge Worker (Read path)
export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const fingerprint = await computeFingerprint(request);

    // Try edge cache first (< 1 ms)
    const edgeCached = await env.EDGE_CACHE.get(fingerprint);
    if (edgeCached) {
      return new Response(edgeCached, {
        headers: { 'X-Cache': 'edge-hit' }
      });
    }

    // Try D1 cache (< 20 ms)
    const d1Cached = await env.DB.prepare(
      'SELECT result FROM cache WHERE fingerprint = ?'
    ).bind(fingerprint).first();
    if (d1Cached) {
      await env.EDGE_CACHE.put(fingerprint, d1Cached.result);
      return new Response(d1Cached.result, {
        headers: { 'X-Cache': 'd1-hit' }
      });
    }

    // Cache miss: forward to CLI cluster (100-500 ms)
    const cliResult = await fetch(`https://cli.thread.example.com/api/analyze`, {
      method: 'POST',
      body: JSON.stringify({ fingerprint, request })
    });

    const result = await cliResult.text();

    // Cache result at both layers
    await env.EDGE_CACHE.put(fingerprint, result);
    await env.DB.prepare(
      'INSERT INTO cache (fingerprint, result) VALUES (?, ?)'
    ).bind(fingerprint, result).run();

    return new Response(result, {
      headers: { 'X-Cache': 'miss' }
    });
  }
};
```

---

## Database Placement Strategies

### Strategy 1: Co-located Database (Single Region)

**Pattern**:
```
Same Region/AZ
├─ Thread workers
└─ Postgres (< 1 ms latency)
```

**Characteristics**:
- Workers and database in same datacenter/AZ
- Minimal network latency (< 1 ms)
- Suitable for single-region deployments

**Pros**:
- Lowest latency
- Simplest configuration
- Cost-effective

**Cons**:
- Single point of failure (datacenter outage)
- Not suitable for multi-region
- Limited geographic distribution

**Use Cases**:
- Single-node CLI
- Single-region CLI cluster
- Development and testing

---

### Strategy 2: Multi-AZ Database (Regional HA)

**Pattern**:
```
Region (e.g., us-east-1)
├─ AZ-1: Thread workers + Postgres primary
├─ AZ-2: Thread workers + Postgres replica
└─ AZ-3: Postgres replica (standby)
```

**Characteristics**:
- Database replicated across availability zones
- Automatic failover on AZ failure
- Regional high availability

**Pros**:
- High availability (99.95%+)
- Automatic failover (< 30 seconds)
- Read scaling (replicas)

**Cons**:
- Higher cost (Multi-AZ RDS)
- Replication lag (1-5 seconds)
- Still regional (not global)

**Use Cases**:
- Production CLI deployments
- Regional SaaS platforms
- Compliance requirements (data residency)

**Configuration** (AWS RDS):
```terraform
resource "aws_db_instance" "thread_postgres" {
  identifier = "thread-db"
  engine = "postgres"
  engine_version = "15.5"
  instance_class = "db.r5.2xlarge"

  # Multi-AZ HA
  multi_az = true
  availability_zone = null  # Auto-select

  # Storage
  allocated_storage = 500
  storage_type = "gp3"
  storage_encrypted = true

  # Backup
  backup_retention_period = 7
  backup_window = "03:00-04:00"
  maintenance_window = "Mon:04:00-Mon:05:00"

  # Performance
  max_allocated_storage = 1000
  iops = 3000
  performance_insights_enabled = true
}
```

---

### Strategy 3: Multi-Region Database (Global Distribution)

**Pattern**:
```
Global Deployment
├─ us-east-1: Postgres primary + Thread workers
├─ eu-west-1: Postgres read replica + Thread workers
└─ ap-southeast-1: Postgres read replica + Thread workers
```

**Characteristics**:
- Primary database in home region
- Read replicas in all deployment regions
- Cross-region replication

**Pros**:
- Global read scaling
- Local reads (low latency)
- Geographic distribution

**Cons**:
- High cost (multi-region DB)
- Replication lag (regional: 100-500 ms)
- Write latency (all writes to primary)

**Use Cases**:
- Global CLI deployments
- Multi-region SaaS
- Geo-distributed user base

---

### Strategy 4: Edge Database (D1 Multi-Region)

**Pattern**:
```
Cloudflare Edge (Global)
├─ Primary region: D1 writes
└─ 200+ PoPs: D1 read replicas (automatic)
```

**Characteristics**:
- D1 handles multi-region replication automatically
- Reads from nearest edge location
- Writes to primary region

**Pros**:
- Zero configuration (Cloudflare managed)
- Global read performance (< 20 ms)
- Automatic replication
- Cost-effective

**Cons**:
- Eventual consistency (< 100 ms lag)
- Storage limits (10 GB soft limit)
- Not suitable for complex queries

**Use Cases**:
- Edge deployments
- Global read-heavy workloads
- Content-addressed caching

---

## Geographic Distribution Patterns

### Pattern 1: Single Region (Simplest)

**Deployment**:
```
US-East-1
└─ Thread workers + Database
```

**Latency Profile**:
- US East Coast: 10-20 ms
- US West Coast: 60-80 ms
- Europe: 80-120 ms
- Asia: 180-250 ms

**Use Cases**: Single-region user base, cost-sensitive deployments

---

### Pattern 2: Multi-Region CLI (Regional Optimization)

**Deployment**:
```
├─ us-east-1: Workers + Postgres (Americas)
├─ eu-west-1: Workers + Postgres (Europe)
└─ ap-southeast-1: Workers + Postgres (Asia)
```

**Latency Profile**:
- Local region: 10-20 ms
- Cross-region: 80-250 ms (if routed incorrectly)

**Use Cases**: Multi-region SaaS, data residency compliance

---

### Pattern 3: Global Edge (Optimal)

**Deployment**:
- Cloudflare: 200+ PoPs globally
- Automatic geographic routing
- Edge database replication

**Latency Profile**:
- Global: 10-50 ms p95 (nearest PoP)
- Consistent worldwide performance

**Use Cases**: Global consumer applications, low-latency requirements

---

## Topology Migration Paths

### Migration 1: Single-Node CLI → Multi-Node Cluster

**Steps**:
1. Set up load balancer (HAProxy/Nginx)
2. Deploy 2nd worker node (identical config)
3. Add worker to load balancer backend pool
4. Test traffic distribution
5. Add remaining workers incrementally
6. Upgrade database to Multi-AZ (if needed)

**Downtime**: Zero (rolling deployment)

---

### Migration 2: CLI → Edge

**Steps**:
1. Build WASM target (`cargo run -p xtask build-wasm --release`)
2. Set up D1 database and replicate data
3. Deploy Worker to staging environment
4. Test with 10% traffic (canary deployment)
5. Gradually increase traffic to Edge (10% → 50% → 100%)
6. Decommission CLI workers after full migration

**Downtime**: Zero (gradual traffic shift)

---

### Migration 3: CLI → Hybrid

**Steps**:
1. Deploy Edge workers for read path (cache-first)
2. Keep CLI cluster for write path
3. Implement cache invalidation sync (message queue)
4. Route reads to Edge, writes to CLI
5. Monitor cache hit rate (target: 99%+)
6. Scale down CLI cluster (writes only)

**Downtime**: Zero (additive deployment)

---

## Appendix: Topology Comparison

| Topology | Setup Complexity | Operational Complexity | Cost (Small/Medium/Large) | Latency (p95) | Availability |
|----------|------------------|------------------------|---------------------------|---------------|--------------|
| **Single-Node CLI** | ⭐ Low | ⭐ Low | $15/$50/$250 | Regional (50-100ms) | 99% (no HA) |
| **Multi-Node CLI** | ⭐⭐ Medium | ⭐⭐⭐ High | $300/$1,000/$2,700 | Regional (10-50ms) | 99.9% (HA) |
| **Edge Standard** | ⭐ Low | ⭐ Minimal | $10/$50/$150 | Global (20-50ms) | 99.99% (CF SLA) |
| **Edge Enterprise** | ⭐ Low | ⭐ Minimal | $350/$400/$500 | Global (10-20ms) | 99.99% (CF SLA) |
| **Hybrid** | ⭐⭐⭐ High | ⭐⭐ Medium | $370/$500/$800 | Global (10-20ms) | 99.95% (multi-tier) |

---

**Document Version**: 1.0.0
**Last Updated**: 2026-01-28
**Next Review**: 2026-02-28
**Owner**: Thread Operations Team
