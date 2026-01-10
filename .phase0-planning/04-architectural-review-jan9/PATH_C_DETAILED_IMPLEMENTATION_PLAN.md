# PATH C: Parallel Development Plan
## Hybrid Prototyping Approach - Detailed Specification
**Date:** January 9, 2026  
**Duration:** 3 Weeks (January 13 - January 31, 2026)  
**Decision Date:** January 30, 2026  
**Status:** Ready for Execution  

---

## Executive Summary

This document provides a **detailed, actionable specification** for Path C - the recommended hybrid prototyping approach. Instead of deciding now between services-only (Path A) or services + CocoIndex (Path B), this plan runs **both implementations in parallel over 3 weeks**, validates them with real code and metrics, then makes an evidence-based architectural decision.

### Key Metrics
- **Total Timeline:** 3 weeks (January 13 - 31, 2026)
- **Parallel Tracks:** 2 independent implementation streams
- **Team Requirement:** 2 engineers minimum (1 per track ideally)
- **Decision Date:** January 30, 2026
- **Go/No-Go Decision Point:** Strong commitment after this date

### Why This Approach Works

✅ **Low Risk**: If either path fails, we only spent 3 weeks, not 6-8  
✅ **Evidence-Based**: Real code and benchmarks, not speculation  
✅ **Learning**: Team understands both architectures deeply  
✅ **Flexibility**: Can combine best of both approaches if needed  
✅ **Addresses Core Concern**: "I don't want to be fighting my own architecture in a year"  

---

## Part 1: Organizational Structure

### 1.1 Team Assignments

**Leadership**
- **Architecture Lead**: Makes final decision, resolves blockers
- **Track A Owner**: Leads services implementation
- **Track B Owner**: Leads CocoIndex prototype
- **Integration Manager**: Coordinates weekly syncs, tracks progress

**Optional**
- Technical writer: Documents both approaches as you go
- QA: Designs benchmark methodology, validates metrics

### 1.2 Parallel Track Model

```
Week 1-2: PARALLEL DEVELOPMENT
│
├─ Track A: Minimal Services       (Team A, 1-2 engineers)
│  ├─ Fix compilation errors
│  ├─ Implement AstGrepParser
│  ├─ Implement AstGrepAnalyzer
│  ├─ Create mocks
│  ├─ Contract tests
│  └─ Prove abstraction works
│
└─ Track B: CocoIndex Prototype    (Team B, 1-2 engineers)
   ├─ Set up CocoIndex environment
   ├─ Build ThreadParse transform
   ├─ Build ExtractSymbols transform
   ├─ Wire end-to-end pipeline
   ├─ Performance benchmarks
   └─ Prove integration works
│
Week 3: EVALUATION & DECISION
│
├─ Compare both implementations
├─ Apply decision framework
├─ Make Path A vs B choice
└─ Plan next phase
```

### 1.3 Communication Structure

**Daily**
- Track A standup (15 min, start of day)
- Track B standup (15 min, start of day)

**Weekly (Friday)**
- All-hands sync (30 min)
- Compare progress against milestones
- Resolve blockers
- Adjust scope if needed

**Mid-Week (Wednesday)**
- Architecture Lead + both track owners (30 min)
- Discuss integration points, dependency issues
- Share learnings

---

## Part 2: Track A - Minimal Services Implementation

### 2.1 Track A Overview

**Goal**: Complete minimal Phase 0 implementation to prove services abstraction works

**Scope**: Fix compilation + implement core services only
- AstGrepParser (parse files, extract metadata)
- AstGrepAnalyzer (pattern matching)
- Mock implementations for testing
- Contract tests validating trait contracts

**Timeline**: 1.5-2 weeks
**Success Criteria**: 
- ✅ Workspace compiles cleanly
- ✅ All service implementations working
- ✅ Contract tests passing
- ✅ Basic integration test suite passing
- ✅ Performance < 5% overhead

### 2.2 Week 1-2 Daily Breakdown

#### Days 1-2: Foundation & Compilation Fix

**Day 1 (Monday)**
- [ ] Review services crate structure
- [ ] Identify all compilation errors (36+ from prior analysis)
- [ ] Create fix plan for type system issues
  - [ ] Add PhantomData markers to ParsedDocument<D>
  - [ ] Fix CodeMatch<'tree, D> lifetime/type issues
  - [ ] Fix stub types when ast-grep-backend disabled
  - [ ] OR make ast-grep-backend required feature

**Expected Output**: Compilation errors analyzed, fix strategy documented

**Day 2 (Tuesday)**
- [ ] Implement fixes identified in Day 1
- [ ] Run `cargo check` after each fix
- [ ] Validate `cargo check --workspace` succeeds
- [ ] Document feature flag combinations that work
- [ ] Update Cargo.toml with proper configuration

**Expected Output**: Workspace compiles without errors

#### Days 3-5: AstGrepParser Implementation

**Day 3 (Wednesday)**
- [ ] Create `crates/services/src/implementations/mod.rs`
- [ ] Create `crates/services/src/implementations/ast_grep.rs`
- [ ] Define AstGrepParser struct
  ```rust
  pub struct AstGrepParser {
      language_support: Arc<LanguageSupport>,
      cache: Arc<ContentCache>,
  }
  ```
- [ ] Outline trait implementation (use unimplemented! placeholders)

**Expected Output**: Skeleton implementation that compiles

**Day 4 (Thursday)**
- [ ] Implement parse_content method
  - [ ] Hook into thread-ast-engine Language parsing
  - [ ] Convert ast-grep Root to ParsedDocument
  - [ ] Extract basic metadata (file size, language)
  - [ ] Test with simple Rust file
