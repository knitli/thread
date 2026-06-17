## 2024-05-24 - Pre-canonicalize Base Path to Prevent TOCTOU Bypass
**Vulnerability:** Path traversal bypass in `FileSystemContext`. A physical check against symlink escapes was entirely dependent on `self.base_path.canonicalize()` succeeding within `secure_path`. If canonicalization failed, the check was silently skipped.
**Learning:** Security checks that depend on file system operations like canonicalization must be performed at initialization, not lazily during use, to avoid Time-of-Check to Time-of-Use (TOCTOU) issues and silent bypasses.
**Prevention:** Canonicalize the base path during constructor initialization (`FileSystemContext::new`) and fail fast if it is invalid, returning a `Result`. Subsequent security checks can then rely on the pre-canonicalized path.
