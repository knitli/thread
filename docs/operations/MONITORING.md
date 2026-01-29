# Thread Flow Monitoring & Observability Guide

Comprehensive guide for monitoring Thread Flow in production environments with metrics, logging, dashboards, and alerting.

---

## Table of Contents

1. [Overview](#overview)
2. [Metrics Collection](#metrics-collection)
3. [Structured Logging](#structured-logging)
4. [Dashboard Setup](#dashboard-setup)
5. [Alerting Configuration](#alerting-configuration)
6. [SLIs and SLOs](#slis-and-slos)
7. [Incident Response](#incident-response)

---

## Overview

### Observability Stack

```
┌──────────────────────────────────────────┐
│         Thread Flow Application          │
└──────────────┬───────────────────────────┘
               │
       ┌───────┴────────┐
       │                │
       ▼                ▼
┌─────────────┐  ┌─────────────┐
│   Metrics   │  │   Logging   │
│ (Prometheus)│  │ (JSON/Text) │
└──────┬──────┘  └──────┬──────┘
       │                │
       │         ┌──────┴──────┐
       │         │             │
       ▼         ▼             ▼
┌─────────────┐  ┌─────────────┐
│   Grafana   │  │  DataDog    │
│ (Dashboard) │  │ (APM/Logs)  │
└──────┬──────┘  └──────┬──────┘
       │                │
       ▼                ▼
┌─────────────────────────┐
│    Alerting (PagerDuty) │
└─────────────────────────┘
```

### Key Metrics Tracked

| Category | Metrics | Target |
|----------|---------|--------|
| **Cache** | Hit rate, hits, misses | >90% hit rate |
| **Latency** | p50, p95, p99 query time | <10ms (CLI), <50ms (Edge) |
| **Performance** | Fingerprint, parse, extract times | <1µs, <200µs, <100µs |
| **Throughput** | Files/sec, symbols/sec | 2,500+ (CLI), 40+ (Edge) |
| **Errors** | Error rate, errors by type | <1% error rate |

---

## Metrics Collection

### Enable Metrics in Code

```rust
use thread_flow::monitoring::Metrics;

// Create metrics collector
let metrics = Metrics::new();

// Track cache operations
metrics.record_cache_hit();
metrics.record_cache_miss();

// Track latency (in milliseconds)
let start = Instant::now();
let result = query_database(&hash).await?;
metrics.record_query_latency(start.elapsed().as_millis() as u64);

// Track performance (nanoseconds/microseconds)
metrics.record_fingerprint_time(425);  // 425ns
metrics.record_parse_time(147);        // 147µs

// Track throughput
metrics.record_files_processed(100);
metrics.record_symbols_extracted(1500);

// Track errors
metrics.record_error("database_connection_failed");
```

### Prometheus Metrics Endpoint

**CLI Deployment**:

```rust
use thread_flow::monitoring::Metrics;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let metrics = Metrics::new();

    // Start metrics server on :9090
    let addr = SocketAddr::from(([0, 0, 0, 0], 9090));
    let listener = TcpListener::bind(addr).await?;

    tokio::spawn(async move {
        loop {
            if let Ok((mut socket, _)) = listener.accept().await {
                let metrics_data = metrics.export_prometheus();
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n{}",
                    metrics_data
                );
                let _ = socket.write_all(response.as_bytes()).await;
            }
        }
    });

    // Main application logic...
    Ok(())
}
```

**Edge Deployment** (Cloudflare Workers):

```javascript
// worker/index.js
import { Metrics } from './metrics';

export default {
  async fetch(request, env, ctx) {
    const metrics = new Metrics();

    // Handle metrics endpoint
    if (new URL(request.url).pathname === '/metrics') {
      const stats = await getMetricsFromD1(env.DB);
      return new Response(formatPrometheus(stats), {
        headers: { 'Content-Type': 'text/plain' }
      });
    }

    // Regular request handling with metrics
    const start = Date.now();
    try {
      const result = await analyzeCode(request, env);
      metrics.recordLatency(Date.now() - start);
      return new Response(JSON.stringify(result));
    } catch (error) {
      metrics.recordError(error.message);
      throw error;
    }
  }
};
```

### Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  # CLI deployment (local or server)
  - job_name: 'thread-flow-cli'
    static_configs:
      - targets: ['localhost:9090']
        labels:
          environment: 'production'
          deployment: 'cli'

  # Edge deployment (via Cloudflare Workers)
  - job_name: 'thread-flow-edge'
    static_configs:
      - targets: ['thread-flow-worker.your-account.workers.dev:443']
        labels:
          environment: 'production'
          deployment: 'edge'
    scheme: https
    metrics_path: '/metrics'
```

### Metric Types

**Counter Metrics** (always increasing):
```
thread_cache_hits_total
thread_cache_misses_total
thread_files_processed_total
thread_symbols_extracted_total
thread_errors_total{type="database_error"}
```

**Gauge Metrics** (point-in-time values):
```
thread_cache_hit_rate
thread_throughput_files_per_second
thread_error_rate
```

**Summary Metrics** (percentiles):
```
thread_query_latency_milliseconds{quantile="0.5"}
thread_query_latency_milliseconds{quantile="0.95"}
thread_query_latency_milliseconds{quantile="0.99"}
thread_fingerprint_time_nanoseconds{quantile="0.95"}
thread_parse_time_microseconds{quantile="0.95"}
```

---

## Structured Logging

### Initialize Logging

**CLI Application**:

```rust
use thread_flow::monitoring::logging::{init_cli_logging, LogConfig, LogLevel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simple initialization
    init_cli_logging()?;

    // Or custom configuration
    init_logging(LogConfig {
        level: LogLevel::Info,
        format: LogFormat::Text,
        timestamps: true,
        source_location: false,
        thread_ids: false,
    })?;

    // Application code...
    Ok(())
}
```

**Production/Edge**:

```rust
use thread_flow::monitoring::logging::init_production_logging;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // JSON logging with full context
    init_production_logging()?;

    // Application code...
    Ok(())
}
```

### Log Levels

```bash
# Set via environment variable
export RUST_LOG=thread_flow=debug

# Available levels (from most to least verbose)
export RUST_LOG=trace   # Very verbose (includes tracing)
export RUST_LOG=debug   # Verbose (development)
export RUST_LOG=info    # Normal (production default)
export RUST_LOG=warn    # Warnings only
export RUST_LOG=error   # Errors only
```

### Structured Logging Examples

```rust
use log::{info, warn, error};
use thread_flow::monitoring::logging::structured::LogContext;

// Simple logging
info!("Processing file: {}", file_path);
warn!("Cache miss for hash: {}", hash);
error!("Database connection failed: {}", error);

// Structured context logging
LogContext::new()
    .field("file_path", file_path)
    .field("file_size", file_size)
    .field("language", "rust")
    .info("File analysis started");

// Timed operations
timed_operation!("parse_file", file = file_path, {
    parse_rust_file(file_path)?;
});
// Automatically logs: "parse_file completed in 147µs"
```

### Log Output Formats

**Text Format** (development):
```
2025-01-28T12:34:56.789Z INFO  Processing file src/main.rs
2025-01-28T12:34:56.790Z DEBUG Cache lookup for hash abc123...
2025-01-28T12:34:56.792Z INFO  parse_file completed in 147µs
```

**JSON Format** (production):
```json
{"timestamp":"2025-01-28T12:34:56.789Z","level":"INFO","message":"Processing file src/main.rs","file_path":"src/main.rs"}
{"timestamp":"2025-01-28T12:34:56.790Z","level":"DEBUG","message":"Cache lookup","hash":"abc123..."}
{"timestamp":"2025-01-28T12:34:56.792Z","level":"INFO","message":"parse_file completed","duration_us":147}
```

**Compact Format** (CLI):
```
[INFO] Processing file src/main.rs
[DEBUG] Cache lookup abc123...
[INFO] parse_file: 147µs
```

### Log Aggregation

**Cloudflare Workers** (automatic):
```bash
# Real-time log streaming
wrangler tail

# Filter by log level
wrangler tail --status error

# JSON output for parsing
wrangler tail --format json | jq '.logs[] | select(.level == "ERROR")'
```

**Self-Hosted** (with Loki):
```yaml
# promtail.yml
server:
  http_listen_port: 9080

positions:
  filename: /tmp/positions.yaml

clients:
  - url: http://loki:3100/loki/api/v1/push

scrape_configs:
  - job_name: thread-flow
    static_configs:
      - targets:
          - localhost
        labels:
          job: thread-flow
          __path__: /var/log/thread-flow/*.log
```

---

## Dashboard Setup

### Grafana Dashboard

**Install Grafana**:

```bash
# Docker
docker run -d -p 3000:3000 \
  --name=grafana \
  -e "GF_SECURITY_ADMIN_PASSWORD=admin" \
  grafana/grafana

# Ubuntu/Debian
sudo apt-get install -y software-properties-common
sudo add-apt-repository "deb https://packages.grafana.com/oss/deb stable main"
sudo apt-get update
sudo apt-get install grafana

# Start Grafana
sudo systemctl start grafana-server
sudo systemctl enable grafana-server

# Access at http://localhost:3000 (admin/admin)
```

**Add Prometheus Data Source**:

1. Navigate to Configuration → Data Sources
2. Add Prometheus data source
3. URL: `http://localhost:9090`
4. Save & Test

**Import Thread Flow Dashboard**:

Create `thread-flow-dashboard.json`:

```json
{
  "dashboard": {
    "title": "Thread Flow Monitoring",
    "panels": [
      {
        "title": "Cache Hit Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "thread_cache_hit_rate"
          }
        ],
        "yaxes": [
          {
            "format": "percent",
            "max": 100,
            "min": 0
          }
        ],
        "alert": {
          "conditions": [
            {
              "evaluator": {
                "params": [90],
                "type": "lt"
              },
              "query": {
                "params": ["A", "5m", "now"]
              },
              "type": "query"
            }
          ],
          "name": "Low Cache Hit Rate"
        }
      },
      {
        "title": "Query Latency (p95)",
        "type": "graph",
        "targets": [
          {
            "expr": "thread_query_latency_milliseconds{quantile=\"0.95\"}"
          }
        ],
        "yaxes": [
          {
            "format": "ms"
          }
        ]
      },
      {
        "title": "Throughput (files/sec)",
        "type": "stat",
        "targets": [
          {
            "expr": "rate(thread_files_processed_total[5m])"
          }
        ]
      },
      {
        "title": "Error Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "thread_error_rate"
          }
        ],
        "alert": {
          "conditions": [
            {
              "evaluator": {
                "params": [1],
                "type": "gt"
              }
            }
          ],
          "name": "High Error Rate"
        }
      }
    ]
  }
}
```

Import via: Dashboards → Import → Upload JSON file

### DataDog Integration

**Install DataDog Agent**:

```bash
# Install DataDog agent
DD_AGENT_MAJOR_VERSION=7 DD_API_KEY=<your-api-key> DD_SITE="datadoghq.com" bash -c "$(curl -L https://s3.amazonaws.com/dd-agent/scripts/install_script.sh)"
```

**Configure OpenMetrics Integration**:

```yaml
# /etc/datadog-agent/conf.d/openmetrics.d/conf.yaml
instances:
  - prometheus_url: http://localhost:9090/metrics
    namespace: thread_flow
    metrics:
      - thread_cache_hit_rate
      - thread_query_latency_milliseconds
      - thread_files_processed_total
      - thread_error_rate
    tags:
      - environment:production
      - service:thread-flow
```

**Restart DataDog Agent**:

```bash
sudo systemctl restart datadog-agent
```

**View in DataDog**:
- Navigate to Metrics Explorer
- Search for `thread_flow.*`
- Create custom dashboards and monitors

### Cloudflare Analytics

For Edge deployments, Cloudflare provides built-in analytics:

1. Navigate to Workers → Your Worker → Analytics
2. View metrics:
   - **Requests**: Total requests per time period
   - **Errors**: Error rate and error types
   - **Duration**: p50, p75, p99, pmax
   - **CPU Time**: Average and p99 CPU usage

**Custom Analytics** (via Analytics Engine):

```javascript
// worker/index.js
export default {
  async fetch(request, env, ctx) {
    const start = Date.now();

    try {
      const result = await analyzeCode(request);

      // Log to Analytics Engine
      env.ANALYTICS.writeDataPoint({
        blobs: [request.url],
        doubles: [Date.now() - start],
        indexes: ['success']
      });

      return new Response(JSON.stringify(result));
    } catch (error) {
      env.ANALYTICS.writeDataPoint({
        blobs: [error.message],
        doubles: [Date.now() - start],
        indexes: ['error']
      });
      throw error;
    }
  }
};
```

---

## Alerting Configuration

### Prometheus Alertmanager

**Install Alertmanager**:

```bash
# Docker
docker run -d -p 9093:9093 \
  --name=alertmanager \
  -v /path/to/alertmanager.yml:/etc/alertmanager/alertmanager.yml \
  prom/alertmanager
```

**Configure Alerts** (`alertmanager.yml`):

```yaml
global:
  resolve_timeout: 5m
  slack_api_url: 'https://hooks.slack.com/services/YOUR/WEBHOOK/URL'

route:
  group_by: ['alertname']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 1h
  receiver: 'team-alerts'

receivers:
  - name: 'team-alerts'
    slack_configs:
      - channel: '#thread-flow-alerts'
        title: '{{ .GroupLabels.alertname }}'
        text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'
    pagerduty_configs:
      - service_key: 'YOUR_PAGERDUTY_KEY'
```

**Define Alert Rules** (`alerts.yml`):

```yaml
groups:
  - name: thread_flow_alerts
    interval: 30s
    rules:
      # Cache hit rate alert
      - alert: LowCacheHitRate
        expr: thread_cache_hit_rate < 90
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Low cache hit rate: {{ $value }}%"
          description: "Cache hit rate is {{ $value }}%, below SLO of 90%"

      # High latency alert (CLI)
      - alert: HighQueryLatencyCLI
        expr: thread_query_latency_milliseconds{quantile="0.95"} > 10
        for: 2m
        labels:
          severity: warning
          deployment: cli
        annotations:
          summary: "High query latency: {{ $value }}ms"
          description: "p95 query latency is {{ $value }}ms, above SLO of 10ms"

      # High latency alert (Edge)
      - alert: HighQueryLatencyEdge
        expr: thread_query_latency_milliseconds{quantile="0.95",deployment="edge"} > 50
        for: 2m
        labels:
          severity: warning
          deployment: edge
        annotations:
          summary: "High query latency: {{ $value }}ms"
          description: "p95 query latency is {{ $value }}ms, above SLO of 50ms"

      # High error rate alert
      - alert: HighErrorRate
        expr: thread_error_rate > 1
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "High error rate: {{ $value }}%"
          description: "Error rate is {{ $value }}%, above SLO of 1%"

      # Database connection failures
      - alert: DatabaseConnectionFailures
        expr: increase(thread_errors_total{type="database_connection_failed"}[5m]) > 5
        labels:
          severity: critical
        annotations:
          summary: "Multiple database connection failures"
          description: "{{ $value }} database connection failures in the last 5 minutes"
```

### PagerDuty Integration

**Create Integration**:

1. Go to PagerDuty → Services → Your Service
2. Add Integration → Prometheus
3. Copy Integration Key

**Configure in Alertmanager**:

```yaml
receivers:
  - name: 'critical-alerts'
    pagerduty_configs:
      - service_key: 'YOUR_PAGERDUTY_INTEGRATION_KEY'
        severity: '{{ .GroupLabels.severity }}'
        description: '{{ .GroupLabels.alertname }}: {{ .Annotations.description }}'

route:
  routes:
    - match:
        severity: critical
      receiver: 'critical-alerts'
```

### Slack Notifications

**Create Webhook**:

1. Go to Slack → Apps → Incoming Webhooks
2. Add to Workspace
3. Copy Webhook URL

**Configure in Alertmanager**:

```yaml
receivers:
  - name: 'slack-notifications'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/YOUR/WEBHOOK/URL'
        channel: '#thread-flow-alerts'
        title: '{{ .GroupLabels.alertname }}'
        text: |
          {{ range .Alerts }}
          *Status*: {{ .Status }}
          *Severity*: {{ .Labels.severity }}
          *Description*: {{ .Annotations.description }}
          {{ end }}
        color: '{{ if eq .Status "firing" }}danger{{ else }}good{{ end }}'
```

---

## SLIs and SLOs

### Service Level Indicators (SLIs)

| SLI | Measurement | Target |
|-----|-------------|--------|
| **Availability** | Successful requests / Total requests | 99.9% |
| **Latency (CLI)** | p95 query latency | <10ms |
| **Latency (Edge)** | p95 query latency | <50ms |
| **Cache Efficiency** | Cache hits / (Hits + Misses) | >90% |
| **Correctness** | Successful analyses / Total analyses | >99% |

### Service Level Objectives (SLOs)

**Availability SLO**: 99.9% uptime

```
Error Budget = (1 - 0.999) * 30 days = 43.2 minutes/month
```

**Latency SLO**: 95% of queries <10ms (CLI), <50ms (Edge)

```
Allowed violations: 5% of queries can exceed threshold
```

**Cache SLO**: 90% cache hit rate

```
Minimum: 90 hits per 100 lookups
```

### SLO Monitoring

**Check SLO Compliance**:

```rust
use thread_flow::monitoring::Metrics;

let metrics = Metrics::new();
let snapshot = metrics.snapshot();

match snapshot.meets_slo() {
    SLOStatus::Healthy => {
        println!("✅ All SLOs met");
    }
    SLOStatus::Violated(violations) => {
        for violation in violations {
            eprintln!("❌ SLO violation: {}", violation);
        }
    }
}
```

**Prometheus Queries for SLO**:

```promql
# Availability SLO (99.9%)
1 - (sum(rate(thread_errors_total[30d])) / sum(rate(thread_files_processed_total[30d])))

# Latency SLO (p95 <10ms for CLI)
histogram_quantile(0.95, thread_query_latency_milliseconds) < 10

# Cache SLO (>90% hit rate)
thread_cache_hit_rate > 90
```

---

## Incident Response

### Incident Severity Levels

| Level | Definition | Response Time | Example |
|-------|------------|---------------|---------|
| **SEV-1** | Service completely down | Immediate | Database unreachable, all requests failing |
| **SEV-2** | Major degradation | <15 minutes | Cache hit rate <50%, latency >100ms |
| **SEV-3** | Minor degradation | <1 hour | Cache hit rate 80-90%, intermittent errors |
| **SEV-4** | Monitoring only | <24 hours | Single error spike, brief latency increase |

### Incident Response Playbooks

**SEV-1: Service Down**

1. **Acknowledge**: Page on-call engineer
2. **Assess**: Check health endpoints, logs, metrics
3. **Mitigate**:
   - CLI: Restart service, check PostgreSQL
   - Edge: Check Cloudflare Workers status, D1 availability
4. **Communicate**: Post to status page
5. **Resolve**: Restore service
6. **Post-mortem**: Document incident, root cause, prevention

**SEV-2: High Latency**

1. **Check Metrics**: Query p95/p99 latency
2. **Investigate**:
   - Database slow queries?
   - Cache hit rate low?
   - Increased traffic?
3. **Mitigate**:
   - Scale database connections
   - Clear/warm cache
   - Add read replicas
4. **Monitor**: Watch for improvement
5. **Document**: Update runbook

**SEV-3: Low Cache Hit Rate**

1. **Check Logs**: Look for cache eviction messages
2. **Analyze**:
   - TTL too short?
   - Capacity too small?
   - Unusual file change patterns?
3. **Adjust**:
   - Increase cache capacity
   - Extend TTL
   - Verify fingerprinting working
4. **Validate**: Monitor hit rate recovery

### Debugging Commands

```bash
# Check metrics endpoint
curl http://localhost:9090/metrics

# Get current metrics snapshot
thread-flow metrics

# Enable trace logging
RUST_LOG=trace thread-flow analyze src/

# Check PostgreSQL connections
psql -U thread_user -d thread_cache -c "SELECT count(*) FROM pg_stat_activity;"

# Check D1 query performance
wrangler d1 execute thread-production --command="
  SELECT COUNT(*) as queries, AVG(duration_ms) as avg_ms
  FROM _cf_KV WHERE timestamp > datetime('now', '-1 hour');
"

# Tail Cloudflare Workers logs
wrangler tail --format json | jq '.diagnostics'
```

---

## Monitoring Checklist

### Initial Setup

- [ ] Metrics collection enabled in code
- [ ] Prometheus configured and scraping metrics
- [ ] Grafana dashboard imported and functional
- [ ] Structured logging initialized (JSON for production)
- [ ] Log aggregation configured (Loki, DataDog, or Cloudflare)
- [ ] Alerting rules defined and tested
- [ ] PagerDuty/Slack integration configured
- [ ] SLOs defined and baseline established

### Daily Operations

- [ ] Check dashboard for anomalies
- [ ] Verify cache hit rate >90%
- [ ] Confirm query latency within SLO
- [ ] Review error logs for patterns
- [ ] Check alert history

### Weekly Review

- [ ] Analyze SLO compliance over past week
- [ ] Review incident history and resolutions
- [ ] Identify performance trends
- [ ] Update alert thresholds if needed
- [ ] Capacity planning based on throughput metrics

---

**Monitoring Status**: Production-Ready
**SLO Compliance**: Automated tracking with alerts
**Incident Response**: Defined severity levels and playbooks
