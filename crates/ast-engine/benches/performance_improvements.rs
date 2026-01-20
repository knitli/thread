// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Benchmarks for performance improvements in ast-engine crate
//!
//! Run with: cargo bench --package thread-ast-engine

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use thread_ast_engine::{Pattern, Root};
use thread_language::Tsx;
use thread_utils::RapidMap;

fn bench_pattern_conversion(c: &mut Criterion) {
    let source_code = r#"
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
    "#;

    let pattern_str = "function $NAME($$$ARGS) { $$$BODY }";

    c.bench_function("pattern_conversion_optimized", |b| {
        b.iter(|| {
            let pattern = Pattern::new(black_box(pattern_str), &Tsx);
            let root = Root::str(black_box(source_code), Tsx);
            let node = root.root();
            let matches: Vec<_> = node.find_all(&pattern).collect();
            black_box(matches.len())
        })
    });
}

fn bench_meta_var_env_conversion(c: &mut Criterion) {
    let source_code = "const value = 123; const another = 456; const third = 789;";
    let pattern_str = "const $VAR = $VALUE";

    c.bench_function("meta_var_env_conversion", |b| {
        b.iter(|| {
            let pattern = Pattern::new(black_box(pattern_str), &Tsx);
            let root = Root::str(black_box(source_code), &Tsx);
            let matches: Vec<_> = root.root().find_all(&pattern).collect();

            // Test the optimized string concatenation
            for m in matches {
                let env_map = RapidMap::from(m.get_env().clone());
                black_box(env_map);
            }
        })
    });
}

fn bench_pattern_children_collection(c: &mut Criterion) {
    let source_code = r#"
        class TestClass {
            method1() { return 1; }
            method2() { return 2; }
            method3() { return 3; }
            method4() { return 4; }
            method5() { return 5; }
        }
    "#;

    c.bench_function("pattern_children_collection", |b| {
        b.iter(|| {
            let root = Root::str(black_box(source_code), Tsx);
            let pattern = Pattern::new("class $NAME { $$$METHODS }", &Tsx);
            let matches: Vec<_> = root.root().find_all(&pattern).collect();
            black_box(matches);
        })
    });
}

criterion_group!(
    benches,
    bench_pattern_conversion,
    bench_meta_var_env_conversion,
    bench_pattern_children_collection
);
criterion_main!(benches);
