<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Dashboard Deployment Guide

**Purpose**: Instructions for deploying Thread performance dashboards to Grafana and DataDog

**Constitutional Compliance**: These dashboards monitor the constitutional requirements:
- Cache hit rate >90% (Thread Constitution v2.0.0, Principle VI)
- D1 p95 latency <50ms (Thread Constitution v2.0.0, Principle VI)

---

## Prerequisites

### For Grafana

1. Grafana 10.0+ installed and running
2. Prometheus data source configured
3. Thread metrics endpoint accessible (`/metrics`)
4. Appropriate permissions to create dashboards

### For DataDog

1. DataDog account with dashboard creation permissions
2. DataDog Agent installed and configured
3. Prometheus metrics integration enabled
4. Thread metrics being scraped by DataDog Agent

---

## Grafana Dashboard Deployment

### Dashboard Files

- **thread-performance-monitoring.json**: Constitutional compliance and performance metrics
- **capacity-monitoring.json**: Capacity planning and scaling indicators

### Import via UI

1. **Navigate to Dashboards**:
   ```
   Grafana UI → Dashboards → Import
   ```

2. **Upload JSON**:
   - Click "Upload JSON file"
   - Select `grafana/dashboards/thread-performance-monitoring.json`
   - OR paste JSON content directly

3. **Configure Data Source**:
   - Select your Prometheus data source from dropdown
   - Ensure the data source UID matches `${DS_PROMETHEUS}`

4. **Complete Import**:
   - Click "Import"
   - Dashboard will be created with UID `thread-performance`

### Import via API

```bash
# Set variables
GRAFANA_URL="http://localhost:3000"
GRAFANA_API_KEY="your-api-key"
DASHBOARD_FILE="grafana/dashboards/thread-performance-monitoring.json"

# Import dashboard
curl -X POST "${GRAFANA_URL}/api/dashboards/db" \
  -H "Authorization: Bearer ${GRAFANA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d @"${DASHBOARD_FILE}"
```

### Import via Terraform

```hcl
# grafana_dashboards.tf
resource "grafana_dashboard" "thread_performance" {
  config_json = file("${path.module}/../../grafana/dashboards/thread-performance-monitoring.json")

  overwrite = true

  message = "Updated Thread Performance Dashboard"
}

resource "grafana_dashboard" "thread_capacity" {
  config_json = file("${path.module}/../../grafana/dashboards/capacity-monitoring.json")

  overwrite = true

  message = "Updated Thread Capacity Monitoring Dashboard"
}
```

### Configure Prometheus Data Source

If Prometheus data source doesn't exist yet:

```bash
# Create Prometheus data source
curl -X POST "${GRAFANA_URL}/api/datasources" \
  -H "Authorization: Bearer ${GRAFANA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Prometheus",
    "type": "prometheus",
    "url": "http://prometheus:9090",
    "access": "proxy",
    "isDefault": true
  }'
```

---

## DataDog Dashboard Deployment

### Dashboard File

- **thread-performance-monitoring.json**: DataDog-compatible dashboard configuration

### Import via UI

1. **Navigate to Dashboards**:
   ```
   DataDog UI → Dashboards → Dashboard List → New Dashboard
   ```

2. **Import JSON**:
   - Click the gear icon (settings) in top right
   - Select "Import dashboard JSON"
   - Paste contents of `datadog/dashboards/thread-performance-monitoring.json`
   - Click "Save"

3. **Verify Metrics**:
   - Ensure Thread metrics are appearing (check Metrics Explorer)
   - Verify template variable `$environment` is populated
   - Confirm widgets are displaying data

### Import via API

```bash
# Set variables
DD_API_KEY="your-api-key"
DD_APP_KEY="your-app-key"
DASHBOARD_FILE="datadog/dashboards/thread-performance-monitoring.json"

# Import dashboard
curl -X POST "https://api.datadoghq.com/api/v1/dashboard" \
  -H "DD-API-KEY: ${DD_API_KEY}" \
  -H "DD-APPLICATION-KEY: ${DD_APP_KEY}" \
  -H "Content-Type: application/json" \
  -d @"${DASHBOARD_FILE}"
```

### Import via Terraform

```hcl
# datadog_dashboards.tf
resource "datadog_dashboard_json" "thread_performance" {
  dashboard = file("${path.module}/../../datadog/dashboards/thread-performance-monitoring.json")
}
```

### Configure Prometheus Integration

Ensure DataDog Agent is configured to scrape Thread metrics:

