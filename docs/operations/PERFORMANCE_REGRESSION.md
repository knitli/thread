# Performance Regression Detection

**Version**: 1.0.0
**Last Updated**: 2026-01-28
**Status**: Production Ready

---

## Overview

This document defines the performance regression detection system for Thread, ensuring deployments maintain or improve performance baselines. The system automatically detects performance degradation and triggers alerts or rollbacks when thresholds are exceeded.

### Purpose

- **Prevent Performance Degradation**: Catch performance regressions before user impact
- **Baseline Tracking**: Maintain historical performance baselines for comparison
- **Automated Detection**: Continuous monitoring with automatic alerting
- **Rollback Triggers**: Automatic rollback on critical performance violations

---

## Performance Baselines

### Baseline Metrics

**API Performance Baselines** (Production):

| Metric | Baseline | Warning Threshold | Critical Threshold |
|--------|----------|-------------------|-------------------|
| **P50 Latency** | 50ms | 75ms (+50%) | 100ms (+100%) |
| **P95 Latency** | 150ms | 225ms (+50%) | 300ms (+100%) |
| **P99 Latency** | 300ms | 450ms (+50%) | 600ms (+100%) |
| **Throughput** | 1000 req/s | 800 req/s (-20%) | 600 req/s (-40%) |
| **Error Rate** | 0.01% | 0.05% (+400%) | 0.1% (+900%) |

**Database Performance Baselines**:

| Metric | Baseline | Warning | Critical |
|--------|----------|---------|----------|
| **Query P95** | 10ms | 15ms | 25ms |
| **Query P99** | 25ms | 40ms | 60ms |
| **Connection Pool** | 50% utilized | 70% | 85% |
| **Lock Wait Time** | 1ms | 5ms | 10ms |

**Cache Performance Baselines**:

| Metric | Baseline | Warning | Critical |
|--------|----------|---------|----------|
| **Hit Rate** | 90% | 80% | 70% |
| **Latency P95** | 1ms | 3ms | 5ms |
| **Memory Usage** | 60% | 80% | 90% |

---

## Detection Methods

### 1. Statistical Analysis

**Moving Average Comparison**:
```python
# Compare current performance to 7-day moving average

import numpy as np
from datetime import datetime, timedelta

def detect_regression(current_p95: float, historical_data: list) -> dict:
    """
    Detect performance regression using statistical analysis.

    Args:
        current_p95: Current P95 latency in milliseconds
        historical_data: List of P95 latencies from past 7 days

    Returns:
        dict with regression status and confidence
    """
    # Calculate baseline statistics
    baseline_mean = np.mean(historical_data)
    baseline_std = np.std(historical_data)

    # Calculate z-score (standard deviations from mean)
    z_score = (current_p95 - baseline_mean) / baseline_std if baseline_std > 0 else 0

    # Detect regression
    regression_detected = False
    confidence = 0.0
    severity = "none"

    if z_score > 3:  # > 3 standard deviations
        regression_detected = True
        confidence = 0.99
        severity = "critical"
    elif z_score > 2:  # > 2 standard deviations
        regression_detected = True
        confidence = 0.95
        severity = "warning"
    elif z_score > 1.5:  # > 1.5 standard deviations
        regression_detected = True
        confidence = 0.85
        severity = "info"

    return {
        "regression_detected": regression_detected,
        "confidence": confidence,
        "severity": severity,
        "z_score": z_score,
        "baseline_mean": baseline_mean,
        "current_value": current_p95,
        "deviation_percent": ((current_p95 - baseline_mean) / baseline_mean * 100) if baseline_mean > 0 else 0
    }

# Example usage
historical_p95 = [45, 48, 52, 46, 50, 49, 51]  # Last 7 days
current_p95 = 120  # Current deployment

result = detect_regression(current_p95, historical_p95)
print(f"Regression: {result['regression_detected']}")
print(f"Confidence: {result['confidence']:.2%}")
print(f"Severity: {result['severity']}")
print(f"Deviation: {result['deviation_percent']:.1f}%")
```

### 2. Threshold-Based Detection

