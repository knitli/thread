<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Post-Deployment Monitoring and Validation

**Version**: 1.0.0
**Last Updated**: 2026-01-28
**Status**: Production Ready

---

## Overview

This document defines comprehensive post-deployment monitoring strategies for Thread across CLI and Edge deployments. It covers real-time monitoring, SLO/SLI tracking, performance validation, incident detection, and continuous optimization.

### Purpose

- **Early Detection**: Identify issues before user impact
- **SLO Compliance**: Monitor and maintain service level objectives
- **Performance Validation**: Ensure deployments meet performance targets
- **Continuous Optimization**: Data-driven improvement opportunities

### Integration Points

- **Day 21 CI/CD**: Automated deployment pipelines trigger monitoring
- **Day 24 Capacity**: Monitoring validates capacity planning assumptions
- **Day 25 Deployment**: Validation gates confirm successful deployments

---

## Monitoring Architecture

### Monitoring Stack

#### CLI Deployment (Self-Hosted)
```
Application (Thread CLI)
    ↓ metrics
Prometheus (Time-series DB)
    ↓ visualization
Grafana (Dashboards)
    ↓ alerts
Alertmanager (Alert routing)
    ↓ notifications
PagerDuty/Slack
```

#### Edge Deployment (Cloudflare Workers)
```
Cloudflare Workers (Thread)
    ↓ logs & analytics
Cloudflare Analytics
    ↓ custom metrics
Workers Analytics Engine
    ↓ alerts
Cloudflare Notifications
    ↓ integration
PagerDuty/Slack
```

---

## SLO/SLI Monitoring

### Service Level Objectives (SLOs)

**Production SLOs** (99.9% uptime):

| Metric | Target | Measurement Window |
|--------|--------|-------------------|
| **Availability** | 99.9% | 30-day rolling |
| **Latency (p95)** | < 200ms | 5-minute window |
| **Latency (p99)** | < 500ms | 5-minute window |
| **Error Rate** | < 0.1% | 1-hour window |
| **Successful Deployments** | > 95% | Per deployment |

**Staging SLOs** (95% uptime):

| Metric | Target | Measurement Window |
|--------|--------|-------------------|
| **Availability** | 95% | 7-day rolling |
| **Latency (p95)** | < 500ms | 15-minute window |
| **Error Rate** | < 1% | 1-hour window |

### Service Level Indicators (SLIs)

**Availability SLI**:
```prometheus
# Availability SLI (successful requests / total requests)
sum(rate(http_requests_total{status!~"5.."}[5m]))
  /
sum(rate(http_requests_total[5m]))
```

**Latency SLI** (p95):
```prometheus
# P95 latency
histogram_quantile(0.95,
  sum(rate(http_request_duration_seconds_bucket[5m])) by (le)
)
```

**Error Rate SLI**:
```prometheus
# Error rate (5xx errors / total requests)
sum(rate(http_requests_total{status=~"5.."}[1h]))
  /
sum(rate(http_requests_total[1h]))
```

---

## Real-Time Monitoring

### Prometheus Configuration

**Prometheus config** (`prometheus.yml`):
```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    cluster: 'thread-production'
    region: 'us-east-1'

# Alert manager configuration
alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:9093']

# Scrape configurations
scrape_configs:
  # Thread CLI workers
  - job_name: 'thread-workers'
    static_configs:
      - targets:
          - 'worker-1:9090'
          - 'worker-2:9090'
          - 'worker-3:9090'
          - 'worker-4:9090'
          - 'worker-5:9090'
    relabel_configs:
      - source_labels: [__address__]
        target_label: instance

  # Database monitoring
  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres-exporter:9187']

  # Redis monitoring
  - job_name: 'redis'
    static_configs:
      - targets: ['redis-exporter:9121']

  # Node monitoring
  - job_name: 'node'
    static_configs:
      - targets:
          - 'worker-1:9100'
          - 'worker-2:9100'
          - 'worker-3:9100'
          - 'worker-4:9100'
          - 'worker-5:9100'
```

