
## 2026-04-08 - [Performance: Defer Allocation during Traversal]
**Learning:** During DAG traversals, creating owned variants of identifiers (like `file.to_path_buf()`) *before* checking `visited` HashSets results in heap allocations (O(E)) for every edge instead of every visited node (O(V)). By moving the `&PathBuf` allocation strictly *after* all HashSet `contains` checks using the borrowed reference (`&Path`), we drastically reduce memory churn.
**Action:** Always check `HashSet::contains` with a borrowed reference *before* creating the owned version required by `HashSet::insert`, especially in performance-critical graph traversal paths.

## 2024-05-18 - [Performance: Optimizing SQL string generation]
**Learning:** Generating SQL statements dynamically in `D1ExportContext` by using intermediate string allocations and vectors (e.g. `vec![]` then `join(", ")`) incurs significant overhead. Direct formatted writes to a pre-allocated `String` using `std::fmt::Write` reduces allocations and drastically improves statement generation performance.
**Action:** Replace intermediate vectors and string slices with single, pre-allocated strings (`String::with_capacity()`) and `write!()` macros in performance critical SQL statement construction logic.
