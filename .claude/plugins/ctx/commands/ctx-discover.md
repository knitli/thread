# /ctx:discover — Context File Inventory

Scan this repository and produce a complete inventory of AI context files across all tool ecosystems.

## Instructions

Search the following locations. For each file found, record its path, which tool ecosystem it belongs to, its type (memory, config, planning, or output), file size, and last-modified date.

### Locations to scan

**Root-level memory files**: `CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `.cursorrules`, `.windsurfrules`, `.continuerules`, `.clinerules`, `ai-rules.yml`, `.ai-rules.yml`, `.aider.conf.yml`, `.aider.model.settings.yml`

**Tool directories**: `.claude/`, `.gemini/`, `.codex/`, `.cursor/`, `.continue/`, `.roo/`, `.serena/`, `.specify/`, `.github/agents/`, `.github/skills/`

**Planning/output directories**: `claudedocs/`, `specs/`, `plans/`, `planning/`, `info/`

**Config files**: `.mcp.json`, `.vscode/mcp.json`, `.vscode/settings.json`

**Heuristic detection**: Scan for any additional markdown files in the repo root with ALL_CAPS names that aren't standard (README, LICENSE, CHANGELOG, CONTRIBUTING, SECURITY, CODE_OF_CONDUCT, VENDORED). Also check for symlinks between any of the above files.

Check the `docs/` directory for files that appear to be AI-agent-targeted (contain phrases like "project overview", "architecture overview", "when working with", etc.) rather than user-facing documentation.

### Output format

Produce a table grouped by tool ecosystem showing all discovered files, then a summary with:
- Total count and total size
- Breakdown by ecosystem
- Oldest and newest files
- Any symlinks detected
- Any unusually large directories (10+ files in a planning/output dir)
