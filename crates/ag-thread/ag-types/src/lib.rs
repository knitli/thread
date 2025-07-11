//! Comprehensive service abstractions for ag-thread multi-environment deployment.
//!
//! This crate provides the foundational service traits and types that enable
//! ast-grep functionality to operate across diverse environments including CLI,
//! Cloudflare Workers, CI/CD pipelines, and customer on-premise deployments.
mod ast;
mod fix;
mod label;
mod language;
mod matcher;
mod maybe;
mod meta_var;
mod ops;
mod replacer;
mod rule;
mod strictness;
mod transversal;
mod ts;

pub use ast::{AstGrep, Edit, SgNode, Doc, Content, Root, NodeData, PinnedNodeData, Position, Node, KindId};
pub use fix::{Transformation, DecomposedTransString, ParseTransError, Rewrite, Trans, Convert, Replace, Delimiter, CaseState, Separator, StringCase, Substring, Fixer, Expansion, FixerError, SerializableFixConfig, SerializableFixer, TransformError, Transform, Transformation, Ctx};
pub use label::{Label, LabelConfig, LabelStyle};
pub use language::Language;
pub use matcher::{KindMatcher, KindMatcherError, Matcher, MatchExt, MatchAll, MatchNone, NodeMatch, Pattern, PatternBuilder, PatternError, PatternNode, RegexMatcher, RegexMatcherError};
pub use maybe::Maybe;
pub use meta_var::{MetaVariable, MetaVarEnv, MetaVarExtract, Underlying, MetaVariableID};
pub use ops::{And, Any, All, Or, Not, Op, NestedAnd, NestedOr};
pub use replacer::{Replacer, DeindentedExtract, TemplateFix, TemplateFixError};
pub use rule::{ContingentRule, RuleBucket, RuleCollection, RuleConfig, Severity, RuleConfigError, SerializableRuleConfig, SerializableRewriter, Metadata, SerializableRule, AtomicRule, Strictness, PatternStyle, RelationalRule, CompositeRule, Rule, RuleSerializeError, RuleCoreError, SerializableRuleCore, RuleCore, SerializableStopBy, StopBy, Inside, Has, Precedes, Follows, Registration, RuleRegistration, RegistrationRef, ReferentRuleError, ReferentRule, GlobalRules, SerializablePosition, SerializableRange, RangeMatcherError, RangeMatcher, NthChildError, NthChildSimple, SerializableNthChild, NthChild, DeserializeEnv, CheckHint};
pub use strictness::{MatchStrictness, MatchOneNode};
pub use transversal::{Visitor, Visit, Traversal, TsPre, Pre, Algorithm, PreOrder, PostOrder, Post, Level};
pub use ts::{TSLanguage, TSTree, TSRange, TSInputEdit, TSNode, TSPoint, TSLanguageError, TSParser, TSTreeCursor, TSParseError, ContentExt, DisplayContext, LanguageExt, StrDoc, LanguageExt};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thread_utils::FastMap;
use std::fmt;
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

// =============================================================================
// Error Handling
// =============================================================================