- [ ] Implement get_language method
- [ ] Add simple tests

**Expected Output**: Can parse single file successfully

**Day 5 (Friday)**
- [ ] Implement metadata extraction
  - [ ] Symbol extraction (functions, structs, traits)
  - [ ] Import/export extraction
  - [ ] Basic call graph
  - [ ] Type information
- [ ] Add metadata tests
- [ ] Weekly sync: review progress with full team

**Expected Output**: Parser fully functional with metadata

#### Days 6-10: AstGrepAnalyzer + Mocks

**Day 6 (Monday)**
- [ ] Create `crates/services/src/implementations/analyzer.rs`
- [ ] Implement AstGrepAnalyzer struct
- [ ] Implement pattern_matches method
  - [ ] Hook into ast-grep pattern matching
  - [ ] Convert NodeMatch to CodeMatch<'tree, impl Doc>
  - [ ] Return list of matches with context
- [ ] Add basic pattern tests

**Expected Output**: Can find patterns in code

**Day 7 (Tuesday)**
- [ ] Implement analyze method
  - [ ] Multi-pattern analysis
  - [ ] Metrics calculation
  - [ ] Batch operations
- [ ] Implement all remaining CodeAnalyzer methods
- [ ] Test with real code samples

**Expected Output**: Full analysis capability

**Day 8 (Wednesday)**
- [ ] Create `crates/services/src/testing/mod.rs`
- [ ] Create MockParser
  - [ ] Deterministic test implementation
  - [ ] Return fixed test data
  - [ ] Validate contract compliance
- [ ] Create MockAnalyzer
  - [ ] Simulate pattern matching
  - [ ] Return predictable results
  - [ ] Test isolation

**Expected Output**: Mocks ready for testing

**Day 9 (Thursday)**
- [ ] Create contract tests
  ```
  tests/contract_tests.rs
  ├─ test_parser_contract_all_implementations
  │  ├─ Test with AstGrepParser
  │  ├─ Test with MockParser
  │  └─ Verify both satisfy contract
  └─ test_analyzer_contract_all_implementations
     ├─ Test with AstGrepAnalyzer
     ├─ Test with MockAnalyzer
     └─ Verify both satisfy contract
  ```
- [ ] Implement contract test suite
- [ ] All implementations pass contract

**Expected Output**: Contract tests passing

**Day 10 (Friday)**
- [ ] Integration tests
  ```
  tests/integration_tests.rs
  ├─ test_parse_simple_rust_file
  ├─ test_parse_with_metadata
  ├─ test_pattern_matching_basic
  ├─ test_pattern_matching_complex
  ├─ test_cross_file_detection (if time)
  └─ test_error_handling
  ```
- [ ] `cargo test --workspace` passes cleanly
- [ ] Weekly sync: full status review, begin evaluation prep

**Expected Output**: Full integration test suite passing

#### Days 11-14: Testing & Performance (Week 3, Part A)

**Day 11 (Monday)**
- [ ] Expand test suite based on issues found
- [ ] Add tests for error cases
- [ ] Improve test coverage to 80%+
- [ ] Fix any failing tests

**Expected Output**: Comprehensive test suite

**Day 12 (Tuesday)**
- [ ] Create performance benchmarks
  ```
  benches/service_benchmarks.rs
  ├─ bench_parse_100_lines
  ├─ bench_parse_1000_lines
  ├─ bench_parse_10000_lines
  ├─ bench_metadata_extraction
  ├─ bench_pattern_match_simple
  ├─ bench_pattern_match_complex
  └─ bench_batch_operations
  ```
- [ ] Establish baseline metrics
- [ ] Measure overhead vs pure ast-grep

**Expected Output**: Performance benchmarks running

**Days 13-14 (Wed-Thu)**
- [ ] Refactor for performance if needed
- [ ] Optimize hot paths
- [ ] Validate <5% overhead target
- [ ] Document performance characteristics

**Expected Output**: Performance validated, documented

### 2.3 Track A Deliverables

**Code**
```
crates/services/
├── src/
│   ├── implementations/
│   │   ├── mod.rs              NEW
│   │   ├── ast_grep.rs         NEW (AstGrepParser, AstGrepAnalyzer)
│   │   └── composite.rs        NEW (if time permits)
│   └── testing/
│       ├── mod.rs              NEW
│       ├── mock_parser.rs      NEW
│       └── mock_analyzer.rs    NEW
├── tests/
│   ├── contract_tests.rs       NEW
│   ├── integration_tests.rs    NEW
│   └── performance_tests.rs    NEW (optional)
└── benches/
    └── service_benchmarks.rs   NEW
```

**Documentation**
- Implementation guide (how services work)
- Migration guide (from direct ast-grep to services)
- Performance characteristics (benchmarks)
- Test coverage report

**Metrics Output**
- Compilation status: ✅ or ❌
- Test pass rate: X% (target 95%+)
- Performance overhead: X% (target <5%)
- Code coverage: X% (target 80%+)

### 2.4 Track A Success Criteria

| Criterion | Target | Priority | Go/No-Go |
|-----------|--------|----------|----------|
| Workspace builds | 100% | Critical | YES |
| Tests pass | 95%+ | High | YES |
| Code coverage | 80%+ | High | YES |
| Performance overhead | <5% | High | YES |
| Trait implementations complete | 100% | Critical | YES |
| Contract tests passing | 100% | High | YES |
| Documentation present | 100% | Medium | NO |

**Go Decision**: If 5 critical/high criteria met, Path A is viable

---

## Part 3: Track B - CocoIndex Integration Prototype

### 3.1 Track B Overview

