
## 2026-04-08 - [Performance: Defer Allocation during Traversal]
**Learning:** During DAG traversals, creating owned variants of identifiers (like `file.to_path_buf()`) *before* checking `visited` HashSets results in heap allocations (O(E)) for every edge instead of every visited node (O(V)). By moving the `&PathBuf` allocation strictly *after* all HashSet `contains` checks using the borrowed reference (`&Path`), we drastically reduce memory churn.
**Action:** Always check `HashSet::contains` with a borrowed reference *before* creating the owned version required by `HashSet::insert`, especially in performance-critical graph traversal paths.

## 2026-04-09 - [Performance: Dynamic SQL Generation]
**Learning:** For dynamic SQL generation (e.g., in `crates/flow/src/targets/d1.rs` for Cloudflare D1 targets), constructing queries using intermediate `Vec` allocations and joining strings with `format!` causes unnecessary heap allocations and string copies.
**Action:** Always use `String::with_capacity` and the `write!` macro (via `std::fmt::Write`) to construct queries directly to minimize memory allocations and improve query building throughput.
