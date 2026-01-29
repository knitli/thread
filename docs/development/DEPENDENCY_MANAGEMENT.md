# Dependency Management Guide

**Version**: 1.0
**Last Updated**: 2026-01-28

---

## Table of Contents

- [Overview](#overview)
- [Dependency Policy](#dependency-policy)
- [Security Scanning](#security-scanning)
- [Update Strategy](#update-strategy)
- [License Compliance](#license-compliance)
- [Best Practices](#best-practices)

---

## Overview

Thread uses Cargo for dependency management with strict policies for security, licensing, and version control.

### Dependency Philosophy

1. **Minimize Dependencies**: Only add dependencies that provide significant value
2. **Security First**: All dependencies must pass security audits
3. **License Compliance**: Only compatible licenses (MIT, Apache-2.0, BSD)
4. **Stability**: Prefer stable, well-maintained crates
5. **Performance**: Consider binary size and compile time impact

### Current Dependencies

**Production Dependencies**: ~20 direct dependencies
**Development Dependencies**: ~15 dev-only dependencies
**Total Crate Count**: ~150 including transitive dependencies

---

## Dependency Policy

### Adding New Dependencies

**Before Adding**:

1. **Evaluate Necessity**:
   - Can the functionality be implemented internally?
   - Is there a lighter alternative?
   - Does it provide significant value?

2. **Security Check**:
   ```bash
   # Add dependency
   cargo add <crate-name>

   # Immediate security audit
   cargo audit

   # Check for known issues
   cargo deny check all
   ```

3. **License Verification**:
   ```bash
   # Check license compatibility
   cargo license | grep <crate-name>

   # Verify no GPL/AGPL
   cargo deny check licenses
   ```

4. **Maintenance Assessment**:
   - Last release within 12 months
   - Active maintainer(s)
   - Reasonable issue response time
   - CI/CD in place

5. **Impact Analysis**:
   ```bash
   # Check compile time impact
   cargo build --timings

   # Check binary size impact
   cargo bloat --release
   ```

**Required Documentation**:

When adding a dependency, document in PR:
```markdown
## Dependency Addition: <crate-name>

**Purpose**: <why this dependency is needed>
**Alternatives Considered**: <other options>
**License**: <license type>
**Maintenance**: Last release <date>, <number> contributors
**Security**: cargo-audit clean, no known CVEs
**Impact**: +<X>KB binary size, +<Y>s compile time
```

### Dependency Categories

#### Core Dependencies

**Criteria**:
- Used across multiple crates
- Fundamental to functionality
- Stable API
- Strong maintenance

**Examples**:
- `serde` - Serialization
- `tokio` - Async runtime
- `tree-sitter` - AST parsing

**Review Frequency**: Quarterly

#### Feature Dependencies

**Criteria**:
- Optional features
- Can be disabled
- Feature-gated

**Examples**:
- `rayon` - Parallel processing (optional)
- `moka` - Caching (optional)

**Review Frequency**: Semi-annually

#### Development Dependencies

**Criteria**:
- Testing and benchmarking only
- Not in production builds
- Can be more lenient

**Examples**:
- `criterion` - Benchmarking
- `cargo-nextest` - Testing

**Review Frequency**: Annually

---

## Security Scanning

### Automated Scanning

**Daily Scans** (via GitHub Actions):
```yaml
# .github/workflows/security.yml
schedule:
  - cron: '0 2 * * *'  # 2 AM UTC daily
```

**PR Scans**:
- Triggered on `Cargo.lock` changes
- Blocks merge if vulnerabilities found
- Dependency review action

### Manual Scanning

```bash
# Full security audit
cargo audit

# Check for advisories
cargo deny check advisories

# Check with custom config
cargo audit --file Cargo.lock --deny warnings
```

### Vulnerability Response

**Critical Vulnerabilities** (CVSS ≥9.0):
1. **Immediate**: Alert security team
2. **Within 24h**: Assess impact and exploitability
3. **Within 72h**: Patch or mitigate
4. **Within 7d**: Release patched version

**High Vulnerabilities** (CVSS 7.0-8.9):
1. **Within 48h**: Assess and prioritize
2. **Within 14d**: Patch or mitigate
3. **Within 30d**: Release patched version

**Medium/Low Vulnerabilities**:
1. **Within 7d**: Assess
2. **Within 30-90d**: Address in regular release cycle

### Exemptions

Some vulnerabilities may be exempt if:
- Not applicable to our use case
- No patch available and risk is acceptable
- Working on alternative solution

**Document exemptions**:
```toml
# .cargo/audit.toml
[advisories]
ignore = [
    "RUSTSEC-YYYY-NNNN",  # Reason: Not exploitable in our usage
]
```

---

## Update Strategy

### Update Frequency

**Patch Updates** (0.1.x → 0.1.y):
- **Security patches**: Immediate
- **Bug fixes**: Weekly
- **Performance improvements**: Bi-weekly

**Minor Updates** (0.x.0 → 0.y.0):
- **Regular updates**: Monthly
- **After testing**: 1-2 week soak period

**Major Updates** (x.0.0 → y.0.0):
- **Planned updates**: Quarterly
- **Thorough testing**: 4-6 week testing period
- **Migration guide required**

### Update Process

**1. Check for Updates**:
```bash
# List outdated dependencies
cargo outdated

# Check specific crate
cargo outdated -p <crate-name>
```

**2. Create Update Branch**:
```bash
git checkout -b deps/update-<crate-name>
```

**3. Update Dependencies**:
```bash
# Update specific dependency
cargo update -p <crate-name>

# Update all patch versions
cargo update

# Update to latest compatible version
cargo upgrade  # requires cargo-edit
```

**4. Test Thoroughly**:
```bash
# Run full test suite
cargo nextest run --all-features

# Run benchmarks
cargo bench --workspace

# Build all targets
cargo build --all-targets --all-features
```

**5. Verify Security**:
```bash
cargo audit
cargo deny check all
```

**6. Document Changes**:
```markdown
## Dependency Update: <crate-name> <old> → <new>

**Type**: [Security/Bug Fix/Feature/Breaking]
**Changes**: <link to changelog>
**Testing**: All tests pass, benchmarks stable
**Security**: cargo-audit clean
```

**7. Create PR**:
```bash
git add Cargo.toml Cargo.lock
git commit -m "deps: update <crate-name> to <version>"
git push origin deps/update-<crate-name>
```

### Cargo.lock Management

**When to Commit**:
- ✅ Always commit for applications
- ✅ Always commit for workspaces
- ❌ Don't commit for libraries (optional)

**Lock File Hygiene**:
```bash
# Update minimal versions
cargo update --dry-run

# Check for duplicate dependencies
cargo tree --duplicates

# Clean up dependencies
cargo tree --invert <crate-name>
```

---

## License Compliance

### Acceptable Licenses

**Permissive** (Preferred):
- MIT
- Apache-2.0
- BSD-2-Clause
- BSD-3-Clause
- ISC

**Weak Copyleft** (Acceptable):
- MPL-2.0 (specific cases)

**Strong Copyleft** (Not Acceptable):
- GPL-3.0
- AGPL-3.0
- GPL-2.0 (without linking exception)

### License Checking

**Automated Checks**:
```bash
# Check all licenses
cargo license

# Check for incompatible licenses
cargo deny check licenses

# Generate license report
cargo license --json > licenses.json
```

**CI/CD Integration**:
```yaml
# Runs on all PRs
- name: License Check
  run: cargo deny check licenses
```

### Dual Licensing

Thread is dual-licensed under:
- MIT OR Apache-2.0

**Requirements**:
- All dependencies must be compatible with both
- Vendored code retains original licenses
- Attribution maintained in `VENDORED.md`

### License Attribution

**REUSE Compliance**:
```bash
# Check REUSE compliance
reuse lint

# Add license headers
reuse addheader --license MIT --copyright "Knitli Inc." file.rs
```

---

## Best Practices

### Dependency Pinning

**Don't Pin** (allow updates within semver range):
```toml
[dependencies]
serde = "1.0"  # ✅ Allows 1.x updates
tokio = "1.35" # ✅ Allows 1.35.x updates
```

**Do Pin** (exact version for critical dependencies):
```toml
[dependencies]
critical-crate = "=1.2.3"  # ⚠️ Only when necessary
```

### Feature Flags

**Minimize Default Features**:
```toml
[dependencies]
serde = { version = "1.0", default-features = false, features = ["derive"] }
```

**Optional Dependencies**:
```toml
[dependencies]
rayon = { version = "1.8", optional = true }

[features]
parallel = ["dep:rayon"]
```

### Platform-Specific Dependencies

```toml
[target.'cfg(unix)'.dependencies]
libc = "0.2"

[target.'cfg(windows)'.dependencies]
winapi = "0.3"
```

### Avoiding Dependency Hell

**Check Duplicate Versions**:
```bash
# Find duplicates
cargo tree --duplicates

# Investigate specific crate
cargo tree --invert serde
```

**Unify Versions**:
```toml
[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }

[dependencies]
serde = { workspace = true }
```

### Binary Size Optimization

**Profile Configuration**:
```toml
[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Single codegen unit
strip = true        # Strip symbols
```

**Feature Selection**:
```bash
# Build with minimal features
cargo build --release --no-default-features

# Check size impact
cargo bloat --release --crates
```

### Compile Time Optimization

**Workspace Configuration**:
```toml
[profile.dev]
incremental = true

[profile.dev.package."*"]
opt-level = 1  # Optimize dependencies in dev
```

**Caching**:
```bash
# Use sccache for faster builds
cargo install sccache
export RUSTC_WRAPPER=sccache
```

---

## Tools and Commands

### Essential Tools

```bash
# Install dependency management tools
cargo install cargo-audit      # Security audits
cargo install cargo-deny       # Policy enforcement
cargo install cargo-outdated   # Check for updates
cargo install cargo-edit       # Edit Cargo.toml
cargo install cargo-license    # License checking
cargo install cargo-bloat      # Binary size analysis
cargo install cargo-geiger     # Unsafe code detection
```

### Common Commands

**Security**:
```bash
cargo audit                    # Security audit
cargo audit --fix             # Apply security fixes
cargo deny check all          # Full policy check
cargo geiger                  # Find unsafe code
```

**Updates**:
```bash
cargo outdated                # List outdated deps
cargo outdated --workspace    # Workspace-wide check
cargo update                  # Update Cargo.lock
cargo upgrade                 # Upgrade versions
```

**Analysis**:
```bash
cargo tree                    # Dependency tree
cargo tree --duplicates       # Find duplicates
cargo bloat --release         # Size analysis
cargo build --timings         # Compile time analysis
```

**Licensing**:
```bash
cargo license                 # List licenses
cargo license --json          # JSON output
reuse lint                    # REUSE compliance
```

---

## Dependency Review Checklist

Before merging PR with dependency changes:

- [ ] Security audit passes (`cargo audit`)
- [ ] License check passes (`cargo deny check licenses`)
- [ ] No new duplicate dependencies
- [ ] Binary size impact acceptable
- [ ] Compile time impact acceptable
- [ ] All tests pass
- [ ] Benchmarks stable or improved
- [ ] Documentation updated if needed
- [ ] Changelog entry added
- [ ] Alternative solutions considered

---

## Emergency Procedures

### Critical Vulnerability Found

**1. Immediate Actions**:
```bash
# Verify vulnerability
cargo audit

# Check affected versions
cargo tree --invert <vulnerable-crate>

# Assess exploitability in our context
```

**2. Mitigation Options**:

**Option A - Update**:
```bash
cargo update -p <vulnerable-crate>
cargo test --all-features
```

**Option B - Patch**:
```toml
[patch.crates-io]
vulnerable-crate = { git = "https://github.com/maintainer/repo", branch = "security-fix" }
```

**Option C - Replace**:
```bash
# Find alternative
cargo search <functionality>

# Replace and test
cargo add <alternative>
cargo remove <vulnerable-crate>
```

**3. Release Procedure**:
```bash
# Bump patch version
# Update CHANGELOG.md with security fix
# Create security advisory
# Release immediately
```

### Dependency Disappeared

**1. Verify**:
```bash
cargo build  # Will fail if dependency unavailable
```

**2. Options**:

**Vendoring**:
```bash
cargo vendor
```

**Fork and Maintain**:
```bash
# Fork repository
# Update dependency to use fork
git = "https://github.com/your-org/forked-repo"
```

**Replace**:
```bash
# Find alternative
# Update and test thoroughly
```

---

## Resources

### Documentation

- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [RustSec Advisory Database](https://rustsec.org/)
- [SPDX License List](https://spdx.org/licenses/)
- [REUSE Specification](https://reuse.software/spec/)

### Tools

- [cargo-audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit)
- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny)
- [cargo-outdated](https://github.com/kbknapp/cargo-outdated)
- [cargo-edit](https://github.com/killercup/cargo-edit)

### References

- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [OWASP Dependency Check](https://owasp.org/www-project-dependency-check/)

---

**Last Updated**: 2026-01-28
**Review Cycle**: Quarterly
**Next Review**: 2026-04-28
