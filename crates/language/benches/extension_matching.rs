// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

//! Benchmarks for extension matching performance optimization.
//!
//! This benchmark suite compares the performance of different extension matching strategies:
//! 1. Original implementation (O(n*m) fallback)
//! 2. Character-based bucketing
//! 3. Aho-Corasick matching
//! 4. Hybrid approach (bucketing + aho-corasick)

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::path::Path;
use thread_language::{SupportLang, extension_matcher, Language, from_extension};

/// Original implementation for comparison (O(n*m) fallback)
fn from_extension_original(path: &Path) -> Option<SupportLang> {
    let ext = path.extension()?.to_str()?;
    
    // Fast path: try most common extensions first
    match ext {
        "c" | "h" => return Some(SupportLang::C),
        "cpp" | "cc" | "cxx" => return Some(SupportLang::Cpp),
        "css" => return Some(SupportLang::Css),
        "go" => return Some(SupportLang::Go),
        "html" | "htm" => return Some(SupportLang::Html),
        "java" => return Some(SupportLang::Java),
        "js" | "mjs" | "cjs" => return Some(SupportLang::JavaScript),
        "json" => return Some(SupportLang::Json),
        "py" | "py3" | "pyi" => return Some(SupportLang::Python),
        "rs" => return Some(SupportLang::Rust),
        "ts" | "cts" | "mts" => return Some(SupportLang::TypeScript),
        "tsx" => return Some(SupportLang::Tsx),
        "yaml" | "yml" => return Some(SupportLang::Yaml),
        _ => {}
    }

    // Original O(n*m) fallback
    SupportLang::all_langs()
        .iter()
        .copied()
        .find(|&l| thread_language::extensions(l).contains(&ext))
}

/// Test cases covering different scenarios
fn get_test_cases() -> Vec<(&'static str, &'static str)> {
    vec![
        // Fast path extensions (should be equally fast in all implementations)
        ("main.rs", "rs"),
        ("app.js", "js"),
        ("index.html", "html"),
        ("data.json", "json"),
        ("script.py", "py"),
        ("main.go", "go"),
        ("style.css", "css"),
        ("component.tsx", "tsx"),
        
        // Less common extensions (benefit most from optimization)
        ("build.gradle.kts", "kts"),
        ("config.yml", "yml"),
        ("script.sh", "sh"),
        ("app.swift", "swift"),
        ("main.cpp", "cpp"),
        ("header.hpp", "hpp"),
        ("style.scss", "scss"),
        ("script.rb", "rb"),
        ("main.scala", "scala"),
        ("app.kt", "kt"),
        
        // Case variations
        ("Main.RS", "RS"),
        ("App.JS", "JS"),
        ("Config.YML", "YML"),
        
        // Non-existent extensions (worst case)
        ("file.xyz", "xyz"),
        ("test.unknown", "unknown"),
        ("data.fake", "fake"),
    ]
}

fn bench_original_implementation(c: &mut Criterion) {
    let test_cases = get_test_cases();
    
    c.bench_function("original_implementation", |b| {
        b.iter(|| {
            for (filename, _) in &test_cases {
                let path = Path::new(filename);
                black_box(from_extension_original(path));
            }
        })
    });
}

fn bench_char_bucket_matching(c: &mut Criterion) {
    let test_cases = get_test_cases();
    
    c.bench_function("char_bucket_matching", |b| {
        b.iter(|| {
            for (_, ext) in &test_cases {
                black_box(extension_matcher::match_by_char_bucket(ext));
            }
        })
    });
}

fn bench_aho_corasick_matching(c: &mut Criterion) {
    let test_cases = get_test_cases();
    
    c.bench_function("aho_corasick_matching", |b| {
        b.iter(|| {
            for (_, ext) in &test_cases {
                black_box(extension_matcher::match_by_aho_corasick(ext));
            }
        })
    });
}

fn bench_hybrid_matching(c: &mut Criterion) {
    let test_cases = get_test_cases();
    
    c.bench_function("hybrid_matching", |b| {
        b.iter(|| {
            for (_, ext) in &test_cases {
                black_box(extension_matcher::match_extension_optimized(ext));
            }
        })
    });
}

fn bench_new_from_extension(c: &mut Criterion) {
    let test_cases = get_test_cases();
    
    c.bench_function("new_from_extension", |b| {
        b.iter(|| {
            for (filename, _) in &test_cases {
                let path = Path::new(filename);
                black_box(from_extension(path));
            }
        })
    });
}

fn bench_by_extension_type(c: &mut Criterion) {
    let mut group = c.benchmark_group("by_extension_type");
    
    // Common extensions (fast path)
    let common_extensions = ["rs", "js", "py", "go", "html", "css", "json"];
    for ext in common_extensions {
        group.bench_with_input(BenchmarkId::new("common", ext), &ext, |b, &ext| {
            b.iter(|| {
                black_box(extension_matcher::match_extension_optimized(ext));
            })
        });
    }
    
    // Uncommon extensions (fallback path)
    let uncommon_extensions = ["kts", "swift", "scala", "rb", "hpp", "scss"];
    for ext in uncommon_extensions {
        group.bench_with_input(BenchmarkId::new("uncommon", ext), &ext, |b, &ext| {
            b.iter(|| {
                black_box(extension_matcher::match_extension_optimized(ext));
            })
        });
    }
    
    // Non-existent extensions (worst case)
    let nonexistent_extensions = ["xyz", "unknown", "fake", "test"];
    for ext in nonexistent_extensions {
        group.bench_with_input(BenchmarkId::new("nonexistent", ext), &ext, |b, &ext| {
            b.iter(|| {
                black_box(extension_matcher::match_extension_optimized(ext));
            })
        });
    }
    
    group.finish();
}

fn bench_case_sensitivity(c: &mut Criterion) {
    let mut group = c.benchmark_group("case_sensitivity");
    
    let test_extensions = [
        ("rs", "RS"),
        ("js", "JS"),
        ("py", "PY"),
        ("cpp", "CPP"),
        ("html", "HTML"),
    ];
    
    for (lower, upper) in test_extensions {
        group.bench_with_input(BenchmarkId::new("lowercase", lower), &lower, |b, &ext| {
            b.iter(|| {
                black_box(extension_matcher::match_extension_optimized(ext));
            })
        });
        
        group.bench_with_input(BenchmarkId::new("uppercase", upper), &upper, |b, &ext| {
            b.iter(|| {
                black_box(extension_matcher::match_extension_optimized(ext));
            })
        });
    }
    
    group.finish();
}

fn bench_initialization_overhead(c: &mut Criterion) {
    c.bench_function("optimization_stats", |b| {
        b.iter(|| {
            black_box(extension_matcher::get_optimization_stats());
        })
    });
}

criterion_group!(
    benches,
    bench_original_implementation,
    bench_char_bucket_matching,
    bench_aho_corasick_matching,
    bench_hybrid_matching,
    bench_new_from_extension,
    bench_by_extension_type,
    bench_case_sensitivity,
    bench_initialization_overhead
);

criterion_main!(benches);
