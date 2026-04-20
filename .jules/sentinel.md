## 2026-01-28 - Path Traversal Vulnerability in Component Resolution
**Vulnerability:** Path traversal possible through careless handling of `ParentDir` when resolving paths during `resolve_module_path` in `TypeScriptDependencyExtractor`.
**Learning:** Calling `components.pop()` when encountering `Component::ParentDir` on empty vectors or trailing `ParentDir`s leads to improper path normalization, resulting in escapes from intended directories.
**Prevention:** Avoid blindly calling `components.pop()` for `ParentDir`. Check `components.last()`. If empty or `ParentDir`, `push(Component::ParentDir)`. Never pop `RootDir` or `Prefix`. Alternatively, use `validated_path` checks against the base directory.
