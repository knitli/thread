
## $(date +%Y-%m-%d) - Path Traversal Bypass in Manual Normalization
**Vulnerability:** Path traversal (`../../`) bypass during custom path normalization using `std::path::Component::ParentDir`.
**Learning:** Naively calling `.pop()` on a `Vec<Component>` for `ParentDir` silently swallows consecutive `..` components in relative paths and can maliciously pop root/prefix components (turning absolute paths into relative).
**Prevention:** When manually resolving paths with `std::path::Components`, explicitly check the `last()` component. Ignore `ParentDir` if the last component is `RootDir` or `Prefix`. If the list is empty or ends in `ParentDir`, the current `ParentDir` must be pushed to correctly construct paths like `../../a`.
