
//! This implements the architecture proposed in the analysis, focusing on
//! environment differences rather than over-abstracting ast-grep's core functionality.

use crate::prelude::*;
use async_trait::async_trait;
use thread_utils::FastMap;

// =============================================================================
// Core Service Traits (3 instead of 6)
// =============================================================================

/// Service for environment-specific I/O operations.
/// This handles the differences between CLI, Cloudflare Workers, CI/CD, etc.
#[async_trait]
pub trait EnvironmentAdapter: Send + Sync {
    /// Read a file from the environment-specific storage.
    async fn read_file(&self, path: &str) -> Result<String>;

    /// Write output using environment-specific mechanisms.
    async fn write_output(&self, content: &str, format: OutputFormat) -> Result<()>;

    /// Discover files using environment-specific file system access.
    async fn discover_files(&self, patterns: &[String]) -> Result<Vec<String>>;

    /// Check if a file exists in the environment.
    async fn file_exists(&self, path: &str) -> bool;

    /// Get file metadata if available.
    async fn get_file_metadata(&self, path: &str) -> Result<FastMap<String, serde_json::Value>>;
}

/// Service for loading and validating rule configurations.
/// Much simpler than the previous ConfigurationService.
#[async_trait]
pub trait RuleProvider: Send + Sync {
    /// Load rules from a source (file, URL, database, etc.).
    async fn load_rules(&self, source: &str) -> Result<String>;

    /// Validate rule content format.
    async fn validate_rules(&self, content: &str) -> Result<()>;

    /// Get default rules if no source is specified.
    async fn get_default_rules(&self) -> Result<String>;
}

/// Service for formatting and outputting results.
/// Replaces the complex OutputService with focused formatting logic.
pub trait OutputFormatter: Send + Sync {
    /// Format scan results for the target environment.
    fn format_scan_results(&self, matches: &[EnrichedMatch], format: OutputFormat) -> Result<String>;

    /// Format search results for the target environment.
    fn format_search_results(&self, matches: &[EnrichedMatch], format: OutputFormat) -> Result<String>;

    /// Format fix results for the target environment.
    fn format_fix_results(&self, results: &[FixResult], format: OutputFormat) -> Result<String>;

    /// Format diagnostics and errors.
    fn format_diagnostics(&self, diagnostics: &[Diagnostic]) -> Result<String>;
}

// =============================================================================
// Simplified Service Registry
// =============================================================================

/// Simplified service registry with just the 3 essential services.
pub struct ServiceRegistry {
    pub environment: Box<dyn EnvironmentAdapter>,
    pub rule_provider: Box<dyn RuleProvider>,
    pub formatter: Box<dyn OutputFormatter>,
}

impl ServiceRegistry {
    /// Create a new registry with the provided services.
    pub fn new(
        environment: Box<dyn EnvironmentAdapter>,
        rule_provider: Box<dyn RuleProvider>,
        formatter: Box<dyn OutputFormatter>,
    ) -> Self {
        Self {
            environment,
            rule_provider,
            formatter,
        }
    }

    /// Create a registry for CLI environment with sensible defaults.
    pub fn for_cli() -> Self {
        Self {
            environment: Box::new(CliEnvironmentAdapter::new()),
            rule_provider: Box::new(FileRuleProvider::new(".")),
            formatter: Box::new(TerminalFormatter::new()),
        }
    }

    /// Create a registry for Cloudflare Workers environment.
    pub fn for_cloudflare_workers() -> Self {
        Self {
            environment: Box::new(CloudflareEnvironmentAdapter::new()),
            rule_provider: Box::new(CloudflareRuleProvider::new()),
            formatter: Box::new(JsonFormatter::new()),
        }
    }

    /// Create a registry for CI/CD environment.
    pub fn for_ci_cd(provider: CiProvider) -> Self {
        match provider {
            CiProvider::GitHub => Self {
                environment: Box::new(GitHubActionsAdapter::new()),
                rule_provider: Box::new(FileRuleProvider::new(".")),
                formatter: Box::new(GitHubFormatter::new()),
            },
            CiProvider::GitLab => Self {
                environment: Box::new(GitLabCiAdapter::new()),
                rule_provider: Box::new(FileRuleProvider::new(".")),
                formatter: Box::new(GitLabFormatter::new()),
            },
            CiProvider::Jenkins => Self {
                environment: Box::new(JenkinsAdapter::new()),
                rule_provider: Box::new(FileRuleProvider::new(".")),
                formatter: Box::new(JenkinsFormatter::new()),
            },
            CiProvider::Custom(name) => Self {
                environment: Box::new(GenericCiAdapter::new(&name)),
                rule_provider: Box::new(FileRuleProvider::new(".")),
                formatter: Box::new(PlainTextFormatter::new()),
            },
        }
    }
}

