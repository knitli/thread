# Thread Flow Troubleshooting Guide

Comprehensive troubleshooting guide for common issues, debugging strategies, and solutions across CLI and Edge deployments.

---

## Table of Contents

1. [Quick Diagnostics](#quick-diagnostics)
2. [Build and Compilation Issues](#build-and-compilation-issues)
3. [Runtime Errors](#runtime-errors)
4. [Database Connection Issues](#database-connection-issues)
5. [Performance Problems](#performance-problems)
6. [Configuration Issues](#configuration-issues)
7. [Edge Deployment Gotchas](#edge-deployment-gotchas)
8. [Debugging Strategies](#debugging-strategies)

---

## Quick Diagnostics

### Health Check Commands

```bash
# Verify Thread Flow installation
thread --version
# Expected: thread 0.1.0

# Check Rust toolchain
rustc --version
# Expected: rustc 1.75.0+ (edition 2024)

# Check cargo features
cargo tree --features | grep -E "(rayon|moka|recoco)"
# Expected: rayon, moka (if enabled), recoco

# Verify PostgreSQL connection (CLI)
psql -U thread_user -d thread_cache -c "SELECT 1;"
# Expected: 1 row returned

# Verify D1 connection (Edge)
wrangler d1 execute thread-production --command="SELECT 1;"
# Expected: 1 row returned
```

### Environment Validation

```bash
# Check environment variables
env | grep -E "(DATABASE_URL|RAYON|THREAD|RUST_LOG)"

# Verify feature flags
cargo build --features "recoco-postgres,parallel,caching" --dry-run 2>&1 | grep -i feature

# Test with minimal config
RUST_LOG=debug thread analyze --help
```

---

## Build and Compilation Issues

### Issue: "feature `recoco-postgres` not found"

**Symptom**:
```
error: Package `thread-flow v0.1.0` does not have feature `recoco-postgres`.
```

**Cause**: Typo or incorrect feature flag name

**Solution**:
```bash
# Check available features
cat crates/flow/Cargo.toml | grep -A 10 "\[features\]"

# Correct feature flags:
cargo build --features "recoco-postgres,parallel,caching"

# NOT: recoco_postgres, postgres, recoco-pg
```

---

### Issue: "cannot find crate `rayon`"

**Symptom**:
```
error[E0463]: can't find crate for `rayon`
  --> crates/flow/src/batch.rs:74:9
```

**Cause**: Parallel feature not enabled

**Solution**:
```bash
# Enable parallel feature
cargo build --features parallel

# Or make it default in Cargo.toml
[features]
default = ["recoco-minimal", "parallel"]
```

---

### Issue: WASM build fails with "filesystem not supported"

**Symptom**:
```
error: the wasm32-unknown-unknown target does not support filesystem operations
```

**Cause**: Trying to use filesystem APIs in WASM build

**Solution**:
```bash
# Ensure worker feature is set and parallel is DISABLED
cargo build \
  --target wasm32-unknown-unknown \
  --no-default-features \
  --features worker

# Verify in code:
#[cfg(not(target_arch = "wasm32"))]
use std::fs;  // Only for non-WASM targets
```

---

### Issue: "tree-sitter parser failed to compile"

**Symptom**:
```
error: failed to run custom build command for `tree-sitter-rust v0.21.0`
```

**Cause**: Missing C compiler or build tools

**Solution**:
```bash
# Linux: Install build-essential
sudo apt install build-essential

# macOS: Install Xcode command line tools
xcode-select --install

# Windows: Install Visual Studio Build Tools
# Download from: https://visualstudio.microsoft.com/downloads/

# Then rebuild
cargo clean
cargo build --release
```

---

## Runtime Errors

### Issue: "Connection refused" (PostgreSQL)

**Symptom**:
```
Error: Connection refused (os error 111)
Database URL: postgresql://thread_user@localhost:5432/thread_cache
```

**Diagnosis**:
```bash
# 1. Check if PostgreSQL is running
sudo systemctl status postgresql
# or: ps aux | grep postgres

# 2. Check if port 5432 is listening
sudo netstat -tlnp | grep 5432

# 3. Test connection manually
psql -U thread_user -h localhost -d thread_cache
```

**Solutions**:

**A. PostgreSQL not running**:
```bash
# Linux
sudo systemctl start postgresql
sudo systemctl enable postgresql

# macOS
brew services start postgresql@14
```

**B. Wrong port or host**:
```bash
# Check PostgreSQL config
sudo cat /etc/postgresql/14/main/postgresql.conf | grep port

# Update DATABASE_URL with correct port
export DATABASE_URL="postgresql://thread_user:pass@localhost:5433/thread_cache"
```

**C. Authentication failure**:
```bash
# Reset user password
sudo -u postgres psql
postgres=# ALTER USER thread_user WITH PASSWORD 'new_password';
postgres=# \q

# Update .env
DATABASE_URL="postgresql://thread_user:new_password@localhost:5432/thread_cache"
```

---

### Issue: "D1 API error: 401 Unauthorized"

**Symptom**:
```
Error: D1 API request failed with 401 Unauthorized
Account ID: abc123def456
Database ID: ghi789jkl012
```

**Diagnosis**:
```bash
# 1. Verify wrangler authentication
wrangler whoami

# 2. Check account ID matches
cat wrangler.toml | grep account_id

# 3. Test D1 access manually
wrangler d1 list
```

**Solutions**:

**A. Not authenticated**:
```bash
wrangler logout
wrangler login  # Re-authenticate with browser
```

**B. Wrong account ID**:
```bash
# Get correct account ID
wrangler whoami | grep "Account ID"

# Update wrangler.toml
account_id = "correct-account-id-here"
```

**C. Insufficient permissions**:
```bash
# Verify Workers Paid plan is active
open https://dash.cloudflare.com/your-account-id/workers/plans

# Ensure D1 is enabled
wrangler d1 list  # Should not error
```

---

### Issue: "Blake3 hash collision detected"

**Symptom**:
```
Warning: Potential hash collision detected
File 1: src/main.rs (hash: abc123...)
File 2: src/backup.rs (hash: abc123...)
```

**Cause**: Extremely unlikely (2^-256 probability) but theoretically possible

**Solution**:
```bash
# 1. Verify files are actually different
diff src/main.rs src/backup.rs

# 2. If identical, this is expected (deduplication working)
# If different, report as bug (extremely rare)

# 3. Temporary workaround: use file path as secondary key
# (Already implemented in Thread Flow)
```

---

### Issue: "Out of memory" (Edge deployment)

**Symptom**:
```
Error: Worker exceeded memory limit (128 MB)
Request aborted
```

**Diagnosis**:
```bash
# Check worker logs
wrangler tail --format json | jq '.outcome'

# Look for memory spikes in analytics
open https://dash.cloudflare.com/workers/analytics
```

**Solutions**:

**A. Large file analysis**:
```javascript
// Limit input size
export default {
  async fetch(request, env, ctx) {
    const { code } = await request.json();

    // Reject files >1MB
    if (code.length > 1_000_000) {
      return new Response('File too large (max 1MB)', { status: 413 });
    }

    // Process normally
    return analyzeCode(code);
  }
};
```

**B. Cache accumulation**:
```javascript
// Implement cache eviction
const MAX_CACHE_SIZE = 1000;
if (cache.size > MAX_CACHE_SIZE) {
  cache.clear();  // Simple eviction strategy
}
```

**C. Memory leak in WASM**:
```bash
# Rebuild WASM with leak detection
cargo build --target wasm32-unknown-unknown --release
wasm-opt -O3 --detect-features thread_flow_bg.wasm -o optimized.wasm
```

---

## Database Connection Issues

### Issue: "Too many connections" (PostgreSQL)

**Symptom**:
```
Error: FATAL: sorry, too many clients already
```

**Diagnosis**:
```sql
-- Check current connections
SELECT COUNT(*) FROM pg_stat_activity;

-- Check max connections
SHOW max_connections;
```

**Solutions**:

**A. Increase connection limit**:
```sql
-- Edit postgresql.conf
sudo vim /etc/postgresql/14/main/postgresql.conf

-- Increase max_connections
max_connections = 200  # Up from 100

-- Restart PostgreSQL
sudo systemctl restart postgresql
```

**B. Reduce pool size**:
```bash
# .env
DB_POOL_SIZE=10  # Down from 20
```

**C. Connection leak**:
```rust
// Ensure connections are properly closed
let result = {
    let conn = pool.get().await?;
    conn.query(...).await?
};  // Connection returned to pool here
```

---

### Issue: "D1 rate limit exceeded"

**Symptom**:
```
Error: D1 API rate limit exceeded (500 writes/minute)
Retry after: 60 seconds
```

**Diagnosis**:
```bash
# Check D1 usage
wrangler d1 info thread-production

# Monitor write rate
wrangler tail | grep "D1 write"
```

**Solutions**:

**A. Batch writes**:
```javascript
// Bad: Individual writes
for (const item of items) {
  await env.DB.prepare('INSERT INTO ...').bind(item).run();
}

// Good: Batched writes
const batch = items.map(item =>
  env.DB.prepare('INSERT INTO ...').bind(item)
);
await env.DB.batch(batch);  // Single API call
```

**B. Implement retry logic**:
```javascript
async function writeWithRetry(db, query, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await query.run();
    } catch (error) {
      if (error.message.includes('rate limit') && i < maxRetries - 1) {
        await sleep(2 ** i * 1000);  // Exponential backoff
        continue;
      }
      throw error;
    }
  }
}
```

**C. Upgrade plan** (if needed):
```bash
# Workers Paid includes:
# - 50M reads/month
# - 500K writes/month (500 writes/minute burst)

# For higher limits, contact Cloudflare Enterprise
```

---

## Performance Problems

### Issue: "Analysis taking >10 seconds for small codebase"

**Symptom**:
```
Analyzing 100 files...
Time: 15.2 seconds (expected: <1 second)
```

**Diagnosis**:
```bash
# Enable debug logging
RUST_LOG=thread_flow=debug thread analyze src/

# Look for:
# - "Cache hit: false" (cache not working)
# - "Rayon threads: 1" (parallel not enabled)
# - "Database query: 2.5s" (database slow)
```

**Solutions**:

**A. Cache not enabled**:
```bash
# Check feature flags
cargo tree --features | grep moka

# If missing, rebuild with caching
cargo build --release --features caching
```

**B. Parallel processing disabled**:
```bash
# Check feature flags
cargo tree --features | grep rayon

# If missing, rebuild with parallel
cargo build --release --features parallel

# Verify thread count
export RAYON_NUM_THREADS=4
```

**C. Database index missing**:
```sql
-- Check if indexes exist
SELECT indexname FROM pg_indexes WHERE tablename = 'code_symbols';

-- If missing, create them
CREATE INDEX CONCURRENTLY idx_symbols_hash ON code_symbols(content_hash);
```

---

### Issue: "Cache hit rate <50% (expected >90%)"

**Symptom**:
```
Cache statistics:
Hit rate: 42.3% (expected >90%)
Misses: 578 / 1000 lookups
```

**Diagnosis**:
```bash
# Check cache configuration
echo $THREAD_CACHE_MAX_CAPACITY
echo $THREAD_CACHE_TTL_SECONDS

# Check if fingerprinting is working
RUST_LOG=thread_flow::cache=trace thread analyze src/
```

**Solutions**:

**A. TTL too short**:
```bash
# Increase TTL
export THREAD_CACHE_TTL_SECONDS=3600  # 1 hour (up from 5 minutes)
```

**B. Capacity too small**:
```bash
# Increase capacity
export THREAD_CACHE_MAX_CAPACITY=100000  # 100k entries (up from 10k)
```

**C. Files changing frequently**:
```bash
# This is expected for rapid development
# Cache hit rate will be low during active editing
# Check hit rate during stable periods (e.g., CI/CD)
```

---

### Issue: "Worker CPU time exceeded (>50ms)"

**Symptom**:
```
Error: Worker exceeded CPU time limit
CPU time: 67ms (limit: 50ms)
```

**Diagnosis**:
```bash
# Check worker logs
wrangler tail | grep "CPU time"

# Identify slow operations
wrangler tail --format json | jq '.diagnostics.cpuTime'
```

**Solutions**:

**A. Offload to async**:
```javascript
// Break long operations into chunks
async function analyzeWithYield(code) {
  const lines = code.split('\n');
  const chunks = [];

  for (let i = 0; i < lines.length; i += 1000) {
    const chunk = lines.slice(i, i + 1000);
    chunks.push(analyzeChunk(chunk));

    // Yield between chunks
    await new Promise(resolve => setTimeout(resolve, 0));
  }

  return await Promise.all(chunks);
}
```

**B. Use cache aggressively**:
```javascript
// Check cache FIRST, avoid expensive parsing
const cached = await getFromCache(hash);
if (cached) {
  return cached;  // <1ms
}

// Only parse if absolutely necessary
return await parseAndCache(code);  // May hit CPU limit
```

**C. Limit input size**:
```javascript
// Reject large files
if (code.length > 50_000) {  // 50KB limit
  return new Response('File too large', { status: 413 });
}
```

---

## Configuration Issues

### Issue: "Environment variable not found"

**Symptom**:
```
Error: DATABASE_URL environment variable not set
```

**Diagnosis**:
```bash
# Check if .env exists
ls -la .env

# Check if loaded correctly
cat .env | grep DATABASE_URL

# Test environment
env | grep DATABASE_URL
```

**Solutions**:

**A. .env file missing**:
```bash
# Create .env
cat > .env << 'EOF'
DATABASE_URL=postgresql://thread_user:password@localhost:5432/thread_cache
RAYON_NUM_THREADS=4
RUST_LOG=thread_flow=info
EOF
```

**B. .env not loaded**:
```bash
# Load manually
export $(cat .env | xargs)

# Verify
echo $DATABASE_URL
```

**C. Systemd service not reading .env**:
```ini
# /etc/systemd/system/thread-analyzer.service
[Service]
EnvironmentFile=/etc/thread/config.env  # Correct path
```

---

### Issue: "Wrangler secrets not working"

**Symptom**:
```
Worker: env.THREAD_API_KEY is undefined
```

**Diagnosis**:
```bash
# List secrets
wrangler secret list

# Check worker binding
cat wrangler.toml | grep -A 5 "\[vars\]"
```

**Solutions**:

**A. Secret not created**:
```bash
# Create secret
wrangler secret put THREAD_API_KEY
# Enter value at prompt
```

**B. Wrong environment**:
```bash
# Secrets are environment-specific
wrangler secret put THREAD_API_KEY --env production
wrangler secret put THREAD_API_KEY --env development
```

**C. Accessing secret incorrectly**:
```javascript
// Wrong:
const key = process.env.THREAD_API_KEY;  // undefined

// Correct:
const key = env.THREAD_API_KEY;  // From worker env parameter
```

---

## Edge Deployment Gotchas

### Issue: "SharedArrayBuffer not supported"

**Symptom**:
```
Error: SharedArrayBuffer is not defined
This feature requires cross-origin isolation
```

**Cause**: Using multi-threaded WASM in non-isolated context

**Solution**:
```bash
# For Cloudflare Workers, do NOT use multi-threading
cargo build --target wasm32-unknown-unknown \
  --no-default-features \
  --features worker  # NO parallel feature

# Parallel processing is CLI-only
```

---

### Issue: "D1 database not found in worker"

**Symptom**:
```
Error: env.DB is undefined
Worker has no D1 binding
```

**Diagnosis**:
```bash
# Check wrangler.toml binding
cat wrangler.toml | grep -A 5 "d1_databases"
```

**Solution**:
```toml
# Ensure D1 binding exists in wrangler.toml
[[d1_databases]]
binding = "DB"  # Must match usage in worker
database_name = "thread-production"
database_id = "your-database-id-here"
```

---

### Issue: "WASM module failed to instantiate"

**Symptom**:
```
Error: WebAssembly.instantiate(): Compiling function #42 failed
```

**Diagnosis**:
```bash
# Validate WASM module
wasm-validate worker/thread_flow_bg.wasm

# Check WASM features
wasm-objdump -x worker/thread_flow_bg.wasm | grep -i import
```

**Solutions**:

**A. Invalid WASM build**:
```bash
# Rebuild from scratch
cargo clean
cargo run -p xtask build-wasm --release
```

**B. Unsupported WASM features**:
```bash
# Check for forbidden features (threads, SIMD)
wasm-objdump -x worker/thread_flow_bg.wasm | grep -E "(thread|atomic|simd)"

# If found, disable in Cargo.toml
[target.wasm32-unknown-unknown]
# Remove: target-feature = "+atomics,+bulk-memory"
```

**C. Corrupted WASM file**:
```bash
# Verify file integrity
md5sum worker/thread_flow_bg.wasm

# Re-upload to worker
wrangler deploy --no-bundle
```

---

## Debugging Strategies

### Enable Debug Logging

```bash
# Maximum logging
export RUST_LOG=trace

# Module-specific logging
export RUST_LOG=thread_flow=debug,thread_services=info

# Filter by log level
export RUST_LOG=thread_flow=debug,warn

# Run with logging
thread analyze src/
```

### Use GDB/LLDB for Crashes

```bash
# Build with debug symbols
cargo build --features parallel

# Run under debugger
gdb --args ./target/debug/thread analyze src/

# On crash, get backtrace
(gdb) run
(gdb) backtrace
```

### Profile Performance

```bash
# CPU profiling
perf record --call-graph=dwarf thread analyze large-codebase/
perf report

# Memory profiling
valgrind --tool=massif thread analyze src/
ms_print massif.out.*
```

### Inspect Database State

```sql
-- PostgreSQL
SELECT * FROM code_symbols WHERE content_hash = 'abc123...' \gx

-- D1
wrangler d1 execute thread-production \
  --command="SELECT * FROM code_symbols LIMIT 10;"
```

### Examine WASM Module

```bash
# Disassemble WASM
wasm-objdump -d worker/thread_flow_bg.wasm > disassembly.txt

# View exports
wasm-objdump -x worker/thread_flow_bg.wasm | grep Export

# Analyze size
wasm-opt --print-stats worker/thread_flow_bg.wasm
```

---

## Common Error Messages Reference

| Error Message | Likely Cause | Quick Fix |
|---------------|--------------|-----------|
| "Connection refused" | PostgreSQL not running | `systemctl start postgresql` |
| "401 Unauthorized" | D1 authentication failure | `wrangler login` |
| "feature not found" | Wrong feature flag | Check `Cargo.toml` [features] |
| "Too many connections" | PostgreSQL pool exhausted | Reduce `DB_POOL_SIZE` |
| "Rate limit exceeded" | D1 write limit hit | Implement batching |
| "CPU time exceeded" | Worker timeout | Add async yields, use cache |
| "Memory limit exceeded" | Worker OOM | Limit input size, evict cache |
| "Hash collision" | Blake3 collision (rare) | Report as bug |
| "WASM instantiation failed" | Invalid WASM build | Rebuild with `xtask` |
| "SharedArrayBuffer not defined" | Multi-threading in worker | Disable `parallel` feature |

---

## Getting Help

### Self-Service Resources

1. **Documentation**: `docs/` directory
   - Architecture: `docs/architecture/THREAD_FLOW_ARCHITECTURE.md`
   - API Reference: `docs/api/D1_INTEGRATION_API.md`
   - Deployment: `docs/deployment/`

2. **Examples**: `crates/flow/examples/`
   - D1 integration: `examples/d1_local_test/`
   - Query caching: `examples/query_cache_example/`

3. **Tests**: `crates/flow/tests/`
   - Integration tests: `tests/integration_tests.rs`
   - D1 target tests: `tests/d1_target_tests.rs`

### Reporting Issues

When reporting issues, include:

```bash
# System information
uname -a
rustc --version
cargo --version

# Thread Flow version
thread --version

# Environment
env | grep -E "(DATABASE_URL|RAYON|THREAD|RUST_LOG)"

# Error logs
RUST_LOG=debug thread analyze src/ 2>&1 | tee error.log

# Database state (CLI)
psql -U thread_user -d thread_cache -c "\d code_symbols"

# Worker logs (Edge)
wrangler tail --format json > worker_logs.json
```

---

## Troubleshooting Checklist

### Before Deployment

- [ ] Rust 1.75+ installed (`rustc --version`)
- [ ] Correct feature flags enabled (check `cargo tree --features`)
- [ ] Environment variables configured (`.env` exists and loaded)
- [ ] Database connection successful (PostgreSQL or D1)
- [ ] Health checks passing (`thread --version`, `thread db-check`)

### After Deployment

- [ ] Logs showing normal operation (`RUST_LOG=info`)
- [ ] Cache hit rate >90% after warm-up
- [ ] Query latency <10ms (CLI), <50ms (Edge)
- [ ] No error spikes in metrics
- [ ] CPU/memory usage within limits

### When Issues Occur

- [ ] Check logs first (`RUST_LOG=debug`)
- [ ] Verify environment variables
- [ ] Test database connection manually
- [ ] Review recent configuration changes
- [ ] Check for resource limits (connections, memory, CPU)
- [ ] Consult error message reference table
- [ ] Try minimal reproduction case

---

**Common Issue Resolution Time**:
- Configuration errors: <5 minutes
- Database connection: 5-15 minutes
- Performance tuning: 30-60 minutes
- WASM build issues: 15-30 minutes
- Edge deployment: 10-20 minutes
