<!--
SPDX-FileCopyrightText: 2026 Knitli Inc. <knitli@knit.li>

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.1.0 (2026-03-02)

<csr-id-d5519df7aac003c85d71d15b561d29b63fe4c00d/>
<csr-id-659510d42574fe2efe5639bb749d9f29445e7d88/>
<csr-id-f9c25705eba3262b41e538a4db64e8d8e1ea4185/>
<csr-id-aac8239716f1a57806c308f0e44e41289e6cc9d0/>
<csr-id-36bde537ea20079f4923856b149efa99b9957063/>

### New Features

 - <csr-id-eead51dcb62ee0b6fe99856ce40ef929df1c7d41/> Implement main thread crate to unify crate exposure
   * feat(thread): Implement main thread crate to unify crate exposure
* fix(tests): fixed failing tests and clarified language in planning docs
* fix(tests): fixed several test and typing issues
* fix: missing re-export causing failing tests. All tests across codebase now passing
* fix: multiple test and typing fixes. test suite now all green.
* chore(ci): update cargo-deny to ignore trivial dependency lockfile duplications
* chore(lint): formatting and minor fixes
* chore(fmt): formatted codebase
* feat: add support for terraform, nix, and solidity; update language tests
* feat: add support for terraform, nix, and solidity; update language tests
* fix: fixed issue where all-features caused failures in CI due to conflicting feature flags
* fix(ci): correct issue where Rust stable causes lint failures (thread-services uses experimental trait aliases)
* fix(ci): Remove cargo license check from CI; redundant with cargo deny, which is more reliable
* fix(ci): remove Semgrep-SAST check; semgrep-action is deprecated and caused failing CI from deprecation warnings
* [38;5;238m─────┬──────────────────────────────────────────────────────────────────────────[0m
        [38;5;238m│ [0m[1mSTDIN[0m
   [38;5;238m─────┼──────────────────────────────────────────────────────────────────────────[0m
   [38;5;238m   1[0m [38;5;238m│[0m [38;5;231mfix(thread): expose flow module under worker feature; fix test type annotation[0m
   [38;5;238m   2[0m [38;5;238m│[0m
   [38;5;238m   3[0m [38;5;238m│[0m [38;5;231m- Change `#[cfg(feature = "flow")]` to `#[cfg(any(feature = "flow", feature = "worker"))]`[0m
   [38;5;238m   4[0m [38;5;238m│[0m [38;5;231m  on `pub mod flow` in lib.rs. The `worker` feature implicitly enables `dep:thread-flow`[0m
   [38;5;238m   5[0m [38;5;238m│[0m [38;5;231m  via `thread-flow/worker`, but the flow module was never exposed because the crate's own[0m
   [38;5;238m   6[0m [38;5;238m│[0m [38;5;231m  `flow` feature remained false. Simply adding "flow" to worker was not viable since it[0m
   [38;5;238m   7[0m [38;5;238m│[0m [38;5;231m  would also pull in `thread-flow/parallel` and `thread-flow/postgres-backend`, which are[0m
   [38;5;238m   8[0m [38;5;238m│[0m [38;5;231m  incompatible with WASM/edge deployment.[0m
   [38;5;238m   9[0m [38;5;238m│[0m [38;5;231m- Remove incorrect `Root<_>` type annotation in integration test; `LanguageExt::ast_grep`[0m
   [38;5;238m  10[0m [38;5;238m│[0m [38;5;231m  returns `AstGrep<...>`, not `Root`. Drop the now-unused `use thread::Root` import.[0m
   [38;5;238m  11[0m [38;5;238m│[0m
   [38;5;238m  12[0m [38;5;238m│[0m [38;5;231mCo-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>[0m
   [38;5;238m─────┴──────────────────────────────────────────────────────────────────────────[0m
* [38;5;238m─────┬──────────────────────────────────────────────────────────────────────────[0m
        [38;5;238m│ [0m[1mSTDIN[0m
   [38;5;238m─────┼──────────────────────────────────────────────────────────────────────────[0m
   [38;5;238m   1[0m [38;5;238m│[0m [38;5;231mfix(ci): resolve three clippy errors causing CI failures[0m
   [38;5;238m   2[0m [38;5;238m│[0m
   [38;5;238m   3[0m [38;5;238m│[0m [38;5;231m- comparison_benchmarks.rs: use ThreadSupportLang (already imported) instead[0m
   [38;5;238m   4[0m [38;5;238m│[0m [38;5;231m  of thread_language::TypeScript for from_yaml_string type parameter. TypeScript[0m
   [38;5;238m   5[0m [38;5;238m│[0m [38;5;231m  only implements Deserialize when the `typescript` feature is enabled; SupportLang[0m
   [38;5;238m   6[0m [38;5;238m│[0m [38;5;231m  is the correct type here and is already used consistently everywhere else in the[0m
   [38;5;238m   7[0m [38;5;238m│[0m [38;5;231m  same bench file.[0m
   [38;5;238m   8[0m [38;5;238m│[0m [38;5;231m- d1.rs: remove redundant closure in filter_map; PathBuf::from can be passed[0m
   [38;5;238m   9[0m [38;5;238m│[0m [38;5;231m  directly as the function item.[0m
   [38;5;238m  10[0m [38;5;238m│[0m [38;5;231m- performance.rs: replace manual checked-division pattern (if x > 0 { a / x })[0m
   [38;5;238m  11[0m [38;5;238m│[0m [38;5;231m  with checked_div().unwrap_or(0) in fingerprint_stats and query_stats.[0m
   [38;5;238m  12[0m [38;5;238m│[0m
   [38;5;238m  13[0m [38;5;238m│[0m [38;5;231mCo-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>[0m
   [38;5;238m─────┴──────────────────────────────────────────────────────────────────────────[0m
* [38;5;238m─────┬──────────────────────────────────────────────────────────────────────────[0m
        [38;5;238m│ [0m[1mSTDIN[0m
   [38;5;238m─────┼──────────────────────────────────────────────────────────────────────────[0m
   [38;5;238m   1[0m [38;5;238m│[0m [38;5;231mfix(ci): resolve additional clippy lints in flow crate[0m
   [38;5;238m   2[0m [38;5;238m│[0m
   [38;5;238m   3[0m [38;5;238m│[0m [38;5;231m- types.rs: fix inconsistent digit grouping in test timestamp literal[0m
   [38;5;238m   4[0m [38;5;238m│[0m [38;5;231m  (1706400000_000_000 → 1_706_400_000_000_000)[0m
   [38;5;238m   5[0m [38;5;238m│[0m [38;5;231m- graph.rs: replace PathBuf::from("A/B") with Path::new("A/B") in assertion[0m
   [38;5;238m   6[0m [38;5;238m│[0m [38;5;231m  comparison to avoid allocating owned values (cmp_owned lint); clippy[0m
   [38;5;238m   7[0m [38;5;238m│[0m [38;5;231m  suggested bare &str but PathBuf doesn't impl PartialEq<&str>, so use Path[0m
   [38;5;238m   8[0m [38;5;238m│[0m [38;5;231m- observability_metrics_tests.rs: replace &[x.clone()] with[0m
   [38;5;238m   9[0m [38;5;238m│[0m [38;5;231m  std::slice::from_ref(&x) for four single-element slice args[0m
   [38;5;238m  10[0m [38;5;238m│[0m [38;5;231m  (cloned_ref_to_slice_refs lint)[0m
   [38;5;238m  11[0m [38;5;238m│[0m
   [38;5;238m  12[0m [38;5;238m│[0m [38;5;231mCo-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>[0m
   [38;5;238m─────┴──────────────────────────────────────────────────────────────────────────[0m
* fix(lint): fix lint warnings
* feat: replace HashMap/Set usage with RapidMap/Set across flow crate; linting
* [38;5;238m─────┬──────────────────────────────────────────────────────────────────────────[0m
        [38;5;238m│ [0m[1mSTDIN[0m
   [38;5;238m─────┼──────────────────────────────────────────────────────────────────────────[0m
   [38;5;238m   1[0m [38;5;238m│[0m [38;5;231mfeat(alloc): add mimalloc as optional global allocator across all crates[0m
   [38;5;238m   2[0m [38;5;238m│[0m
   [38;5;238m   3[0m [38;5;238m│[0m [38;5;231mAdd mimalloc as an optional dependency to all crates, gated behind a[0m
   [38;5;238m   4[0m [38;5;238m│[0m [38;5;231m`mimalloc` feature flag. When enabled (and not in `worker` mode), each[0m
   [38;5;238m   5[0m [38;5;238m│[0m [38;5;231mcrate registers MiMalloc as the global allocator.[0m
   [38;5;238m   6[0m [38;5;238m│[0m
   [38;5;238m   7[0m [38;5;238m│[0m [38;5;231mFix broken cfg attribute syntax that was identical across all six lib.rs[0m
   [38;5;238m   8[0m [38;5;238m│[0m [38;5;231mfiles — the closing paren after not(feature = "worker") was terminating[0m
   [38;5;238m   9[0m [38;5;238m│[0m [38;5;231mall() too early:[0m
   [38;5;238m  10[0m [38;5;238m│[0m
   [38;5;238m  11[0m [38;5;238m│[0m [38;5;231m  all(not(feature = "worker")), (feature = "mimalloc")  [broken][0m
   [38;5;238m  12[0m [38;5;238m│[0m [38;5;231m  all(not(feature = "worker"), feature = "mimalloc")    [fixed][0m
   [38;5;238m  13[0m [38;5;238m│[0m
   [38;5;238m  14[0m [38;5;238m│[0m [38;5;231mAlso fix thread/src/lib.rs which was missing the feature = "mimalloc"[0m
   [38;5;238m  15[0m [38;5;238m│[0m [38;5;231mguard entirely (had only not(feature = "worker")), pin mimalloc workspace[0m
   [38;5;238m  16[0m [38;5;238m│[0m [38;5;231mdep to 0.1.48, make flow's mimalloc dep optional (was incorrectly[0m
   [38;5;238m  17[0m [38;5;238m│[0m [38;5;231mnon-optional), and apply tombi TOML formatting to thread/flow/language[0m
   [38;5;238m  18[0m [38;5;238m│[0m [38;5;231mCargo.toml files.[0m
   [38;5;238m  19[0m [38;5;238m│[0m
   [38;5;238m  20[0m [38;5;238m│[0m [38;5;231mCo-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>[0m
   [38;5;238m─────┴──────────────────────────────────────────────────────────────────────────[0m
* [38;5;238m─────┬──────────────────────────────────────────────────────────────────────────[0m
        [38;5;238m│ [0m[1mSTDIN[0m
   [38;5;238m─────┼──────────────────────────────────────────────────────────────────────────[0m
   [38;5;238m   1[0m [38;5;238m│[0m [38;5;231mfix(alloc): remove duplicate #[global_allocator] from all library crates[0m
   [38;5;238m   2[0m [38;5;238m│[0m
   [38;5;238m   3[0m [38;5;238m│[0m [38;5;231mRust allows exactly one #[global_allocator] per linked binary. Defining[0m
   [38;5;238m   4[0m [38;5;238m│[0m [38;5;231mit in multiple library crates causes link-time conflicts whenever two or[0m
   [38;5;238m   5[0m [38;5;238m│[0m [38;5;231mmore of them are compiled together (which is always the case in this[0m
   [38;5;238m   6[0m [38;5;238m│[0m [38;5;231mworkspace).[0m
   [38;5;238m   7[0m [38;5;238m│[0m
   [38;5;238m   8[0m [38;5;238m│[0m [38;5;231mThe allocator must only be registered at the root/binary level. Remove[0m
   [38;5;238m   9[0m [38;5;238m│[0m [38;5;231mthe cfg+global_allocator+static GLOBAL block from the six library crates:[0m
   [38;5;238m  10[0m [38;5;238m│[0m [38;5;231m  - thread-ast-engine[0m
   [38;5;238m  11[0m [38;5;238m│[0m [38;5;231m  - thread-flow[0m
   [38;5;238m  12[0m [38;5;238m│[0m [38;5;231m  - thread-language[0m
   [38;5;238m  13[0m [38;5;238m│[0m [38;5;231m  - thread-rule-engine[0m
   [38;5;238m  14[0m [38;5;238m│[0m [38;5;231m  - thread-services[0m
   [38;5;238m  15[0m [38;5;238m│[0m [38;5;231m  - thread-utils[0m
   [38;5;238m  16[0m [38;5;238m│[0m
   [38;5;238m  17[0m [38;5;238m│[0m [38;5;231mThe block is retained only in the thread facade crate, which is the[0m
   [38;5;238m  18[0m [38;5;238m│[0m [38;5;231mintended entry point and acts as the effective root for library consumers.[0m
   [38;5;238m  19[0m [38;5;238m│[0m [38;5;231mThe mimalloc optional dep and feature flag remain in each crate in case[0m
   [38;5;238m  20[0m [38;5;238m│[0m [38;5;231mthey are needed independently.[0m
   [38;5;238m  21[0m [38;5;238m│[0m
   [38;5;238m  22[0m [38;5;238m│[0m [38;5;231mCo-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>[0m
   [38;5;238m─────┴──────────────────────────────────────────────────────────────────────────[0m
* fix: developer error with mimalloc usage :)
* [38;5;238m─────┬──────────────────────────────────────────────────────────────────────────[0m
        [38;5;238m│ [0m[1mSTDIN[0m
   [38;5;238m─────┼──────────────────────────────────────────────────────────────────────────[0m
   [38;5;238m   1[0m [38;5;238m│[0m [38;2;248;248;242mfix(lint): resolve clippy warnings in thread-utils tests[0m
   [38;5;238m   2[0m [38;5;238m│[0m
   [38;5;238m   3[0m [38;5;238m│[0m [38;2;248;248;242m- byte_char_slices: &[b'a'] → b"a" in simd.rs[0m
   [38;5;238m   4[0m [38;5;238m│[0m [38;2;248;248;242m- unreadable_literal: add numeric separators in hash_tests.rs[0m
   [38;5;238m   5[0m [38;5;238m│[0m [38;2;248;248;242m- uninlined_format_args: use inline format args throughout[0m
   [38;5;238m   6[0m [38;5;238m│[0m
   [38;5;238m   7[0m [38;5;238m│[0m [38;2;248;248;242mCo-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>[0m
   [38;5;238m─────┴──────────────────────────────────────────────────────────────────────────[0m
* [38;5;238m─────┬──────────────────────────────────────────────────────────────────────────[0m
        [38;5;238m│ [0m[1mSTDIN[0m
   [38;5;238m─────┼──────────────────────────────────────────────────────────────────────────[0m
   [38;5;238m   1[0m [38;5;238m│[0m [38;5;231mfix(lint): pass MatchStrictness by value in ast-engine match functions[0m
   [38;5;238m   2[0m [38;5;238m│[0m
   [38;5;238m   3[0m [38;5;238m│[0m [38;5;231mMatchStrictness derives Copy; passing by reference adds indirection[0m
   [38;5;238m   4[0m [38;5;238m│[0m [38;5;231mwith no benefit. Pass by value throughout match_node_impl and friends.[0m
   [38;5;238m   5[0m [38;5;238m│[0m
   [38;5;238m   6[0m [38;5;238m│[0m [38;5;231mCo-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>[0m
   [38;5;238m─────┴──────────────────────────────────────────────────────────────────────────[0m
* [38;5;238m─────┬──────────────────────────────────────────────────────────────────────────[0m
        [38;5;238m│ [0m[1mSTDIN[0m
   [38;5;238m─────┼──────────────────────────────────────────────────────────────────────────[0m
   [38;5;238m   1[0m [38;5;238m│[0m [38;5;231mfix(tooling): pin mise rust tool to nightly[0m
   [38;5;238m   2[0m [38;5;238m│[0m
   [38;5;238m   3[0m [38;5;238m│[0m [38;5;231mThe workspace requires Rust nightly: edition 2024 and experimental[0m
   [38;5;238m   4[0m [38;5;238m│[0m [38;5;231mtrait aliases (thread-services) are nightly-only. Using "latest"[0m
   [38;5;238m   5[0m [38;5;238m│[0m [38;5;231mresolves to stable at install time and breaks the build.[0m
   [38;5;238m   6[0m [38;5;238m│[0m
   [38;5;238m   7[0m [38;5;238m│[0m [38;5;231mCo-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>[0m
   [38;5;238m─────┴──────────────────────────────────────────────────────────────────────────[0m
* [38;5;238m─────┬──────────────────────────────────────────────────────────────────────────[0m
        [38;5;238m│ [0m[1mSTDIN[0m
   [38;5;238m─────┼──────────────────────────────────────────────────────────────────────────[0m
   [38;5;238m   1[0m [38;5;238m│[0m [38;5;231mfix(ci): scope integration tests to flow crate[0m
   [38;5;238m   2[0m [38;5;238m│[0m
   [38;5;238m   3[0m [38;5;238m│[0m [38;5;231mcargo nextest run --all-features without --manifest-path in a workspace[0m
   [38;5;238m   4[0m [38;5;238m│[0m [38;5;231mcompiles all members with all features, including thread-language's[0m
   [38;5;238m   5[0m [38;5;238m│[0m [38;5;231mnapi-* features which disable tree-sitter at runtime. Scope the[0m
   [38;5;238m   6[0m [38;5;238m│[0m [38;5;231mintegration test run to crates/flow/Cargo.toml where these tests live.[0m
   [38;5;238m   7[0m [38;5;238m│[0m
   [38;5;238m   8[0m [38;5;238m│[0m [38;5;231mBrings the integration job in line with the test, coverage, clippy,[0m
   [38;5;238m   9[0m [38;5;238m│[0m [38;5;231mand doc-test jobs which already apply the napi exclusion pattern.[0m
   [38;5;238m  10[0m [38;5;238m│[0m
   [38;5;238m  11[0m [38;5;238m│[0m [38;5;231mCo-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>[0m
   [38;5;238m─────┴──────────────────────────────────────────────────────────────────────────[0m
* [38;5;238m─────┬──────────────────────────────────────────────────────────────────────────[0m
        [38;5;238m│ [0m[1mSTDIN[0m
   [38;5;238m─────┼──────────────────────────────────────────────────────────────────────────[0m
   [38;5;238m   1[0m [38;5;238m│[0m [38;5;231mfix(ci): use single-line run for integration test command[0m
   [38;5;238m   2[0m [38;5;238m│[0m
   [38;5;238m   3[0m [38;5;238m│[0m [38;5;231myq (the yml hook) normalises folded block scalars to single-line[0m
   [38;5;238m   4[0m [38;5;238m│[0m [38;5;231mstrings; using >- caused the pre-push check to fail with a diff.[0m
   [38;5;238m   5[0m [38;5;238m│[0m [38;5;231mInline the cargo nextest command to match what yq expects.[0m
   [38;5;238m   6[0m [38;5;238m│[0m
   [38;5;238m   7[0m [38;5;238m│[0m [38;5;231mCo-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>[0m
   [38;5;238m─────┴──────────────────────────────────────────────────────────────────────────[0m
* [38;5;238m─────┬──────────────────────────────────────────────────────────────────────────[0m
        [38;5;238m│ [0m[1mSTDIN[0m
   [38;5;238m─────┼──────────────────────────────────────────────────────────────────────────[0m
   [38;5;238m   1[0m [38;5;238m│[0m [38;5;231mfix(lint): resolve clippy warnings in test, bench, and example files[0m
   [38;5;238m   2[0m [38;5;238m│[0m
   [38;5;238m   3[0m [38;5;238m│[0m [38;5;231mPre-push hook (--all-targets) surfaced pre-existing lint debt across[0m
   [38;5;238m   4[0m [38;5;238m│[0m [38;5;231mbench, test, and example targets that aren't compiled in normal builds.[0m
   [38;5;238m   5[0m [38;5;238m│[0m
   [38;5;238m   6[0m [38;5;238m│[0m [38;5;231m- needless_collect: use .count() directly on iterator (ast-engine bench)[0m
   [38;5;238m   7[0m [38;5;238m│[0m [38;5;231m- approx_constant: use std::f{32,64}::consts::{PI,E} (d1 tests)[0m
   [38;5;238m   8[0m [38;5;238m│[0m [38;5;231m- inconsistent_digit_grouping: add separators to large literals (incremental tests)[0m
   [38;5;238m   9[0m [38;5;238m│[0m [38;5;231m- cloned_ref_to_slice_refs: &[x.clone()] → std::slice::from_ref(&x)[0m
   [38;5;238m  10[0m [38;5;238m│[0m [38;5;231m  (analyzer_tests, integration_e2e_tests, observability_example)[0m
   [38;5;238m  11[0m [38;5;238m│[0m [38;5;231m- match_single_binding: replace single-arm match with let binding (extractor_tests)[0m
   [38;5;238m  12[0m [38;5;238m│[0m [38;5;231m- too_many_arguments: #[allow] on test helper with 8 params (d1_local_test)[0m
   [38;5;238m  13[0m [38;5;238m│[0m [38;5;231m- various auto-fixed lints across bench and test files[0m
   [38;5;238m  14[0m [38;5;238m│[0m
   [38;5;238m  15[0m [38;5;238m│[0m [38;5;231mCo-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>[0m
   [38;5;238m─────┴──────────────────────────────────────────────────────────────────────────[0m
* fix: corrected version to current 0.2.0

### Other

 - <csr-id-d5519df7aac003c85d71d15b561d29b63fe4c00d/> Add thread-flow crate, integrates ReCoco (CocoIndex) capabilities into Thread, providing dataflow driven ETL pipeline management
   * feat: add initial specification and quality checklist for Real-Time Code Graph Intelligence
   
   * Add research findings for Real-Time Code Graph Intelligence
   
   - Documented integration architecture for CocoIndex, including trait abstraction layer and optional runtime integration.
   - Evaluated component selection between ast-grep and CodeWeaver, deciding to use existing ast-grep components for MVP.
   - Established API protocol strategy, opting for a hybrid RPC over HTTP/WebSockets due to Cloudflare Workers constraints.
   - Designed a hybrid relational architecture for graph database layer with in-memory acceleration.
   - Selected WebSocket as primary real-time protocol with Server-Sent Events as fallback.
   - Organized crate structure to extend existing Thread workspace with new graph-focused crates.
   - Implemented multi-tier conflict detection strategy for progressive feedback.
   - Developed storage backend abstraction pattern to support multiple backends with optimizations.
   - Ongoing research on best practices for Rust WebAssembly, content-addressed caching, and real-time collaboration architecture.
   
   * feat: Add spec and planning documents for realtime codegraph feat
   
   * Remove workspace author and edition fields from Cargo.toml files in services, utils, wasm, and xtask crates
   
   * Update rapidhash implementation in thread-utils (#45)
   
   * Update thread-utils to use latest rapidhash API
   
   - Update `hash_help.rs` to use `rapidhash::v3` for stable file/byte hashing.
   - Update `hash_help.rs` to use `rapidhash::fast` for `RapidMap`/`RapidSet` (optimized for speed).
   - Fix build issues in workspace crates (authors, dependency conflicts) to allow tests to run.
   
   * Initial plan
   
   * Add comprehensive tests for hash_help module

### Chore

 - <csr-id-b97b45c95ff854b4bf159bd03ce1e3ff4de74d41/> Release thread-services v0.1.0, thread-flow v0.1.0, thread-rule-engine v0.1.0, thread v0.1.0

### Chore

 - <csr-id-36bde537ea20079f4923856b149efa99b9957063/> Release thread-ast-engine v0.1.0, thread-language v0.1.0, thread-services v0.1.0, thread-flow v0.1.0, thread-rule-engine v0.1.0, thread v0.1.0

### Bug Fixes

 - <csr-id-b1bd3d4053fdb978e3db6582dbf487c76e119b62/> break circular dev-dependency between ast-engine and language
   thread-ast-engine had thread-language in [dev-dependencies] for its
   performance_improvements bench, but thread-language depends on
   thread-ast-engine as a regular dependency. This circular publish
   dependency caused cargo to abort when publishing thread-ast-engine
   (it searched crates.io for thread-language, which wasn't yet published).
 - <csr-id-ad487324231dd072843972b9d341baae921be248/> Removed asterix dev dependencies

### Chore

 - <csr-id-aac8239716f1a57806c308f0e44e41289e6cc9d0/> Release thread-utilities v0.1.3, thread-ast-engine v0.1.0, thread-language v0.1.0, thread-services v0.1.0, thread-flow v0.1.0, thread-rule-engine v0.1.0, thread v0.1.0

### Refactor

 - <csr-id-659510d42574fe2efe5639bb749d9f29445e7d88/> rename thread-utils to thread-utilities across the codebase. Crates.io namespace conflict required the rename.
   - Updated all occurrences of `thread_utils` to `thread_utilities` in source files, tests, and documentation.
   - Adjusted Cargo.toml files to reflect the new crate name.
 - <csr-id-f9c25705eba3262b41e538a4db64e8d8e1ea4185/> Moved crates to individual versioning; updated ignores accordingly

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 9 commits contributed to the release over the course of 30 calendar days.
 - 9 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#48](https://github.com/knitli/thread/issues/48), [#75](https://github.com/knitli/thread/issues/75)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#48](https://github.com/knitli/thread/issues/48)**
    - Add thread-flow crate, integrates ReCoco (CocoIndex) capabilities into Thread, providing dataflow driven ETL pipeline management ([`d5519df`](https://github.com/knitli/thread/commit/d5519df7aac003c85d71d15b561d29b63fe4c00d))
 * **[#75](https://github.com/knitli/thread/issues/75)**
    - Implement main thread crate to unify crate exposure ([`eead51d`](https://github.com/knitli/thread/commit/eead51dcb62ee0b6fe99856ce40ef929df1c7d41))
 * **Uncategorized**
    - Removed asterix dev dependencies ([`ad48732`](https://github.com/knitli/thread/commit/ad487324231dd072843972b9d341baae921be248))
    - Release thread-services v0.1.0, thread-flow v0.1.0, thread-rule-engine v0.1.0, thread v0.1.0 ([`b97b45c`](https://github.com/knitli/thread/commit/b97b45c95ff854b4bf159bd03ce1e3ff4de74d41))
    - Release thread-ast-engine v0.1.0, thread-language v0.1.0, thread-services v0.1.0, thread-flow v0.1.0, thread-rule-engine v0.1.0, thread v0.1.0 ([`36bde53`](https://github.com/knitli/thread/commit/36bde537ea20079f4923856b149efa99b9957063))
    - Break circular dev-dependency between ast-engine and language ([`b1bd3d4`](https://github.com/knitli/thread/commit/b1bd3d4053fdb978e3db6582dbf487c76e119b62))
    - Release thread-utilities v0.1.3, thread-ast-engine v0.1.0, thread-language v0.1.0, thread-services v0.1.0, thread-flow v0.1.0, thread-rule-engine v0.1.0, thread v0.1.0 ([`aac8239`](https://github.com/knitli/thread/commit/aac8239716f1a57806c308f0e44e41289e6cc9d0))
    - Rename thread-utils to thread-utilities across the codebase. Crates.io namespace conflict required the rename. ([`659510d`](https://github.com/knitli/thread/commit/659510d42574fe2efe5639bb749d9f29445e7d88))
    - Moved crates to individual versioning; updated ignores accordingly ([`f9c2570`](https://github.com/knitli/thread/commit/f9c25705eba3262b41e538a4db64e8d8e1ea4185))
</details>

