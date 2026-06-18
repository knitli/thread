
## 2026-04-08 - [Performance: Defer Allocation during Traversal]
**Learning:** During DAG traversals, creating owned variants of identifiers (like `file.to_path_buf()`) *before* checking `visited` HashSets results in heap allocations (O(E)) for every edge instead of every visited node (O(V)). By moving the `&PathBuf` allocation strictly *after* all HashSet `contains` checks using the borrowed reference (`&Path`), we drastically reduce memory churn.
**Action:** Always check `HashSet::contains` with a borrowed reference *before* creating the owned version required by `HashSet::insert`, especially in performance-critical graph traversal paths.

## 2024-05-24 - Dynamic SQL Generation Optimization
**Learning:** For dynamic SQL generation (e.g., in `crates/flow/src/targets/d1.rs` for Cloudflare D1 targets), using intermediate `Vec` allocations and `format!` in loops is a significant performance bottleneck due to excessive heap allocations and string copies.
**Action:** Always use `String::with_capacity` and the `write!` macro (via `std::fmt::Write`) to construct queries directly to minimize heap allocations and string copies.