**Goal**: Validate CocoIndex integration feasibility and performance with real code

**Scope**: Minimal viable integration
- Set up CocoIndex environment
- Build ThreadParse custom transform (parse code → ParsedDocument)
- Build ExtractSymbols transform (extract metadata)
- Wire: FileSource → ThreadParse → ExtractSymbols → Qdrant
- Performance benchmark vs pure Thread

**Timeline**: 1-2 weeks
**Success Criteria**:
- ✅ CocoIndex environment working
- ✅ Custom transforms functioning
- ✅ Type system bridge working (no metadata loss)
- ✅ Performance benchmarks complete
- ✅ Incremental update efficiency validated

### 3.2 Week 1-2 Daily Breakdown

#### Days 1-2: Environment Setup

**Day 1 (Monday)**
- [ ] Clone CocoIndex repository
- [ ] Review CocoIndex documentation
  - [ ] Dataflow programming model
  - [ ] Custom transform development
  - [ ] Storage backend integration
  - [ ] Performance optimization
- [ ] Set up development environment
  - [ ] Rust toolchain
  - [ ] PostgreSQL (CocoIndex state store)
  - [ ] Qdrant vector database (optional, for storage validation)
- [ ] Build CocoIndex examples
- [ ] Understand architecture deeply

**Expected Output**: Environment ready, CocoIndex examples running

**Day 2 (Tuesday)**
- [ ] Design ThreadParse transform
  ```rust
  pub struct ThreadParseTransform {
      parser: Arc<AstGrepParser>,
      chunker: Arc<CodeChunker>,
  }
  
  impl Transform for ThreadParseTransform {
      async fn execute(&self, input: Row) -> Result<Row> {
          // Parse code, extract metadata
          // Return enriched row
      }
  }
  ```
- [ ] Design ExtractSymbols transform
  - [ ] Input: ParsedDocument with metadata
  - [ ] Output: Structured symbol records
  - [ ] Relationships as edges
- [ ] Design data flow pipeline
  - [ ] File source (watch directory)
  - [ ] ThreadParse transform
  - [ ] ExtractSymbols transform
  - [ ] Storage sink (Postgres + optional Qdrant)
- [ ] Create detailed design document

**Expected Output**: Transform and pipeline design documented

#### Days 3-5: ThreadParse Transform

**Day 3 (Wednesday)**
- [ ] Implement ThreadParseTransform skeleton
- [ ] Hook into Thread's parsing
- [ ] Implement type conversion
  - [ ] ParsedDocument → CocoIndex Row
  - [ ] Validate round-trip (no data loss)
- [ ] Test with single file
- [ ] Measure overhead

**Expected Output**: Single file parsing works

**Day 4 (Thursday)**
- [ ] Batch file processing
- [ ] Metadata extraction through transform
- [ ] Test with multi-file codebase
- [ ] Validate metadata preservation
- [ ] Add error handling

**Expected Output**: Multi-file processing works

**Day 5 (Friday)**
- [ ] Optimize for performance
- [ ] Cache management
- [ ] Content hashing validation
- [ ] Weekly sync: progress review with team

**Expected Output**: ThreadParse transform production-ready

#### Days 6-10: ExtractSymbols + Pipeline Wiring

**Day 6 (Monday)**
- [ ] Implement ExtractSymbols transform
- [ ] Convert ParsedDocument symbols to structured records
- [ ] Build relationship graph from imports/exports
- [ ] Handle language-specific patterns
- [ ] Test extraction accuracy

**Expected Output**: Symbol extraction working

**Day 7 (Tuesday)**
- [ ] Integrate with file system watcher
  - [ ] Detect file changes
  - [ ] Trigger re-parsing incrementally
  - [ ] Update affected dependencies
- [ ] Test incremental updates
- [ ] Measure change propagation time

**Expected Output**: Change detection working

**Day 8 (Wednesday)**
- [ ] Wire to storage backend
  - [ ] Postgres for metadata
  - [ ] Optional Qdrant for vector embeddings (if time)
  - [ ] Test round-trip
- [ ] Validate data persistence
- [ ] Query extracted data from store

**Expected Output**: End-to-end pipeline working

**Day 9 (Thursday)**
- [ ] Build benchmark harness
  ```
  Initial index (1000-file codebase):
    - Time: X seconds
    - Memory: X MB
    - CPU: X%
  
  Incremental (10 files changed):
    - Time: X seconds (target 50x+ faster)
    - Memory: X MB
    - CPU: X%
  
  Compare to pure Thread baseline
  ```
- [ ] Run benchmarks
- [ ] Collect metrics

**Expected Output**: Initial benchmark results

**Day 10 (Friday)**
- [ ] Optimize hot paths based on benchmark results
- [ ] Profile CPU/memory usage
- [ ] Implement caching if beneficial
- [ ] Validate claimed CocoIndex efficiency (99%+ cost reduction claim)
- [ ] Weekly sync: present findings to team

**Expected Output**: Performance-optimized, metrics collected

#### Days 11-14: Validation & Documentation (Week 3, Part B)

**Day 11 (Monday)**
- [ ] Type system bridge validation
  - [ ] Round-trip metadata (ParsedDocument → Row → ParsedDocument)
  - [ ] Validate no information loss
  - [ ] Check type safety at boundaries
- [ ] Test with complex codebases
- [ ] Identify edge cases

**Expected Output**: Type system validated

**Day 12 (Tuesday)**
- [ ] Dependency extraction validation
  - [ ] Can swap CocoIndex for alternative dataflow engine?
  - [ ] How much is tied to CocoIndex specifics?
  - [ ] Design abstraction boundary
