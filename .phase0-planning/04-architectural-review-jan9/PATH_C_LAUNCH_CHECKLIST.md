# Path C Launch Checklist
## Getting Started This Week (January 9-13)

Use this checklist to prepare for Path C launch on Monday, January 13, 2026.

---

## Pre-Launch Approval (This Week)

### Stakeholder Alignment
- [ ] **Present Path C to decision makers** (30 min)
  - Why parallel tracks instead of deciding now?
  - Benefits: evidence-based decision, low risk, addresses core concern
  - Cost: 3 weeks investment, full team focus required
  - Outcome: Choose Path A or B with confidence by Jan 30

- [ ] **Secure commitment** from:
  - [ ] Product lead
  - [ ] Engineering lead
  - [ ] CTO/Architecture authority
  - [ ] Budget owner (if cost implications)

- [ ] **Confirm stakeholder expectations**:
  - Path C is NOT choosing both (A and B)
  - Path C IS running both to choose one
  - By Jan 30, MUST decide on A or B
  - No ambiguity on selected path after decision

### Risk Acceptance
- [ ] Stakeholders accept that one path's work may not ship
  (Track A or B code may be archived, not merged)
- [ ] Stakeholders accept 3-week timeline commitment
  (Both teams fully focused, no context switching)
- [ ] Stakeholders accept decision finality
  (Once chosen on Jan 30, commit to implementation, no second-guessing)

---

## Team Structure (This Week)

### Assign Track Owners

**Track A Owner** (Services Implementation)
- [ ] Name: _________________
- [ ] Confirmed availability: [ ] Full-time for 3 weeks
- [ ] Rust expertise level: [ ] Intermediate+ required
- [ ] Responsibilities:
  - Daily standups
  - AstGrepParser/Analyzer implementation
  - Testing and validation
  - Performance measurement

**Track B Owner** (CocoIndex Prototype)
- [ ] Name: _________________
- [ ] Confirmed availability: [ ] Full-time for 3 weeks
- [ ] Background:
  - [ ] Dataflow programming experience (helpful but not required)
  - [ ] Comfortable learning new frameworks
  - [ ] Infrastructure/systems thinking
- [ ] Responsibilities:
  - CocoIndex research and integration
  - Custom transform development
  - Benchmarking and performance analysis

**Architecture Lead** (Decision Maker)
- [ ] Name: _________________
- [ ] Responsibilities:
  - Unblock teams daily
  - Resolve architectural questions
  - Moderate discussions
  - Make final decision on Jan 30

**Optional: Integration Manager**
- [ ] Name: _________________
- [ ] Responsibilities:
  - Coordinate weekly syncs
  - Track progress against milestones
  - Manage scope/schedule risks

### Team Logistics
- [ ] Communication channels established
  - [ ] Track A Slack channel
  - [ ] Track B Slack channel
  - [ ] All-hands Slack channel
- [ ] Meeting times scheduled
  - [ ] Track A daily standup: Time: ________
  - [ ] Track B daily standup: Time: ________
  - [ ] Weekly all-hands: Friday 10am (30 min)
  - [ ] Mid-week sync: Wednesday 2pm (30 min, optional but recommended)
- [ ] Tools access confirmed
  - [ ] GitHub access for both teams
  - [ ] CI/CD infrastructure
  - [ ] Performance benchmarking tools

---

## Infrastructure & Environment (By Monday)

### Track A Requirements
- [ ] Development machine with Rust toolchain
- [ ] Latest stable Rust installed
  ```bash
  rustup update stable
  ```
- [ ] Project dependencies installed
  ```bash
  cargo fetch
  cd crates/services
  cargo check  # Should show compilation errors to fix
  ```
- [ ] VS Code or IDE with Rust analyzer configured
- [ ] Git repository cloned and set up
- [ ] Cargo-nextest installed for test running
  ```bash
  cargo install cargo-nextest
  ```

### Track B Requirements
- [ ] CocoIndex repository cloned locally
  ```bash
  git clone https://github.com/cocoindex-io/cocoindex.git
  ```
