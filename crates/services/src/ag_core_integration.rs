//! Direct ag-core integration module that replaces the problematic ScanEngine implementation.
//!
//! This module provides a clean interface to ag-core's powerful AST processing capabilities
//! without reinventing the wheel.

use crate::prelude::*;
use ag_service_ast::{Node, Root, Doc};
use ag_service_pattern::{Pattern, Matcher, MatcherExt};
use ag_service_tree_sitter::{Language, AstGrep};
use thread_languages::SupportedLanguage
use thread_utils::FastMap;
use std::path::Path;



/// Detect language from file path using ag-core's Language::from_path method.
pub fn detect_language(file_path: &str) -> Result<Box<dyn Language + Send + Sync>> {
    let path = Path::new(file_path);

    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        match ext {
            "ts" | "tsx" => Ok(Box::new(TypeScript)),
            "rs" => Ok(Box::new(Rust)),
            "js" | "jsx" | "mjs" => Ok(Box::new(JavaScript)),
            "py" => Ok(Box::new(Python)),
            _ => Err(AstGrepError::language_error(
                &format!("Unsupported file extension: {}", ext),
                Some(ext.to_string())
            )),
        }
    } else {
        Err(AstGrepError::language_error(
            "Could not determine file language - no extension found",
            None
        ))
    }
}

/// ag-core based scanning service that replaces the problematic ScanEngine.
pub struct AgCoreService {
    /// Cache of compiled patterns for performance
    pattern_cache: FastMap<String, Pattern>,
}

impl AgCoreService {
    pub fn new() -> Self {
        Self {
            pattern_cache: FastMap::new(),
        }
    }

    /// Scan a file using ag-core's powerful pattern matching.
    /// This replaces the entire ScanEngine::scan_file_with_rules method.
    pub async fn scan_file_with_pattern<L: Language + Clone>(
        &mut self,
        file_path: &str,
        file_content: &str,
        pattern_str: &str,
        language: L,
        options: &ScanOptions,
    ) -> Result<Vec<EnrichedMatch>> {
        // Use ag-core's Pattern directly - much more robust than custom parsing
        let pattern = if let Some(cached_pattern) = self.pattern_cache.get(pattern_str) {
            cached_pattern.clone()
        } else {
            let new_pattern = Pattern::new(pattern_str, language.clone());
            self.pattern_cache.insert(pattern_str.to_string(), new_pattern.clone());
            new_pattern
        };

        // Create ag-core Root document - this handles all the complex AST parsing
        let root = Root::str(file_content, language);

        // Use ag-core's find_all method - much more sophisticated than our custom matching
        let matches: Vec<_> = root.root().find_all(&pattern).collect();

        // Convert ag-core's NodeMatch to our enriched match format
        let enriched_matches = matches
            .into_iter()
            .map(|node_match| {
                let range = node_match.range();
                let start_pos = node_match.get_node().start_pos();
                let end_pos = node_match.get_node().end_pos();

                // Extract context lines
                let (context_before, context_after) = self.extract_context(
                    file_content,
                    start_pos.line(),
                    end_pos.line(),
                    options.context_lines_before,
                    options.context_lines_after,
                );

                EnrichedMatch {
                    id: uuid::Uuid::new_v4(),
                    file_path: file_path.to_string(),
                    rule_id: pattern_str.to_string(), // Use pattern as rule ID for now
                    message: format!("Pattern '{}' matched", pattern_str),
                    severity: Severity::Info,
                    start_line: start_pos.line() + 1, // Convert to 1-based
                    end_line: end_pos.line() + 1,
                    start_column: range.start,
                    end_column: range.end,
                    matched_text: node_match.text().to_string(),
                    context_before,
                    context_after,
                    metadata: FastMap::new(),
                }
            })
            .collect();

        Ok(enriched_matches)
    }

    /// Apply fixes using ag-core's replace functionality.
    pub async fn apply_fix<L: Language + Clone>(
        &mut self,
        file_content: &str,
        pattern_str: &str,
        replacement: &str,
        language: L,
    ) -> Result<String> {
        // Create mutable root for in-place editing
        let mut root = Root::str(file_content, language);

        // Use ag-core's replace method - handles all the complex AST manipulation
        let _replaced = root.replace(pattern_str, replacement)
            .map_err(|e| AstGrepError::pattern_error(&e, pattern_str, "unknown"))?;

        // Generate the modified source
        Ok(root.generate())
    }

