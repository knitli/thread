<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Load Balancing Strategies

**Version**: 1.0.0
**Last Updated**: 2026-01-28
**Status**: Production Ready

---

## Overview

This document defines load balancing strategies for Thread deployments across CLI and Edge environments. It covers request routing, health checking, failover mechanisms, and geographic distribution patterns.

### Purpose

- **High Availability**: Ensure service continuity during node failures
- **Performance Optimization**: Distribute load for optimal resource utilization
- **Geographic Proximity**: Route requests to nearest processing node
- **Cost Efficiency**: Balance load to minimize infrastructure costs

### Integration Points

- **Day 15 Performance**: Parallel processing with Rayon (CLI), async with tokio (Edge)
- **Day 20 Monitoring**: Health checks, metrics collection, SLO tracking
- **Day 23 Optimization**: Load testing framework, performance benchmarks
- **Day 24 Capacity Planning**: Resource requirements, scaling thresholds

---

## CLI Load Balancing (Rayon Parallelism)

### Architecture Overview

Thread CLI uses Rayon for CPU-bound parallelism on multi-core systems. Load balancing occurs at the thread level within a single process.

### Rayon Thread Pool Configuration

**Default Configuration**:
```rust
use rayon::ThreadPoolBuilder;

// Thread pool initialization (CLI only, feature-gated)
#[cfg(feature = "parallel")]
pub fn init_thread_pool(num_threads: Option<usize>) -> Result<(), rayon::ThreadPoolBuildError> {
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads.unwrap_or_else(num_cpus::get))
        .thread_name(|idx| format!("thread-worker-{}", idx))
        .stack_size(4 * 1024 * 1024) // 4 MB stack per thread
        .build_global()?;

    Ok(())
}
```

**Optimal Thread Count**:
- **Small projects**: Match CPU core count (e.g., 4 threads for 4 cores)
- **Medium projects**: CPU cores (maximize parallelism)
- **Large projects**: CPU cores (avoid over-subscription)
- **Rule of thumb**: `num_threads = num_cpus` for CPU-bound workloads

### Work Stealing Algorithm

Rayon uses work stealing for dynamic load balancing:

```rust
#[cfg(feature = "parallel")]
pub fn parallel_fingerprint_batch(files: &[String]) -> Vec<Fingerprint> {
    use rayon::prelude::*;

    files.par_iter()
        .map(|content| compute_content_fingerprint(content))
        .collect()
}
```

**How Work Stealing Works**:
1. **Initial Distribution**: Tasks divided equally among threads
2. **Dynamic Balancing**: Idle threads steal work from busy threads
3. **Cache Locality**: Threads prefer local work (reduce contention)
4. **Adaptive Splitting**: Large tasks split recursively for fine-grained balance

**Benefits**:
- Automatic load balancing (no manual tuning)
- High CPU utilization (minimal idle time)
- Good cache locality (threads work on nearby data)
- Scales linearly up to core count

### CLI Multi-Node Load Balancing

For CLI cluster deployments (enterprise projects), use external load balancing.

**Option 1: HAProxy (Recommended for CLI)**

```haproxy
# haproxy.cfg
global
    maxconn 4096
    log /dev/log local0

defaults
    mode http
    timeout connect 5000ms
    timeout client 50000ms
    timeout server 50000ms
    option httplog
    option dontlognull

frontend thread_frontend
    bind *:8080
    default_backend thread_workers

backend thread_workers
    balance leastconn           # Route to least-loaded worker
    option httpchk GET /health  # Health check endpoint

    server worker1 10.0.1.10:8080 check inter 2000ms rise 2 fall 3
    server worker2 10.0.1.11:8080 check inter 2000ms rise 2 fall 3
    server worker3 10.0.1.12:8080 check inter 2000ms rise 2 fall 3
    server worker4 10.0.1.13:8080 check inter 2000ms rise 2 fall 3
    server worker5 10.0.1.14:8080 check inter 2000ms rise 2 fall 3
```

