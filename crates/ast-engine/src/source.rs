// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

//! # Document and Content Abstraction
//!
//! Core traits for abstracting source code documents and their encoding across different platforms.
//!
//! ## Multi-Platform Support
//!
//! thread-ast-engine supports multiple text encodings to work across different environments:
//! - **UTF-8** - Standard for CLI applications and most Rust code
//! - **UTF-16** - Required for Node.js NAPI bindings
//! - **`Vec<char>`** - Used in WASM environments for JavaScript interop
//!
//! Different encodings affect how byte positions and ranges are calculated in tree-sitter nodes,
//! so this abstraction ensures consistent behavior across platforms.
//!
//! ## Key Concepts
//!
//! ### Documents ([`Doc`])
//! Represents a complete source code document with its language and parsing information.
//! Provides methods to access the source text, perform edits, and get AST nodes.
//!
//! ### Content ([`Content`])
//! Abstracts the underlying text representation (bytes, UTF-16 code units, etc.).
//! Handles encoding/decoding operations needed for text manipulation and replacement.
//!
//! ### Node Interface ([`SgNode`])
//! Generic interface for AST nodes that works across different parser backends.
//! Provides navigation, introspection, and traversal methods.
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! // Documents abstract over different source encodings
//! let doc = StrDoc::new("const x = 42;", Language::JavaScript);
//! let root = doc.root_node();
//!
//! // Content trait handles encoding differences transparently
//! let source_bytes = doc.get_source().get_range(0..5); // "const"
//! ```

use crate::{Position, language::Language, node::KindId};
use std::borrow::Cow;
use std::ops::Range;

/// Represents an edit operation on source code.
///
/// Edits specify where in the source to make changes and what new content
/// to insert. Used for incremental parsing and code transformation.
///
/// # Type Parameters
///
/// - `S: Content` - The content type (determines encoding)
///
/// # Example
///
/// ```rust,ignore
/// let edit = Edit {
///     position: 5,           // Start at byte position 5
///     deleted_length: 3,     // Delete 3 bytes
///     inserted_text: "new".as_bytes().to_vec(), // Insert "new"
/// };
/// ```
// https://github.com/tree-sitter/tree-sitter/blob/e4e5ffe517ca2c668689b24cb17c51b8c6db0790/cli/src/parse.rs
#[derive(Debug, Clone)]
pub struct Edit<S: Content> {
    /// Byte position where the edit starts
    pub position: usize,
    /// Number of bytes to delete from the original content
    pub deleted_length: usize,
    /// New content to insert (in the content's underlying representation)
    pub inserted_text: Vec<S::Underlying>,
}

/// Generic interface for AST nodes across different parser backends.
///
/// `SgNode` (`SourceGraph` Node) provides a consistent API for working with
/// AST nodes regardless of the underlying parser implementation. Supports
/// navigation, introspection, and traversal operations.
///
/// # Lifetime
///
/// The lifetime `'r` ties the node to its root document, ensuring memory safety.
///
/// # Note
///
/// Some method names match tree-sitter's API. Use fully qualified syntax
/// if there are naming conflicts with tree-sitter imports.
///
/// See: <https://stackoverflow.com/a/44445976/2198656>
pub trait SgNode<'r>: Clone {
    fn parent(&self) -> Option<Self>;
    fn children(&self) -> impl ExactSizeIterator<Item = Self>;
    fn kind(&self) -> Cow<'_, str>;
    fn kind_id(&self) -> KindId;
    fn node_id(&self) -> usize;
    fn range(&self) -> std::ops::Range<usize>;
    fn start_pos(&self) -> Position;
    fn end_pos(&self) -> Position;

    // default implementation
    #[allow(clippy::needless_collect)]
    fn ancestors(&self, _root: Self) -> impl Iterator<Item = Self> {
        let mut ancestors = vec![];
        let mut current = self.clone();
        while let Some(parent) = current.parent() {
            ancestors.push(parent.clone());
            current = parent;
        }
        ancestors.reverse();
        ancestors.into_iter()
    }
    fn dfs(&self) -> impl Iterator<Item = Self> {
        let mut stack = vec![self.clone()];
        std::iter::from_fn(move || {
            if let Some(node) = stack.pop() {
                let children: Vec<_> = node.children().collect();
                stack.extend(children.into_iter().rev());
                Some(node)
            } else {
                None
            }
        })
    }
    fn child(&self, nth: usize) -> Option<Self> {
        self.children().nth(nth)
    }
    fn next(&self) -> Option<Self> {
        let parent = self.parent()?;
        let mut children = parent.children();
        while let Some(child) = children.next() {
            if child.node_id() == self.node_id() {
                return children.next();
            }
        }
        None
    }
    fn prev(&self) -> Option<Self> {
        let parent = self.parent()?;
        let children = parent.children();
        let mut prev = None;
        for child in children {
            if child.node_id() == self.node_id() {
                return prev;
            }
            prev = Some(child);
        }
        None
    }
    fn next_all(&self) -> impl Iterator<Item = Self> {
        let mut next = self.next();
        std::iter::from_fn(move || {
            let n = next.clone()?;
            next = n.next();
            Some(n)
        })
    }
    fn prev_all(&self) -> impl Iterator<Item = Self> {
        let mut prev = self.prev();
        std::iter::from_fn(move || {
            let n = prev.clone()?;
            prev = n.prev();
            Some(n)
        })
    }
    fn is_named(&self) -> bool {
        true
    }
    /// N.B. it is different from `is_named` && `is_leaf`
    /// if a node has no named children.
    fn is_named_leaf(&self) -> bool {
        self.is_leaf()
    }
    fn is_leaf(&self) -> bool {
        self.children().count() == 0
    }

    // missing node is a tree-sitter specific concept
    fn is_missing(&self) -> bool {
        false
    }
    fn is_error(&self) -> bool {
        false
    }

    fn field(&self, name: &str) -> Option<Self>;
    fn field_children(&self, field_id: Option<u16>) -> impl Iterator<Item = Self>;
    fn child_by_field_id(&self, field_id: u16) -> Option<Self>;
}

