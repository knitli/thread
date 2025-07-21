// SPDX-FileCopyrightText: 2025 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::path::Path;
use std::str::FromStr;
use thread_language::*;

fn bench_pre_process_pattern(c: &mut Criterion) {
    let patterns = [
        "$VAR",
        "function $NAME($ARGS) { $BODY }",
        "class $CLASS extends $PARENT { $METHODS }",
        "import $MODULE from '$PATH'",
        "const $VAR = $VALUE;",
        "if ($CONDITION) { $THEN } else { $ELSE }",
        "$$$A",            // Anonymous multiple
        "no dollars here", // No processing needed
    ];

    c.bench_function("pre_process_pattern", |b| {
        b.iter(|| {
            for pattern in &patterns {
                let lang = thread_language::Python;
                black_box(lang.pre_process_pattern(black_box(pattern)));
            }
        })
    });
}

fn bench_from_str(c: &mut Criterion) {
    let languages = [
        "rust",
        "rs",
        "javascript",
        "js",
        "typescript",
        "ts",
        "python",
        "py",
        "java",
        "cpp",
        "c",
        "go",
        "html",
        "css",
    ];

    c.bench_function("from_str", |b| {
        b.iter(|| {
            for lang_str in &languages {
                black_box(SupportLang::from_str(black_box(lang_str)).ok());
            }
        })
    });
}

fn bench_from_extension(c: &mut Criterion) {
    let files = [
        "main.rs",
        "app.js",
        "index.ts",
        "script.tsx",
        "main.py",
        "App.java",
        "main.cpp",
        "main.c",
        "main.go",
        "index.html",
        "style.css",
        "config.json",
        "data.yaml",
        "rare.scala",
    ];

    c.bench_function("from_extension", |b| {
        b.iter(|| {
            for file in &files {
                let path = Path::new(black_box(file));
                black_box(from_extension(path));
            }
        })
    });
}

fn bench_language_loading(c: &mut Criterion) {
    c.bench_function("language_loading", |b| {
        b.iter(|| {
            // Test cached language loading
            for _ in 0..10 {
                black_box(thread_language::parsers::language_rust());
                black_box(thread_language::parsers::language_javascript());
                black_box(thread_language::parsers::language_python());
            }
        })
    });
}

fn bench_html_injection(c: &mut Criterion) {
    let html_content = r#"
        <html>
            <head>
                <style>
                    .class { color: red; }
                    .other { background: blue; }
                </style>
                <style lang="scss">
                    $color: red;
                    .scss { color: $color; }
                </style>
            </head>
            <body>
                <script>
                    console.log("Hello world");
                    function test() { return 42; }
                </script>
                <script lang="ts">
                    const x: number = 42;
                    console.log(x);
                </script>
            </body>
        </html>
    "#;

    c.bench_function("html_injection_extraction", |b| {
        b.iter(|| {
            let root = Html.ast_grep(black_box(html_content));
            black_box(Html.extract_injections(root.root()));
        })
    });
}

criterion_group!(
    benches,
    bench_pre_process_pattern,
    bench_from_str,
    bench_from_extension,
    bench_language_loading,
    bench_html_injection
);
criterion_main!(benches);