```yaml
# datadog.yaml
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

---

## Metrics Endpoint Configuration

### Thread Metrics Export

The Thread service must expose Prometheus metrics at `/metrics`:

```rust
// In your Thread service main.rs or lib.rs
use thread_flow::monitoring::performance::PerformanceMetrics;

// Create metrics instance
let metrics = PerformanceMetrics::new();

// Export endpoint (example with axum)
async fn metrics_handler(
    State(metrics): State<PerformanceMetrics>,
) -> String {
    metrics.export_prometheus()
}

// Add route
let app = Router::new()
    .route("/metrics", get(metrics_handler))
    .with_state(metrics);
```

### Verify Metrics Export

```bash
# Test metrics endpoint
curl http://localhost:8080/metrics

# Expected output:
# HELP thread_cache_hit_rate_percent Cache hit rate percentage
# TYPE thread_cache_hit_rate_percent gauge
# thread_cache_hit_rate_percent 95.5
# ...
```

---

## Dashboard Features

### Constitutional Compliance Section

**Cache Hit Rate Gauge** (Panel 1):
- **Metric**: `thread_cache_hit_rate_percent`
- **Target**: >90% (green zone)
- **Warning**: 80-90% (yellow zone)
- **Critical**: <80% (red zone)

**Query Latency Gauge** (Panel 2):
- **Metric**: `thread_query_avg_duration_seconds * 1000` (converted to ms)
- **Target**: <50ms (green zone)
- **Warning**: 40-50ms (yellow zone)
- **Critical**: >50ms (red zone)

**Cache Hit Rate Trend** (Panel 3):
- Time series showing cache hit percentage over time
- Constitutional minimum threshold line at 90%

### Performance Metrics Section

**Fingerprint Computation** (Panel 4):
- Average Blake3 fingerprint computation time
- Rate of fingerprint operations

**Query Execution** (Panel 5):
- Average query execution time
- Query rate over time
- Constitutional maximum threshold line at 50ms

### Throughput & Operations Section

**File Processing Rate** (Panel 6):
- Files processed per second
- Indicates system throughput

**Data Throughput** (Panel 7):
- Bytes processed per second (in MB/s)
- Data pipeline performance

**Batch Processing Rate** (Panel 8):
- Batches processed per second
- Batch operation efficiency

### Cache Operations Section

**Cache Hit/Miss Rate** (Panel 9):
- Stacked area chart showing hits vs misses
- Visual representation of cache effectiveness

**Cache Eviction Rate** (Panel 10):
- LRU eviction rate
- Indicates cache pressure

### Error Tracking Section

**Query Error Rate** (Panel 11):
- Current error rate percentage
- Target: <1% error rate

**Query Error Rate Over Time** (Panel 12):
- Time series of error rate
- Helps identify error spikes

---

## Alert Configuration

### Grafana Alerts

The dashboards include built-in alert configurations. To enable:

1. **Navigate to Alert Rules**:
   ```
   Grafana UI → Alerting → Alert Rules
   ```

2. **Configure Notification Channel**:
   - Create notification channel (Slack, PagerDuty, email, etc.)
   - Link to alert rules

3. **Key Alerts**:
   - Cache hit rate <90% for 5 minutes
   - Query latency p95 >50ms for 5 minutes
   - Error rate >1% for 1 minute

### DataDog Monitors

Create monitors for constitutional compliance:

```bash
# Cache hit rate monitor
curl -X POST "https://api.datadoghq.com/api/v1/monitor" \
  -H "DD-API-KEY: ${DD_API_KEY}" \
  -H "DD-APPLICATION-KEY: ${DD_APP_KEY}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Thread Cache Hit Rate Below Constitutional Minimum",
    "type": "metric alert",
    "query": "avg(last_5m):avg:thread.cache_hit_rate_percent{*} < 90",
    "message": "Cache hit rate is below 90% constitutional requirement. Current: {{value}}%",
    "tags": ["team:thread", "priority:high", "constitutional-compliance"],
    "options": {
      "thresholds": {
        "critical": 90,
        "warning": 85
      },
      "notify_no_data": false,
      "notify_audit": false
    }
  }'

# Query latency monitor
curl -X POST "https://api.datadoghq.com/api/v1/monitor" \
  -H "DD-API-KEY: ${DD_API_KEY}" \
  -H "DD-APPLICATION-KEY: ${DD_APP_KEY}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Thread Query Latency Exceeds Constitutional Maximum",
    "type": "metric alert",
    "query": "avg(last_5m):avg:thread.query_avg_duration_seconds{*} * 1000 > 50",
    "message": "Query latency p95 exceeds 50ms constitutional requirement. Current: {{value}}ms",
    "tags": ["team:thread", "priority:high", "constitutional-compliance"],
    "options": {
      "thresholds": {
        "critical": 50,
        "warning": 45
      },
      "notify_no_data": false,
      "notify_audit": false
    }
  }'
