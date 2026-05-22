
## 2024-05-22 - [Fix path traversal vulnerability in module resolution]
**Vulnerability:** The manual path resolution fallback in the TypeScript extractor popped path components unconditionally when encountering `..` (ParentDir), ignoring edge cases where `..` escapes the root directory or when leading `..` components should be preserved.
**Learning:** Manual path normalization logic must strictly follow semantic path construction and guard against unexpected component states like traversing above `RootDir` or losing relative paths like `../../foo`.
**Prevention:** Always validate the existing state of path components before modifying them during `ParentDir` handling, treating `RootDir` and `Prefix` as non-poppable, and preserving `ParentDir` when needed.
