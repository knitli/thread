// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later


// Request and Response types for Execution, Storage, and Editing Services

use tower_service::Service;

// crates/thread-core/src/services/mod.rs
#[cfg(feature = serde)]
use serde::{Deserialize, Serialize};

use crate::fastmap::FastMap;
use tower::Service;

#[non_exhaustive]
#[derive(Debug, Clone, Default, #[cfg(feature = serde)] Serialize, #[cfg(feature = serde)]Deserialize)]
pub enum ScanType {
    #[default]
    Full,
    Incremental,
    Unspecified,
}

pub fn register_new_scan_types(scan_types: &mut FastMap<String, ScanType>, name: &str, scan_type: ScanType) {
    scan_types.insert(name.to_string(), scan_type);
}

#[non_exhaustive]
#[derive(Debug, Clone, Default, #[cfg(feature = serde)] Serialize, #[cfg(feature = serde)]Deserialize)]
pub enum ScanMedium {
    #[default]
    Filesystem,
    Network,
}

/// Registers new scan mediums in the provided FastMap.
pub fn register_new_scan_mediums(scan_mediums: &mut FastMap<String, ScanMedium>, name: &str, scan_medium: ScanMedium) {
    scan_mediums.insert(name.to_string(), scan_medium);
}



/// Request/Response types for execution service

#[derive(Debug, Clone, #[cfg(feature = serde)] Serialize, #[cfg(feature = serde)]Deserialize)]
pub struct ScanRequest {
    pub scan_type: ScanType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResponse {
    pub functions: Vec<Function>,
    pub imports: Vec<Import>,
    pub file_hash: String,
    pub line_count: usize,
}

#[derive(Debug, Clone)]
pub struct BatchParseRequest {
    pub tasks: Vec<ParseRequest>,
}

#[derive(Debug, Clone)]
pub struct BatchParseResponse {
    pub results: Vec<ParseResponse>,
}

// Request/Response types for storage service
pub type ContentHash = String;

#[derive(Debug, Clone)]
pub struct StoreContentRequest {
    pub hash: ContentHash,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct GetContentRequest {
    pub hash: ContentHash,
}

#[derive(Debug, Clone)]
pub struct ContentResponse {
    pub content: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StoreAnalysisRequest {
    pub hash: ContentHash,
    pub analysis: ParseResponse,
}

#[derive(Debug, Clone)]
pub struct GetAnalysisRequest {
    pub hash: ContentHash,
}

#[derive(Debug, Clone)]
pub struct AnalysisResponse {
    pub analysis: Option<ParseResponse>,
}
