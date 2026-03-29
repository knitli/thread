# Context Hygiene Instructions

This section teaches any AI coding agent how to audit and maintain the project's AI context files.

## Context file awareness

This repository has AI context files from multiple tools. These files can become stale and contradict each other. Before relying on information in any context file, verify key claims against the actual codebase.

### Known context file locations in this repo

Check for files in: `CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `.claude/`, `.gemini/`, `.codex/`, `.cursor/`, `.serena/`, `.specify/`, `.roo/`, `claudedocs/`, `specs/`, `docs/`, `info/`, `.mcp.json`, `.cursorrules`, `.windsurfrules`, `.continuerules`, `.clinerules`

### Common staleness patterns

- **Version drift**: Context files reference old versions of dependencies or toolchains. Always check the actual `Cargo.toml`, `package.json`, or equivalent.
- **Path rot**: Context files reference files or directories that have been moved, renamed, or deleted.
- **Architecture drift**: Context files describe an older architecture that has been superseded by a constitutional amendment or major refactor.
- **Count drift**: "N crates" or "supports N languages" — the actual count has changed.
- **Tool memory divergence**: Each tool's memory files (Claude's, Serena's, Gemini's, etc.) describe the project slightly differently, and the differences grow over time.

### After making significant changes

When you change file paths, dependency versions, module structure, or architectural patterns, check whether any context files reference the things you changed. The most important files to keep current are the root-level memory files (CLAUDE.md, AGENTS.md) since other tools often read these.

### Authoritative sources (in order)

1. Actual source code and package manifests
2. Governance documents (constitutions, ADRs)
3. Primary agent memory files (CLAUDE.md, AGENTS.md)
4. Tool-specific memory files
5. Planning and output documents
