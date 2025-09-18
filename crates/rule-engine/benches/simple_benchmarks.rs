// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
// SPDX-License-Identifier: MIT OR Apache-2.0

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use thread_language::SupportLang;

use thread_rule_engine::{GlobalRules, from_yaml_string};

// Benchmark data
struct BenchmarkData {
    simple_patterns: Vec<&'static str>,
    complex_rules: Vec<&'static str>,
    test_code: &'static str,
}

impl BenchmarkData {
    fn new() -> Self {
        Self {
            simple_patterns: vec![
                "console.log($A)",
                "function $F() { $$$ }",
                "let $VAR = $VALUE",
                "import $MODULE from '$PATH'",
                "class $CLASS { $$$ }",
            ],
            complex_rules: vec![
                r#"
id: complex-pattern-1
language: TypeScript
rule:
  all:
    - pattern: console.log($A)
    - inside:
        pattern: function $F() { $$$ }
        stopBy: end
"#,
                r#"
id: complex-pattern-2
language: TypeScript
rule:
  any:
    - pattern: let $VAR = $VALUE
    - pattern: const $VAR = $VALUE
    - pattern: var $VAR = $VALUE
"#,
            ],
            test_code: include_str!("../test_data/sample_typescript.ts"),
        }
    }
}

fn bench_rule_parsing(c: &mut Criterion) {
    let data = BenchmarkData::new();
    let globals = GlobalRules::default();
    let mut group = c.benchmark_group("rule_parsing");

    // Benchmark simple rule parsing
    for (i, pattern) in data.simple_patterns.iter().enumerate() {
        let yaml = format!(
            r#"
id: test-rule-{}
message: test rule
severity: info
language: TypeScript
rule:
  pattern: {}
"#,
            i, pattern
        );

        group.bench_with_input(BenchmarkId::new("simple_rule", i), &yaml, |b, yaml| {
            b.iter(|| {
                let _rule = from_yaml_string::<SupportLang>(black_box(yaml), &globals)
                    .expect("should parse");
            });
        });
    }

    // Benchmark complex rule parsing
    for (i, pattern) in data.complex_rules.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("complex_rule", i),
            pattern,
            |b, pattern| {
                b.iter(|| {
                    let _rule = from_yaml_string::<SupportLang>(black_box(pattern), &globals)
                        .expect("should parse");
                });
            },
        );
    }

    group.finish();
}

fn bench_rule_compilation(c: &mut Criterion) {
    let data = BenchmarkData::new();
    let globals = GlobalRules::default();
    let mut group = c.benchmark_group("rule_compilation");

    // Create multiple rules
    let mut rule_yamls = Vec::new();
    for (i, pattern) in data.simple_patterns.iter().enumerate() {
        let yaml = format!(
            r#"
id: test-rule-{}
message: test rule {}
severity: info
language: TypeScript
rule:
  pattern: {}
"#,
            i, i, pattern
        );
        rule_yamls.push(yaml);
    }

    // Benchmark rule compilation
    group.bench_function("multiple_rules", |b| {
        b.iter(|| {
            let mut all_rules = Vec::new();
            for yaml in &rule_yamls {
                let rules = from_yaml_string::<SupportLang>(black_box(yaml), &globals)
                    .expect("should parse");
                all_rules.extend(rules);
            }
            black_box(all_rules);
        });
    });

    group.finish();
}

fn bench_rule_transformation(c: &mut Criterion) {
    let globals = GlobalRules::default();
    let mut group = c.benchmark_group("rule_transformation");

    let transform_rule_yaml = r#"
id: test-transform
message: test transformation
severity: info
language: TypeScript
rule:
  pattern: console.log($A)
transform:
  transformed:
    substring:
      source: $A
      startChar: 1
      endChar: -1
"#;

    group.bench_function("transformation_parsing", |b| {
        b.iter(|| {
            let _rule = from_yaml_string::<SupportLang>(black_box(transform_rule_yaml), &globals)
                .expect("should parse");
        });
    });

    group.finish();
}

fn bench_yaml_deserialization(c: &mut Criterion) {
    let globals = GlobalRules::default();
    let mut group = c.benchmark_group("yaml_deserialization");

    let large_rule_yaml = r#"
id: large-rule
message: large rule with many patterns
severity: info
language: TypeScript
rule:
  any:
    - pattern: console.log($A)
    - pattern: console.warn($A)
    - pattern: console.error($A)
    - pattern: console.debug($A)
    - pattern: console.info($A)
    - pattern: console.trace($A)
    - pattern: console.table($A)
    - pattern: console.group($A)
    - pattern: console.groupEnd($A)
    - pattern: console.time($A)
    - pattern: console.timeEnd($A)
    - pattern: console.count($A)
    - pattern: console.countReset($A)
    - pattern: console.clear($A)
    - pattern: console.assert($A, $B)
    - pattern: console.dir($A)
    - pattern: console.dirxml($A)
  all:
    - pattern: console.log($A)
    - inside:
        pattern: function $B() {$$$}
        stopBy: end
constraints:
  B:
    regex: test
  A:
    regex: ^[a-zA-Z_][a-zA-Z0-9_]*$

transform:
  substring:
    source: $A
    startChar: 1
    endChar: -1
  convert:
      toCase: lowerCase
      source: $A
  substring:
      source: $A
      startChar: 1
      endChar: -1
  convert:
    toCase: upperCase
    source: $A
  substring:
    source: $A
    startChar: 1
    endChar: -1
  convert:
    toCase: camelCase
    source: $A
  convert:
    toCase: camelCase
    source: $A
  convert:
    toCase: snakeCase
    source: $A
"#;

    group.bench_function("large_rule_parsing", |b| {
        b.iter(|| {
            let _rule = from_yaml_string::<SupportLang>(black_box(large_rule_yaml), &globals)
                .expect("should parse");
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_rule_parsing,
    bench_rule_compilation,
    bench_rule_transformation,
    bench_yaml_deserialization
);
criterion_main!(benches);
