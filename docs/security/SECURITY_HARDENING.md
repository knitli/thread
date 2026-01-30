<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Security Hardening Guide

**Version**: 1.0
**Last Updated**: 2026-01-28
**Classification**: Public

---

## Table of Contents

- [Overview](#overview)
- [Threat Model](#threat-model)
- [Security Architecture](#security-architecture)
- [Hardening CLI Deployments](#hardening-cli-deployments)
- [Hardening Edge Deployments](#hardening-edge-deployments)
- [Database Security](#database-security)
- [Network Security](#network-security)
- [Application Security](#application-security)
- [Monitoring and Detection](#monitoring-and-detection)

---

## Overview

This guide provides comprehensive security hardening recommendations for Thread deployments across CLI, Edge, and containerized environments.

### Security Principles

1. **Defense in Depth**: Multiple layers of security controls
2. **Least Privilege**: Minimal permissions by default
3. **Fail Secure**: Default to secure state on failure
4. **Complete Mediation**: Check every access
5. **Separation of Privilege**: Require multiple conditions for critical operations

### Compliance Standards

- **OWASP Top 10 (2021)**: All categories addressed
- **CWE Top 25**: Mitigations implemented
- **NIST Cybersecurity Framework**: Aligned with core functions

---

## Threat Model

### Assets

**Primary Assets**:
- Source code being analyzed
- Analysis results and metadata
- Database contents (PostgreSQL, D1)
- API keys and credentials
- User data and configurations

**Secondary Assets**:
- Build artifacts
- Deployment infrastructure
- Monitoring data
- Log files

### Threat Actors

**External Attackers**:
- **Motivation**: Data theft, service disruption, unauthorized access
- **Capability**: Low to high sophistication
- **Access**: Internet-facing services

**Insider Threats**:
- **Motivation**: Data exfiltration, sabotage
- **Capability**: Medium to high sophistication
- **Access**: Internal systems, code repositories

**Supply Chain**:
- **Motivation**: Widespread compromise
- **Capability**: Variable
- **Access**: Dependencies, build tools

### Attack Vectors

**1. Code Injection**:
- **Risk**: High
- **Impact**: Remote code execution
- **Mitigations**:
  - Input validation
  - Parameterized queries
  - Sandboxing (WASM)

**2. Dependency Vulnerabilities**:
- **Risk**: Medium
- **Impact**: Variable based on vulnerability
- **Mitigations**:
  - Daily security scans
  - Rapid patching
  - Dependency pinning

**3. Credential Compromise**:
- **Risk**: Medium
- **Impact**: Unauthorized access
- **Mitigations**:
  - Secrets management
  - Credential rotation
  - MFA where applicable

**4. Denial of Service**:
- **Risk**: Medium
- **Impact**: Service unavailability
- **Mitigations**:
  - Rate limiting
  - Resource quotas
  - Circuit breakers

**5. Data Exfiltration**:
- **Risk**: Low to Medium
- **Impact**: Confidentiality breach
- **Mitigations**:
  - Access logging
  - Encryption in transit
  - Least privilege access

---

## Security Architecture

### Layered Defense

```
┌─────────────────────────────────────────────┐
│         Application Layer                    │
│  • Input validation                          │
│  • Output encoding                           │
│  • Authentication/Authorization              │
└─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────┐
│         Runtime Layer                        │
│  • Process isolation                         │
│  • Resource limits                           │
│  • Sandboxing (WASM)                        │
└─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────┐
│         Network Layer                        │
│  • TLS encryption                            │
│  • Firewall rules                            │
│  • Rate limiting                             │
└─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────┐
│         Infrastructure Layer                 │
│  • OS hardening                              │
│  • Access controls                           │
│  • Audit logging                             │
└─────────────────────────────────────────────┘
```

### Security Boundaries

**Trust Boundaries**:
1. User input → Application
2. Application → Database
3. CLI → Network services
4. Edge Worker → D1 database

**Each boundary requires**:
- Input validation
- Authentication
- Authorization
- Audit logging

---

## Hardening CLI Deployments

### System-Level Hardening

**Operating System**:

```bash
# Update system packages
apt update && apt upgrade -y

# Install security updates automatically
apt install unattended-upgrades
dpkg-reconfigure -plow unattended-upgrades

# Configure firewall
ufw default deny incoming
ufw default allow outgoing
ufw allow 22/tcp  # SSH
ufw allow 8080/tcp  # Application
ufw enable
```

**User and Permissions**:

```bash
# Create dedicated service user
useradd --system --no-create-home --shell /bin/false thread

# Set up working directory
mkdir -p /var/lib/thread
chown thread:thread /var/lib/thread
chmod 750 /var/lib/thread

# Limit user permissions
usermod -a -G nogroup thread
```

### Systemd Service Hardening

**Enhanced systemd configuration**:

```ini
[Unit]
Description=Thread Code Analysis Service
After=network.target postgresql.service

[Service]
Type=simple
User=thread
Group=thread
WorkingDirectory=/var/lib/thread

# Binary and environment
ExecStart=/usr/local/bin/thread serve
Environment="RUST_LOG=info"
EnvironmentFile=-/etc/thread/environment

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
PrivateDevices=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/thread
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
RestrictAddressFamilies=AF_INET AF_INET6 AF_UNIX
RestrictNamespaces=true
LockPersonality=true
RestrictRealtime=true
RestrictSUIDSGID=true
RemoveIPC=true
PrivateMounts=true
SystemCallFilter=@system-service
SystemCallErrorNumber=EPERM
SystemCallArchitectures=native

# Resource limits
LimitNOFILE=65536
LimitNPROC=512
MemoryMax=2G
CPUQuota=200%
TasksMax=1024

# Restart policy
Restart=on-failure
RestartSec=10s
StartLimitBurst=5
StartLimitIntervalSec=100s

[Install]
WantedBy=multi-user.target
```

### File System Security

**Permissions**:

```bash
# Binary permissions
chmod 755 /usr/local/bin/thread
chown root:root /usr/local/bin/thread

# Configuration files
chmod 640 /etc/thread/config.toml
chown root:thread /etc/thread/config.toml

# Data directory
chmod 750 /var/lib/thread
chown thread:thread /var/lib/thread

# Log directory
chmod 750 /var/log/thread
chown thread:adm /var/log/thread
```

**AppArmor Profile** (optional):

```
# /etc/apparmor.d/usr.local.bin.thread
#include <tunables/global>

/usr/local/bin/thread {
  #include <abstractions/base>
  #include <abstractions/nameservice>

  /usr/local/bin/thread mr,
  /var/lib/thread/** rw,
  /var/log/thread/** w,

  network inet stream,
  network inet6 stream,

  deny /proc/** w,
  deny /sys/** w,
  deny /home/** r,
}
```

### Environment Variables Security

**Never store in systemd unit**:

```bash
# Create environment file
cat > /etc/thread/environment <<EOF
DATABASE_URL=postgresql://thread:$(openssl rand -base64 32)@localhost:5432/thread
API_KEY=$(openssl rand -base64 32)
EOF

# Secure permissions
chmod 600 /etc/thread/environment
chown root:root /etc/thread/environment

# Reference in systemd
EnvironmentFile=/etc/thread/environment
```

---

## Hardening Edge Deployments

### Cloudflare Workers Security

**Environment Variables**:

```bash
# Never commit secrets
wrangler secret put DATABASE_URL
wrangler secret put API_KEY

# Use different secrets per environment
wrangler secret put DATABASE_URL --env production
wrangler secret put DATABASE_URL --env staging
```

**wrangler.toml Security**:

```toml
# NO SECRETS IN THIS FILE
name = "thread-production"

[env.production]
# Public configuration only
vars = {
  ENVIRONMENT = "production",
  LOG_LEVEL = "info"
}

# Secrets go in environment (wrangler secret)
# DATABASE_URL - set via wrangler secret
# API_KEY - set via wrangler secret
```

### WASM Sandboxing

**Automatic Protections**:
- No filesystem access
- No network access (except fetch API)
- Memory limits (128MB default)
- CPU time limits (50ms per request, 30s per cron)
- No process spawning

**Additional Hardening**:

```rust
// Limit request payload size
if request.size() > 1_000_000 {
    return Response::error("Payload too large", 413);
}

// Validate content type
let content_type = request.headers().get("content-type")?;
if !["application/json", "text/plain"].contains(&content_type.as_str()) {
    return Response::error("Invalid content type", 415);
}

// Implement request timeout
let result = tokio::time::timeout(
    Duration::from_secs(25),
    process_request(request)
).await?;
```

### D1 Database Security

**Connection Security**:
- Automatic encryption in transit
- No direct network access (Workers-only)
- Built-in SQL injection protection

**Query Hardening**:

```rust
// Use parameterized queries (always)
let result = db.prepare("SELECT * FROM files WHERE hash = ?1")
    .bind(&[hash])?
    .all()
    .await?;

// Implement row limits
let result = db.prepare("SELECT * FROM files LIMIT ?1")
    .bind(&[100])?  // Hard limit
    .all()
    .await?;

// Validate query complexity
if query.contains("JOIN") && query.matches("JOIN").count() > 3 {
    return Err("Query too complex");
}
```

---

## Database Security

### PostgreSQL Hardening

**Connection Security**:

```ini
# postgresql.conf
ssl = on
ssl_cert_file = '/etc/postgresql/15/main/server.crt'
ssl_key_file = '/etc/postgresql/15/main/server.key'
ssl_ca_file = '/etc/postgresql/15/main/root.crt'

password_encryption = scram-sha-256
```

**Authentication**:

```ini
# pg_hba.conf
# TYPE  DATABASE        USER            ADDRESS                 METHOD

# Local connections
local   all             postgres                                peer
local   all             thread                                  scram-sha-256

# Remote connections (require SSL)
hostssl thread          thread          10.0.0.0/8              scram-sha-256
hostssl all             all             0.0.0.0/0               reject
```

**User Privileges**:

```sql
-- Create application user with minimal privileges
CREATE USER thread WITH PASSWORD 'secure_password';
GRANT CONNECT ON DATABASE thread TO thread;
GRANT USAGE ON SCHEMA public TO thread;

-- Table-specific permissions
GRANT SELECT, INSERT, UPDATE ON files TO thread;
GRANT SELECT ON symbols TO thread;  -- Read-only where appropriate

-- Revoke dangerous permissions
REVOKE CREATE ON SCHEMA public FROM PUBLIC;
REVOKE ALL ON pg_catalog.pg_authid FROM thread;

-- Create read-only user for reporting
CREATE USER thread_readonly WITH PASSWORD 'readonly_password';
GRANT CONNECT ON DATABASE thread TO thread_readonly;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO thread_readonly;
```

**Query Logging**:

```sql
-- Enable query logging for security auditing
ALTER SYSTEM SET log_statement = 'mod';  -- Log modifications
ALTER SYSTEM SET log_min_duration_statement = 1000;  -- Log slow queries
ALTER SYSTEM SET log_connections = on;
ALTER SYSTEM SET log_disconnections = on;

SELECT pg_reload_conf();
```

### Connection Pooling Security

```rust
// Use connection pooling with limits
use sqlx::postgres::PgPoolOptions;

let pool = PgPoolOptions::new()
    .max_connections(10)  // Limit concurrent connections
    .min_connections(2)
    .connect_timeout(Duration::from_secs(5))
    .idle_timeout(Duration::from_secs(600))
    .max_lifetime(Duration::from_secs(1800))
    .connect(&database_url)
    .await?;
```

---

## Network Security

### TLS Configuration

**Nginx TLS Setup**:

```nginx
server {
    listen 443 ssl http2;
    server_name thread.example.com;

    # SSL certificate
    ssl_certificate /etc/letsencrypt/live/thread.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/thread.example.com/privkey.pem;

    # Modern TLS configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_prefer_server_ciphers on;
    ssl_ciphers 'ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256';

    # HSTS
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # OCSP stapling
    ssl_stapling on;
    ssl_stapling_verify on;
    ssl_trusted_certificate /etc/letsencrypt/live/thread.example.com/chain.pem;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Rate Limiting

**Nginx Rate Limiting**:

```nginx
# Define rate limit zones
limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
limit_req_zone $binary_remote_addr zone=auth:10m rate=5r/m;

server {
    location /api/ {
        limit_req zone=api burst=20 nodelay;
        limit_req_status 429;
    }

    location /auth/ {
        limit_req zone=auth burst=10;
        limit_req_status 429;
    }
}
```

**Application-Level Rate Limiting**:

```rust
use tower::limit::RateLimitLayer;

let rate_limit = RateLimitLayer::new(
    100,  // requests
    Duration::from_secs(60)  // per minute
);

app.layer(rate_limit)
```

### Firewall Rules

**UFW Configuration**:

```bash
# Default deny
ufw default deny incoming
ufw default allow outgoing

# SSH (consider changing default port)
ufw limit 22/tcp

# HTTP/HTTPS
ufw allow 80/tcp
ufw allow 443/tcp

# PostgreSQL (only from application server)
ufw allow from 10.0.1.0/24 to any port 5432

# Prometheus metrics (internal only)
ufw allow from 10.0.0.0/8 to any port 9090

ufw enable
```

---

## Application Security

### Input Validation

**Validation Framework**:

```rust
use validator::Validate;

#[derive(Debug, Validate)]
struct FileAnalysisRequest {
    #[validate(length(min = 1, max = 255))]
    file_path: String,

    #[validate(regex = "^[a-f0-9]{64}$")]
    hash: String,

    #[validate(range(min = 1, max = 1000000))]
    max_symbols: Option<usize>,
}

// Validate all inputs
let request = FileAnalysisRequest { /* ... */ };
request.validate()?;
```

**SQL Injection Prevention**:

```rust
// ALWAYS use parameterized queries
let result = sqlx::query!(
    "SELECT * FROM files WHERE hash = $1",
    hash
).fetch_one(&pool).await?;

// NEVER concatenate user input
// ❌ WRONG
// let query = format!("SELECT * FROM files WHERE hash = '{}'", hash);
```

### Authentication and Authorization

**API Key Management**:

```rust
// Secure API key verification
use constant_time_eq::constant_time_eq;

fn verify_api_key(provided: &str, expected: &str) -> bool {
    // Prevent timing attacks
    constant_time_eq(provided.as_bytes(), expected.as_bytes())
}

// Middleware for authentication
async fn auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response> {
    let api_key = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(Error::Unauthorized)?;

    if !verify_api_key(api_key, &CONFIG.api_key) {
        return Err(Error::Unauthorized);
    }

    next.run(request).await
}
```

### Secure Error Handling

**Never leak sensitive information in errors**:

```rust
// ❌ WRONG - Leaks database details
return Err(format!("Database connection failed: {}", db_error));

// ✅ CORRECT - Generic error to user, detailed logging
log::error!("Database connection failed: {}", db_error);
return Err("Internal server error".into());
```

### Logging Security

**Sanitize logs**:

```rust
// Remove sensitive data from logs
fn sanitize_for_logging(data: &str) -> String {
    // Redact API keys
    let re = Regex::new(r"api_key=[^&]+").unwrap();
    let sanitized = re.replace_all(data, "api_key=REDACTED");

    // Redact tokens
    let re = Regex::new(r"token=[^&]+").unwrap();
    re.replace_all(&sanitized, "token=REDACTED").to_string()
}

// Log sanitized version
log::info!("Request: {}", sanitize_for_logging(&request_data));
```

---

## Monitoring and Detection

### Security Event Logging

**Audit Log Events**:
- Authentication attempts (success/failure)
- Authorization failures
- Configuration changes
- Data access
- Privileged operations

**Implementation**:

```rust
// Security audit logging
log::warn!(
    "auth_failure: ip={}, user={}, reason={}",
    remote_ip,
    username,
    "invalid_credentials"
);

log::info!(
    "config_change: user={}, setting={}, old={}, new={}",
    user,
    setting_name,
    old_value,
    new_value
);
```

### Intrusion Detection

**fail2ban Configuration**:

```ini
# /etc/fail2ban/jail.local
[thread-auth]
enabled = true
port = 8080
filter = thread-auth
logpath = /var/log/thread/access.log
maxretry = 5
bantime = 3600
findtime = 600

# /etc/fail2ban/filter.d/thread-auth.conf
[Definition]
failregex = auth_failure: ip=<HOST>
ignoreregex =
```

### Alerting Rules

**Prometheus Alerts**:

```yaml
groups:
  - name: security
    rules:
      - alert: HighAuthFailureRate
        expr: rate(auth_failures_total[5m]) > 10
        for: 5m
        annotations:
          summary: "High authentication failure rate detected"

      - alert: DatabaseConnectionFailures
        expr: database_connection_errors_total > 5
        for: 5m
        annotations:
          summary: "Multiple database connection failures"

      - alert: UnusualTrafficPattern
        expr: rate(http_requests_total[5m]) > 1000
        for: 2m
        annotations:
          summary: "Unusual traffic pattern detected"
```

---

## Security Checklist

### Pre-Deployment

- [ ] Security audit completed
- [ ] Dependencies scanned (cargo audit)
- [ ] Secrets not in code or configs
- [ ] TLS certificates configured
- [ ] Firewall rules implemented
- [ ] Rate limiting configured
- [ ] Monitoring and alerting set up
- [ ] Backup and recovery tested
- [ ] Incident response plan documented

### Post-Deployment

- [ ] Initial security scan
- [ ] Monitor logs for anomalies
- [ ] Verify rate limiting works
- [ ] Test incident response
- [ ] Review access logs
- [ ] Validate monitoring alerts
- [ ] Confirm backups working

### Regular Maintenance

**Daily**:
- [ ] Review security alerts
- [ ] Check audit logs
- [ ] Monitor for anomalies

**Weekly**:
- [ ] Review access logs
- [ ] Check for outdated dependencies
- [ ] Verify backup integrity

**Monthly**:
- [ ] Security scan
- [ ] Access review
- [ ] Update dependencies
- [ ] Test incident response

**Quarterly**:
- [ ] Full security audit
- [ ] Penetration testing
- [ ] Update threat model
- [ ] Review and update documentation

---

**Last Updated**: 2026-01-28
**Review Cycle**: Quarterly
**Next Review**: 2026-04-28
