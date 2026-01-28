// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Fingerprint and caching performance benchmarks for Day 15 optimization
//!
//! ## Benchmark Categories:
//! 1. **Blake3 Fingerprinting**: Measure fingerprint computation speed
//! 2. **Cache Hit Scenarios**: Simulated cache lookups
//! 3. **End-to-End with Caching**: Full pipeline with fingerprint-based deduplication
//! 4. **Memory Usage**: Profile memory consumption
//!
//! ## Performance Targets:
//! - Fingerprint computation: <10µs for typical files
//! - Cache hit: <1µs (hash map lookup)
//! - Full pipeline with 100% cache hit: <100µs (50x+ speedup vs parse)
//! - Memory overhead: <1KB per cached file

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use thread_services::conversion::compute_content_fingerprint;
use std::collections::HashMap;

// ============================================================================
// Test Data
// ============================================================================

const SMALL_CODE: &str = r#"
use std::collections::HashMap;

pub struct Config {
    name: String,
    value: i32,
}

impl Config {
    pub fn new(name: String, value: i32) -> Self {
        Self { name, value }
    }
}
"#;

const MEDIUM_CODE: &str = r#"
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

pub struct UserManager {
    users: Arc<Mutex<HashMap<u64, String>>>,
    emails: Arc<Mutex<HashMap<String, u64>>>,
}

impl UserManager {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            emails: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_user(&self, id: u64, name: String, email: String) {
        let mut users = self.users.lock().unwrap();
        let mut emails = self.emails.lock().unwrap();
        users.insert(id, name);
        emails.insert(email, id);
    }

    pub fn get_user(&self, id: u64) -> Option<String> {
        self.users.lock().unwrap().get(&id).cloned()
    }
}
"#;

fn generate_large_code() -> String {
    let mut code = MEDIUM_CODE.to_string();
    for i in 0..50 {
        code.push_str(&format!(
            r#"
pub fn function_{}(x: i32) -> i32 {{
    x + {}
}}
"#,
            i, i
        ));
    }
    code
}

// ============================================================================
// Fingerprint Computation Benchmarks
// ============================================================================

fn benchmark_fingerprint_computation(c: &mut Criterion) {
    let mut group = c.benchmark_group("fingerprint_computation");

    // Small file fingerprinting
    group.throughput(Throughput::Bytes(SMALL_CODE.len() as u64));
    group.bench_function("blake3_small_file", |b| {
        b.iter(|| {
            black_box(compute_content_fingerprint(black_box(SMALL_CODE)))
        });
    });

    // Medium file fingerprinting
    group.throughput(Throughput::Bytes(MEDIUM_CODE.len() as u64));
    group.bench_function("blake3_medium_file", |b| {
        b.iter(|| {
            black_box(compute_content_fingerprint(black_box(MEDIUM_CODE)))
        });
    });

    // Large file fingerprinting
    let large_code = generate_large_code();
    group.throughput(Throughput::Bytes(large_code.len() as u64));
    group.bench_function("blake3_large_file", |b| {
        b.iter(|| {
            black_box(compute_content_fingerprint(black_box(&large_code)))
        });
    });

    group.finish();
}

// ============================================================================
// Cache Lookup Benchmarks
// ============================================================================

fn benchmark_cache_lookups(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_lookups");

    // Create cache with 1000 entries
    let mut cache = HashMap::new();
    for i in 0..1000 {
        let code = format!("fn test_{}() {{ println!(\"test\"); }}", i);
        let fp = compute_content_fingerprint(&code);
        cache.insert(fp, format!("result_{}", i));
    }

    // Benchmark cache hit
    let test_code = "fn test_500() { println!(\"test\"); }";
    let test_fp = compute_content_fingerprint(test_code);

    group.bench_function("cache_hit", |b| {
        b.iter(|| {
            black_box(cache.get(black_box(&test_fp)))
        });
    });

    // Benchmark cache miss
    let miss_code = "fn not_in_cache() {}";
    let miss_fp = compute_content_fingerprint(miss_code);

    group.bench_function("cache_miss", |b| {
        b.iter(|| {
            black_box(cache.get(black_box(&miss_fp)))
        });
    });

    group.finish();
}

