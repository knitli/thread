<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-License-Identifier: MIT OR Apache-2.0
-->
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [Unreleased]
### Bug Fixes

- Lockfile errors([`3723c40`](https://github.com/knitli/thread/commit/3723c40012db5b0bf11fb394bf3894f5f227429a))

### Documentation

- Add AI-native knowledge layer architectural design report (#71)([`591f187`](https://github.com/knitli/thread/commit/591f187e8b41e61605459dccbbcf5431e604cd42))

### Features

- Add semantic classifications and spec in preparation for semantic classification layer([`e308eb2`](https://github.com/knitli/thread/commit/e308eb27f28859aa5fc101723100ecc0892e337f))

- (**thread**) Implement main thread crate to unify crate exposure (#75)([`eead51d`](https://github.com/knitli/thread/commit/eead51dcb62ee0b6fe99856ce40ef929df1c7d41))

### Miscellaneous

- Update CLAUDE.md([`11826f9`](https://github.com/knitli/thread/commit/11826f9dde6d0e59d9fb6b980321749d803b654b))

- (**docs**) Substantial updates to 001-planning to reflect revisions, requirement changes, and the Thread-Flow implementation([`8e9e7f5`](https://github.com/knitli/thread/commit/8e9e7f535223221d4cb5c0f45f574558483dd2b2))


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



