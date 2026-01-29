# Thread Deployment Guide

**Version**: 1.0
**Last Updated**: 2026-01-28

---

## Overview

Thread supports three primary deployment models:

1. **CLI Deployment** - Native binaries on Linux, macOS, Windows
2. **Edge Deployment** - Cloudflare Workers with WASM
3. **Docker Deployment** - Containerized deployment with orchestration

Each deployment model is optimized for specific use cases and infrastructure requirements.

---

## Quick Start

### CLI Deployment (Ubuntu/Debian)

```bash
# Download deployment script
curl -LO https://raw.githubusercontent.com/knitli/thread/main/docs/deployment/cli-deployment.sh

# Make executable
chmod +x cli-deployment.sh

# Run as root
sudo ./cli-deployment.sh
```

### Edge Deployment (Cloudflare Workers)

```bash
# Set environment variables
export CLOUDFLARE_API_TOKEN=your_token
export CLOUDFLARE_ACCOUNT_ID=your_account_id

# Run deployment script
./edge-deployment.sh
```

### Docker Deployment

```bash
# Set database password
export DB_PASSWORD=your_secure_password

# Start services
docker-compose up -d

# Check status
docker-compose ps
```

---

## Deployment Scripts

### cli-deployment.sh

**Purpose**: Automated CLI installation on Linux servers

**Features**:
- Downloads and installs latest or specific version
- Creates systemd service for background operation
- Sets up service user and permissions
- Configures database connection
- Includes health checks and rollback support

**Usage**:

```bash
# Install latest version
sudo ./cli-deployment.sh

# Install specific version
sudo VERSION=0.1.0 ./cli-deployment.sh

# Custom installation directory
sudo INSTALL_DIR=/opt/thread ./cli-deployment.sh

# Custom architecture
sudo TARGET_ARCH=aarch64-unknown-linux-gnu ./cli-deployment.sh
```

**Environment Variables**:

| Variable | Default | Description |
|----------|---------|-------------|
| `VERSION` | `latest` | Version to install |
| `TARGET_ARCH` | `x86_64-unknown-linux-gnu` | Target architecture |
| `INSTALL_DIR` | `/usr/local/bin` | Installation directory |
| `SERVICE_USER` | `thread` | System user for service |
| `SYSTEMD_SERVICE` | `thread` | Systemd service name |

**Post-Installation**:

1. Configure database:
   ```bash
   sudo -u postgres psql
   CREATE DATABASE thread;
   CREATE USER thread WITH PASSWORD 'your_password';
   GRANT ALL PRIVILEGES ON DATABASE thread TO thread;
   ```

2. Update service configuration:
   ```bash
   sudo vi /etc/systemd/system/thread.service
   # Update DATABASE_URL with actual credentials
   ```

3. Restart service:
   ```bash
   sudo systemctl restart thread.service
   sudo systemctl status thread.service
   ```

---

### edge-deployment.sh

**Purpose**: Automated deployment to Cloudflare Workers

**Features**:
- Builds optimized WASM for Edge
- Validates Cloudflare credentials
- Runs pre-deployment tests
- Deploys to specified environment
- Includes smoke tests and rollback support

**Usage**:

```bash
# Deploy to production
ENVIRONMENT=production ./edge-deployment.sh

# Deploy to staging
ENVIRONMENT=staging ./edge-deployment.sh

# Development build
./edge-deployment.sh --dev

# Skip tests
./edge-deployment.sh --skip-tests

# Rollback deployment
./edge-deployment.sh --rollback
```

**Environment Variables**:

| Variable | Required | Description |
|----------|----------|-------------|
| `CLOUDFLARE_API_TOKEN` | Yes | Cloudflare API token |
| `CLOUDFLARE_ACCOUNT_ID` | Yes | Cloudflare account ID |
| `ENVIRONMENT` | No | Deployment environment (default: production) |
| `WASM_BUILD` | No | Build type: release or dev (default: release) |

**Getting Cloudflare Credentials**:

1. API Token:
   - Visit https://dash.cloudflare.com/profile/api-tokens
   - Create token with "Edit Cloudflare Workers" template
   - Copy token: `export CLOUDFLARE_API_TOKEN=your_token`

2. Account ID:
   - Visit https://dash.cloudflare.com
   - Select your account
   - Copy Account ID from URL or Overview page
   - `export CLOUDFLARE_ACCOUNT_ID=your_account_id`

