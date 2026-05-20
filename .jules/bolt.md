
## 2026-04-08 - [Performance: Defer Allocation during Traversal]
**Learning:** During DAG traversals, creating owned variants of identifiers (like `file.to_path_buf()`) *before* checking `visited` HashSets results in heap allocations (O(E)) for every edge instead of every visited node (O(V)). By moving the `&PathBuf` allocation strictly *after* all HashSet `contains` checks using the borrowed reference (`&Path`), we drastically reduce memory churn.
**Action:** Always check `HashSet::contains` with a borrowed reference *before* creating the owned version required by `HashSet::insert`, especially in performance-critical graph traversal paths.

## 2026-05-20 - [Performance: Direct String Formatting for SQL Queries]
**Learning:** In hot paths building dynamic strings (like SQL query generators `build_upsert_stmt`), repeatedly using `format!` and creating intermediate vectors for `.join()` results in high memory churn and excessive heap allocations per query.
**Action:** Use `String::with_capacity` paired with the `write!` macro (`std::fmt::Write`) and exact-capacity `Vec` pre-allocations to build dynamic strings directly into a single buffer, drastically reducing intermediate allocations.
