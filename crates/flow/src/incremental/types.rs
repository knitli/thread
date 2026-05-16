// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Core data structures for the incremental update system.
//!
//! This module defines the foundational types used for fingerprint tracking,
//! dependency edges, and symbol-level dependency information. The design is
//! adapted from Recoco's `FieldDefFingerprint` pattern (analyzer.rs:69-84).

use recoco::utils::fingerprint::{Fingerprint, Fingerprinter};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Tracks the fingerprint and source files for an analysis result.
///
/// Adapted from Recoco's `FieldDefFingerprint` pattern. Combines content
/// fingerprinting with source file tracking to enable precise invalidation
/// scope determination.
///
/// # Examples
///
/// ```rust
/// use thread_flow::incremental::types::AnalysisDefFingerprint;
///
/// // Create a fingerprint from file content
/// let fp = AnalysisDefFingerprint::new(b"fn main() {}");
/// assert!(fp.content_matches(b"fn main() {}"));
/// assert!(!fp.content_matches(b"fn other() {}"));
/// ```
#[derive(Debug, Clone)]
pub struct AnalysisDefFingerprint {
    /// Source files that contribute to this analysis result.
    /// Used to determine invalidation scope when dependencies change.
    pub source_files: HashSet<PathBuf>,

    /// Content fingerprint of the analyzed file (Blake3, 16 bytes).
    /// Combines file content hash for change detection.
    pub fingerprint: Fingerprint,

    /// Timestamp of last successful analysis (Unix microseconds).
    /// `None` if this fingerprint has never been persisted.
    pub last_analyzed: Option<i64>,
}

/// A dependency edge representing a relationship between two files.
///
/// Edges are directed: `from` depends on `to`. For example, if `main.rs`
/// imports `utils.rs`, the edge is `from: main.rs, to: utils.rs`.
///
/// # Examples
///
/// ```rust
/// use thread_flow::incremental::types::{DependencyEdge, DependencyType};
/// use std::path::PathBuf;
///
/// let edge = DependencyEdge {
///     from: PathBuf::from("src/main.rs"),
///     to: PathBuf::from("src/utils.rs"),
///     dep_type: DependencyType::Import,
///     symbol: None,
/// };
/// assert_eq!(edge.dep_type, DependencyType::Import);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DependencyEdge {
    /// Source file path (the file that depends on another).
    pub from: PathBuf,

    /// Target file path (the file being depended upon).
    pub to: PathBuf,

    /// The type of dependency relationship.
    pub dep_type: DependencyType,

    /// Optional symbol-level dependency information.
    /// When present, enables finer-grained invalidation.
    pub symbol: Option<SymbolDependency>,
}

/// The type of dependency relationship between files.
///
/// Determines how changes propagate through the dependency graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependencyType {
    /// Direct import/require/use statement (e.g., `use crate::utils;`).
    Import,

    /// Export declaration that other files may consume.
    Export,

    /// Macro expansion dependency.
    Macro,

    /// Type dependency (e.g., TypeScript interfaces, Rust type aliases).
    Type,

    /// Trait implementation dependency (Rust-specific).
    Trait,
}

/// The strength of a dependency relationship.
///
/// Strong dependencies always trigger reanalysis on change.
/// Weak dependencies may be skipped during invalidation traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependencyStrength {
    /// Hard dependency: change always requires reanalysis of dependents.
    Strong,

    /// Soft dependency: change may require reanalysis (e.g., dev-dependencies).
    Weak,
}

/// Symbol-level dependency tracking for fine-grained invalidation.
///
/// Tracks which specific symbol in the source file depends on which
/// specific symbol in the target file.
///
/// # Examples
///
/// ```rust
/// use thread_flow::incremental::types::{SymbolDependency, SymbolKind, DependencyStrength};
///
/// let dep = SymbolDependency {
///     from_symbol: "parse_config".to_string(),
///     to_symbol: "ConfigReader".to_string(),
///     kind: SymbolKind::Function,
///     strength: DependencyStrength::Strong,
/// };
/// assert_eq!(dep.kind, SymbolKind::Function);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolDependency {
    /// Symbol path in the source file (the dependent symbol).
    pub from_symbol: String,

    /// Symbol path in the target file (the dependency).
    pub to_symbol: String,

    /// The kind of symbol being depended upon.
    pub kind: SymbolKind,

    /// Strength of this symbol-level dependency.
    pub strength: DependencyStrength,
}

