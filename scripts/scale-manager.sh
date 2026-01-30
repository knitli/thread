#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2026 Github
# SPDX-FileCopyrightText: 2026 Knitli Inc.
#
# SPDX-License-Identifier: AGPL-3.0-or-later
# SPDX-License-Identifier: MIT
# SPDX-License-Identifier: MIT OR Apache-2.0

# Thread Scaling Manager
#
# Automated capacity management and scaling decision logic
# Monitors Thread metrics and triggers scale-up/scale-down actions
#
# Usage:
#   ./scripts/scale-manager.sh monitor        # Start monitoring (daemon mode)
#   ./scripts/scale-manager.sh check         # One-time check and scale decision
#   ./scripts/scale-manager.sh scale-up      # Manual scale-up
#   ./scripts/scale-manager.sh scale-down    # Manual scale-down
#   ./scripts/scale-manager.sh status        # Show current scaling status

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Prometheus endpoint
PROMETHEUS_URL="${PROMETHEUS_URL:-http://localhost:9090}"

# Scaling thresholds
CPU_SCALE_UP_THRESHOLD="${CPU_SCALE_UP_THRESHOLD:-70}"    # CPU > 70% for 5 minutes
CPU_SCALE_DOWN_THRESHOLD="${CPU_SCALE_DOWN_THRESHOLD:-20}"  # CPU < 20% for 15 minutes
MEMORY_SCALE_UP_THRESHOLD="${MEMORY_SCALE_UP_THRESHOLD:-80}"  # Memory > 80%
MEMORY_SCALE_DOWN_THRESHOLD="${MEMORY_SCALE_DOWN_THRESHOLD:-40}"  # Memory < 40%
QUEUE_DEPTH_SCALE_UP_THRESHOLD="${QUEUE_DEPTH_SCALE_UP_THRESHOLD:-100}"  # Queue > 100
CACHE_HIT_RATE_THRESHOLD="${CACHE_HIT_RATE_THRESHOLD:-90}"  # Cache hit rate < 90%

# Scaling configuration
MIN_INSTANCES="${MIN_INSTANCES:-2}"
MAX_INSTANCES="${MAX_INSTANCES:-10}"
SCALE_UP_INCREMENT="${SCALE_UP_INCREMENT:-2}"  # Add 2 instances at a time
SCALE_DOWN_INCREMENT="${SCALE_DOWN_INCREMENT:-1}"  # Remove 1 instance at a time
COOLDOWN_PERIOD="${COOLDOWN_PERIOD:-300}"  # 5 minutes between scaling actions

# State file for tracking
STATE_FILE="${STATE_FILE:-/tmp/thread-scale-manager.state}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $(date '+%Y-%m-%d %H:%M:%S') $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $(date '+%Y-%m-%d %H:%M:%S') $*"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $(date '+%Y-%m-%d %H:%M:%S') $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $(date '+%Y-%m-%d %H:%M:%S') $*" >&2
}

# Query Prometheus metrics
query_prometheus() {
    local query="$1"
    local result

    result=$(curl -s -G \
        --data-urlencode "query=$query" \
        "${PROMETHEUS_URL}/api/v1/query" \
        | jq -r '.data.result[0].value[1]' 2>/dev/null)

    if [[ "$result" == "null" || -z "$result" ]]; then
        echo "0"
    else
        echo "$result"
    fi
}

# Get current CPU utilization (average across all instances)
get_cpu_utilization() {
    local query='100 - (avg by (instance) (irate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)'
    query_prometheus "$query"
}

# Get current memory utilization
get_memory_utilization() {
    local query='(node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes) / node_memory_MemTotal_bytes * 100'
    query_prometheus "$query"
}

# Get queue depth
get_queue_depth() {
    local query='thread_request_queue_depth'
    query_prometheus "$query"
}

# Get cache hit rate
get_cache_hit_rate() {
    local query='thread_cache_hit_rate_percent'
    query_prometheus "$query"
}

# Get current instance count
get_current_instances() {
    # Try Kubernetes first
    if command -v kubectl &>/dev/null; then
        kubectl get deployment thread-worker -n thread -o jsonpath='{.spec.replicas}' 2>/dev/null || echo "$MIN_INSTANCES"
    else
        # Fallback: count from state file or default
        if [[ -f "$STATE_FILE" ]]; then
            jq -r '.current_instances // 2' "$STATE_FILE" 2>/dev/null || echo "$MIN_INSTANCES"
        else
            echo "$MIN_INSTANCES"
        fi
    fi
}

