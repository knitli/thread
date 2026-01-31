// SPDX-FileCopyrightText: 2026 Knitli Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! D1 Query Profiling Benchmarks
//!
//! Measures D1-related performance metrics and validates constitutional requirements.
//!
//! # Benchmark Coverage
//!
//! 1. SQL statement generation latency
//! 2. Cache lookup performance
//! 3. Performance metrics overhead
//! 4. Context creation overhead
//!
//! # Running Benchmarks
//!
//! ```bash
//! # All D1 profiling benchmarks
//! cargo bench --bench d1_profiling --features caching
//!
//! # Specific benchmark group
//! cargo bench --bench d1_profiling statement_generation
//! cargo bench --bench d1_profiling cache_operations
//! cargo bench --bench d1_profiling metrics_tracking
//! ```
//!
//! # Constitutional Compliance
//!
//! - Database p95 latency target: <50ms (D1)
//! - Cache hit rate target: >90%
//! - These benchmarks measure infrastructure overhead, not actual D1 API latency

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use recoco::base::schema::{BasicValueType, EnrichedValueType, FieldSchema, ValueType};
use recoco::base::value::{BasicValue, FieldValues, KeyPart, KeyValue};
use std::sync::Arc;
use std::time::Duration;
use thread_flow::monitoring::performance::PerformanceMetrics;
use thread_flow::targets::d1::D1ExportContext;

/// Helper to create test FieldSchema
fn test_field_schema(name: &str, value_type: BasicValueType, nullable: bool) -> FieldSchema {
    FieldSchema::new(
        name,
        EnrichedValueType {
            typ: ValueType::Basic(value_type),
            nullable,
            attrs: Default::default(),
        },
    )
}

/// Create a test D1 context for benchmarking
fn create_benchmark_context() -> D1ExportContext {
    let metrics = PerformanceMetrics::new();

    let key_schema = vec![
        test_field_schema("content_hash", BasicValueType::Str, false),
        test_field_schema("file_path", BasicValueType::Str, false),
    ];

    let value_schema = vec![
        test_field_schema("symbol_name", BasicValueType::Str, false),
        test_field_schema("symbol_type", BasicValueType::Str, false),
        test_field_schema("line_number", BasicValueType::Int64, false),
    ];

    D1ExportContext::new_with_default_client(
        "benchmark-database".to_string(),
        "code_symbols".to_string(),
        "benchmark-account".to_string(),
        "benchmark-token".to_string(),
        key_schema,
        value_schema,
        metrics,
    )
    .expect("Failed to create benchmark context")
}

/// Benchmark 1: SQL Statement Generation
///
/// Measures overhead of building UPSERT/DELETE SQL statements.
fn bench_statement_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("statement_generation");

    let context = create_benchmark_context();

    let test_key = KeyValue(Box::new([
        KeyPart::Str("abc123def456".into()),
        KeyPart::Str("src/main.rs".into()),
    ]));

    let test_values = FieldValues {
        fields: vec![
            recoco::base::value::Value::Basic(BasicValue::Str("main".into())),
            recoco::base::value::Value::Basic(BasicValue::Str("function".into())),
            recoco::base::value::Value::Basic(BasicValue::Int64(42)),
        ],
    };

    group.bench_function("build_upsert_statement", |b| {
        b.iter(|| {
            let _ = black_box(context.build_upsert_stmt(&test_key, &test_values));
        });
    });

    group.bench_function("build_delete_statement", |b| {
        b.iter(|| {
            let _ = black_box(context.build_delete_stmt(&test_key));
        });
    });

    // Benchmark batch statement generation
    group.bench_function("build_10_upsert_statements", |b| {
        let keys_values: Vec<_> = (0..10)
            .map(|i| {
                let key = KeyValue(Box::new([
                    KeyPart::Str(format!("hash{:08x}", i).into()),
                    KeyPart::Str(format!("src/file{}.rs", i).into()),
                ]));
                let values = test_values.clone();
                (key, values)
            })
            .collect();

        b.iter(|| {
            for (key, values) in &keys_values {
                let _ = black_box(context.build_upsert_stmt(key, values));
            }
        });
    });

    group.finish();
}