/// Represents a source code document with its language and parsed AST.
///
/// `Doc` provides the core interface for working with parsed source code documents.
/// It combines the source text, language information, and AST representation in
/// a single abstraction that supports editing and node operations.
///
/// # Type Parameters
///
/// - `Source: Content` - The text representation (String, UTF-16, etc.)
/// - `Lang: Language` - The programming language implementation
/// - `Node: SgNode` - The AST node implementation
///
/// # Example
///
/// ```rust,ignore
/// // Documents provide access to source, language, and AST
/// let doc = StrDoc::new("const x = 42;", JavaScript);
///
/// // Access different aspects of the document
/// let source = doc.get_source();  // Get source text
/// let lang = doc.get_lang();      // Get language info
/// let root = doc.root_node();     // Get AST root
///
/// // Extract text from specific nodes
/// let node_text = doc.get_node_text(&some_node);
/// ```
pub trait Doc: Clone + 'static {
    /// The source code representation (String, UTF-16, etc.)
    type Source: Content;
    /// The programming language implementation
    type Lang: Language;
    /// The AST node type for this document
    type Node<'r>: SgNode<'r>;

    /// Get the language implementation for this document
    fn get_lang(&self) -> &Self::Lang;

    /// Get the source code content
    fn get_source(&self) -> &Self::Source;

    /// Apply an edit to the document, updating both source and AST
    fn do_edit(&mut self, edit: &Edit<Self::Source>) -> Result<(), String>;

    /// Get the root AST node
    fn root_node(&self) -> Self::Node<'_>;

    /// Extract the text content of a specific AST node
    fn get_node_text<'a>(&'a self, node: &Self::Node<'a>) -> Cow<'a, str>;
}

/// Abstracts source code text representation across different encodings.
///
/// `Content` allows the same AST operations to work with different text encodings
/// (UTF-8, UTF-16, etc.) by providing encoding/decoding operations and position
/// calculations. Essential for cross-platform support.
///
/// # Type Parameters
///
/// - `Underlying` - The basic unit type (u8 for UTF-8, u16 for UTF-16, etc.)
///
/// # Example
///
/// ```rust,ignore
/// // Content trait abstracts encoding differences
/// let content = "Hello, world!";
/// let bytes = content.get_range(0..5);  // [72, 101, 108, 108, 111] for UTF-8
/// let column = content.get_char_column(0, 7); // Character position
/// ```
pub trait Content: Sized {
    /// The underlying data type (u8, u16, char, etc.)
    type Underlying: Clone + PartialEq;

    /// Get a slice of the underlying data for the given byte range
    fn get_range(&self, range: Range<usize>) -> &[Self::Underlying];

    /// Convert a string to this content's underlying representation.
    ///
    /// Used during text replacement to ensure proper encoding.
    fn decode_str(src: &str) -> Cow<'_, [Self::Underlying]>;

    /// Convert underlying data back to a string.
    ///
    /// Used to extract text content after transformations.
    fn encode_bytes(bytes: &[Self::Underlying]) -> Cow<'_, str>;

    /// Calculate the character column position at a given byte offset.
    ///
    /// Handles Unicode properly by computing actual character positions
    /// rather than byte positions.
    fn get_char_column(&self, column: usize, offset: usize) -> usize;
}

impl Content for String {
    type Underlying = u8;
    fn get_range(&self, range: Range<usize>) -> &[Self::Underlying] {
        &self.as_bytes()[range]
    }
    fn decode_str(src: &str) -> Cow<'_, [Self::Underlying]> {
        Cow::Borrowed(src.as_bytes())
    }
    fn encode_bytes(bytes: &[Self::Underlying]) -> Cow<'_, str> {
        Self::from_utf8_lossy(bytes)
    }

    /// This is an O(n) operation optimized with SIMD. SIMD allows efficient processing
    /// of unusually long lines. Modest improvements for standard code lines (~100 chars)
    fn get_char_column(&self, _col: usize, offset: usize) -> usize {
        // Use SIMD-optimized version from utils crate
        thread_utils::get_char_column_simd(self, offset)
    }
}