/// Classification of symbols for dependency tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolKind {
    /// Function or method definition.
    Function,

    /// Class or struct definition.
    Class,

    /// Interface or trait definition.
    Interface,

    /// Type alias or typedef.
    TypeAlias,

    /// Constant or static variable.
    Constant,

    /// Enum definition.
    Enum,

    /// Module or namespace.
    Module,

    /// Macro definition.
    Macro,
}

// ─── Implementation ──────────────────────────────────────────────────────────

impl AnalysisDefFingerprint {
    /// Creates a new fingerprint from raw file content bytes.
    ///
    /// Computes a Blake3-based fingerprint of the content using Recoco's
    /// `Fingerprinter` builder pattern.
    ///
    /// # Arguments
    ///
    /// * `content` - The raw bytes of the file to fingerprint.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use thread_flow::incremental::types::AnalysisDefFingerprint;
    ///
    /// let fp = AnalysisDefFingerprint::new(b"hello world");
    /// assert!(fp.content_matches(b"hello world"));
    /// ```
    pub fn new(content: &[u8]) -> Self {
        let mut fingerprinter = Fingerprinter::default();
        fingerprinter.write_raw_bytes(content);
        Self {
            source_files: HashSet::new(),
            fingerprint: fingerprinter.into_fingerprint(),
            last_analyzed: None,
        }
    }

    /// Creates a new fingerprint with associated source files.
    ///
    /// The source files represent the set of files that contributed to
    /// this analysis result, enabling precise invalidation scope.
    ///
    /// # Arguments
    ///
    /// * `content` - The raw bytes of the primary file.
    /// * `source_files` - Files that contributed to this analysis.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use thread_flow::incremental::types::AnalysisDefFingerprint;
    /// use std::collections::HashSet;
    /// use std::path::PathBuf;
    ///
    /// let sources = HashSet::from([PathBuf::from("dep.rs")]);
    /// let fp = AnalysisDefFingerprint::with_sources(b"content", sources);
    /// assert_eq!(fp.source_files.len(), 1);
    /// ```
    pub fn with_sources(content: &[u8], source_files: HashSet<PathBuf>) -> Self {
        let mut fingerprinter = Fingerprinter::default();
        fingerprinter.write_raw_bytes(content);
        Self {
            source_files,
            fingerprint: fingerprinter.into_fingerprint(),
            last_analyzed: None,
        }
    }

    /// Updates the fingerprint with new content, preserving source files.
    ///
    /// Returns a new `AnalysisDefFingerprint` with an updated fingerprint
    /// computed from the new content bytes.
    ///
    /// # Arguments
    ///
    /// * `content` - The new raw bytes to fingerprint.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use thread_flow::incremental::types::AnalysisDefFingerprint;
    ///
    /// let fp = AnalysisDefFingerprint::new(b"old content");
    /// let updated = fp.update_fingerprint(b"new content");
    /// assert!(!updated.content_matches(b"old content"));
    /// assert!(updated.content_matches(b"new content"));
    /// ```
    pub fn update_fingerprint(&self, content: &[u8]) -> Self {
        let mut fingerprinter = Fingerprinter::default();
        fingerprinter.write_raw_bytes(content);
        Self {
            source_files: self.source_files.clone(),
            fingerprint: fingerprinter.into_fingerprint(),
            last_analyzed: None,
        }
    }

    /// Checks if the given content matches this fingerprint.
    ///
    /// Computes a fresh fingerprint from the content and compares it
    /// byte-for-byte with the stored fingerprint.
    ///
    /// # Arguments
    ///
    /// * `content` - The raw bytes to check against the stored fingerprint.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use thread_flow::incremental::types::AnalysisDefFingerprint;
    ///
    /// let fp = AnalysisDefFingerprint::new(b"fn main() {}");
    /// assert!(fp.content_matches(b"fn main() {}"));
    /// assert!(!fp.content_matches(b"fn main() { println!(); }"));
    /// ```
    pub fn content_matches(&self, content: &[u8]) -> bool {
        let mut fingerprinter = Fingerprinter::default();
        fingerprinter.write_raw_bytes(content);
        let other = fingerprinter.into_fingerprint();
        self.fingerprint.as_slice() == other.as_slice()
    }

