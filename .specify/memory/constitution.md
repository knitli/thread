<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: AGPL-3.0-or-later
-->

<!--
================================================================================
SYNC IMPACT REPORT - Constitution Update
================================================================================

Version Change: 1.0.0 → 2.0.0 (MAJOR - Architectural Philosophy Shift)

Modified Principles:
  - Principle I: "Library-First Architecture" → "Service-Library Architecture"
    MAJOR redefinition to acknowledge dual architecture pattern
  - Principle IV: "Modular Design" → Extended with CocoIndex integration guidance

Added Sections:
  - Principle VI: Service Architecture & Persistence (NEW)
  - Deployment Architecture (NEW section after Contribution & Review Process)
  - Service & Storage Quality Gates (added to Quality Standards)

Removed Sections: None

Templates Requiring Updates:
  ⚠ .specify/templates/plan-template.md (needs service architecture context)
  ⚠ .specify/templates/spec-template.md (needs deployment success criteria)
  ⚠ .specify/templates/tasks-template.md (needs service testing phases)
  ✅ All templates architecturally compatible but should reference new principles

Follow-up TODOs:
  - Update plan-template.md Constitution Check gates to include:
    * Service architecture compliance
    * Deployment target specification (CLI vs Edge)
    * Storage backend selection justification
  - Update spec-template.md Success Criteria to include:
    * Cache performance metrics
    * Incremental update benchmarks
    * Edge deployment validation
  - Update tasks-template.md to include:
    * Service testing phases (storage, caching, incremental)
    * Edge deployment tasks
    * Database migration tasks

Rationale for v2.0.0:
  MAJOR version bump warranted due to fundamental architectural philosophy shift
  from "library-first" to "service-library" dual architecture. This changes how
  all features are conceived, designed, and implemented. Path B (CocoIndex
  integration) represents a strategic commitment to service-first capabilities
  (persistence, caching, orchestration) that redefines Thread's core identity.

  The addition of Principle VI (Service Architecture) and Deployment Architecture
  section establishes service capabilities as co-equal with library capabilities,
  not merely additive. This is a governance redefinition, not an expansion.

Amendment Authority:
  Based on architectural review document:
  - /home/knitli/thread/.phase0-planning/04-architectural-review-jan9/
    2026-01-10-FINAL_DECISION_PATH_B.md
  - PATH_B_IMPLEMENTATION_GUIDE.md
  Approved by: Thread Architecture Team
  Effective Date: January 10, 2026

================================================================================
-->

# Thread Constitution

## Core Principles

### I. Service-Library Architecture

Thread follows a **dual-architecture pattern** where both service and library capabilities are first-class citizens:

**Library Core**:
- Parsing, analysis, and transformation logic MUST be implemented as library crates (ast-engine, language, rule-engine, utils, services, wasm)
- Libraries MUST remain self-contained, independently testable, and reusable across contexts
- Each library crate MUST be documented with rustdoc and serve a clear technical purpose aligned with Thread's mission
- No organizational-only libraries permitted—every crate must solve a concrete technical problem

**Service Layer**:
- Orchestration, persistence, caching, and real-time intelligence MUST be implemented as service components
- Services provide long-lived, continuously-running capabilities with incremental updates
- Services leverage CocoIndex dataflow framework for content-addressed caching and dependency tracking
- All features MUST consider both library API design AND service deployment characteristics

**Rationale**: Thread is simultaneously a reusable library ecosystem (enabling embedding in other tools) AND a persistent service platform (enabling intelligent, cached analysis). Library purity enables composition and reuse; service architecture enables performance at scale and real-time intelligence. Neither aspect is subordinate—both define Thread's value proposition.

### II. Performance & Safety

Rust's memory safety guarantees MUST be preserved throughout the codebase. Use of `unsafe` MUST be explicitly justified with safety invariants documented. Performance-critical code paths MUST be benchmarked with regression prevention. SIMD optimizations MUST be applied to hot paths where appropriate. Memory efficiency is a first-class design constraint—allocations in critical paths require justification.

**Rationale**: Thread's value proposition is "safe, fast, flexible" analysis. Safety and performance are not negotiable—they define the project's competitive advantage over dynamic language alternatives.

### III. Test-First Development (NON-NEGOTIABLE)

