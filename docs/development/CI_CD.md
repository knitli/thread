# CI/CD Pipeline Documentation

**Version**: 1.0
**Last Updated**: 2026-01-28
**Status**: Production Ready

---

## Table of Contents

- [Overview](#overview)
- [CI Pipeline](#ci-pipeline)
- [Release Pipeline](#release-pipeline)
- [Deployment Strategies](#deployment-strategies)
- [Secrets Management](#secrets-management)
- [Troubleshooting](#troubleshooting)
- [Best Practices](#best-practices)

---

## Overview

Thread uses GitHub Actions for continuous integration and deployment across multiple platforms:

- **CLI Builds**: Multi-platform native binaries (Linux, macOS, Windows)
- **Edge Deployment**: Cloudflare Workers with WASM
- **Docker Images**: Multi-arch containers for deployment
- **Package Publishing**: crates.io for Rust ecosystem

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      GitHub Actions                          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌────────────┐  ┌─────────────┐  ┌──────────────┐        │
│  │   CI/CD    │  │  Release    │  │  Deployment  │        │
│  │  Pipeline  │  │  Automation │  │   Workflows  │        │
│  └────────────┘  └─────────────┘  └──────────────┘        │
│        │               │                   │                │
│        ▼               ▼                   ▼                │
│  ┌────────────┐  ┌─────────────┐  ┌──────────────┐        │
│  │  Testing   │  │   Build &   │  │  Cloudflare  │        │
│  │  Coverage  │  │   Package   │  │   Workers    │        │
│  │  Security  │  │             │  │              │        │
│  └────────────┘  └─────────────┘  └──────────────┘        │
└─────────────────────────────────────────────────────────────┘
```

---

## CI Pipeline

### Workflow File

`.github/workflows/ci.yml`

### Trigger Conditions

```yaml
on:
  push:
    branches: [main, develop, staging, "001-*"]
  pull_request:
    branches: [main, develop, staging]
  workflow_dispatch:  # Manual trigger
```

### Pipeline Jobs

#### 1. Quick Checks (Fast Fail)

**Purpose**: Fail fast on formatting and linting issues

**Jobs**:
- `cargo fmt --check` - Code formatting validation
- `cargo clippy` - Linting with zero warnings policy
- `typos` - Spell checking

**Runtime**: ~2-3 minutes

```bash
# Run locally before push
mise run lint
# or
cargo fmt --all -- --check
cargo clippy --workspace --all-features --all-targets -- -D warnings
```

#### 2. Multi-Platform Testing

**Purpose**: Ensure compatibility across operating systems

**Matrix**:
| OS | Rust Versions |
|----|--------------|
| ubuntu-latest | stable, beta, nightly |
| macos-latest | stable |
| windows-latest | stable |

**Test Strategy**:
- `cargo nextest` for parallel test execution
- `cargo test --doc` for documentation tests
- Integration tests with Postgres (main branch only)

**Runtime**: ~8-15 minutes per platform

```bash
# Run locally
mise run test
# or
cargo nextest run --all-features --no-fail-fast
```

#### 3. WASM Build Verification

**Purpose**: Validate Edge deployment target

**Steps**:
1. Install `wasm32-unknown-unknown` target
2. Build development WASM
3. Build release WASM (optimized)
4. Upload artifacts for inspection

**Runtime**: ~5-7 minutes

```bash
# Run locally
mise run build-wasm-release
# or
cargo run -p xtask build-wasm --release
```

#### 4. Security Audit

**Purpose**: Detect vulnerable dependencies

**Tools**:
- `cargo-audit` - RustSec vulnerability database
- License compliance with REUSE

**Runtime**: ~1-2 minutes

```bash
# Run locally
cargo audit
reuse lint
```

#### 5. Code Coverage (PR/Main Only)

**Purpose**: Track test coverage trends

**Tools**:
- `cargo-llvm-cov` for coverage generation
- Codecov for visualization and tracking

**Triggers**:
- Pull requests to main
- Pushes to main branch

**Runtime**: ~10-12 minutes

```bash
# Run locally
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
```

#### 6. Performance Benchmarks (Main Only)

**Purpose**: Detect performance regressions

**Triggers**:
- Pushes to main branch
- Manual workflow dispatch

**Benchmarks**:
- Parsing performance
- Fingerprinting speed
- Cache efficiency

**Runtime**: ~15-20 minutes

```bash
# Run locally
cargo bench --workspace
```

#### 7. Integration Tests (Main Only)

**Purpose**: Test against real databases

**Infrastructure**:
- Postgres 15 container
- D1 local development

**Runtime**: ~5-8 minutes

---

## Release Pipeline

### Workflow File

`.github/workflows/release.yml`

### Trigger Conditions

**Automated Releases**:
```bash
git tag v0.1.0
git push origin v0.1.0
```

**Manual Releases**:
```yaml
workflow_dispatch:
  inputs:
    version: "0.1.0"
```

### Release Jobs

#### 1. Create GitHub Release

**Responsibilities**:
- Parse version from tag or input
- Generate changelog from `CHANGELOG.md`
- Create GitHub release with notes

**Output**:
- `upload_url` for asset uploads
- `version` for downstream jobs

#### 2. Build CLI Binaries

**Platform Matrix**:

| Platform | Target | Static Linking | Stripped |
|----------|--------|----------------|----------|
| Linux x86_64 | `x86_64-unknown-linux-gnu` | No | Yes |
| Linux x86_64 (static) | `x86_64-unknown-linux-musl` | Yes | Yes |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | No | No |
| macOS x86_64 | `x86_64-apple-darwin` | No | Yes |
| macOS ARM64 | `aarch64-apple-darwin` | No | Yes |
| Windows x86_64 | `x86_64-pc-windows-msvc` | No | No |

**Build Process**:
1. Cross-compilation with `cross` (when needed)
2. Release build with `parallel,caching` features
3. Binary stripping for size reduction
4. Archive creation (`.tar.gz` or `.zip`)
5. Upload to GitHub release

**Artifacts**:
```
thread-0.1.0-x86_64-unknown-linux-gnu.tar.gz
thread-0.1.0-x86_64-unknown-linux-musl.tar.gz
thread-0.1.0-aarch64-unknown-linux-gnu.tar.gz
thread-0.1.0-x86_64-apple-darwin.tar.gz
thread-0.1.0-aarch64-apple-darwin.tar.gz
thread-0.1.0-x86_64-pc-windows-msvc.zip
```

#### 3. Build WASM Package

**Responsibilities**:
- Build optimized WASM for Edge
- Package with TypeScript definitions
- Upload to GitHub release

**Artifacts**:
```
thread-wasm-0.1.0.tar.gz
  ├── thread_wasm_bg.wasm
  ├── thread_wasm.js
  ├── thread_wasm.d.ts
  └── package.json
```

#### 4. Build Docker Images

**Registries**:
- `ghcr.io/knitli/thread` (GitHub Container Registry)

**Platforms**:
- `linux/amd64`
- `linux/arm64`

**Tags**:
- `0.1.0` - Specific version
- `0.1` - Minor version
- `0` - Major version
- `latest` - Latest stable

**Build Strategy**:
- Multi-platform builds with BuildKit
- Layer caching for faster builds
- Optimized image size

#### 5. Publish to crates.io

**Requirements**:
- `CARGO_REGISTRY_TOKEN` secret configured
- Only on tagged releases

**Publication Order** (respecting dependencies):
1. `thread-utils`
2. `thread-language`
3. `thread-ast-engine`
4. `thread-rule-engine`
5. `thread-services`
6. `thread-flow`
7. `thread-wasm`

**Safety**:
- `--allow-dirty` for release builds
- Continue on already published packages

#### 6. Deploy to Cloudflare Edge

**Requirements**:
- `CLOUDFLARE_API_TOKEN` secret
- `CLOUDFLARE_ACCOUNT_ID` secret

**Process**:
1. Build WASM release
2. Deploy with `wrangler`
3. Target production environment

**URL**: `https://thread.knit.li`

---

## Deployment Strategies

### 1. CLI Deployment

#### Local Installation

```bash
# Download latest release
curl -LO https://github.com/knitli/thread/releases/latest/download/thread-VERSION-TARGET.tar.gz

# Extract
tar xzf thread-VERSION-TARGET.tar.gz

# Install
sudo mv thread /usr/local/bin/
```

#### Homebrew (Future)

```bash
brew install knitli/tap/thread
```

#### Cargo Install

```bash
cargo install thread-flow
```

### 2. Edge Deployment (Cloudflare Workers)

#### Production Deployment

```bash
# Build WASM
cargo run -p xtask build-wasm --release

# Deploy to Cloudflare
wrangler deploy --env production
```

#### Staging Deployment

```bash
wrangler deploy --env staging
```

#### Environment Configuration

```toml
# wrangler.toml
[env.production]
name = "thread-production"
route = "thread.knit.li/*"
vars = { ENVIRONMENT = "production" }

[env.staging]
name = "thread-staging"
route = "thread-staging.knit.li/*"
vars = { ENVIRONMENT = "staging" }
```

### 3. Docker Deployment

#### Pull Image

```bash
docker pull ghcr.io/knitli/thread:latest
```

#### Run Container

```bash
docker run -d \
  --name thread \
  -p 8080:8080 \
  -e DATABASE_URL=postgresql://... \
  ghcr.io/knitli/thread:latest
```

#### Docker Compose

```yaml
version: '3.8'

services:
  thread:
    image: ghcr.io/knitli/thread:latest
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgresql://postgres:postgres@db:5432/thread
      - RUST_LOG=info
    depends_on:
      - db

  db:
    image: postgres:15
    environment:
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=postgres
      - POSTGRES_DB=thread
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

---

## Secrets Management

### Required Secrets

| Secret | Purpose | Scope |
|--------|---------|-------|
| `GITHUB_TOKEN` | Automatic, for releases | Repository (auto-provided) |
| `CODECOV_TOKEN` | Coverage reporting | Repository |
| `CARGO_REGISTRY_TOKEN` | crates.io publishing | Repository |
| `CLOUDFLARE_API_TOKEN` | Workers deployment | Environment: production-edge |
| `CLOUDFLARE_ACCOUNT_ID` | Workers deployment | Environment: production-edge |

### Configuration Steps

#### GitHub Repository Secrets

1. Navigate to `Settings` → `Secrets and variables` → `Actions`
2. Add each required secret:

**CODECOV_TOKEN**:
```bash
# Get from https://codecov.io
# Settings → Repository → Upload Token
```

**CARGO_REGISTRY_TOKEN**:
```bash
# Get from https://crates.io/settings/tokens
# Create new token with "publish-update" scope
```

#### GitHub Environment Secrets

1. Navigate to `Settings` → `Environments`
2. Create `production-edge` environment
3. Add environment-specific secrets:

**CLOUDFLARE_API_TOKEN**:
```bash
# Get from Cloudflare Dashboard
# My Profile → API Tokens → Create Token
# Use "Edit Cloudflare Workers" template
```

**CLOUDFLARE_ACCOUNT_ID**:
```bash
# Get from Cloudflare Dashboard
# Workers & Pages → Overview → Account ID
```

---

## Troubleshooting

### Common CI Failures

#### 1. Formatting Failures

**Error**:
```
Diff in .../src/lib.rs at line 42:
```

**Solution**:
```bash
cargo fmt --all
git add .
git commit --amend --no-edit
git push --force
```

#### 2. Clippy Warnings

**Error**:
```
error: this expression creates a reference which is immediately dereferenced
```

**Solution**:
```bash
cargo clippy --fix --workspace --all-features --allow-dirty
```

#### 3. Test Failures

**Error**:
```
test result: FAILED. 14 passed; 1 failed
```

**Solution**:
```bash
# Run specific failing test locally
cargo nextest run --test test_name -- --nocapture

# Debug with logging
RUST_LOG=debug cargo nextest run --test test_name
```

#### 4. WASM Build Failures

**Error**:
```
error: can't find crate for `std`
```

**Solution**:
```bash
# Ensure wasm32 target installed
rustup target add wasm32-unknown-unknown

# Rebuild
cargo run -p xtask build-wasm --release
```

#### 5. Cross-Compilation Failures

**Error**:
```
error: linker `aarch64-linux-gnu-gcc` not found
```

**Solution**:
- Cross-compilation handled by `cross` tool automatically
- Local builds: install target-specific toolchain

### Release Troubleshooting

#### 1. Tag Already Exists

**Error**:
```
error: tag 'v0.1.0' already exists
```

**Solution**:
```bash
# Delete local tag
git tag -d v0.1.0

# Delete remote tag
git push origin :refs/tags/v0.1.0

# Create new tag
git tag v0.1.1
git push origin v0.1.1
```

#### 2. Asset Upload Failures

**Error**:
```
Error uploading asset: 422 Validation Failed
```

**Solution**:
- Check asset name uniqueness
- Verify upload_url is valid
- Ensure release exists before upload

#### 3. Cloudflare Deployment Failures

**Error**:
```
Error: Failed to publish your Function
```

**Solution**:
```bash
# Verify secrets
echo $CLOUDFLARE_API_TOKEN
echo $CLOUDFLARE_ACCOUNT_ID

# Test locally
wrangler deploy --dry-run

# Check wrangler.toml configuration
```

---

## Best Practices

### 1. Branch Strategy

**Main Branch**:
- Protected, requires PR reviews
- All CI checks must pass
- Automatically deployed to production on merge

**Develop Branch**:
- Integration branch for features
- Staging deployments
- Regular merges to main

**Feature Branches**:
- Pattern: `001-feature-name`, `002-fix-bug`
- Short-lived, merged to develop
- Delete after merge

### 2. Commit Conventions

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `perf`: Performance improvement
- `refactor`: Code refactoring
- `test`: Add or update tests
- `docs`: Documentation
- `chore`: Maintenance

**Example**:
```
feat(flow): add D1 target support

Implement Cloudflare D1 database target for Edge deployment.
Includes query result caching and async batch processing.

Closes #42
```

### 3. Version Management

**Semantic Versioning**:
- `MAJOR.MINOR.PATCH` (e.g., `0.1.0`)
- MAJOR: Breaking changes
- MINOR: New features (backward compatible)
- PATCH: Bug fixes

**Release Process**:
1. Update `CHANGELOG.md` with version changes
2. Bump version in `Cargo.toml` files
3. Commit: `chore: bump version to 0.1.0`
4. Tag: `git tag v0.1.0`
5. Push: `git push origin main --tags`

### 4. Testing Strategy

**Unit Tests**:
- Test individual functions and modules
- Fast, isolated, deterministic
- `cargo nextest run --lib`

**Integration Tests**:
- Test component interactions
- Use test databases
- `cargo nextest run --test integration_tests`

**Benchmarks**:
- Track performance over time
- Run on main branch only
- `cargo bench --workspace`

**Coverage Goals**:
- Minimum: 70% line coverage
- Target: 85% line coverage
- Critical paths: 95%+ coverage

### 5. Security Practices

**Dependency Management**:
```bash
# Regular dependency audits
cargo audit

# Update dependencies quarterly
cargo update --workspace
```

**Vulnerability Response**:
1. Security advisory created
2. Patch developed on security branch
3. Expedited review and merge
4. Immediate release with patch version bump

**Secret Rotation**:
- Rotate API tokens annually
- Use environment-specific secrets
- Never commit secrets to repository

### 6. Performance Optimization

**Build Optimization**:
```toml
# Cargo.toml
[profile.release]
lto = true
codegen-units = 1
opt-level = 3
strip = true
```

**Cache Strategy**:
- Use `Swatinem/rust-cache` for dependencies
- Cache build artifacts across jobs
- Invalidate on `Cargo.lock` changes

**Parallel Execution**:
- Use `cargo nextest` for parallel testing
- Matrix builds run concurrently
- Fail-fast strategy for quick feedback

---

## Metrics and Monitoring

### CI/CD Metrics

**Build Times**:
- Quick checks: 2-3 minutes
- Full test suite: 8-15 minutes per platform
- Release builds: 20-30 minutes total
- Docker builds: 5-10 minutes

**Success Rates** (Target: >95%):
- Main branch CI: 98%+
- PR builds: 95%+
- Release builds: 99%+

**Coverage Trends**:
- Track via Codecov
- Review monthly
- Address declining coverage

### Deployment Metrics

**Deployment Frequency**:
- CLI releases: Monthly
- Edge updates: Weekly
- Hotfixes: As needed

**Mean Time to Recovery** (Target: <30 minutes):
- Revert deployment
- Rollback release
- Patch critical bugs

**Change Failure Rate** (Target: <5%):
- Track failed deployments
- Root cause analysis
- Process improvements

---

## Resources

### Documentation

- [GitHub Actions Docs](https://docs.github.com/en/actions)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Cloudflare Workers](https://developers.cloudflare.com/workers/)
- [REUSE Specification](https://reuse.software/spec/)

### Tools

- `cargo-nextest` - Fast test runner
- `cargo-llvm-cov` - Coverage tool
- `cargo-audit` - Security auditing
- `cross` - Cross-compilation
- `wrangler` - Cloudflare Workers CLI

### Support

- **Issues**: https://github.com/knitli/thread/issues
- **Discussions**: https://github.com/knitli/thread/discussions
- **Security**: security@knit.li

---

**Last Updated**: 2026-01-28
**Maintained By**: Thread Development Team
**Review Cycle**: Quarterly
