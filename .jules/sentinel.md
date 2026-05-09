## 2026-05-09 - Prevent Path Traversal in Manual Path Resolution
**Vulnerability:** The TypeScript dependency extractor (`crates/flow/src/incremental/extractors/typescript.rs`) manually normalized paths without preventing `ParentDir` components from popping `RootDir` or `Prefix`, allowing paths to escape the intended directory.
**Learning:** Manual path normalization (e.g., when `canonicalize()` fails) must strictly handle standard library boundaries like `RootDir` and `Prefix` to prevent path traversal outside of absolute or project boundaries.
**Prevention:** Always check if the path component being popped represents a boundary, or use robust external path normalization libraries instead of rolling custom loops with `std::path::Component`.