/// Comprehensive error type for all ag-service operations.
#[derive(Debug, Error)]
pub enum AstGrepError {
    #[error("Source error: {message}")]
    Source {
        message: String,
        context: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Configuration error: {message}")]
    Config {
        message: String,
        config_type: ConfigType,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Language processing error: {message}")]
    Language {
        message: String,
        language: Option<String>,
        content_sample: Option<String>,
    },

    #[error("Pattern parsing error: {message}")]
    Pattern {
        message: String,
        pattern: String,
        language: String,
    },

    #[error("Fix application error: {message}")]
    Fix {
        message: String,
        fix_instruction: String,
        original_content: String,
    },

    #[error("Output adapter error: {message}")]
    Output {
        message: String,
        output_type: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Interaction error: {message}")]
    Interaction {
        message: String,
        interaction_type: String,
    },

    #[error("Async runtime error: {message}")]
    Runtime { message: String },

    #[error("Validation error: {field}: {message}")]
    Validation { field: String, message: String },

    #[error("Test execution error: {message}")]
    Test {
        message: String,
        test_id: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigType {
    Rules,
    ProjectSettings,
    GlobalRules,
    LanguageSettings,
}

impl AstGrepError {
    pub fn source_error(message: &str, context: &str) -> Self {
        Self::Source {
            message: message.to_string(),
            context: context.to_string(),
            source: None,
        }
    }

    pub fn config_error(message: &str, config_type: ConfigType) -> Self {
        Self::Config {
            message: message.to_string(),
            config_type,
            source: None,
        }
    }

    pub fn language_error(message: &str, language: Option<String>) -> Self {
        Self::Language {
            message: message.to_string(),
            language,
            content_sample: None,
        }
    }

    pub fn pattern_error(message: &str, pattern: &str, language: &str) -> Self {
        Self::Pattern {
            message: message.to_string(),
            pattern: pattern.to_string(),
            language: language.to_string(),
        }
    }

    pub fn validation_error(field: &str, message: &str) -> Self {
        Self::Validation {
            field: field.to_string(),
            message: message.to_string(),
        }
    }
}

pub type Result<T> = std::result::Result<T, AstGrepError>;

// =============================================================================
// Core Data Types
// =============================================================================

/// Represents a file item discovered by the file discovery service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileItem {
    pub path: String,
    pub language: Option<String>,
    pub size: Option<u64>,
    pub metadata: FastMap<String, serde_json::Value>,
}

/// Request for discovering files in the source system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiscoveryRequest {
    pub paths: Vec<String>,
    pub patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub follow_symlinks: bool,
    pub max_depth: Option<usize>,
    pub language_filter: Option<Vec<String>>,
}

impl Default for FileDiscoveryRequest {
    fn default() -> Self {
        Self {
            paths: vec![".".to_string()],
            patterns: vec!["**/*".to_string()],
            exclude_patterns: vec![],
            follow_symlinks: false,
            max_depth: None,
            language_filter: None,
        }
    }
}

/// Configuration source specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigSource {
    File(String),
    Inline(String),
    Remote(String),
    Database(String),
    Memory(Vec<u8>),
}

/// Output format specifications for different environments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Json {
        pretty: bool,
        include_metadata: bool,
    },
    Plain,
    Colored {
        style: ReportStyle,
    },
    GitHub,
    GitLab,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportStyle {
    Rich,
    Compact,
    Minimal,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Colored {
            style: ReportStyle::Rich,
        }
    }
}

/// Severity levels for diagnostic results.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
    Off,
}

// =============================================================================
// Operation Options
// =============================================================================

/// Comprehensive options for scanning files with rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanOptions {
    pub paths: Vec<String>,
    pub file_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub language_filter: Option<Vec<String>>,
    pub config_source: ConfigSource,
    pub output_format: OutputFormat,
    pub context_lines_before: usize,
    pub context_lines_after: usize,
    pub severity_filter: Option<Severity>,
    pub rule_filter: Option<String>,
    pub interactive: bool,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            paths: vec![".".to_string()],
            file_patterns: vec!["**/*".to_string()],
            exclude_patterns: vec![],
            language_filter: None,
            config_source: ConfigSource::File("sgconfig.yml".to_string()),
            output_format: OutputFormat::default(),
            context_lines_before: 0,
            context_lines_after: 0,
            severity_filter: None,
            rule_filter: None,
            interactive: false,
        }
    }
}

/// Options for applying fixes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixOptions {
    pub dry_run: bool,
    pub interactive: bool,
    pub backup: bool,
}

impl Default for FixOptions {
    fn default() -> Self {
        Self {
            dry_run: false,
            interactive: false,
            backup: true,
        }
    }
}

/// Options for test execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestOptions {
    pub parallel: bool,
    pub max_concurrency: Option<usize>,
    pub timeout: Option<Duration>,
    pub update_snapshots: bool,
}

impl Default for TestOptions {
    fn default() -> Self {
        Self {
            parallel: true,
            max_concurrency: None,
            timeout: Some(Duration::from_secs(30)),
            update_snapshots: false,
        }
    }
}

// =============================================================================
// Request/Response Types
// =============================================================================


/// Request for applying fixes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixRequest {
    pub fixes: Vec<FixInstruction>,
    pub discovery: FileDiscoveryRequest,
    pub options: FixOptions,
    pub output_format: OutputFormat,
}