/// Benchmark 2: Cache Operations
///
/// Measures cache lookup and insertion performance.
#[cfg(feature = "caching")]
fn bench_cache_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_operations");

    let context = create_benchmark_context();
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Warm cache with entries
    runtime.block_on(async {
        for i in 0..100 {
            let key = format!("warm{:08x}", i);
            context
                .query_cache
                .insert(key, serde_json::json!({"value": i}))
                .await;
        }
    });

    group.bench_function("cache_hit_lookup", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let _ = black_box(context.query_cache.get(&"warm00000000".to_string()).await);
            });
        });
    });

    group.bench_function("cache_miss_lookup", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let _ = black_box(context.query_cache.get(&"nonexistent".to_string()).await);
            });
        });
    });

    group.bench_function("cache_insert", |b| {
        let mut counter = 0u64;
        b.iter(|| {
            runtime.block_on(async {
                let key = format!("insert{:016x}", counter);
                counter += 1;
                context
                    .query_cache
                    .insert(key, serde_json::json!({"value": counter}))
                    .await;
            });
        });
    });

    group.bench_function("cache_stats_retrieval", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let _ = black_box(context.cache_stats().await);
            });
        });
    });

    group.bench_function("cache_entry_count", |b| {
        b.iter(|| {
            let _ = black_box(context.query_cache.entry_count());
        });
    });

    group.finish();
}

/// Benchmark 3: Performance Metrics Tracking
///
/// Measures overhead of metrics collection.
fn bench_metrics_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("metrics_tracking");

    let metrics = PerformanceMetrics::new();

    group.bench_function("record_cache_hit", |b| {
        b.iter(|| {
            metrics.record_cache_hit();
        });
    });

    group.bench_function("record_cache_miss", |b| {
        b.iter(|| {
            metrics.record_cache_miss();
        });
    });

    group.bench_function("record_query_10ms", |b| {
        b.iter(|| {
            metrics.record_query(Duration::from_millis(10), true);
        });
    });

    group.bench_function("record_query_50ms", |b| {
        b.iter(|| {
            metrics.record_query(Duration::from_millis(50), true);
        });
    });

    group.bench_function("record_query_error", |b| {
        b.iter(|| {
            metrics.record_query(Duration::from_millis(100), false);
        });
    });

    group.bench_function("get_cache_stats", |b| {
        b.iter(|| {
            black_box(metrics.cache_stats());
        });
    });

    group.bench_function("get_query_stats", |b| {
        b.iter(|| {
            black_box(metrics.query_stats());
        });
    });

    group.bench_function("export_prometheus", |b| {
        b.iter(|| {
            black_box(metrics.export_prometheus());
        });
    });

    group.finish();
}

/// Benchmark 4: Context Creation Overhead
///
/// Measures D1ExportContext initialization performance.
fn bench_context_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("context_creation");

    let key_schema = vec![
        test_field_schema("content_hash", BasicValueType::Str, false),
        test_field_schema("file_path", BasicValueType::Str, false),
    ];

    let value_schema = vec![
        test_field_schema("symbol_name", BasicValueType::Str, false),
        test_field_schema("symbol_type", BasicValueType::Str, false),
        test_field_schema("line_number", BasicValueType::Int64, false),
    ];

    group.bench_function("create_d1_context", |b| {
        b.iter(|| {
            let metrics = PerformanceMetrics::new();
            let _ = black_box(D1ExportContext::new_with_default_client(
                "benchmark-database".to_string(),
                "code_symbols".to_string(),
                "benchmark-account".to_string(),
                "benchmark-token".to_string(),
                key_schema.clone(),
                value_schema.clone(),
                metrics,
            ));
        });
    });

    group.bench_function("create_performance_metrics", |b| {
        b.iter(|| {
            let _ = black_box(PerformanceMetrics::new());
        });
    });

    group.finish();
}

