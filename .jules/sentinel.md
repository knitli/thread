## 2024-05-27 - Path Traversal in Custom Path Normalization
**Vulnerability:** Path normalization logic in `typescript.rs` unconditionally called `components.pop()` when encountering a `ParentDir` (`..`).
**Learning:** This implementation flaw allowed dropping `RootDir` (`/`) or `Prefix` (`C:\`), and mishandled sequential `..` segments when the components list was empty, potentially allowing path resolution outside expected scopes.
**Prevention:** When manually resolving paths with `std::path::Component`, explicitly check the last component before popping. Ignore `..` if the last element is root/prefix, push `..` if the list is empty or the last element is also `..`, and only pop otherwise.
