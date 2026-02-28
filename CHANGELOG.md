<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [Unreleased]
### Bug Fixes

- Lockfile errors([`3723c40`](https://github.com/knitli/thread/commit/3723c40012db5b0bf11fb394bf3894f5f227429a))

- (**tests**) Fixed failing tests and clarified language in planning docs([`badc089`](https://github.com/knitli/thread/commit/badc08904d9be9ba231afd0ace1568d33ce97f46))

- (**tests**) Fixed several test and typing issues([`f287cb4`](https://github.com/knitli/thread/commit/f287cb44a12f3e0c3ae4270363f1682ba66f07b1))

- Missing re-export causing failing tests. All tests across codebase now passing([`1a4f08f`](https://github.com/knitli/thread/commit/1a4f08f1c7eabae45aea5e8786ac410958b8d5b3))

- Multiple test and typing fixes. test suite now all green.([`400eb5a`](https://github.com/knitli/thread/commit/400eb5a97c8409b2308320a1e6551d5327f9c41b))

- Fixed issue where all-features caused failures in CI due to conflicting feature flags([`73ddabc`](https://github.com/knitli/thread/commit/73ddabc786557d8c5db9d8766c7717a3ea427f31))

- (**ci**) Correct issue where Rust stable causes lint failures (thread-services uses experimental trait aliases)([`10e86ef`](https://github.com/knitli/thread/commit/10e86ef85840ed2a3e3a0cf9bd7e01f51b4de4ab))

- (**ci**) Remove cargo license check from CI; redundant with cargo deny, which is more reliable([`81dfb6e`](https://github.com/knitli/thread/commit/81dfb6ec8fa1f72d40f6c4cc66ed2fe90f90fcdd))

- (**ci**) Remove Semgrep-SAST check; semgrep-action is deprecated and caused failing CI from deprecation warnings([`e2e5f1a`](https://github.com/knitli/thread/commit/e2e5f1a0b79d7ca7558dde87197529630c05b5ba))

- (**lint**) Fix lint warnings([`4a8d31c`](https://github.com/knitli/thread/commit/4a8d31c2cf1d5e8c82dc8e741473d5ac657fdfe9))

- Developer error with mimalloc usage :)([`fc93538`](https://github.com/knitli/thread/commit/fc93538f940e1499a93098a9d0a7f6162d3f19d7))

### Documentation

- Add AI-native knowledge layer architectural design report (#71)([`591f187`](https://github.com/knitli/thread/commit/591f187e8b41e61605459dccbbcf5431e604cd42))

### Features

- Add semantic classifications and spec in preparation for semantic classification layer([`e308eb2`](https://github.com/knitli/thread/commit/e308eb27f28859aa5fc101723100ecc0892e337f))

- (**thread**) Implement main thread crate to unify crate exposure([`42c5239`](https://github.com/knitli/thread/commit/42c5239773dfe02a7543f725431131b281294fc0))

- Add support for terraform, nix, and solidity; update language tests([`cf30a27`](https://github.com/knitli/thread/commit/cf30a27cc3c0b723fa8f34ab717638b897e38f9a))

- Add support for terraform, nix, and solidity; update language tests([`090a285`](https://github.com/knitli/thread/commit/090a2852083eefbb12fb2d7c254fc49f7dd334f9))

- Replace HashMap/Set usage with RapidMap/Set across flow crate; linting([`082bfa4`](https://github.com/knitli/thread/commit/082bfa42891ca0a270db81799031881cba2b9487))

### Miscellaneous

- Update CLAUDE.md([`11826f9`](https://github.com/knitli/thread/commit/11826f9dde6d0e59d9fb6b980321749d803b654b))

- (**docs**) Substantial updates to 001-planning to reflect revisions, requirement changes, and the Thread-Flow implementation([`8e9e7f5`](https://github.com/knitli/thread/commit/8e9e7f535223221d4cb5c0f45f574558483dd2b2))

- (**ci**) Update cargo-deny to ignore trivial dependency lockfile duplications([`0da1c6a`](https://github.com/knitli/thread/commit/0da1c6a7f39640007534fcad20f3bfea338c70b3))

- (**lint**) Formatting and minor fixes([`39d90b3`](https://github.com/knitli/thread/commit/39d90b3993d00f489243e46e03affb1d11d21185))

- (**fmt**) Formatted codebase([`6c1063b`](https://github.com/knitli/thread/commit/6c1063b82104aa09e026cc45def20575702539f7))


## [0.2.0] - 2026-01-31
### Bug Fixes

- Correct Authorization header syntax in CLA workflow([`7962d8f`](https://github.com/knitli/thread/commit/7962d8f1eae6ca66cede29412d83ab250f81893d))

### Feat

- (**flow**) Add thread-flow crate, integrates ReCoco (CocoIndex) capabilities into Thread, providing dataflow driven ETL pipeline management (#48)([`d5519df`](https://github.com/knitli/thread/commit/d5519df7aac003c85d71d15b561d29b63fe4c00d))

### Features

- Optimize extension matching with aho-corasick and character bucketing([`ea98e37`](https://github.com/knitli/thread/commit/ea98e375724c0a81c6a9b0a4900d37d67694d420))

- Add length-based bucketing optimization for extension matching([`7784283`](https://github.com/knitli/thread/commit/77842833cc3db31b73ddae5148f5915287ea4dde))

- Add comprehensive project documentation and development commands([`bb4a9d0`](https://github.com/knitli/thread/commit/bb4a9d0f14c61cb4d99e562408335535859cb3fb))

- Add CocoIndex Rust API documentation and resources([`b332540`](https://github.com/knitli/thread/commit/b332540ae2552b2705b1fe75fd55fa304363e2cb))

- Add initial specification and quality checklist for Real-Time C… (#47)([`c146c69`](https://github.com/knitli/thread/commit/c146c694bedb22d87612e22d2e6b96138fd9672f))

### Miscellaneous

- Update font([`add7064`](https://github.com/knitli/thread/commit/add7064b74e72a195096ef09e0fd005b4b485415))

### Refactoring

- Update Cargo.toml for improved workspace configuration and dependency management([`d7e9d0d`](https://github.com/knitli/thread/commit/d7e9d0dfd8503d6b93d6a0c4d79eeef909a9fc7f))

- (**mcp.json, settings.json**) Remove obsolete filesystem server and update allowed models([`c507148`](https://github.com/knitli/thread/commit/c507148d816880a571bead4ce1ed60dd1d4bf197))



