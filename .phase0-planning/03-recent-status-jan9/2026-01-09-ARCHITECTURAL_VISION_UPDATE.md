# Thread Architectural Vision

**Status**: Working Document for Architectural Refinement  
**Last Updated**: January 7, 2026  
**Context**: This document captures architectural direction emerging from analysis of CocoIndex's dataflow model and Thread's long-term vision. It's intended as a starting point for architectural specialists to brainstorm and refine.

---

## Executive Summary

Thread is infrastructure for a category of products that doesn't exist yet: real-time coordination systems where humans and AI collaborate on knowledge work, with AI handling what humans are worst at (large data sets, complex interdependencies, large-scale awareness) while humans focus on experience, value, and usefulness.

The immediate goal is helping AI understand codebases in forms AI can work with natively. The architecture must support this near-term goal while remaining adaptable to the broader vision.

**Key architectural decision**: Adopt dataflow as the core internal paradigm, built on CocoIndex's infrastructure, with Thread's existing service traits as the external API layer.

---

## The Long-Term Vision

### Where Thread Is Going (v2+)

Thread evolves into a real-time coordination engine:

```
ÚÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄ¿
³                         Thread                               ³
³                                                              ³
³  Inputs (streaming):              Outputs (reactive):        ³
³   Local file changes              Notifications to humans  ³
³   PR/branch state                 AI agent instructions    ³
³   Sprint/schedule data            Conflict predictions     ³
³   Team chat/docs                  Suggested actions        ³
³   External APIs                   Materialized files       ³
³                                                              ³
³            ÚÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄ¿
³            ³              Semantic Graph                     ³
³            ³    Versioned (rollback broken changes)         ³
³            ³    Forkable (branch for experimentation)       ³
³            ³    Mergeable (reconcile parallel work)         ³
³            ÀÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÙ
³                                                              ³
³  Continuously computing: "Who needs to know what, when?"    ³
ÀÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÙ
```

**Example scenario**: Dave is working locally on a feature. Thread detects his changes touch code that Berlin's team is sprinting on this week. Before Dave even submits a PR, Thread messages both Dave and Berlin: "You should sync up."

**Key insight**: The graph becomes the source of truth, not the files. Files are one rendering of the graph-the one humans need. AI agents work with the graph directly, enabling atomic codebase changes in minutes.

### What Thread Enables

A model for knowledge work where:
- Humans work continuously with AI assistants
- Creativity is the only constraint
- AI solves coordination, complexity, and scale problems
- Humans focus on experience, genuine value, and usefulness

---

## Why Dataflow

### The Paradigm Decision

| Paradigm | Optimizes For | Thread Fit |
|----------|---------------|------------|
| **Services** | Request/response, API boundaries, testability | External interfaces û |
| **Dataflow** | Transformation pipelines, incremental updates, parallelism, composability | Internal processing û |
| **Event Sourcing** | Audit trails, replay, time-travel | Maybe later for versioning |
| **Graph-centric** | Relationship traversal, pattern matching | Data model, not computation model |

**Services** create rigid boundaries. Good for stable APIs, bad for "I don't know what I'll need yet."

**Dataflow** creates composable transformations. Source  Transform  Transform  Sink. Need new data sources? Add a source. Need new processing? Add a transform. Need new outputs? Add a sink.

Thread's stage requires adaptability over rigidity. The vision involves layering intelligence on data sources we haven't identified yet, generating outputs we haven't imagined yet. Dataflow's composability is essential.

### Architecture Layers

```
ÚÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄ¿
³             External Interface: Service Traits               ³
³     (Stable API contracts for consumers of Thread)          ³
³     CodeParser, CodeAnalyzer, StorageService, etc.          ³
ÃÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄ´
³             Internal Processing: Dataflow                    ³
³     (Composable, adaptable transformation graphs)           ³
³     Built on CocoIndex primitives                           ³
ÃÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄ´
³             Infrastructure: CocoIndex                        ³
³     (Incremental dataflow, storage backends, lineage)       ³
³     Don't build plumbing                                    ³
ÃÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄ´
³             Thread's Focus: Semantic Intelligence            ³
³     (Deep code understanding, relationship extraction,      ³
³      AI context optimization, human-AI bridge)              ³
ÀÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÙ
```

