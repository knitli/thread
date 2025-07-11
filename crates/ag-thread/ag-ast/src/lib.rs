/*!
This module contains the core AST abstractions for ast-grep.

It provides foundational APIs for working with AST nodes, including:
- Document abstraction and content handling
- Language trait definitions
- Node wrapper and traversal
- Memory management for cross-thread usage
*/

mod content;
mod node;

#[doc(hidden)]
pub mod pinned;

// Reexport implemented traits and types for easier access
pub use AstGrep;
pub use content::Content;
pub use node::{Doc, Node, Position, Root};
pub use pinned::{PinnedNodeData, NodeData};
