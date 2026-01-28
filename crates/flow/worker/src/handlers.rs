// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: PROPRIETARY

//! HTTP request handlers for Thread Worker API.

use worker::{Request, Response, RouteContext};
use crate::error::WorkerError;
use crate::types::{AnalyzeRequest, AnalyzeResponse, AnalysisStatus, FileHash};
use std::time::Instant;

/// Handle POST /analyze - Analyze source code files.
pub async fn handle_analyze(
    mut req: Request,
    ctx: RouteContext<()>,
) -> worker::Result<Response> {
    let start = Instant::now();

    // Parse request body
    let request: AnalyzeRequest = match req.json().await {
        Ok(r) => r,
        Err(e) => {
            return WorkerError::InvalidRequest(format!("Invalid JSON: {}", e)).to_response();
        }
    };

    // Validate request
    if request.files.is_empty() {
        return WorkerError::InvalidRequest("No files provided".to_string()).to_response();
    }

    log::info!("Analyzing {} files", request.files.len());

    // Get D1 bindings from environment
    let env = ctx.env;
    let account_id = match env.var("D1_ACCOUNT_ID") {
        Ok(v) => v.to_string(),
        Err(_) => {
            return WorkerError::Internal("D1_ACCOUNT_ID not configured".to_string()).to_response();
        }
    };

    let database_id = match env.var("D1_DATABASE_ID") {
        Ok(v) => v.to_string(),
        Err(_) => {
            return WorkerError::Internal("D1_DATABASE_ID not configured".to_string()).to_response();
        }
    };

    let api_token = match env.secret("D1_API_TOKEN") {
        Ok(v) => v.to_string(),
        Err(_) => {
            return WorkerError::Internal("D1_API_TOKEN not configured".to_string()).to_response();
        }
    };

    // TODO: Implement actual Thread analysis pipeline
    // This is a placeholder - actual implementation would:
    // 1. Parse each file with thread-ast-engine
    // 2. Extract symbols, imports, calls
    // 3. Compute content hashes
    // 4. Upsert to D1 using thread-flow D1 target
    //
    // For now, return mock response
    let response = AnalyzeResponse {
        status: AnalysisStatus::Success,
        files_analyzed: request.files.len(),
        symbols_extracted: 0, // Would be computed from actual analysis
        imports_found: 0,
        calls_found: 0,
        duration_ms: start.elapsed().as_millis() as u64,
        content_hashes: request
            .files
            .iter()
            .map(|f| FileHash {
                file_path: f.path.clone(),
                content_hash: "placeholder_hash".to_string(),
                cached: false,
            })
            .collect(),
    };

    Response::from_json(&response)
}

/// Handle GET /symbols/{file_path} - Query symbols for a file.
pub async fn handle_query_symbols(ctx: RouteContext<()>) -> worker::Result<Response> {
    let file_path = match ctx.param("file_path") {
        Some(path) => path,
        None => {
            return WorkerError::InvalidRequest("Missing file_path parameter".to_string())
                .to_response();
        }
    };

    log::info!("Querying symbols for: {}", file_path);

    // TODO: Implement D1 query
    // For now, return mock response
    Response::from_json(&serde_json::json!({
        "file_path": file_path,
        "symbols": []
    }))
}

/// Handle GET /health - Health check.
pub fn handle_health() -> worker::Result<Response> {
    Response::from_json(&serde_json::json!({
        "status": "healthy",
        "service": "thread-worker",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
