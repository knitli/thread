# Context Hygiene Skill

## name
context-hygiene

## description
Audit and maintain AI context files across tool ecosystems. Detects stale claims, contradictions between CLAUDE.md/AGENTS.md/Serena/Cursor/etc., and validates references against the actual codebase.

## instructions

When activated, perform a context hygiene audit on the current repository:

### 1. Discovery
Scan for AI context files across all known tool ecosystems: CLAUDE.md, AGENTS.md, GEMINI.md, .claude/, .gemini/, .codex/, .cursor/, .serena/, .specify/, .roo/, .continue/, claudedocs/, specs/, plans/, docs/, info/, .mcp.json, .cursorrules, .windsurfrules, .continuerules, .clinerules, .aider.conf.yml

Report what you find as an inventory grouped by tool.

### 2. Staleness check
For each memory/instruction file, extract factual claims (paths, versions, symbols, counts, commands, dependencies) and validate them against the actual codebase. Report what's valid, what's stale, what's broken.

### 3. Drift detection
Compare memory files against each other to find contradictions and gaps. Flag where different tools have divergent views of the same project.

### 4. Summary
Produce a prioritized list of issues to fix, with specific file paths, line numbers, and recommended corrections.

### After code changes
When you make significant changes to file structure, dependencies, versions, or architecture, proactively check whether any context files need updating. The most important files to keep current are the root-level memory files (CLAUDE.md, AGENTS.md).
