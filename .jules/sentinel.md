
## 2024-05-25 - Path Traversal in TypeScript Extractor
**Vulnerability:** The manual path resolution logic in `crates/flow/src/incremental/extractors/typescript.rs` naively popped the last component when encountering `..` (`std::path::Component::ParentDir`). This allowed escaping the root directory by popping `RootDir` or `Prefix`, and failed to handle relative paths like `../../a` correctly.
**Learning:** `std::path::Path::components()` and `Vec::pop()` are not sufficient for secure path normalization. Popping `RootDir` effectively makes an absolute path relative, and popping an empty list discards `..` segments that might be necessary for relative traversal.
**Prevention:** Always check `components.last()` before popping. Prevent popping if the last component is `RootDir` or `Prefix`. If the list is empty or the last component is already `ParentDir`, push the new `ParentDir` to preserve correct relative path structure.
