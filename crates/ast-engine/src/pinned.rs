// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

//! # Lifetime Extension for AST Nodes Across Thread and FFI Boundaries
//!
//! Enables safe passing of AST nodes across threads and FFI boundaries by extending
//! their lifetimes beyond the normal borrow checker constraints.
//!
//! ## The Problem
//!
//! Normally, AST nodes have lifetimes tied to their root document:
//! ```rust,ignore
//! let root = parse_code("let x = 42;");
//! let node = root.find("$VAR").unwrap(); // node lifetime tied to root
//! // Can't send node to another thread without root
//! ```
//!
//! ## The Solution
//!
//! [`PinnedNodeData`] keeps the root alive while allowing nodes to have `'static` lifetimes:
//! ```rust,ignore
//! let pinned = PinnedNodeData::new(root, |static_root| {
//!     static_root.find("$VAR").unwrap() // Now has 'static lifetime
//! });
//! // Can safely send `pinned` across threads
//! ```
//!
//! ## Safety
//!
//! This module uses unsafe code to extend lifetimes, but maintains safety by:
//! - Keeping the root document alive as long as nodes exist
//! - Re-adopting nodes when accessing them to ensure validity
//! - Using tree-sitter's heap-allocated node pointers which remain stable
//!
//! ## Use Cases
//!
//! - **Threading**: Send AST analysis results between threads
//! - **FFI**: Pass nodes to JavaScript (NAPI) or Python (PyO3)
//! - **Async**: Store nodes across await points
//! - **Caching**: Keep processed nodes in long-lived data structures

use crate::Doc;
#[cfg(feature = "matching")]
use crate::NodeMatch;
use crate::node::{Node, Root};

// ast-grep Node contains a reference to Root. It implies that
// node can be used only when the Root is valid and not dropped.
// By default, tree-sitter Node<'r> is scoped by ast Root's lifetime
// That is, Node can be only used when root is on the call stack (RAII)
// It is usually sufficient but for following scenario the brwchck is too conservative:
// 1. passing Root and Node across threads
// 2. passing Root and Node across FFI boundary (from Rust to napi/pyo3)
//
// This resembles self-referencing pattern and we can use solution similar to PinBox.
// Actually, tree-sitter's Node reference is already pointing to a heap address.
// N.B. it is not documented but can be inferred from the source code and concurrency doc paragraph.
// https://github.com/tree-sitter/tree-sitter/blob/20924fa4cdeb10d82ac308481e39bf8519334e55/lib/src/tree.c#L9-L20
// https://github.com/tree-sitter/tree-sitter/blob/20924fa4cdeb10d82ac308481e39bf8519334e55/lib/src/tree.c#L37-L39
// https://tree-sitter.github.io/tree-sitter/using-parsers#concurrency
//
/// Container that extends AST node lifetimes by keeping their root document alive.
///
/// `PinnedNodeData` solves the problem of passing AST nodes across thread boundaries
/// or FFI interfaces where normal lifetime constraints are too restrictive. It combines
/// a root document with data containing nodes, ensuring the nodes remain valid.
///
/// # Type Parameters
///
/// - `D: Doc` - The document type (e.g., `StrDoc<Language>`)
/// - `T` - Data containing nodes with `'static` lifetimes
///
/// # Safety Model
///
/// The container uses unsafe code to extend node lifetimes, but maintains safety by:
/// - Keeping the root document alive to prevent tree deallocation
/// - Re-adopting nodes when accessed to ensure they point to valid memory
/// - Leveraging tree-sitter's stable heap-allocated node pointers
///
/// # Usage Patterns
///
/// ## 1. Borrowing Pattern (Recommended)
/// Use through references to guarantee safety:
/// ```rust,ignore
/// let pinned = PinnedNodeData::new(root, |static_root| {
///     static_root.find("pattern").unwrap()
/// });
/// let node = pinned.get_data(); // Safe access
/// ```
///
/// ## 2. Ownership Pattern (Advanced)
/// Take ownership but ensure root stays alive:
/// ```rust,ignore
/// let (root, node_data) = pinned.into_raw();
/// // You must keep `root` alive while using `node_data`
/// ```
///
/// # Thread Safety
///
/// Safe to send across threads as long as the contained data is `Send`:
/// ```rust,ignore
/// std::thread::spawn(move || {
///     let node = pinned.get_data();
///     // Process node in background thread
/// });
/// ```
#[doc(hidden)]
pub struct PinnedNodeData<D: Doc, T> {
    /// Root document kept alive to ensure node validity
    pin: Root<D>,
    /// Data containing nodes with extended lifetimes
    data: T,
}

