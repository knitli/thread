# Thread Context Doctor — Product Spec

**Status**: Draft
**Author**: Adam Poulemanos / Claude brainstorm
**Date**: 2026-03-19
**Target**: First usable release in 2-3 weeks

---

## What this is

A CLI tool that scans a codebase, finds every AI context file (memory files, planning docs, rules, configs), detects what's stale, what contradicts the actual code, and what contradicts other context files. Ships as a Rust binary. Runs in seconds.

The name is TBD. Working candidates: `thread doctor`, `thread ctx`, `ctx`, `ctxlint`. Throughout this doc we'll call it **`ctx`**.

## Why now

Every developer using AI coding tools has this problem today. Nobody has a solution. The tools are proliferating faster than anyone anticipated, and each one writes its own view of project reality into its own files. The result is dozens of markdown files that silently rot and actively mislead agents.

This is the "context layer" problem that Thread was built to solve — applied to the most immediate, painful, universal manifestation of it.

## Who it's for

Any developer using 2+ AI coding tools on a codebase. That's most of them at this point.

---

## Core capabilities (v0.1)

### 1. Discovery

Scan a repo and produce a complete inventory of AI context files.

#### Known tool locations

| Tool | Memory / Instructions | Config / Settings | Planning / Output |
|------|----------------------|-------------------|-------------------|
| **Claude Code** | `CLAUDE.md` | `.claude/settings*.json`, `.claude/commands/`, `.claude/skills/` | `claudedocs/*` |
| **GitHub Copilot / Codex** | `AGENTS.md`, `.github/agents/*`, `.github/skills/*` | `.codex/config.toml`, `.vscode/mcp.json`, `.vscode/settings.json` | — |
| **Google Gemini** | `GEMINI.md` | `.gemini/settings*`, `.gemini/commands/`, `.gemini/skills/` | — |
| **Cursor** | `.cursorrules` (legacy), `.cursor/rules/*.mdc` | `.cursor/settings*` | — |
| **Aider** | `.aider.conf.yml`, `.aider.model.settings.yml` | — | — |
| **Continue** | `.continue/*`, `.continuerules` | `.continue/config.json` | — |
| **Roo / Cline** | `.roo/*`, `.clinerules` | `.roo/mcp.json` | — |
| **Serena** | `.serena/memories/*` | `.serena/project.yml` | — |
| **spec-kit** | `.specify/memory/*` | `.specify/templates/*` | `specs/*` |
| **Windsurf** | `.windsurfrules` | — | — |
| **Generic / shared** | `AGENTS.md` (de facto cross-tool standard) | `.mcp.json` (root MCP config) | `plans/*`, `docs/*`, `info/*` |

#### Heuristic detection

Not everything lands in a known location. The scanner also flags:

- Any markdown file in the repo root that matches AI-context patterns (all-caps name, contains phrases like "when working with this repo", "project overview", "you are an AI", etc.)
- Directories named `planning`, `plans`, `ai-docs`, or similar
- Markdown files with front matter containing `agent`, `llm`, `ai`, or `model` keys
- Symlinks between context files (like GEMINI.md → CLAUDE.md)
- Files matching `*rules*`, `*memory*`, `*context*` patterns in the root or dotfile directories

#### Output: Context inventory

```
$ ctx discover

Thread Context Doctor — Discovery Report
═════════════════════════════════════════

Repository: /home/knitli/thread
Files found: 67 context files across 9 tool ecosystems

 Tool            Files  Locations
 ─────────────── ───── ────────────────────────────
 Claude Code       46   CLAUDE.md, .claude/*, claudedocs/* (42 files)
 Serena             7   .serena/memories/* (6), .serena/project.yml
 spec-kit           2   .specify/memory/*, specs/*
 Gemini             1   GEMINI.md (symlink → CLAUDE.md)
 Roo/Cline          1   .roo/mcp.json
 Shared/Generic     4   AGENTS.md, .mcp.json, info/*
 Heuristic          6   docs/architecture/*, docs/guides/*

 Total size: 847 KB of context across 67 files
 Oldest file: claudedocs/PHASE1_COMPLETE.md (modified 58 days ago)
 Symlinks: GEMINI.md → CLAUDE.md

⚠ 42 files in claudedocs/ — this is unusually high.
  Consider archiving completed phase reports.
```

### 2. Staleness detection

Cross-reference claims in context files against the actual codebase.

#### What counts as a "claim"

A claim is any assertion in a context file that can be validated against code. Examples:

