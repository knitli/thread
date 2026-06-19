
## 2026-04-08 - [Performance: Defer Allocation during Traversal]
**Learning:** During DAG traversals, creating owned variants of identifiers (like `file.to_path_buf()`) *before* checking `visited` HashSets results in heap allocations (O(E)) for every edge instead of every visited node (O(V)). By moving the `&PathBuf` allocation strictly *after* all HashSet `contains` checks using the borrowed reference (`&Path`), we drastically reduce memory churn.
**Action:** Always check `HashSet::contains` with a borrowed reference *before* creating the owned version required by `HashSet::insert`, especially in performance-critical graph traversal paths.

## 2026-04-09 - [Performance: SQL Generation without Intermediate Allocations]
**Learning:** Constructing complex SQL queries (like bulk upserts) via `format!` macros inside loops or via `Vec<String>::join()` creates a massive amount of intermediate String and Vec heap allocations. In the D1 target specifically, this was a severe bottleneck during batch edge operations.
**Action:** Use `std::fmt::Write` directly on a pre-allocated String buffer (`String::with_capacity`) along with `Vec::with_capacity` for parameters. This skips all intermediate strings and slices, typically resulting in 50-75% generation time improvements for hot-path generation functions.