- [ ] PostgreSQL installed (or Docker container ready)
  ```bash
  docker run -d --name cocoindex-db \
    -e POSTGRES_PASSWORD=password \
    -p 5432:5432 postgres:15
  ```
- [ ] Qdrant installed (optional, for vector storage)
  ```bash
  docker run -d --name qdrant \
    -p 6333:6333 \
    qdrant/qdrant:latest
  ```
- [ ] CocoIndex examples running successfully
  ```bash
  cd cocoindex
  cargo run --example simple_pipeline
  ```
- [ ] Development workspace set up
- [ ] Thread repository cloned with all dependencies ready

### Shared Infrastructure
- [ ] Performance benchmarking framework
  - [ ] Criterion.rs configured (for Track A)
  - [ ] Custom benchmark harness (for Track B)
- [ ] Metrics collection spreadsheet created
  - [ ] Shared Google Sheet or Airtable
  - [ ] Both teams can enter daily metrics
- [ ] Progress tracking tool
  - [ ] GitHub Issues with daily milestones, OR
  - [ ] Jira board with daily tasks, OR
  - [ ] Spreadsheet with checklist
- [ ] Decision framework scorecard template prepared
  - [ ] Both pathways have same evaluation criteria
  - [ ] Scoring methodology documented

---

## Documentation Preparation (This Week)

### Create Working Documents
- [ ] Path C epic/issue created in tracking system
- [ ] Daily milestone issues created
  - [ ] Track A Days 1-14 (14 separate milestones)
  - [ ] Track B Days 1-14 (14 separate milestones)
- [ ] Week 3 evaluation template prepared
  - [ ] Metrics collection sheet
  - [ ] Decision scorecard
  - [ ] Comparison matrix

### Share Key Documents
- [ ] PATH_C_DETAILED_IMPLEMENTATION_PLAN.md shared with team
- [ ] PATH_C_QUICK_START.md shared with stakeholders
- [ ] Weekly schedule and standup times confirmed in calendar
- [ ] GitHub issues linked to shared planning docs

### Create Team Wiki/Confluence Page
- [ ] Overview: What is Path C?
- [ ] Track A: Daily breakdown, deliverables
- [ ] Track B: Daily breakdown, deliverables
- [ ] Week 3: Decision process, dates
- [ ] Resources: Links to CocoIndex docs, ast-grep docs
- [ ] Contact info: Who to ask for what

---

## Kick-Off Preparation

### Kick-Off Meeting Agenda (30 min, Monday morning)

**Time**: ________  
**Attendees**: Track A owner, Track B owner, Arch Lead, stakeholders

```
1. Welcome & Overview (5 min)
   - Path C approach explained
   - Why this matters (evidence-based decision)
   - Timeline and deliverables

2. Track A Deep-Dive (8 min)
   - Track A owner explains scope and daily plan
   - Key deliverables
   - Success criteria
   - Questions?

3. Track B Deep-Dive (8 min)
   - Track B owner explains scope and daily plan
   - Key deliverables
   - Success criteria
   - Questions?

4. Logistics & Expectations (7 min)
   - Communication channels
   - Daily standups and weekly syncs
   - Metric collection process
   - Blocker escalation
   - Timeline commitment

5. Questions & Concerns (2 min)
```

**Preparation**:
- [ ] Track A owner prepares 2-3 slide overview
- [ ] Track B owner prepares 2-3 slide overview
- [ ] Arch Lead prepares 1 slide on decision process
- [ ] Share deck with attendees 24 hours before

---

## Pre-Launch Code Verification (By Friday)

### Track A: Verify Build State
- [ ] Clone fresh copy of Thread
- [ ] Navigate to services crate
  ```bash
  cd crates/services
  ```
- [ ] Check compilation errors
  ```bash
  cargo check 2>&1 | tee compilation_errors.txt
  ```
- [ ] Count errors: __________ (should be ~36 from prior analysis)
- [ ] Document error types in a spreadsheet
- [ ] Track A owner reviews errors, plans fixes

