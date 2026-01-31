<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Environment Management

**Version**: 1.0.0
**Last Updated**: 2026-01-28
**Status**: Production Ready

---

## Overview

This document defines environment management strategies for Thread across development, staging, and production environments. It covers configuration hierarchy, environment-specific settings, promotion workflows, and validation procedures.

### Purpose

- **Environment Isolation**: Separate dev, staging, and production environments
- **Configuration Management**: Manage environment-specific settings consistently
- **Promotion Workflows**: Safe promotion from dev → staging → production
- **Validation**: Ensure configuration correctness before deployment

### Integration Points

- **Day 21 CI/CD**: Automated testing and deployment pipelines
- **Day 22 Security**: Secrets management and access control
- **Day 25 Deployment**: Deployment strategies and rollback procedures

---

## Environment Definitions

### Development Environment

**Purpose**: Local development and feature testing

**Characteristics**:
- **Persistence**: Ephemeral (can be recreated)
- **Data**: Synthetic test data
- **Access**: All developers
- **Uptime**: No SLO (downtime acceptable)
- **Cost**: Minimal (shared resources)

**Infrastructure**:
```
Development Environment
├─ Local Postgres (Docker)
├─ Single Thread instance (localhost:8080)
├─ Local caching (in-memory)
└─ Mock external services
```

**Configuration** (`config/dev.toml`):
```toml
[environment]
name = "development"
log_level = "debug"

[database]
url = "postgresql://thread:dev@localhost:5432/thread_dev"
max_connections = 10
connection_timeout = 5

[cache]
enabled = true
type = "in-memory"
max_size_mb = 100

[features]
parallel_processing = true
experimental_features = true  # Enable for testing
```

---

### Staging Environment

**Purpose**: Pre-production testing and validation

**Characteristics**:
- **Persistence**: Persistent (production-like)
- **Data**: Anonymized production data or realistic synthetic data
- **Access**: Developers + QA team
- **Uptime**: 95% SLO (maintenance windows acceptable)
- **Cost**: Medium (scaled-down production)

**Infrastructure**:
```
Staging Environment (AWS)
├─ 2 Thread worker instances (m5.large)
├─ RDS Postgres (db.t3.small)
├─ Redis cache (cache.t3.micro)
└─ Production-like configuration
```

**Configuration** (`config/staging.toml`):
```toml
[environment]
name = "staging"
log_level = "info"

[database]
url = "${DATABASE_URL}"  # From environment variable
max_connections = 50
connection_timeout = 10
ssl_mode = "require"

[cache]
enabled = true
type = "redis"
url = "${REDIS_URL}"
ttl_seconds = 3600

[monitoring]
prometheus_enabled = true
metrics_port = 9090

[features]
parallel_processing = true
experimental_features = false
```

---

### Production Environment

**Purpose**: Live customer-facing service

**Characteristics**:
- **Persistence**: Persistent (critical data)
- **Data**: Real customer data
- **Access**: Restricted (ops team only)
- **Uptime**: 99.9% SLO
- **Cost**: Optimized for performance and availability

**Infrastructure**:
```
Production Environment (AWS Multi-AZ)
├─ 5 Thread worker instances (c5.2xlarge)
├─ RDS Postgres Multi-AZ (db.r5.xlarge)
├─ Redis cluster (cache.r5.large × 3)
├─ Load balancer (ALB)
└─ CloudWatch monitoring
```

**Configuration** (`config/production.toml`):
```toml
[environment]
name = "production"
log_level = "warn"

[database]
url = "${DATABASE_URL}"
max_connections = 200
connection_timeout = 10
ssl_mode = "require"
pool_timeout = 30

[cache]
enabled = true
type = "redis-cluster"
url = "${REDIS_CLUSTER_URLS}"
ttl_seconds = 7200

[monitoring]
prometheus_enabled = true
metrics_port = 9090
alerting_enabled = true

[features]
parallel_processing = true
experimental_features = false  # Never in production

[security]
require_https = true
cors_origins = ["https://thread.example.com"]
rate_limit_per_minute = 1000
```

---

## Configuration Hierarchy

### Configuration Loading Order

Thread loads configuration in this order (later sources override earlier):

