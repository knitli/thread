#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2026 Github
# SPDX-FileCopyrightText: 2026 Knitli Inc.
#
# SPDX-License-Identifier: MIT
# SPDX-License-Identifier: MIT OR Apache-2.0

# Performance profiling script for Thread
# Supports flamegraphs, perf, memory profiling, and custom benchmarks

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PROFILE_DIR="$PROJECT_ROOT/target/profiling"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*"
}

# Check dependencies
check_dependencies() {
    local missing_deps=()

    # Check for cargo-flamegraph
    if ! command -v cargo-flamegraph &> /dev/null; then
        missing_deps+=("cargo-flamegraph")
    fi

    # Check for perf (Linux only)
    if [[ "$OSTYPE" == "linux-gnu"* ]] && ! command -v perf &> /dev/null; then
        log_warning "perf not found (optional for flamegraphs)"
    fi

    # Check for valgrind (optional)
    if ! command -v valgrind &> /dev/null; then
        log_warning "valgrind not found (optional for memory profiling)"
    fi

    # Check for heaptrack (optional)
    if ! command -v heaptrack &> /dev/null; then
        log_warning "heaptrack not found (optional for heap profiling)"
    fi

    if [ ${#missing_deps[@]} -gt 0 ]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        log_info "Install with: cargo install ${missing_deps[*]}"
        exit 1
    fi
}

# Generate flamegraph
generate_flamegraph() {
    local bench_name="${1:-all}"
    local output_file="${2:-flamegraph.svg}"

    log_info "Generating flamegraph for: $bench_name"
    mkdir -p "$PROFILE_DIR"

    cd "$PROJECT_ROOT"

    if [[ "$bench_name" == "all" ]]; then
        cargo flamegraph --bench fingerprint_benchmark \
            --output "$PROFILE_DIR/$output_file" \
            -- --bench
    else
        cargo flamegraph --bench "$bench_name" \
            --output "$PROFILE_DIR/$output_file" \
            -- --bench
    fi

    log_success "Flamegraph saved to: $PROFILE_DIR/$output_file"
}

# Profile with perf (Linux only)
profile_perf() {
    local bench_name="${1:-fingerprint_benchmark}"
    local duration="${2:-10}"

    if [[ "$OSTYPE" != "linux-gnu"* ]]; then
        log_error "perf profiling only available on Linux"
        return 1
    fi

    log_info "Profiling with perf for ${duration}s: $bench_name"
    mkdir -p "$PROFILE_DIR"

    cd "$PROJECT_ROOT"

    # Build release binary
    cargo build --release --bench "$bench_name"

    # Run perf record
    perf record -F 99 -g --call-graph dwarf \
        -o "$PROFILE_DIR/perf.data" \
        target/release/deps/"$bench_name"-* --bench \
        2>&1 | head -n "$duration"

    # Generate perf report
    perf report -i "$PROFILE_DIR/perf.data" > "$PROFILE_DIR/perf-report.txt"

    log_success "Perf data saved to: $PROFILE_DIR/perf.data"
    log_info "View with: perf report -i $PROFILE_DIR/perf.data"
}

# Memory profiling with valgrind
profile_memory_valgrind() {
    local test_name="${1:-fingerprint}"

    log_info "Memory profiling with valgrind: $test_name"
    mkdir -p "$PROFILE_DIR"

    cd "$PROJECT_ROOT"

    # Build test binary
    cargo test --no-run --release -p thread-flow --lib "$test_name"

    # Find test binary
    local test_binary
    test_binary=$(find target/release/deps -name "thread_flow-*" -type f -executable | head -1)

    if [[ -z "$test_binary" ]]; then
        log_error "Could not find test binary"
        return 1
    fi

    log_info "Running valgrind on: $test_binary"

    # Run valgrind with massif (heap profiler)
    valgrind --tool=massif \
        --massif-out-file="$PROFILE_DIR/massif.out" \
        "$test_binary" "$test_name" 2>&1 | tee "$PROFILE_DIR/valgrind.log"

    # Generate report
    ms_print "$PROFILE_DIR/massif.out" > "$PROFILE_DIR/massif-report.txt"

    log_success "Memory profile saved to: $PROFILE_DIR/massif.out"
    log_info "View with: ms_print $PROFILE_DIR/massif.out"
}

# Heap profiling with heaptrack (Linux only)
profile_heap() {
    local bench_name="${1:-fingerprint_benchmark}"

    if ! command -v heaptrack &> /dev/null; then
        log_error "heaptrack not installed"
        log_info "Install with: sudo apt-get install heaptrack (Ubuntu/Debian)"
        return 1
    fi

    log_info "Heap profiling with heaptrack: $bench_name"
    mkdir -p "$PROFILE_DIR"

    cd "$PROJECT_ROOT"

    # Build release binary
    cargo build --release --bench "$bench_name"

    # Find benchmark binary
    local bench_binary
    bench_binary=$(find target/release/deps -name "${bench_name}-*" -type f -executable | head -1)

    if [[ -z "$bench_binary" ]]; then
        log_error "Could not find benchmark binary"
        return 1
    fi

    log_info "Running heaptrack on: $bench_binary"

    # Run heaptrack
    heaptrack -o "$PROFILE_DIR/heaptrack" "$bench_binary" --bench

    log_success "Heap profile saved to: $PROFILE_DIR/heaptrack.*.gz"
    log_info "View with: heaptrack --analyze $PROFILE_DIR/heaptrack.*.gz"
}

# Run comprehensive profiling suite
profile_comprehensive() {
    log_info "Running comprehensive profiling suite"

    check_dependencies

    log_info "Step 1/4: Generating flamegraph"
    generate_flamegraph "fingerprint_benchmark" "flamegraph-fingerprint.svg"

    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        log_info "Step 2/4: Running perf profiling"
        profile_perf "fingerprint_benchmark" 10
    else
        log_warning "Skipping perf profiling (Linux only)"
    fi

    if command -v valgrind &> /dev/null; then
        log_info "Step 3/4: Memory profiling with valgrind"
        profile_memory_valgrind "cache"
    else
        log_warning "Skipping valgrind profiling (not installed)"
    fi

    if command -v heaptrack &> /dev/null && [[ "$OSTYPE" == "linux-gnu"* ]]; then
        log_info "Step 4/4: Heap profiling with heaptrack"
        profile_heap "fingerprint_benchmark"
    else
        log_warning "Skipping heaptrack profiling (not available)"
    fi

    log_success "Comprehensive profiling complete!"
    log_info "Results in: $PROFILE_DIR"
}

# Quick profiling (flamegraph only)
profile_quick() {
    log_info "Running quick profiling (flamegraph only)"
    check_dependencies
    generate_flamegraph "all" "flamegraph-quick.svg"
}

# Custom benchmark profiling
profile_benchmark() {
    local bench_name="$1"
    local profile_type="${2:-flamegraph}"

    case "$profile_type" in
        flamegraph)
            generate_flamegraph "$bench_name" "flamegraph-${bench_name}.svg"
            ;;
        perf)
            profile_perf "$bench_name"
            ;;
        memory)
            profile_memory_valgrind "$bench_name"
            ;;
        heap)
            profile_heap "$bench_name"
            ;;
        *)
            log_error "Unknown profile type: $profile_type"
            log_info "Valid types: flamegraph, perf, memory, heap"
            return 1
            ;;
    esac
}

