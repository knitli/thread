# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.1.0 (2026-03-01)

### New Features

 - <csr-id-cadf4b562ccb390180e1bcfa96ed24d68e7d9a75/> improve repo tasks; remove outdated docs
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
 - <csr-id-bb4a9d0f14c61cb4d99e562408335535859cb3fb/> add comprehensive project documentation and development commands

### Bug Fixes

 - <csr-id-9cc123caedfe261bef14b62788bdff85315ee96a/> Corrected categories across crates to be below 5 limit and match crates.io category slugs

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

### Refactor

 - <csr-id-659510d42574fe2efe5639bb749d9f29445e7d88/> rename thread-utils to thread-utilities across the codebase. Crates.io namespace conflict required the rename.
   - Updated all occurrences of `thread_utils` to `thread_utilities` in source files, tests, and documentation.
   - Adjusted Cargo.toml files to reflect the new crate name.
 - <csr-id-f9c25705eba3262b41e538a4db64e8d8e1ea4185/> Moved crates to individual versioning; updated ignores accordingly

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 19 commits contributed to the release.
 - 7 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 3 unique issues were worked on: [#45](https://github.com/knitli/thread/issues/45), [#48](https://github.com/knitli/thread/issues/48), [#75](https://github.com/knitli/thread/issues/75)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#45](https://github.com/knitli/thread/issues/45)**
    - Update rapidhash implementation in thread-utils ([`b830374`](https://github.com/knitli/thread/commit/b8303748688074a04062d77c48472056a7d68fa8))
 * **[#48](https://github.com/knitli/thread/issues/48)**
    - Add thread-flow crate, integrates ReCoco (CocoIndex) capabilities into Thread, providing dataflow driven ETL pipeline management ([`d5519df`](https://github.com/knitli/thread/commit/d5519df7aac003c85d71d15b561d29b63fe4c00d))
 * **[#75](https://github.com/knitli/thread/issues/75)**
    - Implement main thread crate to unify crate exposure ([`eead51d`](https://github.com/knitli/thread/commit/eead51dcb62ee0b6fe99856ce40ef929df1c7d41))
 * **Uncategorized**
    - Rename thread-utils to thread-utilities across the codebase. Crates.io namespace conflict required the rename. ([`659510d`](https://github.com/knitli/thread/commit/659510d42574fe2efe5639bb749d9f29445e7d88))
    - Corrected categories across crates to be below 5 limit and match crates.io category slugs ([`9cc123c`](https://github.com/knitli/thread/commit/9cc123caedfe261bef14b62788bdff85315ee96a))
    - Moved crates to individual versioning; updated ignores accordingly ([`f9c2570`](https://github.com/knitli/thread/commit/f9c25705eba3262b41e538a4db64e8d8e1ea4185))
    - Improve repo tasks; remove outdated docs ([`cadf4b5`](https://github.com/knitli/thread/commit/cadf4b562ccb390180e1bcfa96ed24d68e7d9a75))
    - Docs(templates,claude): align with constitution v2.0.0 - service architecture ([`216c076`](https://github.com/knitli/thread/commit/216c07665b897da2f3742adec3928e4f4f944141))
    - Add comprehensive project documentation and development commands ([`bb4a9d0`](https://github.com/knitli/thread/commit/bb4a9d0f14c61cb4d99e562408335535859cb3fb))
    - Merge pull request #43 from knitli/feat-phase-zero ([`5af12b4`](https://github.com/knitli/thread/commit/5af12b4bd8e08318757fd42676127410d159ee5a))
    - Sped up ext handling in language crate using constants and aho corasick; probably premature optimization, but it is faster ([`413d3bf`](https://github.com/knitli/thread/commit/413d3bf76dce25f59d7c045bd4e4e202e8c12de0))
    - Overhauled documentation in thread-ast-engine ([`a076d02`](https://github.com/knitli/thread/commit/a076d022c69b179ff3f61192f5142c3139d7abbb))
    - Updated reuse compliance ([`7952908`](https://github.com/knitli/thread/commit/795290882740f847de884fd4b7ecfbaf4264dfa3))
    - Linting and formatting ([`34cbd08`](https://github.com/knitli/thread/commit/34cbd0808b70d004f6d6b67896b7e8137fc8e8bd))
    - Added benchmarks to rule_engine, debug/clone consistency ([`6341018`](https://github.com/knitli/thread/commit/6341018feced023641933e291a85da88fb0c6e9d))
    - First analysis of serialization separation; prospect poor. Going to table this for later. ([`6579e15`](https://github.com/knitli/thread/commit/6579e15003065dd2d56344cedc2d05ea0d6151de))
    - About to try separating serialization logic ([`0f758a4`](https://github.com/knitli/thread/commit/0f758a484325e7c9c74ec17efb47a01e0f9ada6f))
    - Removed fastmap implementation in favor of a simpler faster hash pattern ([`bf25a28`](https://github.com/knitli/thread/commit/bf25a2867739a2c056c82d14a20dabe3f9e2a5b7))
    - Third time's the charm on refactoring, it seems ([`0dcb5a3`](https://github.com/knitli/thread/commit/0dcb5a3f1dfadb62787805214a48086697082e60))
</details>