1. **Default Configuration** (`config/default.toml`) - Base defaults
2. **Environment Configuration** (`config/{env}.toml`) - Environment-specific
3. **Environment Variables** - Runtime overrides
4. **Command-Line Arguments** - Explicit overrides

**Example**:
```rust
// Configuration loading in code
use config::{Config, File, Environment};

let config = Config::builder()
    // 1. Load defaults
    .add_source(File::with_name("config/default"))
    // 2. Load environment-specific
    .add_source(File::with_name(&format!("config/{}", env)).required(false))
    // 3. Load environment variables (prefix: THREAD_)
    .add_source(Environment::with_prefix("THREAD"))
    // 4. Build final configuration
    .build()?;
```

### Configuration Overrides

**Environment Variable Format**:
```bash
# Override database URL
export THREAD_DATABASE_URL="postgresql://user:pass@host/db"

# Override log level
export THREAD_LOG_LEVEL="debug"

# Override nested configuration (using __)
export THREAD_CACHE__TTL_SECONDS="3600"
```

**Command-Line Arguments**:
```bash
# Override via CLI
thread-cli serve \
    --port 8080 \
    --database-url "postgresql://..." \
    --log-level info
```

---

## Environment Promotion Workflow

### Promotion Pipeline

```
Developer Laptop (local dev)
    │
    ├─ Code changes
    ├─ Local testing
    └─ Commit + Push
    │
    ▼
Development Environment (CI/CD)
    │
    ├─ Automated tests
    ├─ Security scans
    └─ Build artifacts
    │
    ▼
Staging Environment
    │
    ├─ Integration testing
    ├─ Performance testing
    ├─ QA validation
    └─ Manual approval
    │
    ▼
Production Environment
    │
    ├─ Blue-green deployment
    ├─ Smoke tests
    └─ Monitoring
```

### Promotion Criteria

**Development → Staging**:
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] Security scan shows no critical vulnerabilities
- [ ] Code review approved
- [ ] Build succeeds

**Staging → Production**:
- [ ] All staging tests pass
- [ ] QA approval obtained
- [ ] Performance benchmarks meet SLOs
- [ ] Change management approval (if required)
- [ ] Rollback plan documented
- [ ] Monitoring dashboards ready

### Automated Promotion Script

```bash
#!/bin/bash
# Promote build from staging to production

set -e

ARTIFACT_VERSION="$1"

if [[ -z "$ARTIFACT_VERSION" ]]; then
    echo "Usage: $0 <version>"
    exit 1
fi

echo "Promoting $ARTIFACT_VERSION to production..."

# 1. Verify staging tests passed
echo "Verifying staging tests..."
if ! ./scripts/check-staging-tests.sh "$ARTIFACT_VERSION"; then
    echo "ERROR: Staging tests failed"
    exit 1
fi

# 2. Verify QA approval
echo "Checking QA approval..."
if ! ./scripts/check-qa-approval.sh "$ARTIFACT_VERSION"; then
    echo "ERROR: QA approval missing"
    exit 1
fi

# 3. Create deployment tag
echo "Creating deployment tag..."
git tag -a "deploy/production/$ARTIFACT_VERSION" -m "Production deployment: $ARTIFACT_VERSION"
git push origin "deploy/production/$ARTIFACT_VERSION"

# 4. Trigger production deployment
echo "Triggering production deployment..."
gh workflow run deploy-production.yml \
    --ref "deploy/production/$ARTIFACT_VERSION" \
    -f deployment_strategy=blue-green

echo "Production deployment initiated"
echo "Monitor at: https://github.com/org/repo/actions"
```

---

## Environment-Specific Configuration

### Database Configuration

**Development**:
```toml
[database]
url = "postgresql://thread:dev@localhost/thread_dev"
max_connections = 10
ssl_mode = "disable"  # Local development
pool_min = 0
pool_max = 10
```

**Staging**:
```toml
[database]
url = "${DATABASE_URL}"  # From AWS Secrets Manager
max_connections = 50
ssl_mode = "require"
pool_min = 10
pool_max = 50
connection_timeout = 10
idle_timeout = 600
```

**Production**:
```toml
[database]
url = "${DATABASE_URL}"  # From AWS Secrets Manager
max_connections = 200
ssl_mode = "require"
pool_min = 50
pool_max = 200
connection_timeout = 10
idle_timeout = 600
statement_timeout = 30000
```