**Balancing Algorithms for CLI**:
- **leastconn**: Best for long-running analysis requests (recommended)
- **roundrobin**: Simple, fair distribution (for similar request sizes)
- **source**: Consistent routing by client IP (for cache affinity)

**Option 2: Nginx**

```nginx
# nginx.conf
upstream thread_cluster {
    least_conn;                     # Least connections algorithm

    server 10.0.1.10:8080 max_fails=3 fail_timeout=30s;
    server 10.0.1.11:8080 max_fails=3 fail_timeout=30s;
    server 10.0.1.12:8080 max_fails=3 fail_timeout=30s;
    server 10.0.1.13:8080 max_fails=3 fail_timeout=30s;
    server 10.0.1.14:8080 max_fails=3 fail_timeout=30s;
}

server {
    listen 80;

    location / {
        proxy_pass http://thread_cluster;
        proxy_next_upstream error timeout http_502 http_503 http_504;
        proxy_connect_timeout 5s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;

        # Health check
        health_check interval=10s fails=3 passes=2 uri=/health;
    }
}
```

**Option 3: Kubernetes Service**

```yaml
# thread-service.yaml
apiVersion: v1
kind: Service
metadata:
  name: thread-service
spec:
  type: LoadBalancer
  selector:
    app: thread-worker
  ports:
    - port: 80
      targetPort: 8080
      protocol: TCP
  sessionAffinity: ClientIP  # Optional: for cache affinity
  sessionAffinityConfig:
    clientIP:
      timeoutSeconds: 3600
```

---

## Edge Load Balancing (Cloudflare Workers)

### Architecture Overview

Cloudflare Workers provide automatic global load balancing through their CDN infrastructure. No manual configuration needed for basic load distribution.

### Cloudflare's Built-in Load Balancing

**Automatic Features**:
1. **Geographic Routing**: Requests route to nearest data center (200+ locations)
2. **Auto-Scaling**: Workers scale horizontally on demand (no capacity limits)
3. **Load Distribution**: Cloudflare manages request distribution across isolates
4. **Health Checking**: Automatic unhealthy worker detection and routing

**How It Works**:
```
User Request (New York)
  ↓
Cloudflare Edge (New York data center) ← Automatic routing
  ↓
Worker Isolate (spun up on demand)
  ↓
D1 Database (regional replica, nearest)
  ↓
Response (< 50 ms p95)
```

### Custom Load Balancing Logic (Advanced)

For complex routing scenarios, implement custom logic in Worker:

```typescript
// worker.ts - Custom load balancing
export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);

    // Route by request type
    if (url.pathname.startsWith('/api/analyze')) {
      return handleAnalysis(request, env);
    } else if (url.pathname.startsWith('/api/cache')) {
      return handleCache(request, env);
    }

    return new Response('Not Found', { status: 404 });
  }
};

async function handleAnalysis(request: Request, env: Env): Promise<Response> {
  // Check fingerprint cache first (99%+ hit rate)
  const fingerprint = await computeFingerprint(request);
  const cached = await env.CACHE.get(fingerprint);

  if (cached) {
    return new Response(cached, {
      headers: {
        'Content-Type': 'application/json',
        'Cache-Control': 'public, max-age=3600',
        'X-Cache-Status': 'HIT'
      }
    });
  }

  // Cache miss: analyze and store
  const result = await analyzeCode(request, env);
  await env.CACHE.put(fingerprint, result, { expirationTtl: 3600 });

  return new Response(result, {
    headers: {
      'Content-Type': 'application/json',
      'Cache-Control': 'public, max-age=3600',
      'X-Cache-Status': 'MISS'
    }
  });
}
```

### Geographic Load Balancing with Durable Objects

For stateful workloads, use Durable Objects for consistent routing:

```typescript
// durable-object.ts
export class AnalysisCoordinator {
  constructor(private state: DurableObjectState, private env: Env) {}

  async fetch(request: Request): Promise<Response> {
    // Durable Object ensures all requests for same project
    // route to same instance (for consistent caching)
    const projectId = new URL(request.url).searchParams.get('project');

    // Get cached analysis state
    const cached = await this.state.storage.get(`analysis-${projectId}`);
    if (cached) {
      return new Response(JSON.stringify(cached), {
        headers: { 'X-Cache-Status': 'HIT' }
      });
    }

    // Perform analysis and cache
    const result = await this.analyzeProject(projectId);
    await this.state.storage.put(`analysis-${projectId}`, result);

    return new Response(JSON.stringify(result), {
      headers: { 'X-Cache-Status': 'MISS' }
    });
  }

  private async analyzeProject(projectId: string): Promise<any> {
    // Analysis logic here
    return { projectId, analyzed: true };
  }
}
```

**Durable Object Routing**:
```typescript
// worker.ts
export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    const projectId = url.searchParams.get('project');

    // Route to Durable Object based on project ID
    const id = env.COORDINATOR.idFromName(projectId);
    const stub = env.COORDINATOR.get(id);

    return stub.fetch(request);
  }
};
```

### Multi-Region D1 Load Balancing

D1 provides automatic read replica routing:

```typescript
// d1-load-balancing.ts
export async function queryD1(env: Env, query: string): Promise<any> {
  // D1 automatically routes to nearest read replica
  // Writes go to primary region
  const stmt = env.DB.prepare(query);

  // Read query (routed to nearest replica)
  if (query.trim().toUpperCase().startsWith('SELECT')) {
    return stmt.all();
  }

  // Write query (routed to primary)
  return stmt.run();
}
```

**D1 Replication Architecture**:
```
Primary Region (us-east-1)
  ├─ Write operations (INSERT, UPDATE, DELETE)
  └─ Replicates to read replicas (async, < 100 ms lag)

Read Replicas (global)
  ├─ Europe (eu-west-1) ← Auto-routed for European users
  ├─ Asia (ap-southeast-1) ← Auto-routed for Asian users
  └─ Americas (us-west-1) ← Auto-routed for West Coast users
```

---

## Health Checking and Failover

### Health Check Endpoints

**CLI Health Check**:
```rust
// src/health.rs
use axum::{Router, routing::get, Json};
use serde::Serialize;

#[derive(Serialize)]
struct HealthStatus {
    status: String,
    version: String,
    uptime_seconds: u64,
    checks: HealthChecks,
}

#[derive(Serialize)]
struct HealthChecks {
    database: bool,
    cache: bool,
    thread_pool: bool,
}

pub fn health_router() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check))
        .route("/health/live", get(liveness_check))
}

async fn health_check() -> Json<HealthStatus> {
    Json(HealthStatus {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: get_uptime(),
        checks: HealthChecks {
            database: check_database().await,
            cache: check_cache().await,
            thread_pool: check_thread_pool(),
        },
    })
}

async fn readiness_check() -> (StatusCode, &'static str) {
    // Ready to accept traffic?
    if check_database().await && check_cache().await {
        (StatusCode::OK, "ready")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "not ready")
    }
}

async fn liveness_check() -> (StatusCode, &'static str) {
    // Process still alive?
    (StatusCode::OK, "alive")
}
```

**Edge Health Check**:
```typescript
// worker.ts - Health endpoint
export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);

    if (url.pathname === '/health') {
      return new Response(JSON.stringify({
        status: 'healthy',
        timestamp: Date.now(),
        checks: {
          d1: await checkD1(env),
          cache: await checkCache(env)
        }
      }), {
        headers: { 'Content-Type': 'application/json' },
        status: 200
      });
    }

    // ... other routes
  }
};

async function checkD1(env: Env): Promise<boolean> {
  try {
    await env.DB.prepare('SELECT 1').first();
    return true;
  } catch {
    return false;
  }
}

async function checkCache(env: Env): Promise<boolean> {
  try {
    await env.CACHE.get('health-check-key');
    return true;
  } catch {
    return false;
  }
}
```

### Failover Strategies

**CLI Cluster Failover** (HAProxy):