```

---

## Troubleshooting

### No Data Appearing

**Check Prometheus Scrape Configuration**:
```bash
# Verify Prometheus is scraping Thread metrics
curl http://prometheus:9090/api/v1/targets | jq '.data.activeTargets[] | select(.labels.job == "thread")'
```

**Check Thread Metrics Endpoint**:
```bash
# Verify metrics are being exported
curl http://thread-service:8080/metrics | grep thread_cache_hit_rate_percent
```

**Check DataDog Agent Integration**:
```bash
# Verify DataDog Agent is collecting metrics
datadog-agent status | grep thread
```

### Incorrect Metric Names

If metric names don't match:

1. Check `PerformanceMetrics::export_prometheus()` implementation
2. Verify metric prefix is `thread_` not `thread.` (Prometheus uses underscores)
3. For DataDog, metrics are auto-converted (`thread_` → `thread.`)

### Missing Panels

If panels show "No Data":

1. Verify time range is appropriate (default: last 6 hours)
2. Check template variable `$environment` is set correctly
3. Ensure Prometheus data source is selected

### Permission Errors

**Grafana**:
- Requires "Editor" role or higher to import dashboards
- API key needs "Admin" permissions

**DataDog**:
- API key needs dashboard creation permissions
- App key must belong to user with appropriate role

---

## Customization

### Adding Custom Panels

**Grafana**:
1. Click "Add panel" in dashboard edit mode
2. Use Thread metrics from `thread_*` namespace
3. Configure visualization and thresholds
4. Save panel

**DataDog**:
1. Click "Add Widget" button
2. Select widget type (timeseries, query value, etc.)
3. Configure query using `thread.*` metrics
4. Save widget

### Modifying Thresholds

**Constitutional Requirements** (DO NOT MODIFY):
- Cache hit rate: >90% (immutable per Constitution v2.0.0)
- Query latency: <50ms (immutable per Constitution v2.0.0)

**Warning Thresholds** (can be adjusted):
- Cache hit rate warning: 80-90% (configurable)
- Query latency warning: 40-50ms (configurable)

### Adding Environment Labels

If using multi-environment deployment:

```yaml
# Add environment label to metrics
thread_cache_hits_total{environment="production"} 1000
thread_cache_hits_total{environment="staging"} 500
```

Update template variables in dashboards to filter by environment.

---

## Maintenance

### Dashboard Version Control

1. **Export Updated Dashboards**:
   ```bash
   # Grafana
   curl -H "Authorization: Bearer ${GRAFANA_API_KEY}" \
     "${GRAFANA_URL}/api/dashboards/uid/thread-performance" | \
     jq '.dashboard' > grafana/dashboards/thread-performance-monitoring.json

   # DataDog
   curl -H "DD-API-KEY: ${DD_API_KEY}" \
        -H "DD-APPLICATION-KEY: ${DD_APP_KEY}" \
     "https://api.datadoghq.com/api/v1/dashboard/${DASHBOARD_ID}" | \
     jq '.' > datadog/dashboards/thread-performance-monitoring.json
   ```

2. **Commit to Version Control**:
   ```bash
   git add grafana/dashboards/*.json datadog/dashboards/*.json
   git commit -m "docs: update monitoring dashboards"
   ```

3. **Deploy via CI/CD**:
   - Use Terraform or direct API calls
   - Ensure idempotent deployment (use `overwrite` flags)

### Regular Review

- **Monthly**: Review dashboard effectiveness and metrics coverage
- **Quarterly**: Update thresholds based on actual performance data
- **After Incidents**: Add panels for newly identified metrics

---

## Related Documentation

- **Constitutional Requirements**: `.specify/memory/constitution.md`
- **Performance Metrics**: `crates/flow/src/monitoring/performance.rs`
- **Prometheus Export**: `PerformanceMetrics::export_prometheus()` method
- **Capacity Planning**: `docs/operations/CAPACITY_PLANNING.md`
- **Monitoring Guide**: `docs/operations/MONITORING.md`

---

**Version**: 1.0.0
**Last Updated**: 2026-01-28
**Author**: Thread Operations Team (via Claude Sonnet 4.5)
