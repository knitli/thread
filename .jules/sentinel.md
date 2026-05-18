## 2024-05-24 - [Path Traversal in Manual Path Resolution]
**Vulnerability:** Path traversal via manual `..` (ParentDir) path normalization blindly popping the last component without checking if it's the root directory or if the path is already empty.
**Learning:** `std::path::PathBuf::components()` lexical resolution requires explicit handling of `Component::ParentDir` to avoid popping `Component::RootDir` or losing preceding `..` directives when resolving relative paths that point outside the base directory.
**Prevention:** Always check `components.last()` when popping. If it's a `RootDir` or `Prefix`, do not pop. If the stack is empty or the last item is also `ParentDir`, push the `ParentDir` component instead of ignoring it.