/// Benchmark 5: Value Conversion Performance
///
/// Measures JSON conversion overhead for D1 API calls.
fn bench_value_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_conversion");

    use thread_flow::targets::d1::{basic_value_to_json, key_part_to_json, value_to_json};

    let test_str_value = BasicValue::Str("test_string".into());
    let test_int_value = BasicValue::Int64(42);
    let test_bool_value = BasicValue::Bool(true);

    group.bench_function("basic_value_to_json_str", |b| {
        b.iter(|| {
            let _ = black_box(basic_value_to_json(&test_str_value));
        });
    });

    group.bench_function("basic_value_to_json_int", |b| {
        b.iter(|| {
            let _ = black_box(basic_value_to_json(&test_int_value));
        });
    });

    group.bench_function("basic_value_to_json_bool", |b| {
        b.iter(|| {
            let _ = black_box(basic_value_to_json(&test_bool_value));
        });
    });

    let test_key_part_str = KeyPart::Str("test_key".into());
    let test_key_part_int = KeyPart::Int64(123456);

    group.bench_function("key_part_to_json_str", |b| {
        b.iter(|| {
            let _ = black_box(key_part_to_json(&test_key_part_str));
        });
    });

    group.bench_function("key_part_to_json_int", |b| {
        b.iter(|| {
            let _ = black_box(key_part_to_json(&test_key_part_int));
        });
    });

    let test_value = recoco::base::value::Value::Basic(BasicValue::Str("test".into()));

    group.bench_function("value_to_json", |b| {
        b.iter(|| {
            let _ = black_box(value_to_json(&test_value));
        });
    });

    group.finish();
}

/// Benchmark 6: HTTP Connection Pool Performance
///
/// Validates connection pool efficiency from Task #59.
fn bench_http_pool_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("http_pool_performance");

    // Create shared HTTP client with connection pooling
    let http_client = Arc::new(
        reqwest::Client::builder()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Some(Duration::from_secs(90)))
            .tcp_keepalive(Some(Duration::from_secs(60)))
            .http2_keep_alive_interval(Some(Duration::from_secs(30)))
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client"),
    );

    // Benchmark context creation with shared client
    group.bench_function("create_context_with_shared_client", |b| {
        let metrics = PerformanceMetrics::new();
        let key_schema = vec![test_field_schema("id", BasicValueType::Int64, false)];
        let value_schema = vec![test_field_schema("data", BasicValueType::Str, false)];

        b.iter(|| {
            let client = Arc::clone(&http_client);
            let _ = black_box(D1ExportContext::new(
                "test-db".to_string(),
                "test_table".to_string(),
                "test-account".to_string(),
                "test-token".to_string(),
                client,
                key_schema.clone(),
                value_schema.clone(),
                metrics.clone(),
            ));
        });
    });

    // Benchmark Arc cloning overhead (should be negligible)
    group.bench_function("arc_clone_http_client", |b| {
        b.iter(|| {
            let _ = black_box(Arc::clone(&http_client));
        });
    });

    // Create 10 contexts sharing the same pool
    group.bench_function("create_10_contexts_shared_pool", |b| {
        b.iter(|| {
            let contexts: Vec<_> = (0..10)
                .map(|i| {
                    let metrics = PerformanceMetrics::new();
                    let key_schema = vec![test_field_schema("id", BasicValueType::Int64, false)];
                    let value_schema = vec![test_field_schema("data", BasicValueType::Str, false)];
                    let client = Arc::clone(&http_client);

                    D1ExportContext::new(
                        format!("db-{}", i),
                        format!("table_{}", i),
                        "account".to_string(),
                        "token".to_string(),
                        client,
                        key_schema,
                        value_schema,
                        metrics,
                    )
                    .expect("Failed to create context")
                })
                .collect();
            black_box(contexts)
        });
    });

    group.finish();
}