**Simple Threshold Alerts**:
```prometheus
# Prometheus alert rule for P95 latency regression

groups:
  - name: performance_regression
    interval: 1m
    rules:
      # Warning: P95 latency 50% above baseline
      - alert: PerformanceRegressionWarning
        expr: |
          histogram_quantile(0.95,
            sum(rate(http_request_duration_seconds_bucket[5m])) by (le)
          ) > 0.225  # 150ms baseline * 1.5
        for: 5m
        labels:
          severity: warning
          team: thread
        annotations:
          summary: "Performance regression detected (warning)"
          description: "P95 latency is {{ $value }}s (baseline: 150ms, threshold: 225ms)"
          baseline: "150ms"
          current: "{{ $value | humanizeDuration }}"
          deviation: "{{ ($value - 0.15) / 0.15 * 100 | humanize }}%"

      # Critical: P95 latency 100% above baseline
      - alert: PerformanceRegressionCritical
        expr: |
          histogram_quantile(0.95,
            sum(rate(http_request_duration_seconds_bucket[5m])) by (le)
          ) > 0.3  # 150ms baseline * 2
        for: 3m
        labels:
          severity: critical
          team: thread
          action: rollback
        annotations:
          summary: "CRITICAL performance regression detected"
          description: "P95 latency is {{ $value }}s (baseline: 150ms, threshold: 300ms)"
          runbook_url: "https://docs.thread.io/runbooks/performance-regression"
```

### 3. Load Test Comparison

**Pre-Deployment vs Post-Deployment**:
```bash
#!/bin/bash
# Performance regression test via load testing

set -e

DEPLOYMENT_ID="${1:-unknown}"
BASELINE_RESULTS="${2:-baseline.json}"
DURATION="${3:-60}"  # seconds

# Run load test
run_load_test() {
    local endpoint="$1"
    local duration="$2"

    echo "Running load test against $endpoint for ${duration}s..."

    k6 run --duration "${duration}s" - <<'EOF'
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

const errorRate = new Rate('errors');
const latencyTrend = new Trend('latency');

export let options = {
    stages: [
        { duration: '10s', target: 50 },   // Ramp up
        { duration: '40s', target: 100 },  // Sustained load
        { duration: '10s', target: 0 },    // Ramp down
    ],
    thresholds: {
        'http_req_duration': ['p(95)<200', 'p(99)<500'],
        'errors': ['rate<0.01'],
    },
};

export default function() {
    const response = http.post(__ENV.ENDPOINT + '/api/query', JSON.stringify({
        pattern: 'function $NAME() {}',
        language: 'javascript'
    }), {
        headers: { 'Content-Type': 'application/json' },
    });

    check(response, {
        'status is 200': (r) => r.status === 200,
        'response time < 200ms': (r) => r.timings.duration < 200,
    }) || errorRate.add(1);

    latencyTrend.add(response.timings.duration);

    sleep(0.1);
}
EOF
}

# Compare results
compare_results() {
    local baseline_file="$1"
    local current_file="$2"

    echo "Comparing performance results..."

    # Extract metrics
    baseline_p95=$(jq -r '.metrics.http_req_duration.values["p(95)"]' "$baseline_file")
    current_p95=$(jq -r '.metrics.http_req_duration.values["p(95)"]' "$current_file")

    baseline_p99=$(jq -r '.metrics.http_req_duration.values["p(99)"]' "$baseline_file")
    current_p99=$(jq -r '.metrics.http_req_duration.values["p(99)"]' "$current_file")

    baseline_error_rate=$(jq -r '.metrics.errors.values.rate' "$baseline_file")
    current_error_rate=$(jq -r '.metrics.errors.values.rate' "$current_file")

    # Calculate deviations
    p95_deviation=$(echo "scale=2; ($current_p95 - $baseline_p95) / $baseline_p95 * 100" | bc)
    p99_deviation=$(echo "scale=2; ($current_p99 - $baseline_p99) / $baseline_p99 * 100" | bc)

    echo "P95 Latency:"
    echo "  Baseline: ${baseline_p95}ms"
    echo "  Current: ${current_p95}ms"
    echo "  Deviation: ${p95_deviation}%"

    echo "P99 Latency:"
    echo "  Baseline: ${baseline_p99}ms"
    echo "  Current: ${current_p99}ms"
    echo "  Deviation: ${p99_deviation}%"

    # Determine pass/fail
    regression_detected=false

    if (( $(echo "$p95_deviation > 50" | bc -l) )); then
        echo "❌ CRITICAL: P95 latency regression > 50%"
        regression_detected=true
    elif (( $(echo "$p95_deviation > 25" | bc -l) )); then
        echo "⚠️  WARNING: P95 latency regression > 25%"
    else
        echo "✅ P95 latency within acceptable range"
    fi

    if (( $(echo "$p99_deviation > 50" | bc -l) )); then
        echo "❌ CRITICAL: P99 latency regression > 50%"
        regression_detected=true
    elif (( $(echo "$p99_deviation > 25" | bc -l) )); then
        echo "⚠️  WARNING: P99 latency regression > 25%"
    else
        echo "✅ P99 latency within acceptable range"
    fi

    if $regression_detected; then
        echo "🚨 Performance regression detected - triggering rollback"
        return 1
    else
        echo "✅ No significant performance regression detected"
        return 0
    fi
}

# Main execution
main() {
    echo "Performance Regression Test - Deployment: $DEPLOYMENT_ID"

    # Run load test and save results
    current_results="results-${DEPLOYMENT_ID}.json"
    ENDPOINT="${THREAD_ENDPOINT:-https://api.thread.io}" run_load_test "$THREAD_ENDPOINT" "$DURATION" | tee "$current_results"

    # Compare with baseline
    if [[ -f "$BASELINE_RESULTS" ]]; then
        compare_results "$BASELINE_RESULTS" "$current_results"
        exit_code=$?

        if [[ $exit_code -ne 0 ]]; then
            # Trigger rollback
            echo "Triggering automatic rollback..."
            ./scripts/rollback-deployment.sh "$DEPLOYMENT_ID"
        fi

        exit $exit_code
    else
        echo "No baseline results found - this will become the new baseline"
        cp "$current_results" "$BASELINE_RESULTS"
    fi
}

main
```

