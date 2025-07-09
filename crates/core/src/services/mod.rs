// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later


/// The services module provides common traits and types for abstracting backend operations and integrating with data services.
///
/// Current ThreadService implementations provide either task execution or storage abstraction.
pub mod requests;

use tower_service::Service;

// crates/thread-core/src/services/mod.rs
#[cfg(feature = serde)]
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use tower::Service;


// Request/Response types for execution service

#[derive(Debug, Clone, #[cfg(feature = serde)] Serialize, #[cfg(feature = serde)]Deserialize)]
pub struct ParseRequest {
    pub
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