TDD is mandatory: Tests written → User/stakeholder approval → Tests fail → Then implement. Red-Green-Refactor cycle MUST be strictly enforced. All tests MUST execute via `cargo nextest`. Integration tests MUST cover crate boundaries and public APIs. Contract tests MUST validate behavior guarantees for library consumers.

**Rationale**: Test-first development catches design flaws early, documents intended behavior, and prevents regression. For a parsing library where correctness is paramount, this discipline is non-negotiable.

### IV. Modular Design

Each crate MUST have a single, well-defined responsibility (ast-engine, language, rule-engine, utils, services, wasm). Dependencies MUST flow in one direction—no circular dependencies permitted. Workspace-level configuration MUST enforce consistency across crates. Feature flags MUST be used for optional functionality to minimize binary size and compilation overhead.

**CocoIndex Integration**:
- CocoIndex is approved as a **foundational framework dependency** for dataflow orchestration
- Thread operators MUST be implemented as native Rust traits (SourceFactory, SimpleFunctionFactory, TargetFactory)
- Dataflow pipelines MUST use declarative YAML specifications with version control
- Content-addressed caching MUST be enabled for all transformations unless explicitly justified
- CocoIndex integration MUST preserve library-service boundary: libraries remain independent, services leverage CocoIndex

**Rationale**: Clean module boundaries enable parallel development, selective compilation, and independent evolution of subsystems without cascading breakage. CocoIndex provides production-grade dataflow orchestration that would be cost-prohibitive to build from scratch.

### V. Open Source Compliance

All new code MUST be licensed under AGPL-3.0-or-later. REUSE specification compliance is mandatory—every file MUST include license headers or `.license` companion files. Forked or vendored code MUST maintain original attribution per `VENDORED.md` requirements. Software Bill of Materials (SBOM) at `sbom.spdx` MUST be maintained automatically via build tooling.

**Rationale**: Legal clarity protects contributors and users. AGPL ensures derivative works remain open source. REUSE compliance prevents license ambiguity at scale.

### VI. Service Architecture & Persistence

Thread operates as a **long-lived, persistent service** with incremental intelligence capabilities:

**Service Characteristics**:
- **Long-Lived Execution**: Thread services run continuously, maintaining state and caching across requests
- **Real-Time Updates**: Code changes trigger automatic incremental re-analysis, not full repository scans
- **Content-Addressed Caching**: All analysis results MUST use content-addressable storage for 50x+ performance gains on repeated analysis
- **Persistent Storage**: Native integration with Postgres (local), D1 (edge), and Qdrant (vectors) is mandatory for production deployments

**Dataflow Orchestration**:
- **CocoIndex Framework**: All ETL pipelines MUST use CocoIndex dataflow for dependency tracking and incremental processing
- **Automatic Dependencies**: System MUST automatically detect which components need re-analysis when source changes
- **Declarative Pipelines**: Analysis workflows MUST be specified as declarative dataflow graphs, not imperative code

**Concurrency Model**:
- **Local CLI**: Rayon for CPU-bound parallelism (multi-core utilization on single machine)
- **Cloud/Edge**: tokio async runtime for I/O-bound operations and horizontal scaling
- Both concurrency models MUST coexist; selection based on deployment context

**Rationale**: Service-first architecture distinguishes Thread from traditional one-shot analysis tools. Persistence and incremental caching enable instant retrieval on subsequent queries and intelligent re-analysis when code changes. This is Thread's core competitive advantage for real-time code intelligence.

## Quality Standards

All code MUST pass quality gates before merge:

- **Linting**: `mise run lint` (clippy + rustfmt) MUST pass with zero warnings
- **Testing**: `cargo nextest run --all-features` MUST pass with 100% success rate
- **CI Pipeline**: `mise run ci` MUST complete successfully
- **Documentation**: Public APIs MUST have rustdoc documentation; undocumented public items are build failures
- **Benchmarks**: Performance-sensitive changes MUST include benchmark comparisons; regressions require explicit justification

**Coverage Requirements**: Core crates (ast-engine, rule-engine) MUST maintain >80% line coverage. New features MUST include tests achieving equivalent coverage before merge.

**Service & Storage Quality Gates**:
- **Storage Benchmarks**: Database operations MUST meet performance targets:
  - Postgres: <10ms p95 latency for index queries
  - D1: <50ms p95 latency for edge queries (includes network overhead)
  - Qdrant: <100ms p95 latency for vector similarity search