### Grafana Dashboards

**Main Dashboard Panels**:

1. **System Health Overview**
   - Uptime percentage (SLO compliance)
   - Request rate (requests/second)
   - Error rate (errors/second)
   - Active connections

2. **Latency Metrics**
   - P50 latency (median response time)
   - P95 latency (95th percentile)
   - P99 latency (99th percentile)
   - Max latency

3. **Resource Utilization**
   - CPU usage (per worker)
   - Memory usage (per worker)
   - Network I/O
   - Disk I/O

4. **Database Performance**
   - Query duration (p95, p99)
   - Connection pool usage
   - Active queries
   - Transaction rate

5. **Cache Performance**
   - Hit rate percentage
   - Miss rate
   - Eviction rate
   - Memory usage

**Dashboard JSON** (`grafana/thread-production.json`):
```json
{
  "dashboard": {
    "title": "Thread Production Monitoring",
    "panels": [
      {
        "title": "Request Rate",
        "targets": [
          {
            "expr": "sum(rate(http_requests_total[5m]))",
            "legendFormat": "Requests/sec"
          }
        ],
        "type": "graph"
      },
      {
        "title": "P95 Latency",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, sum(rate(http_request_duration_seconds_bucket[5m])) by (le))",
            "legendFormat": "P95 Latency"
          }
        ],
        "type": "graph",
        "yaxes": [{"format": "s"}]
      },
      {
        "title": "Error Rate",
        "targets": [
          {
            "expr": "sum(rate(http_requests_total{status=~\"5..\"}[5m])) / sum(rate(http_requests_total[5m]))",
            "legendFormat": "Error Rate"
          }
        ],
        "type": "graph",
        "yaxes": [{"format": "percentunit"}],
        "alert": {
          "conditions": [
            {
              "evaluator": {"params": [0.001], "type": "gt"},
              "query": {"params": ["A", "5m", "now"]},
              "type": "query"
            }
          ],
          "executionErrorState": "alerting",
          "frequency": "1m",
          "handler": 1,
          "name": "High Error Rate",
          "noDataState": "no_data"
        }
      }
    ]
  }
}
```

---

## Health Check Monitoring

### Application Health Checks

**Health Check Endpoint** (`src/health.rs`):
```rust
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct HealthStatus {
    status: String,
    version: String,
    uptime_seconds: u64,
    checks: HealthChecks,
}

#[derive(Serialize, Deserialize)]
pub struct HealthChecks {
    database: CheckStatus,
    cache: CheckStatus,
    storage: CheckStatus,
}

#[derive(Serialize, Deserialize)]
pub struct CheckStatus {
    healthy: bool,
    latency_ms: Option<f64>,
    message: Option<String>,
}

pub async fn health_check(
    db_pool: web::Data<DbPool>,
    cache: web::Data<Cache>,
) -> impl Responder {
    let start_time = std::time::Instant::now();

    // Check database connectivity
    let db_check = check_database(&db_pool).await;

    // Check cache connectivity
    let cache_check = check_cache(&cache).await;

    // Check storage (if applicable)
    let storage_check = check_storage().await;

    let all_healthy = db_check.healthy && cache_check.healthy && storage_check.healthy;

    let status = HealthStatus {
        status: if all_healthy { "healthy".to_string() } else { "unhealthy".to_string() },
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: start_time.elapsed().as_secs(),
        checks: HealthChecks {
            database: db_check,
            cache: cache_check,
            storage: storage_check,
        },
    };

    if all_healthy {
        HttpResponse::Ok().json(status)
    } else {
        HttpResponse::ServiceUnavailable().json(status)
    }
}

async fn check_database(pool: &DbPool) -> CheckStatus {
    let start = std::time::Instant::now();
    match sqlx::query("SELECT 1").fetch_one(pool).await {
        Ok(_) => CheckStatus {
            healthy: true,
            latency_ms: Some(start.elapsed().as_secs_f64() * 1000.0),
            message: None,
        },
        Err(e) => CheckStatus {
            healthy: false,
            latency_ms: None,
            message: Some(format!("Database error: {}", e)),
        },
    }
}

async fn check_cache(cache: &Cache) -> CheckStatus {
    let start = std::time::Instant::now();
    match cache.ping().await {
        Ok(_) => CheckStatus {
            healthy: true,
            latency_ms: Some(start.elapsed().as_secs_f64() * 1000.0),
            message: None,
        },
        Err(e) => CheckStatus {
            healthy: false,
            latency_ms: None,
            message: Some(format!("Cache error: {}", e)),
        },
    }
}

async fn check_storage() -> CheckStatus {
    // Check storage connectivity (filesystem, S3, etc.)
    CheckStatus {
        healthy: true,
        latency_ms: Some(1.0),
        message: None,
    }
}
```

