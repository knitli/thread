# Task Completion Checklist

When completing a development task in Thread, ensure you follow this checklist:

## 1. Code Quality

### Formatting
```bash
# Auto-fix formatting issues
mise run fix
# or: hk fix
# This runs: cargo fmt, typos fix, reuse fix, yamlfmt, taplo
```

### Linting
```bash
# Run all linters
mise run lint
# or: hk run check
# This runs: cargo clippy, cargo check, cargo deny, actionlint, typos, reuse
```

### Manual Checks
- [ ] No compiler warnings
- [ ] No clippy warnings (except explicitly allowed ones)
- [ ] Code follows naming conventions (snake_case, PascalCase)
- [ ] Proper error handling (no unwraps in production code)

## 2. Testing

### Run Tests
```bash
# Run all tests
mise run test
# or: hk run test
# or: cargo nextest run --all-features --no-fail-fast -j 1
```

### Test Coverage
- [ ] New functionality has unit tests
- [ ] Integration tests added if needed
- [ ] Edge cases covered
- [ ] All tests pass

## 3. Documentation

### Code Documentation
- [ ] Public APIs have rustdoc comments
- [ ] Examples in rustdoc if helpful
- [ ] Module-level documentation updated

### Project Documentation
- [ ] README.md updated if public API changed
- [ ] CLAUDE.md updated if workflow changed
- [ ] CHANGELOG.md updated (if applicable)

## 4. License Compliance

### License Headers
```bash
# Update license headers on all files
mise run update-licenses
# or: ./scripts/update-licenses.py
```

### Verification
- [ ] All source files have SPDX license headers
- [ ] Proper SPDX identifiers used (AGPL-3.0-or-later or MIT/Apache-2.0)
- [ ] REUSE compliance verified: `reuse lint`

## 5. Dependencies

### Dependency Updates
```bash
# If you added/updated dependencies
mise run update
# or: cargo update && cargo update --workspace
```

### Verification
- [ ] No unnecessary dependencies added
- [ ] Dependencies use workspace versions where possible
- [ ] Security audit passes: `cargo audit`
- [ ] License compliance passes: `cargo deny check`

## 6. Build Verification

### Standard Build
```bash
# Verify standard build works
mise run build
# or: cargo build --workspace
```

### Release Build
```bash
# Verify release build (if significant changes)
mise run build-release
# or: cargo build --workspace --release --features inline
```

### WASM Build (if applicable)
```bash
# Verify WASM build (if WASM crate changed)
mise run build-wasm
# or: cargo run -p xtask build-wasm
```

## 7. Git Workflow

### Before Committing
- [ ] Changes are on a feature branch (not main/master)
- [ ] Unnecessary files not staged (no debug artifacts)
- [ ] Meaningful commit message prepared
- [ ] Git hooks will pass (pre-commit checks)

### Commit Process
```bash
# Review changes
git diff

# Stage changes
git add <files>

# Commit (pre-commit hooks will run automatically)
git commit -m "descriptive message"
```

## 8. CI Pipeline

### Local CI Verification
```bash
# Run complete CI pipeline locally
mise run ci
# This runs: build → lint → test
```

### Before Creating PR
- [ ] Local CI passes completely
- [ ] No test failures
- [ ] No linting errors
- [ ] Build succeeds
- [ ] WASM build succeeds (if WASM changed)

## Quick Reference

**Before starting work:**
```bash
git checkout -b feature/my-feature
mise run install-tools  # If first time
```

**During development:**
```bash
mise run build  # Build
mise run test   # Test
mise run fix    # Auto-fix issues
```

**Before committing:**
```bash
mise run ci              # Full CI check
mise run update-licenses # License headers
git add .
git commit -m "feat: description"
```

**After commit:**
```bash
git push origin feature/my-feature
# Create PR via GitHub
```