**Post-Deployment**:

```bash
# View live logs
wrangler tail --env production

# Check deployments
wrangler deployments list --env production

# Test endpoint
curl https://thread.knit.li/health
```

---

### docker-compose.yml

**Purpose**: Full-stack containerized deployment

**Services Included**:
- `thread` - Main application (port 8080)
- `postgres` - PostgreSQL database (port 5432)
- `redis` - Caching layer (port 6379)
- `prometheus` - Metrics collection (port 9091)
- `grafana` - Dashboard visualization (port 3000)
- `nginx` - Reverse proxy (ports 80/443)

**Usage**:

```bash
# Start all services
docker-compose up -d

# Start specific service
docker-compose up -d thread postgres

# View logs
docker-compose logs -f thread

# Scale application
docker-compose up -d --scale thread=3

# Stop all services
docker-compose down

# Stop and remove volumes
docker-compose down -v
```

**Environment Configuration**:

Create `.env` file:

```env
# Database
DB_PASSWORD=your_secure_password

# Grafana
GRAFANA_PASSWORD=admin_password

# Application
RUST_LOG=info
ENABLE_CACHING=true
```

**Volume Management**:

```bash
# List volumes
docker volume ls | grep thread

# Backup database
docker exec thread-postgres pg_dump -U thread thread > backup.sql

# Restore database
cat backup.sql | docker exec -i thread-postgres psql -U thread thread
```

**Accessing Services**:

| Service | URL | Credentials |
|---------|-----|-------------|
| Application | http://localhost:8080 | - |
| Grafana | http://localhost:3000 | admin / ${GRAFANA_PASSWORD} |
| Prometheus | http://localhost:9091 | - |
| Postgres | postgresql://localhost:5432/thread | thread / ${DB_PASSWORD} |

---

## Monitoring and Observability

### Prometheus Metrics

**Metrics Endpoint**: `http://localhost:9090/metrics`

**Key Metrics**:
- `thread_cache_hit_rate` - Cache efficiency
- `thread_query_latency_milliseconds` - Query performance
- `thread_error_rate` - Error percentage
- `thread_files_processed_total` - Throughput counter

### Grafana Dashboards

**Dashboard Import**:

```bash
# Copy dashboard configuration
cp docs/dashboards/grafana-dashboard.json grafana/dashboards/

# Restart Grafana
docker-compose restart grafana
```

**Access**:
- URL: http://localhost:3000
- Username: `admin`
- Password: Value of `$GRAFANA_PASSWORD`

### Viewing Logs

**Docker Logs**:
```bash
# Application logs
docker-compose logs -f thread

# Database logs
docker-compose logs -f postgres

# All services
docker-compose logs -f
```

**Systemd Logs** (CLI deployment):
```bash
# View live logs
journalctl -fu thread.service

# Last 100 lines
journalctl -u thread.service -n 100

# Logs since boot
journalctl -u thread.service -b
```

---

## Security Considerations

### SSL/TLS Configuration

**Docker Nginx**:

```bash
# Generate self-signed certificate (development)
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout ssl/thread.key \
  -out ssl/thread.crt

# Use Let's Encrypt (production)
certbot certonly --standalone -d thread.example.com
cp /etc/letsencrypt/live/thread.example.com/*.pem ssl/
```

**Cloudflare Edge**:
- SSL/TLS automatic with Cloudflare
- Configure in Cloudflare Dashboard → SSL/TLS
- Recommended: Full (strict) mode

### Database Security

**PostgreSQL Hardening**:

```sql
-- Revoke public schema access
REVOKE CREATE ON SCHEMA public FROM PUBLIC;

-- Create read-only user
CREATE USER thread_readonly WITH PASSWORD 'password';
GRANT CONNECT ON DATABASE thread TO thread_readonly;
GRANT USAGE ON SCHEMA public TO thread_readonly;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO thread_readonly;

-- Enable SSL connections
ALTER SYSTEM SET ssl = on;
```

**Connection String** (with SSL):
```
postgresql://thread:password@localhost:5432/thread?sslmode=require
```

### Secrets Management

**Docker Secrets**:

```bash
# Create secret
echo "my_db_password" | docker secret create db_password -

# Use in compose file
secrets:
  db_password:
    external: true
```

**Environment Variables**:
- Never commit `.env` file to version control
- Use `.env.example` as template
- Rotate credentials regularly