/// Individual fix instruction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixInstruction {
    pub pattern: String,
    pub replacement: String,
    pub language: Option<String>,
}

// =============================================================================
// Result Types
// =============================================================================

/// Result of a scan operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResults {
    pub matches: Vec<ScanMatch>,
    pub execution_time: Option<Duration>,
    pub files_processed: usize,
}

/// Individual scan match result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanMatch {
    pub id: Uuid,
    pub file_path: String,
    pub rule_id: String,
    pub message: String,
    pub severity: Severity,
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub matched_text: String,
    pub context_before: Vec<String>,
    pub context_after: Vec<String>,
    pub metadata: FastMap<String, serde_json::Value>,
}

/// Result of a fix operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixResults {
    pub files: Vec<FixedFile>,
    pub execution_time: Option<Duration>,
}

/// Individual file fix result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedFile {
    pub path: String,
    pub result: FixResult,
}

/// Fix operation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FixResult {
    Applied {
        original_content: String,
        fixed_content: String,
        changes_applied: usize,
    },
    DryRun {
        diffs: Vec<Diff>,
    },
    Failed {
        error: String,
    },
}

/// Represents a diff/change in content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diff {
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub original_text: String,
    pub replacement_text: String,
}

// =============================================================================
// Test Types
// =============================================================================

/// Test case definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub id: String,
    pub name: String,
    pub file_path: String,
    pub rule_path: String,
    pub expected_matches: usize,
    pub metadata: FastMap<String, serde_json::Value>,
}

/// Test execution result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_case: TestCase,
    pub status: TestStatus,
    pub execution_time: Duration,
    pub actual_matches: usize,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Error,
}

/// Snapshot action for test management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SnapshotAction {
    Update { test_id: String, new_snapshot: String },
    Create { test_id: String, snapshot: String },
    Delete { test_id: String },
}

// =============================================================================
// Interaction Types
// =============================================================================

/// Result of user interaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionResult {
    Accepted,
    Rejected,
    Modified(String),
    Aborted,
}

/// Diagnostic information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub level: Severity,
    pub message: String,
    pub file_path: Option<String>,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub suggestion: Option<String>,
}

// =============================================================================
// Service Trait Definitions
// =============================================================================

/// Service for discovering and reading files.
#[async_trait]
pub trait FileDiscoveryService: Send + Sync {
    async fn discover_files(&self, request: FileDiscoveryRequest) -> Result<Vec<FileItem>>;
    async fn read_file_content(&self, path: &str) -> Result<String>;
    async fn get_file_language(&self, path: &str) -> Option<String>;
    async fn file_exists(&self, path: &str) -> bool;
}

/// Service for managing configuration.
#[async_trait]
pub trait ConfigurationService: Send + Sync {
    async fn load_rules(&self, source: ConfigSource) -> Result<String>;
    async fn load_project_config(&self, path: Option<String>) -> Result<String>;
    async fn validate_config(&self, config: &str) -> Result<()>;
}

/// Service for handling output.
#[async_trait]
pub trait OutputService: Send + Sync {
    async fn write_scan_results(&self, results: Vec<ScanMatch>, format: OutputFormat) -> Result<()>;
    async fn write_search_results(&self, results: Vec<PatternMatch>, format: OutputFormat) -> Result<()>;
    async fn write_fix_results(&self, results: Vec<FixResult>, format: OutputFormat) -> Result<()>;
    async fn write_diagnostics(&self, diagnostics: Vec<Diagnostic>) -> Result<()>;
}

/// Service for user interaction.
#[async_trait]
pub trait InteractionService: Send + Sync {
    async fn prompt_user(&self, message: &str, options: &[&str]) -> Result<String>;
    async fn confirm_action(&self, message: &str) -> Result<bool>;
    async fn display_diffs(&self, diffs: Vec<Diff>) -> Result<InteractionResult>;
    fn supports_interaction(&self) -> bool;
}

/// Service for terminal operations.
#[async_trait]
pub trait TerminalService: Send + Sync {
    async fn clear_screen(&self) -> Result<()>;
    async fn enter_alternate_screen(&self) -> Result<()>;
    async fn leave_alternate_screen(&self) -> Result<()>;
    fn supports_terminal(&self) -> bool;
}

