// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # Service Layer Error Types
//!
//! Comprehensive error handling for the Thread service layer using Tower's BoxError pattern
//! for unified error handling and improved performance. This design enables composable
//! service layers while maintaining excellent error context and recovery capabilities.

use std::borrow::Cow;
use std::fmt;
use std::path::PathBuf;
use thiserror::Error;

// Import Tower's BoxError pattern for unified error handling
#[cfg(feature = "tower-services")]
pub use tower::BoxError;

#[cfg(not(feature = "tower-services"))]
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

// Conditionally import thread dependencies when available
#[cfg(feature = "ast-grep-backend")]
use thread_ast_engine::tree_sitter::TSParseError;

#[cfg(feature = "ast-grep-backend")]
use thread_language::SupportLangErr;

#[cfg(all(feature = "matching", feature = "ast-grep-backend"))]
use thread_ast_engine::PatternError;

/// Service result type using Tower's BoxError pattern for unified error handling
pub type ServiceResult<T> = Result<T, BoxError>;

/// Main error type for service layer operations with optimized string handling
#[derive(Error, Debug)]
pub enum ServiceError {
    /// Errors related to parsing source code
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    /// Errors related to code analysis and pattern matching
    #[error("Analysis error: {0}")]
    Analysis(#[from] AnalysisError),

    /// Errors related to storage operations
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    /// Errors related to execution context
    #[error("Execution error: {message}")]
    Execution { message: Cow<'static, str> },

    /// I/O related errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Configuration errors with optimized string storage
    #[error("Configuration error: {message}")]
    Config { message: Cow<'static, str> },

    /// Language support errors (when ast-grep-backend available)
    #[cfg(feature = "ast-grep-backend")]
    #[error("Language error: {0}")]
    Language(#[from] SupportLangErr),

    /// Timeout errors with duration context
    #[error("Operation timed out after {duration:?}: {operation}")]
    Timeout {
        operation: Cow<'static, str>,
        duration: std::time::Duration,
    },

    /// Concurrency/threading errors
    #[error("Concurrency error: {message}")]
    Concurrency { message: Cow<'static, str> },

    /// Generic service errors
    #[error("Service error: {message}")]
    Generic { message: Cow<'static, str> },
}

// Note: ServiceError already implements Error, so it automatically converts to BoxError via alloc's blanket impl

// Helper functions for creating optimized error instances
impl ServiceError {
    /// Create execution error with static string (zero allocation)
    pub fn execution_static(msg: &'static str) -> Self {
        Self::Execution {
            message: Cow::Borrowed(msg),
        }
    }

    /// Create execution error with dynamic string
    pub fn execution_dynamic(msg: String) -> Self {
        Self::Execution {
            message: Cow::Owned(msg),
        }
    }

    /// Create config error with static string (zero allocation)
    pub fn config_static(msg: &'static str) -> Self {
        Self::Config {
            message: Cow::Borrowed(msg),
        }
    }

    /// Create config error with dynamic string
    pub fn config_dynamic(msg: String) -> Self {
        Self::Config {
            message: Cow::Owned(msg),
        }
    }

    /// Create timeout error with operation context
    pub fn timeout(operation: impl Into<Cow<'static, str>>, duration: std::time::Duration) -> Self {
        Self::Timeout {
            operation: operation.into(),
            duration,
        }
    }
}

/// Errors related to parsing source code into ASTs with performance optimization
#[derive(Error, Debug)]
pub enum ParseError {
    /// Tree-sitter parsing failed (when ast-grep-backend available)
    #[cfg(feature = "ast-grep-backend")]
    #[error("Tree-sitter parse error: {0}")]
    TreeSitter(#[from] TSParseError),

    /// Language not supported
    #[error("Language not supported for file: {file_path}")]
    UnsupportedLanguage { file_path: PathBuf },

    /// Language could not be detected
    #[error("Could not detect language for file: {file_path}")]
    LanguageDetectionFailed { file_path: PathBuf },

    /// File could not be read
    #[error("Could not read file: {file_path}: {source}")]
    FileRead {
        file_path: PathBuf,
        source: BoxError,
    },

    /// Source code is empty or invalid with optimized string storage
    #[error("Invalid source code: {message}")]
    InvalidSource { message: Cow<'static, str> },

    /// Content is too large to process
    #[error("Content too large: {size} bytes (max: {max_size})")]
    ContentTooLarge { size: usize, max_size: usize },

    /// Generic parsing error with optimized string storage
    #[error("Parse error: {message}")]
    Generic { message: Cow<'static, str> },

    /// Encoding issues
    #[error("Encoding error in file {file_path}: {message}")]
    Encoding { file_path: PathBuf, message: String },

    /// Parser configuration errors
    #[error("Parser configuration error: {message}")]
    Configuration { message: String },
}

/// Errors related to code analysis and pattern matching
#[derive(Error, Debug)]
pub enum AnalysisError {
    /// Pattern matching errors
    #[cfg(feature = "matching")]
    #[error("Pattern error: {0}")]
    Pattern(#[from] PatternError),

    /// Pattern compilation failed
    #[error("Pattern compilation failed: {pattern}: {message}")]
    PatternCompilation { pattern: String, message: String },

    /// Invalid pattern syntax
    #[error("Invalid pattern syntax: {pattern}")]
    InvalidPattern { pattern: String },

    /// Meta-variable errors
    #[error("Meta-variable error: {variable}: {message}")]
    MetaVariable { variable: String, message: String },

    /// Cross-file analysis errors
    #[error("Cross-file analysis error: {message}")]
    CrossFile { message: String },

    /// Graph construction errors
    #[error("Graph construction error: {message}")]
    GraphConstruction { message: String },

    /// Dependency resolution errors
    #[error("Dependency resolution error: {message}")]
    DependencyResolution { message: String },

    /// Symbol resolution errors
    #[error("Symbol resolution error: symbol '{symbol}' in file {file_path}")]
    SymbolResolution { symbol: String, file_path: PathBuf },

    /// Type analysis errors
    #[error("Type analysis error: {message}")]
    TypeAnalysis { message: String },

    /// Scope analysis errors
    #[error("Scope analysis error: {message}")]
    ScopeAnalysis { message: String },

    /// Analysis depth limit exceeded
    #[error("Analysis depth limit exceeded: {current_depth} > {max_depth}")]
    DepthLimitExceeded {
        current_depth: usize,
        max_depth: usize,
    },

    /// Analysis operation cancelled
    #[error("Analysis operation cancelled: {reason}")]
    Cancelled { reason: String },

    /// Resource exhaustion during analysis
    #[error("Resource exhaustion: {resource}: {message}")]
    ResourceExhaustion { resource: String, message: String },
}

/// Errors related to storage operations (commercial boundary)
#[derive(Error, Debug)]
pub enum StorageError {
    /// Database connection errors
    #[error("Database connection error: {message}")]
    Connection { message: String },

    /// Database query errors
    #[error("Database query error: {query}: {message}")]
    Query { query: String, message: String },

    /// Serialization errors
    #[error("Serialization error: {message}")]
    Serialization { message: String },

    /// Deserialization errors
    #[error("Deserialization error: {message}")]
    Deserialization { message: String },

    /// Cache errors
    #[error("Cache error: {operation}: {message}")]
    Cache { operation: String, message: String },

    /// Transaction errors
    #[error("Transaction error: {message}")]
    Transaction { message: String },

    /// Storage quota exceeded
    #[error("Storage quota exceeded: {used} > {limit}")]
    QuotaExceeded { used: u64, limit: u64 },

    /// Storage corruption detected
    #[error("Storage corruption detected: {message}")]
    Corruption { message: String },

    /// Backup/restore errors
    #[error("Backup/restore error: {operation}: {message}")]
    BackupRestore { operation: String, message: String },
}

/// Context information for errors
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// File being processed when error occurred
    pub file_path: Option<PathBuf>,

    /// Line number where error occurred
    pub line: Option<usize>,

    /// Column where error occurred
    pub column: Option<usize>,

    /// Operation being performed
    pub operation: Option<String>,

    /// Additional context data
    pub context_data: std::collections::HashMap<String, String>,
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self {
            file_path: None,
            line: None,
            column: None,
            operation: None,
            context_data: std::collections::HashMap::new(),
        }
    }
}

impl ErrorContext {
    /// Create new error context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set file path
    pub fn with_file_path(mut self, file_path: PathBuf) -> Self {
        self.file_path = Some(file_path);
        self
    }

    /// Set line number
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    /// Set column number
    pub fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    /// Set operation name
    pub fn with_operation(mut self, operation: String) -> Self {
        self.operation = Some(operation);
        self
    }

    /// Add context data
    pub fn with_context_data(mut self, key: String, value: String) -> Self {
        self.context_data.insert(key, value);
        self
    }
}

/// Enhanced error type that includes context information
#[derive(Error, Debug)]
pub struct ContextualError {
    /// The underlying error
    pub error: ServiceError,

    /// Additional context information
    pub context: ErrorContext,
}

impl fmt::Display for ContextualError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)?;

        if let Some(ref file_path) = self.context.file_path {
            write!(f, " (file: {})", file_path.display())?;
        }

        if let Some(line) = self.context.line {
            write!(f, " (line: {})", line)?;
        }

        if let Some(column) = self.context.column {
            write!(f, " (column: {})", column)?;
        }

        if let Some(ref operation) = self.context.operation {
            write!(f, " (operation: {})", operation)?;
        }

        Ok(())
    }
}

impl From<ServiceError> for ContextualError {
    fn from(error: ServiceError) -> Self {
        Self {
            error,
            context: ErrorContext::default(),
        }
    }
}

/// Compatibility type for legacy ServiceError usage
pub type LegacyServiceResult<T> = Result<T, ServiceError>;

/// Result type for contextual operations
pub type ContextualResult<T> = Result<T, ContextualError>;

/// Helper trait for adding context to errors
pub trait ErrorContextExt {
    type Output;

    /// Add context to the error
    fn with_context(self, context: ErrorContext) -> Self::Output;

    /// Add file path context
    fn with_file(self, file_path: PathBuf) -> Self::Output;

    /// Add line context
    fn with_line(self, line: usize) -> Self::Output;

    /// Add operation context
    fn with_operation(self, operation: &str) -> Self::Output;
}

impl<T> ErrorContextExt for Result<T, ServiceError> {
    type Output = ContextualResult<T>;

    fn with_context(self, context: ErrorContext) -> Self::Output {
        self.map_err(|error| ContextualError { error, context })
    }

    fn with_file(self, file_path: PathBuf) -> Self::Output {
        self.with_context(ErrorContext::new().with_file_path(file_path))
    }

    fn with_line(self, line: usize) -> Self::Output {
        self.with_context(ErrorContext::new().with_line(line))
    }

    fn with_operation(self, operation: &str) -> Self::Output {
        self.with_context(ErrorContext::new().with_operation(operation.to_string()))
    }
}

/// Error recovery strategies
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Retry the operation
    Retry { max_attempts: usize },

    /// Skip the current item and continue
    Skip,

    /// Use a fallback approach
    Fallback { strategy: String },

    /// Abort the entire operation
    Abort,

    /// Continue with partial results
    Partial,
}

/// Error recovery information
#[derive(Debug, Clone)]
pub struct ErrorRecovery {
    /// Suggested recovery strategy
    pub strategy: RecoveryStrategy,

    /// Human-readable recovery instructions
    pub instructions: String,

    /// Whether automatic recovery is possible
    pub auto_recoverable: bool,
}

/// Trait for errors that support recovery
pub trait RecoverableError {
    /// Get recovery information for this error
    fn recovery_info(&self) -> Option<ErrorRecovery>;

    /// Check if this error is retryable
    fn is_retryable(&self) -> bool {
        matches!(
            self.recovery_info(),
            Some(ErrorRecovery {
                strategy: RecoveryStrategy::Retry { .. },
                ..
            })
        )
    }

    /// Check if this error allows partial continuation
    fn allows_partial(&self) -> bool {
        matches!(
            self.recovery_info(),
            Some(ErrorRecovery {
                strategy: RecoveryStrategy::Partial | RecoveryStrategy::Skip,
                ..
            })
        )
    }
}

impl RecoverableError for ServiceError {
    fn recovery_info(&self) -> Option<ErrorRecovery> {
        match self {
            #[cfg(feature = "ast-grep-backend")]
            ServiceError::Parse(ParseError::TreeSitter(_)) => Some(ErrorRecovery {
                strategy: RecoveryStrategy::Retry { max_attempts: 3 },
                instructions: "Tree-sitter parsing failed. Retry with error recovery enabled."
                    .to_string(),
                auto_recoverable: true,
            }),

            #[cfg(all(feature = "matching", feature = "ast-grep-backend"))]
            ServiceError::Analysis(AnalysisError::PatternCompilation { .. }) => {
                Some(ErrorRecovery {
                    strategy: RecoveryStrategy::Skip,
                    instructions: "Pattern compilation failed. Skip this pattern and continue."
                        .to_string(),
                    auto_recoverable: true,
                })
            }

            ServiceError::Io(_) => Some(ErrorRecovery {
                strategy: RecoveryStrategy::Retry { max_attempts: 3 },
                instructions: "I/O operation failed. Retry with exponential backoff.".to_string(),
                auto_recoverable: true,
            }),

            ServiceError::Timeout { .. } => Some(ErrorRecovery {
                strategy: RecoveryStrategy::Retry { max_attempts: 2 },
                instructions: "Operation timed out. Retry with increased timeout.".to_string(),
                auto_recoverable: true,
            }),

            ServiceError::Storage(StorageError::Connection { .. }) => Some(ErrorRecovery {
                strategy: RecoveryStrategy::Retry { max_attempts: 5 },
                instructions: "Storage connection failed. Retry with exponential backoff."
                    .to_string(),
                auto_recoverable: true,
            }),

            _ => None,
        }
    }
}

/// Macro for creating parse errors with context
#[macro_export]
macro_rules! parse_error {
    ($variant:ident, $($field:ident: $value:expr),* $(,)?) => {
        $crate::error::ServiceError::Parse(
            $crate::error::ParseError::$variant {
                $($field: $value,)*
            }
        )
    };
}

/// Macro for creating analysis errors with context
#[macro_export]
macro_rules! analysis_error {
    ($variant:ident, $($field:ident: $value:expr),* $(,)?) => {
        $crate::error::ServiceError::Analysis(
            $crate::error::AnalysisError::$variant {
                $($field: $value,)*
            }
        )
    };
}

/// Macro for creating storage errors with context
#[macro_export]
macro_rules! storage_error {
    ($variant:ident, $($field:ident: $value:expr),* $(,)?) => {
        $crate::error::ServiceError::Storage(
            $crate::error::StorageError::$variant {
                $($field: $value,)*
            }
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_error_context() {
        let context = ErrorContext::new()
            .with_file_path(PathBuf::from("test.rs"))
            .with_line(42)
            .with_operation("pattern_matching".to_string());

        assert_eq!(context.file_path, Some(PathBuf::from("test.rs")));
        assert_eq!(context.line, Some(42));
        assert_eq!(context.operation, Some("pattern_matching".to_string()));
    }

    #[test]
    fn test_contextual_error_display() {
        let error = ServiceError::config_dynamic("test error".to_string());
        let contextual = ContextualError {
            error,
            context: ErrorContext::new()
                .with_file_path(PathBuf::from("test.rs"))
                .with_line(42),
        };

        let display = format!("{}", contextual);
        assert!(display.contains("test error"));
        assert!(display.contains("test.rs"));
        assert!(display.contains("42"));
    }

    #[test]
    fn test_recovery_info() {
        let error = ServiceError::timeout("test timeout", std::time::Duration::from_secs(1));
        let recovery = error.recovery_info().unwrap();

        assert!(matches!(
            recovery.strategy,
            RecoveryStrategy::Retry { max_attempts: 2 }
        ));
        assert!(recovery.auto_recoverable);
    }
}
