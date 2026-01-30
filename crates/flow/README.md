<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# thread-flow

[![Crate](https://img.shields.io/crates/v/thread-flow.svg)](https://crates.io/crates/thread-flow)
[![Documentation](https://docs.rs/thread-flow/badge.svg)](https://docs.rs/thread-flow)
[![License](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue.svg)](../../LICENSE)

Thread's dataflow integration for incremental code analysis, using [CocoIndex](https://github.com/cocoindex/cocoindex) for content-addressed caching and dependency tracking.

## Overview

`thread-flow` bridges Thread's imperative AST analysis engine with CocoIndex's declarative dataflow framework, enabling persistent incremental updates and multi-backend storage. It provides:

- ✅ **Content-Addressed Caching**: 50x+ performance gains via automatic incremental updates
- ✅ **Dependency Tracking**: File-level and symbol-level dependency graph management
- ✅ **Multi-Backend Storage**: Postgres (CLI), D1 (Edge), and in-memory (testing)
- ✅ **Dual Deployment**: Single codebase compiles to CLI (Rayon parallelism) and Edge (tokio async)
- ✅ **Language Extractors**: Built-in support for Rust, Python, TypeScript, and Go

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    thread-flow Crate                         │
├─────────────────────────────────────────────────────────────┤
│  Incremental System                                          │
│  ├─ Analyzer: Change detection & invalidation               │
│  ├─ Extractors: Language-specific dependency parsing        │
│  │  ├─ Rust: use declarations, pub use re-exports          │
│  │  ├─ Python: import/from...import statements             │
│  │  ├─ TypeScript: ES6 imports, CommonJS requires          │
│  │  └─ Go: import blocks, module path resolution           │
│  ├─ Graph: BFS traversal, topological sort, cycles         │
│  └─ Storage: Backend abstraction with factory pattern       │
│     ├─ Postgres: Connection pooling, prepared statements    │
│     ├─ D1: Cloudflare REST API, HTTP client                 │
│     └─ InMemory: Testing and development                    │
├─────────────────────────────────────────────────────────────┤
│  CocoIndex Integration                                       │
│  ├─ Bridge: Adapts Thread → CocoIndex operators            │
│  ├─ Flows: Declarative analysis pipeline builder           │
│  └─ Runtime: CLI (Rayon) vs Edge (tokio) strategies        │
└─────────────────────────────────────────────────────────────┘
```

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
thread-flow = { version = "0.1", features = ["postgres-backend", "parallel"] }
```

### Basic Usage

```rust
use thread_flow::incremental::{
    create_backend, BackendType, BackendConfig,
    IncrementalAnalyzer,
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create storage backend
    let backend = create_backend(
        BackendType::Postgres,
        BackendConfig::Postgres {
            database_url: std::env::var("DATABASE_URL")?,
        },
    ).await?;

    // Initialize analyzer
    let mut analyzer = IncrementalAnalyzer::new(backend);

    // Analyze changes
    let files = vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("src/lib.rs"),
    ];

    let result = analyzer.analyze_changes(&files).await?;

    println!("Changed: {} files", result.changed_files.len());
    println!("Affected: {} files", result.affected_files.len());
    println!("Cache hit rate: {:.1}%", result.cache_hit_rate * 100.0);
    println!("Analysis time: {}µs", result.analysis_time_us);

    Ok(())
}
```

### Dependency Extraction

```rust
use thread_flow::incremental::extractors::{RustDependencyExtractor, LanguageDetector};
use std::path::Path;

async fn extract_dependencies(file_path: &Path) -> Result<Vec<DependencyEdge>, Box<dyn std::error::Error>> {
    let source = tokio::fs::read_to_string(file_path).await?;

    // Detect language
    let detector = LanguageDetector::new();
    let lang = detector.detect_from_path(file_path)?;

    // Extract dependencies
    let extractor = RustDependencyExtractor::new();
    let edges = extractor.extract(file_path, &source)?;

    Ok(edges)
}
```

### Invalidation and Re-analysis

```rust
use thread_flow::incremental::IncrementalAnalyzer;
use std::path::PathBuf;

async fn handle_file_change(
    analyzer: &mut IncrementalAnalyzer,
    changed_file: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Invalidate dependents
    let affected = analyzer.invalidate_dependents(&[changed_file.clone()]).await?;

    println!("Invalidated {} dependent files", affected.len());

    // Re-analyze affected files
    let mut files_to_analyze = vec![changed_file];
    files_to_analyze.extend(affected);

    let result = analyzer.reanalyze_invalidated(&files_to_analyze).await?;

    println!("Re-analyzed {} files in {}µs",
        result.changed_files.len(),
        result.analysis_time_us
    );

    Ok(())
}
```

## Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `postgres-backend` | Postgres storage with connection pooling | ✅ |
| `d1-backend` | Cloudflare D1 backend for edge deployment | ❌ |
| `parallel` | Rayon-based parallelism (CLI only) | ✅ |
| `caching` | Query result caching with Moka | ❌ |
| `recoco-minimal` | Local file source for CocoIndex | ✅ |
| `recoco-postgres` | PostgreSQL target for CocoIndex | ✅ |
| `worker` | Edge deployment optimizations | ❌ |

### Feature Combinations

**CLI Deployment (recommended):**
```toml
thread-flow = { version = "0.1", features = ["postgres-backend", "parallel"] }
```

**Edge Deployment (Cloudflare Workers):**
```toml
thread-flow = { version = "0.1", features = ["d1-backend", "worker"] }
```

**Testing:**
```toml
[dev-dependencies]
thread-flow = "0.1"  # InMemory backend always available
```

## Deployment Modes

### CLI Deployment

Uses Postgres for persistent storage with Rayon for CPU-bound parallelism:

```rust
use thread_flow::incremental::{create_backend, BackendType, BackendConfig};

let backend = create_backend(
    BackendType::Postgres,
    BackendConfig::Postgres {
        database_url: "postgresql://localhost/thread".to_string(),
    },
).await?;

// Configure for CLI
// - Rayon parallel processing enabled via `parallel` feature
// - Connection pooling via deadpool-postgres
// - Batch operations for improved throughput
```

**Performance targets:**
- Storage latency: <10ms p95
- Cache hit rate: >90%
- Parallel speedup: 3-4x on quad-core

### Edge Deployment

Uses Cloudflare D1 for distributed storage with tokio async I/O:

```rust
use thread_flow::incremental::{create_backend, BackendType, BackendConfig};

let backend = create_backend(
    BackendType::D1,
    BackendConfig::D1 {
        account_id: std::env::var("CF_ACCOUNT_ID")?,
        database_id: std::env::var("CF_DATABASE_ID")?,
        api_token: std::env::var("CF_API_TOKEN")?,
    },
).await?;

// Configure for Edge
// - HTTP API client for D1 REST API
// - Async-first with tokio runtime
// - No filesystem access (worker feature)
```

**Performance targets:**
- Storage latency: <50ms p95
- Cache hit rate: >90%
- Horizontal scaling across edge locations

## API Documentation

Comprehensive API docs and integration guides:

- **Incremental System**: See [incremental module docs](https://docs.rs/thread-flow/latest/thread_flow/incremental/)
- **D1 Integration**: See [`docs/api/D1_INTEGRATION_API.md`](../../docs/api/D1_INTEGRATION_API.md)
- **CocoIndex Bridge**: See [bridge module docs](https://docs.rs/thread-flow/latest/thread_flow/bridge/)
- **Language Extractors**: See [extractors module docs](https://docs.rs/thread-flow/latest/thread_flow/incremental/extractors/)

## Examples

Run examples with:

```bash
# Observability instrumentation
cargo run --example observability_example

# D1 local testing (requires D1 emulator)
cargo run --example d1_local_test

# D1 integration testing (requires D1 credentials)
cargo run --example d1_integration_test --features d1-backend
```

## Testing

```bash
# Run all tests
cargo nextest run --all-features

# Run incremental system tests
cargo nextest run -p thread-flow --test incremental_integration_tests

# Run backend-specific tests
cargo nextest run -p thread-flow --test incremental_postgres_tests --features postgres-backend
cargo nextest run -p thread-flow --test incremental_d1_tests --features d1-backend

# Run performance regression tests
cargo nextest run -p thread-flow --test performance_regression_tests
```

## Benchmarking

```bash
# Fingerprint performance
cargo bench --bench fingerprint_benchmark

# D1 profiling (requires credentials)
cargo bench --bench d1_profiling --features d1-backend

# Load testing
cargo bench --bench load_test
```

## Performance Characteristics

### Incremental Updates

- **Fingerprint computation**: <5µs per file (Blake3)
- **Dependency extraction**: 1-10ms per file (language-dependent)
- **Graph traversal**: O(V+E) for BFS invalidation
- **Cache hit rate**: >90% typical, >95% ideal

### Storage Backends

| Backend | Read Latency (p95) | Write Latency (p95) | Throughput |
|---------|-------------------|---------------------|------------|
| InMemory | <1ms | <1ms | 10K+ ops/sec |
| Postgres | <10ms | <15ms | 1K+ ops/sec |
| D1 | <50ms | <100ms | 100+ ops/sec |

### Language Extractors

| Language | Parse Time (p95) | Complexity |
|----------|-----------------|------------|
| Rust | 2-5ms | High (macros, visibility) |
| TypeScript | 1-3ms | Medium (ESM + CJS) |
| Python | 1-2ms | Low (simple imports) |
| Go | 1-3ms | Medium (module resolution) |

## Contributing

### Development Setup

```bash
# Install development tools
mise install

# Run tests
cargo nextest run --all-features

# Run linting
cargo clippy --all-features

# Format code
cargo fmt
```

### Architecture Principles

1. **Service-Library Dual Architecture**: Features consider both library API design AND service deployment
2. **Test-First Development**: Tests → Approve → Fail → Implement (mandatory)
3. **Constitutional Compliance**: All changes must adhere to Thread Constitution v2.0.0

See [CLAUDE.md](../../CLAUDE.md) for complete development guidelines.

## License

AGPL-3.0-or-later

## Related Crates

- [`thread-ast-engine`](../ast-engine): Core AST parsing and pattern matching
- [`thread-language`](../language): Language definitions and tree-sitter parsers
- [`thread-services`](../services): High-level service interfaces
- [`recoco`](https://github.com/cocoindex/cocoindex): CocoIndex dataflow engine

---

**Status**: Production-ready (Phase 5 complete)
**Maintainer**: Knitli Inc. <knitli@knit.li>
**Contributors**: Claude Sonnet 4.5 <noreply@anthropic.com>
