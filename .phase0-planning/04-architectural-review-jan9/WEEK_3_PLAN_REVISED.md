# Week 3 Implementation Plan - REVISED FOR PURE RUST

**Date**: January 27, 2026
**Status**: READY TO START
**Context**: Pure Rust implementation (no Python bridge), vendored ReCoco with minimal features

---

## Overview

Week 3 focuses on **edge deployment** with Cloudflare Workers + D1, adapted for our pure Rust architecture.

**Key Changes from Original Plan**:
- ❌ No Python bridge to optimize (we removed Python)
- ❌ No `thread-py` module (pure Rust)
- ✅ Direct Rust WASM compilation for Workers
- ✅ D1 integration via HTTP API from Workers
- ✅ Focus on Rust → WASM → Edge deployment path

---

## Week 3 Goals

1. **D1 Integration** (Days 11-12): Design and implement D1 storage backend
2. **Edge Deployment** (Days 13-14): Deploy Thread analysis to Cloudflare Workers/D1
3. **Performance Validation** (Day 15): Benchmark and optimize edge execution

---

## Days 11-12 (Monday-Tuesday): D1 Integration Design & Implementation

### Goal
Design and implement D1 target factory for storing Thread analysis results on Cloudflare's edge database.

### Background: What is D1?

**Cloudflare D1** is a distributed SQLite database built for edge deployment:
- **Architecture**: SQLite at the edge with global replication
- **API**: HTTP-based SQL execution (Workers binding or REST API)
- **Limits**:
  - 10 GB per database
  - 100,000 rows read/query
  - 1,000 rows written/query
- **Latency**: <50ms p95 (edge-local reads)

### Architecture Decision: D1 Target Only (Not Source)

**Rationale**:
- **Primary use case**: Store analysis results for querying (target)
- **Source**: Local files via `local_file` source (CLI) or GitHub webhook (edge)
- **Simplification**: Defer D1 source until we need cross-repository analysis

### Tasks

#### Task 1: D1 Schema Design
**File**: `crates/flow/src/targets/d1_schema.sql`

Design schema for storing Thread analysis results:

```sql
-- Symbols table (primary analysis output)
CREATE TABLE code_symbols (
    file_path TEXT NOT NULL,
    name TEXT NOT NULL,
    kind TEXT NOT NULL,          -- function, class, variable, etc.
    scope TEXT,                   -- namespace/module scope
    line_start INTEGER,
    line_end INTEGER,
    content_hash TEXT NOT NULL,   -- For incremental updates
    indexed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (file_path, name)
);

-- Imports table
CREATE TABLE code_imports (
    file_path TEXT NOT NULL,
    symbol_name TEXT NOT NULL,
    source_path TEXT NOT NULL,
    kind TEXT,                    -- named, default, namespace
    content_hash TEXT NOT NULL,
    indexed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (file_path, symbol_name, source_path)
);

-- Function calls table
CREATE TABLE code_calls (
    file_path TEXT NOT NULL,
    function_name TEXT NOT NULL,
    arguments_count INTEGER,
    line_number INTEGER,
    content_hash TEXT NOT NULL,
    indexed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (file_path, function_name, line_number)
);

-- Metadata table (file tracking)
CREATE TABLE file_metadata (
    file_path TEXT PRIMARY KEY,
    content_hash TEXT NOT NULL,
    language TEXT NOT NULL,
    last_analyzed DATETIME DEFAULT CURRENT_TIMESTAMP,
    analysis_version INTEGER DEFAULT 1
);

-- Indexes for common queries
CREATE INDEX idx_symbols_kind ON code_symbols(kind);
CREATE INDEX idx_symbols_name ON code_symbols(name);
CREATE INDEX idx_imports_source ON code_imports(source_path);
CREATE INDEX idx_metadata_hash ON file_metadata(content_hash);
```

**Deliverable**: Schema design document and SQL file

---

#### Task 2: D1 HTTP API Research
**File**: `crates/flow/docs/D1_API_GUIDE.md`

Research Cloudflare D1 API for implementation:

**API Endpoints**:
```
POST /client/v4/accounts/{account_id}/d1/database/{database_id}/query
Authorization: Bearer {api_token}
Content-Type: application/json

{
  "sql": "INSERT INTO code_symbols (file_path, name, kind) VALUES (?, ?, ?)",
  "params": ["src/lib.rs", "main", "function"]
}
```

**Response Format**:
```json
{
  "result": [
    {
      "results": [...],
      "success": true,
      "meta": {
        "rows_read": 0,
        "rows_written": 1
      }
    }
  ]
}
```

