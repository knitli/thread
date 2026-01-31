<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Thread Flow CLI Deployment Guide

Comprehensive guide for deploying Thread Flow in CLI/local environments with PostgreSQL backend and parallel processing.

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Local Development Setup](#local-development-setup)
3. [PostgreSQL Backend Configuration](#postgresql-backend-configuration)
4. [Parallel Processing Setup](#parallel-processing-setup)
5. [Production CLI Deployment](#production-cli-deployment)
6. [Environment Variables](#environment-variables)
7. [Verification](#verification)
8. [Next Steps](#next-steps)

---

## Prerequisites

### System Requirements

- **Operating System**: Linux, macOS, or Windows with WSL2
- **Rust**: 1.75.0 or later (edition 2024)
- **CPU**: Multi-core recommended for parallel processing (2+ cores)
- **Memory**: 4GB minimum, 8GB+ recommended for large codebases
- **Disk**: 500MB+ for Thread binaries and dependencies

### Required Software

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# PostgreSQL 14+ (for persistent caching)
# Ubuntu/Debian
sudo apt install postgresql postgresql-contrib

# macOS
brew install postgresql@14

# Verify installations
rustc --version  # Should be 1.75.0+
psql --version   # Should be 14+
```

### Optional Tools

- **mise**: Development environment manager (`curl https://mise.run | sh`)
- **cargo-nextest**: Fast test runner (`cargo install cargo-nextest`)
- **cargo-watch**: Auto-rebuild on changes (`cargo install cargo-watch`)

---

## Local Development Setup

### 1. Clone and Build

```bash
# Clone repository
git clone https://github.com/your-org/thread.git
cd thread

# Install development tools (if using mise)
mise run install-tools

# Build with all features (CLI configuration)
cargo build --workspace --all-features --release

# Verify build
./target/release/thread --version
```

### 2. Feature Flags for CLI

Thread Flow CLI builds use these default features:

```toml
# Cargo.toml - CLI configuration
[features]
default = ["recoco-minimal", "parallel"]

# PostgreSQL backend support
recoco-postgres = ["recoco-minimal", "recoco/target-postgres"]

# Parallel processing (Rayon)
parallel = ["dep:rayon"]

# Query result caching (optional but recommended)
caching = ["dep:moka"]
```

**Recommended CLI Build**:

```bash
# Full-featured CLI with PostgreSQL, parallelism, and caching
cargo build --release --features "recoco-postgres,parallel,caching"
```

### 3. Directory Structure

```
thread/
├── crates/flow/           # Thread Flow library
├── target/release/        # Compiled binaries
│   └── thread             # Main CLI binary
├── data/                  # Analysis results (create this)
└── .env                   # Environment configuration (create this)
```

Create required directories:

```bash
mkdir -p data
touch .env
```

---

## PostgreSQL Backend Configuration

### 1. Database Setup

```bash
# Start PostgreSQL service
# Linux
sudo systemctl start postgresql
sudo systemctl enable postgresql

# macOS
brew services start postgresql@14

# Create database and user
sudo -u postgres psql

# Inside psql:
CREATE DATABASE thread_cache;
CREATE USER thread_user WITH ENCRYPTED PASSWORD 'your_secure_password';
GRANT ALL PRIVILEGES ON DATABASE thread_cache TO thread_user;
\q
```

### 2. Schema Initialization

Thread Flow uses ReCoco's PostgreSQL target which auto-creates tables. The schema includes:

```sql
-- Content-addressed cache table (auto-created by ReCoco)
CREATE TABLE IF NOT EXISTS code_symbols (
    content_hash TEXT PRIMARY KEY,      -- Blake3 fingerprint
    file_path TEXT NOT NULL,
    language TEXT,
    symbols JSONB,                      -- Extracted symbol data
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Indexes for fast lookups
CREATE INDEX idx_symbols_file_path ON code_symbols(file_path);
CREATE INDEX idx_symbols_language ON code_symbols(language);
CREATE INDEX idx_symbols_created ON code_symbols(created_at);
```

No manual schema creation needed—ReCoco handles this automatically.

### 3. Connection Configuration

Create `.env` in project root:

```bash
# .env - PostgreSQL connection
DATABASE_URL=postgresql://thread_user:your_secure_password@localhost:5432/thread_cache

# Optional: Connection pool settings
DB_POOL_SIZE=10
DB_CONNECTION_TIMEOUT=30
```

### 4. Verify PostgreSQL Connection

```bash
# Test connection
psql -U thread_user -d thread_cache -h localhost

# Inside psql - verify tables exist after first run
\dt
\d code_symbols
\q
```

---

## Parallel Processing Setup

### 1. Rayon Configuration

Thread Flow uses **Rayon** for CPU-bound parallel processing in CLI environments.

**Default Behavior** (automatic):
- Rayon detects available CPU cores
- Spawns worker threads = num_cores
- Distributes file processing across threads

**Manual Thread Control** (optional):

```bash
# Set RAYON_NUM_THREADS environment variable
export RAYON_NUM_THREADS=4  # Use 4 cores

# Or in .env file
echo "RAYON_NUM_THREADS=4" >> .env
```

### 2. Performance Characteristics

| CPU Cores | 100 Files | 1000 Files | 10,000 Files |
|-----------|-----------|------------|--------------|
| 1 core    | ~1.6s     | ~16s       | ~160s        |
| 2 cores   | ~0.8s     | ~8s        | ~80s         |
| 4 cores   | ~0.4s     | ~4s        | ~40s         |
| 8 cores   | ~0.2s     | ~2s        | ~20s         |

**Speedup**: Linear with core count (2-8x typical)

### 3. Optimal Thread Count

**Recommended Settings**:

```bash
# CPU-bound workloads (parsing, AST analysis)
# Use all physical cores
RAYON_NUM_THREADS=$(nproc)  # Linux
RAYON_NUM_THREADS=$(sysctl -n hw.ncpu)  # macOS

# I/O-bound workloads (file reading, database queries)
# Use 2x physical cores
RAYON_NUM_THREADS=$(($(nproc) * 2))  # Linux

# Mixed workloads (default)
# Let Rayon auto-detect
unset RAYON_NUM_THREADS
```

### 4. Verify Parallel Processing

```bash
# Check feature is enabled
cargo tree --features | grep rayon

# Expected output:
# └── rayon v1.10.0

# Run with parallel logging
RUST_LOG=thread_flow=debug cargo run --release --features parallel

# Look for log entries indicating parallel execution:
# [DEBUG thread_flow::batch] Processing 100 files across 4 threads
```

---

## Production CLI Deployment

### 1. Build Optimized Binary

```bash
# Release build with full optimizations
cargo build \
  --release \
  --features "recoco-postgres,parallel,caching" \
  --workspace

# Binary location
ls -lh target/release/thread
# Should be ~15-25MB

# Optional: Strip debug symbols for smaller binary
strip target/release/thread
# Should reduce to ~10-15MB
```

### 2. Install System-Wide

```bash
# Copy binary to system path
sudo cp target/release/thread /usr/local/bin/

# Verify installation
thread --version
thread --help
```

### 3. Production Configuration

Create production config file:

```bash
# /etc/thread/config.env
DATABASE_URL=postgresql://thread_user:strong_password@db.production.com:5432/thread_cache
RAYON_NUM_THREADS=8
RUST_LOG=thread_flow=info

# Cache configuration
THREAD_CACHE_MAX_CAPACITY=100000  # 100k entries
THREAD_CACHE_TTL_SECONDS=3600     # 1 hour

# Performance tuning
THREAD_BATCH_SIZE=100  # Files per batch
```

### 4. Systemd Service (Linux)

```ini
# /etc/systemd/system/thread-analyzer.service
[Unit]
Description=Thread Code Analyzer Service
After=network.target postgresql.service

[Service]
Type=simple
User=thread
Group=thread
EnvironmentFile=/etc/thread/config.env
ExecStart=/usr/local/bin/thread analyze --watch /var/projects
Restart=on-failure
RestartSec=10

# Resource limits
MemoryLimit=4G
CPUQuota=400%  # 4 cores max

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable thread-analyzer
sudo systemctl start thread-analyzer
sudo systemctl status thread-analyzer
```

### 5. Docker Deployment (Alternative)

```dockerfile
# Dockerfile - Production CLI
FROM rust:1.75-slim as builder

WORKDIR /build
COPY . .

# Build with production features
RUN cargo build --release \
    --features "recoco-postgres,parallel,caching" \
    --workspace

FROM debian:bookworm-slim

# Install PostgreSQL client libraries
RUN apt-get update && apt-get install -y \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/thread /usr/local/bin/

# Create non-root user
RUN useradd -m -u 1001 thread
USER thread

ENTRYPOINT ["thread"]
```

Build and run:

```bash
# Build image
docker build -t thread-cli:latest .

# Run with PostgreSQL connection
docker run --rm \
  -e DATABASE_URL=postgresql://thread_user:pass@host.docker.internal:5432/thread_cache \
  -e RAYON_NUM_THREADS=4 \
  -v $(pwd)/data:/data \
  thread-cli:latest analyze /data
```

---

## Environment Variables

### Core Configuration

| Variable | Purpose | Default | Example |
|----------|---------|---------|---------|
| `DATABASE_URL` | PostgreSQL connection string | None (required) | `postgresql://user:pass@localhost/thread` |
| `RAYON_NUM_THREADS` | Parallel processing thread count | Auto-detect | `4` |
| `RUST_LOG` | Logging level | `info` | `thread_flow=debug` |

### Cache Configuration

| Variable | Purpose | Default | Example |
|----------|---------|---------|---------|
| `THREAD_CACHE_MAX_CAPACITY` | Max cache entries | `10000` | `100000` |
| `THREAD_CACHE_TTL_SECONDS` | Cache entry lifetime | `300` (5 min) | `3600` (1 hour) |

### Performance Tuning

| Variable | Purpose | Default | Example |
|----------|---------|---------|---------|
| `THREAD_BATCH_SIZE` | Files per batch | `100` | `500` |
| `DB_POOL_SIZE` | PostgreSQL connection pool size | `10` | `20` |
| `DB_CONNECTION_TIMEOUT` | Database connection timeout (sec) | `30` | `60` |

### Example `.env` File

```bash
# Production CLI Configuration

# PostgreSQL backend
DATABASE_URL=postgresql://thread_user:secure_password@localhost:5432/thread_cache
DB_POOL_SIZE=20
DB_CONNECTION_TIMEOUT=60

# Parallel processing
RAYON_NUM_THREADS=8

# Caching (100k entries, 1 hour TTL)
THREAD_CACHE_MAX_CAPACITY=100000
THREAD_CACHE_TTL_SECONDS=3600

# Performance
THREAD_BATCH_SIZE=500

# Logging
RUST_LOG=thread_flow=info,thread_services=info
```

---

## Verification

### 1. Health Checks

```bash
# Verify binary works
thread --version
# Expected: thread 0.1.0

# Verify PostgreSQL connection
thread db-check
# Expected: ✅ PostgreSQL connection successful

# Verify parallel processing
thread system-info
# Expected:
# CPU Cores: 8
# Rayon Threads: 8
# Parallel Processing: Enabled

# Verify cache configuration
thread cache-stats
# Expected:
# Cache Capacity: 100,000 entries
# Cache TTL: 3600 seconds
# Current Entries: 0
```

### 2. Test Analysis Run

```bash
# Analyze small test project
thread analyze ./test-project

# Expected output:
# Analyzing 10 files across 4 threads...
# Blake3 fingerprinting: 10 files in 4.25µs
# Cache hits: 0 (0.0%)
# Parsing: 10 files in 1.47ms
# Extracting symbols: 150 symbols found
# PostgreSQL export: 10 records inserted
# Total time: 15.2ms

# Second run (cache hit)
thread analyze ./test-project

# Expected output:
# Analyzing 10 files across 4 threads...
# Blake3 fingerprinting: 10 files in 4.25µs
# Cache hits: 10 (100.0%)  ← All files cached!
# Total time: 0.5ms  ← 30x faster!
```

### 3. PostgreSQL Data Verification

```bash
# Query cached data
psql -U thread_user -d thread_cache -c "
  SELECT
    content_hash,
    file_path,
    language,
    jsonb_array_length(symbols) as symbol_count
  FROM code_symbols
  LIMIT 5;
"

# Expected output:
#     content_hash     |     file_path      | language | symbol_count
# --------------------+--------------------+----------+--------------
#  abc123...          | src/main.rs        | rust     | 15
#  def456...          | src/lib.rs         | rust     | 42
```

### 4. Performance Benchmarks

```bash
# Run official benchmarks
cargo bench --features "parallel,caching"

# Expected results:
# fingerprint_benchmark  425 ns per file (Blake3)
# parse_benchmark        147 µs per file (tree-sitter)
# cache_hit_benchmark    <1 µs (memory lookup)

# Speedup:
# Fingerprint vs Parse: 346x faster
# Cache vs Parse:       147,000x faster
```

---

## Next Steps

### For Production Deployment

1. **Set up monitoring** → See `docs/operations/PERFORMANCE_TUNING.md`
2. **Configure alerts** → Database connection failures, cache misses >10%
3. **Enable backup** → PostgreSQL regular backups for cache data
4. **Load testing** → Test with production-scale codebases

### For Development Workflow

1. **Install cargo-watch** → Auto-rebuild on code changes
   ```bash
   cargo install cargo-watch
   cargo watch -x "run --features parallel"
   ```

2. **Enable debug logging** → Detailed execution traces
   ```bash
   RUST_LOG=thread_flow=trace cargo run
   ```

3. **Profile performance** → Identify bottlenecks
   ```bash
   cargo build --release --features parallel
   perf record ./target/release/thread analyze large-project/
   perf report
   ```

### Related Documentation

- **Edge Deployment**: `crates/cloudflare/docs/EDGE_DEPLOYMENT.md` (segregated - see crates/cloudflare/)
- **Performance Tuning**: `docs/operations/PERFORMANCE_TUNING.md`
- **Troubleshooting**: `docs/operations/TROUBLESHOOTING.md`
- **Architecture Overview**: `docs/architecture/THREAD_FLOW_ARCHITECTURE.md`

---

## Deployment Checklist

Before deploying Thread Flow CLI to production:

- [ ] PostgreSQL 14+ installed and configured
- [ ] Database user and permissions created
- [ ] Environment variables configured in `.env` or systemd service
- [ ] Binary built with `--release --features "recoco-postgres,parallel,caching"`
- [ ] Health checks passing (`thread --version`, `thread db-check`)
- [ ] Test analysis run successful with cache hits on second run
- [ ] Logging configured (`RUST_LOG` appropriate for environment)
- [ ] Resource limits set (systemd `MemoryLimit`, `CPUQuota`)
- [ ] Backup strategy for PostgreSQL cache data
- [ ] Monitoring and alerting configured

---

**Deployment Target**: CLI/Local environments with PostgreSQL backend
**Concurrency Model**: Rayon (multi-threaded parallelism)
**Storage Backend**: PostgreSQL (persistent caching)
**Performance**: 2-8x speedup on multi-core, 99.7% cache cost reduction
