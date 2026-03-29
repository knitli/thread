# /ctx — Full Context Audit

Run a complete context hygiene audit: discover all AI context files, check them for staleness against the actual codebase, and detect drift/contradictions across tool boundaries.

## Instructions

You are running a context hygiene audit on this repository. Complete all three phases in order, then produce a summary report.

### Phase 1: Discovery

Scan the repository for ALL AI context files. Check every location in this list:

**Memory / instruction files** (check repo root and subdirectories):
- `CLAUDE.md`, `AGENTS.md`, `GEMINI.md` — root-level agent memory files
- `.github/agents/*`, `.github/skills/*` — GitHub Copilot agent config
- `.claude/skills/`, `.claude/commands/`, `.claude/analyze_conversation.md` — Claude Code skills and commands
- `.gemini/skills/`, `.gemini/commands/` — Gemini CLI config
- `.codex/`, `.codex/AGENTS.md`, `.codex/config.toml` — OpenAI Codex config
- `.cursor/rules/*.mdc`, `.cursorrules` — Cursor rules (legacy and current)
- `.continue/*`, `.continuerules` — Continue config
- `.roo/*`, `.clinerules` — Roo/Cline config
- `.serena/memories/*`, `.serena/project.yml` — Serena MCP tool
- `.specify/memory/*`, `.specify/templates/*` — spec-kit
- `.windsurfrules` — Windsurf
- `.aider.conf.yml`, `.aider.model.settings.yml` — Aider
- `ai-rules.yml`, `.ai-rules.yml` — generic AI rules

**Planning and output directories**:
- `claudedocs/*` — Claude Code planning docs
- `specs/*` — specification documents
- `plans/*`, `planning/*` — planning directories
- `docs/*` — documentation (flag files that appear AI-generated or agent-targeted)
- `info/*` — informational docs

**Config files**:
- `.mcp.json` — root MCP server config
- `.vscode/mcp.json`, `.vscode/settings.json` — VS Code / Copilot MCP config

**Heuristic checks**:
- Any markdown file in repo root with all-caps filename not in the standard set (README, LICENSE, CHANGELOG, CONTRIBUTING, SECURITY, CODE_OF_CONDUCT)
- Symlinks between context files (e.g., GEMINI.md → CLAUDE.md)
- Files containing phrases like "when working with this repo", "you are an AI", "assistant instructions"

For each file found, record: path, tool ecosystem, file type (memory/config/planning/output), size, last modified date.

Produce the discovery inventory as a formatted table.

### Phase 2: Staleness Check

For each **memory and instruction file** found in Phase 1 (skip config-only files and old planning docs), extract factual claims and validate them against the actual codebase.

**Claims to extract and validate**:

1. **Path references** — Any file path or directory path mentioned. Check: does it exist?
2. **Version numbers** — Rust edition, rust-version, dependency versions. Check against: `Cargo.toml`, `package.json`, `pyproject.toml`, `go.mod`, or equivalent.
3. **Dependency names and versions** — Named dependencies with pinned versions. Check against lock files and manifests.
4. **Symbol references** — Type names, function names, module names in backticks. Check: do they exist in the codebase? Search for them.
5. **Structural claims** — "N crates", "N languages", "seven main modules". Check: count the actual items.
6. **Command references** — Build/test/lint commands. Check: do they exist in `mise.toml`, `Makefile`, `package.json` scripts, `justfile`, etc.?
7. **Architecture descriptions** — Technology claims ("uses ReCoco", "built with tree-sitter"). Check: is that dependency actually present?

For each claim, assign a status:
- ✅ **Valid** — matches the codebase
- ⚠️ **Stale** — references something that has changed (provide the actual value)
- ❌ **Broken** — references something that doesn't exist at all
- ❓ **Unverifiable** — too vague to check programmatically

Report findings grouped by file, with line numbers where possible.

### Phase 3: Drift Detection

Compare memory/instruction files **against each other** to find contradictions.

For each pair of files that describe the same project:

1. **Identify shared topics** — Both describe the architecture? Both list dependencies? Both mention build commands?
2. **Compare claims on shared topics** — Do they agree? Where do they disagree?
3. **Flag contradictions** — Where file A says X and file B says Y about the same thing.
4. **Flag gaps** — Where file A covers a major topic (like a core dependency or architectural pattern) that file B omits entirely. This often means file B predates a significant change.

Also check for **orphaned planning docs**:
- Files in `claudedocs/` or similar that reference completed/superseded phases
- Planning docs whose proposed changes appear to have been implemented (or abandoned)
- Multiple versions of the same plan without clear resolution

### Summary Report

Produce a final summary with:

1. **Discovery stats**: Total files, breakdown by tool ecosystem, total size
2. **Staleness score**: X of Y verifiable claims are valid. Percentage.
3. **Critical issues**: The most important things to fix (broken paths, wrong versions, major contradictions)
4. **Drift summary**: Which files contradict each other and on what
5. **Recommendations**: Prioritized list of what to fix first
6. **Maintenance suggestions**: Files that should be archived, consolidated, or deleted

Format the report clearly with headers and use emoji status indicators (✅ ⚠️ ❌ ❓) for scannability.
