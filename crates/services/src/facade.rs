// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # Thread Service Facade
//!
//! This module provides a simplified high-level interface for consuming Thread services.
//! It hides the complexity of underlying dataflow graphs and storage implementations,
//! offering a clean API for CLI, LSP, and other tools.

use crate::error::ServiceResult;
use crate::traits::CodeAnalyzer;
#[cfg(feature = "storage-traits")]
use crate::traits::StorageService;
use crate::types::ParsedDocument;
use std::path::Path;
use std::sync::Arc;

/// Main entry point for Thread services.
///
/// The Facade pattern is used here to provide a simplified interface to a
/// complex subsystem (the CocoIndex dataflow engine and storage backend).
pub struct ThreadService<A: CodeAnalyzer<D>, D: crate::types::Doc + Send + Sync> {
    #[allow(dead_code)]
    analyzer: Arc<A>,
    #[cfg(feature = "storage-traits")]
    storage: Option<Arc<dyn StorageService>>,
    _marker: std::marker::PhantomData<D>,
}

impl<A: CodeAnalyzer<D>, D: crate::types::Doc + Send + Sync> ThreadService<A, D> {
    /// Create a new ThreadService with provided components
    #[cfg(feature = "storage-traits")]
    pub fn new(analyzer: Arc<A>, storage: Option<Arc<dyn StorageService>>) -> Self {
        Self {
            analyzer,
            storage,
            _marker: std::marker::PhantomData,
        }
    }

    #[cfg(not(feature = "storage-traits"))]
    pub fn new(analyzer: Arc<A>) -> Self {
        Self {
            analyzer,
            _marker: std::marker::PhantomData,
        }
    }

    /// Analyze a single file or directory path.
    ///
    /// This method orchestrates the analysis process:
    /// 1. Discovers files (if path is directory)
    /// 2. Parses and analyzes code
    /// 3. Stores results (if storage is configured)
    pub async fn analyze_path(&self, _path: &Path) -> ServiceResult<Vec<ParsedDocument<D>>> {
        // Implementation would delegate to analyzer
        // This is a placeholder for the facade interface

        Ok(vec![])
    }
}
