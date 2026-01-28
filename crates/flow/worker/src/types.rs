// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: PROPRIETARY

//! Request and response types for Thread Worker API.

use serde::{Deserialize, Serialize};

/// Request to analyze source code files.
#[derive(Debug, Clone, Deserialize)]
pub struct AnalyzeRequest {
    /// Files to analyze with their content.
    pub files: Vec<FileContent>,

    /// Programming language (optional, auto-detected if not provided).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Repository URL (optional metadata).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_url: Option<String>,

    /// Branch name (optional metadata).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
}

/// File content for analysis.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileContent {
    /// File path (relative to repository root).
    pub path: String,

    /// Source code content.
    pub content: String,
}

/// Response from analysis operation.
#[derive(Debug, Clone, Serialize)]
pub struct AnalyzeResponse {
    /// Analysis status.
    pub status: AnalysisStatus,

    /// Number of files analyzed.
    pub files_analyzed: usize,

    /// Number of symbols extracted.
    pub symbols_extracted: usize,

    /// Number of imports found.
    pub imports_found: usize,

    /// Number of function calls found.
    pub calls_found: usize,

    /// Analysis duration in milliseconds.
    pub duration_ms: u64,

    /// Content hash for incremental updates.
    pub content_hashes: Vec<FileHash>,
}

/// Analysis status.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AnalysisStatus {
    Success,
    Partial,
    Failed,
}

/// File content hash for incremental updates.
#[derive(Debug, Clone, Serialize)]
pub struct FileHash {
    pub file_path: String,
    pub content_hash: String,
    pub cached: bool,
}

/// Symbol query response.
#[derive(Debug, Clone, Serialize)]
pub struct SymbolsResponse {
    pub file_path: String,
    pub symbols: Vec<Symbol>,
}

/// Code symbol information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: String,
    pub scope: Option<String>,
    pub line_start: Option<i32>,
    pub line_end: Option<i32>,
}
