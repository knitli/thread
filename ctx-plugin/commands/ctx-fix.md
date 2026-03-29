# /ctx:fix — Auto-Reconcile Context Issues

Fix staleness and drift issues found by /ctx:check and /ctx:drift. Operates interactively: proposes each fix and waits for approval before applying.

## Instructions

First, run a full audit (/ctx) to identify all issues. Then work through fixes in priority order.

### Fix priority

1. **Broken claims** (❌) — References to files, symbols, or commands that don't exist. These actively mislead agents.
2. **Version mismatches** — Wrong version numbers are easy to fix and high-impact.
3. **Contradictions between files** — Pick the authoritative source and update the others.
4. **Gaps in secondary files** — Where Serena/Gemini/Cursor memories are missing information that CLAUDE.md or the constitution has.
5. **Orphaned planning docs** — Suggest archival or deletion.

### How to fix

**For each fixable issue**, propose the specific edit:
- Show the current text (with file path and line number)
- Show the proposed replacement
- Explain why this is the correct fix
- Ask for approval before applying

**For version/path/symbol fixes**: The codebase is authoritative. Update the context file to match reality.

**For contradictions across files**: Identify the most authoritative source (usually: constitution > CLAUDE.md > AGENTS.md > tool-specific memories > planning docs). Update less-authoritative files to match.

**For orphaned docs**: Propose moving to an `archive/` directory or deletion. Never delete without explicit approval.

**For gaps**: Propose adding the missing information to the file, sourced from the most authoritative document that has it.

### What NOT to auto-fix

- Architecture descriptions that might intentionally differ between tools (e.g., CLAUDE.md might include service details while a Cursor rule focuses only on code style)
- Files you can't determine the authoritative source for
- Anything that requires understanding intent rather than fact

For these, report the issue and recommend manual review.

### Output

After all fixes are applied (or declined), produce a summary:
- Issues fixed
- Issues skipped (with reasons)
- Issues requiring manual review
- Updated staleness score (before → after)