---

## CocoIndex as Infrastructure

### What CocoIndex Provides

CocoIndex is a dataflow-based indexing engine built by ex-Google infrastructure engineers. Apache 2.0 licensed with ~5.4k GitHub stars.

**Core capabilities:**
- Declarative dataflow pipelines (Source  Transform  Sink)
- Incremental processing (only recompute what changed)
- Data lineage tracking (observable transformations)
- Multiple storage backends (Postgres, Qdrant, LanceDB, Graph DBs)
- Rust core with Python API
- Production-ready file watching and ingestion

**CocoIndex's dataflow API:**

```
ÚÄÄÄÄÄÄÄÄÄÄÄÄÄ¿   ÚÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄ¿   ÚÄÄÄÄÄÄÄÄÄÄÄÄÄ¿
³ LOCAL FILES ³   ³            TRANSFORMS               ³   ³RELATIONAL DB³
ÃÄÄÄÄÄÄÄÄÄÄÄÄÄ´   ÃÄÄÄÄÄÄÄÂÄÄÄÄÄÄÄÂÄÄÄÄÄÄÄÂÄÄÄÄÄÄÄÄÄÄÄÄ´   ÃÄÄÄÄÄÄÄÄÄÄÄÄÄ´
³INGESTION API³ÄÄ?³ PARSE ³ CHUNK ³ DEDUP ³            ³ÄÄ?³  VECTOR DB  ³
ÃÄÄÄÄÄÄÄÄÄÄÄÄÄ´   ÃÄÄÄÄÄÄÄÅÄÄÄÄÄÄÄÅÄÄÄÄÄÄÄÅÄÄÄÄÄÄÄÄÄÄÄÄ´   ÃÄÄÄÄÄÄÄÄÄÄÄÄÄ´
³CLOUD STORAGE³   ³EXTRACT³ EMBED ³CLUSTER³ RECONCILE  ³   ³  GRAPH DB   ³
ÃÄÄÄÄÄÄÄÄÄÄÄÄÄ´   ³STRUCT ³       ³       ³            ³   ÃÄÄÄÄÄÄÄÄÄÄÄÄÄ´
³     WEB     ³   ³       ³  MAP  ³       ³            ³   ³  OBJECT DB  ³
ÀÄÄÄÄÄÄÄÄÄÄÄÄÄÙ   ÀÄÄÄÄÄÄÄÁÄÄÄÄÄÄÄÁÄÄÄÄÄÄÄÁÄÄÄÄÄÄÄÄÄÄÄÄÙ   ÀÄÄÄÄÄÄÄÄÄÄÄÄÄÙ
```

### What CocoIndex Lacks (Thread's Differentiation)

CocoIndex uses tree-sitter for better chunking, not semantic analysis. Their "code embedding" example is generic text chunking with language-aware splitting.

**CocoIndex does NOT provide:**
- Symbol extraction (functions, classes, variables)
- Cross-file relationship tracking (calls, imports, inherits)
- Code graph construction
- AI context optimization
- Semantic understanding of code structure

**Technical evidence**: CocoIndex has 27 tree-sitter parsers as direct dependencies (not 166). Most languages fall back to regex-based splitting. Their chunking is sophisticated but shallow-they parse to chunk better, not to understand code.

### Integration Model

Thread plugs into CocoIndex's dataflow as custom transforms:

```
CocoIndex.LocalFiles 
     CocoIndex.Parse (basic) 
     Thread.DeepParse (ast-grep, semantic extraction)
     Thread.ExtractRelationships (symbols, calls, imports)
     Thread.BuildGraph (petgraph)
     CocoIndex.GraphDB / Qdrant / Postgres
```

Thread gets:
- File watching and ingestion (free)
- Incremental processing (free)
- Storage backends (free)
- Lineage tracking (free)

Thread focuses on:
- Deep semantic extraction (ast-grep integration)
- Relationship graph construction
- AI context generation
- Intelligence primitives

---

## Existing Architecture Assessment

### Current Thread Crates

