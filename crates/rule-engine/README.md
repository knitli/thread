<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: AGPL-3.0-or-later AND MIT
-->

# thread-rule-engine

Rule-based scanning and transformation engine for Thread.

Provides YAML-configurable rule definitions for code analysis using AST pattern matching.

## Overview

`thread-rule-engine` lets you define static analysis rules declaratively in YAML. Rules specify
patterns to match, messages to emit, optional fixes, and severity levels—all without writing
custom Rust code.

It is a fork of the `ast-grep-rules` engine, enhanced for Thread's service-library architecture.

## Quick Start

### Define a Rule in YAML

```yaml
id: no-var-declarations
message: "Use 'let' or 'const' instead of 'var'"
language: JavaScript
severity: warning
rule:
  pattern: "var $NAME = $VALUE"
fix: "let $NAME = $VALUE"
```

### Use Rules in Rust

```rust
use thread_rule_engine::RuleConfig;

let rule: RuleConfig = serde_yaml::from_str(r#"
id: no-console-log
message: "Remove debug console.log calls before committing"
language: JavaScript
severity: error
rule:
  pattern: "console.log($$$ARGS)"
"#)?;

// Apply rule to an AST root
let matches = rule.find_all(&root);
for m in matches {
    println!("{}: {}", m.rule_id(), m.message());
}
```

## Rule Schema

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | ✅ | Unique rule identifier |
| `message` | string | ✅ | Human-readable violation message |
| `language` | string | ✅ | Target language (e.g. `Rust`, `JavaScript`) |
| `severity` | string | ✅ | `error`, `warning`, `info`, or `hint` |
| `rule` | object | ✅ | Matching criteria (`pattern`, `kind`, `any`, `all`, …) |
| `fix` | string | — | Replacement template to auto-fix violations |
| `note` | string | — | Additional context for the user |

### Composite Rules

```yaml
id: bad-equality
message: "Prefer '===' over '=='"
language: JavaScript
severity: warning
rule:
  any:
    - pattern: "$A == $B"
    - pattern: "$A != $B"
```

## Feature Flags

- **`default`** — Includes pattern matching (`thread-ast-engine/matching`)

## Related Crates

- [`thread-ast-engine`](../ast-engine) — Core AST engine providing pattern matching primitives
- [`thread-language`](../language) — Language parsers used by rules
- [`thread-services`](../services) — High-level service layer that orchestrates rule scanning

## License

AGPL-3.0-or-later AND MIT