- [ ] Document extraction path
- [ ] Identify coupling points

**Expected Output**: Extraction path documented, feasible

**Days 13-14 (Wed-Thu)**
- [ ] Write comprehensive documentation
  - [ ] Architecture overview
  - [ ] Custom transform development guide
  - [ ] Performance characteristics
  - [ ] Lessons learned
- [ ] Code comments and examples
- [ ] Performance comparison charts

**Expected Output**: Complete documentation

### 3.2 Track B Deliverables

**Code**
```
prototype/cocoindex/
├── transforms/
│   ├── thread_parse.rs          NEW (ThreadParseTransform)
│   ├── extract_symbols.rs       NEW (ExtractSymbols transform)
│   └── mod.rs
├── pipelines/
│   ├── file_watch.rs            NEW (FileSystemSource)
│   └── storage.rs               NEW (Storage sinks)
├── types.rs                      NEW (Data structures)
├── benchmarks.rs                 NEW (Performance harness)
└── examples/
    ├── basic_pipeline.rs         NEW
    └── incremental_updates.rs    NEW

documentation/
├── cocoindex_integration.md      NEW
├── performance_analysis.md       NEW
├── type_system_bridge.md         NEW
└── extraction_path.md            NEW
```

**Documentation**
- Architecture overview (how CocoIndex enhances Thread)
- Custom transform development guide
- Performance benchmarks and analysis
- Type system bridging explanation
- Extraction path and dependency inversion strategy

**Metrics Output**
- Environment setup: ✅ or ❌
- Transforms working: ✅ or ❌
- Type system preservation: ✅ or ❌
- Performance metrics:
  - Initial index: X seconds, Y MB memory
  - Incremental update: X seconds (target 50x faster)
  - Cost reduction: X% (target 99%+)
- Dependency extraction: Feasible or not

### 3.3 Track B Success Criteria

| Criterion | Target | Priority | Go/No-Go |
|-----------|--------|----------|----------|
| CocoIndex environment working | ✅ | Critical | YES |
| Custom transforms functional | ✅ | Critical | YES |
| Type system bridge working | ✅ | High | YES |
| No metadata loss | 0% | High | YES |
| Extraction path feasible | Yes | High | YES |
| Incremental >50% speedup | ✅ | High | YES |
| Documentation complete | 100% | Medium | NO |

**Go Decision**: If 5 critical/high criteria met, Path B is viable

---

## Part 4: Week 3 - Evaluation & Decision Framework

### 4.1 Evaluation Timeline

**Monday, January 27** (Start of Week 3)
- [ ] Gather all metrics from both tracks
- [ ] Document findings in standardized format
- [ ] Identify any gaps in evaluation

**Tuesday, January 28**
- [ ] Score both approaches against decision framework
- [ ] Prepare presentation for decision meeting
- [ ] Architecture Lead + track owners review scoring

**Wednesday, January 29**
- [ ] Team meeting to discuss findings (2 hours)
- [ ] Debate pros/cons
- [ ] Address any questions or concerns
- [ ] Consensus building

**Thursday, January 30**
- [ ] Final decision meeting with stakeholders
- [ ] Announce chosen path
- [ ] Plan next phase based on decision
- [ ] Close out Path C planning

### 4.2 Decision Framework Scorecard

**Template for scoring both approaches:**

```
DECISION SCORECARD - Path C Week 3 Evaluation
Date: January 30, 2026

                            Path A          Path B          Winner
                         Services Only   + CocoIndex      (A/B/Mixed)
============================================================

PERFORMANCE (40% weight)
├─ Overhead vs target         X%              X%           _____
│  Target: <5%
├─ Incremental update speed   X%              X%           _____
│  Target: >50% faster with CocoIndex
└─ Memory efficiency          X%              X%           _____
  Subjective scoring

PERFORMANCE SCORE (40%):      X/100           X/100


TYPE SAFETY (20% weight)
├─ Metadata preservation      X/10            X/10         _____
│  (0=complete loss, 10=perfect preservation)
├─ Type system complexity     X/10            X/10         _____
│  (0=impossible, 10=trivial)
└─ Ease of bridging           X/10            X/10         _____
  (0=unclear, 10=obvious)

TYPE SAFETY SCORE (20%):      X/100           X/100


COMPLEXITY (15% weight)
├─ Lines of code              X               X            _____
│  Lower is better
├─ Abstraction layers         X               X            _____
│  Fewer is better
├─ Learning curve             X/10            X/10         _____
│  (0=very steep, 10=shallow)
└─ Maintenance burden         X/10            X/10         _____
  (0=heavy, 10=light)

COMPLEXITY SCORE (15%):       X/100           X/100


EXTENSIBILITY (15% weight)
├─ Adding new transforms      X/10            X/10         _____
│  (0=impossible, 10=trivial)
├─ Adding new engines         X/10            X/10         _____
│  (0=locked in, 10=pluggable)
├─ Real-time vision ready     X/10            X/10         _____
│  (0=major rework, 10=native fit)
└─ Phase 1/2 compatibility    X/10            X/10         _____
  (0=incompatible, 10=perfect)

EXTENSIBILITY SCORE (15%):    X/100           X/100


RISK (10% weight)
├─ Extraction risk (Path B)   X/10            X/10         _____
│  (0=locked in, 10=clean boundary)
├─ Unknown unknowns           X/10            X/10         _____
│  (0=many surprises, 10=well understood)
└─ Implementation risk        X/10            X/10         _____
  (0=high, 10=low)

RISK SCORE (10%):             X/100           X/100


============================================================
WEIGHTED TOTALS:
  Path A: (PERF*0.4 + TYPE*0.2 + COMPLEXITY*0.15 + EXT*0.15 + RISK*0.1)
  Path B: (PERF*0.4 + TYPE*0.2 + COMPLEXITY*0.15 + EXT*0.15 + RISK*0.1)

FINAL SCORES:                 X/100           X/100


============================================================
QUALITATIVE ASSESSMENT:

Path A Strengths:
  - 

Path A Weaknesses:
  - 

Path B Strengths:
  - 

Path B Weaknesses:
  - 

============================================================
RECOMMENDATION:

Choose Path: A / B / Mixed

Rationale:
  [2-3 paragraph justification based on scores and qualitative assessment]

Next Steps:
  [Timeline and actions for chosen path]
```

