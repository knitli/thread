#!/bin/bash

# SPDX-FileCopyrightText: 2026 Github
# SPDX-FileCopyrightText: 2026 Knitli Inc.
#
# SPDX-License-Identifier: MIT
# SPDX-License-Identifier: MIT OR Apache-2.0

# Continuous Post-Deployment Validation Script
# Runs comprehensive validation checks after deployments

set -e

# Configuration
ENVIRONMENT="${1:-production}"
ENDPOINT="${THREAD_ENDPOINT:-https://api.thread.io}"
DATABASE_URL="${DATABASE_URL}"
REDIS_URL="${REDIS_URL}"
SLACK_WEBHOOK="${SLACK_WEBHOOK_URL}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Metrics
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0
START_TIME=$(date +%s)

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"
}

success() {
    echo -e "${GREEN}✓${NC} $*"
    ((PASSED_CHECKS++))
    ((TOTAL_CHECKS++))
}

fail() {
    echo -e "${RED}✗${NC} $*"
    ((FAILED_CHECKS++))
    ((TOTAL_CHECKS++))
}

warn() {
    echo -e "${YELLOW}⚠${NC} $*"
}

alert_slack() {
    local message="$1"
    if [[ -n "$SLACK_WEBHOOK" ]]; then
        curl -s -X POST "$SLACK_WEBHOOK" \
            -H 'Content-Type: application/json' \
            -d "{\"text\":\"🔍 Validation Alert [$ENVIRONMENT]: $message\"}" >/dev/null 2>&1
    fi
}

# ============================================================================
# Health Check Validation
# ============================================================================

validate_health_check() {
    log "Validating health check endpoint..."

    local response
    local http_code

    response=$(curl -s -w "\n%{http_code}" "$ENDPOINT/health" 2>&1)
    http_code=$(echo "$response" | tail -n1)

    if [[ "$http_code" == "200" ]]; then
        success "Health check endpoint responding (HTTP 200)"

        # Parse health check JSON
        local body=$(echo "$response" | head -n-1)
        local status=$(echo "$body" | jq -r '.status' 2>/dev/null)

        if [[ "$status" == "healthy" ]]; then
            success "Health status is healthy"
        else
            fail "Health status is not healthy: $status"
            alert_slack "Health check status: $status"
        fi

        # Check individual components
        local db_healthy=$(echo "$body" | jq -r '.checks.database.healthy' 2>/dev/null)
        local cache_healthy=$(echo "$body" | jq -r '.checks.cache.healthy' 2>/dev/null)

        if [[ "$db_healthy" == "true" ]]; then
            success "Database health check passed"
        else
            fail "Database health check failed"
            alert_slack "Database health check failed"
        fi

        if [[ "$cache_healthy" == "true" ]]; then
            success "Cache health check passed"
        else
            fail "Cache health check failed"
            alert_slack "Cache health check failed"
        fi
    else
        fail "Health check endpoint returned HTTP $http_code"
        alert_slack "Health check failed: HTTP $http_code"
    fi
}

# ============================================================================
# API Functionality Validation
# ============================================================================

validate_api_query() {
    log "Validating API query functionality..."

    local response
    local http_code

    response=$(curl -s -w "\n%{http_code}" "$ENDPOINT/api/query" \
        -H "Content-Type: application/json" \
        -d '{"pattern":"function $NAME() {}","language":"javascript"}' 2>&1)

    http_code=$(echo "$response" | tail -n1)

    if [[ "$http_code" == "200" ]]; then
        success "API query endpoint responding (HTTP 200)"

        # Validate response structure
        local body=$(echo "$response" | head -n-1)
        local has_results=$(echo "$body" | jq 'has("results")' 2>/dev/null)

        if [[ "$has_results" == "true" ]]; then
            success "API query response has expected structure"
        else
            fail "API query response missing expected fields"
        fi
    else
        fail "API query endpoint returned HTTP $http_code"
        alert_slack "API query failed: HTTP $http_code"
    fi
}

validate_api_performance() {
    log "Validating API performance..."

    local start_time=$(date +%s%N)
    local response
    local http_code

    response=$(curl -s -w "\n%{http_code}" "$ENDPOINT/api/query" \
        -H "Content-Type: application/json" \
        -d '{"pattern":"const $VAR = $VALUE","language":"javascript"}' 2>&1)

    local end_time=$(date +%s%N)
    local duration_ms=$(( (end_time - start_time) / 1000000 ))

    http_code=$(echo "$response" | tail -n1)

    if [[ "$http_code" == "200" ]]; then
        if [[ "$duration_ms" -lt 500 ]]; then
            success "API query performance: ${duration_ms}ms (< 500ms)"
        elif [[ "$duration_ms" -lt 1000 ]]; then
            warn "API query performance: ${duration_ms}ms (acceptable but slow)"
            ((TOTAL_CHECKS++))
            ((PASSED_CHECKS++))
        else
            fail "API query performance: ${duration_ms}ms (> 1000ms)"
            alert_slack "API performance degraded: ${duration_ms}ms"
        fi
    else
        fail "API query failed (HTTP $http_code)"
    fi
}

