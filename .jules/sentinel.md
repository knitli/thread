## 2025-02-24 - Path Traversal in Path Normalization
**Vulnerability:** Path traversal vulnerability in TypeScript module resolution where resolving paths with multiple `..` components could pop `RootDir` or `Prefix` components due to indiscriminate `components.pop()`.
**Learning:** Manual path normalization logic using `std::path::Component` needs to handle edge cases explicitly, such as stopping `ParentDir` components from popping root or prefix components, and correctly handling multiple `..` components when at the root or start of a relative path.
**Prevention:** Always check `components.last()` when popping for `ParentDir` and preserve root, prefix, and leading `ParentDir` components. Use established and safe normalization utilities where possible.