# Get last scaling action timestamp
get_last_scaling_timestamp() {
    if [[ -f "$STATE_FILE" ]]; then
        jq -r '.last_scaling_timestamp // 0' "$STATE_FILE" 2>/dev/null || echo "0"
    else
        echo "0"
    fi
}

# Check if in cooldown period
is_in_cooldown() {
    local last_scaling=$(get_last_scaling_timestamp)
    local current_time=$(date +%s)
    local elapsed=$((current_time - last_scaling))

    if [[ $elapsed -lt $COOLDOWN_PERIOD ]]; then
        log_info "In cooldown period ($elapsed/$COOLDOWN_PERIOD seconds)"
        return 0
    else
        return 1
    fi
}

# Update state file
update_state() {
    local instances="$1"
    local action="$2"

    cat > "$STATE_FILE" <<EOF
{
    "current_instances": $instances,
    "last_scaling_timestamp": $(date +%s),
    "last_action": "$action",
    "last_action_time": "$(date '+%Y-%m-%d %H:%M:%S')"
}
EOF
}

# Scale up instances
scale_up() {
    local current_instances=$(get_current_instances)
    local new_instances=$((current_instances + SCALE_UP_INCREMENT))

    # Cap at max instances
    if [[ $new_instances -gt $MAX_INSTANCES ]]; then
        new_instances=$MAX_INSTANCES
    fi

    if [[ $new_instances -eq $current_instances ]]; then
        log_warning "Already at maximum instances ($MAX_INSTANCES)"
        return 1
    fi

    log_info "Scaling up from $current_instances to $new_instances instances"

    # Execute scaling based on platform
    if command -v kubectl &>/dev/null; then
        # Kubernetes
        kubectl scale deployment thread-worker -n thread --replicas="$new_instances"
    elif [[ -f /etc/haproxy/haproxy.cfg ]]; then
        # HAProxy (manual node activation)
        log_info "Using HAProxy - activate additional worker nodes manually"
        log_info "Edit /etc/haproxy/haproxy.cfg and reload: systemctl reload haproxy"
    else
        # Standalone mode (informational only)
        log_info "Standalone mode: Start $SCALE_UP_INCREMENT additional Thread instances"
    fi

    update_state "$new_instances" "scale_up"
    log_success "Scaled up to $new_instances instances"
}

# Scale down instances
scale_down() {
    local current_instances=$(get_current_instances)
    local new_instances=$((current_instances - SCALE_DOWN_INCREMENT))

    # Cap at min instances
    if [[ $new_instances -lt $MIN_INSTANCES ]]; then
        new_instances=$MIN_INSTANCES
    fi

    if [[ $new_instances -eq $current_instances ]]; then
        log_warning "Already at minimum instances ($MIN_INSTANCES)"
        return 1
    fi

    log_info "Scaling down from $current_instances to $new_instances instances"

    # Execute scaling based on platform
    if command -v kubectl &>/dev/null; then
        # Kubernetes
        kubectl scale deployment thread-worker -n thread --replicas="$new_instances"
    elif [[ -f /etc/haproxy/haproxy.cfg ]]; then
        # HAProxy (manual node deactivation)
        log_info "Using HAProxy - deactivate excess worker nodes manually"
        log_info "Edit /etc/haproxy/haproxy.cfg and reload: systemctl reload haproxy"
    else
        # Standalone mode (informational only)
        log_info "Standalone mode: Stop $SCALE_DOWN_INCREMENT Thread instances"
    fi

    update_state "$new_instances" "scale_down"
    log_success "Scaled down to $new_instances instances"
}

