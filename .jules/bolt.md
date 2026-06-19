
## 2026-04-08 - [Performance: Defer Allocation during Traversal]
**Learning:** During DAG traversals, creating owned variants of identifiers (like `file.to_path_buf()`) *before* checking `visited` HashSets results in heap allocations (O(E)) for every edge instead of every visited node (O(V)). By moving the `&PathBuf` allocation strictly *after* all HashSet `contains` checks using the borrowed reference (`&Path`), we drastically reduce memory churn.
**Action:** Always check `HashSet::contains` with a borrowed reference *before* creating the owned version required by `HashSet::insert`, especially in performance-critical graph traversal paths.

## 2025-05-14 - Optimize SQL generation by avoiding intermediate Vec allocations and format! in loops
**Learning:** In D1 targets `build_upsert_stmt` and `build_delete_stmt`, strings are frequently joined inside loops resulting in unneeded `Vec` allocations and string interpolations, particularly since queries are generated dynamically at scale. Preallocating large `String`s using `with_capacity` and generating SQL queries using `write!` significantly reduces memory allocations and string allocations for edge targets mapping.
**Action:** Always prefer `std::fmt::Write` + `String::with_capacity` to assemble queries efficiently instead of `format!` and `Vec::join` during batch code exports.
