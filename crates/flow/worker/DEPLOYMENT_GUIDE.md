# Deployment Guide - Thread Worker

Step-by-step guide for deploying Thread analysis to Cloudflare Workers.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Initial Setup](#initial-setup)
3. [Staging Deployment](#staging-deployment)
4. [Production Deployment](#production-deployment)
5. [Rollback Procedure](#rollback-procedure)
6. [Monitoring](#monitoring)

## Prerequisites

### Required Tools

- [x] Node.js 18+ and npm
- [x] Rust toolchain (1.85+)
- [x] Cloudflare account with Workers enabled
- [x] Cloudflare API token with D1 permissions

### Install Wrangler

```bash
npm install -g wrangler
wrangler login
```

### Install worker-build

```bash
cargo install worker-build
```

## Initial Setup

### 1. Project Configuration

Navigate to worker directory:

```bash
cd crates/flow/worker
```

### 2. Create D1 Databases

Create development database:

```bash
wrangler d1 create thread-analysis-dev
# Save the database ID from output
```

Create staging database:

```bash
wrangler d1 create thread-analysis-staging
# Save the database ID from output
```

Create production database:

```bash
wrangler d1 create thread-analysis-prod
# Save the database ID from output
```

### 3. Update wrangler.toml

Edit `wrangler.toml` and fill in the database IDs:

```toml
[[d1_databases]]
binding = "DB"
database_name = "thread-analysis"
database_id = "your-dev-database-id-here"

[env.staging.d1_databases]
# ... staging database ID

[env.production.d1_databases]
# ... production database ID
```

### 4. Apply Database Schema

Development:
```bash
wrangler d1 execute thread-analysis-dev \
  --local \
  --file=../src/targets/d1_schema.sql
```

Staging:
```bash
wrangler d1 execute thread-analysis-staging \
  --file=../src/targets/d1_schema.sql
```

Production:
```bash
wrangler d1 execute thread-analysis-prod \
  --file=../src/targets/d1_schema.sql
```

### 5. Set Up Secrets

Development (.dev.vars file):
```bash
cat > .dev.vars << EOF
D1_ACCOUNT_ID=your-cloudflare-account-id
D1_DATABASE_ID=your-dev-database-id
D1_API_TOKEN=your-api-token
EOF
```

Staging:
```bash
echo "your-api-token" | wrangler secret put D1_API_TOKEN --env staging
echo "your-account-id" | wrangler secret put D1_ACCOUNT_ID --env staging
echo "staging-db-id" | wrangler secret put D1_DATABASE_ID --env staging
```

Production:
```bash
echo "your-api-token" | wrangler secret put D1_API_TOKEN --env production
echo "your-account-id" | wrangler secret put D1_ACCOUNT_ID --env production
echo "prod-db-id" | wrangler secret put D1_DATABASE_ID --env production
```

## Staging Deployment

### 1. Pre-Deployment Checklist

- [ ] All code changes committed to git
- [ ] Local tests passing
- [ ] Schema applied to staging D1
- [ ] Secrets configured

### 2. Build WASM

```bash
# Clean previous builds
cargo clean

# Build optimized WASM
worker-build --release
```

### 3. Deploy to Staging

```bash
wrangler deploy --env staging
```

Expected output:
```
✨ Built successfully!
✨ Successfully published your Worker!
🌍 https://thread-analysis-worker-staging.your-subdomain.workers.dev
```

### 4. Smoke Test Staging

Health check:
```bash
STAGING_URL="https://thread-analysis-worker-staging.your-subdomain.workers.dev"
curl $STAGING_URL/health
```

Expected response:
```json
{
  "status": "healthy",
  "service": "thread-worker",
  "version": "0.1.0"
}
```

Analysis test:
```bash
curl -X POST $STAGING_URL/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "files": [
      {
        "path": "test.rs",
        "content": "fn test() { println!(\"test\"); }"
      }
    ],
    "language": "rust"
  }'
```

### 5. Staging Validation

Run integration tests:
```bash
# TODO: Create integration test suite
cargo test --test edge_integration -- --test-threads=1
```

Check D1 data:
```bash
wrangler d1 execute thread-analysis-staging \
  --command "SELECT COUNT(*) as total FROM code_symbols"
```

Monitor logs:
```bash
wrangler tail --env staging
```

## Production Deployment

### 1. Production Checklist

- [ ] Staging deployment successful
- [ ] Integration tests passing on staging
- [ ] Performance validated (<100ms p95)
- [ ] Error rate acceptable (<1%)
- [ ] Database migrations applied
- [ ] Secrets configured
- [ ] Rollback plan documented
- [ ] Monitoring alerts configured

### 2. Pre-Deployment Communication

Notify team:
```
Deploying Thread Worker to production
- Release: v0.1.0
- Changes: Initial edge deployment
- Estimated downtime: 0 seconds (zero-downtime deployment)
- Rollback plan: Immediate via wrangler rollback
```

### 3. Deploy to Production

```bash
# Final build verification
worker-build --release

# Deploy
wrangler deploy --env production

# Save deployment ID
DEPLOYMENT_ID=$(wrangler deployments list --env production | head -2 | tail -1 | awk '{print $1}')
echo "Deployment ID: $DEPLOYMENT_ID"
```

### 4. Production Smoke Tests

```bash
PROD_URL="https://thread-analysis-worker-prod.your-subdomain.workers.dev"

# Health check
curl $PROD_URL/health

# Quick analysis test
curl -X POST $PROD_URL/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "files": [{
      "path": "smoke_test.rs",
      "content": "fn main() {}"
    }]
  }'
```

### 5. Post-Deployment Monitoring

Watch logs for 15 minutes:
```bash
wrangler tail --env production --status error
```

Check metrics:
```bash
wrangler analytics --env production
```

Verify D1 writes:
```bash
wrangler d1 execute thread-analysis-prod \
  --command "SELECT file_path, last_analyzed FROM file_metadata ORDER BY last_analyzed DESC LIMIT 5"
```

## Rollback Procedure

### Immediate Rollback

If issues detected within 15 minutes of deployment:

```bash
# List recent deployments
wrangler deployments list --env production

# Rollback to previous deployment
wrangler rollback --env production --message "Rollback due to [issue]"
```

### Manual Rollback

If automatic rollback fails:

```bash
# Redeploy previous version from git
git checkout <previous-commit>
wrangler deploy --env production
git checkout main
```

### Post-Rollback

1. Investigate root cause
2. Fix issues in development
3. Test thoroughly in staging
4. Redeploy to production

## Monitoring

### Real-Time Logs

```bash
# All logs
wrangler tail --env production

# Errors only
wrangler tail --env production --status error

# Specific search
wrangler tail --env production --search "D1Error"
```

### Analytics

```bash
# Request counts
wrangler analytics --env production

# Error rates
wrangler analytics --env production --metrics errors
```

### D1 Health Checks

```bash
# Table row counts
wrangler d1 execute thread-analysis-prod \
  --command "
    SELECT
      'symbols' as table_name, COUNT(*) as rows FROM code_symbols
    UNION ALL
    SELECT 'imports', COUNT(*) FROM code_imports
    UNION ALL
    SELECT 'calls', COUNT(*) FROM code_calls
    UNION ALL
    SELECT 'metadata', COUNT(*) FROM file_metadata
  "

# Recent activity
wrangler d1 execute thread-analysis-prod \
  --command "
    SELECT
      file_path,
      last_analyzed,
      analysis_version
    FROM file_metadata
    ORDER BY last_analyzed DESC
    LIMIT 10
  "
```

### Performance Monitoring

```bash
# Latency percentiles (via analytics dashboard)
wrangler analytics --env production --metrics duration

# CPU time usage
wrangler analytics --env production --metrics cpu_time
```

## Troubleshooting

### Deployment Fails

```bash
# Check syntax
wrangler publish --dry-run --env production

# Verbose logging
RUST_LOG=debug wrangler deploy --env production
```

### Worker Errors After Deployment

```bash
# Check error logs
wrangler tail --env production --status error

# View recent deployments
wrangler deployments list --env production

# Immediate rollback
wrangler rollback --env production
```

### D1 Connection Issues

```bash
# Verify database exists
wrangler d1 list

# Check binding configuration
cat wrangler.toml | grep -A5 "d1_databases"

# Test D1 connectivity
wrangler d1 execute thread-analysis-prod --command "SELECT 1"
```

### High Error Rate

1. Check logs: `wrangler tail --env production --status error`
2. Identify error pattern
3. If critical: rollback immediately
4. If non-critical: monitor and fix in next release

### High Latency

1. Check analytics: `wrangler analytics --env production --metrics duration`
2. Identify slow operations
3. Check D1 performance: row counts, index usage
4. Consider optimization in next release

## Maintenance

### Database Cleanup

```bash
# Remove old analysis data (optional)
wrangler d1 execute thread-analysis-prod \
  --command "
    DELETE FROM file_metadata
    WHERE last_analyzed < datetime('now', '-30 days')
  "
```

### Schema Updates

```bash
# Create migration script
cat > migration_v2.sql << EOF
-- Add new column
ALTER TABLE code_symbols ADD COLUMN metadata TEXT;

-- Create index
CREATE INDEX IF NOT EXISTS idx_symbols_metadata ON code_symbols(metadata);
EOF

# Apply to staging
wrangler d1 execute thread-analysis-staging --file=migration_v2.sql

# Test in staging

# Apply to production
wrangler d1 execute thread-analysis-prod --file=migration_v2.sql
```

## Emergency Contacts

- **Cloudflare Support**: https://support.cloudflare.com
- **Status Page**: https://www.cloudflarestatus.com
- **Documentation**: https://developers.cloudflare.com/workers

## Success Criteria

- [ ] Health endpoint returns 200 OK
- [ ] Analysis endpoint processes requests successfully
- [ ] D1 writes confirmed
- [ ] Error rate <1%
- [ ] p95 latency <100ms
- [ ] No critical logs in first 15 minutes
- [ ] Monitoring dashboards show green status