# ============================================================================
# Database Validation
# ============================================================================

validate_database_connectivity() {
    log "Validating database connectivity..."

    if [[ -z "$DATABASE_URL" ]]; then
        warn "DATABASE_URL not set, skipping database validation"
        return
    fi

    # Test database connectivity using psql
    if command -v psql &> /dev/null; then
        if psql "$DATABASE_URL" -c "SELECT 1;" &> /dev/null; then
            success "Database connectivity verified"
        else
            fail "Database connectivity check failed"
            alert_slack "Database connectivity failed"
        fi
    else
        warn "psql not available, using API health check only"
    fi
}

validate_database_performance() {
    log "Validating database query performance..."

    local response
    local http_code

    local start_time=$(date +%s%N)
    response=$(curl -s -w "\n%{http_code}" "$ENDPOINT/health/database" 2>&1)
    local end_time=$(date +%s%N)
    local duration_ms=$(( (end_time - start_time) / 1000000 ))

    http_code=$(echo "$response" | tail -n1)

    if [[ "$http_code" == "200" ]]; then
        if [[ "$duration_ms" -lt 100 ]]; then
            success "Database query performance: ${duration_ms}ms (< 100ms)"
        elif [[ "$duration_ms" -lt 200 ]]; then
            warn "Database query performance: ${duration_ms}ms (acceptable)"
            ((TOTAL_CHECKS++))
            ((PASSED_CHECKS++))
        else
            fail "Database query performance: ${duration_ms}ms (> 200ms)"
            alert_slack "Database performance degraded: ${duration_ms}ms"
        fi
    else
        fail "Database health check failed (HTTP $http_code)"
    fi
}

# ============================================================================
# Cache Validation
# ============================================================================

validate_cache_connectivity() {
    log "Validating cache connectivity..."

    if [[ -z "$REDIS_URL" ]]; then
        warn "REDIS_URL not set, skipping cache validation"
        return
    fi

    # Test cache connectivity using redis-cli
    if command -v redis-cli &> /dev/null; then
        if redis-cli -u "$REDIS_URL" PING | grep -q "PONG"; then
            success "Cache connectivity verified"
        else
            fail "Cache connectivity check failed"
            alert_slack "Cache connectivity failed"
        fi
    else
        warn "redis-cli not available, using API health check only"
    fi
}

validate_cache_performance() {
    log "Validating cache performance..."

    local response
    local http_code

    response=$(curl -s -w "\n%{http_code}" "$ENDPOINT/health/cache" 2>&1)
    http_code=$(echo "$response" | tail -n1)

    if [[ "$http_code" == "200" ]]; then
        success "Cache health check passed"

        # Parse latency if available
        local body=$(echo "$response" | head -n-1)
        local latency=$(echo "$body" | jq -r '.latency_ms // empty' 2>/dev/null)

        if [[ -n "$latency" ]]; then
            if (( $(echo "$latency < 10" | bc -l) )); then
                success "Cache latency: ${latency}ms (< 10ms)"
            elif (( $(echo "$latency < 50" | bc -l) )); then
                warn "Cache latency: ${latency}ms (acceptable)"
                ((TOTAL_CHECKS++))
                ((PASSED_CHECKS++))
            else
                fail "Cache latency: ${latency}ms (> 50ms)"
                alert_slack "Cache performance degraded: ${latency}ms"
            fi
        fi
    else
        fail "Cache health check failed (HTTP $http_code)"
    fi
}

# ============================================================================
# Integration Validation
# ============================================================================

validate_end_to_end_flow() {
    log "Validating end-to-end user flow..."

    # Simulate complete user workflow: query → parse → cache → return

    # Step 1: Query API
    local query_response
    local http_code

    query_response=$(curl -s -w "\n%{http_code}" "$ENDPOINT/api/query" \
        -H "Content-Type: application/json" \
        -d '{"pattern":"class $NAME {}","language":"javascript"}' 2>&1)

    http_code=$(echo "$query_response" | tail -n1)

    if [[ "$http_code" != "200" ]]; then
        fail "End-to-end flow: Query failed (HTTP $http_code)"
        return
    fi

    # Step 2: Verify response has results
    local body=$(echo "$query_response" | head -n-1)
    local has_results=$(echo "$body" | jq 'has("results")' 2>/dev/null)

    if [[ "$has_results" != "true" ]]; then
        fail "End-to-end flow: Response missing results"
        return
    fi

    # Step 3: Verify cache is populated (second request should be faster)
    local start_time=$(date +%s%N)
    curl -s "$ENDPOINT/api/query" \
        -H "Content-Type: application/json" \
        -d '{"pattern":"class $NAME {}","language":"javascript"}' >/dev/null 2>&1
    local end_time=$(date +%s%N)
    local cached_duration_ms=$(( (end_time - start_time) / 1000000 ))

    if [[ "$cached_duration_ms" -lt 100 ]]; then
        success "End-to-end flow: Cache working (${cached_duration_ms}ms cached request)"
    else
        warn "End-to-end flow: Cache may not be working efficiently"
        ((TOTAL_CHECKS++))
        ((PASSED_CHECKS++))
    fi

    success "End-to-end flow completed successfully"
}