```
crates/
ÃÄÄ ast-engine/    # ast-grep integration (solid foundation)
ÃÄÄ language/      # Language support (20+ languages)
ÃÄÄ rule-engine/   # Pattern matching rules
ÃÄÄ services/      # Service traits (well-designed, no implementations)
ÃÄÄ utils/         # Shared utilities
ÀÄÄ wasm/          # WASM bindings
```

### Service Traits Analysis

The existing service traits in `crates/services/src/traits/` are well-designed:

**`parser.rs` - CodeParser trait**
- `parse_content()`, `parse_file()`, `parse_multiple_files()`
- `ParserCapabilities` describing features and limits
- `ExecutionStrategy` enum (Sequential, Rayon, Chunked)
- Already supports different execution environments

**`analyzer.rs` - CodeAnalyzer trait**
- Pattern matching and analysis interfaces
- Cross-file analysis capabilities flagged

**`storage.rs` - StorageService trait**
- Persistence interfaces
- Feature-gated for commercial separation

**`types.rs` - Core types**
- `ParsedDocument<D>` - wraps ast-grep with metadata
- `DocumentMetadata` - symbols, imports, exports, calls, types
- `CrossFileRelationship` - calls, imports, inherits, implements, uses
- `AnalysisContext`, `ExecutionScope`, `ExecutionStrategy`

### Assessment

The traits are **externally-focused API contracts**. They're good for how consumers interact with Thread. But they don't currently express the internal dataflow model.

**Key question**: How do these traits relate to a dataflow-based internal architecture?

---

## Architectural Adaptation

### Option A: Traits as Dataflow Node Wrappers

Service traits become interfaces to dataflow nodes:

```rust
// CodeParser trait wraps a dataflow transform
impl CodeParser for ThreadParser {
    async fn parse_file(&self, path: &Path, ctx: &AnalysisContext) 
        -> ServiceResult<ParsedDocument<impl Doc>> 
    {
        // Internally, this triggers a dataflow pipeline
        self.dataflow
            .source(FileSource::single(path))
            .transform(AstGrepParse::new())
            .transform(ExtractMetadata::new())
            .execute_one()
            .await
    }
}
```

**Pros**: Preserves existing API, internal change only  
**Cons**: Might fight dataflow's streaming nature

### Option B: Traits as Dataflow Pipeline Builders

Traits shift to describe pipeline configurations:

```rust
pub trait CodeParser: Send + Sync {
    /// Configure a parsing pipeline for batch processing
    fn configure_pipeline(&self, builder: &mut PipelineBuilder) -> Result<()>;
    
    /// Single-file convenience (runs minimal pipeline)
    async fn parse_file(&self, path: &Path) -> ServiceResult<ParsedDocument>;
}
```

**Pros**: More natural fit with dataflow  
**Cons**: API change from current design

### Option C: Separate Concerns Explicitly

Two layers of abstraction:

```rust
// Layer 1: Dataflow transforms (internal)
pub trait Transform: Send + Sync {
    type Input;
    type Output;
    fn transform(&self, input: Self::Input) -> Self::Output;
}

// Layer 2: Service API (external)
pub trait CodeParser: Send + Sync {
    // Current API preserved
    // Implemented by composing transforms
}
```

**Pros**: Clean separation, both models coexist  
**Cons**: More abstraction layers

### Open Questions for Architects

1. **Streaming vs Batch**: The current traits are request/response. Dataflow is naturally streaming. How do we reconcile? Do we need streaming variants of the traits?

2. **Incremental Updates**: CocoIndex tracks what changed. How do Thread's service traits express "only re-analyze changed files"? Is this implicit (infrastructure handles it) or explicit (API expresses it)?

3. **Pipeline Composition**: If Thread exposes transforms, how do users compose them? Do we expose CocoIndex's builder directly? Wrap it? Abstract it?

4. **Type Flow**: Current types (`ParsedDocument`, `DocumentMetadata`, etc.) are Rust structs. CocoIndex dataflow uses its own type system. How do we bridge?

5. **Error Handling**: Dataflow errors can be partial (some files failed). Current traits return `ServiceResult`. How do we express partial success?

