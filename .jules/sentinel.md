## 2024-05-18 - [CRITICAL] Path Traversal in TypeScript Module Resolution
**Vulnerability:** Path traversal existed in `resolve_module_path` (`crates/flow/src/incremental/extractors/typescript.rs`) where `std::path::Component::ParentDir` resolution manually popped components without ensuring it did not cross the root directory.
**Learning:** During manual path canonicalization, `Vec::pop` on `std::path::Components` can silently succeed when popping out of bounds or explicitly pop `RootDir`, allowing arbitrary traversal (`../../etc/passwd`).
**Prevention:** Explicitly match the last component to ensure it is not `RootDir` or `Prefix`, and fail safely by returning an error instead of continuing to resolve paths.
