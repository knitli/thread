/*!
This module defines the service layer interfaces for Thread.

It provides abstract traits and execution contexts that decouple the core
functionality from specific I/O, configuration, and execution environments.
This allows the same core logic to be used in CLI tools, WASM environments,
cloud services, and other contexts.
*/

use std::path::Path;
use thiserror::Error;

/// Error types for service operations
#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Execution error: {0}")]
    Execution(String),
}

/// Abstract execution context that can provide code from various sources
pub trait ExecutionContext {
    /// Read content from a source (could be file, memory, network, etc.)
    fn read_content(&self, source: &str) -> Result<String, ServiceError>;

    /// Write content to a destination
    fn write_content(&self, destination: &str, content: &str) -> Result<(), ServiceError>;

    /// List available sources (files, URLs, etc.)
    fn list_sources(&self) -> Result<Vec<String>, ServiceError>;
}

/// File system based execution context
pub struct FileSystemContext {
    base_path: std::path::PathBuf,
}

impl FileSystemContext {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }
}

impl ExecutionContext for FileSystemContext {
    fn read_content(&self, source: &str) -> Result<String, ServiceError> {
        let path = self.base_path.join(source);
        Ok(std::fs::read_to_string(path)?)
    }

    fn write_content(&self, destination: &str, content: &str) -> Result<(), ServiceError> {
        let path = self.base_path.join(destination);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(std::fs::write(path, content)?)
    }

    fn list_sources(&self) -> Result<Vec<String>, ServiceError> {
        // Basic implementation - can be enhanced with glob patterns, etc.
        let mut sources = Vec::new();
        for entry in std::fs::read_dir(&self.base_path)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                if let Some(name) = entry.file_name().to_str() {
                    sources.push(name.to_string());
                }
            }
        }
        Ok(sources)
    }
}

/// In-memory execution context for testing and WASM environments
pub struct MemoryContext {
    content: std::collections::HashMap<String, String>,
}

impl MemoryContext {
    pub fn new() -> Self {
        Self {
            content: std::collections::HashMap::new(),
        }
    }

    pub fn add_content(&mut self, name: String, content: String) {
        self.content.insert(name, content);
    }
}

impl Default for MemoryContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionContext for MemoryContext {
    fn read_content(&self, source: &str) -> Result<String, ServiceError> {
        self.content
            .get(source)
            .cloned()
            .ok_or_else(|| ServiceError::Execution(format!("Source not found: {}", source)))
    }

    fn write_content(&self, _destination: &str, _content: &str) -> Result<(), ServiceError> {
        // For read-only memory context, we could store writes separately
        // or return an error. For now, we'll just succeed silently.
        Ok(())
    }

    fn list_sources(&self) -> Result<Vec<String>, ServiceError> {
        Ok(self.content.keys().cloned().collect())
    }
}

// Service trait definitions will be added here in future iterations
// For example:
// pub trait ScanService { ... }
// pub trait FixService { ... }
// pub trait RuleValidationService { ... }
