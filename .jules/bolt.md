
## 2026-04-08 - [Performance: Defer Allocation during Traversal]
**Learning:** During DAG traversals, creating owned variants of identifiers (like `file.to_path_buf()`) *before* checking `visited` HashSets results in heap allocations (O(E)) for every edge instead of every visited node (O(V)). By moving the `&PathBuf` allocation strictly *after* all HashSet `contains` checks using the borrowed reference (`&Path`), we drastically reduce memory churn.
**Action:** Always check `HashSet::contains` with a borrowed reference *before* creating the owned version required by `HashSet::insert`, especially in performance-critical graph traversal paths.

## 2024-05-18 - [Performance: Dynamic SQL Generation for Cloudflare D1]
**Learning:** During dynamic SQL generation for Cloudflare D1 targets (e.g., `build_upsert_stmt` and `build_delete_stmt`), using intermediate `Vec` allocations and `format!` in loops causes unnecessary heap allocations and string copies.
**Action:** For dynamic SQL generation, always use `String::with_capacity` to preallocate the exact or approximate size, and use the `write!` macro (via `std::fmt::Write`) to construct queries directly to minimize heap allocations.
