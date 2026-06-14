## 2024-05-24 - [Fix Path Traversal in TypeScript Extractor]
**Vulnerability:** Path traversal vulnerability during manual path resolution in `crates/flow/src/incremental/extractors/typescript.rs`.
**Learning:** Manual path normalization (popping `ParentDir`) can incorrectly eliminate `../` when the path is relative or outside the base directory, allowing traversal attacks if relative references exceed the current depth.
**Prevention:** Explicitly block `Component::ParentDir` from popping `Component::RootDir` or `Component::Prefix`, and push `Component::ParentDir` if the components list is empty or ends in `Component::ParentDir`.