| Claim type | Example | Validation method |
|---|---|---|
| **Path reference** | "the API is in `src/server/main.rs`" | Check if path exists |
| **Module/crate reference** | "thread-language supports 20+ languages" | Count language definitions |
| **Version pin** | "rust-version = 1.85" | Compare to Cargo.toml |
| **Dependency reference** | "tree-sitter (v0.26.3)" | Compare to Cargo.toml/lock |
| **Symbol reference** | "the `ThreadService` facade" | AST search for symbol |
| **Structural claim** | "seven main crates" | Count workspace members |
| **Command reference** | "run `mise run lint`" | Check if mise task exists |

#### Validation pipeline

1. **Extract claims** — Parse markdown, identify code spans, path-like strings, version numbers, crate/module names, command invocations. This is mostly regex + structure-aware markdown parsing, not LLM-dependent.

2. **Resolve claims against code** — Use Thread's AST engine + file system to check:
   - Do referenced paths exist?
   - Do referenced symbols exist? (AST-grep search)
   - Do version numbers match Cargo.toml / package.json / pyproject.toml?
   - Do referenced commands exist in mise.toml / Makefile / package.json scripts?
   - Do structural counts match? (e.g., "7 crates" vs actual workspace members)

3. **Score staleness** — Each claim gets a status:
   - ✅ **Valid** — claim matches code
   - ⚠️ **Stale** — claim references something that has changed
   - ❌ **Broken** — claim references something that doesn't exist
   - ❓ **Unverifiable** — claim is too vague to validate programmatically

#### Output: Staleness report

```
$ ctx check

Thread Context Doctor — Staleness Report
═════════════════════════════════════════

Scanned 67 context files against codebase

CLAUDE.md (14 issues)
  ❌ L42: "rust-version 1.85" — actual: 1.89 (Cargo.toml)
  ❌ L58: "tree-sitter (v0.26.3)" — actual: >=0.25.0 (Cargo.toml)
  ⚠️ L15: "seven main crates" — actual: 8 workspace members
  ⚠️ L102: references `codeweaver_semantic_package/` — directory exists
        but appears to be a leftover (no Cargo.toml inside)
  ...

.serena/memories/project_overview.md (6 issues)
  ❌ L18: "rust-version 1.85" — actual: 1.89
  ❌ L19: "tree-sitter (v0.26.3)" — actual: >=0.25.0
  ⚠️ L8: "forked from ast-grep" — CLAUDE.md says "forked from ast-grep
        and enhanced with ReCoco" (more complete description)
  ...

Summary:
  ✅ Valid:        342 claims
  ⚠️ Stale:         28 claims
  ❌ Broken:        14 claims
  ❓ Unverifiable:  89 claims
  ─────────────────────────
  Staleness score: 8.9% (28+14 of 473 verifiable claims)
```

### 3. Drift detection

Compare context files against each other to find contradictions across tool boundaries.

#### Cross-document comparison

For each pair of context files that describe the same project:

1. **Extract shared topics** — Identify sections/paragraphs that cover the same subject (architecture, dependencies, commands, etc.)
2. **Diff factual content** — Compare concrete claims on shared topics
3. **Flag contradictions** — Where file A says X and file B says Y about the same thing

#### Two modes

**Structural diff** (fast, no LLM): Compare extracted claims directly. If CLAUDE.md says `rust-version = 1.85` and the constitution says `rust-version = 1.89`, that's a structural contradiction. This catches version mismatches, path disagreements, command differences, and structural count mismatches.

**Semantic diff** (optional, uses LLM): Send pairs of related sections to Claude via ReCoco's LLM integration and ask: "Do these two descriptions of the same project contradict each other? If so, how?" This catches subtler issues like one file describing the architecture as "library-first" while another says "service-library dual architecture."

#### Output: Drift report

```
$ ctx drift

Thread Context Doctor — Drift Report
═════════════════════════════════════

Cross-referencing 67 context files for contradictions

CONTRADICTION: rust-version
  CLAUDE.md (L42):                    1.85
  .serena/memories/project_overview:  1.85
  Cargo.toml (actual):               1.89
  ➜ 2 context files are stale. Cargo.toml is authoritative.

CONTRADICTION: tree-sitter version
  CLAUDE.md (L58):                    v0.26.3
  .serena/memories/project_overview:  v0.26.3
  Cargo.toml (actual):               >=0.25.0
  ➜ Context files pin a specific version; Cargo.toml uses range.

DRIFT: Architecture description
  CLAUDE.md:          "service-library dual architecture"
  .serena/memories/:  "code analysis and parsing library" (no mention of service)
  Constitution:       "service-library dual architecture" (v2.0.0)
  ➜ Serena memories predate the v2.0.0 constitutional amendment.
     They describe the old library-first model.

DRIFT: Project description
  CLAUDE.md:          Mentions ReCoco, CocoIndex, dataflow, ETL
  .serena/memories/:  No mention of ReCoco or dataflow
  ➜ Serena memories missing major architectural component.

ORPHAN DETECTION:
  claudedocs/PHASE1_COMPLETE.md — references Phase 1 milestones that
    appear to be superseded by Phase 2 and Phase 5 completion reports.
  claudedocs/DATABASE_OPTIMIZATION_ROADMAP.md — references planned
    optimizations; unclear if completed or abandoned.

Summary:
  Contradictions found:     4
  Drift warnings:           7
  Orphaned/superseded docs: 12
```

