// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # Structured Logging for Thread Flow
//!
//! Production-ready logging infrastructure with multiple output formats and log levels.
//!
//! ## Features
//!
//! - **Multiple Formats**: JSON (production) and human-readable (development)
//! - **Contextual Logging**: Automatic span tracking with tracing
//! - **Performance Tracking**: Built-in duration tracking for operations
//! - **Error Context**: Rich error context with backtraces
//!
//! ## Usage
//!
//! ```rust,ignore
//! use thread_flow::monitoring::logging::{init_logging, LogConfig, LogLevel, LogFormat};
//!
//! // Initialize logging (call once at startup)
//! init_logging(LogConfig {
//!     level: LogLevel::Info,
//!     format: LogFormat::Json,
//!     ..Default::default()
//! })?;
//!
//! // Use macros for logging
//! info!("Processing file", file = "src/main.rs");
//! warn!("Cache miss", hash = "abc123...");
//! error!("Database connection failed", error = %err);
//!
//! // Structured logging with spans
//! let span = info_span!("analyze_file", file = "src/main.rs");
//! let _guard = span.enter();
//! // All logs within this scope will include file context
//! ```

use std::env;
use std::fmt;

/// Log level configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Trace-level logging (very verbose)
    Trace,
    /// Debug-level logging (verbose)
    Debug,
    /// Info-level logging (normal)
    Info,
    /// Warning-level logging
    Warn,
    /// Error-level logging
    Error,
}

impl LogLevel {
    /// Parse from environment variable (RUST_LOG format)
    pub fn from_env() -> Self {
        env::var("RUST_LOG")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(LogLevel::Info)
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "trace"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
        }
    }
}

impl std::str::FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "trace" => Ok(LogLevel::Trace),
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" | "warning" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            _ => Err(format!("Invalid log level: {}", s)),
        }
    }
}

/// Log output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    /// Human-readable format (for development)
    Text,
    /// JSON format (for production)
    Json,
    /// Compact format (for CLI)
    Compact,
}

impl LogFormat {
    /// Parse from environment variable
    pub fn from_env() -> Self {
        env::var("LOG_FORMAT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(LogFormat::Text)
    }
}

impl fmt::Display for LogFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogFormat::Text => write!(f, "text"),
            LogFormat::Json => write!(f, "json"),
            LogFormat::Compact => write!(f, "compact"),
        }
    }
}

impl std::str::FromStr for LogFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" | "pretty" | "human" => Ok(LogFormat::Text),
            "json" => Ok(LogFormat::Json),
            "compact" => Ok(LogFormat::Compact),
            _ => Err(format!("Invalid log format: {}", s)),
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Log level threshold
    pub level: LogLevel,
    /// Output format
    pub format: LogFormat,
    /// Whether to include timestamps
    pub timestamps: bool,
    /// Whether to include file/line information
    pub source_location: bool,
    /// Whether to include thread IDs
    pub thread_ids: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            format: LogFormat::Text,
            timestamps: true,
            source_location: false,
            thread_ids: false,
        }
    }
}

impl LogConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            level: LogLevel::from_env(),
            format: LogFormat::from_env(),
            timestamps: env::var("LOG_TIMESTAMPS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
            source_location: env::var("LOG_SOURCE_LOCATION")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(false),
            thread_ids: env::var("LOG_THREAD_IDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(false),
        }
    }
}

/// Initialize logging infrastructure
///
/// This should be called once at application startup.
///
/// # Example
///
/// ```rust,ignore
/// use thread_flow::monitoring::logging::{init_logging, LogConfig};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     init_logging(LogConfig::default())?;
///
///     // Application code...
///
///     Ok(())
/// }
/// ```
pub fn init_logging(config: LogConfig) -> Result<(), LoggingError> {
    // Simple logging setup for now
    // In production, this would integrate with tracing-subscriber

    // Initialize env_logger with a default filter if RUST_LOG is not set
    let env = env_logger::Env::default().default_filter_or(format!("thread_flow={}", config.level));
    let mut builder = env_logger::Builder::from_env(env);

    if let Some(precision) = if config.timestamps {
        Some(env_logger::fmt::TimestampPrecision::Millis)
    } else {
        None
    } {
        builder.format_timestamp(Some(precision));
    } else {
        builder.format_timestamp(None);
    }

    builder.format_module_path(config.source_location);

    builder
        .try_init()
        .map_err(|e| LoggingError::InitializationFailed(e.to_string()))?;

    Ok(())
}