/// Benchmark 7: End-to-End Query Pipeline
///
/// Simulates complete D1 query pipeline with cache integration.
#[cfg(feature = "caching")]
fn bench_e2e_query_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("e2e_query_pipeline");

    let context = create_benchmark_context();
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Create test data
    let test_entries: Vec<_> = (0..100)
        .map(|i| {
            let key = KeyValue(Box::new([
                KeyPart::Str(format!("hash{:08x}", i).into()),
                KeyPart::Str(format!("src/file{}.rs", i).into()),
            ]));
            let values = FieldValues {
                fields: vec![
                    recoco::base::value::Value::Basic(BasicValue::Str(
                        format!("func_{}", i).into(),
                    )),
                    recoco::base::value::Value::Basic(BasicValue::Str("function".into())),
                    recoco::base::value::Value::Basic(BasicValue::Int64(i as i64)),
                ],
            };
            (key, values)
        })
        .collect();

    // Warm cache with all entries
    runtime.block_on(async {
        for (i, (key, values)) in test_entries.iter().enumerate() {
            let query_key = format!("query_{:08x}", i);
            let result = serde_json::json!({
                "key": format!("{:?}", key),
                "values": format!("{:?}", values),
            });
            context.query_cache.insert(query_key, result).await;
        }
    });

    // Benchmark: Cache hit path (optimal scenario)
    group.bench_function("pipeline_cache_hit_100_percent", |b| {
        let mut idx = 0;
        b.iter(|| {
            runtime.block_on(async {
                // 1. Check cache (should hit)
                let query_key = format!("query_{:08x}", idx % 100);
                let cached = context.query_cache.get(&query_key).await;
                black_box(cached);
                idx += 1;
            });
        });
    });

    // Benchmark: Cache miss path (worst case)
    group.bench_function("pipeline_cache_miss", |b| {
        let mut idx = 0;
        b.iter(|| {
            runtime.block_on(async {
                let (key, values) = &test_entries[idx % 100];

                // 1. Check cache (will miss)
                let query_key = format!("miss_{:08x}", idx);
                let cached = context.query_cache.get(&query_key).await;

                if cached.is_none() {
                    // 2. Build SQL statement
                    let stmt = context.build_upsert_stmt(key, values);
                    let _ = black_box(stmt);

                    // 3. Would execute HTTP request here (simulated)
                    // 4. Cache result
                    let result = serde_json::json!({"simulated": true});
                    context.query_cache.insert(query_key, result).await;
                }
                idx += 1;
            });
        });
    });

    // Benchmark: 90/10 cache hit/miss ratio (constitutional target)
    group.bench_function("pipeline_90_percent_cache_hit", |b| {
        let mut idx = 0;
        b.iter(|| {
            runtime.block_on(async {
                let (key, values) = &test_entries[idx % 100];

                // 90% of requests use cached queries, 10% are new
                let query_key = if idx % 10 == 0 {
                    format!("new_{:08x}", idx) // Cache miss (10%)
                } else {
                    format!("query_{:08x}", idx % 100) // Cache hit (90%)
                };

                let cached = context.query_cache.get(&query_key).await;

                if cached.is_none() {
                    let stmt = context.build_upsert_stmt(key, values);
                    let _ = black_box(stmt);
                    let result = serde_json::json!({"simulated": true});
                    context.query_cache.insert(query_key, result).await;
                }
                idx += 1;
            });
        });
    });

    group.finish();
}

