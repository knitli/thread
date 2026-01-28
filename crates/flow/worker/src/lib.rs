// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: PROPRIETARY

//! Thread code analysis worker for Cloudflare Workers.
//!
//! Provides HTTP API for edge-based code analysis with D1 storage.
//!
//! ## API Endpoints
//!
//! ### POST /analyze
//! Analyze source code files and store results in D1.
//!
//! ```json
//! {
//!   "files": [
//!     {
//!       "path": "src/main.rs",
//!       "content": "fn main() { println!(\"Hello\"); }"
//!     }
//!   ],
//!   "language": "rust"
//! }
//! ```
//!
//! ### GET /health
//! Health check endpoint.
//!
//! ### GET /symbols/{file_path}
//! Query symbols for a specific file.

use serde::{Deserialize, Serialize};
use worker::*;

mod error;
mod handlers;
mod types;

use error::WorkerError;
use handlers::{handle_analyze, handle_health, handle_query_symbols};
use types::{AnalyzeRequest, AnalyzeResponse};

/// Main entry point for Cloudflare Worker.
///
/// Routes requests to appropriate handlers based on path and method.
#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Set up panic hook for better error messages
    console_error_panic_hook::set_once();

    // Initialize logging
    console_log::init_with_level(log::Level::Info).ok();

    // Route requests
    Router::new()
        .post_async("/analyze", |mut req, ctx| async move {
            handle_analyze(req, ctx).await
        })
        .get_async("/symbols/:file_path", |_req, ctx| async move {
            handle_query_symbols(ctx).await
        })
        .get("/health", |_req, _ctx| {
            handle_health()
        })
        .run(req, env)
        .await
}