    /// Extract context lines around a match (helper method).
    fn extract_context(
        &self,
        file_content: &str,
        start_line: usize,
        end_line: usize,
        context_before: usize,
        context_after: usize,
    ) -> (Vec<String>, Vec<String>) {
        let lines: Vec<&str> = file_content.lines().collect();

        let context_start = start_line.saturating_sub(context_before);
        let context_end = std::cmp::min(end_line + context_after + 1, lines.len());

        let before: Vec<String> = lines[context_start..start_line]
            .iter()
            .map(|s| s.to_string())
            .collect();

        let after: Vec<String> = lines[end_line + 1..context_end]
            .iter()
            .map(|s| s.to_string())
            .collect();

        (before, after)
    }
}

impl Default for AgCoreService {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced match result that includes ag-core's NodeMatch capabilities
/// plus additional metadata needed for service layer.
#[derive(Debug, Clone)]
pub struct EnrichedMatch {
    pub id: uuid::Uuid,
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

/// Parse YAML rule configuration into individual patterns.
/// Much simpler than the previous implementation since we delegate to ag-core.
pub fn parse_rule_patterns(rules_content: &str) -> Result<Vec<RulePattern>> {
    #[cfg_attr(feature = "serde", derive(serde::Deserialize))]
    struct RuleFile {
        rules: Option<Vec<SingleRule>>,
        #[cfg_attr(feature = "serde", serde(flatten))]
        single_rule: Option<SingleRule>,
    }

    #[cfg_attr(feature = "serde", derive(serde::Deserialize))]
    struct SingleRule {
        id: String,
        pattern: String,
        message: Option<String>,
        severity: Option<String>,
        language: String,
    }
    cfg_if::cfg_if! {
        if #[cfg(feature = "serde")] {
        let rule_file: RuleFile = serde_yaml::from_str(rules_content)
        .map_err(|e| AstGrepError::config_error(&e.to_string(), ConfigType::Rules))?;
        } else {
        // TODO: Implement an alternative parsing method if serde is not available
        panic!("Serde feature is required for parsing rules");
        }
    }
    let rule_file: RuleFile = serde_yaml::from_str(rules_content)
        .map_err(|e| AstGrepError::config_error(&e.to_string(), ConfigType::Rules))?;

    let rules = if let Some(rules) = rule_file.rules {
        rules
    } else if let Some(single_rule) = rule_file.single_rule {
        vec![single_rule]
    } else {
        return Err(AstGrepError::validation_error("rules", "No rules found in configuration"));
    };

    let patterns = rules
        .into_iter()
        .map(|rule| {
            let severity = rule.severity
                .as_ref()
                .and_then(|s| match s.to_lowercase().as_str() {
                    "error" => Some(Severity::Error),
                    "warning" => Some(Severity::Warning),
                    "info" => Some(Severity::Info),
                    "hint" => Some(Severity::Hint),
                    _ => None,
                })
                .unwrap_or(Severity::Info);

            RulePattern {
                id: rule.id,
                pattern: rule.pattern,
                message: rule.message.unwrap_or_else(|| "Pattern matched".to_string()),
                severity,
                language: rule.language,
            }
        })
        .collect();

    Ok(patterns)
}

/// Simplified rule pattern structure.
#[derive(Debug, Clone)]
pub struct RulePattern {
    pub id: String,
    pub pattern: String,
    pub message: String,
    pub severity: Severity,
    pub language: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        assert!(detect_language("test.ts").is_ok());
        assert!(detect_language("test.rs").is_ok());
        assert!(detect_language("test.js").is_ok());
        assert!(detect_language("test.py").is_ok());
        assert!(detect_language("test.unknown").is_err());
    }

    #[test]
    fn test_rule_parsing() {
        let rules = r#"
rules:
  - id: test-rule
    pattern: "console.log($msg)"
    message: "Found console.log"
    severity: "warning"
    language: "javascript"
"#;

        let patterns = parse_rule_patterns(rules).unwrap();
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].id, "test-rule");
        assert_eq!(patterns[0].pattern, "console.log($msg)");
    }

    #[tokio::test]
    async fn test_agcore_service_creation() {
        let service = AgCoreService::new();
        assert!(service.pattern_cache.is_empty());
    }
}