6. **Caching/Memoization**: CocoIndex handles incremental caching at infrastructure level. Do Thread's traits need to express caching semantics, or is it transparent?

---

## CodeWeaver Relationship

### Current Stack

```
CodeWeaver (MCP interface + search quality)
    
Thread (semantic intelligence)  WE ARE HERE
    
CocoIndex (infrastructure)
```

### CodeWeaver's Unique Value

CodeWeaver provides search quality that CocoIndex lacks:
- 17 embedding providers
- 6 reranking model providers  
- 2 sparse embedding providers
- Hybrid search with RRF fusion
- Multivector by default

CocoIndex does: chunk  embed  store (basic vector search)  
CodeWeaver does: chunk  embed (multi) + sparse  rerank  hybrid fusion

### Integration Path

After Alpha 6 ships, CodeWeaver enters maintenance mode. Thread becomes the focus.

Long-term, CodeWeaver is either:
- **A**: The MCP-specific thin client wrapping Thread/CocoIndex
- **B**: Absorbed into Thread as one output mode
- **C**: Independent simpler tool for different market segment

This decision deferred until Thread's architecture solidifies.

---

## Phase 0 Revised: Dataflow Foundation

### Objectives

1. Validate CocoIndex integration approach
2. Implement Thread transforms that plug into CocoIndex dataflow
3. Preserve existing service traits as external API
4. Demonstrate: File  Parse  Extract  Graph pipeline

### Concrete Steps

**Week 1-2: CocoIndex Exploration**
- Set up CocoIndex in Thread development environment
- Build minimal custom transform (hello world level)
- Understand their type system and extension points

**Week 2-3: Bridge Implementation**
- Implement `ThreadParse` transform (wraps ast-grep)
- Implement `ExtractSymbols` transform
- Implement `ExtractRelationships` transform
- Wire into CocoIndex pipeline: File  ThreadParse  Extract  Qdrant

**Week 3-4: Service Trait Adaptation**
- Decide on trait adaptation approach (A, B, or C above)
- Implement `CodeParser` against new dataflow internals
- Ensure existing service trait tests pass

**Week 4+: Validation**
- Benchmark against pure-Thread implementation
- Validate incremental update behavior
- Document architecture decisions

### Success Criteria

- [ ] Thread transforms run in CocoIndex pipeline
- [ ] Incremental updates work (change one file, only that file reprocesses)
- [ ] Service traits remain stable external API
- [ ] Performance within 10% of current implementation
- [ ] Graph output matches current Thread extraction quality

---

## Open Questions

### Architectural

1. How does versioned/forkable graph state work with CocoIndex's model?
2. What's the transaction model for AI agent graph edits?
3. How do we handle the "Dave and Berlin" real-time notification scenario computationally?
4. Is petgraph still the right graph representation, or does CocoIndex's graph DB integration change that?

### Strategic

1. What level of CocoIndex coupling is acceptable? (Light integration vs deep dependency)
2. If CocoIndex pivots or dies, what's the extraction path?
3. Should Thread contribute upstream to CocoIndex?
4. How does this affect AGPL licensing (CocoIndex is Apache 2.0)?

### Technical

1. Rust-to-Rust integration (Thread) with Rust+Python system (CocoIndex)?
2. Performance characteristics of custom transforms in CocoIndex?
3. How does CocoIndex handle transform failures and partial results?
4. What's the debugging/observability story for complex pipelines?

---

## Summary

**Decision**: Dataflow is the internal paradigm for Thread, built on CocoIndex infrastructure.

**Rationale**: Thread's vision requires adaptability over rigidity. Dataflow's composability-add sources, transforms, and sinks as needs emerge-aligns with building infrastructure for products that don't exist yet.

**Preservation**: Existing service traits remain as external API contracts. They become the stable interface through which consumers interact with Thread, backed by dataflow internals.

**Focus**: Thread's differentiation is semantic intelligence-deep code understanding, relationship extraction, AI context optimization. CocoIndex handles the plumbing.

**Next**: Validate this architecture through hands-on CocoIndex integration before committing fully.

---

*This document is a starting point for discussion, not a final specification. The goal is to sharpen the architectural vision through iteration and expert input.*