# Check metrics and make scaling decision
check_and_scale() {
    log_info "Checking metrics for scaling decision"

    # Get current metrics
    local cpu=$(get_cpu_utilization)
    local memory=$(get_memory_utilization)
    local queue_depth=$(get_queue_depth)
    local cache_hit_rate=$(get_cache_hit_rate)
    local current_instances=$(get_current_instances)

    # Convert to integers for comparison
    cpu=${cpu%.*}
    memory=${memory%.*}
    queue_depth=${queue_depth%.*}
    cache_hit_rate=${cache_hit_rate%.*}

    log_info "Current metrics:"
    log_info "  CPU: ${cpu}% (scale-up: >${CPU_SCALE_UP_THRESHOLD}%, scale-down: <${CPU_SCALE_DOWN_THRESHOLD}%)"
    log_info "  Memory: ${memory}% (scale-up: >${MEMORY_SCALE_UP_THRESHOLD}%, scale-down: <${MEMORY_SCALE_DOWN_THRESHOLD}%)"
    log_info "  Queue depth: ${queue_depth} (scale-up: >${QUEUE_DEPTH_SCALE_UP_THRESHOLD})"
    log_info "  Cache hit rate: ${cache_hit_rate}% (alert: <${CACHE_HIT_RATE_THRESHOLD}%)"
    log_info "  Current instances: ${current_instances}"

    # Check if in cooldown period
    if is_in_cooldown; then
        log_warning "Skipping scaling decision due to cooldown"
        return 0
    fi

    # Scale-up decision
    local should_scale_up=false
    local scale_up_reasons=()

    if [[ $cpu -gt $CPU_SCALE_UP_THRESHOLD ]]; then
        should_scale_up=true
        scale_up_reasons+=("CPU ${cpu}% > ${CPU_SCALE_UP_THRESHOLD}%")
    fi

    if [[ $memory -gt $MEMORY_SCALE_UP_THRESHOLD ]]; then
        should_scale_up=true
        scale_up_reasons+=("Memory ${memory}% > ${MEMORY_SCALE_UP_THRESHOLD}%")
    fi

    if [[ $queue_depth -gt $QUEUE_DEPTH_SCALE_UP_THRESHOLD ]]; then
        should_scale_up=true
        scale_up_reasons+=("Queue depth ${queue_depth} > ${QUEUE_DEPTH_SCALE_UP_THRESHOLD}")
    fi

    if [[ $cache_hit_rate -lt $CACHE_HIT_RATE_THRESHOLD ]]; then
        log_warning "Low cache hit rate: ${cache_hit_rate}% < ${CACHE_HIT_RATE_THRESHOLD}%"
        log_warning "Consider increasing cache size rather than scaling"
    fi

    if [[ "$should_scale_up" == true ]]; then
        log_warning "Scale-up triggered by: ${scale_up_reasons[*]}"
        scale_up
        return 0
    fi

    # Scale-down decision
    local should_scale_down=false
    local scale_down_reasons=()

    if [[ $cpu -lt $CPU_SCALE_DOWN_THRESHOLD ]]; then
        should_scale_down=true
        scale_down_reasons+=("CPU ${cpu}% < ${CPU_SCALE_DOWN_THRESHOLD}%")
    fi

    if [[ $memory -lt $MEMORY_SCALE_DOWN_THRESHOLD ]]; then
        should_scale_down=true
        scale_down_reasons+=("Memory ${memory}% < ${MEMORY_SCALE_DOWN_THRESHOLD}%")
    fi

    if [[ $queue_depth -eq 0 ]]; then
        should_scale_down=true
        scale_down_reasons+=("Queue is empty")
    fi

    if [[ "$should_scale_down" == true ]]; then
        log_info "Scale-down triggered by: ${scale_down_reasons[*]}"
        scale_down
        return 0
    fi

    log_success "No scaling action needed (metrics within thresholds)"
}

# Monitor mode (daemon)
monitor() {
    log_info "Starting monitoring mode (check interval: 60 seconds)"
    log_info "Thresholds: CPU scale-up>${CPU_SCALE_UP_THRESHOLD}%, scale-down<${CPU_SCALE_DOWN_THRESHOLD}%"
    log_info "            Memory scale-up>${MEMORY_SCALE_UP_THRESHOLD}%, scale-down<${MEMORY_SCALE_DOWN_THRESHOLD}%"
    log_info "            Queue depth scale-up>${QUEUE_DEPTH_SCALE_UP_THRESHOLD}"
    log_info "            Cooldown period: ${COOLDOWN_PERIOD}s"

    while true; do
        check_and_scale || true
        log_info "Sleeping for 60 seconds..."
        sleep 60
    done
}