// =============================================================================
// CLI Environment Adapter Implementation
// =============================================================================

/// CLI environment adapter that uses the local filesystem and terminal.
pub struct CliEnvironmentAdapter {
    base_path: String,
}

impl CliEnvironmentAdapter {
    pub fn new() -> Self {
        Self {
            base_path: ".".to_string(),
        }
    }

    pub fn with_base_path(base_path: &str) -> Self {
        Self {
            base_path: base_path.to_string(),
        }
    }
}

#[async_trait]
impl EnvironmentAdapter for CliEnvironmentAdapter {
    async fn read_file(&self, path: &str) -> Result<String> {
        let full_path = if path.starts_with(&self.base_path) {
            path.to_string()
        } else {
            format!("{}/{}", self.base_path, path)
        };

        tokio::fs::read_to_string(full_path)
            .await
            .map_err(|e| AstGrepError::source_error(&e.to_string(), path))
    }

    async fn write_output(&self, content: &str, _format: OutputFormat) -> Result<()> {
        println!("{}", content);
        Ok(())
    }

    async fn discover_files(&self, patterns: &[String]) -> Result<Vec<String>> {
        use ignore::WalkBuilder;

        let mut files = Vec::new();
        let walker = WalkBuilder::new(&self.base_path)
            .hidden(false)
            .git_ignore(true)
            .build();

        for entry in walker {
            let entry = entry.map_err(|e| AstGrepError::source_error(&e.to_string(), "file_discovery"))?;

            if entry.file_type().map_or(false, |ft| ft.is_file()) {
                let path = entry.path().to_string_lossy().to_string();

                // Simple pattern matching - in a full implementation would use glob crate
                if patterns.is_empty() || patterns.iter().any(|pattern| {
                    if pattern == "**/*" {
                        true
                    } else if pattern.starts_with("*.") {
                        let ext = &pattern[2..];
                        path.ends_with(&format!(".{}", ext))
                    } else {
                        path.contains(pattern)
                    }
                }) {
                    files.push(path);
                }
            }
        }

        Ok(files)
    }

    async fn file_exists(&self, path: &str) -> bool {
        let full_path = if path.starts_with(&self.base_path) {
            path.to_string()
        } else {
            format!("{}/{}", self.base_path, path)
        };

        tokio::fs::metadata(full_path).await.is_ok()
    }

    async fn get_file_metadata(&self, path: &str) -> Result<FastMap<String, serde_json::Value>> {
        let full_path = if path.starts_with(&self.base_path) {
            path.to_string()
        } else {
            format!("{}/{}", self.base_path, path)
        };

        let metadata = tokio::fs::metadata(&full_path)
            .await
            .map_err(|e| AstGrepError::source_error(&e.to_string(), path))?;

        let mut meta = FastMap::new();
        meta.insert("size".to_string(), serde_json::Value::Number(metadata.len().into()));
        meta.insert("is_file".to_string(), serde_json::Value::Bool(metadata.is_file()));
        meta.insert("is_dir".to_string(), serde_json::Value::Bool(metadata.is_dir()));

        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                meta.insert("modified".to_string(), serde_json::Value::Number(duration.as_secs().into()));
            }
        }

        Ok(meta)
    }
}

// =============================================================================
// File-based Rule Provider Implementation
// =============================================================================

/// Rule provider that loads rules from local files.
pub struct FileRuleProvider {
    search_paths: Vec<String>,
}

impl FileRuleProvider {
    pub fn new(base_path: &str) -> Self {
        Self {
            search_paths: vec![
                format!("{}/sgconfig.yml", base_path),
                format!("{}/.sgconfig.yml", base_path),
                format!("{}/rules", base_path),
            ],
        }
    }

    pub fn with_search_paths(paths: Vec<String>) -> Self {
        Self {
            search_paths: paths,
        }
    }
}