---

## Architecture

### Where it lives in Thread

```
crates/
  ctx/                    ← New crate: the CLI binary
    src/
      main.rs             ← CLI entry point (clap)
      discover.rs         ← Discovery engine
      claims.rs           ← Claim extraction from markdown
      check.rs            ← Staleness validation
      drift.rs            ← Cross-document drift detection
      report.rs           ← Output formatting
      patterns.rs         ← Tool location registry
      
  ctx-core/               ← New crate: library (for future MCP/service use)
    src/
      lib.rs
      scanner.rs          ← File discovery logic
      extractor.rs        ← Claim extraction pipeline  
      validator.rs        ← Claim-to-code validation
      comparator.rs       ← Cross-document comparison
      types.rs            ← Claim, ValidationResult, DriftReport, etc.
```

### Why two crates

The CLI (`ctx`) is the ship-fast deliverable. The library (`ctx-core`) is the interface that Thread's service layer, an MCP server, a CI integration, or a VS Code extension can consume later. This follows Thread's own service-library dual architecture.

### Dependencies on existing Thread crates

| Crate | What ctx uses it for |
|---|---|
| `thread-ast-engine` | Symbol existence checks via pattern matching |
| `thread-language` | Language detection for parsed files |
| `thread-utilities` | Fast string operations, hashing |

### Dependencies NOT needed for v0.1

- `thread-flow` / ReCoco — not needed until we add the canonical store and incremental pipeline
- `thread-services` — not needed until the service/daemon mode
- `thread-wasm` — not needed until edge deployment

### New dependencies (minimal)

- `pulldown-cmark` or `comrak` — markdown parsing for claim extraction
- `clap` — CLI argument parsing (likely already in workspace)
- `globset` — for matching file patterns during discovery
- `aho-corasick` (already in workspace) — fast multi-pattern string matching for claim extraction

---

## Claim extraction: the hard part

This is the novel piece. Everything else is plumbing.

### Approach: structure-first, LLM-optional

The extractor works in layers, from cheap/fast to expensive/smart:

#### Layer 1: Structural extraction (no LLM, always runs)

- **Code spans**: Anything in backticks. Extract and classify:
  - Looks like a path? → PathClaim
  - Looks like a command? (starts with `cargo`, `mise`, `npm`, etc.) → CommandClaim  
  - Looks like a symbol? (PascalCase, snake_case, contains `::`) → SymbolClaim
  - Looks like a version? (semver pattern) → VersionClaim
- **Markdown structure**: Headers map to topics. Bullet lists map to enumerations (checkable counts). Tables map to structured data.
- **Front matter**: YAML/TOML front matter in markdown files often contains structured metadata.
- **Quantity claims**: Regex for patterns like "N languages", "N crates", "N+ performance gain" — extract the number and what it's counting.

#### Layer 2: Pattern extraction (no LLM, always runs)

- **"X is Y" patterns**: "Thread is a service-library dual architecture" → DescriptionClaim
- **"X uses Y" patterns**: "the project uses ReCoco for dataflow" → TechnologyClaim  
- **"X lives in Y" patterns**: "deployment materials are in `crates/cloudflare/`" → LocationClaim
- **Temporal markers**: "Last updated: 2026-01-28", "as of Phase 5" → TimestampClaim

These are imperfect. They'll have false positives. That's fine — we mark confidence levels and let the validation step filter.

#### Layer 3: LLM extraction (optional, behind `--deep` flag)

Send sections of context files to Claude with a structured prompt:

> Extract all factual claims from this section that could be validated against
> a codebase. Return as JSON. Each claim should have: the literal text, the
> claim type (path, symbol, version, architecture, dependency, command, count),
> and what you'd check to validate it.

This catches claims that structural parsing misses, like "Thread's core strength is AST-based pattern matching" → ArchitectureClaim that could be cross-referenced against whether the AST engine crate actually contains pattern matching logic.

### Claim data model

