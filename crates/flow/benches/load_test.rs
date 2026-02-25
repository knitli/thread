// SPDX-FileCopyrightText: 2026 Knitli Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Load testing benchmarks for Thread
//!
//! Tests realistic workload scenarios including:
//! - Large codebase analysis (1000+ files)
//! - Concurrent query processing
//! - Cache hit/miss patterns
//! - Incremental updates
//! - Memory usage under load

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use std::time::Duration;
use thread_services::conversion::compute_content_fingerprint;

/// Generate synthetic code files for load testing
fn generate_synthetic_code(file_count: usize, lines_per_file: usize) -> Vec<String> {
    (0..file_count)
        .map(|file_idx| {
            let mut content = String::new();
            for line_idx in 0..lines_per_file {
                content.push_str(&format!(
                    "function file{}_func{}() {{\n",
                    file_idx, line_idx
                ));
                content.push_str(&format!("  return {};\n", file_idx * 1000 + line_idx));
                content.push_str("}\n\n");
            }
            content
        })
        .collect()
}

/// Benchmark fingerprinting large codebase
fn bench_large_codebase_fingerprinting(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_codebase_fingerprinting");
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(10));

    // Test various codebase sizes
    for file_count in [100, 500, 1000, 2000].iter() {
        let files = generate_synthetic_code(*file_count, 50);
        let total_bytes: usize = files.iter().map(|s| s.len()).sum();

        group.throughput(Throughput::Bytes(total_bytes as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_files", file_count)),
            file_count,
            |b, _| {
                b.iter(|| {
                    for file_content in &files {
                        black_box(compute_content_fingerprint(file_content));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent processing patterns
#[cfg(feature = "parallel")]
fn bench_concurrent_processing(c: &mut Criterion) {
    use rayon::prelude::*;
    use thread_flow::batch::process_files_batch;

    let mut group = c.benchmark_group("concurrent_processing");
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(10));

    let file_count = 1000;
    let files = generate_synthetic_code(file_count, 50);
    let file_paths: Vec<String> = (0..file_count).map(|i| format!("file_{}.rs", i)).collect();

    group.bench_function("sequential_fingerprinting", |b| {
        b.iter(|| {
            for file_content in &files {
                black_box(compute_content_fingerprint(file_content));
            }
        });
    });

    group.bench_function("parallel_fingerprinting", |b| {
        b.iter(|| {
            files.par_iter().for_each(|file_content| {
                black_box(compute_content_fingerprint(file_content));
            });
        });
    });

    group.bench_function("batch_processing", |b| {
        b.iter(|| {
            let results = process_files_batch(&file_paths, |_path| {
                // Simulate file processing
                Ok::<_, String>(())
            });
            black_box(results);
        });
    });

    group.finish();
}

/// Benchmark cache hit/miss patterns
#[cfg(feature = "caching")]
fn bench_cache_patterns(c: &mut Criterion) {
    use thread_flow::cache::{CacheConfig, QueryCache};

    let mut group = c.benchmark_group("cache_patterns");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(8));

    // Create cache with reasonable capacity
    let cache = QueryCache::<String, String>::new(CacheConfig {
        max_capacity: 1000,
        ttl_seconds: 300,
    });

    // Pre-populate cache with different hit rates
    let total_keys = 1000;
    let keys: Vec<String> = (0..total_keys).map(|i| format!("key_{}", i)).collect();
    let values: Vec<String> = (0..total_keys).map(|i| format!("value_{}", i)).collect();

    // Test different cache hit rates
    for hit_rate in [0, 25, 50, 75, 95, 100].iter() {
        let preload_count = (total_keys * hit_rate) / 100;

        // Pre-populate cache - use tokio runtime for async operations
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            for i in 0..preload_count {
                cache.insert(keys[i].clone(), values[i].clone()).await;
            }
        });

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}%_hit_rate", hit_rate)),
            hit_rate,
            |b, _| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let mut idx = 0;
                b.iter(|| {
                    rt.block_on(async {
                        let key = &keys[idx % total_keys];
                        if let Some(value) = cache.get(key).await {
                            black_box(value);
                        } else {
                            let value = values[idx % total_keys].clone();
                            cache.insert(key.clone(), value.clone()).await;
                            black_box(value);
                        }
                        idx += 1;
                    });
                });
            },
        );
    }

    group.finish();
}

