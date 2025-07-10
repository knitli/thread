//! Shared types and traits for ag-service single-function crates.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Main error type for all ag-service crates.
#[derive(Debug, Error)]
pub enum AstGrepError {
    #[error("Source error: {0}")]
    Source(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Language error: {0}")]
    Language(String),
    #[error("Pattern error: {0}")]
    Pattern(String),
    #[error("Fix error: {0}")]
    Fix(String),
    #[error("Output error: {0}")]
    Output(String),
    #[error("Checks error: {0}")]
    Checks(String),
    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, AstGrepError>;

/// Options for scanning files with rules.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanOptions {
    pub context_lines_before: usize,
    pub context_lines_after: usize,
    pub include_metadata: bool,
    pub severity_filter: Option<String>,
    pub rule_filter: Option<String>,
    pub interactive: bool,
}

/// Options for searching with a pattern.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchOptions {
    pub strictness: Option<String>,
    pub selector: Option<String>,
    pub context_lines_before: usize,
    pub context_lines_after: usize,
}

/// Options for applying fixes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FixOptions {
    pub dry_run: bool,
    pub interactive: bool,
}

/// Trait for a generic input source adapter.
#[async_trait]
pub trait SourceAdapter: Send + Sync {
    type Item: Send + Sync;
    type Error: std::error::Error + Send + Sync;

    async fn read_items(&self) -> std::result::Result<Vec<Self::Item>, Self::Error>;
}

/// Trait for providing content and language for an identifier.
#[async_trait]
pub trait ContentProvider: Send + Sync {
    async fn get_content(
        &self,
        identifier: &str,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_language(&self, identifier: &str) -> Option<String>;
}

/// Trait for a generic output sink adapter.
#[async_trait]
pub trait SinkAdapter<T>: Send + Sync {
    type Error: std::error::Error + Send + Sync;

    async fn write_results(&self, results: Vec<T>) -> std::result::Result<(), Self::Error>;
}

/// Trait for a configuration provider.
#[async_trait]
pub trait ConfigProvider: Send + Sync {
    async fn get_rules(
        &self,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_project_config(
        &self,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>>;
}