### Caching Configuration

**Development**:
```toml
[cache]
enabled = true
type = "in-memory"
max_size_mb = 100
ttl_seconds = 300
```

**Staging**:
```toml
[cache]
enabled = true
type = "redis"
url = "${REDIS_URL}"
max_connections = 20
ttl_seconds = 3600
key_prefix = "staging:"
```

**Production**:
```toml
[cache]
enabled = true
type = "redis-cluster"
url = "${REDIS_CLUSTER_URLS}"
max_connections = 100
ttl_seconds = 7200
key_prefix = "prod:"
eviction_policy = "lru"
```

### Logging Configuration

**Development**:
```toml
[logging]
level = "debug"
format = "pretty"  # Human-readable
output = "stdout"
include_file_location = true
```

**Staging**:
```toml
[logging]
level = "info"
format = "json"
output = "stdout"
include_file_location = true
sample_rate = 1.0  # Log all requests
```

**Production**:
```toml
[logging]
level = "warn"
format = "json"
output = "stdout"
include_file_location = false  # Performance optimization
sample_rate = 0.1  # Log 10% of requests
error_sample_rate = 1.0  # Always log errors
```

### Feature Flags

**Development**:
```toml
[features]
parallel_processing = true
experimental_features = true
debug_endpoints = true
performance_profiling = true
```

**Staging**:
```toml
[features]
parallel_processing = true
experimental_features = false  # Test production config
debug_endpoints = true
performance_profiling = true
```

**Production**:
```toml
[features]
parallel_processing = true
experimental_features = false
debug_endpoints = false  # Security: disable debug
performance_profiling = false  # Performance: disable overhead
```

---

## Configuration Validation

### Pre-Deployment Validation

**Validation Script** (`scripts/validate-config.sh`):
```bash
#!/bin/bash
# Validate environment configuration

ENV="$1"

if [[ -z "$ENV" ]]; then
    echo "Usage: $0 <environment>"
    exit 1
fi

CONFIG_FILE="config/${ENV}.toml"

if [[ ! -f "$CONFIG_FILE" ]]; then
    echo "ERROR: Configuration file not found: $CONFIG_FILE"
    exit 1
fi

echo "Validating $ENV configuration..."

# 1. Parse TOML syntax
echo "Checking TOML syntax..."
if ! toml-lint "$CONFIG_FILE"; then
    echo "ERROR: Invalid TOML syntax"
    exit 1
fi

# 2. Validate required fields
echo "Validating required fields..."
required_fields=(
    "environment.name"
    "database.url"
    "database.max_connections"
    "cache.enabled"
)

for field in "${required_fields[@]}"; do
    if ! toml get "$CONFIG_FILE" "$field" &>/dev/null; then
        echo "ERROR: Missing required field: $field"
        exit 1
    fi
done

# 3. Environment-specific validation
if [[ "$ENV" == "production" ]]; then
    echo "Validating production-specific requirements..."

    # Production must use SSL
    ssl_mode=$(toml get "$CONFIG_FILE" "database.ssl_mode")
    if [[ "$ssl_mode" != "require" ]]; then
        echo "ERROR: Production must use database.ssl_mode = 'require'"
        exit 1
    fi

    # Production must not enable experimental features
    experimental=$(toml get "$CONFIG_FILE" "features.experimental_features")
    if [[ "$experimental" == "true" ]]; then
        echo "ERROR: Production cannot have experimental_features enabled"
        exit 1
    fi

    # Production must not have debug endpoints
    debug=$(toml get "$CONFIG_FILE" "features.debug_endpoints")
    if [[ "$debug" == "true" ]]; then
        echo "ERROR: Production cannot have debug_endpoints enabled"
        exit 1
    fi
fi

echo "Configuration validation: PASSED"
```

### Runtime Configuration Validation

