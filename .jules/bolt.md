
## 2026-04-08 - [Performance: Defer Allocation during Traversal]
**Learning:** During DAG traversals, creating owned variants of identifiers (like `file.to_path_buf()`) *before* checking `visited` HashSets results in heap allocations (O(E)) for every edge instead of every visited node (O(V)). By moving the `&PathBuf` allocation strictly *after* all HashSet `contains` checks using the borrowed reference (`&Path`), we drastically reduce memory churn.
**Action:** Always check `HashSet::contains` with a borrowed reference *before* creating the owned version required by `HashSet::insert`, especially in performance-critical graph traversal paths.
## 2026-04-14 - [Performance: Avoid redundant `to_path_buf` in Hash lookups]
**Learning:** In highly recursive functions like Tarjan's SCC DFS algorithm (`tarjan_dfs`), calling `.to_path_buf()` on `&Path` just to satisfy `.get()` or `.contains()` calls on HashMaps/Sets causes significant unnecessary heap allocation overhead (O(E) instead of O(V)).
**Action:** When working with standard or custom hashing collections (like `RapidMap`, `RapidSet`, `std::collections::HashMap`), pass borrowed types (e.g., `&Path`) for lookups instead of allocating owned types (`PathBuf`). Only create the owned form when actually inserting into the collection, and create it exactly once per node visit.
