
## 2025-05-07 - Path Traversal in Manual Path Resolution
**Vulnerability:** In `crates/flow/src/incremental/extractors/typescript.rs`, manual path resolution for imports popping `std::path::Component::ParentDir` allowed bypassing the base directory when resolving modules, potentially allowing arbitrary file access.
**Learning:** Using a naive `components.pop()` when encountering a `..` (`ParentDir`) component is unsafe because it can pop `RootDir` or `Prefix` components, or discard consecutive `..` components when reconstructing paths, leading to path traversal.
**Prevention:** Always validate path components explicitly. Block or correctly preserve `ParentDir` when the current components list is empty or already ends in a `ParentDir`, and never allow `RootDir` or `Prefix` components to be popped by relative path navigation.
