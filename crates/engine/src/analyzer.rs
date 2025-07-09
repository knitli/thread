// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later


//! High-level analysis orchestration

use crate::*;
use thread_core::*;
use std::path::Path;

/// High-level analyzer that orchestrates the full analysis pipeline
pub struct Analyzer {
    engine: ThreadEngine,
    config: AnalysisConfig,
}

impl Analyzer {
    /// Create a new analyzer with default configuration
    pub fn new() -> Self {
        Self {
            engine: ThreadEngine::new(),
            config: AnalysisConfig::default(),
        }
    }

    /// Create a new analyzer with custom configuration
    pub fn with_config(config: AnalysisConfig) -> Self {
        Self {
            engine: ThreadEngine::new(),
            config,
        }
    }

    /// Analyze a single Rust file (Day 2 MVP target)
    pub fn analyze_rust_file(&mut self, content: &str, file_path: &Path) -> Result<AnalysisResult> {
        // TODO: This is the Day 2 deliverable target
        // 1. Use thread-parse to extract functions from Rust content
        // 2. Build graph representation
        // 3. Return analysis result

        self.engine.analyze_file(file_path, content)
    }

    /// Get current engine statistics
    pub fn stats(&self) -> EngineStats {
        self.engine.stats()
    }

    /// Get a reference to the underlying engine
    pub fn engine(&self) -> &ThreadEngine {
        &self.engine
    }

    /// Get a mutable reference to the underlying engine
    pub fn engine_mut(&mut self) -> &mut ThreadEngine {
        &mut self.engine
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for the analysis process
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    pub include_private: bool,
    pub include_tests: bool,
    pub max_file_size_mb: usize,
    pub extract_docstrings: bool,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            include_private: true,
            include_tests: false,
            max_file_size_mb: 10,
            extract_docstrings: true,
        }
    }
}
