# Our Fork of Ast-Grep

We forked most of the excellent [Ast-Grep][AG] codebase to create Thread. We originally tried using Ast-Grep as a library, but ran into limitations: it’s structured as a CLI-first tool, and pulling in core functionality meant inheriting a lot of CLI baggage. Too much friction for our cloud-native use case.

While Thread includes a CLI (and that’s likely your first encounter with it), our CLI is just the tip of the iceberg. The real focus is on service-oriented architecture for cloud and automation use.

**We forked at Ast-Grep v0.38.7**. See [the original repo at that version](https://github.com/ast-grep/ast-grep/tree/0.38.7) for reference.

---

## Why We Forked

Our changes are mostly structural—we needed finer-grained control over code organization, minimal cold start times, and clean separation between services.

### Where the Fork Lives

* [`thread-languages`](https://github.com/knitli/thread/tree/main/crates/languages): Fork of `ast-grep-language`. We added feature flags per language and made language registration extensible.
* [`thread-threadlang`](https://github.com/knitli/thread/tree/main/crates/threadlang): Originally a submodule of `ast-grep-language` (`SgLang` type, now `ThreadLang`), plus some refactored CLI modules. We wanted a clean, extensible type for language plumbing.
* [`ag-service-*` crates](https://github.com/knitli/thread/tree/main/crates/ag-thread): These are our split-out versions of Ast-Grep’s `cli`, `core`, and `config` crates, abstracted into 8 focused crates (see below). On crates.io, these are all in the `ag-service-*` family.

**What’s in ag-service-\*:**

* [`ag-service-ast`](https://crates.io/crates/ag-service-ast) – from `ast-grep-core`’s core AST types (`SgNode`, `Node`, `Content`, `Doc`)
* [`ag-service-check-rule`](https://crates.io/crates/ag-service-check-rule) – from `ast-grep-cli`’s `verify`
* [`ag-service-core`](https://crates.io/crates/ag-service-core) – from `ast-grep-core`
* [`ag-service-fix`](https://crates.io/crates/ag-service-fix) – from `ast-grep-config`’s fixer/transform
* [`ag-service-label`](https://crates.io/crates/ag-service-label) – from `ast-grep-config`’s label
* [`ag-service-ops`](https://crates.io/crates/ag-service-ops) – from `ast-grep-core`'s `ops` module (supporting matching operations).
* [`ag-service-pattern`](https://crates.io/crates/ag-service-pattern) – from `ast-grep-core`’s pattern matching logic (`Pattern`, `NodeMatch`, etc.)
* [`ag-service-rule`](https://crates.io/crates/ag-service-rule) – main rule/config logic
* [`ag-service-scan`](https://crates.io/crates/ag-service-scan) – from `ast-grep-cli`’s scan
* [`ag-service-transform`](https://crates.io/crates/ag-service-transform) – `ast-grep-core`'s transform and replace logic
* [`ag-service-tree-sitter`](https://crates.io/crates/ag-service-tree-sitter) – from `ast-grep-core`’s Tree-sitter integration, with tree traversal and query logic
* [`ag-service-types`](https://crates.io/crates/ag-service-types) – (nearly) all *unimplemented* structs, enums, and traits from `ast-grep-core` that we use in our services. It's a large crate, but just a type library. Each module is feature-gated anyway so you can use only what you need.
* [`ag-service-utils`](https://crates.io/crates/ag-service-utils) – from `ast-grep-cli`’s utils

> We didn’t move *all* language support into the `ag-service-*` crates because language support is a core part of Thread, and we wanted it fully extensible.

---

## Licensing

**Original Ast-Grep code** is MIT-licensed (see the `LICENSE` file in each crate).
**Our changes and anything Thread-specific** are licensed under the [AGPL v3.0](https://github.com/knitli/thread/blob/main/LICENSE.md).

* If you want pure MIT, use Ast-Grep directly, or cherry-pick the original code.
* Using our fork means AGPL. If AGPL’s not your speed, [contact us for a commercial license](mailto:licensing@knit.li).

> Technically, you can use only the unchanged Ast-Grep bits under MIT—but you’d need to do the diffing yourself, and you’ll miss out on Thread-specific improvements. AGPL means our changes (and anyone else’s) will always be open source.

---

## Why Ast-Grep?

Ast-Grep makes [Tree-sitter][ts] actually usable for code search/replace. We built on it because it solved the hard parts—especially CST-wrangling—so we could focus on new stuff, not rebuilding the same wheel.

> For reasons lost to time, everyone in this ecosystem calls their CSTs “ASTs.” Maybe it’s like the first rule of Tree-sitter Club: we all pretend they’re ASTs.

[AG]: https://github.com/ast-grep/ast-grep
[ts]: https://github.com/tree-sitter/tree-sitter