# ============================================================================
# Security Validation
# ============================================================================

validate_security_headers() {
    log "Validating security headers..."

    local headers
    headers=$(curl -s -I "$ENDPOINT" 2>&1)

    # Check for important security headers
    if echo "$headers" | grep -qi "strict-transport-security"; then
        success "HSTS header present"
    else
        fail "HSTS header missing"
    fi

    if echo "$headers" | grep -qi "x-frame-options"; then
        success "X-Frame-Options header present"
    else
        warn "X-Frame-Options header missing (not critical)"
        ((TOTAL_CHECKS++))
        ((PASSED_CHECKS++))
    fi

    if echo "$headers" | grep -qi "x-content-type-options"; then
        success "X-Content-Type-Options header present"
    else
        warn "X-Content-Type-Options header missing"
        ((TOTAL_CHECKS++))
        ((PASSED_CHECKS++))
    fi
}

validate_https() {
    log "Validating HTTPS enforcement..."

    # Check if HTTP redirects to HTTPS
    if [[ "$ENDPOINT" == https://* ]]; then
        local http_endpoint="${ENDPOINT/https:/http:}"
        local redirect_location=$(curl -s -I "$http_endpoint" 2>&1 | grep -i "^location:" | awk '{print $2}' | tr -d '\r')

        if [[ "$redirect_location" == https://* ]]; then
            success "HTTP to HTTPS redirect working"
        else
            fail "HTTP to HTTPS redirect not configured"
            alert_slack "HTTPS redirect not working"
        fi
    fi

    # Verify TLS certificate
    if command -v openssl &> /dev/null; then
        local cert_info
        cert_info=$(echo | openssl s_client -connect "$(echo "$ENDPOINT" | sed 's|https://||' | sed 's|/.*||'):443" 2>&1)

        if echo "$cert_info" | grep -q "Verify return code: 0 (ok)"; then
            success "TLS certificate valid"
        else
            fail "TLS certificate validation failed"
            alert_slack "TLS certificate issue detected"
        fi
    fi
}

# ============================================================================
# Report Generation
# ============================================================================

generate_report() {
    local end_time=$(date +%s)
    local duration=$((end_time - START_TIME))

    echo ""
    echo "========================================="
    echo "Continuous Validation Report"
    echo "========================================="
    echo "Environment: $ENVIRONMENT"
    echo "Endpoint: $ENDPOINT"
    echo "Timestamp: $(date '+%Y-%m-%d %H:%M:%S')"
    echo "Duration: ${duration}s"
    echo ""
    echo "Results:"
    echo "  Total Checks: $TOTAL_CHECKS"
    echo "  Passed: $PASSED_CHECKS"
    echo "  Failed: $FAILED_CHECKS"
    echo ""

    local pass_rate=$(( PASSED_CHECKS * 100 / TOTAL_CHECKS ))

    if [[ "$FAILED_CHECKS" -eq 0 ]]; then
        echo -e "${GREEN}✓ All validation checks passed!${NC}"
        alert_slack "✅ Validation passed: $PASSED_CHECKS/$TOTAL_CHECKS checks successful"
        return 0
    elif [[ "$pass_rate" -ge 80 ]]; then
        echo -e "${YELLOW}⚠ Some validation checks failed (${pass_rate}% pass rate)${NC}"
        alert_slack "⚠️  Validation partial: $PASSED_CHECKS/$TOTAL_CHECKS checks passed"
        return 1
    else
        echo -e "${RED}✗ Validation failed with ${pass_rate}% pass rate${NC}"
        alert_slack "🚨 Validation failed: Only $PASSED_CHECKS/$TOTAL_CHECKS checks passed"
        return 2
    fi
}

# ============================================================================
# Main Execution
# ============================================================================

main() {
    log "Starting continuous validation for $ENVIRONMENT environment"
    echo ""

    # Health checks
    validate_health_check
    echo ""

    # API validation
    validate_api_query
    validate_api_performance
    echo ""

    # Database validation
    validate_database_connectivity
    validate_database_performance
    echo ""

    # Cache validation
    validate_cache_connectivity
    validate_cache_performance
    echo ""

    # Integration validation
    validate_end_to_end_flow
    echo ""

    # Security validation
    validate_security_headers
    validate_https
    echo ""

    # Generate and display report
    generate_report
}

# Run main function
main
exit $?