---

## Automated Detection Pipeline

### CI/CD Integration

**GitHub Actions Performance Gate** (`.github/workflows/performance-gate.yml`):
```yaml
name: Performance Regression Gate

on:
  deployment_status:

jobs:
  performance-gate:
    name: Performance Regression Check
    runs-on: ubuntu-latest
    if: github.event.deployment_status.state == 'success'

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Wait for deployment stabilization
        run: sleep 60  # Allow 1 minute for deployment to stabilize

      - name: Install k6
        run: |
          curl https://github.com/grafana/k6/releases/download/v0.46.0/k6-v0.46.0-linux-amd64.tar.gz -L | tar xvz
          sudo mv k6-v0.46.0-linux-amd64/k6 /usr/local/bin/

      - name: Run performance regression test
        id: perf-test
        run: |
          ./scripts/performance-regression-test.sh \
            "${{ github.event.deployment.id }}" \
            "baseline.json" \
            "300"  # 5-minute load test
        continue-on-error: true

      - name: Collect metrics from Prometheus
        run: |
          # Query Prometheus for post-deployment metrics
          curl -s "http://prometheus:9090/api/v1/query?query=histogram_quantile(0.95,sum(rate(http_request_duration_seconds_bucket[5m]))by(le))" \
            | jq -r '.data.result[0].value[1]' > current_p95.txt

          # Compare with baseline
          baseline_p95=$(cat baseline_p95.txt || echo "0.15")
          current_p95=$(cat current_p95.txt)

          deviation=$(echo "scale=2; ($current_p95 - $baseline_p95) / $baseline_p95 * 100" | bc)

          echo "p95_deviation=$deviation" >> $GITHUB_OUTPUT

      - name: Evaluate regression
        id: evaluate
        run: |
          deviation="${{ steps.perf-test.outputs.p95_deviation }}"

          if (( $(echo "$deviation > 100" | bc -l) )); then
            echo "result=critical" >> $GITHUB_OUTPUT
            echo "action=rollback" >> $GITHUB_OUTPUT
          elif (( $(echo "$deviation > 50" | bc -l) )); then
            echo "result=warning" >> $GITHUB_OUTPUT
            echo "action=alert" >> $GITHUB_OUTPUT
          else
            echo "result=pass" >> $GITHUB_OUTPUT
            echo "action=none" >> $GITHUB_OUTPUT
          fi

      - name: Trigger rollback if critical
        if: steps.evaluate.outputs.action == 'rollback'
        run: |
          echo "🚨 CRITICAL performance regression detected - triggering rollback"

          # Trigger rollback workflow
          gh workflow run rollback-deployment.yml \
            --ref ${{ github.ref }} \
            -f deployment_id="${{ github.event.deployment.id }}" \
            -f reason="Performance regression: P95 latency increased by ${{ steps.perf-test.outputs.p95_deviation }}%"
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

      - name: Alert on warning
        if: steps.evaluate.outputs.action == 'alert'
        uses: slackapi/slack-github-action@v1
        with:
          webhook-url: ${{ secrets.SLACK_WEBHOOK_URL }}
          payload: |
            {
              "text": "⚠️  Performance regression warning on deployment ${{ github.event.deployment.id }}",
              "blocks": [
                {
                  "type": "section",
                  "text": {
                    "type": "mrkdwn",
                    "text": "*Performance Regression Warning*\n\nP95 latency increased by *${{ steps.perf-test.outputs.p95_deviation }}%* after deployment."
                  }
                }
              ]
            }

      - name: Fail job if regression detected
        if: steps.evaluate.outputs.result != 'pass'
        run: exit 1
```