### Kubernetes Health Probes

**Deployment with Health Probes**:
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: thread-worker
  namespace: production
spec:
  replicas: 5
  template:
    spec:
      containers:
      - name: thread
        image: thread:latest
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 9090
          name: metrics

        # Readiness probe (is the app ready to serve traffic?)
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
          timeoutSeconds: 3
          successThreshold: 1
          failureThreshold: 3

        # Liveness probe (is the app alive?)
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          successThreshold: 1
          failureThreshold: 3

        # Startup probe (has the app started successfully?)
        startupProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 0
          periodSeconds: 5
          timeoutSeconds: 3
          successThreshold: 1
          failureThreshold: 30  # Allow up to 150 seconds for startup
```

---

## Performance Metrics

### Application Metrics

**Metrics Instrumentation** (`src/metrics.rs`):
```rust
use prometheus::{
    Counter, Histogram, HistogramOpts, IntCounter, IntGauge, Opts, Registry,
};
use lazy_static::lazy_static;

lazy_static! {
    // Request counters
    pub static ref HTTP_REQUESTS_TOTAL: Counter = Counter::new(
        "http_requests_total",
        "Total HTTP requests"
    ).unwrap();

    pub static ref HTTP_REQUESTS_SUCCESS: Counter = Counter::new(
        "http_requests_success_total",
        "Successful HTTP requests"
    ).unwrap();

    pub static ref HTTP_REQUESTS_ERRORS: Counter = Counter::new(
        "http_requests_error_total",
        "Failed HTTP requests"
    ).unwrap();

    // Latency histogram
    pub static ref HTTP_REQUEST_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "http_request_duration_seconds",
            "HTTP request latency in seconds"
        )
        .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0])
    ).unwrap();

    // Active connections
    pub static ref ACTIVE_CONNECTIONS: IntGauge = IntGauge::new(
        "active_connections",
        "Number of active connections"
    ).unwrap();

    // Database metrics
    pub static ref DB_QUERY_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "db_query_duration_seconds",
            "Database query latency"
        )
        .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5])
    ).unwrap();

    pub static ref DB_CONNECTIONS_ACTIVE: IntGauge = IntGauge::new(
        "db_connections_active",
        "Active database connections"
    ).unwrap();

    // Cache metrics
    pub static ref CACHE_HITS: IntCounter = IntCounter::new(
        "cache_hits_total",
        "Total cache hits"
    ).unwrap();

    pub static ref CACHE_MISSES: IntCounter = IntCounter::new(
        "cache_misses_total",
        "Total cache misses"
    ).unwrap();
}

pub fn register_metrics(registry: &Registry) -> Result<(), prometheus::Error> {
    registry.register(Box::new(HTTP_REQUESTS_TOTAL.clone()))?;
    registry.register(Box::new(HTTP_REQUESTS_SUCCESS.clone()))?;
    registry.register(Box::new(HTTP_REQUESTS_ERRORS.clone()))?;
    registry.register(Box::new(HTTP_REQUEST_DURATION.clone()))?;
    registry.register(Box::new(ACTIVE_CONNECTIONS.clone()))?;
    registry.register(Box::new(DB_QUERY_DURATION.clone()))?;
    registry.register(Box::new(DB_CONNECTIONS_ACTIVE.clone()))?;
    registry.register(Box::new(CACHE_HITS.clone()))?;
    registry.register(Box::new(CACHE_MISSES.clone()))?;
    Ok(())
}
```

**Metrics Middleware** (Actix-Web):
```rust
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error, HttpMessage};
use actix_web::middleware::{Middleware, Response};
use std::time::Instant;