/// Benchmark incremental update patterns
fn bench_incremental_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_updates");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(8));

    let file_count = 1000;
    let files = generate_synthetic_code(file_count, 50);

    // Pre-compute all fingerprints
    let fingerprints: Vec<_> = files
        .iter()
        .map(|content| compute_content_fingerprint(content))
        .collect();

    // Simulate different change patterns
    for change_rate in [1, 5, 10, 25, 50].iter() {
        let changed_count = (file_count * change_rate) / 100;

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}%_changed", change_rate)),
            change_rate,
            |b, _| {
                b.iter(|| {
                    // Only recompute fingerprints for changed files
                    for i in 0..changed_count {
                        black_box(compute_content_fingerprint(&files[i]));
                    }
                    // Reuse cached fingerprints for unchanged files
                    for i in changed_count..file_count {
                        black_box(fingerprints[i]);
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory usage patterns
fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(8));

    // Test different file sizes
    for file_size_kb in [1, 10, 100, 500].iter() {
        let lines_per_file = (file_size_kb * 1024) / 100; // ~100 bytes per line
        let files = generate_synthetic_code(100, lines_per_file);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}KB_files", file_size_kb)),
            file_size_kb,
            |b, _| {
                b.iter(|| {
                    for file_content in &files {
                        black_box(compute_content_fingerprint(file_content));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark realistic workload scenarios
fn bench_realistic_workloads(c: &mut Criterion) {
    let mut group = c.benchmark_group("realistic_workloads");
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(10));

    // Small project: 50 files, ~100 lines each
    group.bench_function("small_project_50_files", |b| {
        let files = generate_synthetic_code(50, 100);
        b.iter(|| {
            for file_content in &files {
                black_box(compute_content_fingerprint(file_content));
            }
        });
    });

    // Medium project: 500 files, ~200 lines each
    group.bench_function("medium_project_500_files", |b| {
        let files = generate_synthetic_code(500, 200);
        b.iter(|| {
            for file_content in &files {
                black_box(compute_content_fingerprint(file_content));
            }
        });
    });

    // Large project: 2000 files, ~300 lines each
    group.bench_function("large_project_2000_files", |b| {
        let files = generate_synthetic_code(2000, 300);
        b.iter(|| {
            for file_content in &files {
                black_box(compute_content_fingerprint(file_content));
            }
        });
    });

    group.finish();
}

/// Benchmark AST parsing throughput
fn bench_ast_parsing(c: &mut Criterion) {
    use thread_ast_engine::tree_sitter::LanguageExt;
    use thread_language::Rust;

    let mut group = c.benchmark_group("ast_parsing");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(8));

    // Test parsing small to large files
    let small_code = generate_synthetic_code(1, 50)[0].clone();
    let medium_code = generate_synthetic_code(1, 200)[0].clone();
    let large_code = generate_synthetic_code(1, 500)[0].clone();

    group.throughput(Throughput::Bytes(small_code.len() as u64));
    group.bench_function("parse_small_file", |b| {
        b.iter(|| {
            black_box(Rust.ast_grep(&small_code));
        });
    });

    group.throughput(Throughput::Bytes(medium_code.len() as u64));
    group.bench_function("parse_medium_file", |b| {
        b.iter(|| {
            black_box(Rust.ast_grep(&medium_code));
        });
    });

    group.throughput(Throughput::Bytes(large_code.len() as u64));
    group.bench_function("parse_large_file", |b| {
        b.iter(|| {
            black_box(Rust.ast_grep(&large_code));
        });
    });

    // Batch parsing throughput
    let batch_files = generate_synthetic_code(100, 100);
    let total_bytes: usize = batch_files.iter().map(|s| s.len()).sum();
    group.throughput(Throughput::Bytes(total_bytes as u64));
    group.bench_function("parse_batch_100_files", |b| {
        b.iter(|| {
            for code in &batch_files {
                black_box(Rust.ast_grep(code));
            }
        });
    });

    group.finish();
}

/// Benchmark rule matching performance
fn bench_rule_matching(c: &mut Criterion) {
    use thread_ast_engine::tree_sitter::LanguageExt;
    use thread_language::Rust;

    let mut group = c.benchmark_group("rule_matching");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(8));

    let test_code = r#"
        fn test_function() {
            let x = 42;
            let y = "hello";
            println!("{}", x);
        }
        fn another_function(param: i32) -> i32 {
            param * 2
        }
    "#;

    let root = Rust.ast_grep(test_code);

    // Simple pattern matching
    group.bench_function("match_simple_pattern", |b| {
        let pattern = "let $VAR = $VALUE";
        b.iter(|| {
            black_box(root.root().find_all(pattern).count());
        });
    });

    // Complex pattern matching
    group.bench_function("match_complex_pattern", |b| {
        let pattern = "fn $NAME($$$PARAMS) { $$$BODY }";
        b.iter(|| {
            black_box(root.root().find_all(pattern).count());
        });
    });

    // Pattern with meta-variables
    group.bench_function("match_with_metavars", |b| {
        let pattern = "println!($$$ARGS)";
        b.iter(|| {
            black_box(root.root().find_all(pattern).count());
        });
    });

    // Multiple patterns (rule with constraints)
    group.bench_function("match_multiple_patterns", |b| {
        b.iter(|| {
            let count1 = root.root().find_all("let $VAR = $VALUE").count();
            let count2 = root.root().find_all("fn $NAME($$$PARAMS)").count();
            black_box(count1 + count2);
        });
    });

    group.finish();
}

/// Benchmark pattern compilation and caching
fn bench_pattern_compilation(c: &mut Criterion) {
    use thread_ast_engine::tree_sitter::LanguageExt;
    use thread_language::Rust;

    let mut group = c.benchmark_group("pattern_compilation");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(8));

    let patterns = vec![
        "let $VAR = $VALUE",
        "fn $NAME($$$PARAMS) { $$$BODY }",
        "struct $NAME { $$$FIELDS }",
        "impl $NAME { $$$METHODS }",
        "use $$$PATH",
    ];

    // Pattern compilation time
    group.bench_function("compile_single_pattern", |b| {
        b.iter(|| {
            let test_code = "let x = 42;";
            let root = Rust.ast_grep(test_code);
            black_box(root.root().find("let $VAR = $VALUE"));
        });
    });

    // Multiple pattern compilation
    group.bench_function("compile_multiple_patterns", |b| {
        b.iter(|| {
            let test_code = "fn test() { let x = 42; }";
            let root = Rust.ast_grep(test_code);
            for pattern in &patterns {
                black_box(root.root().find(pattern));
            }
        });
    });

    // Pattern reuse (simulates caching benefit)
    group.bench_function("pattern_reuse", |b| {
        let test_codes = generate_synthetic_code(10, 20);
        b.iter(|| {
            for code in &test_codes {
                let root = Rust.ast_grep(code);
                // Reuse same pattern across files
                black_box(root.root().find_all("function $NAME($$$PARAMS)").count());
            }
        });
    });

    group.finish();
}

// Configure criterion groups
criterion_group! {
    name = load_tests;
    config = Criterion::default()
        .sample_size(50)
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(10));
    targets =
        bench_large_codebase_fingerprinting,
        bench_incremental_updates,
        bench_memory_patterns,
        bench_realistic_workloads,
        bench_ast_parsing,
        bench_rule_matching,
        bench_pattern_compilation
}

// Add parallel benchmarks if feature enabled
#[cfg(feature = "parallel")]
criterion_group! {
    name = parallel_tests;
    config = Criterion::default()
        .sample_size(50);
    targets = bench_concurrent_processing
}

// Add cache benchmarks if feature enabled
#[cfg(feature = "caching")]
criterion_group! {
    name = cache_tests;
    config = Criterion::default()
        .sample_size(50);
    targets = bench_cache_patterns
}

// Main criterion entry point with conditional groups
#[cfg(all(feature = "parallel", feature = "caching"))]
criterion_main!(load_tests, parallel_tests, cache_tests);

#[cfg(all(feature = "parallel", not(feature = "caching")))]
criterion_main!(load_tests, parallel_tests);

#[cfg(all(not(feature = "parallel"), feature = "caching"))]
criterion_main!(load_tests, cache_tests);

#[cfg(all(not(feature = "parallel"), not(feature = "caching")))]
criterion_main!(load_tests);
