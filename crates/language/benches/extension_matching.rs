// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

//! Benchmarks for extension matching performance optimization.
//!
//! This benchmark originally compared multiple strategies
//! for file extension matching in the `thread_language` crate.
//! We tried:
//! 1. Original implementation (Ast-Grep's original implementation, which is O(n*m))
//! 2. Character-based bucketing - the idea here was to use the first character to reduce the search space
//! 3. Length-based bucketing -- using the len() of the extension to reduce the search space
//! 4. Combined bucketing (character + length) - (2 + 3)
//! 5. Aho-Corasick matching
//!
//! These benchmarks made it very clear that Aho-Corasick was the clear winner. Initial results looked something like:
//! ```text
//! - Original implementation: ~2µs/extension (the simplified implementation here made this much faster than it probably was in reality)
//! - Character-based bucketing: ~1.1µs/extension
//! - Length-based bucketing: ~1.5µs/extension
//! - Combined (character + length) bucketing: ~2.1µs/extension
//! - Aho-Corasick matching: ~750ns/extension (0.75µs/extension)
//!    - Matches can be as fast as ~70ns for extensions without collisions
//!    - Worst case is about 3x faster than the original implementation
//! A similar attempt to frontload most common extensions before falling back to Aho-Corasick, was very fast for common extensions, but at the expense of uncommon extensions (~3ms/extension).
//!

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::path::Path;
use thread_language::{SupportLang, ext_iden, from_extension};

fn original_match(ext: &str) -> Option<SupportLang> {
    SupportLang::all_langs()
        .iter()
        .copied()
        .find(|&l| thread_language::extensions(l).contains(&ext))
}

/// Original implementation for comparison (O(n*m) fallback)
fn from_extension_original_path(path: &Path) -> Option<SupportLang> {
    let ext = path.extension()?.to_str()?;
    original_match(ext)
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

fn bench_aho_corasick_matching(c: &mut Criterion) {
    let test_cases = get_test_cases();

    c.bench_function("original_matching", |b| {
        b.iter(|| {
            for (_, ext) in &test_cases {
                black_box(original_match(ext));
            }
        })
    });

    c.bench_function("aho_corasick_matching", |b| {
        b.iter(|| {
            for (_, ext) in &test_cases {
                black_box(ext_iden::match_by_aho_corasick(ext));
            }
        })
    });
}

/// This benchmarks how long it takes to match an actual file.
/// Note: most of the overhead here is creating a Path object,
/// so this is not a perfect benchmark for the matching itself.
fn bench_new_from_extension(c: &mut Criterion) {
    let test_cases = get_test_cases();

    c.bench_function("new_from_ext_orig", |b| {
        b.iter(|| {
            for (filename, _) in &test_cases {
                let path = Path::new(filename);
                black_box(from_extension_original_path(&path));
            }
        })
    });

    c.bench_function("new_from_ext_with_Aho_Corasick", |b| {
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

    let common_extensions = ["rs", "js", "py", "go", "html", "css", "json"];
    for ext in common_extensions {
        group.bench_with_input(BenchmarkId::new("common_original", ext), &ext, |b, &ext| {
            b.iter(|| {
                black_box(original_match(ext));
            })
        });
    }

    for ext in common_extensions {
        group.bench_with_input(
            BenchmarkId::new("common_aho_corasick", ext),
            &ext,
            |b, &ext| {
                b.iter(|| {
                    black_box(ext_iden::match_by_aho_corasick(ext));
                })
            },
        );
    }

    let uncommon_extensions = ["kts", "swift", "scala", "rb", "hpp", "scss"];
    for ext in uncommon_extensions {
        group.bench_with_input(
            BenchmarkId::new("uncommon_original", ext),
            &ext,
            |b, &ext| {
                b.iter(|| {
                    black_box(original_match(ext));
                })
            },
        );
    }

    for ext in uncommon_extensions {
        group.bench_with_input(
            BenchmarkId::new("uncommon_aho_corasick", ext),
            &ext,
            |b, &ext| {
                b.iter(|| {
                    black_box(ext_iden::match_by_aho_corasick(ext));
                })
            },
        );
    }

    // Non-existent extensions (worst case)
    let nonexistent_extensions = ["xyz", "unknown", "fake", "test"];
    for ext in nonexistent_extensions {
        group.bench_with_input(
            BenchmarkId::new("nonexistent_original", ext),
            &ext,
            |b, &ext| {
                b.iter(|| {
                    black_box(original_match(ext));
                })
            },
        );
    }

    for ext in nonexistent_extensions {
        group.bench_with_input(
            BenchmarkId::new("nonexistent_aho_corasick", ext),
            &ext,
            |b, &ext| {
                b.iter(|| {
                    black_box(ext_iden::match_by_aho_corasick(ext));
                })
            },
        );
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

    for (lower, upper) in &test_extensions {
        group.bench_with_input(
            BenchmarkId::new("lowercase_original", lower),
            &lower,
            |b, &ext| {
                b.iter(|| {
                    black_box(original_match(ext));
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("uppercase_original", upper),
            &upper,
            |b, &ext| {
                b.iter(|| {
                    black_box(original_match(ext));
                })
            },
        );
    }

    for (lower, upper) in test_extensions {
        group.bench_with_input(
            BenchmarkId::new("lowercase_aho_corasick", lower),
            &lower,
            |b, &ext| {
                b.iter(|| {
                    black_box(ext_iden::match_by_aho_corasick(ext));
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("uppercase_aho_corasick", upper),
            &upper,
            |b, &ext| {
                b.iter(|| {
                    black_box(ext_iden::match_by_aho_corasick(ext));
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_aho_corasick_matching,
    bench_new_from_extension,
    bench_by_extension_type,
    bench_case_sensitivity,
);

criterion_main!(benches);
