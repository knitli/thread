
## 2026-04-08 - [Performance: Defer Allocation during Traversal]
**Learning:** During DAG traversals, creating owned variants of identifiers (like `file.to_path_buf()`) *before* checking `visited` HashSets results in heap allocations (O(E)) for every edge instead of every visited node (O(V)). By moving the `&PathBuf` allocation strictly *after* all HashSet `contains` checks using the borrowed reference (`&Path`), we drastically reduce memory churn.
**Action:** Always check `HashSet::contains` with a borrowed reference *before* creating the owned version required by `HashSet::insert`, especially in performance-critical graph traversal paths.

## 2026-04-09 - [Performance: Pre-allocate and use write! for dynamic SQL]
**Learning:** Constructing dynamic SQL queries in hot loops (like D1 target `upsert` and `delete` batching) using `format!` and `Vec::join` creates excessive intermediate string allocations and memory copies.
**Action:** Use `String::with_capacity` pre-calculated with a conservative estimate, combined with the `write!` macro, to build dynamic SQL statements efficiently without intermediate vectors and heap allocations.