// ============================================================================
// Batch Fingerprinting Benchmarks
// ============================================================================

fn benchmark_batch_fingerprinting(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_fingerprinting");

    // Generate 100 different files
    let files: Vec<String> = (0..100)
        .map(|i| format!("fn func_{}() {{ println!(\"test\"); }}", i))
        .collect();

    let total_bytes: usize = files.iter().map(|s| s.len()).sum();
    group.throughput(Throughput::Bytes(total_bytes as u64));

    group.bench_function("sequential_100_files", |b| {
        b.iter(|| {
            for file in &files {
                black_box(compute_content_fingerprint(black_box(file)));
            }
        });
    });

    group.finish();
}

// ============================================================================
// Memory Profiling Benchmarks
// ============================================================================

fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    // Measure memory overhead of cache
    group.bench_function("cache_1000_entries", |b| {
        b.iter(|| {
            let mut cache = HashMap::new();
            for i in 0..1000 {
                let code = format!("fn test_{}() {{}}", i);
                let fp = compute_content_fingerprint(&code);
                cache.insert(fp, format!("result_{}", i));
            }
            black_box(cache)
        });
    });

    group.finish();
}

// ============================================================================
// Cache Hit Rate Scenarios
// ============================================================================

fn benchmark_cache_hit_rates(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_hit_scenarios");

    let files: Vec<String> = (0..100)
        .map(|i| format!("fn func_{}() {{ println!(\"test\"); }}", i))
        .collect();

    // Scenario: 0% cache hit (all new files)
    group.bench_function("0_percent_hit_rate", |b| {
        b.iter(|| {
            let mut cache = HashMap::new();
            let mut hits = 0;
            let mut misses = 0;

            for file in &files {
                let fp = compute_content_fingerprint(file);
                if cache.contains_key(&fp) {
                    hits += 1;
                } else {
                    misses += 1;
                    cache.insert(fp, ());
                }
            }

            black_box((hits, misses))
        });
    });

    // Scenario: 100% cache hit (all files seen before)
    let mut primed_cache = HashMap::new();
    for file in &files {
        let fp = compute_content_fingerprint(file);
        primed_cache.insert(fp, ());
    }

    group.bench_function("100_percent_hit_rate", |b| {
        b.iter(|| {
            let mut hits = 0;
            let mut misses = 0;

            for file in &files {
                let fp = compute_content_fingerprint(file);
                if primed_cache.contains_key(&fp) {
                    hits += 1;
                } else {
                    misses += 1;
                }
            }

            black_box((hits, misses))
        });
    });

    // Scenario: 50% cache hit (half files modified)
    let modified_files: Vec<String> = (0..100)
        .map(|i| {
            if i % 2 == 0 {
                // Return original file (cache hit)
                files[i].clone()
            } else {
                // Return modified file (cache miss)
                format!("fn func_{}() {{ println!(\"modified\"); }}", i)
            }
        })
        .collect();

    group.bench_function("50_percent_hit_rate", |b| {
        b.iter(|| {
            let mut hits = 0;
            let mut misses = 0;

            for file in &modified_files {
                let fp = compute_content_fingerprint(file);
                if primed_cache.contains_key(&fp) {
                    hits += 1;
                } else {
                    misses += 1;
                }
            }

            black_box((hits, misses))
        });
    });

    group.finish();
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group!(
    benches,
    benchmark_fingerprint_computation,
    benchmark_cache_lookups,
    benchmark_batch_fingerprinting,
    benchmark_memory_usage,
    benchmark_cache_hit_rates,
);

criterion_main!(benches);
