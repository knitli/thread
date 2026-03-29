# /ctx:drift — Cross-Document Drift Detection

Find contradictions, gaps, and inconsistencies between AI context files from different tool ecosystems.

## Instructions

First, identify all memory and instruction files in the repo (run discovery if needed). Then compare them against each other.

### Comparison process

For each pair of context files that describe the same project:

**1. Identify shared topics.** Do both files discuss: project architecture? Dependencies? Build commands? Directory structure? Language support? Deployment targets? Testing approach?

**2. Compare factual claims on shared topics.**
- Version numbers: do they cite the same versions?
- Architecture descriptions: do they describe the same patterns?
- Technology references: do they name the same tools/frameworks?
- Path references: do they point to the same locations?
- Command references: do they list the same commands?

**3. Flag contradictions.** Where file A explicitly says X and file B explicitly says Y about the same thing. Include both quotes and explain the conflict.

**4. Flag gaps.** Where file A covers a significant topic (a core dependency, a major architectural decision, a deployment model) that file B omits entirely. This usually means file B predates a change. Note which file is likely more current based on modification dates and content depth.

### Also check for

**Orphaned planning docs**: Files in output directories (claudedocs/, plans/, etc.) that:
- Reference phases or milestones that have been superseded
- Contain "COMPLETE" in the filename but describe work that may have been revised since
- Are more than 30 days old and reference specific implementation details

**Symlink freshness**: If GEMINI.md symlinks to CLAUDE.md (or similar), note that they share content. If they DON'T symlink but probably should (identical or near-identical content), flag that too.

**Constitution/governance drift**: If a governance document exists (like a project constitution), check whether memory files align with its stated principles or reference outdated versions.

### Output

Report contradictions and drift warnings individually, with:
- The specific files involved
- The topic of disagreement
- Quotes from each file
- Which is likely correct (based on recency, specificity, or code evidence)
- Recommended resolution

End with a summary: total contradictions, total drift warnings, orphaned doc count.
