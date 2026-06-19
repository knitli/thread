
## 2026-04-08 - [Performance: Defer Allocation during Traversal]
**Learning:** During DAG traversals, creating owned variants of identifiers (like `file.to_path_buf()`) *before* checking `visited` HashSets results in heap allocations (O(E)) for every edge instead of every visited node (O(V)). By moving the `&PathBuf` allocation strictly *after* all HashSet `contains` checks using the borrowed reference (`&Path`), we drastically reduce memory churn.
**Action:** Always check `HashSet::contains` with a borrowed reference *before* creating the owned version required by `HashSet::insert`, especially in performance-critical graph traversal paths.

## 2024-05-17 - [Dynamic SQL Generation Performance]
**Learning:** In performance-critical dynamic SQL generation (like Cloudflare D1 integration), mapping fields into multiple temporary `Vec<String>` structures just to `.join(", ")` them creates significant intermediate heap allocations and string copies (O(N) operations per query).
**Action:** Always use `String::with_capacity` paired with the `write!` macro (via `std::fmt::Write`) to construct the final SQL query incrementally. This limits allocations to a single O(1) buffer allocation per query statement.
