//! Single-function scan_files crate for AST-grep.

use lib_ast_grep_types::{ScanOptions, AstGrepError, Result};
use ast_grep_core::AstGrep;
use ast_grep_config::{RuleCollection, RuleConfig};
use ast_grep_language::SupportLang;

/// Represents a single scan match.
#[derive(Debug, Clone)]
pub struct ScanMatch {
    pub file: String,
    pub rule_id: String,
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub matched_text: String,
}

/// The result of a scan operation.
#[derive(Debug, Clone)]
pub struct ScanResults {
    pub matches: Vec<ScanMatch>,
}

/// Scan the provided files/contents using the given rules and options.
/// This is a placeholder signature; integration with adapters and config will be added.
pub fn scan_files(
    files: Vec<(String, String, SupportLang)>,
    rules: &RuleCollection<SupportLang>,
    options: &ScanOptions,
) -> Result<ScanResults> {
    let mut results = Vec::new();
    for (file, content, lang) in files {
        let grep = lang.ast_grep(content);
        for rule in rules.iter() {
            let root = grep.root();
            let matches = root.find_all(&rule.matcher);
            for m in matches {
                let (line, column) = m.range().start_line_col();
                results.push(ScanMatch {
                    file: file.clone(),
                    rule_id: rule.id.clone(),
                    message: rule.message.clone().unwrap_or_default(),
                    line,
                    column,
                    matched_text: m.text().to_string(),
                });
            }
        }
    }
    Ok(ScanResults { matches: results })
}