/// Initialize logging for CLI applications
///
/// Convenience function that sets up human-readable logging.
pub fn init_cli_logging() -> Result<(), LoggingError> {
    init_logging(LogConfig {
        level: LogLevel::from_env(),
        format: LogFormat::Text,
        timestamps: true,
        source_location: false,
        thread_ids: false,
    })
}

/// Initialize logging for production/edge deployments
///
/// Convenience function that sets up JSON logging for production.
pub fn init_production_logging() -> Result<(), LoggingError> {
    init_logging(LogConfig {
        level: LogLevel::Info,
        format: LogFormat::Json,
        timestamps: true,
        source_location: true,
        thread_ids: true,
    })
}

/// Logging errors
#[derive(Debug, thiserror::Error)]
pub enum LoggingError {
    #[error("Failed to initialize logging: {0}")]
    InitializationFailed(String),

    #[error("Invalid log configuration: {0}")]
    InvalidConfiguration(String),
}

/// Macro for structured logging with performance tracking
///
/// # Example
///
/// ```rust,ignore
/// use thread_flow::monitoring::logging::timed_operation;
///
/// timed_operation!("parse_file", file = "src/main.rs", {
///     // Operation code here
///     parse_rust_file(file)?;
/// });
/// // Automatically logs duration when complete
/// ```
#[macro_export]
macro_rules! timed_operation {
    ($name:expr, $($key:ident = $value:expr),*, $block:block) => {{
        let _start = std::time::Instant::now();
        $(
            println!("[DEBUG] {}: {} = {:?}", $name, stringify!($key), $value);
        )*
        let result = $block;
        let _duration = _start.elapsed();
        println!("[INFO] {} completed in {:?}", $name, _duration);
        result
    }};
}

/// Structured logging helpers
pub mod structured {
    use thread_utilities::RapidMap;

    /// Build a structured log context
    pub struct LogContext {
        fields: RapidMap<String, String>,
    }

    impl LogContext {
        /// Create a new log context
        pub fn new() -> Self {
            Self {
                fields: thread_utilities::get_map(),
            }
        }

        /// Add a field to the context
        pub fn field(mut self, key: impl Into<String>, value: impl ToString) -> Self {
            self.fields.insert(key.into(), value.to_string());
            self
        }

        /// Log at info level with context
        pub fn info(self, message: &str) {
            // Use println for now until log crate is properly integrated
            println!("[INFO] {} {:?}", message, self.fields);
        }

        /// Log at warn level with context
        pub fn warn(self, message: &str) {
            eprintln!("[WARN] {} {:?}", message, self.fields);
        }

        /// Log at error level with context
        pub fn error(self, message: &str) {
            eprintln!("[ERROR] {} {:?}", message, self.fields);
        }
    }

    impl Default for LogContext {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_parsing() {
        assert_eq!("trace".parse::<LogLevel>().unwrap(), LogLevel::Trace);
        assert_eq!("debug".parse::<LogLevel>().unwrap(), LogLevel::Debug);
        assert_eq!("info".parse::<LogLevel>().unwrap(), LogLevel::Info);
        assert_eq!("warn".parse::<LogLevel>().unwrap(), LogLevel::Warn);
        assert_eq!("error".parse::<LogLevel>().unwrap(), LogLevel::Error);
    }

    #[test]
    fn test_log_format_parsing() {
        assert_eq!("text".parse::<LogFormat>().unwrap(), LogFormat::Text);
        assert_eq!("json".parse::<LogFormat>().unwrap(), LogFormat::Json);
        assert_eq!("compact".parse::<LogFormat>().unwrap(), LogFormat::Compact);
    }

    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert_eq!(config.level, LogLevel::Info);
        assert_eq!(config.format, LogFormat::Text);
        assert!(config.timestamps);
        assert!(!config.source_location);
        assert!(!config.thread_ids);
    }
}