    /// Adds a source file to the tracked set.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to add to the source files set.
    pub fn add_source_file(&mut self, path: PathBuf) {
        self.source_files.insert(path);
    }

    /// Removes a source file from the tracked set.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to remove from the source files set.
    ///
    /// # Returns
    ///
    /// `true` if the path was present and removed.
    pub fn remove_source_file(&mut self, path: &Path) -> bool {
        self.source_files.remove(path)
    }

    /// Sets the last analyzed timestamp.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - Unix timestamp in microseconds.
    pub fn set_last_analyzed(&mut self, timestamp: i64) {
        self.last_analyzed = Some(timestamp);
    }

    /// Returns the number of source files tracked.
    pub fn source_file_count(&self) -> usize {
        self.source_files.len()
    }

    /// Returns a reference to the underlying [`Fingerprint`].
    pub fn fingerprint(&self) -> &Fingerprint {
        &self.fingerprint
    }
}

impl DependencyEdge {
    /// Creates a new dependency edge with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `from` - The source file path (dependent).
    /// * `to` - The target file path (dependency).
    /// * `dep_type` - The type of dependency.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use thread_flow::incremental::types::{DependencyEdge, DependencyType};
    /// use std::path::PathBuf;
    ///
    /// let edge = DependencyEdge::new(
    ///     PathBuf::from("a.rs"),
    ///     PathBuf::from("b.rs"),
    ///     DependencyType::Import,
    /// );
    /// assert!(edge.symbol.is_none());
    /// ```
    pub fn new(from: PathBuf, to: PathBuf, dep_type: DependencyType) -> Self {
        Self {
            from,
            to,
            dep_type,
            symbol: None,
        }
    }

    /// Creates a new dependency edge with symbol-level tracking.
    ///
    /// # Arguments
    ///
    /// * `from` - The source file path (dependent).
    /// * `to` - The target file path (dependency).
    /// * `dep_type` - The type of dependency.
    /// * `symbol` - Symbol-level dependency information.
    pub fn with_symbol(
        from: PathBuf,
        to: PathBuf,
        dep_type: DependencyType,
        symbol: SymbolDependency,
    ) -> Self {
        Self {
            from,
            to,
            dep_type,
            symbol: Some(symbol),
        }
    }

    /// Returns the effective dependency strength.
    ///
    /// If a symbol-level dependency is present, uses its strength.
    /// Otherwise, defaults to [`DependencyStrength::Strong`] for import/trait
    /// edges and [`DependencyStrength::Weak`] for export edges.
    pub fn effective_strength(&self) -> DependencyStrength {
        if let Some(ref sym) = self.symbol {
            return sym.strength;
        }
        match self.dep_type {
            DependencyType::Import | DependencyType::Trait | DependencyType::Macro => {
                DependencyStrength::Strong
            }
            DependencyType::Export | DependencyType::Type => DependencyStrength::Weak,
        }
    }
}

impl std::fmt::Display for DependencyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Import => write!(f, "import"),
            Self::Export => write!(f, "export"),
            Self::Macro => write!(f, "macro"),
            Self::Type => write!(f, "type"),
            Self::Trait => write!(f, "trait"),
        }
    }
}

impl std::fmt::Display for DependencyStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Strong => write!(f, "strong"),
            Self::Weak => write!(f, "weak"),
        }
    }
}

impl std::fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Function => write!(f, "function"),
            Self::Class => write!(f, "class"),
            Self::Interface => write!(f, "interface"),
            Self::TypeAlias => write!(f, "type_alias"),
            Self::Constant => write!(f, "constant"),
            Self::Enum => write!(f, "enum"),
            Self::Module => write!(f, "module"),
            Self::Macro => write!(f, "macro"),
        }
    }
}

