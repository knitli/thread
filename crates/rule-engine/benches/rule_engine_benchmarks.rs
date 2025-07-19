// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
// SPDX-License-Identifier: MIT OR Apache-2.0

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::hint::black_box;

use thread_language::{LanguageExt, SupportLang};


use thread_rule_engine::{
    from_yaml_string, GlobalRules, RuleCollection, CombinedScan,
};

pub type BenchLanguage = SupportLang;

// Benchmark data structures
struct BenchmarkData {
    simple_patterns: Vec<&'static str>,
    complex_patterns: Vec<&'static str>,
    code_samples: Vec<(&'static str, &'static str)>, // (language, code)
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
            complex_patterns: vec![
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
  constraints:
    VAR:
      regex: ^[a-z]+$
"#,
                r#"
id: complex-pattern-3
language: TypeScript
rule:
  all:
    - pattern: class $CLASS { $$$ }
    - has:
        pattern: constructor($$$) { $$$ }
        stopBy: end
    - has:
        pattern: $METHOD($$$) { $$$ }
        stopBy: end
  constraints:
    CLASS:
      regex: ^[A-Z][a-zA-Z0-9]*$
"#,
            ],
            code_samples: vec![
                ("typescript", include_str!("../test_data/sample_typescript.ts")),
                ("javascript", include_str!("../test_data/sample_javascript.js")),
                ("python", include_str!("../test_data/sample_python.py")),
                ("rust", include_str!("../test_data/sample_rust.rs")),
            ],
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

        group.bench_with_input(
            BenchmarkId::new("simple_rule", i),
            &yaml,
            |b, yaml| {
                b.iter(|| {
                    let _rule = from_yaml_string::<BenchLanguage>(black_box(yaml), &globals)
                        .expect("should parse");
                });
            },
        );
    }

    // Benchmark complex rule parsing
    for (i, pattern) in data.complex_patterns.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("complex_rule", i),
            pattern,
            |b, pattern| {
                b.iter(|| {
                    let _rule = from_yaml_string::<BenchLanguage>(black_box(pattern), &globals)
                        .expect("should parse");
                });
            },
        );
    }

    group.finish();
}

fn bench_rule_matching(c: &mut Criterion) {
    let data = BenchmarkData::new();
    let globals = GlobalRules::default();

    let mut group = c.benchmark_group("rule_matching");

    // Create test rules
    let simple_rule_yaml = r#"
id: test-console-log
message: found console.log
severity: info
language: TypeScript
rule:
  pattern: console.log($A)
"#;

    let complex_rule_yaml = r#"
id: test-function-with-console
message: found function with console.log
severity: info
language: TypeScript
rule:
  all:
    - pattern: console.log($A)
    - inside:
        pattern: function $F() { $$$ }
        stopBy: end
"#;

    let simple_rules = from_yaml_string::<BenchLanguage>(simple_rule_yaml, &globals)
        .expect("should parse");
    let complex_rules = from_yaml_string::<BenchLanguage>(complex_rule_yaml, &globals)
        .expect("should parse");

    // Test against sample code
    for (lang_name, code) in &data.code_samples {
        if *lang_name == "typescript" {
            let grep = BenchLanguage::TypeScript.ast_grep(code);

            group.bench_with_input(
                BenchmarkId::new("simple_match", lang_name),
                &(grep.clone(), &simple_rules),
                |b, (grep, rules)| {
                    b.iter(|| {
                        let root = grep.root();
                        let matches: Vec<_> = rules.iter()
                            .flat_map(|rule| root.find_all(&rule.matcher))
                            .collect();
                        black_box(matches);
                    });
                },
            );
            group.bench_with_input(
                BenchmarkId::new("complex_match", lang_name),
                &(grep.clone(), &complex_rules),
                |b, (grep, rules)| {
                    b.iter(|| {
                        let root = grep.root();
                        let matches: Vec<_> = Vec::from_iter(
                            rules.iter().flat_map(|rule| root.find_all(&rule.matcher)),
                        );
                        black_box(matches);
                    });
                },
            );
        }
    }

    group.finish();
}

fn bench_rule_collection(c: &mut Criterion) {
    let data = BenchmarkData::new();
    let globals = GlobalRules::default();

    let mut group = c.benchmark_group("rule_collection");

    // Create multiple rules
    let mut rules = Vec::new();
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

        let rule = from_yaml_string::<BenchLanguage>(&yaml, &globals)
            .expect("should parse")
            .into_iter()
            .next()
            .unwrap();
        rules.push(rule);
    }
    // Benchmark collection creation
    group.bench_function("collection_creation", |b| {
        b.iter(|| {
            let _collection = RuleCollection::try_new(black_box(rules.clone()))
                .expect("should create collection");
        });
    });

    // Benchmark combined scan

    let rule_refs: Vec<_> = rules.iter().collect();
    let combined_scan = CombinedScan::new(rule_refs);

    for (lang_name, code) in &data.code_samples {
        if *lang_name == "typescript" {
            let grep = BenchLanguage::TypeScript.ast_grep(code);

            group.bench_with_input(
                BenchmarkId::new("combined_scan", lang_name),
                &(grep.clone(), &combined_scan),
                |b, (grep, scan)| {
                    b.iter(|| {
                        let result = scan.scan(black_box(grep), false);
                        black_box(result);
                    });
                },
            );
        }
    }

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
  UPPER:
    uppercase:
      source: $A
  LOWER:
    lowercase:
      source: $A
  SUBSTRING:
    substring:
      source: $A
      startChar: 1
      endChar: -1
"#;

    let rule = from_yaml_string::<BenchLanguage>(transform_rule_yaml, &globals)
        .expect("should parse")[0].clone();

    let test_code = r#"
function test() {
    console.log("Hello World");
    console.log('test string');
    console.log(`template ${variable}`);
}
"#;

    let grep = BenchLanguage::TypeScript.ast_grep(test_code);

    group.bench_function("transformation", |b| {
        b.iter(|| {
            let matches: Vec<_> = grep.root().find_all(&rule.matcher).collect();
            for node_match in matches {
                let env = node_match.get_env();
                // Access transformed variables
                let _ = env.get_transformed("UPPER");
                let _ = env.get_transformed("LOWER");
                let _ = env.get_transformed("SUBSTRING");
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_rule_parsing,
    bench_rule_matching,
    bench_rule_collection,
    bench_rule_transformation
);
criterion_main!(benches);
