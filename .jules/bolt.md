
## 2026-04-08 - [Performance: Defer Allocation during Traversal]
**Learning:** During DAG traversals, creating owned variants of identifiers (like `file.to_path_buf()`) *before* checking `visited` HashSets results in heap allocations (O(E)) for every edge instead of every visited node (O(V)). By moving the `&PathBuf` allocation strictly *after* all HashSet `contains` checks using the borrowed reference (`&Path`), we drastically reduce memory churn.
**Action:** Always check `HashSet::contains` with a borrowed reference *before* creating the owned version required by `HashSet::insert`, especially in performance-critical graph traversal paths.

## 2024-05-30 - [Performance: Direct String Writing over Format]
**Learning:** Constructing complex SQL queries (or other long strings) with dynamic elements using `format!` or joining intermediate arrays causes multiple unnecessary heap allocations. This directly negatively impacts query generation latency, which can be the bottleneck in the pipeline.
**Action:** For string construction with loops or multiple elements, always use `std::fmt::Write` (the `write!` macro) into a `String::with_capacity(...)` pre-allocated buffer to drastically reduce heap allocations and memory copies.