### 4.3 Scoring Guidance

#### Performance Dimension (40% weight)

**Metric 1: Overhead vs Target**
```
Measurement: (Time with abstraction - Time without) / Time without * 100

Path A:
  - Measure AstGrepParser overhead vs pure ast-grep parsing
  - Expected: 3-5%
  - Score: (100 - overhead_percentage)
  - Example: 3% overhead = 97 points

Path B:
  - Measure CocoIndex pipeline overhead vs pure Thread
  - Expected: <10% OR >50% speedup (acceptable trade-off for benefits)
  - Score: Based on whether target met
```

**Metric 2: Incremental Update Efficiency**
```
Test case: 1000-file codebase, change 10 files

Path A:
  - Pure services reparses all 10 changed files
  - No automatic incremental (manual implementation)
  - Score: 0-30 points (depends on manual optimization)

Path B:
  - CocoIndex's content-addressed cache skips unchanged
  - Expected: 50x+ faster per CocoIndex claims
  - Score: 100 points if target met
```

**Metric 3: Memory Efficiency**
```
Path A:
  - Single pass parsing, minimal caching
  - Expected: Lower memory
  - Score: 90-100 points if acceptable

Path B:
  - CocoIndex maintains state, content hashes
  - Expected: Higher memory for incremental benefits
  - Score: 70-90 points if trade-off justified
```

**Performance Score Calculation:**
```
Path A performance = (overhead_score + incremental_score + memory_score) / 3
Path B performance = (overhead_score + incremental_score + memory_score) / 3
```

#### Type Safety Dimension (20% weight)

**Metric 1: Metadata Preservation**
```
Test: Round-trip metadata through entire pipeline

Path A:
  - ParsedDocument → service layer → output
  - Check metadata integrity
  - Perfect preservation expected
  - Score: 10 points if 100% preserved, scale down for loss

Path B:
  - ParsedDocument → CocoIndex Row → output
  - Check metadata integrity through dataflow
  - Expected: Should preserve well
  - Score: 10 points if 100% preserved, scale down for loss
```

**Metric 2: Type System Complexity**
```
Path A:
  - Generic parameters over ast-grep types
  - Expected: Moderate complexity (ParsedDocument<D>)
  - Score: 8 points if understandable

Path B:
  - Type erasure into CocoIndex Row (untyped)
  - Lose compile-time safety
  - Score: 5-7 points (trade-off for flexibility)
```

**Metric 3: Ease of Bridging**
```
Path A:
  - Direct mapping to ast-grep types
  - Expected: Straightforward
  - Score: 9-10 points

Path B:
  - Map to/from untyped Row
  - Expected: Clear but requires care
  - Score: 7-8 points
```

**Type Safety Score:**
```
Path A type safety = (preservation + complexity + bridging) / 3
Path B type safety = (preservation + complexity + bridging) / 3
```

#### Complexity Dimension (15% weight)

**Metric 1: Lines of Code**
```
Path A: Count implementations (ast_grep.rs, etc.)
  - Expected: 500-800 lines
  - Score: 100 - (lines / 800 * 20)  [penalizes excessive code]

Path B: Count CocoIndex integration
  - Expected: 800-1200 lines (more infrastructure)
  - Score: 100 - (lines / 1200 * 20)
```

**Metric 2: Abstraction Layers**
```
Path A:
  - Layer 1: Service traits
  - Layer 2: ast-grep implementations
  - Count: 2 layers
  - Score: 90 points (minimal, clean)

Path B:
  - Layer 1: Service traits (optional)
  - Layer 2: CocoIndex transforms
  - Layer 3: Type bridging
  - Count: 3 layers
  - Score: 75 points (more abstraction, more power)
```

**Metric 3: Learning Curve**
```
Path A:
  - Developers already understand services, ast-grep
  - Expected: 1-2 day ramp-up
  - Score: 9-10 points

Path B:
  - Requires dataflow programming understanding
  - Expected: 3-5 day ramp-up
  - Score: 6-8 points
```

**Complexity Score:**
```
Path A complexity = (code_score + layers_score + learning_score) / 3
Path B complexity = (code_score + layers_score + learning_score) / 3
```

#### Extensibility Dimension (15% weight)

**Metric 1: Adding New Transforms**
```
Path A:
  - Add new service traits or implementations
  - Expected: Straightforward but requires trait changes
  - Score: 7 points (possible but bounded)

Path B:
  - Add new transforms to pipeline
  - Expected: Trivial (composable dataflow)
  - Score: 10 points (natural fit)
```

**Metric 2: Adding New Engines**
```
Path A:
  - Swap ast-grep for alternative parser
  - Expected: Major effort (trait changes ripple)
  - Score: 4-5 points (possible but painful)

Path B:
  - Swap CocoIndex for alternative dataflow engine
  - Expected: Possible if abstraction good
  - Score: 7-8 points (depends on extraction path findings)
```

