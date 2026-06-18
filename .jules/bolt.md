
## 2026-04-08 - [Performance: Defer Allocation during Traversal]
**Learning:** During DAG traversals, creating owned variants of identifiers (like `file.to_path_buf()`) *before* checking `visited` HashSets results in heap allocations (O(E)) for every edge instead of every visited node (O(V)). By moving the `&PathBuf` allocation strictly *after* all HashSet `contains` checks using the borrowed reference (`&Path`), we drastically reduce memory churn.
**Action:** Always check `HashSet::contains` with a borrowed reference *before* creating the owned version required by `HashSet::insert`, especially in performance-critical graph traversal paths.

## 2023-10-25 - [Performance: Dynamic SQL String Generation in D1 Target]
**Learning:** Constructing dynamic SQL queries using intermediate `Vec` allocations (e.g. `columns.join(", ")`) and repeated `format!` calls inside loops causes unnecessary heap allocations, memory fragmentation, and string copying. In D1 targets where `build_upsert_stmt` and `build_delete_stmt` might be called thousands of times sequentially for batched updates, this O(N) allocation pattern per query becomes a CPU bottleneck.
**Action:** Always use `String::with_capacity` and the `write!` macro (via `std::fmt::Write`) to construct queries directly to minimize heap allocations and string copies. Pre-allocate collections (`Vec::with_capacity`) for statement arguments when the length is known from schemas.