# Show current status
show_status() {
    local cpu=$(get_cpu_utilization)
    local memory=$(get_memory_utilization)
    local queue_depth=$(get_queue_depth)
    local cache_hit_rate=$(get_cache_hit_rate)
    local current_instances=$(get_current_instances)

    echo ""
    echo "========================================="
    echo " Thread Scaling Manager Status"
    echo "========================================="
    echo ""
    echo "Current Instances: $current_instances (min: $MIN_INSTANCES, max: $MAX_INSTANCES)"
    echo ""
    echo "Metrics:"
    echo "  CPU:            ${cpu%.*}% (scale-up: >${CPU_SCALE_UP_THRESHOLD}%, scale-down: <${CPU_SCALE_DOWN_THRESHOLD}%)"
    echo "  Memory:         ${memory%.*}% (scale-up: >${MEMORY_SCALE_UP_THRESHOLD}%, scale-down: <${MEMORY_SCALE_DOWN_THRESHOLD}%)"
    echo "  Queue Depth:    ${queue_depth%.*} (scale-up: >${QUEUE_DEPTH_SCALE_UP_THRESHOLD})"
    echo "  Cache Hit Rate: ${cache_hit_rate%.*}% (alert: <${CACHE_HIT_RATE_THRESHOLD}%)"
    echo ""

    if [[ -f "$STATE_FILE" ]]; then
        local last_action=$(jq -r '.last_action // "none"' "$STATE_FILE" 2>/dev/null || echo "none")
        local last_action_time=$(jq -r '.last_action_time // "never"' "$STATE_FILE" 2>/dev/null || echo "never")
        echo "Last Scaling Action: $last_action at $last_action_time"
    else
        echo "Last Scaling Action: none (no state file)"
    fi
    echo ""
    echo "========================================="
}

# Main command handler
main() {
    local command="${1:-}"

    case "$command" in
        monitor)
            monitor
            ;;
        check)
            check_and_scale
            ;;
        scale-up)
            if is_in_cooldown; then
                log_error "Cannot scale up: in cooldown period"
                exit 1
            fi
            scale_up
            ;;
        scale-down)
            if is_in_cooldown; then
                log_error "Cannot scale down: in cooldown period"
                exit 1
            fi
            scale_down
            ;;
        status)
            show_status
            ;;
        help|--help|-h)
            cat <<EOF
Thread Scaling Manager

Automated capacity management and scaling decision logic.

Usage:
    ./scripts/scale-manager.sh <command>

Commands:
    monitor       Start monitoring mode (daemon) - check every 60 seconds
    check         One-time check and scaling decision
    scale-up      Manual scale-up (add $SCALE_UP_INCREMENT instances)
    scale-down    Manual scale-down (remove $SCALE_DOWN_INCREMENT instances)
    status        Show current scaling status and metrics
    help          Show this help message

Environment Variables:
    PROMETHEUS_URL                 Prometheus endpoint (default: http://localhost:9090)
    CPU_SCALE_UP_THRESHOLD         CPU % to trigger scale-up (default: 70)
    CPU_SCALE_DOWN_THRESHOLD       CPU % to trigger scale-down (default: 20)
    MEMORY_SCALE_UP_THRESHOLD      Memory % to trigger scale-up (default: 80)
    MEMORY_SCALE_DOWN_THRESHOLD    Memory % to trigger scale-down (default: 40)
    QUEUE_DEPTH_SCALE_UP_THRESHOLD Queue depth to trigger scale-up (default: 100)
    CACHE_HIT_RATE_THRESHOLD       Cache hit rate alert threshold (default: 90)
    MIN_INSTANCES                  Minimum instances (default: 2)
    MAX_INSTANCES                  Maximum instances (default: 10)
    SCALE_UP_INCREMENT             Instances to add on scale-up (default: 2)
    SCALE_DOWN_INCREMENT           Instances to remove on scale-down (default: 1)
    COOLDOWN_PERIOD                Seconds between scaling actions (default: 300)

Examples:
    # Start daemon mode
    ./scripts/scale-manager.sh monitor

    # Check current status
    ./scripts/scale-manager.sh status

    # Manual scale-up
    ./scripts/scale-manager.sh scale-up

    # One-time check and auto-scale
    ./scripts/scale-manager.sh check

Platform Support:
    - Kubernetes: Uses kubectl to scale deployments
    - HAProxy: Provides manual scaling instructions
    - Standalone: Informational output for manual scaling

Integration:
    - Prometheus: Queries metrics for scaling decisions
    - Day 15: Uses fingerprint and cache metrics
    - Day 20: Uses monitoring infrastructure
    - Day 23: Uses performance benchmarks for thresholds

EOF
            ;;
        *)
            log_error "Unknown command: $command"
            log_info "Run './scripts/scale-manager.sh help' for usage information"
            exit 1
            ;;
    esac
}

# Run main
main "$@"