**Metric 3: Real-Time Vision Ready**
```
Phase 2/3 vision: "Streaming changes → live coordination"

Path A:
  - Would need API redesign (streaming traits)
  - Expected: Significant rework
  - Score: 3-4 points (not naturally compatible)

Path B:
  - Dataflow naturally handles streaming
  - Expected: Additive improvements only
  - Score: 9-10 points (natural fit)
```

**Metric 4: Phase 1/2 Compatibility**
```
Path A:
  - Adds cross-file intelligence later
  - Expected: API changes needed
  - Score: 6-7 points (possible, disruptive)

Path B:
  - Already has cross-file capabilities
  - Expected: Incremental additions only
  - Score: 8-9 points (composable)
```

**Extensibility Score:**
```
Path A extensibility = (transforms + engines + realtime + future) / 4
Path B extensibility = (transforms + engines + realtime + future) / 4
```

#### Risk Dimension (10% weight)

**Metric 1: Extraction Risk (Path B specific)**
```
Question: If CocoIndex becomes unsuitable, can we extract?

Based on Track B findings:
  - 10 = Clean abstraction, easy extraction
  - 7-9 = Some coupling, extractable with effort
  - 4-6 = Deep coupling, significant rework needed
  - 1-3 = Locked in, nearly impossible to extract
  
Path A: Not applicable (no dependency), score: 10
Path B: Score based on actual findings
```

**Metric 2: Unknown Unknowns**
```
Path A:
  - Service traits are proven pattern
  - Expected: Few surprises
  - Score: 9 points (well-known territory)

Path B:
  - CocoIndex is new integration
  - Expected: Some unknowns
  - Score: 7 points (researched but untested at scale)
```

**Metric 3: Implementation Risk**
```
Path A:
  - Straightforward implementation
  - Expected: Low risk
  - Score: 9 points

Path B:
  - Type system bridging unknown risk
  - Expected: Medium risk
  - Score: 7 points
```

**Risk Score:**
```
Path A risk = (extraction_a + unknowns + impl_risk) / 3
Path B risk = (extraction_b + unknowns + impl_risk) / 3
```

### 4.4 Decision Scenarios

**Scenario 1: Path B Wins (Clear Victory)**
- Performance: >80 points
- Type Safety: >75 points
- Extensibility: >85 points
- Risk: >75 points
- **Total Path B: 300+, Path A: 270-**

**Decision**: Adopt dataflow architecture with dual layer (services API + dataflow internals)

**Next Steps**:
- Week of Feb 2: Design service/dataflow integration
- Week of Feb 9: Implement full services + CocoIndex integration
- Week of Feb 16: Integration testing and validation
- Target Phase 0 completion: February 27, 2026

---

**Scenario 2: Path A Wins (Clear Victory)**
- Performance: >80 points
- Type Safety: >80 points
- Complexity: >85 points
- Risk: >85 points
- **Total Path A: 310+, Path B: 260-**

**Decision**: Complete services-only Phase 0, evaluate dataflow for Phase 1

**Next Steps**:
- Week of Feb 2: Complete remaining services implementations
- Week of Feb 9: Full testing and validation
- Week of Feb 16: Performance optimization and polish
- Target Phase 0 completion: February 24, 2026

---

**Scenario 3: Mixed Results (No Clear Winner)**
- Scores within 20 points of each other
- Some dimensions clearly different

**Possible Decisions**:
a) **Path A First, B Later**: Complete Path A (proven, familiar), validate in production, then integrate CocoIndex in Phase 1
b) **Path B with Lighter Weight**: Keep services as optional external API layer, focus internal development on CocoIndex only
c) **Hybrid**: Use both approaches simultaneously (services for external API, CocoIndex for internal dataflow)

**Decision Factors**:
- Risk tolerance
- Long-term vision alignment
- Team's comfort with paradigm shift
- Time available before Phase 1

---

**Scenario 4: Both Viable, Different Strengths**
- Path A: Better performance, simpler, lower risk
- Path B: Better extensibility, better for long-term vision

**Recommendation**: Choose based on Phase 0 vs Phase 2 priorities
- Phase 0 focused: Path A (solid foundation)
- Long-term focused: Path B (better evolution)

---

### 4.5 Post-Decision Actions

**Immediate (Week of Jan 30 - Feb 2)**

If Path A Chosen:
- [ ] Archive Track B work (keep for reference)
- [ ] Plan Phase 0 completion (2-3 more weeks)
- [ ] Set up Phase 0 completion milestone
- [ ] Begin Phase 1 feature planning

If Path B Chosen:
- [ ] Archive Track A work (keep for reference)
- [ ] Design service/dataflow integration (Option C: Dual Layer)
- [ ] Plan integration work (2-3 more weeks)
- [ ] Design CocoIndex abstraction layer

If Hybrid/Mixed:
- [ ] Plan combination approach
- [ ] Schedule follow-up design session
- [ ] Define integration points

**Medium-term (Weeks 3-4)**

All Paths:
- [ ] Complete Phase 0 based on chosen architecture
- [ ] Comprehensive testing suite
- [ ] Performance validation
- [ ] Documentation complete
- [ ] Ready for Phase 1 planning

---

## Part 5: Success Metrics & Validation

### 5.1 Weekly Progress Tracking

**Track A Progress Tracker**

| Week | Milestone | Status | Blockers | % Complete |
|------|-----------|--------|----------|------------|
| 1-2 | Compilation fixed | [ ] | | |
| 1-2 | AstGrepParser impl | [ ] | | |
| 1-2 | AstGrepAnalyzer impl | [ ] | | |
| 1-2 | Mocks working | [ ] | | |
| 1-2 | Contract tests | [ ] | | |
| 1-2 | Integration tests | [ ] | | |
| 3 | Performance validated | [ ] | | |

