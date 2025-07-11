//!  orchestrator that leverages
//! ag-core's capabilities directly while providing clean environment abstraction.

use crate::prelude::*;
use crate::ag_core_integration::{AgCoreService, detect_language, parse_rule_patterns, EnrichedMatch};
use crate::core_services::{EnvironmentAdapter, RuleProvider, OutputFormatter, ServiceRegistry};
use std::time::Instant;

/// Clean orchestrator that coordinates ag-core processing with environment-specific services.
pub struct AstGrepEngine {
    registry: ServiceRegistry,
    ag_core: AgCoreService,
}

impl AstGrepEngine {
    /// Create a new engine with the provided service registry.
    pub fn new(registry: ServiceRegistry) -> Self {
        Self {
            registry,
            ag_core: AgCoreService::new(),
        }
    }

    /// Create an engine for CLI environment with sensible defaults.
    pub fn for_cli() -> Self {
        Self::new(ServiceRegistry::for_cli())
    }

    /// Create an engine for Cloudflare Workers environment.
    pub fn for_cloudflare_workers() -> Self {
        Self::new(ServiceRegistry::for_cloudflare_workers())
    }

    /// Create an engine for CI/CD environment.
    pub fn for_ci_cd(provider: CiProvider) -> Self {
        Self::new(ServiceRegistry::for_ci_cd(provider))
    }

    /// Scan files with rules - much simpler than the previous implementation.
    pub async fn scan(&mut self, request: ScanRequest) -> Result<ScanResults> {
        let start_time = Instant::now();

        // 1. Load rules using the rule provider
        let rules_content = self.registry.rule_provider
            .load_rules(&request.rule_source).await?;

        // 2. Validate rules
        self.registry.rule_provider
            .validate_rules(&rules_content).await?;

        // 3. Parse rules into patterns using ag-core integration
        let rule_patterns = parse_rule_patterns(&rules_content)?;

        // 4. Discover files using environment adapter
        let file_paths = self.registry.environment
            .discover_files(&request.file_patterns).await?;

        if file_paths.is_empty() {
            return Ok(ScanResults {
                matches: Vec::new(),
                execution_time: Some(start_time.elapsed()),
                files_processed: 0,
            });
        }

        // 5. Process files using ag-core
        let mut all_matches = Vec::new();

        for file_path in &file_paths {
            // Skip if file doesn't exist
            if !self.registry.environment.file_exists(file_path).await {
                continue;
            }

            // Read file content
            let content = self.registry.environment.read_file(file_path).await?;

            // Detect language
            let language = match detect_language(file_path) {
                Ok(lang) => lang,
                Err(_) => continue, // Skip unsupported files
            };

            // Process each rule pattern
            for rule_pattern in &rule_patterns {
                // Check if rule applies to this file's language
                if rule_pattern.language != "any" &&
                   !file_path.ends_with(&format!(".{}", get_file_extension(&rule_pattern.language))) {
                    continue;
                }

                // Use ag-core to scan the file
                let scan_options = ScanOptions {
                    paths: vec![file_path.clone()],
                    file_patterns: request.file_patterns.clone(),
                    exclude_patterns: request.exclude_patterns.clone(),
                    language_filter: None,
                    config_source: ConfigSource::Inline(rules_content.clone()),
                    output_format: request.output_format.clone(),
                    context_lines_before: request.context_lines_before,
                    context_lines_after: request.context_lines_after,
                    severity_filter: request.severity_filter.clone(),
                    rule_filter: Some(rule_pattern.id.clone()),
                    interactive: false,
                };

                match self.ag_core.scan_file_with_pattern(
                    file_path,
                    &content,
                    &rule_pattern.pattern,
                    language.as_ref().clone(),
                    &scan_options,
                ).await {
                    Ok(enriched_matches) => {
                        // Convert to ScanMatch for compatibility
                        for em in enriched_matches {
                            all_matches.push(ScanMatch {
                                id: em.id,
                                file_path: em.file_path,
                                rule_id: rule_pattern.id.clone(),
                                message: rule_pattern.message.clone(),
                                severity: rule_pattern.severity.clone(),
                                start_line: em.start_line,
                                end_line: em.end_line,
                                start_column: em.start_column,
                                end_column: em.end_column,
                                matched_text: em.matched_text,
                                context_before: em.context_before,
                                context_after: em.context_after,
                                metadata: em.metadata,
                            });
                        }
                    }
                    Err(_) => continue, // Skip pattern matching errors
                }
            }
        }

        let results = ScanResults {
            matches: all_matches.clone(),
            execution_time: Some(start_time.elapsed()),
            files_processed: file_paths.len(),
        };

        // 6. Format and output results using the formatter
        let enriched_matches: Vec<EnrichedMatch> = all_matches
            .into_iter()
            .map(|sm| EnrichedMatch {
                id: sm.id,
                file_path: sm.file_path,
                rule_id: sm.rule_id,
                message: sm.message,
                severity: sm.severity,
                start_line: sm.start_line,
                end_line: sm.end_line,
                start_column: sm.start_column,
                end_column: sm.end_column,
                matched_text: sm.matched_text,
                context_before: sm.context_before,
                context_after: sm.context_after,
                metadata: sm.metadata,
            })
            .collect();

        let formatted_output = self.registry.formatter
            .format_scan_results(&enriched_matches, request.output_format.clone())?;

        // 7. Output using environment adapter
        self.registry.environment
            .write_output(&formatted_output, request.output_format).await?;

        Ok(results)
    }

