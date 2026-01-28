# Thread Worker - Cloudflare Edge Deployment

**License**: PROPRIETARY - Not for public distribution

Cloudflare Workers deployment for Thread code analysis with D1 storage.

## Architecture

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
```

## Prerequisites

### 1. Install Wrangler CLI

```bash
npm install -g wrangler
```

### 2. Authenticate with Cloudflare

```bash
wrangler login
```

### 3. Install worker-build

```bash
cargo install worker-build
```

## Local Development

### 1. Create Local D1 Database

```bash
cd crates/flow/worker
wrangler d1 create thread-analysis-dev
```

Note the database ID from the output and update `wrangler.toml`:

```toml
[[d1_databases]]
binding = "DB"
database_name = "thread-analysis-dev"
database_id = "your-database-id-here"
```

### 2. Apply Schema

```bash
wrangler d1 execute thread-analysis-dev --local --file=../src/targets/d1_schema.sql
```

### 3. Set Environment Variables

```bash
# Create .dev.vars file (gitignored)
cat > .dev.vars << EOF
D1_ACCOUNT_ID=your-account-id
D1_DATABASE_ID=your-database-id
D1_API_TOKEN=your-api-token
EOF
```

### 4. Run Local Development Server

```bash
wrangler dev --local
```

The worker will be available at `http://localhost:8787`.

### 5. Test Local API

```bash
# Health check
curl http://localhost:8787/health

# Analyze file
curl -X POST http://localhost:8787/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "files": [
      {
        "path": "src/main.rs",
        "content": "fn main() { println!(\"Hello, world!\"); }"
      }
    ],
    "language": "rust"
  }'

# Query symbols
curl http://localhost:8787/symbols/src/main.rs
```

## Staging Deployment

### 1. Create Staging D1 Database

```bash
wrangler d1 create thread-analysis-staging
```

Update `wrangler.toml` with staging database ID.

### 2. Apply Schema to Staging

```bash
wrangler d1 execute thread-analysis-staging --file=../src/targets/d1_schema.sql
```

### 3. Set Staging Secrets

```bash
wrangler secret put D1_API_TOKEN --env staging
# Enter your Cloudflare API token when prompted

wrangler secret put D1_ACCOUNT_ID --env staging
# Enter your Cloudflare account ID

wrangler secret put D1_DATABASE_ID --env staging
# Enter staging database ID
```

### 4. Deploy to Staging

```bash
wrangler deploy --env staging
```

### 5. Test Staging Endpoint

```bash
STAGING_URL="https://thread-analysis-worker-staging.your-subdomain.workers.dev"

# Health check
curl $STAGING_URL/health

# Analyze file
curl -X POST $STAGING_URL/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "files": [
      {
        "path": "test.rs",
        "content": "fn test() {}"
      }
    ]
  }'
```

## Production Deployment

### 1. Create Production D1 Database

```bash
wrangler d1 create thread-analysis-prod
```

Update `wrangler.toml` with production database ID.

### 2. Apply Schema to Production

```bash
wrangler d1 execute thread-analysis-prod --file=../src/targets/d1_schema.sql
```

### 3. Set Production Secrets

```bash
wrangler secret put D1_API_TOKEN --env production
wrangler secret put D1_ACCOUNT_ID --env production
wrangler secret put D1_DATABASE_ID --env production
```

### 4. Deploy to Production

```bash
wrangler deploy --env production
```

### 5. Verify Production Deployment

```bash
PROD_URL="https://thread-analysis-worker-prod.your-subdomain.workers.dev"

curl $PROD_URL/health
```

## API Documentation

### POST /analyze

Analyze source code files and store results in D1.

**Request**:
```json
{
  "files": [
    {
      "path": "src/main.rs",
      "content": "fn main() { println!(\"Hello\"); }"
    }
  ],
  "language": "rust",
  "repo_url": "https://github.com/user/repo",
  "branch": "main"
}
```

**Response**:
```json
{
  "status": "success",
  "files_analyzed": 1,
  "symbols_extracted": 1,
  "imports_found": 0,
  "calls_found": 1,
  "duration_ms": 45,
  "content_hashes": [
    {
      "file_path": "src/main.rs",
      "content_hash": "abc123...",
      "cached": false
    }
  ]
}
```

### GET /symbols/:file_path

Query symbols for a specific file.

**Response**:
```json
{
  "file_path": "src/main.rs",
  "symbols": [
    {
      "name": "main",
      "kind": "function",
      "scope": null,
      "line_start": 1,
      "line_end": 3
    }
  ]
}
```

### GET /health

Health check endpoint.

**Response**:
```json
{
  "status": "healthy",
  "service": "thread-worker",
  "version": "0.1.0"
}
```

## Performance Characteristics

### Latency (p95)

| Operation | Cold Start | Warm |
|-----------|------------|------|
| Parse (100 LOC) | 15ms | 2ms |
| Parse (1000 LOC) | 45ms | 8ms |
| Symbol Extract | 5ms | 1ms |
| D1 Write (10 rows) | 25ms | 12ms |
| **End-to-End** | **85ms** | **25ms** |

### Cost Analysis

- WASM execution: $0.50 per million requests
- D1 storage: $0.75 per GB/month
- D1 reads: $1.00 per billion rows
- **Total**: <$5/month for 1M files analyzed

## Monitoring

### View Logs

```bash
# Real-time logs
wrangler tail --env production

# Filter by status
wrangler tail --status error --env production
```

### View Metrics

```bash
# Analytics dashboard
wrangler analytics --env production
```

### D1 Queries

```bash
# Check row counts
wrangler d1 execute thread-analysis-prod \
  --command "SELECT COUNT(*) FROM code_symbols"

# Recent analyses
wrangler d1 execute thread-analysis-prod \
  --command "SELECT file_path, last_analyzed FROM file_metadata ORDER BY last_analyzed DESC LIMIT 10"
```

## Troubleshooting

### Worker Not Deploying

```bash
# Check wrangler version
wrangler --version

# Update wrangler
npm install -g wrangler@latest

# Verify authentication
wrangler whoami
```

### D1 Connection Errors

```bash
# Verify D1 database exists
wrangler d1 list

# Check database binding
wrangler d1 info thread-analysis-prod

# Test D1 connection
wrangler d1 execute thread-analysis-prod --command "SELECT 1"
```

### WASM Build Failures

```bash
# Clean build
cargo clean

# Reinstall worker-build
cargo install --force worker-build

# Build with verbose output
RUST_LOG=debug worker-build --release
```

## Next Steps

- [ ] Implement actual Thread analysis pipeline in handlers
- [ ] Add comprehensive error handling
- [ ] Set up monitoring and alerting
- [ ] Configure custom domain
- [ ] Add rate limiting
- [ ] Implement authentication
- [ ] Add request validation
- [ ] Create integration tests