---

## Scaling and High Availability

### Horizontal Scaling

**Docker Swarm**:

```bash
# Initialize swarm
docker swarm init

# Deploy stack
docker stack deploy -c docker-compose.yml thread

# Scale service
docker service scale thread_thread=5
```

**Kubernetes** (Future):
- Helm charts for deployment
- Horizontal Pod Autoscaler
- Persistent Volume Claims

### Load Balancing

**Nginx Configuration**:

```nginx
upstream thread_backend {
    least_conn;
    server thread1:8080;
    server thread2:8080;
    server thread3:8080;
}

server {
    listen 80;
    server_name thread.example.com;

    location / {
        proxy_pass http://thread_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

**Cloudflare Edge**:
- Automatic global load balancing
- Geographic distribution
- DDoS protection included

### Database Replication

**Postgres Streaming Replication**:

```bash
# Primary server
wal_level = replica
max_wal_senders = 3
max_replication_slots = 3

# Replica server
primary_conninfo = 'host=primary port=5432 user=replicator'
```

---

## Troubleshooting

### Common Issues

**1. Service Won't Start**

```bash
# Check service status
sudo systemctl status thread.service

# View detailed logs
journalctl -xeu thread.service

# Verify binary
/usr/local/bin/thread --version

# Check permissions
ls -la /usr/local/bin/thread
```

**2. Database Connection Failures**

```bash
# Test connection
psql -h localhost -U thread -d thread

# Check PostgreSQL status
sudo systemctl status postgresql

# Verify network
netstat -tlnp | grep 5432
```

**3. Docker Container Crashes**

```bash
# Check container status
docker-compose ps

# View container logs
docker-compose logs thread

# Inspect container
docker inspect thread-app

# Restart container
docker-compose restart thread
```

**4. WASM Build Failures**

```bash
# Verify wasm32 target
rustup target list --installed

# Clean and rebuild
cargo clean
cargo run -p xtask build-wasm --release

# Check wasm-pack version
wasm-pack --version
```

### Performance Issues

**High CPU Usage**:
```bash
# Check process stats
top -p $(pgrep thread)

# Profile with perf
sudo perf record -F 99 -p $(pgrep thread) -g -- sleep 60
sudo perf report
```

**Memory Leaks**:
```bash
# Monitor memory usage
watch -n 1 'ps aux | grep thread'

# Enable allocation profiling
RUST_BACKTRACE=full RUST_LOG=debug thread serve
```

**Slow Queries**:
```sql
-- Enable query logging
ALTER SYSTEM SET log_min_duration_statement = 100;  -- Log queries >100ms

-- Analyze slow queries
SELECT query, mean_exec_time, calls
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;
```

---

## Maintenance

### Backups

**Database Backup**:

```bash
# Automated backup script
#!/bin/bash
BACKUP_DIR=/var/backups/thread
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup
pg_dump -U thread -h localhost thread | gzip > "${BACKUP_DIR}/thread_${DATE}.sql.gz"

# Retain last 30 days
find "${BACKUP_DIR}" -name "thread_*.sql.gz" -mtime +30 -delete
```

**Docker Volume Backup**:

```bash
# Backup volume
docker run --rm \
  -v thread_postgres_data:/data \
  -v $(pwd):/backup \
  alpine tar czf /backup/postgres_data.tar.gz /data

# Restore volume
docker run --rm \
  -v thread_postgres_data:/data \
  -v $(pwd):/backup \
  alpine tar xzf /backup/postgres_data.tar.gz -C /
```

### Updates

**CLI Update**:

```bash
# Download new version
sudo VERSION=0.2.0 ./cli-deployment.sh

# Verify update
thread --version

# Restart service
sudo systemctl restart thread.service
```

**Docker Update**:

```bash
# Pull new image
docker-compose pull thread

# Recreate container
docker-compose up -d thread

# Verify
docker-compose ps
```

**Edge Update**:

```bash
# Redeploy
./edge-deployment.sh

# Verify
curl https://thread.knit.li/version
```

---

## Support and Resources

- **Documentation**: https://github.com/knitli/thread/tree/main/docs
- **Issues**: https://github.com/knitli/thread/issues
- **Discussions**: https://github.com/knitli/thread/discussions
- **Security**: security@knit.li

---

**Last Updated**: 2026-01-28
**Maintained By**: Thread Development Team