pub struct MetricsMiddleware;

impl<S, B> Middleware<S> for MetricsMiddleware
where
    S: 'static,
{
    fn handle(&self, req: ServiceRequest, srv: &mut S) -> Response {
        let start_time = Instant::now();

        // Increment active connections
        crate::metrics::ACTIVE_CONNECTIONS.inc();

        // Process request
        let res = srv.call(req);

        // Record metrics
        let duration = start_time.elapsed();
        crate::metrics::HTTP_REQUEST_DURATION.observe(duration.as_secs_f64());
        crate::metrics::HTTP_REQUESTS_TOTAL.inc();

        if res.status().is_success() {
            crate::metrics::HTTP_REQUESTS_SUCCESS.inc();
        } else if res.status().is_server_error() {
            crate::metrics::HTTP_REQUESTS_ERRORS.inc();
        }

        // Decrement active connections
        crate::metrics::ACTIVE_CONNECTIONS.dec();

        res
    }
}
```

---

## Alert Configuration

### Prometheus Alert Rules

**Alert rules** (`prometheus/alerts.yml`):
```yaml
groups:
  - name: thread_production_alerts
    interval: 30s
    rules:
      # High error rate alert
      - alert: HighErrorRate
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[5m]))
            /
          sum(rate(http_requests_total[5m])) > 0.01
        for: 5m
        labels:
          severity: critical
          team: thread
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value | humanizePercentage }} (threshold: 1%)"
          runbook_url: "https://docs.thread.io/runbooks/high-error-rate"

      # High latency alert (P95)
      - alert: HighLatencyP95
        expr: |
          histogram_quantile(0.95,
            sum(rate(http_request_duration_seconds_bucket[5m])) by (le)
          ) > 0.2
        for: 5m
        labels:
          severity: warning
          team: thread
        annotations:
          summary: "High P95 latency detected"
          description: "P95 latency is {{ $value }}s (threshold: 200ms)"
          runbook_url: "https://docs.thread.io/runbooks/high-latency"

      # High latency alert (P99)
      - alert: HighLatencyP99
        expr: |
          histogram_quantile(0.99,
            sum(rate(http_request_duration_seconds_bucket[5m])) by (le)
          ) > 0.5
        for: 5m
        labels:
          severity: critical
          team: thread
        annotations:
          summary: "High P99 latency detected"
          description: "P99 latency is {{ $value }}s (threshold: 500ms)"
          runbook_url: "https://docs.thread.io/runbooks/high-latency"

      # Service down alert
      - alert: ServiceDown
        expr: up{job="thread-workers"} == 0
        for: 1m
        labels:
          severity: critical
          team: thread
        annotations:
          summary: "Thread worker is down"
          description: "Worker {{ $labels.instance }} has been down for more than 1 minute"
          runbook_url: "https://docs.thread.io/runbooks/service-down"

      # High CPU usage
      - alert: HighCPUUsage
        expr: |
          100 - (avg by (instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100) > 80
        for: 10m
        labels:
          severity: warning
          team: thread
        annotations:
          summary: "High CPU usage"
          description: "CPU usage on {{ $labels.instance }} is {{ $value }}% (threshold: 80%)"
          runbook_url: "https://docs.thread.io/runbooks/high-cpu"

      # High memory usage
      - alert: HighMemoryUsage
        expr: |
          (node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes)
            / node_memory_MemTotal_bytes * 100 > 85
        for: 10m
        labels:
          severity: warning
          team: thread
        annotations:
          summary: "High memory usage"
          description: "Memory usage on {{ $labels.instance }} is {{ $value }}% (threshold: 85%)"
          runbook_url: "https://docs.thread.io/runbooks/high-memory"

      # Database connection pool exhaustion
      - alert: DatabaseConnectionPoolExhausted
        expr: db_connections_active / db_connections_max > 0.9
        for: 5m
        labels:
          severity: critical
          team: thread
        annotations:
          summary: "Database connection pool near exhaustion"
          description: "Connection pool usage is {{ $value | humanizePercentage }} (threshold: 90%)"
          runbook_url: "https://docs.thread.io/runbooks/db-pool-exhausted"

      # Cache hit rate low
      - alert: LowCacheHitRate
        expr: |
          sum(rate(cache_hits_total[5m]))
            /
          (sum(rate(cache_hits_total[5m])) + sum(rate(cache_misses_total[5m]))) < 0.7
        for: 15m
        labels:
          severity: warning
          team: thread
        annotations:
          summary: "Low cache hit rate"
          description: "Cache hit rate is {{ $value | humanizePercentage }} (threshold: 70%)"
          runbook_url: "https://docs.thread.io/runbooks/low-cache-hit-rate"

      # SLO violation (availability)
      - alert: SLOAvailabilityViolation
        expr: |
          sum(rate(http_requests_total{status!~"5.."}[30d]))
            /
          sum(rate(http_requests_total[30d])) < 0.999
        for: 1h
        labels:
          severity: critical
          team: thread
        annotations:
          summary: "SLO availability violation"
          description: "30-day availability is {{ $value | humanizePercentage }} (SLO: 99.9%)"
          runbook_url: "https://docs.thread.io/runbooks/slo-violation"
```

### Alertmanager Configuration

**Alertmanager config** (`alertmanager.yml`):
```yaml
global:
  resolve_timeout: 5m
  slack_api_url: '${SLACK_WEBHOOK_URL}'
  pagerduty_url: 'https://events.pagerduty.com/v2/enqueue'

# Route tree
route:
  receiver: 'default'
  group_by: ['alertname', 'cluster', 'service']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 12h

  routes:
    # Critical alerts → PagerDuty + Slack
    - match:
        severity: critical
      receiver: pagerduty-critical
      continue: true

    - match:
        severity: critical
      receiver: slack-critical

    # Warning alerts → Slack only
    - match:
        severity: warning
      receiver: slack-warnings

# Receivers
receivers:
  - name: 'default'
    slack_configs:
      - channel: '#alerts'
        title: 'Thread Alert'
        text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'

  - name: 'pagerduty-critical'
    pagerduty_configs:
      - service_key: '${PAGERDUTY_SERVICE_KEY}'
        description: '{{ .CommonAnnotations.summary }}'
        details:
          firing: '{{ .Alerts.Firing | len }}'
          resolved: '{{ .Alerts.Resolved | len }}'

  - name: 'slack-critical'
    slack_configs:
      - channel: '#incidents'
        title: '🚨 CRITICAL: {{ .CommonAnnotations.summary }}'
        text: |
          {{ range .Alerts }}
          *Alert:* {{ .Labels.alertname }}
          *Description:* {{ .Annotations.description }}
          *Runbook:* {{ .Annotations.runbook_url }}
          {{ end }}
        color: danger

  - name: 'slack-warnings'
    slack_configs:
      - channel: '#alerts'
        title: '⚠️  WARNING: {{ .CommonAnnotations.summary }}'
        text: |
          {{ range .Alerts }}
          *Alert:* {{ .Labels.alertname }}
          *Description:* {{ .Annotations.description }}
          {{ end }}
        color: warning

# Inhibition rules (suppress alerts based on other alerts)
inhibit_rules:
  # If service is down, don't alert on latency/errors
  - source_match:
      alertname: 'ServiceDown'
    target_match_re:
      alertname: 'HighLatency.*|HighErrorRate'
    equal: ['instance']
```

---

## Cloudflare Workers Analytics

### Workers Analytics Engine

**Analytics Bindings** (`wrangler.toml`):
```toml
name = "thread-worker"
main = "src/index.js"
compatibility_date = "2024-01-01"

[[analytics_engine_datasets]]
binding = "ANALYTICS"
```

**Analytics Instrumentation** (`src/analytics.js`):
```javascript
export default {
  async fetch(request, env, ctx) {
    const startTime = Date.now();

    try {
      // Process request
      const response = await handleRequest(request, env);

      // Record analytics
      const duration = Date.now() - startTime;

      ctx.waitUntil(
        env.ANALYTICS.writeDataPoint({
          indexes: [request.cf.colo],
          blobs: [
            request.url,
            request.method,
            response.status.toString(),
          ],
          doubles: [duration],
        })
      );

      return response;
    } catch (error) {
      // Record error
      const duration = Date.now() - startTime;

      ctx.waitUntil(
        env.ANALYTICS.writeDataPoint({
          indexes: [request.cf.colo, 'error'],
          blobs: [
            request.url,
            request.method,
            '500',
            error.message,
          ],
          doubles: [duration],
        })
      );

      return new Response('Internal Server Error', { status: 500 });
    }
  },
};
```

**Query Analytics** (via GraphQL):
```graphql
query {
  viewer {
    accounts(filter: {accountTag: $accountId}) {
      workersAnalyticsEngine(filter: {
        dataset: "thread-worker"
        datetime_geq: "2024-01-01T00:00:00Z"
        datetime_lt: "2024-01-02T00:00:00Z"
      }) {
        sum {
          double1  # Total request duration
        }
        count
        avg {
          double1  # Average request duration
        }
        quantiles {
          double1P50: double1Quantile(quantile: 0.5)
          double1P95: double1Quantile(quantile: 0.95)
          double1P99: double1Quantile(quantile: 0.99)
        }
      }
    }
  }
}
```

---

## Continuous Validation

### Synthetic Monitoring

**Synthetic Transaction Script** (`scripts/synthetic-monitoring.sh`):
```bash
#!/bin/bash
# Continuous synthetic transaction monitoring

set -e

ENDPOINT="${1:-https://api.thread.io}"
SLACK_WEBHOOK="${SLACK_WEBHOOK_URL}"
INTERVAL="${2:-60}"  # seconds

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"
}