/// Benchmark 8: Batch Operation Performance
///
/// Measures bulk operation efficiency for realistic workloads.
fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");

    let context = create_benchmark_context();

    // Create batch test data
    let batch_10: Vec<_> = (0..10).map(|i| create_test_entry(i)).collect();
    let batch_100: Vec<_> = (0..100).map(|i| create_test_entry(i)).collect();
    let batch_1000: Vec<_> = (0..1000).map(|i| create_test_entry(i)).collect();

    group.bench_function("batch_upsert_10_entries", |b| {
        b.iter(|| {
            for (key, values) in &batch_10 {
                let _ = black_box(context.build_upsert_stmt(key, values));
            }
        });
    });

    group.bench_function("batch_upsert_100_entries", |b| {
        b.iter(|| {
            for (key, values) in &batch_100 {
                let _ = black_box(context.build_upsert_stmt(key, values));
            }
        });
    });

    group.bench_function("batch_upsert_1000_entries", |b| {
        b.iter(|| {
            for (key, values) in &batch_1000 {
                let _ = black_box(context.build_upsert_stmt(key, values));
            }
        });
    });

    group.bench_function("batch_delete_10_entries", |b| {
        b.iter(|| {
            for (key, _) in &batch_10 {
                let _ = black_box(context.build_delete_stmt(key));
            }
        });
    });

    group.bench_function("batch_delete_100_entries", |b| {
        b.iter(|| {
            for (key, _) in &batch_100 {
                let _ = black_box(context.build_delete_stmt(key));
            }
        });
    });

    group.finish();
}

/// Helper function to create test entry
fn create_test_entry(idx: usize) -> (KeyValue, FieldValues) {
    let key = KeyValue(Box::new([
        KeyPart::Str(format!("hash{:08x}", idx).into()),
        KeyPart::Str(format!("src/file{}.rs", idx).into()),
    ]));
    let values = FieldValues {
        fields: vec![
            recoco::base::value::Value::Basic(BasicValue::Str(format!("symbol_{}", idx).into())),
            recoco::base::value::Value::Basic(BasicValue::Str("function".into())),
            recoco::base::value::Value::Basic(BasicValue::Int64(idx as i64)),
        ],
    };
    (key, values)
}

/// Benchmark 9: P95 Latency Validation
///
/// Validates constitutional requirement: D1 p95 latency <50ms
#[cfg(feature = "caching")]
fn bench_p95_latency_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("p95_latency_validation");
    group.sample_size(1000); // Larger sample for accurate p95 calculation

    let context = create_benchmark_context();
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Warm cache
    runtime.block_on(async {
        for i in 0..1000 {
            let query_key = format!("warm{:08x}", i);
            context
                .query_cache
                .insert(query_key, serde_json::json!({"value": i}))
                .await;
        }
    });

    // Simulate realistic workload: mostly cache hits with some misses
    group.bench_function("realistic_workload_p95", |b| {
        let mut idx = 0;
        b.iter(|| {
            runtime.block_on(async {
                // 95% cache hits, 5% misses (better than constitutional 90% target)
                let query_key = if idx % 20 == 0 {
                    format!("miss{:08x}", idx)
                } else {
                    format!("warm{:08x}", idx % 1000)
                };

                let result = context.query_cache.get(&query_key).await;

                if result.is_none() {
                    // Simulate query execution overhead
                    let (key, values) = create_test_entry(idx);
                    let stmt = context.build_upsert_stmt(&key, &values);
                    let _ = black_box(stmt);
                    context
                        .query_cache
                        .insert(query_key, serde_json::json!({"new": true}))
                        .await;
                }

                idx += 1;
            });
        });
    });

    group.finish();
}

// Benchmark groups
criterion_group!(
    benches,
    bench_statement_generation,
    bench_metrics_tracking,
    bench_context_creation,
    bench_value_conversion,
    bench_http_pool_performance,
    bench_batch_operations,
);

#[cfg(feature = "caching")]
criterion_group!(
    cache_benches,
    bench_cache_operations,
    bench_e2e_query_pipeline,
    bench_p95_latency_validation,
);

// Main benchmark runner
#[cfg(feature = "caching")]
criterion_main!(benches, cache_benches);

#[cfg(not(feature = "caching"))]
criterion_main!(benches);