/// Service for test execution and management.
#[async_trait]
pub trait TestExecutionService: Send + Sync {
    async fn discover_test_cases(&self, path: &str) -> Result<Vec<TestCase>>;
    async fn run_tests(&self, cases: Vec<TestCase>, options: TestOptions) -> Result<Vec<TestResult>>;
    async fn manage_snapshots(&self, actions: Vec<SnapshotAction>) -> Result<()>;
}

// =============================================================================
// Legacy Compatibility Traits
// =============================================================================

/// Legacy trait for backward compatibility.
#[async_trait]
pub trait SourceAdapter: Send + Sync {
    type Item: Send + Sync;
    type Error: std::error::Error + Send + Sync;

    async fn read_items(&self) -> std::result::Result<Vec<Self::Item>, Self::Error>;
}

/// Legacy trait for backward compatibility.
#[async_trait]
pub trait ContentProvider: Send + Sync {
    async fn get_content(
        &self,
        identifier: &str,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_language(&self, identifier: &str) -> Option<String>;
}

/// Legacy trait for backward compatibility.
#[async_trait]
pub trait SinkAdapter<T>: Send + Sync {
    type Error: std::error::Error + Send + Sync;

    async fn write_results(&self, results: Vec<T>) -> std::result::Result<(), Self::Error>;
}

/// Legacy trait for backward compatibility.
#[async_trait]
pub trait ConfigProvider: Send + Sync {
    async fn get_rules(
        &self,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_project_config(
        &self,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>>;
}

// =============================================================================
// Environment Types
// =============================================================================

/// Supported deployment environments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Environment {
    Cli,
    CiCd(CiProvider),
    Wasm,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CiProvider {
    GitHub,
    GitLab,
    Jenkins,
    Custom(String),
}

// =============================================================================
// Utility Implementations
// =============================================================================

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Environment::Cli => write!(f, "CLI"),
            Environment::CloudflareWorkers => write!(f, "Cloudflare Workers"),
            Environment::CiCd(provider) => write!(f, "CI/CD ({})", provider),
            Environment::Wasm => write!(f, "WASM"),
            Environment::Custom => write!(f, "Custom"),
        }
    }
}

impl fmt::Display for CiProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CiProvider::GitHub => write!(f, "GitHub Actions"),
            CiProvider::GitLab => write!(f, "GitLab CI"),
            CiProvider::Jenkins => write!(f, "Jenkins"),
            CiProvider::Custom(name) => write!(f, "{}", name),
        }
    }
}

// =============================================================================
// Feature Gated Tower Integration
// =============================================================================

#[cfg(feature = "tower")]
pub mod tower_support {
    use super::*;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    /// Tower service request types.
    #[derive(Debug, Clone)]
    pub enum AstGrepRequest {
        Scan(ScanOptions),
        Fix(FixRequest),
    }

    /// Tower service response types.
    #[derive(Debug, Clone)]
    pub enum AstGrepResponse {
        Scan(ScanResults),
        Fix(FixResults),
    }

    /// Future type for async service calls.
    pub type AstGrepFuture = Pin<Box<dyn Future<Output = Result<AstGrepResponse>> + Send>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = AstGrepError::source_error("test error", "test context");
        assert!(matches!(err, AstGrepError::Source { .. }));
    }

    #[test]
    fn test_default_options() {
        let scan_opts = ScanOptions::default();
        assert_eq!(scan_opts.paths, vec![".".to_string()]);

        let search_opts = SearchOptions::default();
        assert_eq!(search_opts.context_lines_before, 0);

        let fix_opts = FixOptions::default();
        assert!(!fix_opts.dry_run);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Error > Severity::Warning);
        assert!(Severity::Warning > Severity::Info);
        assert!(Severity::Info > Severity::Hint);
        assert!(Severity::Hint > Severity::Off);
    }

    #[test]
    fn test_environment_display() {
        let env = Environment::Cli;
        assert_eq!(env.to_string(), "CLI");

        let ci_env = Environment::CiCd(CiProvider::GitHub);
        assert_eq!(ci_env.to_string(), "CI/CD (GitHub Actions)");
    }
}
