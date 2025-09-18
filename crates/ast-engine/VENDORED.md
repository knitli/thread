<!--
SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: MIT OR Apache-2.0
-->
# Our Fork of Ast-Grep

We forked most of the excellent [Ast-Grep][AG] codebase to create Thread. We originally tried using Ast-Grep as a library, but ran into limitations. The `core` module is intended to work as a library, but our plans for Thread required finer control over features at build-time.

While Thread includes a CLI (and that’s likely your first encounter with it), our CLI is just the tip of the iceberg. The real focus is on service-oriented architecture for cloud and automation use.

**We forked at Ast-Grep v0.38.7**. See [the original repo at that version](https://github.com/ast-grep/ast-grep/tree/0.38.7) for reference.

---

## Why We Forked

We tried multiple approaches to integrating Ast-Grep, from working with it as a library with a complex feature-gating scheme, to vendoring and dividing four crates into granular components (14 crates!). That latter one was overkill, and was probably us jumping the shark early :shark:⛷️.

We settled on a middle ground. We forked `core`, `config`, and `language`, and will continue to use `dynamic` and others as dependencies as needed. We also did our best to make as few changes as possible -- mostly focusing on separating features with gating, and abstracting some core elements to better fit our service oriented approach.

Our changes are mostly structural—we needed finer-grained control over organization, minimal cold start times, and clean separation between services.

### Where the Fork Lives

* [`thread-ast-engine`](https://github.com/knitli/thread/tree/main/crates/ast-engine): Fork of `ast-grep-core`. We separated its features into `parsing`, and `matching` features so that we could better control their usage in our services.
* [`thread-rule-engine`](https://github.com/knitli/thread/tree/main/crates/rule-engine): Fork of `ast-grep-config`. We isolated rule management, parsing, and validation functionality, and made changes to separate the logic from the assumption of a config file, allowing us more flexibility to implement rule-based operations in different environments.
* [`thread-language`](https://github.com/knitli/thread/tree/main/crates/language): We changed very little here, we needed the languages publicly exposed to feature gate each one separately. We also plan to add different languages more suitable for our needs.

We admittedly didn't have this conversation with the Ast-Grep contributors, which we will once the dust settles a bit and we can divert attention from delivering an MVP. Our changes are intentionally reversible, and we'd like to find a way to return to using the core crates and contributing there (but that may not be realistic with different goals between the projects).

### Licensing

**Original Ast-Grep code** is MIT-licensed (see the `LICENSE-MIT` file in each crate).
**Our changes and anything Thread-specific** are licensed under the [AGPL v3.0](https://github.com/knitli/thread/blob/main/LICENSE.md).

* If you want pure MIT, use Ast-Grep directly, or cherry-pick the original code. The relationships are:

  * `thread-ast-engine` → `ast-grep-core`
  * `thread-rule-engine` → `ast-grep-config`
  * `thread-language` → `ast-grep-language`

* Using our fork means AGPL; sharing required. If you want to treat your code based on Thread like :ring: Gollum :ring:, [contact us for a commercial license](mailto:licensing@knit.li), and you can keep your *precious*.
* Our project meets the [Reuse Specification](https://reuse.software/). Every file in the project is marked in its header with license information, or with an accompanying `.license` file. Code from `Ast-Grep` will be marked `AGPL-3.0-or-later AND MIT` (this isn't an `or` where you can choose between them).

> Technically, you *can* only use the unchanged Ast-Grep bits under MIT—but you’d need to do the diffing yourself, and you’ll miss out on Thread-specific improvements (not sure why you would do that instead of just forking Ast-Grep...). AGPL means our changes (and anyone else’s) will always be open source.

---

## We're Going to Contribute to Ast-Grep, too

Most of Thread's Ast-Grep codebase is unchanged for now, and where we identify bugs or areas for improvement, we'll submit them upstream under Ast-Grep's MIT license. Similarly, we'll monitor changes to Ast-Grep and incorporate fixes and improvements into Thread.

## So Are You Going to Try to Keep the Changes Minimal Forever?

Probably not. Our first commitment is making Thread as great as we can, even if we diverge from Ast-Grep. We'd love to see the projects grow together, but they may not always align perfectly. Ast-Grep has its own roadmap and priorities, and we have ours. Thread is not Ast-Grep; it is just built on top of it.

## Why Ast-Grep?

Ast-Grep makes [Tree-sitter][ts] actually usable for code search/replace. We built on it because it solved the hard parts—especially CST-wrangling—so we could focus on new stuff, not rebuilding the same wheel.[^1]

> For reasons lost to time, everyone in this ecosystem calls their [CSTs][csts] “ASTs.” Maybe it’s like the first rule of Tree-sitter Club: we all pretend they’re ASTs :fist:.

[^1]: If our initial attempts at integrating Ast-Grep represent how we would reinvent the wheel, we probably would have made our version square and in 15 parts, assembly required.

[AG]: https://github.com/ast-grep/ast-grep
[ts]: https://github.com/tree-sitter/tree-sitter
[csts]: https://en.wikipedia.org/wiki/Concrete_syntax_tree
