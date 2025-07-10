//! Single-function search_pattern crate for AST-grep.

use lib_ast_grep_types::{SearchOptions, AstGrepError, Result};
use ast_grep_core::{AstGrep, Pattern};
use ast_grep_language::SupportLang;

/// Represents a single pattern match.
#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub matched_text: String,
}

/// The result of a pattern search operation.
#[derive(Debug, Clone)]
pub struct SearchResults {
    pub matches: Vec<PatternMatch>,
}

/// Search the provided files/contents for a pattern.
/// This is a placeholder signature; integration with adapters and config will be added.
pub fn search_pattern(
    files: Vec<(String, String, SupportLang)>,
    pattern: &str,
    lang: SupportLang,
    options: &SearchOptions,
) -> Result<SearchResults> {
    let mut results = Vec::new();
    let pat = Pattern::try_new(pattern, lang)
        .map_err(|e| AstGrepError::Pattern(format!("Pattern parse error: {e}")))?;
    for (file, content, file_lang) in files {
        if file_lang != lang {
            continue;
        }
        let grep = lang.ast_grep(content);
        let root = grep.root();
        let matches = root.find_all(&pat);
        for m in matches {
            let (line, column) = m.range().start_line_col();
            results.push(PatternMatch {
                file: file.clone(),
                line,
                column,
                matched_text: m.text().to_string(),
            });
        }
    }
    Ok(SearchResults { matches: results })
}