**Research Topics**:
1. Batch insert limits (how many rows per request?)
2. Transaction support (can we batch upserts?)
3. Error handling (conflicts, constraint violations)
4. Rate limits (requests per second)
5. Workers binding vs REST API (which to use?)

**Deliverable**: API research document with examples

---

#### Task 3: D1 Target Factory Implementation
**File**: `crates/flow/src/targets/d1.rs`

Implement ReCoco target factory for D1:

```rust
use recoco::ops::factory_bases::TargetFactoryBase;
use recoco::base::value::Value;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct D1TargetSpec {
    pub account_id: String,
    pub database_id: String,
    pub api_token: String,
    pub table: String,
    pub primary_key: Vec<String>,
}

pub struct D1TargetFactory;

#[async_trait]
impl TargetFactoryBase for D1TargetFactory {
    type Spec = D1TargetSpec;
    type ResolvedArgs = D1ResolvedArgs;

    fn name(&self) -> &str { "d1" }

    async fn analyze<'a>(
        &'a self,
        spec: &'a Self::Spec,
        args_resolver: &mut OpArgsResolver<'a>,
        context: &FlowInstanceContext,
    ) -> Result<TargetAnalysisOutput<Self::ResolvedArgs>> {
        // Validate D1 connection
        // Build resolved args with connection info
        Ok(TargetAnalysisOutput {
            resolved_args: D1ResolvedArgs { /* ... */ },
        })
    }

    async fn build_executor(
        self: Arc<Self>,
        spec: Self::Spec,
        resolved_args: Self::ResolvedArgs,
        context: Arc<FlowInstanceContext>,
    ) -> Result<impl TargetExecutor> {
        Ok(D1TargetExecutor::new(spec, resolved_args))
    }
}

pub struct D1TargetExecutor {
    client: D1Client,
    table: String,
    primary_key: Vec<String>,
}

#[async_trait]
impl TargetExecutor for D1TargetExecutor {
    async fn apply_mutation(
        &self,
        upserts: Vec<Row>,
        deletes: Vec<Row>,
    ) -> Result<()> {
        // Batch upsert to D1 via HTTP API
        // Handle primary key conflicts (UPSERT)
        // Execute deletes
        Ok(())
    }
}
```

**Implementation Details**:
1. HTTP client for D1 API (use `reqwest`)
2. Batch operations (multiple rows per request)
3. UPSERT logic using SQLite `INSERT ... ON CONFLICT`
4. Error handling and retries
5. Content-addressed deduplication

**Deliverable**: Working D1 target factory

---

#### Task 4: Local Testing with Wrangler
**File**: `crates/flow/examples/d1_local_test.rs`

Test D1 integration locally using Wrangler dev:

```bash
# Install Wrangler CLI
npm install -g wrangler

# Create D1 database locally
wrangler d1 create thread-analysis-dev
wrangler d1 execute thread-analysis-dev --local --file=./crates/flow/src/targets/d1_schema.sql

# Test D1 target
cargo run --example d1_local_test
```

**Test Cases**:
1. Insert symbols from parsed Rust file
2. Query symbols by name
3. Update symbols (UPSERT on conflict)
4. Delete symbols by file_path
5. Verify content-hash deduplication

**Deliverable**: Local D1 integration tests passing

---

### Deliverables Summary (Days 11-12)

- ✅ D1 schema design (`d1_schema.sql`)
- ✅ D1 API research document (`D1_API_GUIDE.md`)
- ✅ D1 target factory implementation (`targets/d1.rs`)
- ✅ Local Wrangler tests (`examples/d1_local_test.rs`)
- ✅ All tests passing with local D1 database

---

## Days 13-14 (Wednesday-Thursday): Edge Deployment

### Goal
Deploy Thread analysis pipeline to Cloudflare Workers with D1 storage.

### Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│               Cloudflare Edge Network                   │
│                                                          │
│  ┌──────────────┐         ┌─────────────────────────┐  │
│  │   Worker     │────────▶│   Thread WASM Module    │  │
│  │  (HTTP API)  │         │  (Parse + Analysis)     │  │
│  └──────┬───────┘         └───────────┬─────────────┘  │
│         │                              │                │
│         │                              │                │
│         ▼                              ▼                │
│  ┌──────────────────────────────────────────────────┐  │
│  │              D1 Database                         │  │
│  │  Tables: code_symbols, code_imports, code_calls │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘

