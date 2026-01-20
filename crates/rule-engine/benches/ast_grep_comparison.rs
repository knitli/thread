// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
// SPDX-License-Identifier: MIT OR Apache-2.0

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;

// Thread imports
use thread_language::{LanguageExt as ThreadLanguageExt, SupportLang as ThreadSupportLang};
use thread_rule_engine::{
    CombinedScan as ThreadCombinedScan, GlobalRules as ThreadGlobalRules,
    from_yaml_string as thread_from_yaml_string,
};

// AstGrep imports
use ast_grep_config::{
    CombinedScan as AstGrepCombinedScan, GlobalRules as AstGrepGlobalRules,
    from_yaml_string as ast_grep_from_yaml_string,
};
use ast_grep_language::{LanguageExt as AstGrepLanguageExt, SupportLang as AstGrepSupportLang};

struct ComparisonData {
    rules: Vec<&'static str>,
    test_code: &'static str,
}

impl ComparisonData {
    fn new() -> Self {
        Self {
            rules: vec![
                r#"
id: simple-console-log
message: found console.log
severity: info
language: TypeScript
rule:
  pattern: console.log($A)
"#,
                r#"
id: function-declaration
message: found function declaration
severity: info
language: TypeScript
rule:
  pattern: function $F($$$) { $$$ }
"#, /*
                                                                                                                                                                                                                        r#"
id: class-with-constructor
message: found class with constructor
severity: info
language: TypeScript
rule:
  all:
    - pattern: class $C { $$$ }
    - has:
        pattern: constructor_type($$$) { $$$ }
        stopBy: end
"#,*/
                r#"
id: import-statement
message: found import statement
severity: info
language: TypeScript
rule:
  any:
    - pattern: import $M from '$P'
    - pattern: import { $$$ } from '$P'
    - pattern: import * as $M from '$P'
"#,
                r#"
id: async-function-with-await
message: found async function with await
severity: info
language: TypeScript
rule:
  all:
    - pattern: async function $F($$$) { $$$ }
    - has:
        pattern: await $E
        stopBy: end
"#,
            ],
            test_code: include_str!("../test_data/sample_typescript.ts"),
        }
    }
}

fn bench_rule_parsing_comparison(c: &mut Criterion) {
    let data = ComparisonData::new();
    let mut group = c.benchmark_group("rule_parsing_comparison");

    for (rule_idx, rule_yaml) in data.rules.iter().enumerate() {
        // Benchmark thread-rule-engine
        group.bench_with_input(
            BenchmarkId::new("thread_rule_engine", rule_idx),
            rule_yaml,
            |b, yaml| {
                let globals = ThreadGlobalRules::default();
                b.iter(|| {
                    let _rules =
                        thread_from_yaml_string::<ThreadSupportLang>(black_box(yaml), &globals)
                            .expect("should parse");
                });
            },
        );

        // Benchmark ast-grep-config
        group.bench_with_input(
            BenchmarkId::new("ast_grep_config", rule_idx),
            rule_yaml,
            |b, yaml| {
                let globals = AstGrepGlobalRules::default();
                b.iter(|| {
                    let _rules =
                        ast_grep_from_yaml_string::<AstGrepSupportLang>(black_box(yaml), &globals)
                            .expect("should parse");
                });
            },
        );
    }

    group.finish();
}

fn bench_rule_matching_comparison(c: &mut Criterion) {
    let data = ComparisonData::new();
    let mut group = c.benchmark_group("rule_matching_comparison");

    let test_rule = r#"
id: test-console-log
message: found console.log
severity: info
language: TypeScript
rule:
  pattern: console.log($A)
"#;

    // Prepare rules for both libraries
    let thread_globals = ThreadGlobalRules::default();
    let ast_grep_globals = AstGrepGlobalRules::default();

    let thread_rules = thread_from_yaml_string::<ThreadSupportLang>(test_rule, &thread_globals)
        .expect("should parse");
    let ast_grep_rules =
        ast_grep_from_yaml_string::<AstGrepSupportLang>(test_rule, &ast_grep_globals)
            .expect("should parse");

    let thread_grep = ThreadSupportLang::TypeScript.ast_grep(data.test_code);
    let ast_grep_grep = AstGrepSupportLang::TypeScript.ast_grep(data.test_code);

    // Benchmark thread-rule-engine
    group.bench_function("thread_rule_engine", |b| {
        b.iter(|| {
            let matches: Vec<_> = thread_grep
                .root()
                .find_all(&thread_rules[0].matcher)
                .collect();
            black_box(matches);
        });
    });

    // Benchmark ast-grep-config
    group.bench_function("ast_grep_config", |b| {
        b.iter(|| {
            let matches: Vec<_> = ast_grep_grep
                .root()
                .find_all(&ast_grep_rules[0].matcher)
                .collect();
            black_box(matches);
        });
    });

    group.finish();
}

