// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later


//! Types and structures for Thread scanning operations.
/// Represents the types of scans that can be performed on a project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanType {
    // Scan the entire project, including all files and directories
    Full,
    // Scan only the files that have changed since the last scan
    OnChange,
    // Scan only the files that are requested on demand
    OnDemand,
    // Scan at regular intervals (like cron jobs, timed scans)
    OnInterval,
    // Custom scan type, can be used for specific use cases
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanOptions {
    pub search_pattern: Option<String>,
    pub exclude_pattern: Option<String>,
    pub only_public: bool,
    pub include_tests: bool,
    pub include_docs: bool,
    pub include_comments: bool,
    pub respect_gitignore: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanRequest {
    pub id: String,
    pub scan_type: ScanType,
    pub root_path: String,
    pub options: ScanOptions,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            search_pattern: None,
            exclude_pattern: None,
            only_public: false,
            include_tests: false,
            include_docs: true,
            include_comments: true,
            respect_gitignore: true,
        }
    }
}