- **Cache Performance**: Content-addressed cache MUST achieve >90% hit rate for repeated analysis of unchanged code
- **Incremental Updates**: Code changes MUST trigger only affected component re-analysis, not full repository re-scan
- **Edge Deployment**: WASM builds MUST complete successfully via `mise run build-wasm-release`
- **Schema Migrations**: Database schema changes MUST include rollback scripts and forward/backward compatibility testing
- **Dataflow Validation**: CocoIndex pipeline specifications MUST be validated against schema before deployment

## Contribution & Review Process

All code changes MUST follow this workflow:

1. **Branch Creation**: Feature branches named `feature/description` or `fix/description`
2. **Implementation**: Follow TDD cycle (test → fail → implement → pass → refactor)
3. **Quality Gate**: Run `mise run ci` locally; all checks MUST pass
4. **Pull Request**: Submit PR with clear description linking to issue/spec
5. **Code Review**: At least one approving review required; reviewers MUST verify constitution compliance
6. **Contributor License Agreement**: First-time contributors MUST sign CLA per `CONTRIBUTORS_LICENSE_AGREEMENT.md`
7. **Merge**: Squash or rebase merge (no merge commits); delete feature branch post-merge

**Breaking Changes**: API changes that break backward compatibility MUST:
- Be called out explicitly in PR description
- Include migration guide in CHANGELOG.md
- Trigger MAJOR version bump per semantic versioning

## Deployment Architecture

Thread supports **dual deployment targets** with different runtime characteristics:

### Local CLI Deployment

- **Runtime**: Rayon for CPU-bound parallelism
- **Storage**: Postgres for persistent caching and analysis results
- **Use Case**: Fast local analysis with incremental updates on developer machines
- **Concurrency**: Multi-core utilization on single machine
- **Distribution**: Single binary via cargo install or package managers

### Cloud/Edge Deployment (Cloudflare)

- **Runtime**: tokio async for horizontal scaling and I/O-bound operations
- **Storage**: D1 (edge SQL database) for distributed caching across CDN nodes
- **State Management**: Durable Objects for coordination and real-time updates
- **Compute**: Serverless containers (WASM) on Cloudflare edge network
- **Use Case**: Real-time code intelligence at global scale with <50ms latency
- **Distribution**: WASM deployment to Cloudflare Workers

### Deployment Standards

- **Single Binary**: Thread MUST compile to both native binary (CLI) and WASM (edge) from same codebase
- **Storage Abstraction**: Storage layer MUST support pluggable backends (Postgres, D1, Qdrant) via trait-based abstraction
- **Content-Addressed Caching**: Edge deployment MUST use content-addressed storage to minimize bandwidth and maximize cache hit rates
- **Database Migrations**: Schema changes MUST be versioned, tested, and include rollback procedures
- **Zero-Downtime Updates**: Service deployments MUST support rolling updates without analysis interruption

**Deployment Validation**:
- CLI builds MUST be tested on Linux, macOS, and Windows
- Edge builds MUST be validated against Cloudflare Workers runtime
- Storage backends MUST pass integration tests for both local and edge contexts

## Governance

This constitution supersedes all other development practices and guidelines. Amendments require:

1. **Documentation**: Proposed change documented with clear rationale
2. **Review**: Discussion and approval from maintainers
3. **Migration Plan**: If changes affect existing code/workflows, migration path MUST be specified
4. **Version Bump**: Constitution version MUST increment per semantic versioning:
   - **MAJOR**: Backward-incompatible changes (principle removal, redefinition)
   - **MINOR**: New principles or material expansions
   - **PATCH**: Clarifications, wording improvements, non-semantic refinements

**Compliance Enforcement**: All pull request reviews MUST verify adherence to constitutional principles. Complexity that violates principles (e.g., circular dependencies, unsafe code) MUST be explicitly justified with technical reasoning.

**Runtime Guidance**: Day-to-day development guidance lives in `CLAUDE.md` and `.specify/templates/`. These files MUST align with constitutional principles but may evolve without constitutional amendments.

**Version**: 2.0.0 | **Ratified**: 2026-01-10 | **Last Amended**: 2026-01-10
