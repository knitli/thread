// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Benchmarks for performance improvements in ast-engine crate
//!
//! Run with: cargo bench --package thread-ast-engine
//!
//! Key optimizations measured:
//! - Pattern compilation cache: thread-local cache avoids re-parsing patterns
//! - Arc<str> interning: `MetaVariableID` uses Arc<str> to reduce clone costs
//! - `MetaVarEnv` operations: allocation behavior of the matching environment

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use thread_ast_engine::{Pattern, Root};
use thread_language::Tsx;
use thread_utils::RapidMap;

fn bench_pattern_conversion(c: &mut Criterion) {
    let source_code = r"
        function complexFunction(a, b, c) {
            if (a > b) {
                return c.map(x => x * 2).filter(x => x > 10);
            } else {
                const result = [];
                for (let i = 0; i < c.length; i++) {
                    if (c[i] % 2 === 0) {
                        result.push(c[i] * 3);
                    }
                }
                return result;
            }
        }
    ";

    let pattern_str = "function $NAME($$$ARGS) { $$$BODY }";

    c.bench_function("pattern_conversion_optimized", |b| {
        b.iter(|| {
            let pattern = Pattern::new(black_box(pattern_str), &Tsx);
            let root = Root::str(black_box(source_code), Tsx);
            let node = root.root();
            black_box(node.find_all(&pattern).count())
        });
    });
}

fn bench_meta_var_env_conversion(c: &mut Criterion) {
    let source_code = "const value = 123; const another = 456; const third = 789;";
    let pattern_str = "const $VAR = $VALUE";

    c.bench_function("meta_var_env_conversion", |b| {
        b.iter(|| {
            let pattern = Pattern::new(black_box(pattern_str), &Tsx);
            let root = Root::str(black_box(source_code), Tsx);
            let matches: Vec<_> = root.root().find_all(&pattern).collect();

            // Test the optimized string concatenation
            for m in matches {
                let env_map: RapidMap<String, String> = RapidMap::from(m.get_env().clone());
                black_box(env_map);
            }
        });
    });
}

fn bench_pattern_children_collection(c: &mut Criterion) {
    let source_code = r"
        class TestClass {
            method1() { return 1; }
            method2() { return 2; }
            method3() { return 3; }
            method4() { return 4; }
            method5() { return 5; }
        }
    ";

    c.bench_function("pattern_children_collection", |b| {
        b.iter(|| {
            let root = Root::str(black_box(source_code), Tsx);
            let pattern = Pattern::new("class $NAME { $$$METHODS }", &Tsx);
            let matches: Vec<_> = root.root().find_all(&pattern).collect();
            black_box(matches);
        });
    });
}

/// Benchmark: Pattern cache hit performance.
///
/// This measures the speedup from the thread-local pattern compilation cache.
/// When the same pattern string is used repeatedly (typical in rule scanning),
/// subsequent calls avoid re-parsing via tree-sitter.
fn bench_pattern_cache_hit(c: &mut Criterion) {
    let source_code = "let x = 42; let y = 100; let z = 200;";
    let pattern_str = "let $VAR = $VALUE";

    let mut group = c.benchmark_group("pattern_cache");

    // Warm up the cache by matching once
    group.bench_function("first_match_cold_cache", |b| {
        b.iter(|| {
            let root = Root::str(black_box(source_code), Tsx);
            let node = root.root();
            // Using &str triggers `impl Matcher for str` which uses the cache
            let found = node.find(black_box(pattern_str));
            black_box(found.is_some())
        });
    });

    // Measure repeated matching - the pattern cache should provide large speedup
    group.bench_function("repeated_match_warm_cache", |b| {
        // Warm the cache
        {
            let root = Root::str(source_code, Tsx);
            let _ = root.root().find(pattern_str);
        }
        b.iter(|| {
            let root = Root::str(black_box(source_code), Tsx);
            let node = root.root();
            let found = node.find(black_box(pattern_str));
            black_box(found.is_some())
        });
    });

    // Compare with pre-compiled pattern (no cache overhead at all)
    group.bench_function("precompiled_pattern", |b| {
        let pattern = Pattern::new(pattern_str, &Tsx);
        b.iter(|| {
            let root = Root::str(black_box(source_code), Tsx);
            let node = root.root();
            let found = node.find(&pattern);
            black_box(found.is_some())
        });
    });

    group.finish();
}

/// Benchmark: `MetaVarEnv` clone cost with Arc<str> keys.
///
/// Arc<str> cloning is a single atomic increment (~1ns) vs `String::clone`
/// which copies the entire buffer. This benchmark measures the env clone
/// overhead in the pattern matching hot path.
fn bench_env_clone_cost(c: &mut Criterion) {
    let source_code = r"
        function foo(a, b, c, d, e) {
            return a + b + c + d + e;
        }
    ";
    let pattern_str = "function $NAME($$$PARAMS) { $$$BODY }";

    c.bench_function("env_clone_with_arc_str", |b| {
        let pattern = Pattern::new(pattern_str, &Tsx);
        let root = Root::str(source_code, Tsx);
        let matches: Vec<_> = root.root().find_all(&pattern).collect();
        assert!(!matches.is_empty(), "should have at least one match");

        b.iter(|| {
            for m in &matches {
                let cloned = m.get_env().clone();
                black_box(cloned);
            }
        });
    });
}

/// Benchmark: Multiple patterns on the same source (rule scanning scenario).
///
/// This simulates a real-world scenario where multiple rules are applied
/// to the same source code, demonstrating the value of per-pattern caching.
fn bench_multi_pattern_scanning(c: &mut Criterion) {
    let source_code = r#"
        const x = 42;
        let y = "hello";
        var z = true;
        function foo() { return x; }
        class Bar { constructor() { this.x = 1; } }
    "#;

    let patterns = [
        "const $VAR = $VALUE",
        "let $VAR = $VALUE",
        "var $VAR = $VALUE",
        "function $NAME() { $$$BODY }",
        "class $NAME { $$$BODY }",
    ];

    c.bench_function("multi_pattern_scan", |b| {
        let compiled: Vec<_> = patterns.iter().map(|p| Pattern::new(p, &Tsx)).collect();
        b.iter(|| {
            let root = Root::str(black_box(source_code), Tsx);
            let node = root.root();
            let mut total = 0usize;
            for pattern in &compiled {
                total += node.find_all(pattern).count();
            }
            black_box(total)
        });
    });
}

criterion_group!(
    benches,
    bench_pattern_conversion,
    bench_meta_var_env_conversion,
    bench_pattern_children_collection,
    bench_pattern_cache_hit,
    bench_env_clone_cost,
    bench_multi_pattern_scanning,
);
criterion_main!(benches);