    /// Search for patterns - much simpler implementation.
    pub async fn search(&mut self, request: SearchRequest) -> Result<SearchResults> {
        let start_time = Instant::now();

        // 1. Discover files
        let file_paths = self.registry.environment
            .discover_files(&request.discovery.patterns).await?;

        if file_paths.is_empty() {
            return Ok(SearchResults {
                matches: Vec::new(),
                execution_time: Some(start_time.elapsed()),
                files_processed: 0,
            });
        }

        // 2. Process files with the pattern
        let mut all_matches = Vec::new();

        for file_path in &file_paths {
            if !self.registry.environment.file_exists(file_path).await {
                continue;
            }

            let content = self.registry.environment.read_file(file_path).await?;
            let language = match detect_language(file_path) {
                Ok(lang) => lang,
                Err(_) => continue,
            };

            let scan_options = ScanOptions {
                paths: vec![file_path.clone()],
                file_patterns: vec![],
                exclude_patterns: vec![],
                language_filter: None,
                config_source: ConfigSource::Inline("".to_string()),
                output_format: request.output_format.clone(),
                context_lines_before: request.options.context_lines_before,
                context_lines_after: request.options.context_lines_after,
                severity_filter: None,
                rule_filter: None,
                interactive: false,
            };

            match self.ag_core.scan_file_with_pattern(
                file_path,
                &content,
                &request.pattern,
                language.as_ref().clone(),
                &scan_options,
            ).await {
                Ok(enriched_matches) => {
                    for em in enriched_matches {
                        all_matches.push(PatternMatch {
                            id: em.id,
                            file_path: em.file_path,
                            pattern: request.pattern.clone(),
                            start_line: em.start_line,
                            end_line: em.end_line,
                            start_column: em.start_column,
                            end_column: em.end_column,
                            matched_text: em.matched_text,
                            context_before: em.context_before,
                            context_after: em.context_after,
                            metadata: em.metadata,
                        });
                    }
                }
                Err(_) => continue,
            }
        }

        let results = SearchResults {
            matches: all_matches.clone(),
            execution_time: Some(start_time.elapsed()),
            files_processed: file_paths.len(),
        };

        // Format and output results
        let enriched_matches: Vec<EnrichedMatch> = all_matches
            .into_iter()
            .map(|pm| EnrichedMatch {
                id: pm.id,
                file_path: pm.file_path,
                rule_id: pm.pattern,
                message: "Pattern matched".to_string(),
                severity: Severity::Info,
                start_line: pm.start_line,
                end_line: pm.end_line,
                start_column: pm.start_column,
                end_column: pm.end_column,
                matched_text: pm.matched_text,
                context_before: pm.context_before,
                context_after: pm.context_after,
                metadata: pm.metadata,
            })
            .collect();

        let formatted_output = self.registry.formatter
            .format_search_results(&enriched_matches, request.output_format.clone())?;

        self.registry.environment
            .write_output(&formatted_output, request.output_format).await?;

        Ok(results)
    }

