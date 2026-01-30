#!/bin/bash

# SPDX-FileCopyrightText: 2026 Github
# SPDX-FileCopyrightText: 2026 Knitli Inc.
#
# SPDX-License-Identifier: MIT
# SPDX-License-Identifier: MIT OR Apache-2.0

# Performance Regression Detection Script
# Compares current deployment performance against baseline

set -e

DEPLOYMENT_ID="${1:-unknown}"
BASELINE_FILE="${2:-baseline.json}"
DURATION="${3:-300}"  # Default 5-minute test
ENDPOINT="${THREAD_ENDPOINT:-https://api.thread.io}"

# Thresholds
WARNING_THRESHOLD=25  # 25% degradation
CRITICAL_THRESHOLD=50  # 50% degradation

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"
}

success() {
    echo -e "${GREEN}✓${NC} $*"
}

warn() {
    echo -e "${YELLOW}⚠${NC} $*"
}

fail() {
    echo -e "${RED}✗${NC} $*"
}

alert_slack() {
    if [[ -n "$SLACK_WEBHOOK_URL" ]]; then
        curl -s -X POST "$SLACK_WEBHOOK_URL" \
            -H 'Content-Type: application/json' \
            -d "{\"text\":\"$1\"}" >/dev/null 2>&1
    fi
}

# Run k6 load test
run_load_test() {
    log "Running ${DURATION}s load test against $ENDPOINT"

    k6 run --quiet --out json=results.json --duration "${DURATION}s" - <<EOF
import http from 'k6/http';
import { check, sleep } from 'k6';

export let options = {
    stages: [
        { duration: '30s', target: 50 },
        { duration: '${DURATION - 60}s', target: 100 },
        { duration: '30s', target: 0 },
    ],
};

export default function() {
    const response = http.post('${ENDPOINT}/api/query', JSON.stringify({
        pattern: 'function \$NAME() {}',
        language: 'javascript'
    }), {
        headers: { 'Content-Type': 'application/json' },
    });

    check(response, {
        'status is 200': (r) => r.status === 200,
    });

    sleep(0.1);
}
EOF

    # Convert k6 JSON output to summary
    jq -s '[.[] | select(.type=="Point")] |
           group_by(.metric) |
           map({metric: .[0].metric,
                values: ([.[] | .data.value] |
                         {p50: (sort | .[length/2 | floor]),
                          p95: (sort | .[length * 0.95 | floor]),
                          p99: (sort | .[length * 0.99 | floor]),
                          avg: (add / length)})})' \
        results.json > summary.json

    log "Load test completed"
}

# Compare results with baseline
compare_results() {
    local baseline="$1"
    local current="summary.json"

    if [[ ! -f "$baseline" ]]; then
        log "No baseline found - this will become the new baseline"
        cp "$current" "$baseline"
        success "Baseline created: $baseline"
        return 0
    fi

    log "Comparing performance with baseline..."

    # Extract P95 latency
    baseline_p95=$(jq -r '.[] | select(.metric=="http_req_duration") | .values.p95' "$baseline" 2>/dev/null || echo "0")
    current_p95=$(jq -r '.[] | select(.metric=="http_req_duration") | .values.p95' "$current" 2>/dev/null || echo "0")

    # Extract P99 latency
    baseline_p99=$(jq -r '.[] | select(.metric=="http_req_duration") | .values.p99' "$baseline" 2>/dev/null || echo "0")
    current_p99=$(jq -r '.[] | select(.metric=="http_req_duration") | .values.p99' "$current" 2>/dev/null || echo "0")

    # Calculate deviations
    p95_deviation=$(echo "scale=2; ($current_p95 - $baseline_p95) / $baseline_p95 * 100" | bc 2>/dev/null || echo "0")
    p99_deviation=$(echo "scale=2; ($current_p99 - $baseline_p99) / $baseline_p99 * 100" | bc 2>/dev/null || echo "0")

    echo ""
    echo "========================================="
    echo "Performance Comparison Results"
    echo "========================================="
    echo "Deployment ID: $DEPLOYMENT_ID"
    echo ""
    echo "P95 Latency:"
    echo "  Baseline:  ${baseline_p95}ms"
    echo "  Current:   ${current_p95}ms"
    echo "  Deviation: ${p95_deviation}%"
    echo ""
    echo "P99 Latency:"
    echo "  Baseline:  ${baseline_p99}ms"
    echo "  Current:   ${current_p99}ms"
    echo "  Deviation: ${p99_deviation}%"
    echo ""

    # Evaluate regression
    regression_level="none"
    exit_code=0

    if (( $(echo "$p95_deviation > $CRITICAL_THRESHOLD" | bc -l 2>/dev/null || echo 0) )); then
        regression_level="critical"
        fail "CRITICAL: P95 latency regression > ${CRITICAL_THRESHOLD}%"
        alert_slack "🚨 CRITICAL Performance Regression: P95 latency +${p95_deviation}% on deployment $DEPLOYMENT_ID"
        exit_code=2
    elif (( $(echo "$p95_deviation > $WARNING_THRESHOLD" | bc -l 2>/dev/null || echo 0) )); then
        regression_level="warning"
        warn "WARNING: P95 latency regression > ${WARNING_THRESHOLD}%"
        alert_slack "⚠️  WARNING: Performance Regression: P95 latency +${p95_deviation}% on deployment $DEPLOYMENT_ID"
        exit_code=1
    else
        success "P95 latency within acceptable range"
    fi

    if (( $(echo "$p99_deviation > $CRITICAL_THRESHOLD" | bc -l 2>/dev/null || echo 0) )); then
        regression_level="critical"
        fail "CRITICAL: P99 latency regression > ${CRITICAL_THRESHOLD}%"
        alert_slack "🚨 CRITICAL Performance Regression: P99 latency +${p99_deviation}% on deployment $DEPLOYMENT_ID"
        exit_code=2
    elif (( $(echo "$p99_deviation > $WARNING_THRESHOLD" | bc -l 2>/dev/null || echo 0) )); then
        if [[ "$regression_level" != "critical" ]]; then
            regression_level="warning"
            warn "WARNING: P99 latency regression > ${WARNING_THRESHOLD}%"
            exit_code=1
        fi
    else
        success "P99 latency within acceptable range"
    fi

    echo ""
    echo "Regression Level: $regression_level"
    echo "========================================="

    # Output for CI/CD integration
    echo "p95_deviation=$p95_deviation" >> "$GITHUB_OUTPUT" 2>/dev/null || true
    echo "p99_deviation=$p99_deviation" >> "$GITHUB_OUTPUT" 2>/dev/null || true
    echo "regression_level=$regression_level" >> "$GITHUB_OUTPUT" 2>/dev/null || true

    return $exit_code
}

# Main execution
main() {
    log "Performance Regression Test - Deployment: $DEPLOYMENT_ID"

    # Check dependencies
    if ! command -v k6 &> /dev/null; then
        fail "k6 not installed. Install from: https://k6.io/docs/getting-started/installation/"
        exit 1
    fi

    if ! command -v jq &> /dev/null; then
        fail "jq not installed. Install from package manager."
        exit 1
    fi

    # Run load test
    run_load_test

    # Compare with baseline
    compare_results "$BASELINE_FILE"
    exit_code=$?

    # Cleanup
    rm -f results.json summary.json

    exit $exit_code
}

main
