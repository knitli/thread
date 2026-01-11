# Feature Specification: [FEATURE NAME]

**Feature Branch**: `[###-feature-name]`  
**Created**: [DATE]  
**Status**: Draft  
**Input**: User description: "$ARGUMENTS"

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.
  
  Assign priorities (P1, P2, P3, etc.) to each story, where P1 is the most critical.
  Think of each story as a standalone slice of functionality that can be:
  - Developed independently
  - Tested independently
  - Deployed independently
  - Demonstrated to users independently
-->

### User Story 1 - [Brief Title] (Priority: P1)

[Describe this user journey in plain language]

**Why this priority**: [Explain the value and why it has this priority level]

**Independent Test**: [Describe how this can be tested independently - e.g., "Can be fully tested by [specific action] and delivers [specific value]"]

**Acceptance Scenarios**:

1. **Given** [initial state], **When** [action], **Then** [expected outcome]
2. **Given** [initial state], **When** [action], **Then** [expected outcome]

---

### User Story 2 - [Brief Title] (Priority: P2)

[Describe this user journey in plain language]

**Why this priority**: [Explain the value and why it has this priority level]

**Independent Test**: [Describe how this can be tested independently]

**Acceptance Scenarios**:

1. **Given** [initial state], **When** [action], **Then** [expected outcome]

---

### User Story 3 - [Brief Title] (Priority: P3)

[Describe this user journey in plain language]

**Why this priority**: [Explain the value and why it has this priority level]

**Independent Test**: [Describe how this can be tested independently]

**Acceptance Scenarios**:

1. **Given** [initial state], **When** [action], **Then** [expected outcome]

---

[Add more user stories as needed, each with an assigned priority]

### Edge Cases

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right edge cases.
-->

- What happens when [boundary condition]?
- How does system handle [error scenario]?

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right functional requirements.
-->

### Functional Requirements

- **FR-001**: System MUST [specific capability, e.g., "allow users to create accounts"]
- **FR-002**: System MUST [specific capability, e.g., "validate email addresses"]  
- **FR-003**: Users MUST be able to [key interaction, e.g., "reset their password"]
- **FR-004**: System MUST [data requirement, e.g., "persist user preferences"]
- **FR-005**: System MUST [behavior, e.g., "log all security events"]

*Example of marking unclear requirements:*

- **FR-006**: System MUST authenticate users via [NEEDS CLARIFICATION: auth method not specified - email/password, SSO, OAuth?]
- **FR-007**: System MUST retain user data for [NEEDS CLARIFICATION: retention period not specified]

### Key Entities *(include if feature involves data)*

- **[Entity 1]**: [What it represents, key attributes without implementation]
- **[Entity 2]**: [What it represents, relationships to other entities]

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria.
  These must be technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: [Measurable metric, e.g., "Users can complete account creation in under 2 minutes"]
- **SC-002**: [Measurable metric, e.g., "System handles 1000 concurrent users without degradation"]
- **SC-003**: [User satisfaction metric, e.g., "90% of users successfully complete primary task on first attempt"]
- **SC-004**: [Business metric, e.g., "Reduce support tickets related to [X] by 50%"]

### Service Architecture Success Criteria *(include if feature has service layer)*

<!--
  ACTION REQUIRED: If feature includes service layer (persistence, caching, orchestration),
  define service-specific success criteria. Mark as N/A if purely library feature.
-->

**Deployment Targets**: [CLI | Edge | Both | N/A]

#### Cache Performance *(if applicable)*

- **SC-CACHE-001**: Content-addressed cache achieves >90% hit rate for repeated analysis
- **SC-CACHE-002**: Cache invalidation occurs within [X]ms of source code change
- **SC-CACHE-003**: [Additional cache metric, e.g., "Cache size remains under 500MB for 10k file repository"]

#### Incremental Updates *(if applicable)*

- **SC-INCR-001**: Code changes trigger only affected component re-analysis (not full scan)
- **SC-INCR-002**: Incremental update completes in <[X]% of full analysis time
- **SC-INCR-003**: [Additional incremental metric, e.g., "Dependency graph updates in <100ms"]

#### Storage Performance *(if applicable)*

- **SC-STORE-001**: Database operations meet constitutional targets:
  - Postgres (CLI): <10ms p95 latency for index queries
  - D1 (Edge): <50ms p95 latency for edge queries
  - Qdrant (vectors): <100ms p95 latency for similarity search
- **SC-STORE-002**: [Additional storage metric, e.g., "Schema migrations complete in <30 seconds"]

#### Edge Deployment *(if WASM target applicable)*

- **SC-EDGE-001**: WASM binary compiles successfully via `mise run build-wasm-release`
- **SC-EDGE-002**: Edge deployment serves requests with <50ms p95 latency globally
- **SC-EDGE-003**: [Additional edge metric, e.g., "WASM bundle size under 5MB compressed"]

**Mark as N/A if not applicable**: [Explain why service architecture doesn't apply to this feature]
