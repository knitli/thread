
## 2026-04-08 - [Performance: Defer Allocation during Traversal]
**Learning:** During DAG traversals, creating owned variants of identifiers (like `file.to_path_buf()`) *before* checking `visited` HashSets results in heap allocations (O(E)) for every edge instead of every visited node (O(V)). By moving the `&PathBuf` allocation strictly *after* all HashSet `contains` checks using the borrowed reference (`&Path`), we drastically reduce memory churn.
**Action:** Always check `HashSet::contains` with a borrowed reference *before* creating the owned version required by `HashSet::insert`, especially in performance-critical graph traversal paths.
## 2026-04-10 - [Performance: SQL Generation without intermediate strings]
**Learning:** SQL string generation inside loops heavily allocates intermediate strings when using Vec<String>::join. Since Cloudflare D1 targets are frequently accessed and performance is critical for network-bound operations, we must optimize format string generation.
**Action:** Use std::fmt::Write to append to a pre-allocated String instead of pushing formatting strings and joining them later to reduce string allocations.