// ─── Tests (TDD: Written BEFORE implementation) ──────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── AnalysisDefFingerprint Tests ─────────────────────────────────────

    #[test]
    fn test_fingerprint_new_creates_valid_fingerprint() {
        let content = b"fn main() { println!(\"hello\"); }";
        let fp = AnalysisDefFingerprint::new(content);

        // Fingerprint should be 16 bytes
        assert_eq!(fp.fingerprint.as_slice().len(), 16);
        // No source files by default
        assert!(fp.source_files.is_empty());
        // Not yet analyzed
        assert!(fp.last_analyzed.is_none());
    }

    #[test]
    fn test_fingerprint_content_matches_same_content() {
        let content = b"use std::collections::HashMap;";
        let fp = AnalysisDefFingerprint::new(content);
        assert!(fp.content_matches(content));
    }

    #[test]
    fn test_fingerprint_content_does_not_match_different_content() {
        let fp = AnalysisDefFingerprint::new(b"original content");
        assert!(!fp.content_matches(b"modified content"));
    }

    #[test]
    fn test_fingerprint_deterministic() {
        let content = b"deterministic test content";
        let fp1 = AnalysisDefFingerprint::new(content);
        let fp2 = AnalysisDefFingerprint::new(content);
        assert_eq!(fp1.fingerprint.as_slice(), fp2.fingerprint.as_slice());
    }

    #[test]
    fn test_fingerprint_different_content_different_hash() {
        let fp1 = AnalysisDefFingerprint::new(b"content A");
        let fp2 = AnalysisDefFingerprint::new(b"content B");
        assert_ne!(fp1.fingerprint.as_slice(), fp2.fingerprint.as_slice());
    }

    #[test]
    fn test_fingerprint_empty_content() {
        let fp = AnalysisDefFingerprint::new(b"");
        assert_eq!(fp.fingerprint.as_slice().len(), 16);
        assert!(fp.content_matches(b""));
        assert!(!fp.content_matches(b"non-empty"));
    }

    #[test]
    fn test_fingerprint_with_sources() {
        let sources = HashSet::from([
            PathBuf::from("src/utils.rs"),
            PathBuf::from("src/config.rs"),
        ]);
        let fp = AnalysisDefFingerprint::with_sources(b"content", sources.clone());
        assert_eq!(fp.source_files, sources);
        assert!(fp.content_matches(b"content"));
    }

    #[test]
    fn test_fingerprint_update_changes_hash() {
        let fp = AnalysisDefFingerprint::new(b"old content");
        let updated = fp.update_fingerprint(b"new content");

        assert_ne!(
            fp.fingerprint.as_slice(),
            updated.fingerprint.as_slice(),
            "Updated fingerprint should differ from original"
        );
        assert!(updated.content_matches(b"new content"));
        assert!(!updated.content_matches(b"old content"));
    }

    #[test]
    fn test_fingerprint_update_preserves_source_files() {
        let sources = HashSet::from([PathBuf::from("dep.rs")]);
        let fp = AnalysisDefFingerprint::with_sources(b"old", sources.clone());
        let updated = fp.update_fingerprint(b"new");
        assert_eq!(updated.source_files, sources);
    }

    #[test]
    fn test_fingerprint_update_resets_timestamp() {
        let mut fp = AnalysisDefFingerprint::new(b"content");
        fp.set_last_analyzed(1000000);
        let updated = fp.update_fingerprint(b"new content");
        assert!(
            updated.last_analyzed.is_none(),
            "Updated fingerprint should reset timestamp"
        );
    }

    #[test]
    fn test_fingerprint_add_source_file() {
        let mut fp = AnalysisDefFingerprint::new(b"content");
        assert_eq!(fp.source_file_count(), 0);

        fp.add_source_file(PathBuf::from("a.rs"));
        assert_eq!(fp.source_file_count(), 1);

        fp.add_source_file(PathBuf::from("b.rs"));
        assert_eq!(fp.source_file_count(), 2);

        // Duplicate should not increase count
        fp.add_source_file(PathBuf::from("a.rs"));
        assert_eq!(fp.source_file_count(), 2);
    }

    #[test]
    fn test_fingerprint_remove_source_file() {
        let mut fp = AnalysisDefFingerprint::with_sources(
            b"content",
            HashSet::from([PathBuf::from("a.rs"), PathBuf::from("b.rs")]),
        );

        assert!(fp.remove_source_file(Path::new("a.rs")));
        assert_eq!(fp.source_file_count(), 1);

        // Removing non-existent returns false
        assert!(!fp.remove_source_file(Path::new("c.rs")));
        assert_eq!(fp.source_file_count(), 1);
    }

    #[test]
    fn test_fingerprint_set_last_analyzed() {
        let mut fp = AnalysisDefFingerprint::new(b"content");
        assert!(fp.last_analyzed.is_none());

        fp.set_last_analyzed(1706400000_000_000); // Some timestamp
        assert_eq!(fp.last_analyzed, Some(1706400000_000_000));
    }

    #[test]
    fn test_fingerprint_accessor() {
        let fp = AnalysisDefFingerprint::new(b"test");
        let fingerprint_ref = fp.fingerprint();
        assert_eq!(fingerprint_ref.as_slice().len(), 16);
    }

    // ── DependencyEdge Tests ─────────────────────────────────────────────

    #[test]
    fn test_dependency_edge_new() {
        let edge = DependencyEdge::new(
            PathBuf::from("src/main.rs"),
            PathBuf::from("src/utils.rs"),
            DependencyType::Import,
        );

        assert_eq!(edge.from, PathBuf::from("src/main.rs"));
        assert_eq!(edge.to, PathBuf::from("src/utils.rs"));
        assert_eq!(edge.dep_type, DependencyType::Import);
        assert!(edge.symbol.is_none());
    }

    #[test]
    fn test_dependency_edge_with_symbol() {
        let symbol = SymbolDependency {
            from_symbol: "main".to_string(),
            to_symbol: "parse_config".to_string(),
            kind: SymbolKind::Function,
            strength: DependencyStrength::Strong,
        };

        let edge = DependencyEdge::with_symbol(
            PathBuf::from("main.rs"),
            PathBuf::from("config.rs"),
            DependencyType::Import,
            symbol.clone(),
        );

        assert!(edge.symbol.is_some());
        assert_eq!(edge.symbol.unwrap().to_symbol, "parse_config");
    }

    #[test]
    fn test_dependency_edge_effective_strength_import() {
        let edge = DependencyEdge::new(
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
            DependencyType::Import,
        );
        assert_eq!(edge.effective_strength(), DependencyStrength::Strong);
    }

    #[test]
    fn test_dependency_edge_effective_strength_export() {
        let edge = DependencyEdge::new(
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
            DependencyType::Export,
        );
        assert_eq!(edge.effective_strength(), DependencyStrength::Weak);
    }

    #[test]
    fn test_dependency_edge_effective_strength_trait() {
        let edge = DependencyEdge::new(
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
            DependencyType::Trait,
        );
        assert_eq!(edge.effective_strength(), DependencyStrength::Strong);
    }

    #[test]
    fn test_dependency_edge_effective_strength_macro() {
        let edge = DependencyEdge::new(
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
            DependencyType::Macro,
        );
        assert_eq!(edge.effective_strength(), DependencyStrength::Strong);
    }

    #[test]
    fn test_dependency_edge_effective_strength_type() {
        let edge = DependencyEdge::new(
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
            DependencyType::Type,
        );
        assert_eq!(edge.effective_strength(), DependencyStrength::Weak);
    }

    #[test]
    fn test_dependency_edge_symbol_overrides_strength() {
        let symbol = SymbolDependency {
            from_symbol: "a".to_string(),
            to_symbol: "b".to_string(),
            kind: SymbolKind::Function,
            strength: DependencyStrength::Weak,
        };

        // Import would be Strong, but symbol overrides to Weak
        let edge = DependencyEdge::with_symbol(
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
            DependencyType::Import,
            symbol,
        );
        assert_eq!(edge.effective_strength(), DependencyStrength::Weak);
    }

    #[test]
    fn test_dependency_edge_equality() {
        let edge1 = DependencyEdge::new(
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
            DependencyType::Import,
        );
        let edge2 = DependencyEdge::new(
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
            DependencyType::Import,
        );
        assert_eq!(edge1, edge2);
    }

    #[test]
    fn test_dependency_edge_inequality_different_type() {
        let edge1 = DependencyEdge::new(
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
            DependencyType::Import,
        );
        let edge2 = DependencyEdge::new(
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
            DependencyType::Export,
        );
        assert_ne!(edge1, edge2);
    }

    // ── DependencyEdge Serialization Tests ───────────────────────────────

    #[test]
    fn test_dependency_edge_serialization_roundtrip() {
        let edge = DependencyEdge::new(
            PathBuf::from("src/main.rs"),
            PathBuf::from("src/lib.rs"),
            DependencyType::Import,
        );

        let json = serde_json::to_string(&edge).expect("serialize");
        let deserialized: DependencyEdge = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(edge, deserialized);
    }

    #[test]
    fn test_dependency_edge_with_symbol_serialization_roundtrip() {
        let symbol = SymbolDependency {
            from_symbol: "handler".to_string(),
            to_symbol: "Router".to_string(),
            kind: SymbolKind::Class,
            strength: DependencyStrength::Strong,
        };

        let edge = DependencyEdge::with_symbol(
            PathBuf::from("api.rs"),
            PathBuf::from("router.rs"),
            DependencyType::Import,
            symbol,
        );

        let json = serde_json::to_string(&edge).expect("serialize");
        let deserialized: DependencyEdge = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(edge, deserialized);
    }

    // ── DependencyType Display Tests ─────────────────────────────────────

    #[test]
    fn test_dependency_type_display() {
        assert_eq!(format!("{}", DependencyType::Import), "import");
        assert_eq!(format!("{}", DependencyType::Export), "export");
        assert_eq!(format!("{}", DependencyType::Macro), "macro");
        assert_eq!(format!("{}", DependencyType::Type), "type");
        assert_eq!(format!("{}", DependencyType::Trait), "trait");
    }

    #[test]
    fn test_dependency_strength_display() {
        assert_eq!(format!("{}", DependencyStrength::Strong), "strong");
        assert_eq!(format!("{}", DependencyStrength::Weak), "weak");
    }

    #[test]
    fn test_symbol_kind_display() {
        assert_eq!(format!("{}", SymbolKind::Function), "function");
        assert_eq!(format!("{}", SymbolKind::Class), "class");
        assert_eq!(format!("{}", SymbolKind::Interface), "interface");
        assert_eq!(format!("{}", SymbolKind::TypeAlias), "type_alias");
        assert_eq!(format!("{}", SymbolKind::Constant), "constant");
        assert_eq!(format!("{}", SymbolKind::Enum), "enum");
        assert_eq!(format!("{}", SymbolKind::Module), "module");
        assert_eq!(format!("{}", SymbolKind::Macro), "macro");
    }

    // ── SymbolDependency Tests ───────────────────────────────────────────

    #[test]
    fn test_symbol_dependency_creation() {
        let dep = SymbolDependency {
            from_symbol: "parse".to_string(),
            to_symbol: "Config".to_string(),
            kind: SymbolKind::Class,
            strength: DependencyStrength::Strong,
        };

        assert_eq!(dep.from_symbol, "parse");
        assert_eq!(dep.to_symbol, "Config");
        assert_eq!(dep.kind, SymbolKind::Class);
        assert_eq!(dep.strength, DependencyStrength::Strong);
    }

    #[test]
    fn test_symbol_dependency_serialization_roundtrip() {
        let dep = SymbolDependency {
            from_symbol: "main".to_string(),
            to_symbol: "run_server".to_string(),
            kind: SymbolKind::Function,
            strength: DependencyStrength::Strong,
        };

        let json = serde_json::to_string(&dep).expect("serialize");
        let deserialized: SymbolDependency = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(dep, deserialized);
    }

    // ── Large Content Tests ──────────────────────────────────────────────

    #[test]
    fn test_fingerprint_large_content() {
        // 1MB of content
        let large_content: Vec<u8> = (0..1_000_000).map(|i| (i % 256) as u8).collect();
        let fp = AnalysisDefFingerprint::new(&large_content);
        assert!(fp.content_matches(&large_content));

        // Changing one byte should invalidate
        let mut modified = large_content.clone();
        modified[500_000] = modified[500_000].wrapping_add(1);
        assert!(!fp.content_matches(&modified));
    }

    #[test]
    fn test_fingerprint_binary_content() {
        // Binary content (null bytes, high bytes)
        let binary = vec![0u8, 1, 255, 128, 0, 0, 64, 32];
        let fp = AnalysisDefFingerprint::new(&binary);
        assert!(fp.content_matches(&binary));
    }
}
