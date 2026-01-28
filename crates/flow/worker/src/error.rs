// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: PROPRIETARY

//! Error types for Thread Worker.

use thiserror::Error;
use worker::Response;

#[derive(Debug, Error)]
pub enum WorkerError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Analysis failed: {0}")]
    AnalysisFailed(String),

    #[error("D1 error: {0}")]
    D1Error(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<WorkerError> for worker::Error {
    fn from(err: WorkerError) -> Self {
        worker::Error::RustError(err.to_string())
    }
}

impl WorkerError {
    /// Convert error to HTTP response.
    pub fn to_response(&self) -> worker::Result<Response> {
        let (status, message) = match self {
            WorkerError::InvalidRequest(msg) => (400, msg.clone()),
            WorkerError::AnalysisFailed(msg) => (500, format!("Analysis failed: {}", msg)),
            WorkerError::D1Error(msg) => (500, format!("Database error: {}", msg)),
            WorkerError::Internal(msg) => (500, format!("Internal error: {}", msg)),
        };

        Response::error(message, status)
    }
}
