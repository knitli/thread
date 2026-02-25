<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.

SPDX-License-Identifier: AGPL-3.0-or-later
-->

# Thread Flow Edge Deployment Guide

Comprehensive guide for deploying Thread Flow to Cloudflare Workers with D1 distributed database backend.

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Cloudflare Account Setup](#cloudflare-account-setup)
3. [D1 Database Initialization](#d1-database-initialization)
4. [Wrangler Configuration](#wrangler-configuration)
5. [WASM Build Process](#wasm-build-process)
6. [Edge Deployment](#edge-deployment)
7. [Environment Secrets Management](#environment-secrets-management)
8. [Verification](#verification)
9. [Next Steps](#next-steps)

---

## Prerequisites

### System Requirements

- **Node.js**: 18.0.0 or later (for wrangler CLI)
- **Rust**: 1.75.0 or later with wasm32 target
- **wasm-pack**: WebAssembly build tool
- **Cloudflare Account**: With Workers and D1 enabled

### Install Required Tools

```bash
# Node.js (if not installed)
# Ubuntu/Debian
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# macOS
brew install node@18

# Rust WASM target
rustup target add wasm32-unknown-unknown

# wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Wrangler CLI (Cloudflare Workers CLI)
npm install -g wrangler

# Verify installations
node --version      # Should be 18+
wrangler --version  # Should be 3.0+
rustc --version     # Should be 1.75+
wasm-pack --version # Should be 0.12+
```

### Cloudflare Account Requirements

- **Workers Paid Plan** (required for D1)
  - $5/month minimum
  - Includes 10M requests/month
  - D1 database access

- **D1 Database** (included in Workers Paid)
  - Unlimited databases
  - 10GB storage
  - 50M reads/month
  - 500K writes/month

---

## Cloudflare Account Setup

### 1. Create Cloudflare Account

```bash
# Sign up at https://dash.cloudflare.com/sign-up

# Authenticate wrangler
wrangler login

# This opens browser for OAuth authentication
# Grant wrangler access to your account
```

### 2. Verify Authentication

```bash
# Check account details
wrangler whoami

# Expected output:
# ┌───────────────────┬──────────────────────────────────┐
# │ Account Name      │ Your Account Name                │
# ├───────────────────┼──────────────────────────────────┤
# │ Account ID        │ abc123def456...                  │
# ├───────────────────┼──────────────────────────────────┤
# │ Email             │ you@example.com                  │
# └───────────────────┴──────────────────────────────────┘
```

### 3. Upgrade to Workers Paid Plan

```bash
# Navigate to Workers dashboard
# https://dash.cloudflare.com/your-account-id/workers/plans

# Select "Workers Paid" plan ($5/month)
# Confirm payment method
```

---

## D1 Database Initialization

### 1. Create D1 Database

```bash
# Create production database
wrangler d1 create thread-production

# Expected output:
# ✅ Successfully created DB 'thread-production' in region WNAM
#
# [[d1_databases]]
# binding = "DB"
# database_name = "thread-production"
# database_id = "abc123-def456-ghi789-jkl012"

# Save the database_id - you'll need it for wrangler.toml
```

### 2. Initialize Database Schema

Thread Flow automatically creates tables on first use, but you can pre-initialize:

```bash
# Create schema file
cat > schema.sql << 'EOF'
-- Content-addressed symbol cache
CREATE TABLE IF NOT EXISTS code_symbols (
    content_hash TEXT PRIMARY KEY,
    file_path TEXT NOT NULL,
    language TEXT,
    symbols TEXT,  -- JSON-encoded symbol data
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER DEFAULT (strftime('%s', 'now'))
);

-- Indexes for fast lookups
CREATE INDEX IF NOT EXISTS idx_symbols_file_path ON code_symbols(file_path);
CREATE INDEX IF NOT EXISTS idx_symbols_language ON code_symbols(language);
CREATE INDEX IF NOT EXISTS idx_symbols_created ON code_symbols(created_at);
EOF

# Execute schema
wrangler d1 execute thread-production --file=schema.sql

# Expected output:
# 🌀 Mapping SQL input into an array of statements
# 🌀 Parsing 4 statements
# 🌀 Executing on thread-production (abc123-def456-ghi789-jkl012):
# ✅ Successfully executed 4 commands
```

### 3. Verify Database

```bash
# Query database info
wrangler d1 info thread-production

# Expected output:
# Database: thread-production
# UUID: abc123-def456-ghi789-jkl012
# Version: 1
# Created: 2025-01-28T12:00:00Z

# List tables
wrangler d1 execute thread-production --command="SELECT name FROM sqlite_master WHERE type='table';"

# Expected output:
# ┌──────────────┐
# │ name         │
# ├──────────────┤
# │ code_symbols │
# └──────────────┘
```

### 4. Create Development Database (Optional)

```bash
# Create separate database for development/testing
wrangler d1 create thread-development

# Use --local flag for local D1 testing
wrangler d1 execute thread-development --local --file=schema.sql
```

---

## Wrangler Configuration

### 1. Create `wrangler.toml`

```bash
# Navigate to your worker directory
cd crates/flow

# Create wrangler.toml
cat > wrangler.toml << 'EOF'
name = "thread-flow-worker"
main = "worker/index.js"
compatibility_date = "2024-01-01"

# Account and workers configuration
account_id = "your-account-id"  # From 'wrangler whoami'
workers_dev = true

# D1 Database binding
[[d1_databases]]
binding = "DB"
database_name = "thread-production"
database_id = "your-database-id"  # From 'wrangler d1 create'

# Environment variables (non-sensitive)
[vars]
ENVIRONMENT = "production"
LOG_LEVEL = "info"

# Resource limits
[limits]
cpu_ms = 50  # 50ms CPU time per request (D1 queries are fast)

# Build configuration
[build]
command = "cargo run -p xtask build-wasm --release"

[build.upload]
format = "modules"
dir = "worker"
main = "./index.js"

# Routes (customize for your domain)
routes = [
  { pattern = "api.yourdomain.com/thread/*", zone_name = "yourdomain.com" }
]
EOF
```

### 2. Configure for Multiple Environments

```bash
# Production environment (in wrangler.toml)
cat >> wrangler.toml << 'EOF'

# Development environment
[env.development]
name = "thread-flow-worker-dev"
vars = { ENVIRONMENT = "development", LOG_LEVEL = "debug" }

[[env.development.d1_databases]]
binding = "DB"
database_name = "thread-development"
database_id = "dev-database-id"

# Staging environment
[env.staging]
name = "thread-flow-worker-staging"
vars = { ENVIRONMENT = "staging", LOG_LEVEL = "info" }

[[env.staging.d1_databases]]
binding = "DB"
database_name = "thread-staging"
database_id = "staging-database-id"
EOF
```

### 3. Worker Entry Point

Create `worker/index.js`:

```javascript
import init, { analyze_code } from './thread_flow_bg.wasm';

export default {
  async fetch(request, env, ctx) {
    // Initialize WASM module
    await init();

    // Extract request data
    const { code, language } = await request.json();

    try {
      // Run Thread Flow analysis
      const symbols = analyze_code(code, language);

      // Cache in D1
      const contentHash = computeHash(code);
      await env.DB.prepare(
        'INSERT OR REPLACE INTO code_symbols (content_hash, symbols) VALUES (?, ?)'
      ).bind(contentHash, JSON.stringify(symbols)).run();

      return new Response(JSON.stringify(symbols), {
        headers: { 'Content-Type': 'application/json' }
      });
    } catch (error) {
      return new Response(JSON.stringify({ error: error.message }), {
        status: 500,
        headers: { 'Content-Type': 'application/json' }
      });
    }
  }
};

function computeHash(content) {
  // Simple hash for demo - use crypto API in production
  return btoa(content).substring(0, 32);
}
```

---

## WASM Build Process

### 1. Build WASM Module

```bash
# Navigate to Thread Flow directory
cd crates/flow

# Build WASM for edge deployment (no parallel, no filesystem)
cargo run -p xtask build-wasm --release

# Expected output:
# Building WASM module for Cloudflare Workers...
# Features: worker (no parallel, no filesystem)
# Target: wasm32-unknown-unknown
# Optimizing with wasm-opt...
# ✅ WASM build complete: worker/thread_flow_bg.wasm (2.1 MB)
```

### 2. Verify WASM Build

```bash
# Check WASM file size
ls -lh worker/thread_flow_bg.wasm

# Expected: ~2-3 MB (optimized)

# Verify WASM module structure
wasm-objdump -h worker/thread_flow_bg.wasm

# Expected sections:
# - Type
# - Function
# - Memory
# - Export
```

### 3. Build Optimizations

For production, use maximum optimization:

```bash
# Build with size optimization
cargo run -p xtask build-wasm --release --optimize-size

# Expected output:
# Optimization level: s (optimize for size)
# wasm-opt passes: -Os -Oz
# ✅ Optimized size: 1.8 MB (15% reduction)
```

### 4. Feature Flags for Edge

Edge builds MUST exclude certain features:

```toml
# Cargo.toml - Edge configuration
[features]
# Edge deployment - NO parallel, NO filesystem
worker = []

# Default features DISABLED for edge
default = []  # Empty for edge builds
```

Build command:

```bash
# Explicitly set features for edge
cargo build \
  --target wasm32-unknown-unknown \
  --release \
  --no-default-features \
  --features worker
```

---

## Edge Deployment

### 1. Deploy to Cloudflare Workers

```bash
# Deploy to production
wrangler deploy

# Expected output:
# ⛅️ wrangler 3.78.0
# ------------------
# Total Upload: 2.34 MB / gzip: 892 KB
# Uploaded thread-flow-worker (2.1 sec)
# Published thread-flow-worker (3.2 sec)
#   https://thread-flow-worker.your-account.workers.dev
# Current Deployment ID: abc123def456

# Deploy to specific environment
wrangler deploy --env development
wrangler deploy --env staging
```

### 2. Test Deployment

```bash
# Test with curl
curl -X POST https://thread-flow-worker.your-account.workers.dev \
  -H "Content-Type: application/json" \
  -d '{
    "code": "fn main() { println!(\"Hello\"); }",
    "language": "rust"
  }'

# Expected response:
# {
#   "symbols": [
#     { "kind": "function", "name": "main", "line": 1 }
#   ],
#   "cached": false,
#   "duration_ms": 15
# }

# Second request (cache hit)
# Same curl command - expect "cached": true, duration_ms < 1
```

### 3. View Deployment Logs

```bash
# Tail production logs
wrangler tail

# Expected output (real-time):
# [2025-01-28T12:34:56.789Z] POST /analyze 200 OK (15ms)
# [2025-01-28T12:34:57.123Z] D1 query: cache hit for hash abc123
# [2025-01-28T12:34:57.456Z] POST /analyze 200 OK (<1ms)

# Filter for errors only
wrangler tail --status error
```

### 4. Monitor D1 Database

```bash
# Query database from CLI
wrangler d1 execute thread-production \
  --command="SELECT COUNT(*) as cached_symbols FROM code_symbols;"

# Expected output:
# ┌────────────────┐
# │ cached_symbols │
# ├────────────────┤
# │ 1234           │
# └────────────────┘

# Check cache hit rate
wrangler d1 execute thread-production \
  --command="SELECT
    COUNT(*) as total,
    SUM(CASE WHEN updated_at > created_at THEN 1 ELSE 0 END) as cache_hits
  FROM code_symbols;"
```

---

## Environment Secrets Management

### 1. Add Secrets

```bash
# Add API keys or sensitive configuration
wrangler secret put THREAD_API_KEY
# Enter value at prompt: your-secret-api-key

wrangler secret put CLOUDFLARE_ACCOUNT_ID
# Enter value: your-account-id

# List secrets (values hidden)
wrangler secret list

# Expected output:
# [
#   { "name": "THREAD_API_KEY", "type": "secret_text" },
#   { "name": "CLOUDFLARE_ACCOUNT_ID", "type": "secret_text" }
# ]
```

### 2. Use Secrets in Worker

```javascript
// worker/index.js
export default {
  async fetch(request, env, ctx) {
    // Access secrets from env
    const apiKey = env.THREAD_API_KEY;
    const accountId = env.CLOUDFLARE_ACCOUNT_ID;

    // Validate API key from request header
    const requestKey = request.headers.get('X-API-Key');
    if (requestKey !== apiKey) {
      return new Response('Unauthorized', { status: 401 });
    }

    // Use in D1 queries with account context
    await env.DB.prepare(
      'INSERT INTO analytics (account_id, event) VALUES (?, ?)'
    ).bind(accountId, 'api_call').run();

    // ... rest of handler
  }
};
```

### 3. Environment-Specific Secrets

```bash
# Production secrets
wrangler secret put THREAD_API_KEY --env production
wrangler secret put DATABASE_ENCRYPTION_KEY --env production

# Development secrets (different values)
wrangler secret put THREAD_API_KEY --env development
wrangler secret put DATABASE_ENCRYPTION_KEY --env development
```

### 4. Secret Rotation

```bash
# Generate new API key
NEW_API_KEY=$(openssl rand -hex 32)

# Update secret
echo $NEW_API_KEY | wrangler secret put THREAD_API_KEY

# Verify deployment picked up new secret
wrangler tail --format json | jq '.outcome'
```

---

## Verification

### 1. Deployment Health Check

```bash
# Check worker status
wrangler deployments list

# Expected output:
# Created      Deployment ID  Version    Author
# 5 mins ago   abc123def456   1.0.2      you@example.com

# Check worker is running
curl https://thread-flow-worker.your-account.workers.dev/health

# Expected response:
# { "status": "healthy", "version": "1.0.2", "d1": "connected" }
```

### 2. D1 Performance Check

```bash
# Query D1 latency
wrangler d1 execute thread-production \
  --command="SELECT
    AVG(updated_at - created_at) as avg_query_ms,
    MAX(updated_at - created_at) as max_query_ms
  FROM code_symbols
  LIMIT 1000;"

# Expected:
# ┌──────────────┬──────────────┐
# │ avg_query_ms │ max_query_ms │
# ├──────────────┼──────────────┤
# │ 15           │ 48           │  ← Target: <50ms p95
# └──────────────┴──────────────┘
```

### 3. Cache Hit Rate Verification

```bash
# Test cache performance
for i in {1..10}; do
  curl -s -X POST https://thread-flow-worker.your-account.workers.dev \
    -H "Content-Type: application/json" \
    -d '{"code":"fn test(){}","language":"rust"}' \
    | jq '.cached'
done

# Expected output (after first request):
# false  ← First request (cache miss)
# true   ← Subsequent requests (cache hit)
# true
# true
# ...
```

### 4. Edge Distribution Check

```bash
# Check worker distribution across Cloudflare PoPs
wrangler tail --format json | jq -r '.logs[].colo'

# Expected output (varies by traffic):
# SJC  ← San Jose
# LHR  ← London
# NRT  ← Tokyo
# SYD  ← Sydney

# Indicates global edge deployment working
```

---

## Next Steps

### For Production Operations

1. **Set up monitoring** → Cloudflare Analytics + custom metrics
2. **Configure alerts** → D1 query failures, high latency (>50ms p95)
3. **Enable caching** → Cloudflare Cache API for additional layer
4. **Load testing** → Test with production request volumes

### For Performance Optimization

1. **Review D1 query patterns** → See `docs/operations/PERFORMANCE_TUNING.md`
2. **Optimize WASM size** → Further compression, tree shaking
3. **Implement batching** → Group multiple analyses per request
4. **Add read replicas** → D1 supports multi-region reads

### For Development Workflow

```bash
# Local development with Miniflare (D1 emulator)
wrangler dev --local

# Expected output:
# ⎔ Starting local server...
# ⎔ Ready on http://localhost:8787
# ⎔ D1 database: thread-development (local)

# Test locally
curl http://localhost:8787/analyze -d '{"code":"fn test(){}","language":"rust"}'
```

### Related Documentation

- **CLI Deployment**: `docs/deployment/CLI_DEPLOYMENT.md`
- **Performance Tuning**: `docs/operations/PERFORMANCE_TUNING.md`
- **Troubleshooting**: `docs/operations/TROUBLESHOOTING.md`
- **D1 Integration API**: `docs/api/D1_INTEGRATION_API.md`

---

## Deployment Checklist

Before deploying Thread Flow to Cloudflare Workers production:

- [ ] Cloudflare account with Workers Paid plan ($5/month)
- [ ] D1 database created and schema initialized
- [ ] `wrangler.toml` configured with correct account_id and database_id
- [ ] WASM module built with `--release --no-default-features --features worker`
- [ ] Secrets added via `wrangler secret put` (API keys, etc.)
- [ ] Environment variables configured in `wrangler.toml` [vars]
- [ ] Worker entry point (`worker/index.js`) implemented
- [ ] Deployment successful (`wrangler deploy`)
- [ ] Health check endpoint responding
- [ ] D1 queries executing with <50ms p95 latency
- [ ] Cache hit rate >90% after warm-up
- [ ] Logging and monitoring configured
- [ ] Custom domain/routes configured (if applicable)

---

**Deployment Target**: Cloudflare Workers (Edge/CDN)
**Concurrency Model**: tokio async (single-threaded, event-driven)
**Storage Backend**: Cloudflare D1 (distributed SQLite)
**Performance**: <50ms p95 latency, global edge distribution
**Constraints**: No filesystem, no multi-threading, 50ms CPU limit per request