**Expected Output**: Documented list of compilation errors with fix strategy

### Track B: Verify Dependencies
- [ ] CocoIndex cloned and builds
  ```bash
  cd cocoindex
  cargo build
  ```
- [ ] PostgreSQL running
  ```bash
  psql -U postgres -h localhost -c "SELECT 1;"
  ```
- [ ] CocoIndex example runs
  ```bash
  cargo run --example <example_name>
  ```
- [ ] Thread dependencies available
- [ ] Documentation reviewed

**Expected Output**: All systems operational, no blocking issues

---

## Communication Setup (This Week)

### Create Channels (if using Slack or similar)
- [ ] #thread-path-c: General updates
- [ ] #path-c-track-a: Services implementation discussions
- [ ] #path-c-track-b: CocoIndex discussions
- [ ] #path-c-decision: Decision framework discussions

### Invite People to Channels
- [ ] Track A team → track A channel
- [ ] Track B team → track B channel
- [ ] Architecture Lead → all channels
- [ ] Stakeholders → general channel (optional)

### Create Calendar Events
- [ ] Daily Track A standup: Jan 13-31, 9:30am, 15 min
- [ ] Daily Track B standup: Jan 13-31, 9:45am, 15 min
  - *Staggered so not competing for attention*
- [ ] Weekly all-hands: Jan 17, 24, 31 (Friday, 10am, 30 min)
- [ ] Mid-week sync: Jan 15, 22, 29 (Wednesday, 2pm, 30 min optional)
- [ ] Decision meeting: Jan 30 (Thursday, 2pm, 1 hour)

### Set Up Daily Sync Template
```
TRACK A Daily Standup
Time: 9:30 AM
Format: Async or 15-min sync

What I completed yesterday:
What I'm working on today:
Blockers:
Confidence level: [1-5]
Tomorrow's plan:
```

---

## Metrics Collection Setup (This Week)

### Create Spreadsheet for Daily Tracking

**Track A Metrics**
| Date | Errors Fixed | Tests Written | Coverage % | Build Time | Performance | Confidence | Blockers |
|------|--------------|----------------|-----------|------------|------------|-----------|----------|
| 1/13 | | | | | | | |
| 1/14 | | | | | | | |
| ... | | | | | | | |

**Track B Metrics**
| Date | Setup Done | Transforms Built | Type Bridge | Benchmarks | Speedup | Confidence | Blockers |
|------|-----------|-------------------|------------|-----------|---------|-----------|----------|
| 1/13 | | | | | | | |
| 1/14 | | | | | | | |
| ... | | | | | | | |

### Set Up Performance Tracking
- [ ] Benchmark baseline established for Thread (pure parsing, no services)
- [ ] Benchmark harness created for Track A (services overhead)
- [ ] Benchmark harness created for Track B (CocoIndex overhead)
- [ ] Metrics recorded daily (or every other day minimum)

---

## Success Criteria Review (This Week)

### Review & Confirm Track A Criteria
With Track A owner, confirm these are the measure of success:
- [ ] **Compilation**: Errors reduced from 36+ to 0
- [ ] **Testing**: 95%+ test pass rate achieved
- [ ] **Coverage**: 80%+ code coverage
- [ ] **Performance**: <5% overhead vs pure ast-grep
- [ ] **Completeness**: All trait methods implemented

### Review & Confirm Track B Criteria
With Track B owner, confirm these are the measure of success:
- [ ] **Environment**: CocoIndex integrated, running
- [ ] **Transforms**: ThreadParse and ExtractSymbols working
- [ ] **Type System**: Metadata preserves through pipeline
- [ ] **Benchmarks**: Performance metrics collected
- [ ] **Extraction**: Clear path to swap CocoIndex identified

### Confirm Decision Criteria
With Arch Lead & stakeholders:
- [ ] All 5 dimensions understood (Performance, Type Safety, Complexity, Extensibility, Risk)
- [ ] Weighting confirmed (40%, 20%, 15%, 15%, 10%)
- [ ] Scoring methodology clear
- [ ] Decision date firm (Jan 30)

