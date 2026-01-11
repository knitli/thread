# PATH B: CocoIndex Integration - Implementation Guide
**Service-First Architecture with Rust-Native Dataflow Processing**

**Date:** January 10, 2026
**Duration:** 3 Weeks (January 13 - January 31, 2026)
**Status:** **CONFIRMED** - Rust-native approach validated
**Decision Basis:** Service-first requirements + pure Rust performance

---

## Executive Summary

Thread is a **service-first architecture** - a long-lived, persistent, real-time updating service designed for cloud deployment (Cloudflare edge) and local development (CLI). This requirement fundamentally validates **Path B (CocoIndex integration)** as the correct architectural choice.

### Critical Decision: Rust-Native Integration

Based on COCOINDEX_API_ANALYSIS.md findings, we will use CocoIndex as a **pure Rust library dependency**, not via Python bindings. This provides:

✅ **Zero Python overhead** - No PyO3 bridge, pure Rust performance
✅ **Full type safety** - Compile-time guarantees, no runtime type errors
✅ **Direct API access** - LibContext, FlowContext, internal execution control
✅ **Simpler deployment** - Single Rust binary to Cloudflare
✅ **Better debugging** - Rust compiler errors vs Python runtime exceptions

### Critical Context: Service-First Architecture

Thread is **NOT** a library that returns immediate results. It is:
- ✅ **Long-lived service** - Persistent, continuously running
- ✅ **Real-time updating** - Incrementally processes code changes
- ✅ **Cached results** - Stores analysis for instant retrieval
- ✅ **Cloud-native** - Designed for Cloudflare edge deployment
- ✅ **Dual concurrency** - Rayon (CPU parallelism local) + tokio (async cloud/edge)
- ✅ **Always persistent** - All use cases benefit from caching/storage

### Why Path B Wins (6-0 on Service Requirements)

| Requirement | Path A (Services-Only) | Path B (CocoIndex) | Winner |
|-------------|------------------------|--------------------| ------|
| **Persistent Storage** | Must build from scratch | ✅ Built-in Postgres/D1/Qdrant | **B** |
| **Incremental Updates** | Must implement manually | ✅ Content-addressed caching | **B** |
| **Real-time Intelligence** | Custom change detection | ✅ Automatic dependency tracking | **B** |
| **Cloud/Edge Deployment** | Custom infrastructure | ✅ Serverless containers + D1 | **B** |
| **Concurrency Model** | Rayon only (local) | ✅ tokio async (cloud/edge) | **B** |
| **Data Quality** | Manual implementation | ✅ Built-in freshness/lineage | **B** |

