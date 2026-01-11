# Contract: Streaming Graph Interface (Edge-Compatible)

**Status**: Draft
**Created**: 2026-01-11
**Purpose**: Define the iterator-based graph traversal interface that enables safe operation within Cloudflare Workers' 128MB memory limit.

## Core Principle

**NEVER** load the full graph structure (`Graph<N, E>`) into memory on the Edge. All graph operations must be:
1.  **Lazy**: Fetch data only when requested.
2.  **Streaming**: Process nodes/edges one by one or in small batches.
3.  **Stateless**: Do not retain visited history in memory beyond the current traversal frontier.

## Interface Definition

```rust
#[async_trait]
pub trait GraphStorage {
    type Node: GraphNode;
    type Edge: GraphEdge;
    type Iterator: AsyncIterator<Item = Result<Self::Node>>;

    /// Fetch a single node by ID (O(1))
    async fn get_node(&self, id: &NodeId) -> Result<Option<Self::Node>>;

    /// Streaming iterator for neighbors (Lazy load from DB)
    async fn neighbors(&self, id: &NodeId, direction: Direction) -> Result<Self::Iterator>;

    /// Check reachability using the pre-computed Reachability Index (O(1))
    /// Returns true if `ancestor` affects `descendant`
    async fn leads_to(&self, ancestor: &NodeId, descendant: &NodeId) -> Result<bool>;
}
```

## Implementation Guidelines (D1)

### `D1GraphIterator`

The D1 implementation must utilize SQLite cursors (or emulated cursors via `LIMIT/OFFSET` or `keyset pagination`) to stream results.

```rust
pub struct D1GraphIterator {
    stmt: D1PreparedStatement,
    batch_size: usize,
    current_offset: usize,
}

impl AsyncIterator for D1GraphIterator {
    type Item = Result<GraphNode>;

    async fn next(&mut self) -> Option<Self::Item> {
        // Fetch next batch from D1 if buffer empty
        // Return next item
    }
}
```

### Reachability Optimization

For conflict detection, do NOT traverse. Use the optimization:

```rust
async fn leads_to(&self, ancestor: &NodeId, descendant: &NodeId) -> Result<bool> {
    let query = "SELECT 1 FROM reachability WHERE ancestor_id = ? AND descendant_id = ? LIMIT 1";
    // Execute query
    // Return result
}
```

## Constraints

1.  **Memory Cap**: The implementation MUST NOT buffer more than `batch_size` (default 100) items in memory.
2.  **Recursion**: Recursive traversal algorithms (DFS/BFS) MUST be implemented iteratively using an external stack/queue stored in a Durable Object or handled via the Reachability Index, NOT via call-stack recursion.