External Request:
POST /analyze
{
  "repo_url": "https://github.com/user/repo",
  "files": ["src/main.rs"]
}
```

### Tasks

#### Task 1: WASM Compilation for Workers
**File**: `crates/flow/worker/Cargo.toml`

Create Worker-compatible WASM build:

```toml
[package]
name = "thread-worker"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
thread-flow = { path = ".." }
wasm-bindgen = "0.2"
worker = "0.0.18"  # Cloudflare Workers SDK
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[profile.release]
opt-level = "z"     # Optimize for size
lto = true
codegen-units = 1
```

**WASM Entry Point**:
```rust
// crates/flow/worker/src/lib.rs
use worker::*;
use thread_flow::{ThreadFlowBuilder, ThreadOperators};

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Route: POST /analyze
    if req.path() == "/analyze" && req.method() == Method::Post {
        let body: AnalyzeRequest = req.json().await?;

        // Build flow with D1 target
        let flow = ThreadFlowBuilder::new("edge_analysis")
            .source_local(&body.files)
            .parse()
            .extract_symbols()
            .target_d1(
                env.var("D1_ACCOUNT_ID")?.to_string(),
                env.var("D1_DATABASE_ID")?.to_string(),
                env.secret("D1_API_TOKEN")?.to_string(),
                "code_symbols",
            )
            .build()
            .await?;

        // Execute flow
        flow.run().await?;

        Response::ok("Analysis complete")
    } else {
        Response::error("Not found", 404)
    }
}
```

**Build Command**:
```bash
wasm-pack build --target bundler --out-dir worker/pkg crates/flow/worker
```

**Deliverable**: WASM build pipeline for Workers

---

#### Task 2: Cloudflare Workers Deployment
**File**: `crates/flow/worker/wrangler.toml`

Configure Wrangler for deployment:

```toml
name = "thread-analysis-worker"
main = "worker/src/lib.rs"
compatibility_date = "2024-01-27"

[build]
command = "cargo install -q worker-build && worker-build --release"

[[d1_databases]]
binding = "DB"
database_name = "thread-analysis"
database_id = "your-database-id"

[env.production]
vars = { ENVIRONMENT = "production" }

[env.staging]
vars = { ENVIRONMENT = "staging" }
```

**Deployment Steps**:
```bash
# 1. Create production D1 database
wrangler d1 create thread-analysis-prod

# 2. Apply schema
wrangler d1 execute thread-analysis-prod --file=./crates/flow/src/targets/d1_schema.sql

# 3. Deploy to staging
wrangler deploy --env staging

# 4. Test staging endpoint
curl -X POST https://thread-analysis-worker.username.workers.dev/analyze \
  -H "Content-Type: application/json" \
  -d '{"files": ["test.rs"]}'

# 5. Deploy to production
wrangler deploy --env production
```

**Deliverable**: Worker deployed to staging

---

#### Task 3: Integration Testing
**File**: `crates/flow/tests/edge_integration.rs`

End-to-end tests for edge deployment:

```rust
#[tokio::test]
async fn test_edge_analysis_roundtrip() {
    // 1. Submit analysis request
    let response = reqwest::Client::new()
        .post("https://thread-worker.staging.workers.dev/analyze")
        .json(&AnalyzeRequest {
            files: vec!["src/lib.rs".to_string()],
            content: SAMPLE_RUST_CODE.to_string(),
        })
        .send()
        .await?;

    assert_eq!(response.status(), 200);

    // 2. Query D1 for results
    let symbols = query_d1_symbols("src/lib.rs").await?;
    assert!(symbols.len() > 0);

    // 3. Verify symbol accuracy
    assert_eq!(symbols[0].name, "main");
    assert_eq!(symbols[0].kind, "function");
}

