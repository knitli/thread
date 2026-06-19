
## 2026-04-08 - [Performance: Defer Allocation during Traversal]
**Learning:** During DAG traversals, creating owned variants of identifiers (like `file.to_path_buf()`) *before* checking `visited` HashSets results in heap allocations (O(E)) for every edge instead of every visited node (O(V)). By moving the `&PathBuf` allocation strictly *after* all HashSet `contains` checks using the borrowed reference (`&Path`), we drastically reduce memory churn.
**Action:** Always check `HashSet::contains` with a borrowed reference *before* creating the owned version required by `HashSet::insert`, especially in performance-critical graph traversal paths.

## 2026-06-05 - [Performance: Direct SQL String Formatting]
**Learning:** In highly-frequent query builders, allocating intermediate `Vec<String>` and using `format!` and `join` incurs high heap allocation overhead. In `D1ExportContext::build_upsert_stmt` and `build_delete_stmt`, directly using `String::with_capacity` and formatting using `std::fmt::Write` reduced latencies by ~66% and ~2% respectively.
**Action:** When constructing queries or strings in tight loops, avoid temporary vectors and directly write into pre-allocated `String` buffers using `std::fmt::Write`.