fn bench_combined_scan_comparison(c: &mut Criterion) {
    let data = ComparisonData::new();
    let mut group = c.benchmark_group("combined_scan_comparison");

    // Prepare rules for both libraries
    let thread_globals = ThreadGlobalRules::default();
    let ast_grep_globals = AstGrepGlobalRules::default();

    let mut thread_rules = Vec::new();
    let mut ast_grep_rules = Vec::new();

    for rule_yaml in &data.rules {
        let thread_rule = thread_from_yaml_string::<ThreadSupportLang>(rule_yaml, &thread_globals)
            .expect("should parse")
            .into_iter()
            .next()
            .unwrap();
        let ast_grep_rule =
            ast_grep_from_yaml_string::<AstGrepSupportLang>(rule_yaml, &ast_grep_globals)
                .expect("should parse")
                .into_iter()
                .next()
                .unwrap();

        thread_rules.push(thread_rule);
        ast_grep_rules.push(ast_grep_rule);
    }

    // Create combined scanners
    let thread_rule_refs: Vec<_> = thread_rules.iter().collect();
    let ast_grep_rule_refs: Vec<_> = ast_grep_rules.iter().collect();

    let thread_combined_scan = ThreadCombinedScan::new(thread_rule_refs);
    let ast_grep_combined_scan = AstGrepCombinedScan::new(ast_grep_rule_refs);

    let thread_grep = ThreadSupportLang::TypeScript.ast_grep(data.test_code);
    let ast_grep_grep = AstGrepSupportLang::TypeScript.ast_grep(data.test_code);

    // Benchmark thread-rule-engine
    group.bench_function("thread_rule_engine", |b| {
        b.iter(|| {
            let result = thread_combined_scan.scan(black_box(&thread_grep), false);
            black_box(result);
        });
    });

    // Benchmark ast-grep-config
    group.bench_function("ast_grep_config", |b| {
        b.iter(|| {
            let result = ast_grep_combined_scan.scan(black_box(&ast_grep_grep), false);
            black_box(result);
        });
    });

    group.finish();
}

fn bench_memory_usage_comparison(c: &mut Criterion) {
    let data = ComparisonData::new();
    let mut group = c.benchmark_group("memory_usage_comparison");

    // Benchmark memory allocation during rule creation
    group.bench_function("thread_rule_engine_memory", |b| {
        let globals = ThreadGlobalRules::default();
        b.iter(|| {
            let mut rules = Vec::new();
            for rule_yaml in &data.rules {
                let rule = thread_from_yaml_string::<ThreadSupportLang>(rule_yaml, &globals)
                    .expect("should parse")
                    .into_iter()
                    .next()
                    .unwrap();
                rules.push(rule);
            }
            black_box(rules);
        });
    });

    group.bench_function("ast_grep_config_memory", |b| {
        let globals = AstGrepGlobalRules::default();
        b.iter(|| {
            let mut rules = Vec::new();
            for rule_yaml in &data.rules {
                let rule = ast_grep_from_yaml_string::<AstGrepSupportLang>(rule_yaml, &globals)
                    .expect("should parse")
                    .into_iter()
                    .next()
                    .unwrap();
                rules.push(rule);
            }
            black_box(rules);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_rule_parsing_comparison,
    bench_rule_matching_comparison,
    bench_combined_scan_comparison,
    bench_memory_usage_comparison
);
criterion_main!(benches);
