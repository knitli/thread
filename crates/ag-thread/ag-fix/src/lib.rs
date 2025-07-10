//! Single-function apply_fixes crate for AST-grep.

use lib_ast_grep_types::{FixOptions, AstGrepError, Result};
use ast_grep_core::{AstGrep, Pattern};
use ast_grep_config::Fixer;
use ast_grep_language::SupportLang;

/// Represents a single fix applied to a file.
#[derive(Debug, Clone)]
pub struct FixResult {
    pub file: String,
    pub replacements: Vec<Replacement>,
}

/// Represents a single replacement in a file.
#[derive(Debug, Clone)]
pub struct Replacement {
    pub line: usize,
    pub column: usize,
    pub original: String,
    pub replacement: String,
}

/// Apply fixes to the provided files/contents using the given pattern and replacement.
/// This is a placeholder signature; integration with adapters and config will be added.
pub fn apply_fixes(
    files: Vec<(String, String, SupportLang)>,
    pattern: &str,
    replacement: &str,
    lang: SupportLang,
    options: &FixOptions,
) -> Result<Vec<FixResult>> {
    let mut results = Vec::new();
    let pat = Pattern::try_new(pattern, lang)
        .map_err(|e| AstGrepError::Pattern(format!("Pattern parse error: {e}")))?;
    let fixer = Fixer::from_str(replacement, &lang)
        .map_err(|e| AstGrepError::Fix(format!("Fixer parse error: {e}")))?;
    for (file, content, file_lang) in files {
        if file_lang != lang {
            continue;
        }
        let grep = lang.ast_grep(content.clone());
        let root = grep.root();
        let matches = root.find_all(&pat);
        let mut replacements = Vec::new();
        for m in matches {
            let (line, column) = m.range().start_line_col();
            let edit = m.make_edit(&pat, &fixer);
            let replacement_text = String::from_utf8(edit.inserted_text)
                .unwrap_or_else(|_| "<invalid utf8>".to_string());
            replacements.push(Replacement {
                line,
                column,
                original: m.text().to_string(),
                replacement: replacement_text,
            });
        }
        if !replacements.is_empty() {
            results.push(FixResult {
                file: file.clone(),
                replacements,
            });
        }
    }
    Ok(results)
}