```haproxy
# haproxy.cfg - Failover configuration
backend thread_workers
    balance leastconn
    option httpchk GET /health/ready

    # Primary workers (healthy)
    server worker1 10.0.1.10:8080 check inter 2s rise 2 fall 3
    server worker2 10.0.1.11:8080 check inter 2s rise 2 fall 3

    # Backup workers (only used if all primary fail)
    server backup1 10.0.2.10:8080 check inter 2s rise 2 fall 3 backup
    server backup2 10.0.2.11:8080 check inter 2s rise 2 fall 3 backup
```

**Edge Automatic Failover**:
- Cloudflare handles failover automatically
- Unhealthy workers removed from rotation
- No configuration needed (built-in)

### Database Failover

**Postgres Failover** (CLI):

```yaml
# patroni.yml - HA Postgres with automatic failover
scope: thread-db
namespace: /db/
name: postgres-1

restapi:
  listen: 0.0.0.0:8008
  connect_address: 10.0.1.10:8008

etcd:
  hosts: 10.0.1.20:2379,10.0.1.21:2379,10.0.1.22:2379

bootstrap:
  dcs:
    ttl: 30
    loop_wait: 10
    retry_timeout: 10
    maximum_lag_on_failover: 1048576

postgresql:
  listen: 0.0.0.0:5432
  connect_address: 10.0.1.10:5432
  data_dir: /var/lib/postgresql/data

  # Automatic failover
  use_pg_rewind: true
  remove_data_directory_on_rewind_failure: false
```

**D1 Failover** (Edge):
- Automatic multi-region replication
- Read replicas in all Cloudflare regions
- No manual failover configuration
- Eventual consistency (< 100 ms replication lag)

---

## Request Routing Strategies

### Routing by Content Type

```rust
// CLI - Route by analysis type
pub enum AnalysisType {
    QuickFingerprint,   // < 1 ms, high priority
    FullAnalysis,       // 100-500 ms, normal priority
    DeepAnalysis,       // > 1 second, low priority (background)
}

pub async fn route_request(request: AnalysisRequest) -> Result<AnalysisResponse> {
    match request.analysis_type {
        AnalysisType::QuickFingerprint => {
            // Use fast path (cache only)
            quick_fingerprint_handler(request).await
        }
        AnalysisType::FullAnalysis => {
            // Use normal processing
            full_analysis_handler(request).await
        }
        AnalysisType::DeepAnalysis => {
            // Enqueue for background processing
            enqueue_background_job(request).await
        }
    }
}
```

### Routing by Cache Affinity

**Consistent Hashing for Cache Locality**:

```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn route_to_worker(fingerprint: &Fingerprint, workers: &[WorkerNode]) -> &WorkerNode {
    let mut hasher = DefaultHasher::new();
    fingerprint.hash(&mut hasher);
    let hash = hasher.finish();

    let idx = (hash as usize) % workers.len();
    &workers[idx]
}
```

**Benefits**:
- Same fingerprint always routes to same worker (cache affinity)
- 99%+ cache hit rate on that worker
- Reduce cross-worker cache misses

### Routing by Geographic Proximity

**Edge Automatic Geo-Routing**:
```typescript
// worker.ts - Geographic routing (automatic)
export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    // Cloudflare automatically routes to nearest edge
    const cf = request.cf as IncomingRequestCfProperties;

    // Optional: Log routing decision
    console.log(`Request from ${cf.country} routed to ${cf.colo}`);

    // Process request at edge (low latency)
    return handleRequest(request, env);
  }
};
```

**CLI Manual Geo-Routing** (DNS-based):
```
# Route53 / CloudFlare DNS - Geolocation routing
users in us-east -> lb-us-east.thread.example.com (10.0.1.1)
users in eu-west -> lb-eu-west.thread.example.com (10.0.2.1)
users in ap-southeast -> lb-ap-southeast.thread.example.com (10.0.3.1)
```

---

## Load Balancing Monitoring

### Metrics to Track

