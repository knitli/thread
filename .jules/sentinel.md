
## 2024-05-20 - Lexical Path Normalization Traversal Vulnerability
**Vulnerability:** In `crates/flow/src/incremental/extractors/typescript.rs`, manual path normalization handled `..` components by blindly calling `components.pop()`.
**Learning:** If `components` was empty or contained boundary components like `/` (`RootDir`), popping did nothing, effectively deleting `..` directories and breaking relative navigation. Alternatively, `components.pop()` would happily strip root indicators, allowing path ascensions to escape intended root scopes.
**Prevention:** When manually normalizing paths using `std::path::Component`, explicitly block `ParentDir` from popping `RootDir` or `Prefix`. If the components stack is empty or its top is already `ParentDir`, push the new `ParentDir` to preserve valid ascensions (e.g., `../../file`).