```rust
pub struct Claim {
    /// Where this claim was found
    pub source: ClaimSource,
    /// What kind of claim
    pub kind: ClaimKind,
    /// The raw text of the claim
    pub text: String,
    /// Confidence that this is actually a validatable claim (0.0-1.0)
    pub confidence: f32,
    /// Line number in source file
    pub line: usize,
    /// Validation result (filled in by check phase)
    pub status: Option<ValidationStatus>,
}

pub struct ClaimSource {
    pub file: PathBuf,
    pub tool: ToolEcosystem,
    pub section: Option<String>,  // markdown heading context
}

pub enum ClaimKind {
    Path(PathBuf),
    Symbol { name: String, expected_kind: Option<String> },
    Version { package: String, version: String },
    Command { command: String },
    Count { subject: String, expected: usize },
    Description { subject: String, description: String },
    Technology { subject: String, technology: String },
    Location { subject: String, path: PathBuf },
}

pub enum ValidationStatus {
    Valid,
    Stale { actual: String },
    Broken { reason: String },
    Unverifiable,
}

pub enum ToolEcosystem {
    ClaudeCode,
    GithubCopilot,
    Codex,
    Gemini,
    Cursor,
    Aider,
    Continue,
    Roo,
    Serena,
    SpecKit,
    Windsurf,
    Generic,
    Unknown,
}
```

---

## What v0.1 does NOT do

To ship fast, we explicitly defer:

- **Auto-fix / reconciliation** — v0.1 reports problems, doesn't fix them. Fixing is v0.2.
- **Canonical store** — No unified context database yet. That's the Thread service layer.
- **Materialization** — No generating tool-specific files from a canonical source. That's v0.3+.
- **MCP server** — No serving context via MCP. That's after the canonical store exists.
- **CI integration** — No GitHub Action or pre-commit hook. That's a fast follow once the CLI works.
- **Watch mode / daemon** — No continuous monitoring. That's the Thread service mode.
- **Cross-repo** — Single repo only. Multi-repo is a Thread service feature.

---

## Shipping plan

### Week 1: Discovery + scaffolding

- [ ] Create `ctx` and `ctx-core` crates in the Thread workspace
- [ ] Implement tool location registry (the table above, as data)
- [ ] Implement file scanner with glob patterns + heuristics
- [ ] Implement symlink detection
- [ ] CLI with `ctx discover` subcommand
- [ ] Test against Thread repo itself as dogfood

### Week 2: Staleness detection

- [ ] Implement markdown parser → claim extractor (Layer 1 + 2)
- [ ] Implement path validator (does this file/dir exist?)
- [ ] Implement version validator (compare against Cargo.toml, package.json)
- [ ] Implement symbol validator (AST-grep search for referenced symbols)
- [ ] Implement command validator (check mise.toml, Makefile, package.json scripts)
- [ ] Implement count validator (workspace members, language count, etc.)
- [ ] CLI with `ctx check` subcommand
- [ ] Test against Thread repo — the known issues (rust-version, tree-sitter version) should surface

### Week 3: Drift detection + polish

- [ ] Implement structural cross-document comparison
- [ ] Implement orphan/superseded document detection (based on timestamps and naming)
- [ ] Optional: LLM-powered semantic drift detection behind `--deep` flag
- [ ] CLI with `ctx drift` subcommand
- [ ] `ctx` (no subcommand) runs all three: discover → check → drift
- [ ] Output formatting: terminal colors, optional JSON output for tooling
- [ ] README, demo, first blog post using Thread repo as case study

---

## Content / marketing angle

The Thread repo itself is the perfect demo:

> "I ran ctx on my own repo. I found 67 AI context files, 14 broken claims,
> and 4 contradictions between my Claude, Serena, and Gemini memories.
> My agents have been working with stale information for weeks."

That's the tweet. That's the blog post. That's the conference talk.

The discovery output alone — just showing people how much context spam they have — is shareable content. Screenshots of `ctx discover` on popular open source repos would make great threads.

---

## Open questions

1. **Name**: `ctx` is clean and short but might conflict with existing tools. `thread doctor` ties it to Thread but is longer. `ctxlint` positions it as a linter. What feels right?

2. **Distribution**: Cargo install? Homebrew? Standalone binary releases? All three eventually, but what's fastest for initial adoption?

3. **Language support**: v0.1 should handle at minimum Rust (Cargo.toml), JavaScript/TypeScript (package.json), and Python (pyproject.toml) for version/dependency validation. Others can follow.

4. **LLM integration**: The `--deep` flag is optional but powerful. Should v0.1 ship with it, or is it a distraction from the "ship fast" goal? ReCoco already has the plumbing, so it might be low-effort to include.

5. **Output format**: Terminal-pretty by default, `--json` for tooling, `--markdown` for including in PRs/issues? All three seem necessary fairly quickly.

6. **Pricing model**: The CLI is free/open source. What's the paid upgrade path? Daemon mode? Team sync? Cloud dashboard? SaaS API?