#[tokio::test]
async fn test_edge_latency() {
    let mut latencies = vec![];

    for _ in 0..100 {
        let start = Instant::now();
        let _ = analyze_file("test.rs").await;
        latencies.push(start.elapsed());
    }

    let p95 = percentile(&latencies, 95);
    assert!(p95 < Duration::from_millis(100), "p95 latency too high: {:?}", p95);
}
```

**Test Scenarios**:
1. ✅ Successful analysis with symbol extraction
2. ✅ UPSERT on duplicate file analysis
3. ✅ Error handling (invalid syntax, unsupported language)
4. ✅ Latency validation (<100ms p95)
5. ✅ Content-hash deduplication

**Deliverable**: Integration tests passing against staging

---

### Deliverables Summary (Days 13-14)

- ✅ WASM build for Cloudflare Workers
- ✅ Worker deployed to staging environment
- ✅ Integration tests passing
- ✅ D1 schema applied to production database
- ✅ API endpoint operational

---

## Day 15 (Friday): Performance Optimization & Validation

### Goal
Profile, optimize, and validate performance characteristics of edge deployment.

### Tasks

#### Task 1: Performance Profiling
**File**: `crates/flow/benches/edge_performance.rs`

Benchmark edge execution:

```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_edge_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("edge_analysis");

    // Benchmark different file sizes
    for size in [100, 500, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, &size| {
                let code = generate_rust_code(size);
                b.iter(|| {
                    tokio::runtime::Runtime::new().unwrap().block_on(async {
                        analyze_on_edge(&code).await
                    })
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_edge_analysis);
criterion_main!(benches);
```

**Metrics to Measure**:
- Parse latency by language
- Symbol extraction time
- D1 write latency
- End-to-end request latency
- WASM memory usage
- Content-hash cache hit rate

**Deliverable**: Performance benchmark results

---

#### Task 2: Optimization Strategies

**A. WASM Size Optimization**
```toml
[profile.release]
opt-level = "z"           # Optimize for size
lto = "fat"              # Link-time optimization
codegen-units = 1        # Single compilation unit
strip = true             # Strip symbols
panic = "abort"          # Smaller panic handler
```

**B. Content-Addressed Caching**
```rust
// Skip re-analysis if content hash unchanged
async fn should_analyze(file_path: &str, content_hash: &str) -> bool {
    let existing = query_file_metadata(file_path).await?;
    existing.map_or(true, |meta| meta.content_hash != content_hash)
}
```

**C. Batch D1 Operations**
```rust
// Batch upserts (up to 1000 rows per request)
async fn batch_upsert_symbols(symbols: Vec<Symbol>) -> Result<()> {
    for chunk in symbols.chunks(1000) {
        let sql = build_batch_upsert(chunk);
        execute_d1_query(&sql).await?;
    }
    Ok(())
}
```

**Deliverable**: Optimized WASM build and caching strategies

---

#### Task 3: Performance Documentation
**File**: `crates/flow/docs/EDGE_PERFORMANCE.md`

Document performance characteristics:

```markdown
# Edge Performance Characteristics

## Latency Benchmarks (p95)

| Operation | Local | Edge (Cold Start) | Edge (Warm) |
|-----------|-------|-------------------|-------------|
| Parse (100 LOC) | 0.5ms | 15ms | 2ms |
| Parse (1000 LOC) | 3ms | 45ms | 8ms |
| Symbol Extract | 1ms | 5ms | 1ms |
| D1 Write (10 rows) | N/A | 25ms | 12ms |
| **End-to-End** | **5ms** | **85ms** | **25ms** |

## Cache Effectiveness

- Content-hash hit rate: 95%+ (on incremental updates)
- Speedup on cached files: 50x+
- D1 query cache: <5ms for repeat queries

## Cost Analysis

- WASM execution: $0.50 per million requests
- D1 storage: $0.75 per GB/month
- D1 reads: $1.00 per billion rows
- **Total cost**: <$5/month for 1M files analyzed
```

**Deliverable**: Performance documentation

---

### Deliverables Summary (Day 15)

- ✅ Performance benchmarks with metrics
- ✅ Optimized WASM build (<500KB)
- ✅ Content-addressed caching operational
- ✅ Performance documentation published
- ✅ Week 3 complete and validated

---

## Success Criteria

### Technical Validation
- [ ] D1 integration working (local + production)
- [ ] Worker deployed and operational
- [ ] Integration tests passing (>95%)
- [ ] p95 latency <100ms on edge
- [ ] WASM size <500KB
- [ ] Cache hit rate >90% on incremental updates

### Documentation
- [ ] D1 schema documented
- [ ] API guide for D1 integration
- [ ] Deployment runbook for Workers
- [ ] Performance benchmarks published

### Deployment
- [ ] Staging environment operational
- [ ] Production deployment ready
- [ ] Monitoring and alerting configured

---

## Risk Mitigation

### Risk 1: D1 API Limitations
**Mitigation**: Research limits early (Day 11), design schema within constraints

### Risk 2: WASM Size Bloat
**Mitigation**: Aggressive optimization flags, strip unused features from ReCoco

### Risk 3: Cold Start Latency
**Mitigation**: Keep Workers warm with health checks, optimize for fast initialization

### Risk 4: D1 Write Latency
**Mitigation**: Batch operations, async writes, accept eventual consistency

---

## Next Steps After Week 3

After completing Week 3, we'll have:
- ✅ Pure Rust implementation working locally and on edge
- ✅ D1 integration for persistent storage
- ✅ Cloudflare Workers deployment
- ✅ Performance validated

**Week 4 Preview**: Production readiness
- Comprehensive testing (unit + integration + edge)
- Documentation (architecture + API + deployment)
- Monitoring and observability
- Production deployment
