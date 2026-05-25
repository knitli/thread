
## 2026-04-08 - [Performance: Defer Allocation during Traversal]
**Learning:** During DAG traversals, creating owned variants of identifiers (like `file.to_path_buf()`) *before* checking `visited` HashSets results in heap allocations (O(E)) for every edge instead of every visited node (O(V)). By moving the `&PathBuf` allocation strictly *after* all HashSet `contains` checks using the borrowed reference (`&Path`), we drastically reduce memory churn.
**Action:** Always check `HashSet::contains` with a borrowed reference *before* creating the owned version required by `HashSet::insert`, especially in performance-critical graph traversal paths.

## 2024-05-24 - Defer PathBuf allocations during graph traversal
**Learning:** Repeated `.to_path_buf()` calls in tight loops for map lookups (e.g. `entry()` or `get()`) create redundant `O(E)` memory allocations, becoming a critical bottleneck.
**Action:** Use borrowed `&Path` references for `RapidMap` lookups (`contains_key`, `get`, `get_mut`) and defer `PathBuf` heap allocation until strictly needed for initial insertion.
