## 2025-03-09 - Path Traversal Vulnerability in `Component::ParentDir` normalization

**Vulnerability:**
The `resolve_module_path` method in the TypeScript dependency extractor was vulnerable to path traversal. When normalizing paths using `.components()`, encountering `std::path::Component::ParentDir` caused an unconditional `.pop()` on the accumulated components vector, allowing arbitrary paths to escape the root directory or prefix if enough `..` sequences were provided.

**Learning:**
Naive manual path normalization in Rust that indiscriminately pops elements upon encountering `..` is unsafe. It allows attackers or untrusted paths to bypass base directory restrictions by traversing "above" the root directory.

**Prevention:**
When manually resolving paths using `std::path::Component`, explicitly check the end of the current component list before popping. If the last component is `Component::RootDir` or `Component::Prefix`, do not pop it. Furthermore, correctly handle relative path preservation by pushing `Component::ParentDir` if the current list is empty or also ends in `Component::ParentDir`.