alert() {
    local message="$1"
    log "ALERT: $message"

    if [[ -n "$SLACK_WEBHOOK" ]]; then
        curl -X POST "$SLACK_WEBHOOK" \
            -H 'Content-Type: application/json' \
            -d "{\"text\":\"🚨 Synthetic Monitor Alert: $message\"}"
    fi
}

# Test 1: Health check
test_health_check() {
    local start_time=$(date +%s%N)
    local response=$(curl -s -o /dev/null -w "%{http_code}" "$ENDPOINT/health")
    local end_time=$(date +%s%N)
    local duration=$(( (end_time - start_time) / 1000000 ))

    if [[ "$response" != "200" ]]; then
        alert "Health check failed (status: $response)"
        return 1
    fi

    if [[ "$duration" -gt 1000 ]]; then
        alert "Health check slow (${duration}ms > 1000ms)"
    fi

    log "Health check: OK (${duration}ms)"
    return 0
}

# Test 2: API query
test_api_query() {
    local start_time=$(date +%s%N)
    local response=$(curl -s -w "\n%{http_code}" "$ENDPOINT/api/query" \
        -H "Content-Type: application/json" \
        -d '{"pattern":"function $NAME() {}"}')

    local end_time=$(date +%s%N)
    local duration=$(( (end_time - start_time) / 1000000 ))

    local status=$(echo "$response" | tail -n1)
    local body=$(echo "$response" | head -n-1)

    if [[ "$status" != "200" ]]; then
        alert "API query failed (status: $status)"
        return 1
    fi

    if [[ "$duration" -gt 500 ]]; then
        alert "API query slow (${duration}ms > 500ms)"
    fi

    log "API query: OK (${duration}ms)"
    return 0
}