# Show usage
usage() {
    cat <<EOF
Usage: $0 <command> [options]

Commands:
  quick                    Quick flamegraph profiling
  comprehensive            Full profiling suite (flamegraph, perf, memory, heap)
  flamegraph [bench]       Generate flamegraph for benchmark
  perf [bench] [duration]  Profile with perf (Linux only)
  memory [test]            Memory profiling with valgrind
  heap [bench]             Heap profiling with heaptrack
  benchmark <bench> <type> Custom benchmark profiling

Options:
  bench      Benchmark name (default: fingerprint_benchmark)
  test       Test name for memory profiling
  type       Profile type: flamegraph, perf, memory, heap
  duration   Duration in seconds for perf profiling

Examples:
  $0 quick
  $0 comprehensive
  $0 flamegraph fingerprint_benchmark
  $0 perf fingerprint_benchmark 30
  $0 memory cache
  $0 benchmark fingerprint_benchmark flamegraph

Dependencies:
  Required: cargo-flamegraph
  Optional: perf (Linux), valgrind, heaptrack

Install: cargo install cargo-flamegraph
EOF
}

# Main command dispatcher
main() {
    if [[ $# -eq 0 ]]; then
        usage
        exit 1
    fi

    case "$1" in
        quick)
            profile_quick
            ;;
        comprehensive)
            profile_comprehensive
            ;;
        flamegraph)
            generate_flamegraph "${2:-all}" "${3:-flamegraph.svg}"
            ;;
        perf)
            profile_perf "${2:-fingerprint_benchmark}" "${3:-10}"
            ;;
        memory)
            profile_memory_valgrind "${2:-cache}"
            ;;
        heap)
            profile_heap "${2:-fingerprint_benchmark}"
            ;;
        benchmark)
            if [[ $# -lt 3 ]]; then
                log_error "benchmark requires: <bench_name> <type>"
                usage
                exit 1
            fi
            profile_benchmark "$2" "$3"
            ;;
        help|--help|-h)
            usage
            ;;
        *)
            log_error "Unknown command: $1"
            usage
            exit 1
            ;;
    esac
}

main "$@"
