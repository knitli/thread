
## 2026-04-08 - [Performance: Defer Allocation during Traversal]
**Learning:** During DAG traversals, creating owned variants of identifiers (like `file.to_path_buf()`) *before* checking `visited` HashSets results in heap allocations (O(E)) for every edge instead of every visited node (O(V)). By moving the `&PathBuf` allocation strictly *after* all HashSet `contains` checks using the borrowed reference (`&Path`), we drastically reduce memory churn.
**Action:** Always check `HashSet::contains` with a borrowed reference *before* creating the owned version required by `HashSet::insert`, especially in performance-critical graph traversal paths.
## 2026-05-19 - [Performance: Dynamic SQL Generation]
**Learning:** In highly called database adapter functions (like Cloudflare D1 SQL query generation), using `vec![]` to accumulate string fragments followed by `format!` and `join` creates significant unnecessary O(n) heap allocations. Instead, pre-allocating a `String::with_capacity` and utilizing `std::fmt::Write` via the `write!` macro allows direct, zero-copy string construction.
**Action:** Always prefer `String::with_capacity` combined with manual `write!` appending for dynamic string generation rather than intermediate Vector collections in performance-critical code paths.
