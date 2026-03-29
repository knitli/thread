# /ctx:check — Staleness Validation

Validate claims in AI context files against the actual codebase. Find what's stale, what's broken, and what's drifted from reality.

## Instructions

First, run discovery (same as /ctx:discover) to identify all context files. Then, for each **memory and instruction file** (skip config-only files and old planning output), do the following:

### Extract and validate claims

Read each file and extract every factual claim that can be checked against the codebase:

1. **Paths**: Any file or directory path in backticks or quotes. → Does it exist?
2. **Versions**: Semver strings, edition years, rust-version values. → Compare against manifest files (Cargo.toml, package.json, pyproject.toml, go.mod).
3. **Dependencies**: Named packages with version constraints. → Compare against manifests and lock files.
4. **Symbols**: Type names, function names, module names, crate names in backticks. → Search the codebase for their existence.
5. **Counts**: "N crates", "supports N languages", "seven modules". → Count the actual items.
6. **Commands**: Build/test/lint commands in code blocks. → Check task runner configs (mise.toml, Makefile, package.json scripts, justfile).
7. **Technology claims**: "uses X", "built with Y", "powered by Z". → Verify the dependency/tool is actually present.

### Assign status to each claim

- ✅ **Valid** — the claim matches current codebase state
- ⚠️ **Stale** — the claim was probably once true but the codebase has changed (show actual value)
- ❌ **Broken** — the claim references something that doesn't exist
- ❓ **Unverifiable** — too vague or abstract to validate programmatically

### Output

Report findings grouped by file, with line numbers. Include a summary showing:
- Total claims checked
- Breakdown by status (valid/stale/broken/unverifiable)
- Overall staleness percentage (stale + broken out of verifiable claims)
- Top 5 most critical issues to fix
