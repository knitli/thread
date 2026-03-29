---
name: ctx-session-health-check
enabled: true
event: stop
conditions:
  - field: transcript
    operator: not_contains
    pattern: /ctx|context audit|context hygiene|staleness check
action: warn
---

📋 **Context hygiene reminder**

You completed work in this session without running a context audit. If you modified code structure, versions, dependencies, or file locations, the project's AI context files (CLAUDE.md, AGENTS.md, Serena memories, etc.) may now be stale.

Run `/ctx:check` to validate context files against the current codebase, or `/ctx` for a full audit.