**Track B Progress Tracker**

| Week | Milestone | Status | Blockers | % Complete |
|------|-----------|--------|----------|------------|
| 1 | Environment setup | [ ] | | |
| 1 | Design documented | [ ] | | |
| 2 | ThreadParse transform | [ ] | | |
| 2 | ExtractSymbols transform | [ ] | | |
| 2 | Pipeline wired | [ ] | | |
| 2 | Benchmarks running | [ ] | | |
| 3 | Type system validated | [ ] | | |
| 3 | Extraction path clear | [ ] | | |

### 5.2 Quality Gates

**Go/No-Go Gates for Week 3 Decision**

Track A Must-Pass (5/5):
- [ ] Workspace compiles without errors
- [ ] Tests pass at 95%+ rate
- [ ] Code coverage >80%
- [ ] Performance overhead <5%
- [ ] All trait methods implemented

Track B Must-Pass (5/5):
- [ ] CocoIndex environment working
- [ ] Custom transforms functional
- [ ] Type system bridge operational
- [ ] Benchmarks complete and analyzed
- [ ] Extraction path documented

**If either track fails must-pass gates**, that path becomes non-viable.

### 5.3 Metrics Collection Template

**Track A Metrics**
```yaml
Compilation:
  - Errors before: 36+
  - Errors after: X
  - Build time: X seconds
  - Status: [PASS/FAIL]

Testing:
  - Unit tests written: X
  - Integration tests written: X
  - Test pass rate: X%
  - Coverage: X%
  - Status: [PASS/FAIL]

Performance:
  - Parse 1KB file: X ms
  - Parse 100KB file: X ms
  - Memory overhead: X%
  - Overhead vs target: [PASS/FAIL]
  - Status: [PASS/FAIL]

Code Quality:
  - Implementation size: X lines
  - Trait methods: X/Y implemented
  - Documentation: X%
  - Status: [PASS/FAIL]
```

**Track B Metrics**
```yaml
Environment:
  - CocoIndex setup: [PASS/FAIL]
  - Dependencies resolved: [PASS/FAIL]
  - Examples running: [PASS/FAIL]
  - Status: [PASS/FAIL]

Transforms:
  - ThreadParse transform: [PASS/FAIL]
  - ExtractSymbols transform: [PASS/FAIL]
  - Type conversions: [PASS/FAIL]
  - Custom transform guide: [PASS/FAIL]
  - Status: [PASS/FAIL]

Performance:
  - Initial index (1000 files): X seconds, Y MB
  - Incremental (10 files): X seconds
  - Speedup: Xx faster
  - Cost reduction: X%
  - vs CocoIndex claims: [MET/PARTIAL/FAILED]
  - Status: [PASS/FAIL]

Type System:
  - Round-trip conversion: [PASS/FAIL]
  - Metadata preservation: X% (target 100%)
  - Information loss: X% (target 0%)
  - Type safety: [GOOD/ACCEPTABLE/POOR]
  - Status: [PASS/FAIL]

Extraction Path:
  - Abstraction layer designed: [PASS/FAIL]
  - CocoIndex-specific code isolated: [PASS/FAIL]
  - Extraction effort estimated: X days
  - Feasibility: [FEASIBLE/DIFFICULT/IMPOSSIBLE]
  - Status: [PASS/FAIL]
```

---

## Part 6: Risk Mitigation

### 6.1 Track A Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Type system too complex | Medium | Low | Simplify if needed, use newtype wrappers |
| Compilation errors overwhelming | Low | High | Break into smaller pieces, ask for help |
| Performance overhead too high | Medium | High | Profile early, optimize hot paths |
| Tests hard to write | Low | Medium | Use mocks extensively, contract-based testing |
| Timeline slips | Medium | High | Track daily, adjust scope if needed |

**Mitigation Actions**:
- Daily standups to catch blockers early
- Break implementation into 1-day chunks
- Test-driven development
- Performance profiling on Day 12
- Scope reduction plan if needed

### 6.2 Track B Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| CocoIndex steep learning curve | Medium | Medium | Spend Day 1 on deep learning, ask Cocoindex community |
| Type system bridging fails | Medium | High | Validate early (Day 3), prototype type conversions |
| Performance doesn't match claims | Medium | High | Benchmark daily, investigate deviations immediately |
| Information loss in conversion | Low | High | Test round-trips extensively |
| Integration issues not visible until late | Medium | Medium | Integration test continuously, daily |
| PostgreSQL/Qdrant dependencies | Low | Low | Use Docker containers, simplified setup |

**Mitigation Actions**:
- Detailed learning plan Day 1
- Type system validation checkpoint Day 3
- Performance benchmarking daily (Days 8-10)
- Continuous integration testing
- Docker containers for external dependencies

### 6.3 Track Independence Risks

| Risk | Mitigation |
|------|-----------|
| Tracks diverge in unexpected ways | Weekly syncs, shared test cases |
| One track blocks the other | Clear API contracts, loose coupling |
| Resources unavailable mid-track | Pre-commit team assignments |
| Evaluation framework biased | Objective metrics, formula-based scoring |

---

## Part 7: Executive Summary for Decision

### 7.1 What Path C Answers

**Primary Question**: Should we commit to services-only (Path A) or add dataflow infrastructure (Path B) to Thread?

**Secondary Questions**:
1. Can we successfully integrate CocoIndex without coupling?
2. Will incremental processing efficiency justify added complexity?
3. Can we achieve <5% overhead with services abstraction?
4. Are we ready for Phase 2 vision or just Phase 0?

