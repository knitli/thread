# Thread Development Commands

## Build Commands

### Standard Builds
```bash
# Build all crates (except WASM)
mise run build
# or: cargo build --workspace

# Release build
mise run build-release
# or: cargo build --workspace --release --features inline
```

### WASM Builds
```bash
# Development WASM (Cloudflare Workers target)
mise run build-wasm
# or: cargo run -p xtask build-wasm

# WASM with multi-threading (browser target)
mise run build-wasm-browser-dev
# or: cargo run -p xtask build-wasm --multi-threading

# Release WASM (optimized for size)
mise run build-wasm-release
# or: cargo run -p xtask build-wasm --release

# WASM with profiling
mise run build-wasm-profile
# or: cargo run -p xtask build-wasm --profiling
```

## Testing Commands

```bash
# Run all tests
mise run test
# or: hk run test
# or: cargo nextest run --all-features --no-fail-fast -j 1

# Run specific test
cargo nextest run --manifest-path Cargo.toml test_name --all-features

# Run tests for specific crate
cargo nextest run -p thread-ast-engine --all-features

# Run benchmarks
cargo bench -p thread-rule-engine
```

## Linting and Formatting

```bash
# Full linting (runs all checks)
mise run lint
# or: hk run check

# Auto-fix formatting and linting issues
mise run fix
# or: hk fix

# Update license headers
mise run update-licenses
# or: ./scripts/update-licenses.py
```

## CI Pipeline

```bash
# Run complete CI pipeline locally
mise run ci
# This runs: build → lint → test
```

## Maintenance Commands

```bash
# Update dependencies
mise run update
# or: cargo update && cargo update --workspace

# Clean build artifacts
mise run clean

# Update development tools
mise run update-tools

# Install development tools and hooks
mise run install-tools
```

## Common Cargo Commands

```bash
# Check code without building
cargo check --workspace

# Run clippy linter
cargo clippy --workspace --all-features

# Format code
cargo fmt --all

# View documentation
cargo doc --open

# Audit dependencies for security vulnerabilities
cargo audit

# Check for license compliance
cargo deny check
```

## Quick Aliases (via mise.toml)
- `mise run b` = build
- `mise run br` = build-release
- `mise run bw` = build-wasm
- `mise run bwr` = build-wasm-release
- `mise run t` = test
- `mise run c` = lint/check
- `mise run f` = fix