# Test 3: Database connectivity
test_database() {
    local start_time=$(date +%s%N)
    local response=$(curl -s -w "\n%{http_code}" "$ENDPOINT/health/database")
    local end_time=$(date +%s%N)
    local duration=$(( (end_time - start_time) / 1000000 ))

    local status=$(echo "$response" | tail -n1)

    if [[ "$status" != "200" ]]; then
        alert "Database connectivity check failed (status: $status)"
        return 1
    fi

    if [[ "$duration" -gt 100 ]]; then
        alert "Database check slow (${duration}ms > 100ms)"
    fi

    log "Database check: OK (${duration}ms)"
    return 0
}

# Main monitoring loop
main() {
    log "Starting synthetic monitoring for $ENDPOINT"
    log "Interval: ${INTERVAL}s"

    while true; do
        test_health_check || true
        sleep 5

        test_api_query || true
        sleep 5

        test_database || true

        sleep "$INTERVAL"
    done
}

main
```

**Run as systemd service** (`/etc/systemd/system/thread-synthetic-monitor.service`):
```ini
[Unit]
Description=Thread Synthetic Monitoring
After=network.target

[Service]
Type=simple
User=thread
ExecStart=/opt/thread/scripts/synthetic-monitoring.sh https://api.thread.io 60
Restart=always
RestartSec=10
Environment="SLACK_WEBHOOK_URL=https://hooks.slack.com/services/..."