**Result**: Path B is the **only viable architecture** for service-first Thread.

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Feasibility Validation](#feasibility-validation)
3. [4-Week Implementation Plan](#4-week-implementation-plan)
4. [Rust ↔ Python Bridge Strategy](#rust--python-bridge-strategy)
5. [Edge Deployment Architecture](#edge-deployment-architecture)
6. [Thread's Semantic Intelligence](#threads-semantic-intelligence)
7. [Success Criteria](#success-criteria)
8. [Risk Mitigation](#risk-mitigation)

---

## Architecture Overview

### Rust-Native Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Thread Service Layer                      │
│  ┌────────────────────────────────────────────────────────┐ │
│  │   Public API (thread-services)                         │ │
│  │   - CodeParser, CodeAnalyzer, StorageService traits    │ │
│  │   - Request/response interface for clients             │ │
│  └────────────────┬───────────────────────────────────────┘ │
│                   │                                          │
│  ┌────────────────▼───────────────────────────────────────┐ │
│  │   Internal Processing (CocoIndex Dataflow)             │ │
│  │   - Thread operators as native Rust traits             │ │
│  │   - Incremental ETL pipeline                           │ │
│  │   - Content-addressed caching                          │ │
│  │   - Automatic dependency tracking                      │ │
│  └────────────────┬───────────────────────────────────────┘ │
└───────────────────┼──────────────────────────────────────────┘
                    │
┌───────────────────▼──────────────────────────────────────────┐
│         CocoIndex Framework (Rust Library Dependency)         │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │   Sources   │→ │  Functions   │→ │    Targets       │   │
│  │ LocalFile   │  │ ThreadParse  │  │ Postgres / D1    │   │
│  │ D1 (custom) │  │ ExtractSyms  │  │ Qdrant (vectors) │   │
│  └─────────────┘  └──────────────┘  └──────────────────┘   │
│                                                               │
│  All operators implemented as Rust traits:                   │
│  - SourceFactory, SimpleFunctionFactory, TargetFactory       │
│  - Zero Python overhead, full type safety                    │
└──────────────────────────────────────────────────────────────┘
```

### Rust Native Integration

```rust
// Cargo.toml
[dependencies]
cocoindex = { git = "https://github.com/cocoindex-io/cocoindex" }
thread-ast-engine = { path = "../../crates/thread-ast-engine" }

// Thread operators as native Rust traits
use cocoindex::ops::interface::{SimpleFunctionFactory, SimpleFunctionExecutor};
use thread_ast_engine::{parse, Language};

pub struct ThreadParseFunction;

#[async_trait]
impl SimpleFunctionFactory for ThreadParseFunction {
    async fn build(
        self: Arc<Self>,
        spec: serde_json::Value,
        context: Arc<FlowInstanceContext>,
    ) -> Result<SimpleFunctionBuildOutput> {
        // Direct Rust implementation, no Python bridge
        Ok(SimpleFunctionBuildOutput {
            executor: Arc::new(ThreadParseExecutor),
            // ...
        })
    }
}

// All processing in Rust, maximum performance
```

### Concurrency Strategy

**Local Development (CLI)**:
- **Rayon** - CPU-bound parallelism for fast local parsing
- Single machine, multi-core utilization

**Cloud/Edge Deployment (Cloudflare)**:
- **tokio** - Async I/O for horizontal scaling
- Workers → Durable Objects → D1
- Serverless containers for compute
- Distributed processing across edge network

**Why Both Work**: CocoIndex natively supports tokio async, Thread adds CPU parallelism via custom Rust transforms.

---

## Feasibility Validation

### Proof: CocoIndex Example from Docs

The CocoIndex documentation provides a **working example** that proves Thread's exact use case:

```python
import cocoindex

@cocoindex.flow_def(name="CodeEmbedding")
def code_embedding_flow(flow_builder, data_scope):
    # 1. SOURCE: File system watching
    data_scope["files"] = flow_builder.add_source(
        cocoindex.sources.LocalFile(
            path="../..",
            included_patterns=["*.py", "*.rs", "*.toml", "*.md"],
            excluded_patterns=["**/.*", "target", "**/node_modules"]
        )
    )

    code_embeddings = data_scope.add_collector()

    # 2. TRANSFORM: Tree-sitter semantic chunking
    with data_scope["files"].row() as file:
        file["language"] = file["filename"].transform(
            cocoindex.functions.DetectProgrammingLanguage()
        )

        # CRITICAL: SplitRecursively uses tree-sitter!
        file["chunks"] = file["content"].transform(
            cocoindex.functions.SplitRecursively(),
            language=file["language"],
            chunk_size=1000,
            min_chunk_size=300,
            chunk_overlap=300
        )

        # 3. TRANSFORM: Embeddings (Thread would do Symbol/Import/Call extraction)
        with file["chunks"].row() as chunk:
            chunk["embedding"] = chunk["text"].call(code_to_embedding)

            code_embeddings.collect(
                filename=file["filename"],
                location=chunk["location"],
                code=chunk["text"],
                embedding=chunk["embedding"],
                start=chunk["start"],
                end=chunk["end"]
            )

    # 4. TARGET: Multi-target export with vector indexes
    code_embeddings.export(
        "code_embeddings",
        cocoindex.targets.Postgres(),
        primary_key_fields=["filename", "location"],
        vector_indexes=[
            cocoindex.VectorIndexDef(
                field_name="embedding",
                metric=cocoindex.VectorSimilarityMetric.COSINE_SIMILARITY
            )
        ]
    )
```

### What This Proves

✅ **File watching** - CocoIndex handles incremental file system monitoring
✅ **Tree-sitter integration** - `SplitRecursively()` already uses tree-sitter parsers
✅ **Semantic chunking** - Respects code structure, not naive text splitting
✅ **Custom transforms** - Can call Python functions (we'll call Rust via PyO3)
✅ **Multi-target export** - Postgres with vector indexes built-in
✅ **Content addressing** - Automatic change detection and incremental processing

**What Thread Adds**: Deep semantic intelligence (symbols, imports, calls, relationships) instead of just chunking.

---

## 3-Week Implementation Plan

**Why 3 Weeks (not 4)**: Rust-native approach eliminates Python bridge complexity, saving ~1 week.

### Week 1: Foundation & Design (Jan 13-17)

**Goal**: CocoIndex Rust API mastery + Thread operator design

#### Day 1 (Monday) - Rust Environment Setup
```bash
# Clone CocoIndex
git clone https://github.com/cocoindex-io/cocoindex
cd cocoindex

# Build CocoIndex Rust crates
cargo build --release

# Setup Postgres (CocoIndex state store)
docker run -d \
  --name cocoindex-postgres \
  -e POSTGRES_PASSWORD=cocoindex \
  -p 5432:5432 \
  postgres:16

# Study Rust examples (not Python)
cargo run --example simple_source
cargo run --example custom_function
```

**Tasks**:
- [ ] Review CocoIndex Rust architecture (Section 2 of API analysis)
- [ ] Study operator trait system (`ops/interface.rs`)
- [ ] Analyze builtin operator implementations:
  - [ ] `ops/sources/local_file.rs` - File source pattern
  - [ ] `ops/functions/parse_json.rs` - Function pattern
  - [ ] `ops/targets/postgres.rs` - Target pattern
- [ ] Understand LibContext, FlowContext lifecycle
- [ ] Map Thread's needs to CocoIndex operators

**Deliverable**: Rust environment working, trait system understood

---

#### Day 2 (Tuesday) - Operator Trait Design
**Reference**: `/home/knitli/thread/COCOINDEX_API_ANALYSIS.md` Section 2.2

**Tasks**:
- [ ] Design ThreadParseFunction (SimpleFunctionFactory)
  ```rust
  pub struct ThreadParseFunction;

  #[async_trait]
  impl SimpleFunctionFactory for ThreadParseFunction {
      async fn build(...) -> Result<SimpleFunctionBuildOutput> {
          // Parse code with thread-ast-engine
          // Return executor that processes Row inputs
      }
  }
  ```
- [ ] Design ExtractSymbolsFunction
- [ ] Design ExtractImportsFunction
- [ ] Design ExtractCallsFunction
- [ ] Plan Row schema for parsed code:
  ```rust
  // Input Row: {content: String, language: String, path: String}
  // Output Row: {
  //   ast: Value,           // Serialized AST
  //   symbols: Vec<Symbol>, // Extracted symbols
  //   imports: Vec<Import>, // Import statements
  //   calls: Vec<Call>      // Function calls
  // }
  ```

**Deliverable**: Operator trait specifications documented

---

#### Day 3 (Wednesday) - Value Type System Design

**Pure Rust Approach** - No Python conversion needed!

```rust
use cocoindex::base::value::{Value, ValueType};
use cocoindex::base::schema::FieldSchema;

// Thread's parsed output → CocoIndex Value
fn serialize_parsed_doc(doc: &ParsedDocument) -> Result<Value> {
    let mut fields = HashMap::new();

    // Serialize AST
    fields.insert("ast".to_string(), serialize_ast(&doc.root)?);

    // Serialize symbols
    fields.insert("symbols".to_string(), Value::Array(
        doc.symbols.iter()
            .map(|s| serialize_symbol(s))
            .collect::<Result<Vec<_>>>()?
    ));

    // Serialize imports
    fields.insert("imports".to_string(), serialize_imports(&doc.imports)?);

    // Serialize calls
    fields.insert("calls".to_string(), serialize_calls(&doc.calls)?);

    Ok(Value::Struct(fields))
}
```

**Tasks**:
- [ ] Define CocoIndex ValueType schema for Thread's output
- [ ] Implement Thread → CocoIndex Value serialization
- [ ] Preserve all AST metadata (no information loss)
- [ ] Design symbol/import/call Value representations
- [ ] Plan schema validation strategy
- [ ] Design round-trip tests (Value → Thread types → Value)

**Deliverable**: Value serialization implementation

---

#### Day 4 (Thursday) - D1 Custom Source/Target Design

**Cloudflare D1 Integration**:

```rust
// D1 Source (read indexed code from edge)
pub struct D1Source {
    database_id: String,
    binding: String,  // Cloudflare binding name
}

#[async_trait]
impl SourceFactory for D1Source {
    async fn build(...) -> Result<SourceBuildOutput> {
        // Connect to D1 via wasm_bindgen
        // Query: SELECT file_path, content, hash FROM code_index
        // Stream results as CocoIndex rows
    }
}

// D1 Target (write analysis results to edge)
pub struct D1Target {
    database_id: String,
    table_name: String,
}

#[async_trait]
impl TargetFactory for D1Target {
    async fn build(...) -> Result<...> {
        // Create table schema in D1
        // Bulk insert analysis results
        // Handle conflict resolution (upsert)
    }
}
```

**Tasks**:
- [ ] Research Cloudflare D1 API (SQL over HTTP)
- [ ] Design schema for code index table:
  ```sql
  CREATE TABLE code_index (
      file_path TEXT PRIMARY KEY,
      content_hash TEXT NOT NULL,
      language TEXT,
      symbols JSON,      -- Symbol table
      imports JSON,      -- Import graph
      calls JSON,        -- Call graph
      metadata JSON,     -- File-level metadata
      indexed_at TIMESTAMP,
      version INTEGER
  );
  ```
- [ ] Design D1 source/target interface
- [ ] Plan migration from Postgres (local) to D1 (edge)

**Deliverable**: D1 integration design document

---

#### Day 5 (Friday) - Week 1 Review & Planning

**Tasks**:
- [ ] Document learning from Week 1
- [ ] Finalize Week 2-4 task breakdown
- [ ] Identify risks and mitigation strategies
- [ ] Create detailed implementation checklist
- [ ] Team sync: present design, get feedback

**Deliverable**: Week 2-4 detailed plan approved

---

### Week 2: Core Implementation (Jan 20-24)

**Goal**: Implement ThreadParse + ExtractSymbols transforms

#### Days 6-7 (Mon-Tue) - ThreadParse Function Implementation

**Pure Rust Implementation**:

```rust
// crates/thread-cocoindex/src/functions/parse.rs
use cocoindex::ops::interface::{SimpleFunctionFactory, SimpleFunctionExecutor};
use thread_ast_engine::{parse, Language};
use async_trait::async_trait;

pub struct ThreadParseFunction;

#[async_trait]
impl SimpleFunctionFactory for ThreadParseFunction {
    async fn build(
        self: Arc<Self>,
        spec: serde_json::Value,
        context: Arc<FlowInstanceContext>,
    ) -> Result<SimpleFunctionBuildOutput> {
        Ok(SimpleFunctionBuildOutput {
            executor: Arc::new(ThreadParseExecutor),
            output_value_type: build_output_schema(),
            enable_cache: true,  // Content-addressed caching
            timeout: Some(Duration::from_secs(30)),
        })
    }
}

pub struct ThreadParseExecutor;

#[async_trait]
impl SimpleFunctionExecutor for ThreadParseExecutor {
    async fn evaluate(&self, input: Vec<Value>) -> Result<Value> {
        // Extract input fields
        let content = input[0].as_string()?;
        let language = input[1].as_string()?;

        // Parse with Thread's engine
        let lang = Language::from_str(language)?;
        let doc = parse(content, lang)?;

        // Convert to CocoIndex Value
        serialize_parsed_doc(&doc)
    }

    fn enable_cache(&self) -> bool { true }
    fn timeout(&self) -> Option<Duration> { Some(Duration::from_secs(30)) }
}

fn build_output_schema() -> EnrichedValueType {
    // Define schema for parsed output
    EnrichedValueType::Struct(StructType {
        fields: vec![
            FieldSchema::new("ast", ValueType::Json),
            FieldSchema::new("symbols", ValueType::Array(Box::new(symbol_type()))),
            FieldSchema::new("imports", ValueType::Array(Box::new(import_type()))),
            FieldSchema::new("calls", ValueType::Array(Box::new(call_type()))),
        ]
    })
}
```

**Tasks**:
- [ ] Create `thread-cocoindex` crate (Rust library)
- [ ] Implement SimpleFunctionFactory for ThreadParse
- [ ] Implement SimpleFunctionExecutor with Thread parsing
- [ ] Define output ValueType schema
- [ ] Test with all 166 languages
- [ ] Benchmark vs direct Thread (target <2% overhead)
- [ ] Add error handling and timeout logic

**Deliverable**: ThreadParseFunction working, all languages supported

---

#### Days 8-9 (Wed-Thu) - Flow Builder (Programmatic Rust)

**Rust Flow Construction**:

```rust
// crates/thread-cocoindex/src/flows/analysis.rs
use cocoindex::{
    builder::flow_builder::FlowBuilder,
    base::spec::{FlowInstanceSpec, ImportOpSpec, ReactiveOpSpec, ExportOpSpec},
};

pub async fn build_thread_analysis_flow() -> Result<FlowInstanceSpec> {
    let mut builder = FlowBuilder::new("ThreadCodeAnalysis");

    // 1. SOURCE: Local file system
    let files = builder.add_source(
        "local_file",
        json!({
            "path": ".",
            "included_patterns": ["*.rs", "*.py", "*.ts", "*.go", "*.java"],
            "excluded_patterns": ["**/.*", "target", "node_modules", "dist"]
        }),
        SourceRefreshOptions::default(),
        ExecutionOptions::default(),
    )?;

    // 2. TRANSFORM: Parse with Thread
    let parsed = builder.transform(
        "thread_parse",
        json!({}),
        vec![files.field("content")?, files.field("language")?],
        "parsed"
    )?;

    // 3. COLLECT: Symbols
    let symbols_collector = builder.add_collector("symbols")?;
    builder.collect(
        symbols_collector,
        vec![
            ("file_path", files.field("path")?),
            ("name", parsed.field("symbols")?.field("name")?),
            ("kind", parsed.field("symbols")?.field("kind")?),
            ("signature", parsed.field("symbols")?.field("signature")?),
        ]
    )?;

    // 4. EXPORT: To Postgres
    builder.export(
        "symbols_table",
        "postgres",
        json!({
            "table": "code_symbols",
            "primary_key": ["file_path", "name"]
        }),
        symbols_collector,
        IndexOptions::default()
    )?;

    builder.build_flow()
}

// Register Thread operators
pub fn register_thread_operators() -> Result<()> {
    register_factory(
        "thread_parse",
        ExecutorFactory::SimpleFunction(Arc::new(ThreadParseFunction))
    )?;

    register_factory(
        "extract_symbols",
        ExecutorFactory::SimpleFunction(Arc::new(ExtractSymbolsFunction))
    )?;

    Ok(())
}
```

**Tasks**:
- [ ] Implement programmatic flow builder in Rust
- [ ] Register Thread operators in CocoIndex registry
- [ ] Build complete analysis flow (files → parse → extract → export)
- [ ] Test flow execution with LibContext
- [ ] Validate multi-target export (Postgres + Qdrant)
- [ ] Add error handling for flow construction

**Deliverable**: Full Rust flow working end-to-end

---

#### Day 10 (Friday) - Week 2 Integration Testing

**Tasks**:
- [ ] Test with real Thread codebase (self-analysis)
- [ ] Validate incremental updates (change 1 file, measure propagation)
- [ ] Performance benchmarks:
  - Initial index: 1000-file codebase
  - Incremental: 1, 10, 100 file changes
  - Memory usage
  - CPU utilization
- [ ] Compare vs pure Thread baseline
- [ ] Identify bottlenecks

**Deliverable**: Integration tests passing, benchmarks complete

---

### Week 3: Edge Deployment & Optimization (Jan 27-31)

**Goal**: Cloudflare edge deployment + performance optimization

#### Days 11-12 (Mon-Tue) - D1 Source/Target Implementation

**Tasks**:
- [ ] Implement D1 custom source:
  ```rust
  // Read code index from D1
  pub struct D1Source;

  impl SourceFactory for D1Source {
      async fn read(&self, ...) -> Result<BoxStream<Row>> {
          // Query D1 via HTTP API
          // Stream rows back to CocoIndex
      }
  }
  ```
- [ ] Implement D1 custom target:
  ```rust
  // Write analysis results to D1
  pub struct D1Target;

  impl TargetFactory for D1Target {
      async fn apply_mutation(&self, upserts, deletes) -> Result<()> {
          // Batch upsert to D1
          // Handle conflicts
      }
  }
  ```
- [ ] Test D1 integration locally (Wrangler dev)
- [ ] Deploy to Cloudflare staging

**Deliverable**: D1 integration working

---

#### Days 13-14 (Wed-Thu) - Serverless Container Deployment

**Cloudflare Architecture**:

```
┌───────────────────────────────────────────────────┐
│           Cloudflare Edge Network                 │
│                                                    │
│  ┌─────────────┐      ┌──────────────────────┐   │
│  │   Workers   │─────▶│  Serverless Container │   │
│  │  (API GW)   │      │   (CocoIndex Runtime) │   │
│  └──────┬──────┘      └──────────┬───────────┘   │
│         │                         │               │
│         │                         ▼               │
│         │              ┌──────────────────────┐   │
│         │              │   Durable Objects    │   │
│         │              │  (Flow Coordination) │   │
│         │              └──────────┬───────────┘   │
│         │                         │               │
│         ▼                         ▼               │
│  ┌─────────────────────────────────────────────┐ │
│  │              D1 Database                     │ │
│  │  (Code Index + Analysis Results)            │ │
│  └─────────────────────────────────────────────┘ │
└───────────────────────────────────────────────────┘
```

**Tasks**:
- [ ] Create Dockerfile for CocoIndex + thread-py
- [ ] Deploy to Cloudflare serverless containers
- [ ] Configure Workers → Container routing
- [ ] Test edge deployment:
  - Index code from GitHub webhook
  - Query analysis results via Worker API
  - Measure latency (target <100ms p95)
- [ ] Implement Durable Objects for flow coordination

**Deliverable**: Edge deployment working

---

#### Day 15 (Friday) - Performance Optimization

**Tasks**:
- [ ] Profile CPU/memory usage
- [ ] Optimize Rust ↔ Python bridge (minimize copies)
- [ ] Implement caching strategies:
  - Content-addressed parsing cache
  - Symbol extraction cache
  - Query result cache
- [ ] Batch operations for efficiency
- [ ] Validate CocoIndex's claimed 99% cost reduction
- [ ] Document performance characteristics

**Deliverable**: Optimized, production-ready pipeline

---

### Week 4: Production Readiness (Feb 3-7)

**Goal**: Documentation, testing, productionization

#### Days 16-17 (Mon-Tue) - Comprehensive Testing

**Test Suite**:

```python
# tests/test_thread_cocoindex.py
import pytest
import thread_py
import cocoindex

def test_thread_parse_all_languages():
    """Test ThreadParse with all 166 languages"""
    for lang in thread_py.supported_languages():
        result = thread_py.thread_parse(sample_code[lang], lang)
        assert "symbols" in result
        assert "imports" in result
        assert "calls" in result

def test_incremental_update_efficiency():
    """Validate 99%+ cost reduction claim"""
    # Index 1000 files
    initial_time = time_index(files)

    # Change 10 files
    change_files(files[:10])
    incremental_time = time_index(files)

    # Should be 50x+ faster
    assert incremental_time < initial_time / 50

def test_type_system_round_trip():
    """Ensure no metadata loss in Rust → Python → Rust"""
    doc = parse_rust_file("src/lib.rs")
    row = to_cocoindex_row(doc)
    doc2 = from_cocoindex_row(row)

    assert doc == doc2  # Exact equality

def test_edge_deployment_latency():
    """Validate <100ms p95 latency on edge"""
    latencies = []
    for _ in range(1000):
        start = time.time()
        query_edge_api("GET /symbols?file=src/lib.rs")
        latencies.append(time.time() - start)

    assert percentile(latencies, 95) < 0.1  # 100ms
```

**Tasks**:
- [ ] Unit tests for all transforms (100+ tests)
- [ ] Integration tests for full pipeline (50+ tests)
- [ ] Performance regression tests (benchmarks)
- [ ] Edge deployment tests (latency, throughput)
- [ ] Type safety tests (round-trip validation)
- [ ] Error handling tests (malformed code, network failures)
- [ ] Achieve 90%+ code coverage

**Deliverable**: Comprehensive test suite (95%+ passing)

---

#### Days 18-19 (Wed-Thu) - Documentation

**Documentation Suite**:

1. **Architecture Guide** (`PATH_B_ARCHITECTURE.md`)
   - Service-first design rationale
   - Dual-layer architecture diagram
   - Concurrency strategy (Rayon + tokio)
   - Data flow walkthrough

2. **API Reference** (`PATH_B_API_REFERENCE.md`)
   - `thread_py` module documentation
   - Custom transform API
   - D1 source/target API
   - Example flows

3. **Deployment Guide** (`PATH_B_DEPLOYMENT.md`)
   - Local development setup
   - Cloudflare edge deployment
   - D1 database setup
   - Monitoring and observability

4. **Performance Guide** (`PATH_B_PERFORMANCE.md`)
   - Benchmark methodology
   - Performance characteristics
   - Optimization strategies
   - Comparison vs Path A

**Tasks**:
- [ ] Write architecture documentation
- [ ] Generate API reference (Rust docs + Python docstrings)
- [ ] Create deployment runbooks
- [ ] Document edge cases and troubleshooting
- [ ] Add code examples for common use cases

**Deliverable**: Complete documentation suite

---

#### Day 20 (Friday) - Production Launch Checklist

**Pre-Production Validation**:

- [ ] **Code Quality**
  - [ ] All tests passing (95%+)
  - [ ] Code coverage > 90%
  - [ ] No critical lint warnings
  - [ ] Documentation complete

- [ ] **Performance**
  - [ ] Incremental updates 50x+ faster than full re-index
  - [ ] Edge latency p95 < 100ms
  - [ ] Memory usage < 500MB for 1000-file codebase
  - [ ] CPU utilization < 50% during indexing

- [ ] **Edge Deployment**
  - [ ] Serverless container deployed
  - [ ] D1 database provisioned
  - [ ] Workers routing configured
  - [ ] Durable Objects working

- [ ] **Monitoring**
  - [ ] Metrics collection (Prometheus/Grafana)
  - [ ] Error tracking (Sentry)
  - [ ] Log aggregation (Cloudflare Logs)
  - [ ] Alerting configured

**Deliverable**: Production-ready Path B implementation

---

## Rust-Native Integration Strategy

### Direct CocoIndex Library Usage

```rust
// Cargo.toml
[dependencies]
cocoindex = { git = "https://github.com/cocoindex-io/cocoindex", branch = "main" }
thread-ast-engine = { path = "../thread-ast-engine" }
thread-language = { path = "../thread-language" }
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"

// No PyO3, no Python runtime, pure Rust
```

### Operator Registration

```rust
// crates/thread-cocoindex/src/lib.rs
use cocoindex::ops::registry::register_factory;
use cocoindex::ops::interface::ExecutorFactory;

/// Register all Thread operators with CocoIndex
pub fn register_thread_operators() -> Result<()> {
    // Function operators
    register_factory(
        "thread_parse",
        ExecutorFactory::SimpleFunction(Arc::new(ThreadParseFunction))
    )?;

    register_factory(
        "extract_symbols",
        ExecutorFactory::SimpleFunction(Arc::new(ExtractSymbolsFunction))
    )?;

    register_factory(
        "extract_imports",
        ExecutorFactory::SimpleFunction(Arc::new(ExtractImportsFunction))
    )?;

    register_factory(
        "extract_calls",
        ExecutorFactory::SimpleFunction(Arc::new(ExtractCallsFunction))
    )?;

    // Source operators
    register_factory(
        "d1_source",
        ExecutorFactory::Source(Arc::new(D1SourceFactory))
    )?;

    // Target operators
    register_factory(
        "d1_target",
        ExecutorFactory::ExportTarget(Arc::new(D1TargetFactory))
    )?;

    Ok(())
}
```

### Performance Benefits (vs Python Bridge)

| Aspect | Python Bridge | Rust-Native | Improvement |
|--------|---------------|-------------|-------------|
| **Function Call Overhead** | ~1-5μs (PyO3) | ~0ns (inlined) | **∞** |
| **Data Serialization** | Rust → Python dict | Direct Value | **10-50x** |
| **Type Safety** | Runtime checks | Compile-time | **100%** |
| **Memory Usage** | Dual allocations | Single allocation | **2x** |
| **Debugging** | Python + Rust | Rust only | **Much easier** |
| **Deployment** | Python runtime + binary | Single binary | **Simpler** |

### Example Performance Comparison

```rust
// Python bridge approach (eliminated)
// ThreadParse: 100μs + 5μs PyO3 overhead = 105μs

// Rust-native approach
// ThreadParse: 100μs + 0μs overhead = 100μs
// 5% performance gain, cleaner code
```

---

## Edge Deployment Architecture

### Cloudflare Stack

**Workers** (API Gateway):
```javascript
// worker.js
export default {
  async fetch(request, env) {
    const url = new URL(request.url);

    // Route to serverless container
    if (url.pathname.startsWith('/api/analyze')) {
      return env.CONTAINER.fetch(request);
    }

    // Route to D1
    if (url.pathname.startsWith('/api/query')) {
      const { file_path } = await request.json();
      const result = await env.DB.prepare(
        'SELECT symbols, imports, calls FROM code_index WHERE file_path = ?'
      ).bind(file_path).first();

      return new Response(JSON.stringify(result));
    }
  }
}
```

**Serverless Container** (Pure Rust Binary):
```dockerfile
# Dockerfile
FROM rust:1.75 as builder
WORKDIR /app

# Copy workspace
COPY . .

# Build thread-cocoindex binary (includes CocoIndex + Thread)
RUN cargo build --release -p thread-cocoindex \
    --features cloudflare

# Runtime (minimal distroless image)
FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/thread-cocoindex /app/thread-cocoindex
EXPOSE 8080
CMD ["/app/thread-cocoindex"]
```

**D1 Database** (Edge-distributed SQL):
```sql
-- code_index table
CREATE TABLE code_index (
    file_path TEXT PRIMARY KEY,
    content_hash TEXT NOT NULL,
    language TEXT NOT NULL,
    symbols JSON NOT NULL,
    imports JSON NOT NULL,
    calls JSON NOT NULL,
    metadata JSON,
    indexed_at INTEGER NOT NULL,  -- Unix timestamp
    version INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX idx_language ON code_index(language);
CREATE INDEX idx_indexed_at ON code_index(indexed_at);

-- symbol_search table (for fast lookups)
CREATE TABLE symbol_search (
    symbol_name TEXT,
    symbol_kind TEXT,
    file_path TEXT,
    location TEXT,
    signature TEXT,
    PRIMARY KEY (symbol_name, file_path),
    FOREIGN KEY (file_path) REFERENCES code_index(file_path)
);

CREATE INDEX idx_symbol_name ON symbol_search(symbol_name);
CREATE INDEX idx_symbol_kind ON symbol_search(symbol_kind);
```

### Deployment Process

1. **Build** (Local):
   ```bash
   # Build Rust binary with CocoIndex integration
   cargo build --release -p thread-cocoindex --features cloudflare

   # Build container image
   docker build -t thread-cocoindex:latest .

   # Test locally
   docker run -p 8080:8080 thread-cocoindex:latest
   ```

2. **Deploy** (Cloudflare):
   ```bash
   # Push container to Cloudflare
   wrangler deploy --image thread-cocoindex:latest

   # Create D1 database
   wrangler d1 create code-index
   wrangler d1 execute code-index --file schema.sql

   # Deploy worker (API gateway)
   wrangler publish
   ```

3. **Monitor**:
   ```bash
   # Real-time logs
   wrangler tail

   # Metrics
   curl https://api.cloudflare.com/client/v4/accounts/{account_id}/analytics

   # Container health
   curl https://your-app.workers.dev/health
   ```

---

## Thread's Semantic Intelligence

### What CocoIndex Provides (Out of the Box)

✅ **Tree-sitter chunking** - Semantic code splitting
✅ **Content addressing** - Incremental updates
✅ **Multi-target storage** - Postgres, Qdrant, Neo4j
✅ **Dataflow orchestration** - Declarative pipelines

### What Thread Adds (Semantic Intelligence)

**1. Deep Symbol Extraction**

CocoIndex `SplitRecursively()` chunks code but doesn't extract:
- Function signatures with parameter types
- Class hierarchies and trait implementations
- Visibility modifiers (pub, private, protected)
- Generic type parameters
- Lifetime annotations (Rust)

Thread extracts **structured symbols**:
```json
{
  "name": "parse_document",
  "kind": "function",
  "visibility": "public",
  "signature": "pub fn parse_document<D: Document>(content: &str) -> Result<D>",
  "parameters": [
    {"name": "content", "type": "&str"}
  ],
  "return_type": "Result<D>",
  "generics": ["D: Document"],
  "location": {"line": 42, "column": 5}
}
```

**2. Import Dependency Graph**

CocoIndex doesn't track:
- Module import relationships
- Cross-file dependencies
- Circular dependency detection
- Unused import detection

Thread builds **dependency graph**:
```json
{
  "imports": [
    {
      "module": "thread_ast_engine",
      "items": ["parse", "Language"],
      "location": {"line": 1},
      "used": true
    }
  ],
  "dependency_graph": {
    "src/lib.rs": ["thread_ast_engine", "serde"],
    "src/parser.rs": ["src/lib.rs", "regex"]
  }
}
```

**3. Call Graph Analysis**

CocoIndex doesn't track:
- Function call relationships
- Method invocations
- Trait method resolution

Thread builds **call graph**:
```json
{
  "calls": [
    {
      "caller": "process_file",
      "callee": "parse_document",
      "callee_module": "thread_ast_engine",
      "location": {"line": 15},
      "call_type": "direct"
    },
    {
      "caller": "analyze_symbols",
      "callee": "extract_metadata",
      "call_type": "method",
      "receiver_type": "ParsedDocument"
    }
  ]
}
```

**4. Pattern Matching**

CocoIndex doesn't support:
- AST-based pattern queries
- Structural code search
- Meta-variable matching

Thread provides **ast-grep patterns**:
```rust
// Find all unwrap() calls (dangerous pattern)
pattern!("$EXPR.unwrap()")

// Find all async functions without error handling
pattern!("async fn $NAME($$$PARAMS) { $$$BODY }")
  .without(pattern!("Result"))
```

**5. Type Inference** (Language-dependent)

For typed languages (Rust, TypeScript, Go):
- Infer variable types from usage
- Resolve generic type parameters
- Track type constraints

---

## Success Criteria

### Quantitative Metrics

| Metric | Target | Priority |
|--------|--------|----------|
| **Incremental Update Speed** | 50x+ faster than full re-index | CRITICAL |
| **Edge Latency (p95)** | < 100ms for symbol lookup | HIGH |
| **Memory Usage** | < 500MB for 1000-file codebase | HIGH |
| **Test Coverage** | > 90% | HIGH |
| **Language Support** | All 166 Thread languages | MEDIUM |
| **Type Preservation** | 100% Value round-trip accuracy | CRITICAL |
| **Build Time** | < 3 minutes (release mode) | MEDIUM |
| **Zero Python Overhead** | Pure Rust, no PyO3 calls | CRITICAL |

### Qualitative Validation

✅ **Service-First Architecture** - Persistent, real-time, cached
✅ **Production Ready** - Deployed to Cloudflare edge
✅ **Developer Experience** - Clear API, good documentation
✅ **Semantic Intelligence** - Symbols/imports/calls extracted correctly
✅ **Edge Deployment** - Working serverless containers + D1

---

## Risk Mitigation

### Risk 1: CocoIndex Compilation Complexity

**Risk**: CocoIndex has complex build dependencies
**Mitigation**:
- Use CocoIndex as git dependency with locked revision
- Document build requirements clearly
- Cache compiled CocoIndex in CI
- Monitor build times

**Fallback**: Simplify by removing optional CocoIndex features

---

### Risk 2: D1 Limitations

**Risk**: D1 SQL limitations block complex queries
**Mitigation**:
- Test D1 capabilities early (Week 3 Days 11-12)
- Design schema to work within constraints
- Use Durable Objects for complex queries
- Fallback to Postgres for local development

**Fallback**: Postgres on Hyperdrive (Cloudflare's DB proxy)

---

### Risk 3: Edge Cold Start Latency

**Risk**: Serverless containers have >1s cold start
**Mitigation**:
- Use Durable Objects for warm state
- Implement aggressive caching
- Pre-warm containers on deployment
- Monitor cold start metrics

**Fallback**: Always-on container tier (higher cost)

---

### Risk 4: CocoIndex API Changes

**Risk**: CocoIndex updates break integration
**Mitigation**:
- Pin CocoIndex version in Cargo.toml
- Monitor CocoIndex releases
- Contribute to CocoIndex upstream
- Abstract CocoIndex behind interface

**Fallback**: Fork CocoIndex if needed

---

## Next Steps

### Immediate Actions (Week 1)

1. **Day 1**: Setup CocoIndex environment, run examples
2. **Day 2**: Study API analysis document, design transforms
3. **Day 3**: Design type system mapping
4. **Day 4**: Design D1 integration
5. **Day 5**: Review and finalize plan

### Success Checkpoints

- **Week 1 End**: Design approved, risks identified
- **Week 2 End**: ThreadParse + ExtractSymbols working
- **Week 3 End**: Edge deployment working
- **Week 4 End**: Production ready, documented

### Launch Criteria

Before declaring Path B "production ready":

- [ ] All 166 languages parsing correctly
- [ ] Incremental updates 50x+ faster
- [ ] Edge deployment working (<100ms p95)
- [ ] Test coverage >90%
- [ ] Documentation complete
- [ ] Monitoring configured

---

## Appendices

### Appendix A: API Analysis Reference

Full document: `/home/knitli/thread/COCOINDEX_API_ANALYSIS.md`

**Key Findings**:
- Python API: 30-40% of Rust API surface
- Rust API: Full access to internals
- PyO3 bridge: `Py<PyAny>` references, minimal Python state
- Extension pattern: Factory traits for custom operators

### Appendix B: CocoIndex Example Code

Reference implementation:
```python
# examples/codebase_analysis.py from CocoIndex docs
# Proves file watching, tree-sitter chunking, multi-target export
```

### Appendix C: Cloudflare Resources

- [Serverless Containers](https://developers.cloudflare.com/workers/runtime-apis/bindings/service-bindings/)
- [D1 Database](https://developers.cloudflare.com/d1/)
- [Durable Objects](https://developers.cloudflare.com/durable-objects/)
- [Workers Pricing](https://www.cloudflare.com/plans/developer-platform/)

---

## Summary: Why Rust-Native Path B

### Architectural Validation

**Service-First Requirements** → Path B is the only viable choice:
- ✅ Persistent storage built-in (Postgres/D1/Qdrant)
- ✅ Incremental updates via content-addressing
- ✅ Real-time intelligence with automatic dependency tracking
- ✅ Cloud/edge deployment with tokio async
- ✅ Data quality (freshness, lineage, observability)

**Rust-Native Integration** → Maximum performance and simplicity:
- ✅ Zero Python overhead (no PyO3, no Python runtime)
- ✅ Compile-time type safety (no runtime type errors)
- ✅ Direct CocoIndex API access (LibContext, FlowContext internals)
- ✅ Single binary deployment (simpler Docker, faster cold start)
- ✅ Better debugging (Rust compiler errors only)

### Implementation Strategy

**3 Weeks** (compressed from 4 via Rust-native simplification):
- **Week 1**: CocoIndex Rust API mastery + operator design
- **Week 2**: Implement Thread operators (Parse, ExtractSymbols, etc.)
- **Week 3**: Edge deployment + optimization + production readiness

**Core Components**:
```rust
thread-cocoindex/
├── src/
│   ├── lib.rs              # Operator registration
│   ├── functions/
│   │   ├── parse.rs        # ThreadParseFunction
│   │   ├── symbols.rs      # ExtractSymbolsFunction
│   │   ├── imports.rs      # ExtractImportsFunction
│   │   └── calls.rs        # ExtractCallsFunction
│   ├── sources/
│   │   └── d1.rs           # D1SourceFactory (custom)
│   ├── targets/
│   │   └── d1.rs           # D1TargetFactory (custom)
│   └── flows/
│       └── analysis.rs     # Programmatic flow builder
└── Cargo.toml              # cocoindex dependency
```

### Decision Confidence

**High Confidence** (98%+):
- API analysis confirms pure Rust approach is supported
- CocoIndex example proves feasibility
- Service-first requirements eliminate Path A
- Performance benefits clear (no PyO3 overhead)
- Simpler deployment (single binary)

**Remaining Validation** (Week 1):
- CocoIndex Rust API usability in practice
- Flow builder ergonomics for Rust
- D1 integration complexity

### Next Steps

1. **Approve this plan** - Team review and sign-off
2. **Day 1**: Clone CocoIndex, study Rust operator examples
3. **Day 2**: Design Thread operator traits
4. **Day 3**: Prototype value serialization
5. **Week 2**: Full implementation
6. **Week 3**: Edge deployment + production ready

---

**Document Version**: 2.0 (Rust-Native)
**Last Updated**: January 10, 2026
**Status**: Ready for Implementation
**Approval**: Pending team review
**Key Change**: Eliminated Python bridge, pure Rust integration