---

## Continuous Monitoring

### Real-Time Performance Tracking

**Grafana Dashboard with Baselines**:
```json
{
  "dashboard": {
    "title": "Performance Regression Monitoring",
    "panels": [
      {
        "title": "P95 Latency vs Baseline",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, sum(rate(http_request_duration_seconds_bucket[5m])) by (le))",
            "legendFormat": "Current P95"
          },
          {
            "expr": "0.15",
            "legendFormat": "Baseline P95 (150ms)"
          },
          {
            "expr": "0.225",
            "legendFormat": "Warning Threshold (225ms)"
          },
          {
            "expr": "0.3",
            "legendFormat": "Critical Threshold (300ms)"
          }
        ],
        "alert": {
          "conditions": [
            {
              "evaluator": {"params": [0.3], "type": "gt"},
              "query": {"params": ["A", "5m", "now"]},
              "type": "query"
            }
          ],
          "name": "P95 Latency Critical Regression"
        }
      },
      {
        "title": "Performance Deviation from Baseline",
        "targets": [
          {
            "expr": "(histogram_quantile(0.95, sum(rate(http_request_duration_seconds_bucket[5m])) by (le)) - 0.15) / 0.15 * 100",
            "legendFormat": "P95 Deviation %"
          }
        ],
        "yaxes": [{"format": "percent"}]
      }
    ]
  }
}
```

---

## Rollback Triggers

### Automatic Rollback Criteria

**Rollback Decision Matrix**:

| Condition | Severity | Action | Rollback Type |
|-----------|----------|--------|---------------|
| P95 > 2× baseline | Critical | Automatic Rollback | Immediate |
| P99 > 2× baseline | Critical | Automatic Rollback | Immediate |
| Error rate > 0.1% | Critical | Automatic Rollback | Immediate |
| Throughput < 60% baseline | Critical | Automatic Rollback | Immediate |
| P95 > 1.5× baseline | Warning | Alert + Manual Review | On Approval |
| Cache hit rate < 70% | Warning | Alert Only | Manual |

**Rollback Script** (`scripts/auto-rollback.sh`):
```bash
#!/bin/bash
# Automatic rollback on performance regression

set -e

DEPLOYMENT_ID="$1"
REASON="$2"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"
}

alert_slack() {
    local message="$1"
    curl -s -X POST "$SLACK_WEBHOOK_URL" \
        -H 'Content-Type: application/json' \
        -d "{\"text\":\"🚨 Auto-Rollback: $message\"}" >/dev/null 2>&1
}

# Trigger rollback
trigger_rollback() {
    log "Triggering automatic rollback for deployment $DEPLOYMENT_ID"
    log "Reason: $REASON"

    alert_slack "Automatic rollback initiated for deployment $DEPLOYMENT_ID. Reason: $REASON"

    # Execute rollback based on deployment strategy
    if kubectl get deployment thread-worker-blue -n production &>/dev/null; then
        # Blue-green deployment - switch back traffic
        log "Blue-green rollback: switching traffic back to previous version"

        kubectl patch service thread-service \
            --namespace=production \
            -p '{"spec":{"selector":{"version":"blue"}}}'

        alert_slack "✅ Rollback complete: Traffic switched back to blue environment"
    else
        # Rolling update - rollback to previous revision
        log "Rolling update rollback: reverting to previous revision"

        kubectl rollout undo deployment/thread-worker \
            --namespace=production

        kubectl rollout status deployment/thread-worker \
            --namespace=production \
            --timeout=300s

        alert_slack "✅ Rollback complete: Reverted to previous deployment revision"
    fi

    log "Rollback completed successfully"
}

# Main execution
trigger_rollback
```

---

## Best Practices

### 1. Baseline Management

- **Regular Updates**: Update baselines monthly to account for gradual improvements
- **Multiple Baselines**: Maintain baselines for different traffic patterns (peak, off-peak)
- **Versioned History**: Keep historical baselines for comparison and trend analysis

### 2. Detection Tuning

- **Avoid False Positives**: Set thresholds based on actual traffic patterns
- **Context Awareness**: Consider time-of-day, day-of-week variations
- **Statistical Significance**: Require sustained degradation (5+ minutes) before alerting

### 3. Rollback Strategy

- **Automated for Critical**: Automatic rollback for critical performance violations
- **Manual Review for Warnings**: Alert but don't rollback for minor deviations
- **Post-Rollback Analysis**: Always investigate root cause after rollback

---

**Document Version**: 1.0.0
**Last Updated**: 2026-01-28
**Next Review**: 2026-02-28
**Owner**: Thread Operations Team
