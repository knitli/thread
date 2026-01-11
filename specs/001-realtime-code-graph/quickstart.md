# Quickstart Guide: Real-Time Code Graph Intelligence

**Feature**: Real-Time Code Graph Intelligence  
**Status**: Development  
**Target Audience**: Developers using Thread for code analysis

## Overview

Thread's Real-Time Code Graph Intelligence provides:
- **Real-time dependency tracking** for codebases up to 500k files
- **Conflict prediction** before code merge (95% accuracy, <10% false positives)
- **Incremental analysis** (<10% of full scan time for typical changes)
- **Dual deployment** (CLI for local development, Edge for global scale)

## Installation

### CLI Deployment (Local Development)

**Prerequisites**:
- Rust 1.75+ (edition 2021)
- Postgres 14+ (for persistent caching)
- 8GB RAM minimum (16GB recommended for large codebases)

**Install via cargo**:
```bash
cargo install thread-cli --features graph-intelligence
```

**Or build from source**:
```bash
git clone https://github.com/thread/thread.git
cd thread
cargo build --release --workspace
./target/release/thread --version
```

### Edge Deployment (Cloudflare Workers)

**Prerequisites**:
- Cloudflare Workers account (paid plan for 10MB WASM limit)
- Wrangler CLI installed (`npm install -g wrangler`)

**Deploy to Cloudflare**:
```bash
# Build WASM binary
mise run build-wasm-release

# Deploy to Cloudflare Workers
wrangler publish

# View deployment
wrangler tail
```

**Environment Variables**:
```toml
# wrangler.toml
name = "thread-intelligence"
main = "build/worker/shim.mjs"

[env.production]
vars = { D1_DATABASE = "thread-production" }
```

## Quick Start (CLI)

### 1. Initialize a Repository

```bash
# Initialize Thread for your codebase
thread init --repository /path/to/your/code --language rust

# Or for multiple languages
thread init --repository /path/to/your/code --languages rust,typescript,python
```

**Output**:
```
✓ Initialized Thread repository: repo:abc123
✓ Detected 1,234 files (Rust: 800, TypeScript: 300, Python: 134)
✓ Created Postgres database: thread_repo_abc123
```

### 2. Run Initial Analysis

```bash
# Full analysis (first time)
thread analyze --repository repo:abc123

# Watch for progress
thread status --session <session_id>
```

**Expected Time**:
- Small (<1k files): 10-30 seconds
- Medium (1k-10k files): 1-5 minutes
- Large (10k-100k files): 5-30 minutes

**Output**:
```
Analyzing repository repo:abc123...
[=============>             ] 54% (670/1234 files)
Nodes created: 8,450
Edges created: 32,100
Cache hit rate: 0% (first run)
```

### 3. Query the Graph

#### Find Dependencies

```bash
# Who calls this function?
thread query --node "processPayment" --query-type callers

# What does this function call?
thread query --node "processPayment" --query-type callees

# Full dependency tree (2 hops)
thread query --node "processPayment" --query-type dependencies --depth 2
```

**Sample Output**:
```json
{
  "nodes": [
    {"id": "node:abc123", "name": "validatePayment", "type": "FUNCTION"},
    {"id": "node:def456", "name": "checkoutFlow", "type": "FUNCTION"}
  ],
  "edges": [
    {"source": "node:def456", "target": "node:abc123", "type": "CALLS"}
  ],
  "query_time_ms": 15,
  "cache_hit": true
}
```

#### Semantic Search

```bash
# Find similar functions
thread search --code "fn validate_input(user: &User) -> Result<(), Error>" --top-k 5
```

**Output**:
```
Top 5 similar functions:
1. [0.92] validateUser (src/auth.rs:45)
2. [0.87] checkUserPermissions (src/permissions.rs:102)
3. [0.81] verifyInput (src/validation.rs:67)
4. [0.76] authenticateUser (src/auth.rs:120)
5. [0.72] validateRequest (src/api.rs:88)
```

### 4. Conflict Detection

#### Manual Conflict Check

```bash
# Compare local changes against main branch
thread conflicts --compare main --files src/payment.rs,src/checkout.rs

# Multi-tier detection (all tiers)
thread conflicts --compare main --files src/payment.rs --tiers 1,2,3
```