**Load Distribution Metrics**:
- Requests per worker (should be balanced)
- CPU utilization per worker (should be similar)
- Queue depth per worker (should be low and balanced)
- Response time per worker (detect slow workers)

**Health Check Metrics**:
- Health check success rate (should be 100%)
- Failover events (should be rare)
- Worker availability (should be > 99%)

**Cache Affinity Metrics**:
- Cache hit rate per worker (should be > 90%)
- Cache affinity violations (should be < 1%)
- Cross-worker cache requests (should be minimal)

### Prometheus Queries for Load Balancing

**Request Distribution Balance**:
```promql
# Coefficient of variation (lower = better balance)
stddev(rate(http_requests_total[5m])) / avg(rate(http_requests_total[5m]))

# Alert if imbalance > 30%
(stddev(rate(http_requests_total[5m])) / avg(rate(http_requests_total[5m]))) > 0.3
```

**Worker Health**:
```promql
# Health check success rate
rate(health_check_success_total[5m]) / rate(health_check_total[5m])

# Alert if < 99%
(rate(health_check_success_total[5m]) / rate(health_check_total[5m])) < 0.99
```

**Failover Events**:
```promql
# Failover rate (should be near zero)
rate(worker_failover_total[5m])

# Alert on any failover
rate(worker_failover_total[5m]) > 0
```

### Grafana Dashboard for Load Balancing

**Panel 1: Load Distribution**
- Requests per worker (bar chart)
- CPU utilization per worker (heatmap)
- Queue depth per worker (time series)

**Panel 2: Health Status**
- Worker availability (gauge per worker)
- Health check success rate (time series)
- Failover events (stat)

**Panel 3: Cache Affinity**
- Cache hit rate per worker (bar chart)
- Affinity violations (time series)
- Cross-worker requests (stat)

---

## Best Practices

### 1. Use Least-Connections for Variable Workloads

**Antipattern**: Round-robin for long-running requests (leads to imbalance)

**Best Practice**: Least-connections balancing for analysis workloads

**Rationale**: Analysis times vary (10 ms - 10 seconds), least-connections prevents overload of single worker.

### 2. Implement Health Checks with Meaningful Tests

**Antipattern**: Health check always returns 200 OK

**Best Practice**: Test critical dependencies (database, cache)

**Example**:
```rust
async fn health_check() -> (StatusCode, Json<HealthStatus>) {
    let db_ok = test_database_connection().await;
    let cache_ok = test_cache_connection().await;

    if db_ok && cache_ok {
        (StatusCode::OK, Json(healthy_status()))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(unhealthy_status()))
    }
}
```

### 3. Use Consistent Hashing for Cache Affinity

**Antipattern**: Random routing (kills cache hit rate)

**Best Practice**: Route same fingerprint to same worker

**Impact**: 99%+ cache hit rate (vs 50% with random routing)

### 4. Monitor Load Balance Quality

**Antipattern**: Assume load balancer works (no validation)

**Best Practice**: Track request distribution and imbalance metrics

**Alert**: Trigger on > 30% imbalance for action

### 5. Plan for Failover Testing

**Antipattern**: Never test failover (breaks in production)

**Best Practice**: Regular chaos engineering and failover drills

**Example**: Kill random worker nodes monthly, verify automatic recovery

---

## Appendix: Load Balancing Decision Matrix

| Scenario | CLI Strategy | Edge Strategy |
|----------|--------------|---------------|
| **Single-Node** | Rayon thread pool (automatic) | Cloudflare auto-scaling (built-in) |
| **Multi-Node Cluster** | HAProxy least-connections | N/A (Edge is inherently distributed) |
| **Geographic Distribution** | DNS-based geo-routing | Cloudflare edge routing (automatic) |
| **Cache Affinity** | Consistent hashing | Durable Objects (consistent routing) |
| **Variable Request Times** | Least-connections balancing | Edge auto-scaling handles variability |
| **High Availability** | Active-active with failover (HAProxy/Nginx) | Cloudflare handles automatically |
| **Cost Optimization** | Scale down idle workers | Edge auto-scales down (pay per request) |