    /// Apply fixes using ag-core's replace functionality.
    pub async fn apply_fixes(&mut self, request: FixRequest) -> Result<FixResults> {
        let start_time = Instant::now();

        // 1. Discover files
        let file_paths = self.registry.environment
            .discover_files(&request.discovery.patterns).await?;

        if file_paths.is_empty() {
            return Ok(FixResults {
                files: Vec::new(),
                execution_time: Some(start_time.elapsed()),
            });
        }

        // 2. Apply fixes to each file
        let mut fixed_files = Vec::new();

        for file_path in &file_paths {
            if !self.registry.environment.file_exists(file_path).await {
                continue;
            }

            let content = self.registry.environment.read_file(file_path).await?;
            let language = match detect_language(file_path) {
                Ok(lang) => lang,
                Err(_) => continue,
            };

            let mut current_content = content.clone();
            let mut changes_applied = 0;
            let mut diffs = Vec::new();

            // Apply each fix instruction
            for fix_instruction in &request.fixes {
                if request.options.dry_run {
                    // For dry run, just record what would be changed
                    diffs.push(Diff {
                        file_path: file_path.clone(),
                        start_line: 1,
                        end_line: 1,
                        original_text: fix_instruction.pattern.clone(),
                        replacement_text: fix_instruction.replacement.clone(),
                    });
                } else {
                    // Apply the actual fix using ag-core
                    match self.ag_core.apply_fix(
                        &current_content,
                        &fix_instruction.pattern,
                        &fix_instruction.replacement,
                        language.as_ref().clone(),
                    ).await {
                        Ok(fixed_content) => {
                            if fixed_content != current_content {
                                current_content = fixed_content;
                                changes_applied += 1;
                            }
                        }
                        Err(_) => continue, // Skip failed fixes
                    }
                }
            }

            let result = if request.options.dry_run {
                FixResult::DryRun { diffs }
            } else {
                FixResult::Applied {
                    original_content: content,
                    fixed_content: current_content,
                    changes_applied,
                }
            };

            fixed_files.push(FixedFile {
                path: file_path.clone(),
                result,
            });
        }

        let results = FixResults {
            files: fixed_files.clone(),
            execution_time: Some(start_time.elapsed()),
        };

        // Format and output results
        let fix_results: Vec<FixResult> = fixed_files
            .into_iter()
            .map(|ff| ff.result)
            .collect();

        let formatted_output = self.registry.formatter
            .format_fix_results(&fix_results, request.output_format.clone())?;

        self.registry.environment
            .write_output(&formatted_output, request.output_format).await?;

        Ok(results)
    }
}

/// Helper function to map language names to file extensions.
fn get_file_extension(language: &str) -> &str {
    match language.to_lowercase().as_str() {
        "rust" => "rs",
        "javascript" => "js",
        "typescript" => "ts",
        "python" => "py",
        "java" => "java",
        "go" => "go",
        "cpp" | "c++" => "cpp",
        "c" => "c",
        "csharp" => "cs",
        "php" => "php",
        "ruby" => "rb",
        "swift" => "swift",
        "kotlin" => "kt",
        "scala" => "scala",
        "dart" => "dart",
        _ => "",
    }
}

/// Simplified request structures for the clean orchestrator.

#[derive(Debug, Clone)]
pub struct ScanRequest {
    pub rule_source: String,
    pub file_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub output_format: OutputFormat,
    pub context_lines_before: usize,
    pub context_lines_after: usize,
    pub severity_filter: Option<Severity>,
}

impl Default for ScanRequest {
    fn default() -> Self {
        Self {
            rule_source: "sgconfig.yml".to_string(),
            file_patterns: vec!["**/*".to_string()],
            exclude_patterns: vec![
                "**/target/**".to_string(),
                "**/node_modules/**".to_string(),
                "**/.git/**".to_string(),
            ],
            output_format: OutputFormat::default(),
            context_lines_before: 0,
            context_lines_after: 0,
            severity_filter: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core_services::ServiceRegistry;

    #[tokio::test]
    async fn test_clean_engine_creation() {
        let engine = AstGrepEngine::for_cli();
        // Should not panic
    }

    #[tokio::test]
    async fn test_scan_with_empty_files() {
        let mut engine = AstGrepEngine::for_cli();
        let request = ScanRequest {
            file_patterns: vec!["nonexistent/**/*.xyz".to_string()],
            ..Default::default()
        };

        let result = engine.scan(request).await;
        assert!(result.is_ok());

        let scan_results = result.unwrap();
        assert_eq!(scan_results.files_processed, 0);
        assert!(scan_results.matches.is_empty());
    }

    #[test]
    fn test_language_extension_mapping() {
        assert_eq!(get_file_extension("rust"), "rs");
        assert_eq!(get_file_extension("javascript"), "js");
        assert_eq!(get_file_extension("typescript"), "ts");
        assert_eq!(get_file_extension("unknown"), "");
    }
}
