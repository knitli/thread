
## 2026-04-08 - [Performance: Defer Allocation during Traversal]
**Learning:** During DAG traversals, creating owned variants of identifiers (like `file.to_path_buf()`) *before* checking `visited` HashSets results in heap allocations (O(E)) for every edge instead of every visited node (O(V)). By moving the `&PathBuf` allocation strictly *after* all HashSet `contains` checks using the borrowed reference (`&Path`), we drastically reduce memory churn.
**Action:** Always check `HashSet::contains` with a borrowed reference *before* creating the owned version required by `HashSet::insert`, especially in performance-critical graph traversal paths.
## 2026-04-10 - [Performance: Direct SQL String Formatting]
**Learning:** In D1 target components (`build_upsert_stmt`), dynamically generating SQL using intermediate `Vec` allocations combined with `.join()` operations causes unnecessary heap allocations and copying. Constructing the raw SQL query directly via `std::fmt::Write` to a single pre-allocated `String` significantly reduces memory overhead.
**Action:** For dynamic SQL generation on high-frequency paths, use `String::with_capacity` and `write!` instead of intermediate arrays and `.join()`.
