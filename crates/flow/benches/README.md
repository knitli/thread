<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# thread-flow Benchmarks

Performance benchmarks for the thread-flow crate measuring parsing performance and overhead analysis.

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench -p thread-flow

# Run specific benchmark group
cargo bench -p thread-flow -- direct_parse
cargo bench -p thread-flow -- multi_file
cargo bench -p thread-flow -- language_comparison

# Run with quick sampling (faster, less precise)
cargo bench -p thread-flow -- --quick

# Save baseline for comparison
cargo bench -p thread-flow -- --save-baseline main

# Compare against baseline
cargo bench -p thread-flow -- --baseline main
```

## Benchmark Categories

### 1. Direct Parse Benchmarks
Measures baseline Thread AST parsing performance without ReCoco overhead.

- **rust_small_50_lines**: ~140µs (7 Kfiles/s)
- **rust_medium_200_lines**: ~730µs (1.4 Kfiles/s)
- **rust_large_500_lines**: ~1.4ms (700 files/s)

**Throughput**: ~5-6 MiB/s across file sizes

### 2. Multi-File Batch Processing
Sequential processing of multiple files to measure sustained performance.

- **sequential_10_small_files**: ~1.6ms total (~160µs per file)
- **sequential_10_mixed_files**: ~6ms total (mixed small/medium/large)

**Performance**: Maintains ~5 MiB/s throughput across batch operations

### 3. Language Comparison
Parsing performance across different programming languages.

- **Rust**: ~140µs
- **Python**: ~100µs (faster due to simpler syntax)
- **TypeScript**: ~85µs (faster due to simpler syntax)

### 4. Throughput Metrics
Files processed per second for different file sizes.

- **Small files (50 lines)**: ~7K files/second
- **Medium files (200 lines)**: ~1.4K files/second
- **Large files (500+ lines)**: ~700 files/second

## Performance Baselines

Current performance targets (all met):

- ✅ Small file (50 lines): <500µs (achieved: ~140µs)
- ✅ Medium file (200 lines): <2ms (achieved: ~730µs)
- ✅ Large file (500+ lines): <10ms (achieved: ~1.4ms)
- ✅ Multi-file (10 files): <50ms total (achieved: ~6ms for mixed sizes)

## Interpreting Results

### Time Measurements
- **time**: Average time per iteration with confidence interval
- Lower is better
- Includes parsing, AST construction, and basic operations

### Throughput Measurements
- **thrpt (MiB/s)**: Megabytes of source code per second
- **thrpt (Kelem/s)**: Thousands of files per second
- Higher is better

### Variance
- Small variance indicates stable performance
- Large variance may indicate GC pauses, cache effects, or system noise

## Future Benchmark Plans

### ReCoco Integration Benchmarks (TODO)
Currently disabled due to metadata extraction bugs. Will add:

- Full pipeline with ReCoco executor
- Content-addressed caching performance
- Cache hit/miss scenarios
- Memory usage comparison

### Additional Metrics (TODO)
- Peak memory usage per file size
- Parallel processing benchmarks (rayon)
- Async processing benchmarks (tokio)
- Edge deployment benchmarks (WASM)

## Benchmark Data

Test data is generated programmatically to ensure consistency:

- **Small files**: ~50 lines with basic structs, functions, tests
- **Medium files**: ~200 lines with business logic, error handling, multiple types
- **Large files**: ~500+ lines with extensive trait implementations, enums, patterns

All test data uses realistic Rust code patterns to ensure representative performance measurements.

## Notes

- Benchmarks run in `--release` mode with full optimizations
- Uses criterion.rs for statistical analysis
- Results may vary based on CPU, memory, and system load
- Baseline measurements taken on development machine (see CI for reproducible benchmarks)

## Troubleshooting

If benchmarks fail to compile:
```bash
cargo clean -p thread-flow
cargo build -p thread-flow --benches
```

If benchmarks are too slow:
```bash
# Use quick sampling
cargo bench -p thread-flow -- --quick

# Or reduce sample size
cargo bench -p thread-flow -- --sample-size 10
```
