
## 2026-04-08 - [Performance: Defer Allocation during Traversal]
**Learning:** During DAG traversals, creating owned variants of identifiers (like `file.to_path_buf()`) *before* checking `visited` HashSets results in heap allocations (O(E)) for every edge instead of every visited node (O(V)). By moving the `&PathBuf` allocation strictly *after* all HashSet `contains` checks using the borrowed reference (`&Path`), we drastically reduce memory churn.
**Action:** Always check `HashSet::contains` with a borrowed reference *before* creating the owned version required by `HashSet::insert`, especially in performance-critical graph traversal paths.

## 2024-04-22 - [Performance: Avoid redundant PathBuf allocations in DFS lookup paths]
**Learning:** In DAG traversal algorithms (like Tarjan's SCC in `crates/flow/src/incremental/invalidation.rs`), calling `.to_path_buf()` on borrowed `&Path` values purely for HashMap lookups triggers repeated heap allocations in O(E) dependency loops.
**Action:** Use Rust's `Borrow` trait by passing the raw `&Path` reference directly into `HashMap::get` and `HashMap::get_mut`. Allocate `PathBuf` once per node via `.clone()` ONLY when inserting data into the map or stack, achieving O(1) allocation during lookup steps.