**Progressive Output**:
```
Tier 1 (AST Diff) - 95ms:
  ⚠ Potential conflict: Function signature changed
     Confidence: 0.6

Tier 2 (Semantic) - 850ms:
  🔴 Breaking change detected: 15 callers affected
     Confidence: 0.9
     Suggestion: Update all call sites to new signature

Tier 3 (Graph Impact) - 4.2s:
  🚨 CRITICAL: Checkout flow broken (critical path)
     Confidence: 0.95
     Suggestion: Refactor in 3 steps:
       1. Add adapter layer for backward compatibility
       2. Migrate callers incrementally
       3. Remove old API after migration complete
```

#### Real-Time Monitoring

```bash
# Subscribe to real-time conflict updates
thread watch --repository repo:abc123
```

**Real-Time Feed**:
```
[12:00:05] Code change detected: src/payment.rs
[12:00:05] Conflict detected (Tier 1): SignatureChange (confidence: 0.6)
[12:00:06] Conflict updated (Tier 2): BreakingAPIChange (confidence: 0.9)
[12:00:10] Conflict updated (Tier 3): Critical - checkout flow broken (confidence: 0.95)
```

## Configuration

### Database Setup (Postgres)

```bash
# Create database
createdb thread_graph

# Set connection URL
export DATABASE_URL="postgresql://localhost/thread_graph"

# Run migrations
thread migrate up
```

### Performance Tuning

**File**: `thread.toml` (auto-created by `thread init`)

```toml
[analysis]
max_file_size_mb = 10          # Skip files larger than 10MB
timeout_seconds = 30           # Timeout per-file analysis
parallel_workers = 8           # CPU parallelism (rayon)

[cache]
postgres_url = "postgresql://localhost/thread_graph"
cache_ttl_hours = 24           # Cache expiration
max_cache_size_gb = 10         # Max cache storage

[conflict_detection]
enabled_tiers = [1, 2, 3]      # All tiers enabled
tier1_timeout_ms = 100         # AST diff timeout
tier2_timeout_ms = 1000        # Semantic timeout
tier3_timeout_ms = 5000        # Graph impact timeout
```

## Common Workflows

### Workflow 1: Pre-Commit Conflict Check

```bash
# Check for conflicts before committing
git diff --name-only | xargs thread conflicts --compare HEAD

# If conflicts detected, review and fix
thread query --node <conflicting_symbol> --query-type reverse-dependencies
```

### Workflow 2: Incremental Updates

```bash
# After editing files
thread analyze --incremental --files src/payment.rs,src/checkout.rs

# Verify cache hit rate improved
thread metrics --session <session_id>
# Expected: cache_hit_rate > 0.9 (90%+)
```

### Workflow 3: CI/CD Integration

```yaml
# .github/workflows/thread-analysis.yml
name: Thread Analysis

on: [pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Thread
        run: cargo install thread-cli --features graph-intelligence
      - name: Run Conflict Detection
        run: |
          thread init --repository . --language rust
          thread conflicts --compare ${{ github.base_ref }} --files $(git diff --name-only ${{ github.base_ref }})
```

## Troubleshooting

### Issue: Slow Analysis (>5 minutes for 10k files)

**Diagnosis**:
```bash
thread metrics --session <session_id> --verbose
```

**Solutions**:
- Increase `parallel_workers` in `thread.toml`
- Check Postgres connection (should be <10ms p95 latency)
- Verify cache hit rate (>90% expected after first run)

### Issue: High Memory Usage

**Diagnosis**:
```bash
# Monitor memory during analysis
thread analyze --repository repo:abc123 --profile-memory
```

**Solutions**:
- Reduce `parallel_workers` (trade speed for memory)
- Increase `max_file_size_mb` to skip large files
- Use incremental analysis instead of full scans

### Issue: WebSocket Disconnections

**Diagnosis**:
```bash
thread watch --repository repo:abc123 --debug
```

**Solutions**:
- Check network stability (WebSocket requires persistent connection)
- Enable SSE fallback: `thread watch --fallback sse`
- Enable polling fallback: `thread watch --fallback polling`

## Next Steps

1. **Read the Data Model**: `specs/001-realtime-code-graph/data-model.md`
2. **Explore API Contracts**: `specs/001-realtime-code-graph/contracts/`
3. **Review Implementation Plan**: `specs/001-realtime-code-graph/plan.md`
4. **Check Task Breakdown**: `specs/001-realtime-code-graph/tasks.md` (generated by `/speckit.tasks`)

## Support

- **Documentation**: https://thread.dev/docs/real-time-intelligence
- **GitHub Issues**: https://github.com/thread/thread/issues
- **Community Discord**: https://discord.gg/thread

---

**Status**: This feature is under active development. Refer to `specs/001-realtime-code-graph/spec.md` for the complete specification.
