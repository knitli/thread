# ctx — Context Hygiene for AI-Assisted Codebases

Your AI agents are working with stale information and they don't know it.

Every AI coding tool writes its own view of your project into markdown files: CLAUDE.md, AGENTS.md, Serena memories, Cursor rules, Codex configs, planning docs, and more. These files silently rot as your code evolves. Versions drift. Paths break. Architecture descriptions fossilize. And each tool's memory diverges from every other tool's memory.

**ctx** finds the problems before your agents act on bad information.

## What it does

- **Discovers** every AI context file in your repo across 10+ tool ecosystems
- **Validates** claims against your actual codebase (paths, versions, symbols, counts, commands)
- **Detects drift** between tool-specific memory files (Claude vs Serena vs Gemini vs Cursor)
- **Fixes** stale references interactively with proposed edits

## Install (Claude Code)

```
/plugin install knitli/ctx
```

Then run:

```
/ctx              # Full audit: discover + check + drift
/ctx:discover     # Inventory all context files
/ctx:check        # Validate claims against codebase
/ctx:drift        # Find contradictions between files
/ctx:fix          # Auto-reconcile fixable issues
```

## Other editors

ctx ships cross-client skills for Cursor, Codex, Roo/Cline, and any tool that reads AGENTS.md. See the `cross-client/` directory.

## Example output

Running `/ctx:discover` on a real codebase:

```
Context File Inventory
══════════════════════

Repository: /home/dev/my-project
Files found: 67 context files across 9 tool ecosystems

 Tool             Files  Key locations
 ──────────────── ───── ─────────────────────
 Claude Code        46   CLAUDE.md, claudedocs/* (42 files)
 Serena              7   .serena/memories/*
 spec-kit            2   .specify/memory/*
 Gemini              1   GEMINI.md (→ symlink to CLAUDE.md)
 Roo/Cline           1   .roo/mcp.json
 Generic             4   AGENTS.md, .mcp.json

 Total size: 847 KB across 67 files
```

Running `/ctx:check` finds the real problems:

```
CLAUDE.md
  ❌ Line 42: rust-version "1.85" → actual: 1.89
  ❌ Line 58: tree-sitter "v0.26.3" → actual: >=0.25.0
  ⚠️ Line 15: "seven main crates" → actual: 8

.serena/memories/project_overview.md
  ❌ Line 18: rust-version "1.85" → actual: 1.89
  ⚠️ Line 8: No mention of ReCoco or dataflow layer

Staleness score: 8.9% of verifiable claims are stale or broken
```

## Supported tool ecosystems

| Tool | Memory files | Config | Planning/Output |
|------|-------------|--------|-----------------|
| Claude Code | `CLAUDE.md` | `.claude/*` | `claudedocs/*` |
| GitHub Copilot/Codex | `AGENTS.md` | `.codex/*`, `.github/agents/*` | — |
| Gemini | `GEMINI.md` | `.gemini/*` | — |
| Cursor | `.cursorrules`, `.cursor/rules/*` | `.cursor/*` | — |
| Serena | `.serena/memories/*` | `.serena/project.yml` | — |
| spec-kit | `.specify/memory/*` | `.specify/templates/*` | `specs/*` |
| Roo/Cline | `.clinerules` | `.roo/*` | — |
| Continue | `.continuerules` | `.continue/*` | — |
| Aider | `.aider.conf.yml` | — | — |
| Windsurf | `.windsurfrules` | — | — |

## How it works

ctx uses Claude's own intelligence to read your context files, extract factual claims, and validate them against the actual codebase. No additional API keys or binary dependencies needed — it piggybacks on whatever client you're already using.

A companion Rust engine (coming soon) adds deterministic structural validation via AST parsing for even faster and more reliable checks.

## Built by [Knitli](https://knitli.com)

ctx is part of the [Thread](https://github.com/knitli/thread) code intelligence platform. Thread provides the AST engine, semantic classification, and incremental analysis infrastructure that powers ctx's deep validation.

## License

MIT