#[async_trait]
impl RuleProvider for FileRuleProvider {
    async fn load_rules(&self, source: &str) -> Result<String> {
        // Try the explicit source first
        if !source.is_empty() && source != "sgconfig.yml" {
            if let Ok(content) = tokio::fs::read_to_string(source).await {
                return Ok(content);
            }
        }

        // Try search paths
        for path in &self.search_paths {
            if let Ok(content) = tokio::fs::read_to_string(path).await {
                return Ok(content);
            }
        }

        // Return default empty rules
        self.get_default_rules().await
    }

    async fn validate_rules(&self, content: &str) -> Result<()> {
        // Use the same parsing logic from ag_core_integration
        crate::ag_core_integration::parse_rule_patterns(content)?;
        Ok(())
    }

    async fn get_default_rules(&self) -> Result<String> {
        Ok(r#"
rules:
  - id: example-rule
    pattern: "console.log($msg)"
    message: "Found console.log statement"
    severity: "info"
    language: "javascript"
"#.to_string())
    }
}

// =============================================================================
// Terminal Output Formatter Implementation
// =============================================================================

/// Output formatter for terminal/CLI environments with color support.
pub struct TerminalFormatter {
    use_colors: bool,
}

impl TerminalFormatter {
    pub fn new() -> Self {
        Self {
            use_colors: true, // TODO: Detect terminal capabilities
        }
    }

    pub fn without_colors() -> Self {
        Self {
            use_colors: false,
        }
    }
}

impl OutputFormatter for TerminalFormatter {
    fn format_scan_results(&self, matches: &[EnrichedMatch], format: OutputFormat) -> Result<String> {
        use crate::ag_core_integration::EnrichedMatch;

        match format {
            OutputFormat::Json { pretty, include_metadata } => {
                if pretty {
                    serde_json::to_string_pretty(matches)
                } else {
                    serde_json::to_string(matches)
                }
                .map_err(|e| AstGrepError::output_error(&e.to_string(), "json"))
            }
            OutputFormat::Plain => {
                let mut output = String::new();
                for m in matches {
                    output.push_str(&format!("{}:{}:{}: {}\n",
                        m.file_path, m.start_line, m.start_column, m.message));
                }
                Ok(output)
            }
            OutputFormat::Colored { style: _ } => {
                let mut output = String::new();
                for m in matches {
                    if self.use_colors {
                        output.push_str(&format!(
                            "\x1b[36m{}:\x1b[33m{}:\x1b[32m{}\x1b[0m: {}\n",
                            m.file_path, m.start_line, m.start_column, m.message
                        ));
                    } else {
                        output.push_str(&format!("{}:{}:{}: {}\n",
                            m.file_path, m.start_line, m.start_column, m.message));
                    }
                }
                Ok(output)
            }
            OutputFormat::GitHub => {
                let mut output = String::new();
                for m in matches {
                    output.push_str(&format!(
                        "::warning file={},line={},col={}::{}\n",
                        m.file_path, m.start_line, m.start_column, m.message
                    ));
                }
                Ok(output)
            }
            OutputFormat::GitLab => {
                // GitLab Code Quality format
                serde_json::to_string_pretty(matches)
                    .map_err(|e| AstGrepError::output_error(&e.to_string(), "gitlab"))
            }
            OutputFormat::Custom(format_name) => {
                Err(AstGrepError::output_error(&format!("Unknown custom format: {}", format_name), "custom"))
            }
        }
    }

    fn format_search_results(&self, matches: &[EnrichedMatch], format: OutputFormat) -> Result<String> {
        // For search results, we can reuse the same formatting logic
        self.format_scan_results(matches, format)
    }

    fn format_fix_results(&self, results: &[FixResult], format: OutputFormat) -> Result<String> {
        match format {
            OutputFormat::Json { pretty, .. } => {
                if pretty {
                    serde_json::to_string_pretty(results)
                } else {
                    serde_json::to_string(results)
                }
                .map_err(|e| AstGrepError::output_error(&e.to_string(), "json"))
            }
            _ => {
                let mut output = String::new();
                for result in results {
                    match result {
                        FixResult::Applied { changes_applied, .. } => {
                            output.push_str(&format!("Applied {} changes\n", changes_applied));
                        }
                        FixResult::DryRun { diffs } => {
                            output.push_str(&format!("Would apply {} changes\n", diffs.len()));
                        }
                        FixResult::Failed { error } => {
                            output.push_str(&format!("Failed: {}\n", error));
                        }
                    }
                }
                Ok(output)
            }
        }
    }

