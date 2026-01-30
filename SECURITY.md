<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Security Policy

**Version**: 1.0
**Last Updated**: 2026-01-28

---

## Supported Versions

We actively support and provide security updates for the following versions of Thread:

| Version | Supported          | End of Support |
| ------- | ------------------ | -------------- |
| 0.1.x   | :white_check_mark: | TBD            |
| < 0.1   | :x:                | Immediately    |

**Support Policy**:
- Latest minor version receives security patches
- Previous minor version receives critical security patches for 3 months after new release
- Major versions receive security support for 12 months after new major release

---

## Reporting a Vulnerability

We take security vulnerabilities seriously and appreciate responsible disclosure.

### How to Report

**DO NOT** create public GitHub issues for security vulnerabilities.

Instead, please report security issues to:

**Email**: security@knit.li

**Include in your report**:
1. Description of the vulnerability
2. Steps to reproduce the issue
3. Potential impact and severity assessment
4. Suggested remediation (if available)
5. Your contact information for follow-up

### What to Expect

1. **Acknowledgment**: Within 24 hours of submission
2. **Initial Assessment**: Within 72 hours
3. **Status Update**: Weekly updates on progress
4. **Resolution Timeline**:
   - **Critical**: 7 days
   - **High**: 14 days
   - **Medium**: 30 days
   - **Low**: 90 days

### Disclosure Process

1. **Coordinated Disclosure**: We follow a 90-day disclosure timeline
2. **Security Advisory**: Published on GitHub Security Advisories
3. **CVE Assignment**: Requested for critical and high severity issues
4. **Credit**: Security researchers will be credited unless they prefer to remain anonymous

---

## Security Measures

### Code Security

**Static Analysis**:
- Automated scanning with Semgrep SAST
- Clippy linting with security rules
- Regular security audits

**Dependency Management**:
- Daily automated vulnerability scanning with `cargo-audit`
- Dependency review on all pull requests
- License compliance checks
- Supply chain security with `cargo-deny`

**Code Review**:
- All code changes require review
- Security-sensitive changes require security team review
- Automated checks must pass before merge

### Build Security

**CI/CD Security**:
- Signed releases with checksums
- Reproducible builds
- Minimal build dependencies
- Isolated build environments

**Artifact Verification**:
```bash
# Verify release checksum
sha256sum -c thread-0.1.0-checksums.txt

# Verify with GPG (when available)
gpg --verify thread-0.1.0.tar.gz.sig
```

### Runtime Security

**Sandboxing**:
- Minimal required permissions
- Process isolation where applicable
- Secure defaults for all features

**Data Protection**:
- No credentials stored in logs
- Secure credential handling
- Encrypted data transmission

### Infrastructure Security

**Access Control**:
- Multi-factor authentication required
- Least privilege access model
- Regular access reviews

**Secrets Management**:
- Environment-based secrets
- No secrets in version control
- Regular secret rotation

---

## Security Best Practices

### For Users

**Installation**:
```bash
# Verify download authenticity
curl -LO https://github.com/knitli/thread/releases/latest/download/thread-0.1.0-x86_64-unknown-linux-gnu.tar.gz
sha256sum thread-0.1.0-x86_64-unknown-linux-gnu.tar.gz

# Install from trusted sources only
cargo install thread-flow  # From crates.io
# or
brew install knitli/tap/thread  # From official tap
```

**Configuration**:
- Use environment variables for sensitive configuration
- Never commit credentials to version control
- Rotate database credentials regularly
- Use read-only database users where possible

**Network Security**:
- Use TLS for all database connections
- Enable SSL mode for PostgreSQL: `?sslmode=require`
- Implement firewall rules for database access
- Use private networks for database connections

### For Contributors

**Development Security**:
- Run security checks before committing:
  ```bash
  cargo audit
  cargo clippy -- -D warnings
  ```

- Never commit:
  - API keys or credentials
  - Private keys or certificates
  - Database connection strings with passwords
  - `.env` files with secrets

- Use pre-commit hooks:
  ```bash
  hk install  # Install git hooks
  ```

**Dependency Updates**:
- Review `cargo update` changes carefully
- Check for security advisories before updating
- Test thoroughly after dependency updates

---

## Known Security Considerations

### Database Connections

**PostgreSQL**:
- Use connection pooling with reasonable limits
- Implement query timeouts
- Sanitize user input (handled by sqlx)
- Use prepared statements (default with sqlx)

**D1 (Cloudflare)**:
- Rate limiting applied automatically
- Row limits enforced
- Sandboxed execution environment

### Edge Deployment

**WASM Sandboxing**:
- Limited system access
- No filesystem access
- Memory limits enforced
- CPU time limits

**Cloudflare Workers Security**:
- Isolated V8 contexts
- Automatic DDoS protection
- Built-in rate limiting
- Secure execution environment

### CLI Deployment

**System Access**:
- File system access as configured
- Network access as configured
- Runs with user permissions
- Systemd service isolation (recommended)

---

## Security Advisories

### Active Advisories

Currently no active security advisories.

### Past Advisories

None at this time.

### Subscribe to Advisories

- **GitHub**: Watch repository → Custom → Security alerts
- **Email**: Subscribe to security@knit.li mailing list
- **RSS**: https://github.com/knitli/thread/security/advisories.atom

---

## Vulnerability Response SLA

| Severity | Response Time | Patch Release | Communication |
|----------|---------------|---------------|---------------|
| **Critical** | 24 hours | 7 days | Immediate advisory |
| **High** | 48 hours | 14 days | Security advisory |
| **Medium** | 1 week | 30 days | Release notes |
| **Low** | 2 weeks | 90 days | Release notes |

**Severity Criteria**:

- **Critical**: Remote code execution, privilege escalation, data breach
- **High**: Authentication bypass, significant data exposure, DoS
- **Medium**: Information disclosure, limited DoS, CSRF
- **Low**: Minor information leaks, theoretical attacks

---

## Security Audit History

| Date | Type | Auditor | Report |
|------|------|---------|--------|
| TBD | External Security Audit | TBD | TBD |

---

## Compliance

### Standards

- **OWASP Top 10**: Addressed in design and implementation
- **CWE Top 25**: Mitigated through secure coding practices
- **SANS Top 25**: Covered by security controls

### Certifications

- **SOC 2**: Planned for future
- **ISO 27001**: Planned for future

---

## Security Tools

### Recommended Tools

**For Development**:
- `cargo-audit` - Vulnerability scanning
- `cargo-deny` - Supply chain security
- `cargo-outdated` - Dependency updates
- `cargo-geiger` - Unsafe code detection

**For Operations**:
- `fail2ban` - Intrusion prevention
- `ufw` - Firewall configuration
- `Let's Encrypt` - TLS certificates
- `Vault` - Secret management

### Installation

```bash
# Install security tooling
cargo install cargo-audit cargo-deny cargo-outdated cargo-geiger

# Run security checks
cargo audit
cargo deny check all
cargo geiger
```

---

## Contact

- **Security Issues**: security@knit.li
- **General Questions**: support@knit.li
- **Bug Reports**: https://github.com/knitli/thread/issues (non-security)

---

## Acknowledgments

We would like to thank the following security researchers for responsibly disclosing vulnerabilities:

(None at this time)

---

**Responsible Disclosure**: We are committed to working with security researchers through coordinated disclosure. Thank you for helping keep Thread and our users safe.

---

**Last Updated**: 2026-01-28
**Next Review**: 2026-04-28 (Quarterly)
