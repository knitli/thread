// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Optimized extension matching for language detection.
//!
//! This module provides high-performance file identification.
//! for efficient multi-pattern matching.
//!
//! The optimization strategies significantly improve performance over the naive
//! O(n*m) approach of checking each language's extensions individually.

use crate::{
    SupportLang,
    constants::{EXTENSION_TO_LANG, EXTENSIONS},
};
use aho_corasick::{AhoCorasick, AhoCorasickBuilder, Anchored, Input, MatchKind, StartKind};
use std::sync::LazyLock;

/// Aho-Corasick automaton for efficient multi-pattern matching.
/// Built lazily on first use with all extensions normalized to lowercase.
const AHO_CORASICK: LazyLock<AhoCorasick> = LazyLock::new(|| {
    // Use LeftmostLongest to prefer longer matches (e.g., "cpp" over "c")
    AhoCorasickBuilder::new()
        .match_kind(MatchKind::LeftmostLongest)
        .start_kind(StartKind::Anchored)
        .build(EXTENSIONS)
        .expect("Failed to build Aho-Corasick automaton")
});

/// Aho-Corasick based extension matching for comprehensive pattern matching.
///
/// This function uses a pre-built automaton to efficiently match against
/// all possible extensions simultaneously.
///
/// # Arguments
/// * `ext` - The file extension to match (case-insensitive)
///
/// # Returns
/// * `Some(SupportLang)` if a matching language is found
/// * `None` if no language matches the extension
#[inline(always)]
pub fn match_by_aho_corasick(ext: &str) -> Option<SupportLang> {
    if ext.is_empty() {
        return None;
    }
    let ext_lower = ext.to_ascii_lowercase();
    // Find matches and ensure they span the entire extension
    for mat in AHO_CORASICK.find_iter(Input::new(&ext_lower).anchored(Anchored::Yes)) {
        // Only accept matches that span the entire extension
        if mat.end() == ext_lower.len() {
            let pattern_id = mat.pattern().as_usize();
            return Some(EXTENSION_TO_LANG[pattern_id]);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aho_corasick_matching() {
        // Test basic matching
        assert_eq!(match_by_aho_corasick("rs"), Some(SupportLang::Rust));
        assert_eq!(match_by_aho_corasick("py"), Some(SupportLang::Python));
        assert_eq!(match_by_aho_corasick("js"), Some(SupportLang::JavaScript));

        // Test case insensitivity
        assert_eq!(match_by_aho_corasick("RS"), Some(SupportLang::Rust));
        assert_eq!(match_by_aho_corasick("PY"), Some(SupportLang::Python));

        // Test complex extensions
        assert_eq!(match_by_aho_corasick("tsx"), Some(SupportLang::Tsx));
        assert_eq!(match_by_aho_corasick("cpp"), Some(SupportLang::Cpp));

        // Test ambiguous extensions (C vs C++)
        // "c" extension should match C (first in enum order)
        assert_eq!(match_by_aho_corasick("c"), Some(SupportLang::C));

        // Test non-existent extensions
        assert_eq!(match_by_aho_corasick("xyz"), None);
        assert_eq!(match_by_aho_corasick(""), None);
    }
}