---

## Appendix: Example Configurations

### Complete HAProxy Configuration

```haproxy
# /etc/haproxy/haproxy.cfg - Production Thread Load Balancer

global
    log /dev/log local0
    log /dev/log local1 notice
    chroot /var/lib/haproxy
    stats socket /run/haproxy/admin.sock mode 660 level admin
    stats timeout 30s
    user haproxy
    group haproxy
    daemon

    # Performance tuning
    maxconn 20000
    nbproc 4
    cpu-map auto:1/1-4 0-3

defaults
    log global
    mode http
    option httplog
    option dontlognull
    option http-server-close
    option forwardfor except 127.0.0.0/8
    option redispatch
    retries 3
    timeout connect 5000ms
    timeout client 50000ms
    timeout server 50000ms
    timeout queue 60000ms

# Statistics endpoint
listen stats
    bind *:8404
    stats enable
    stats uri /stats
    stats refresh 30s
    stats admin if TRUE

# Frontend for client requests
frontend thread_frontend
    bind *:80
    bind *:443 ssl crt /etc/haproxy/certs/thread.pem

    # Redirect HTTP to HTTPS
    redirect scheme https code 301 if !{ ssl_fc }

    # ACLs for routing
    acl is_health_check path /health
    acl is_analysis path_beg /api/analyze
    acl is_cache path_beg /api/cache

    # Rate limiting (1000 req/s per IP)
    stick-table type ip size 100k expire 30s store http_req_rate(10s)
    http-request track-sc0 src
    http-request deny if { sc_http_req_rate(0) gt 1000 }

    default_backend thread_workers

# Backend worker pool
backend thread_workers
    balance leastconn
    option httpchk GET /health/ready
    http-check expect status 200

    # Primary workers (us-east-1)
    server worker1 10.0.1.10:8080 check inter 2s rise 2 fall 3 weight 100
    server worker2 10.0.1.11:8080 check inter 2s rise 2 fall 3 weight 100
    server worker3 10.0.1.12:8080 check inter 2s rise 2 fall 3 weight 100

    # Secondary workers (us-west-1, backup)
    server backup1 10.0.2.10:8080 check inter 2s rise 2 fall 3 weight 100 backup
    server backup2 10.0.2.11:8080 check inter 2s rise 2 fall 3 weight 100 backup

    # Connection pooling
    http-reuse safe
```

### Kubernetes Load Balancer Configuration

```yaml
# thread-loadbalancer.yaml - Complete K8s load balancing setup

---
# Service with load balancer
apiVersion: v1
kind: Service
metadata:
  name: thread-lb
  namespace: thread
  annotations:
    service.beta.kubernetes.io/aws-load-balancer-type: "nlb"
    service.beta.kubernetes.io/aws-load-balancer-cross-zone-load-balancing-enabled: "true"
spec:
  type: LoadBalancer
  selector:
    app: thread-worker
  ports:
    - name: http
      port: 80
      targetPort: 8080
      protocol: TCP
    - name: https
      port: 443
      targetPort: 8443
      protocol: TCP
  sessionAffinity: ClientIP
  sessionAffinityConfig:
    clientIP:
      timeoutSeconds: 3600

---
# HorizontalPodAutoscaler for auto-scaling
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: thread-worker-hpa
  namespace: thread
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: thread-worker
  minReplicas: 3
  maxReplicas: 20
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: 80
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
        - type: Percent
          value: 50
          periodSeconds: 60
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
        - type: Percent
          value: 100
          periodSeconds: 30

---
# PodDisruptionBudget for availability during updates
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: thread-worker-pdb
  namespace: thread
spec:
  minAvailable: 2
  selector:
    matchLabels:
      app: thread-worker
```

---

**Document Version**: 1.0.0
**Last Updated**: 2026-01-28
**Next Review**: 2026-02-28
**Owner**: Thread Operations Team
