
## 2026-04-08 - [Performance: Defer Allocation during Traversal]
**Learning:** During DAG traversals, creating owned variants of identifiers (like `file.to_path_buf()`) *before* checking `visited` HashSets results in heap allocations (O(E)) for every edge instead of every visited node (O(V)). By moving the `&PathBuf` allocation strictly *after* all HashSet `contains` checks using the borrowed reference (`&Path`), we drastically reduce memory churn.
**Action:** Always check `HashSet::contains` with a borrowed reference *before* creating the owned version required by `HashSet::insert`, especially in performance-critical graph traversal paths.
## 2026-05-18 - [Performance: Defer Path Allocation in Recursive Traversal]
**Learning:** In hot recursive traversal functions like `tarjan_dfs` and `visit_node`, checking `HashSet` or `HashMap` inclusion with an owned `PathBuf` (`file.to_path_buf()`) inside the loop creates excessive memory churn. By passing and testing with borrowed `&Path` references, and only allocating a `PathBuf` once when actual insertion is required, memory allocation overhead is significantly reduced.
**Action:** Always test map/set membership using borrowed types before allocating owned versions in hot paths. Reuse `.clone()` of an already allocated `PathBuf` instead of creating multiple new ones from a `&Path`.