[Install]
WantedBy=multi-user.target
```

---

## Log Aggregation

### Centralized Logging (ELK Stack)

**Filebeat Configuration** (`filebeat.yml`):
```yaml
filebeat.inputs:
  - type: log
    enabled: true
    paths:
      - /var/log/thread/*.log
    fields:
      service: thread
      environment: production
    fields_under_root: true

    # JSON parsing
    json.keys_under_root: true
    json.add_error_key: true

    # Multiline log handling
    multiline.pattern: '^[0-9]{4}-[0-9]{2}-[0-9]{2}'
    multiline.negate: true
    multiline.match: after

output.elasticsearch:
  hosts: ["elasticsearch:9200"]
  index: "thread-logs-%{+yyyy.MM.dd}"

processors:
  - add_host_metadata: ~
  - add_cloud_metadata: ~
  - add_docker_metadata: ~
```

**Logstash Pipeline** (`logstash/thread.conf`):
```
input {
  beats {
    port => 5044
  }
}

filter {
  # Parse JSON logs
  json {
    source => "message"
  }

  # Extract log level
  mutate {
    rename => { "level" => "log_level" }
  }

  # Parse timestamp
  date {
    match => [ "timestamp", "ISO8601" ]
    target => "@timestamp"
  }

  # Add geo IP data for requests
  geoip {
    source => "client_ip"
    target => "geoip"
  }
}

output {
  elasticsearch {
    hosts => ["elasticsearch:9200"]
    index => "thread-logs-%{+YYYY.MM.dd}"
  }
}
```

---

## Best Practices

### 1. Monitoring Coverage

- **Monitor All Layers**: Application, database, cache, infrastructure
- **End-to-End Validation**: Synthetic transactions covering user journeys
- **Business Metrics**: Not just technical metrics, track business KPIs

### 2. Alert Fatigue Prevention

- **Meaningful Alerts**: Only alert on actionable issues
- **Proper Thresholds**: Tune thresholds to reduce false positives
- **Alert Grouping**: Group related alerts to reduce noise
- **Escalation Policies**: Clear escalation paths for different severity levels

### 3. SLO-Driven Monitoring

- **Define Clear SLOs**: Measurable service level objectives
- **Error Budget**: Track remaining error budget
- **Prioritize by Impact**: Focus on customer-impacting metrics

### 4. Observability

- **Three Pillars**: Metrics (Prometheus), Logs (ELK), Traces (Jaeger)
- **Correlation**: Link metrics, logs, and traces for faster debugging
- **Context Preservation**: Include request IDs, user IDs, trace IDs

---

**Document Version**: 1.0.0
**Last Updated**: 2026-01-28
**Next Review**: 2026-02-28
**Owner**: Thread Operations Team
