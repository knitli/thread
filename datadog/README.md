<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.

SPDX-License-Identifier: AGPL-3.0-or-later
-->

# DataDog Monitoring Configuration

This directory contains DataDog dashboard and monitor configurations for Thread performance monitoring and constitutional compliance validation.

## Directory Structure

```
datadog/
├── dashboards/
│   └── thread-performance-monitoring.json   # Main performance dashboard
└── README.md                                # This file
```

## Dashboard Overview

### thread-performance-monitoring.json

**Purpose**: Monitor Thread's constitutional compliance and operational performance

**Key Features**:
- Constitutional compliance gauges (cache hit rate >90%, query latency <50ms)
- Performance metrics (fingerprint computation, query execution)
- Throughput monitoring (file processing, data throughput, batch operations)
- Cache operations tracking (hits, misses, evictions)
- Error rate monitoring

**Metrics Used**:
- `thread.cache_hit_rate_percent` - Cache hit rate percentage
- `thread.query_avg_duration_seconds` - Average query latency
- `thread.fingerprint_avg_duration_seconds` - Fingerprint computation time
- `thread.files_processed_total` - Total files processed
- `thread.bytes_processed_total` - Total bytes processed
- `thread.batches_processed_total` - Total batches processed
- `thread.cache_hits_total` - Total cache hits
- `thread.cache_misses_total` - Total cache misses
- `thread.cache_evictions_total` - Total cache evictions
- `thread.query_errors_total` - Total query errors
- `thread.query_error_rate_percent` - Query error rate percentage

## Deployment

See `docs/operations/DASHBOARD_DEPLOYMENT.md` for detailed deployment instructions.

### Quick Start

**Via UI**:
1. DataDog UI → Dashboards → New Dashboard → Import JSON
2. Paste contents of `dashboards/thread-performance-monitoring.json`
3. Save dashboard

**Via API**:
```bash
DD_API_KEY="your-api-key"
DD_APP_KEY="your-app-key"

curl -X POST "https://api.datadoghq.com/api/v1/dashboard" \
  -H "DD-API-KEY: ${DD_API_KEY}" \
  -H "DD-APPLICATION-KEY: ${DD_APP_KEY}" \
  -H "Content-Type: application/json" \
  -d @datadog/dashboards/thread-performance-monitoring.json
```

**Via Terraform**:
```hcl
resource "datadog_dashboard_json" "thread_performance" {
  dashboard = file("${path.module}/datadog/dashboards/thread-performance-monitoring.json")
}
```

## Metrics Collection

### DataDog Agent Configuration

Configure the DataDog Agent to scrape Thread's Prometheus metrics endpoint:

```yaml
# /etc/datadog-agent/datadog.yaml
prometheus_scrape:
  enabled: true
  configs:
    - configurations:
      - timeout: 5
        prometheus_url: "http://thread-service:8080/metrics"
        namespace: "thread"
        metrics:
          - "thread_*"
```

### Verify Metrics

```bash
# Check if DataDog is collecting Thread metrics
datadog-agent status | grep thread

# Query metrics via DataDog API
curl -X GET "https://api.datadoghq.com/api/v1/metrics?from=$(date -d '1 hour ago' +%s)&metric=thread.cache_hit_rate_percent" \
  -H "DD-API-KEY: ${DD_API_KEY}" \
  -H "DD-APPLICATION-KEY: ${DD_APP_KEY}"
```

## Alert Configuration

### Recommended Monitors

**Constitutional Compliance Alerts**:

1. **Cache Hit Rate Below 90%**:
   ```json
   {
     "name": "Thread Cache Hit Rate Below Constitutional Minimum",
     "type": "metric alert",
     "query": "avg(last_5m):avg:thread.cache_hit_rate_percent{*} < 90",
     "message": "Cache hit rate is below 90% constitutional requirement",
     "tags": ["team:thread", "priority:high", "constitutional-compliance"]
   }
   ```

2. **Query Latency Exceeds 50ms**:
   ```json
   {
     "name": "Thread Query Latency Exceeds Constitutional Maximum",
     "type": "metric alert",
     "query": "avg(last_5m):avg:thread.query_avg_duration_seconds{*} * 1000 > 50",
     "message": "Query latency exceeds 50ms constitutional requirement",
     "tags": ["team:thread", "priority:high", "constitutional-compliance"]
   }
   ```

**Operational Alerts**:

3. **High Error Rate**:
   ```json
   {
     "name": "Thread Query Error Rate Too High",
     "type": "metric alert",
     "query": "avg(last_5m):avg:thread.query_error_rate_percent{*} > 1",
     "message": "Query error rate exceeds 1%",
     "tags": ["team:thread", "priority:medium"]
   }
   ```

4. **Cache Eviction Storm**:
   ```json
   {
     "name": "Thread High Cache Eviction Rate",
     "type": "metric alert",
     "query": "avg(last_5m):per_second(avg:thread.cache_evictions_total{*}) > 100",
     "message": "Cache eviction rate indicates memory pressure",
     "tags": ["team:thread", "priority:low"]
   }
   ```

## Customization

### Adding Custom Widgets

1. Edit the dashboard JSON file
2. Add new widget definition to `widgets` array
3. Use Thread metrics (`thread.*`)
4. Redeploy dashboard

### Template Variables

The dashboard includes a template variable for environment filtering:

```json
"template_variables": [
  {
    "name": "environment",
    "default": "production",
    "prefix": "environment",
    "available_values": ["production", "staging", "development"]
  }
]
```

To use in queries: `thread.cache_hit_rate_percent{$environment}`

## Integration with Grafana

Thread also provides Grafana dashboards in `grafana/dashboards/`.

**Key Differences**:
- Grafana uses Prometheus metrics directly (underscores: `thread_*`)
- DataDog converts metric names (dots: `thread.*`)
- Both monitor the same underlying metrics from `PerformanceMetrics`

**Choose Based On**:
- **Grafana**: If you already have Prometheus infrastructure
- **DataDog**: If you use DataDog for other services
- **Both**: For redundancy and cross-validation

## Troubleshooting

### No Metrics Appearing

1. **Check Agent Status**:
   ```bash
   sudo datadog-agent status
   ```

2. **Verify Prometheus Integration**:
   ```bash
   sudo datadog-agent check prometheus -t
   ```

3. **Check Metrics Endpoint**:
   ```bash
   curl http://thread-service:8080/metrics | grep thread_cache_hit_rate_percent
   ```

### Incorrect Metric Values

1. **Verify Metric Collection**:
   ```bash
   # DataDog Metrics Explorer
   # Query: thread.cache_hit_rate_percent
   ```

2. **Check Conversion**:
   - Prometheus: `thread_cache_hit_rate_percent` (with underscore)
   - DataDog: `thread.cache_hit_rate_percent` (with dot)
   - DataDog Agent auto-converts underscores to dots

### Dashboard Import Errors

1. **Validate JSON**:
   ```bash
   jq '.' datadog/dashboards/thread-performance-monitoring.json
   ```

2. **Check Permissions**:
   - Ensure API and App keys have dashboard creation permissions
   - Verify user role includes dashboard management

## Related Documentation

- **Deployment Guide**: `docs/operations/DASHBOARD_DEPLOYMENT.md`
- **Performance Metrics**: `crates/flow/src/monitoring/performance.rs`
- **Constitutional Requirements**: `.specify/memory/constitution.md`
- **Monitoring Overview**: `docs/operations/MONITORING.md`

---

**Version**: 1.0.0
**Last Updated**: 2026-01-28
**Author**: Thread Operations Team (via Claude Sonnet 4.5)