**Path C Approach**: Build both, measure with real code, decide with data instead of speculation

### 7.2 Why Path C Works

| Aspect | Benefit |
|--------|---------|
| **Low Risk** | If wrong, only spent 3 weeks not 6-8 |
| **Evidence-Based** | Real code and benchmarks, not speculation |
| **Learning** | Team deeply understands both architectures |
| **Flexibility** | Can combine best of both if needed |
| **Confidence** | Decision backed by working prototypes |

### 7.3 Timeline at a Glance

```
Week 1-2: PARALLEL DEVELOPMENT
├─ Track A: Minimal services (proof of concept)
└─ Track B: CocoIndex prototype (proof of concept)

Week 3: EVALUATION
├─ Metrics collection (Monday)
├─ Scoring and analysis (Tuesday-Wednesday)
├─ Discussion and consensus (Wednesday)
└─ Decision announcement (Thursday)

Weeks 4+: EXECUTION
└─ Complete Phase 0 with chosen architecture
```

### 7.4 Expected Outcomes

**After Week 3, you will have:**
- ✅ Working minimal services implementation (Track A output)
- ✅ Working CocoIndex integration prototype (Track B output)
- ✅ Real performance benchmarks for both
- ✅ Clear understanding of trade-offs
- ✅ Objective scores for each path
- ✅ Team consensus on best direction
- ✅ Concrete plan for next 3 weeks

**This replaces speculation with evidence.**

---

## Part 8: Next Steps

### 8.1 To Begin Path C

**This Week (Week of January 9)**

- [ ] Approval: Get stakeholder sign-off on 3-week plan
- [ ] Team Assignment: Assign engineers to Track A and B
- [ ] Kick-off Meeting: Brief both teams on plan
- [ ] Repository Setup: Create tracking for both tracks
- [ ] Resource Provisioning: Ensure all tools/dependencies available

**Specific Actions**:
1. Schedule 30-min kick-off with team leads
2. Share this document with all stakeholders
3. Create shared tracking (spreadsheet or Jira)
4. Set up weekly sync meetings
5. Provision development environments

### 8.2 Key Dates

- **Start**: Monday, January 13, 2026
- **Mid-point sync**: Friday, January 17, 2026 (end of Week 1)
- **Mid-track adjustments**: Wednesday, January 22, 2026 (mid-Week 2)
- **Final metrics due**: Wednesday, January 29, 2026
- **Decision meeting**: Thursday, January 30, 2026
- **Phase 0 completion target**: February 24-27, 2026 (depending on path)

### 8.3 Success Criteria for Path C Itself

Path C succeeds if:
1. ✅ Both tracks complete with go/no-go assessments
2. ✅ Metrics collected and analyzed objectively
3. ✅ Team reaches consensus on chosen direction
4. ✅ Decision is made based on evidence, not politics
5. ✅ Clear plan exists for Phase 0 completion
6. ✅ Architecture supports long-term vision

---

## Appendix A: Track A Implementation Checklist

### Compilation Fixes
- [ ] Identify all 36+ compilation errors
- [ ] Add PhantomData markers
- [ ] Fix stub types
- [ ] Update feature flags
- [ ] Workspace builds cleanly

### AstGrepParser
- [ ] Struct definition
- [ ] parse_content implementation
- [ ] Metadata extraction
- [ ] Error handling
- [ ] Unit tests

### AstGrepAnalyzer
- [ ] Struct definition
- [ ] pattern_matches implementation
- [ ] analyze implementation
- [ ] Batch operations
- [ ] Unit tests

### Mocks & Tests
- [ ] MockParser implementation
- [ ] MockAnalyzer implementation
- [ ] Contract tests
- [ ] Integration tests
- [ ] 80%+ coverage

### Documentation
- [ ] Implementation guide
- [ ] Migration guide
- [ ] API documentation
- [ ] Performance guide
- [ ] Examples

---

## Appendix B: Track B Implementation Checklist

### Environment
- [ ] CocoIndex cloned and built
- [ ] PostgreSQL running
- [ ] Qdrant (optional) running
- [ ] Examples working
- [ ] Documentation reviewed

### Transforms
- [ ] ThreadParseTransform designed
- [ ] ThreadParseTransform implemented
- [ ] ExtractSymbols designed
- [ ] ExtractSymbols implemented
- [ ] Unit tests for each

### Pipeline
- [ ] FileSystemSource working
- [ ] Pipeline wiring complete
- [ ] End-to-end test working
- [ ] Incremental updates working
- [ ] Error handling

### Benchmarks
- [ ] Benchmark harness created
- [ ] Initial index benchmarks
- [ ] Incremental benchmarks
- [ ] Comparison vs pure Thread
- [ ] Cost analysis

### Documentation
- [ ] Architecture guide
- [ ] Transform development guide
- [ ] Performance analysis
- [ ] Type system bridge doc
- [ ] Extraction path doc

---

## Appendix C: Decision Framework Reference

**If stuck on a decision during Week 3:**

1. **Look at must-pass criteria first**: Did both tracks pass their critical gates?
2. **Compare on "long-term vision alignment"**: Which path supports Phase 2/3 better?
3. **Consider risk tolerance**: How much extra complexity is acceptable?
4. **Think about team**: Which path feels more natural to maintain?
5. **Check against original concern**: Which path avoids "fighting your architecture"?

---

## Document Status

**Status**: COMPLETE - Ready for execution  
**Date**: January 9, 2026  
**Next Review**: January 30, 2026 (decision date)  
**Owner**: Architecture Team  
**Classification**: Strategic Implementation Plan

**To Begin**: Secure stakeholder approval and assign teams to Track A and B.