**Validation in Code**:
```rust
// Runtime configuration validation
use anyhow::{Context, Result};

pub fn validate_config(config: &AppConfig) -> Result<()> {
    // Validate environment-specific rules
    match config.environment.name.as_str() {
        "production" => validate_production_config(config)?,
        "staging" => validate_staging_config(config)?,
        "development" => validate_development_config(config)?,
        _ => anyhow::bail!("Unknown environment: {}", config.environment.name),
    }

    // Validate database configuration
    if config.database.max_connections < 10 {
        anyhow::bail!("database.max_connections must be at least 10");
    }

    // Validate cache configuration
    if config.cache.enabled && config.cache.ttl_seconds == 0 {
        anyhow::bail!("cache.ttl_seconds must be > 0 when cache is enabled");
    }

    Ok(())
}

fn validate_production_config(config: &AppConfig) -> Result<()> {
    // Production-specific validation
    if config.features.experimental_features {
        anyhow::bail!("Experimental features not allowed in production");
    }

    if config.features.debug_endpoints {
        anyhow::bail!("Debug endpoints not allowed in production");
    }

    if !config.database.ssl_mode.contains("require") {
        anyhow::bail!("Production database must use SSL");
    }

    if config.security.require_https != Some(true) {
        anyhow::bail!("Production must require HTTPS");
    }

    Ok(())
}
```

---

## Secrets Management

### Environment Variable Secrets

**Never Commit Secrets**:
```toml
# ❌ WRONG: Hardcoded secret
[database]
url = "postgresql://user:password@host/db"

# ✅ CORRECT: Reference environment variable
[database]
url = "${DATABASE_URL}"
```

**Secrets Loading**:
```bash
# Development: .env file (gitignored)
echo "DATABASE_URL=postgresql://..." > .env
source .env

# Staging/Production: AWS Secrets Manager
aws secretsmanager get-secret-value \
    --secret-id thread/staging/database \
    --query SecretString \
    --output text | jq -r '.DATABASE_URL'
```

### Secrets Configuration Files

**Development** (`.env.development`):
```bash
# Local development secrets (gitignored)
DATABASE_URL=postgresql://thread:dev@localhost/thread_dev
REDIS_URL=redis://localhost:6379
SECRET_KEY=dev-secret-key-not-for-production
```

**Staging** (AWS Secrets Manager):
```json
{
  "DATABASE_URL": "postgresql://...",
  "REDIS_URL": "redis://...",
  "SECRET_KEY": "staging-secret-...",
  "CLOUDFLARE_API_TOKEN": "..."
}
```

**Production** (AWS Secrets Manager):
```json
{
  "DATABASE_URL": "postgresql://...",
  "REDIS_CLUSTER_URLS": "redis://node1,redis://node2,redis://node3",
  "SECRET_KEY": "prod-secret-...",
  "CLOUDFLARE_API_TOKEN": "...",
  "PROMETHEUS_TOKEN": "..."
}
```

---

## Best Practices

### 1. Environment Parity

**Principle**: Keep dev, staging, and production as similar as possible

**Implementation**:
- Use same infrastructure (Postgres in all envs, not SQLite in dev)
- Use same software versions (same Rust version, same dependencies)
- Use production-like configuration in staging
- Scale down resources in staging, but keep architecture identical

### 2. Configuration as Code

**Principle**: All configuration in version control

**Implementation**:
- Store configuration files in Git (except secrets)
- Use pull requests for configuration changes
- Review configuration changes like code changes
- Track configuration changes over time

### 3. Fail-Safe Defaults

**Principle**: Default to most secure/safe settings

**Implementation**:
```toml
# Default configuration (config/default.toml)
[security]
require_https = true  # Default to secure
cors_origins = []     # Default to no CORS (explicit allow)

[features]
experimental_features = false  # Default to stable
debug_endpoints = false        # Default to secure
```

### 4. Validate Before Deploy

**Principle**: Catch configuration errors before deployment

**Implementation**:
- Run `validate-config.sh` in CI/CD pipeline
- Require manual approval for production configuration changes
- Test configuration in staging before production

### 5. Document Configuration Changes

**Principle**: Every configuration change should have documentation

**Implementation**:
```
Commit: Update production database max_connections

Increase max_connections from 100 to 200 to handle increased traffic.

Rationale:
- Current usage: 90 connections (90% of capacity)
- Expected growth: 50% in next month
- New limit: 200 (provides 2× headroom)

Testing:
- Validated in staging with load tests
- Observed no performance degradation

Rollback Plan:
- If issues, revert to 100 via environment variable
- No application restart required
```

---

**Document Version**: 1.0.0
**Last Updated**: 2026-01-28
**Next Review**: 2026-02-28
**Owner**: Thread Operations Team