    fn format_diagnostics(&self, diagnostics: &[Diagnostic]) -> Result<String> {
        let mut output = String::new();
        for diag in diagnostics {
            if self.use_colors {
                let color = match diag.level {
                    Severity::Error => "\x1b[31m", // Red
                    Severity::Warning => "\x1b[33m", // Yellow
                    Severity::Info => "\x1b[36m", // Cyan
                    Severity::Hint => "\x1b[90m", // Gray
                    Severity::Off => "",
                };
                output.push_str(&format!("{}[{}]\x1b[0m {}\n", color,
                    format!("{:?}", diag.level).to_uppercase(), diag.message));
            } else {
                output.push_str(&format!("[{}] {}\n",
                    format!("{:?}", diag.level).to_uppercase(), diag.message));
            }
        }
        Ok(output)
    }
}

// =============================================================================
// Placeholder Implementations for Other Environments
// =============================================================================

// Note: These are placeholder implementations. In a real system, these would be
// fully implemented with environment-specific logic.

pub struct CloudflareEnvironmentAdapter;
impl CloudflareEnvironmentAdapter {
    pub fn new() -> Self { Self }
}

pub struct CloudflareRuleProvider;
impl CloudflareRuleProvider {
    pub fn new() -> Self { Self }
}

pub struct JsonFormatter;
impl JsonFormatter {
    pub fn new() -> Self { Self }
}

pub struct GitHubActionsAdapter;
impl GitHubActionsAdapter {
    pub fn new() -> Self { Self }
}

pub struct GitHubFormatter;
impl GitHubFormatter {
    pub fn new() -> Self { Self }
}

pub struct GitLabCiAdapter;
impl GitLabCiAdapter {
    pub fn new() -> Self { Self }
}

pub struct GitLabFormatter;
impl GitLabFormatter {
    pub fn new() -> Self { Self }
}

pub struct JenkinsAdapter;
impl JenkinsAdapter {
    pub fn new() -> Self { Self }
}

pub struct JenkinsFormatter;
impl JenkinsFormatter {
    pub fn new() -> Self { Self }
}

pub struct GenericCiAdapter;
impl GenericCiAdapter {
    pub fn new(_name: &str) -> Self { Self }
}

pub struct PlainTextFormatter;
impl PlainTextFormatter {
    pub fn new() -> Self { Self }
}

// For now, all these will use the CLI implementations as placeholders.
// In a real implementation, each would have environment-specific logic.

#[async_trait]
impl EnvironmentAdapter for CloudflareEnvironmentAdapter {
    async fn read_file(&self, _path: &str) -> Result<String> {
        todo!("Implement Cloudflare Workers KV/R2 file reading")
    }
    async fn write_output(&self, _content: &str, _format: OutputFormat) -> Result<()> {
        todo!("Implement Cloudflare Workers response output")
    }
    async fn discover_files(&self, _patterns: &[String]) -> Result<Vec<String>> {
        todo!("Implement Cloudflare Workers file discovery")
    }
    async fn file_exists(&self, _path: &str) -> bool {
        todo!("Implement Cloudflare Workers file existence check")
    }
    async fn get_file_metadata(&self, _path: &str) -> Result<FastMap<String, serde_json::Value>> {
        todo!("Implement Cloudflare Workers metadata retrieval")
    }
}

// Similar placeholder implementations for other services...
// This demonstrates the structure while keeping the implementation focused.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplified_registry_creation() {
        let _registry = ServiceRegistry::for_cli();
        // Should not panic
    }

    #[tokio::test]
    async fn test_cli_environment_adapter() {
        let adapter = CliEnvironmentAdapter::new();
        let exists = adapter.file_exists("Cargo.toml").await;
        // This should be true in the project root
        assert!(exists);
    }

    #[tokio::test]
    async fn test_file_rule_provider() {
        let provider = FileRuleProvider::new(".");
        let rules = provider.get_default_rules().await.unwrap();
        assert!(rules.contains("rules:"));
        assert!(provider.validate_rules(&rules).await.is_ok());
    }

    #[test]
    fn test_terminal_formatter() {
        let formatter = TerminalFormatter::new();
        let matches = vec![];
        let result = formatter.format_scan_results(&matches, OutputFormat::Plain);
        assert!(result.is_ok());
    }
}
