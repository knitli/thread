## 2024-05-20 - Pre-canonicalize base path in FileSystemContext to avoid symlink escapes
**Vulnerability:** The `FileSystemContext::new` accepted an uncanonicalized base path, which could potentially lead to symlink escapes during path validation.
**Learning:** By canonicalizing the base path during construction, the check inside `secure_path` accurately determines if the target escapes the expected path, which wasn't fully guaranteed with an uncanonicalized base path.
**Prevention:** Always construct secure contexts with canonicalized paths when implementing file path validation checks, especially to prevent symlink traversal.
