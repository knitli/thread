# ctx — Context Hygiene for AI-Assisted Codebases

## The plan in plain language

Ship a Claude Code plugin that audits AI context files for staleness, contradictions, and drift. Use that as the onramp and adoption engine. Extend to other clients (Cursor, Codex, Roo, etc.) via their native skill/plugin formats. Back it with Thread's Rust engine once the plugin proves the concept and builds an audience.

---

## Phase 1: Claude Code plugin (target: 1 week)

**What ships**: A Claude Code plugin installable via `/plugin install` that provides:

- `/ctx` — Full audit (discover → check → drift)
- `/ctx:discover` — Inventory all AI context files in the repo
- `/ctx:check` — Validate claims against actual code
- `/ctx:drift` — Find contradictions across tool boundaries
- `/ctx:fix` — Auto-reconcile fixable issues (guided, not autonomous)
- SessionStart hook — Lightweight health check injected at session start
- context-auditor agent — Subagent for deep analysis
- claim-validator agent — Specialized claim-checking subagent
- context-hygiene skill — Passive knowledge about context file patterns

**Why Claude Code first**: Market leader in agentic coding. Plugin ecosystem is mature. Hooks let us run automatically. The LLM *is* the analysis engine — no binary needed for v1. Free piggybacking on the user's existing API keys/subscription.

**Distribution**: GitHub repo → Claude Code marketplace. One-command install.

## Phase 2: Cross-client skills (target: +3-5 days after Phase 1)

**What ships**: Equivalent functionality packaged for each client's native format:

| Client | Format | Location |
|--------|--------|----------|
| Cursor | `.cursor/rules/ctx-audit.mdc` | Auto-attached rule |
| Codex | `.codex/SKILL.md` + skill directory | Codex skill |
| Roo/Cline | `.roo/skills/ctx.md` or `.clinerules` addendum | Client rules |
| Aider | Convention doc referenced in `.aider.conf.yml` | Config reference |
| Generic | `AGENTS.md` section | Universal fallback |

These are all prompt-based — same core logic as the Claude Code skill, adapted to each client's instruction format. No binary dependency. Lower fidelity than the Claude Code plugin (no hooks, limited agent orchestration) but broad reach.

## Phase 3: Thread Rust engine (target: +2-3 weeks)

**What ships**: The `ctx-core` Rust crate that the plugin can shell out to for:

- Fast structural claim extraction (compiled, not LLM-dependent)
- AST-based symbol validation via thread-ast-engine
- Version/dependency validation against Cargo.toml, package.json, pyproject.toml
- Deterministic staleness scoring (same input → same output, no LLM variance)
- JSON output for CI integration

The plugin detects if the binary is available and uses it for structural checks. Falls back to pure-LLM analysis if not installed. This is the upgrade path and the moat.

## Phase 4: Continuous sync + canonical store (target: +1-2 months)

**What ships**: Thread service integration:

- Canonical context store (structured facts with provenance and timestamps)
- Materialization layer (generate tool-specific files on session start)
- Watch mode / daemon for continuous monitoring
- Cross-developer sync for teams
- MCP server exposing context as queryable tool

This is where the product becomes paid. Free CLI + plugin for audit. Paid service for continuous sync, canonical store, and team features.

---

## Plugin architecture

```
plugins/ctx/
├── .claude-plugin/
│   └── plugin.json              # Plugin metadata + marketplace listing
├── commands/
│   ├── ctx.md                   # /ctx — full audit
│   ├── ctx-discover.md          # /ctx:discover — inventory
│   ├── ctx-check.md             # /ctx:check — staleness
│   ├── ctx-drift.md             # /ctx:drift — contradictions  
│   └── ctx-fix.md               # /ctx:fix — auto-reconcile
├── agents/
│   ├── context-auditor.md       # Deep analysis subagent
│   └── claim-validator.md       # Claim checking subagent
├── skills/
│   └── context-hygiene.md       # Context file pattern knowledge
├── hooks/
│   └── session-start-audit.md   # Auto-check on session start
├── cross-client/
│   ├── AGENTS.md                # Universal instruction file
│   ├── .cursor/rules/
│   │   └── ctx-audit.mdc        # Cursor rule
│   └── .codex/
│       └── SKILL.md             # Codex skill  
└── README.md
```

## Content strategy

The Thread repo is the demo. Run ctx on it, screenshot the results, post them.

**Immediate content**:
- "I found 67 AI context files in my repo. Here's what was wrong." (blog/tweet)
- "Your CLAUDE.md is lying to your agents." (provocative hook)
- "The markdown memory crisis nobody's talking about." (longer piece)

**Ongoing**:
- Run ctx on popular open source repos, report findings
- Weekly "context hygiene" tips
- Comparisons of how different tools handle project memory

## Open decisions

1. **Plugin name**: `ctx`, `context-doctor`, `context-lint`, `thread-ctx`?
2. **Repo structure**: Standalone repo or subdirectory of Thread?
3. **License**: MIT for the plugin (max adoption) vs AGPL to match Thread?
4. **Marketplace**: Submit to Claude Code marketplace immediately or build audience first?
5. **Hookify integration**: Use hookify for the session-start hook or write native hook config?