impl<T, D: Doc + 'static> PinnedNodeData<D, T> {
    #[allow(clippy::deref_addrof)]
    pub fn new<F>(pin: Root<D>, func: F) -> Self
    where
        F: FnOnce(&'static Root<D>) -> T,
    {
        // TODO: explain why unsafe works here and what guarantee it needs
        let reference = unsafe { &*(&raw const pin) as &'static Root<D> };
        let data = func(reference);
        Self { pin, data }
    }
}

impl<D: Doc + 'static, T> PinnedNodeData<D, T>
where
    T: NodeData<D>,
{
    #[allow(clippy::deref_addrof)] // the lifetimes need to be static
    pub fn get_data(&mut self) -> &T::Data {
        let pin = unsafe { &*(&raw const self.pin) as &'static Root<D> };
        self.data.visit_nodes(|n| unsafe { pin.readopt(n) });
        self.data.get_data()
    }
    pub fn into_raw(self) -> (Root<D>, T) {
        (self.pin, self.data)
    }
}

/// # Safety
/// TODO: explain unsafe trait
pub unsafe trait NodeData<D: Doc> {
    type Data;
    fn get_data(&self) -> &Self::Data;
    fn visit_nodes<F>(&mut self, f: F)
    where
        F: FnMut(&mut Node<'_, D>);
}

unsafe impl<D: Doc> NodeData<D> for Node<'static, D> {
    type Data = Self;
    fn get_data(&self) -> &Self::Data {
        self
    }
    fn visit_nodes<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Node<'_, D>),
    {
        f(self);
    }
}

#[cfg(feature = "matching")]
unsafe impl<D: Doc> NodeData<D> for NodeMatch<'static, D> {
    type Data = Self;
    fn get_data(&self) -> &Self::Data {
        self
    }
    fn visit_nodes<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Node<'_, D>),
    {
        // update the matched Node
        f(unsafe { self.get_node_mut() });
        // update the meta variable captured
        let env = self.get_env_mut();
        env.visit_nodes(f);
    }
}

#[cfg(feature = "matching")]
unsafe impl<D: Doc> NodeData<D> for Vec<NodeMatch<'static, D>> {
    type Data = Self;
    fn get_data(&self) -> &Self::Data {
        self
    }
    fn visit_nodes<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Node<'_, D>),
    {
        for n in self {
            n.visit_nodes(&mut f);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::language::Tsx;
    use crate::node::Root;
    use crate::tree_sitter::StrDoc;

    fn return_from_func() -> PinnedNodeData<StrDoc<Tsx>, Node<'static, StrDoc<Tsx>>> {
        let root = Root::str("let a = 123", Tsx);
        PinnedNodeData::new(root, |r| r.root().child(0).unwrap().child(1).unwrap())
    }

    #[test]
    fn test_borrow() {
        let mut retained = return_from_func();
        let b = retained.get_data();
        assert_eq!(b.text(), "a = 123");
        assert!(matches!(b.lang(), Tsx));
    }

    #[test]
    #[ignore]
    fn test_node_match() {
        todo!()
    }

    fn return_vec() -> PinnedNodeData<StrDoc<Tsx>, Vec<NodeMatch<'static, StrDoc<Tsx>>>> {
        let root = Root::str("let a = 123", Tsx);
        PinnedNodeData::new(root, |r| {
            r.root()
                .child(0)
                .unwrap()
                .children()
                .map(NodeMatch::from)
                .collect()
        })
    }

    #[test]
    fn test_vec_node() {
        let mut pinned = return_vec();
        let nodes = pinned.get_data();
        assert!(!nodes.is_empty());
        assert_eq!(nodes[0].text(), "let");
        assert_eq!(nodes[1].text(), "a = 123");
    }
}