---

## Red Flags (Watch For These)

**If you see any of these before Monday, escalate immediately:**

- [ ] **Team not committed**: "Not sure we can do full-time for 3 weeks"
  - **Action**: Reduce scope, not timeline. Path C requires focus.

- [ ] **Infrastructure issues**: CocoIndex won't build, PostgreSQL issues, etc.
  - **Action**: Fix by end of Friday, or ask Track B to use simpler setup

- [ ] **Unclear success criteria**: "What if we get 3 of 5 done?"
  - **Action**: Clarify before starting. Must-pass gates are non-negotiable.

- [ ] **Scope creep already**: "While we're at it, let's also..."
  - **Action**: Track these as Phase 1 work, NOT Path C scope

- [ ] **Decision maker unavailable Jan 30**: "I'll be out that week"
  - **Action**: Reschedule decision meeting immediately

- [ ] **Stakeholder ambiguity**: "So we're doing both paths?"
  - **Action**: Clarify: "We're running both to choose ONE by Jan 30"

---

## Day-1 Execution (Monday, January 13)

### Morning (Before Daily Standups)
- [ ] All attendees joined communication channels
- [ ] All attendees have access to shared documents
- [ ] Kick-off meeting completed
- [ ] Track owners understand their daily breakdown
- [ ] Arch Lead ready to unblock

### Track A (9:30am Standup)
- [ ] Review compilation errors
- [ ] Plan first day of fixes
- [ ] Commit to Day 1 deliverable: errors analyzed, fix plan documented
- [ ] Set up metrics tracking

### Track B (9:45am Standup)
- [ ] Verify environment fully set up
- [ ] Review CocoIndex examples
- [ ] Plan first day of learning
- [ ] Commit to Day 1 deliverable: design document drafted
- [ ] Set up metrics tracking

### All (End of Day)
- [ ] Both teams report Day 1 progress
- [ ] Any blockers escalated to Arch Lead
- [ ] Metrics entered in spreadsheet
- [ ] Confidence levels recorded

---

## Weekly Checklist (Every Friday)

### Friday Sync (All Teams)
- [ ] Report Week X progress
- [ ] Compare to planned milestones
- [ ] Discuss any scope/timeline adjustments
- [ ] Celebrate wins
- [ ] Plan Week X+1

### Metrics Review
- [ ] Collect all daily metrics
- [ ] Plot performance trends
- [ ] Identify any concerning patterns
- [ ] Adjust if needed

### Blockers
- [ ] Any unresolved blockers?
- [ ] Any risks emerging?
- [ ] Mitigation needed?

---

## Week 3 Execution (January 27-30)

### Monday, January 27
- [ ] Both tracks finalize work
- [ ] Metrics fully collected
- [ ] Both tracks prepare findings doc

### Tuesday, January 28
- [ ] Score both paths on decision framework
- [ ] Arch Lead prepares decision scorecard
- [ ] Both track owners review scorecard for accuracy
- [ ] Prepare presentation materials

### Wednesday, January 29
- [ ] Team meeting to discuss findings (2 hours)
- [ ] Debate pros/cons
- [ ] Build consensus
- [ ] Identify any last questions

### Thursday, January 30
- [ ] Final decision meeting with stakeholders (1 hour)
- [ ] Present findings and recommendation
- [ ] Announce chosen path (A or B)
- [ ] Define Phase 0 completion plan

---

## Sign-Off

**Print this list and get signatures before starting:**

| Role | Name | Signature | Date |
|------|------|-----------|------|
| Architecture Lead | | | |
| Track A Owner | | | |
| Track B Owner | | | |
| Product Lead | | | |
| Engineering Lead | | | |

---

## Done!

You're ready to launch Path C on **Monday, January 13, 2026**.

**Remember**: This 3-week investment prevents 12 months of architectural pain.

**Go build something great. 🚀**
