<!--
SPDX-FileCopyrightText: 2026 Github

SPDX-License-Identifier: MIT
-->

# Implementation Plan: [FEATURE]

**Branch**: `[###-feature-name]` | **Date**: [DATE] | **Spec**: [link]
**Input**: Feature specification from `/specs/[###-feature-name]/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

[Extract from feature spec: primary requirement + technical approach from research]

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: [e.g., Python 3.11, Swift 5.9, Rust 1.75 or NEEDS CLARIFICATION]  
**Primary Dependencies**: [e.g., FastAPI, UIKit, LLVM or NEEDS CLARIFICATION]  
**Storage**: [if applicable, e.g., PostgreSQL, CoreData, files or N/A]  
**Testing**: [e.g., pytest, XCTest, cargo test or NEEDS CLARIFICATION]  
**Target Platform**: [e.g., Linux server, iOS 15+, WASM or NEEDS CLARIFICATION]
**Project Type**: [single/web/mobile - determines source structure]  
**Performance Goals**: [domain-specific, e.g., 1000 req/s, 10k lines/sec, 60 fps or NEEDS CLARIFICATION]  
**Constraints**: [domain-specific, e.g., <200ms p95, <100MB memory, offline-capable or NEEDS CLARIFICATION]  
**Scale/Scope**: [domain-specific, e.g., 10k users, 1M LOC, 50 screens or NEEDS CLARIFICATION]

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Service-Library Architecture ✅/❌

- [ ] **Library Core**: Feature includes reusable library crates (not just service code)
- [ ] **Service Layer**: Feature includes service orchestration/persistence if applicable
- [ ] **Dual Consideration**: Design addresses both library API and service deployment

**Justification if violated**: [Explain why feature is library-only or service-only]

### II. Performance & Safety ✅/❌

- [ ] **Unsafe Code**: No unsafe blocks OR explicitly justified with safety invariants
- [ ] **Benchmarks**: Performance-critical paths include benchmark suite
- [ ] **Memory Efficiency**: Allocations in hot paths are justified

**Justification if violated**: [Explain performance/safety trade-offs]

### III. Test-First Development (NON-NEGOTIABLE) ✅/❌

- [ ] **TDD Workflow**: Tests written → Approved → Fail → Implement
- [ ] **Integration Tests**: Crate boundaries covered
- [ ] **Contract Tests**: Public API behavior guaranteed

**This gate CANNOT be violated. No justification accepted.**

### IV. Modular Design ✅/❌

- [ ] **Single Responsibility**: Each new crate has clear, singular purpose
- [ ] **No Circular Dependencies**: Dependency graph is acyclic
- [ ] **CocoIndex Integration**: If using dataflow, follows declarative YAML patterns

**Justification if violated**: [Explain architectural necessity]

### V. Open Source Compliance ✅/❌

- [ ] **AGPL-3.0**: All new code properly licensed
- [ ] **REUSE Spec**: License headers or .license files present
- [ ] **Attribution**: Forked/vendored code properly attributed

**Justification if violated**: [Explain licensing requirements]

### VI. Service Architecture & Persistence ✅/❌/N/A

- [ ] **Deployment Target**: CLI, Edge, or Both (specify below)
- [ ] **Storage Backend**: Postgres (CLI), D1 (Edge), Qdrant (vectors), or N/A
- [ ] **Caching Strategy**: Content-addressed caching enabled if applicable
- [ ] **Concurrency Model**: Rayon (CLI) or tokio (Edge) - specify below

**Deployment Target**: [CLI | Edge | Both | N/A]
**Storage Backend**: [Postgres | D1 | Qdrant | None | NEEDS CLARIFICATION]
**Justification if N/A**: [Explain why service architecture doesn't apply]

### Quality Standards (Service-Specific) ✅/❌/N/A

- [ ] **Storage Benchmarks**: If using storage, performance targets defined
  - Postgres: <10ms p95 | D1: <50ms p95 | Qdrant: <100ms p95
- [ ] **Cache Performance**: If using caching, >90% hit rate targeted
- [ ] **Incremental Updates**: If service layer, incremental re-analysis implemented
- [ ] **Edge Deployment**: If WASM target, `mise run build-wasm-release` passes

**Justification if N/A**: [Explain why service quality gates don't apply]

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Delete unused options and expand the chosen structure with
  real paths (e.g., apps/admin, packages/something). The delivered plan must
  not include Option labels.
-->

```text
# [REMOVE IF UNUSED] Option 1: Single project (DEFAULT)
src/
├── models/
├── services/
├── cli/
└── lib/

tests/
├── contract/
├── integration/
└── unit/

# [REMOVE IF UNUSED] Option 2: Web application (when "frontend" + "backend" detected)
backend/
├── src/
│   ├── models/
│   ├── services/
│   └── api/
└── tests/

frontend/
├── src/
│   ├── components/
│   ├── pages/
│   └── services/
└── tests/

# [REMOVE IF UNUSED] Option 3: Mobile + API (when "iOS/Android" detected)
api/
└── [same as backend above]

ios/ or android/
└── [platform-specific structure: feature modules, UI flows, platform tests]
```

**Structure Decision**: [Document the selected structure and reference the real
directories captured above]

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
