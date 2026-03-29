# Context Hygiene

When working in this repository, be aware that AI context files (CLAUDE.md, AGENTS.md, Serena memories, Cursor rules, etc.) can become stale and contradictory. This skill helps you maintain context hygiene.

## Known context file locations

| Tool | Memory files | Config | Output/Planning |
|------|-------------|--------|-----------------|
| Claude Code | `CLAUDE.md` | `.claude/settings*.json`, `.claude/commands/`, `.claude/skills/` | `claudedocs/*` |
| GitHub Copilot/Codex | `AGENTS.md`, `.github/agents/*` | `.codex/config.toml`, `.vscode/mcp.json` | — |
| Gemini | `GEMINI.md` | `.gemini/` | — |
| Cursor | `.cursorrules`, `.cursor/rules/*.mdc` | `.cursor/` | — |
| Serena | `.serena/memories/*` | `.serena/project.yml` | — |
| spec-kit | `.specify/memory/*` | `.specify/templates/*` | `specs/*` |
| Roo/Cline | `.roo/*`, `.clinerules` | `.roo/mcp.json` | — |
| Continue | `.continuerules` | `.continue/config.json` | — |
| Aider | `.aider.conf.yml` | — | — |
| Windsurf | `.windsurfrules` | — | — |

## Rules for maintaining context hygiene

### When you modify code

After making significant changes (renaming files, changing versions, restructuring modules, adding/removing dependencies), check whether any context files reference the things you changed. Update them if so.

Pay special attention to:
- **CLAUDE.md** — often contains path references, version numbers, and structural descriptions
- **AGENTS.md** — if it exists, likely shares content with CLAUDE.md or should be kept in sync
- **Tool-specific memories** (Serena, spec-kit) — may have outdated snapshots of project state

### When you read context files

Treat claims in context files with appropriate skepticism. Cross-reference against actual code before relying on:
- Specific version numbers
- File path references
- Structural counts ("N crates", "N languages")
- Architecture descriptions

The authoritative sources are (in order):
1. The actual source code and manifests
2. Governance documents (constitutions, ADRs)
3. Primary agent memory (CLAUDE.md / AGENTS.md)
4. Tool-specific memories
5. Planning and output docs

### When creating new context

If you create planning docs, completion reports, or similar output:
- Put them in the established directory for your tool (e.g., `claudedocs/` for Claude Code)
- Include a date and reference to what they pertain to
- Don't duplicate information that belongs in the primary memory file
- Consider whether old planning docs should be archived when new ones supersede them

## Run /ctx for a full audit

Use the `/ctx` command to run a comprehensive context hygiene audit anytime. Use `/ctx:check` for a quick staleness check, or `/ctx:drift` to find contradictions between files.
