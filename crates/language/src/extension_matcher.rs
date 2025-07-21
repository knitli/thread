// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

//! Optimized extension matching for language detection.
//!
//! This module provides high-performance extension matching using a combination of:
//! 1. Character-based bucketing for fast first-level filtering
//! 2. Aho-Corasick automaton for efficient multi-pattern matching
//! 3. Case-insensitive matching optimizations
//!
//! The optimization strategies significantly improve performance over the naive
//! O(n*m) approach of checking each language's extensions individually.

use crate::SupportLang;
use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
use std::collections::HashMap;
use std::sync::LazyLock;

/// Character-based buckets mapping first characters to possible languages.
/// This provides O(1) first-level filtering to reduce the search space.
static CHAR_BUCKETS: LazyLock<HashMap<char, Vec<SupportLang>>> = LazyLock::new(|| {
    let mut buckets: HashMap<char, Vec<SupportLang>> = HashMap::new();
    
    // Build character buckets from all language extensions
    for &lang in SupportLang::all_langs() {
        for &ext in crate::extensions(lang) {
            // Handle extensions that start with '.' (like .bashrc)
            let first_char = if ext.starts_with('.') && ext.len() > 1 {
                ext.chars().nth(1).unwrap().to_ascii_lowercase()
            } else {
                ext.chars().next().unwrap().to_ascii_lowercase()
            };
            
            buckets.entry(first_char).or_default().push(lang);
        }
    }
    
    // Remove duplicates and sort for consistent ordering
    for langs in buckets.values_mut() {
        langs.sort_by_key(|lang| format!("{:?}", lang));
        langs.dedup();
    }
    
    buckets
});

/// Aho-Corasick automaton for efficient multi-pattern matching.
/// Built lazily on first use with all extensions normalized to lowercase.
static AHO_CORASICK: LazyLock<(AhoCorasick, Vec<SupportLang>)> = LazyLock::new(|| {
    let mut patterns = Vec::new();
    let mut pattern_to_lang = Vec::new();
    
    // Collect all extensions with their corresponding languages
    for &lang in SupportLang::all_langs() {
        for &ext in crate::extensions(lang) {
            patterns.push(ext.to_ascii_lowercase());
            pattern_to_lang.push(lang);
        }
    }
    
    // Build the automaton with case-insensitive matching
    // Use LeftmostLongest to prefer longer matches (e.g., "cpp" over "c")
    let ac = AhoCorasickBuilder::new()
        .match_kind(MatchKind::LeftmostLongest)
        .build(&patterns)
        .expect("Failed to build Aho-Corasick automaton");
    
    (ac, pattern_to_lang)
});

/// Fast extension matching using character bucketing as first-level filter.
///
/// This function provides O(1) lookup for the first character, then only
/// checks extensions for languages that could potentially match.
///
/// # Arguments
/// * `ext` - The file extension to match (case-insensitive)
///
/// # Returns
/// * `Some(SupportLang)` if a matching language is found
/// * `None` if no language matches the extension
pub fn match_by_char_bucket(ext: &str) -> Option<SupportLang> {
    if ext.is_empty() {
        return None;
    }
    
    // Get the first character for bucketing
    let first_char = if ext.starts_with('.') && ext.len() > 1 {
        ext.chars().nth(1).unwrap().to_ascii_lowercase()
    } else {
        ext.chars().next().unwrap().to_ascii_lowercase()
    };
    
    // Get candidate languages for this first character
    let candidates = CHAR_BUCKETS.get(&first_char)?;
    
    // Normalize extension for comparison
    let ext_lower = ext.to_ascii_lowercase();
    
    // Check only the candidate languages
    for &lang in candidates {
        for &lang_ext in crate::extensions(lang) {
            if lang_ext.eq_ignore_ascii_case(&ext_lower) {
                return Some(lang);
            }
        }
    }
    
    None
}

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
pub fn match_by_aho_corasick(ext: &str) -> Option<SupportLang> {
    if ext.is_empty() {
        return None;
    }
    
    let (ref ac, ref pattern_to_lang) = *AHO_CORASICK;
    let ext_lower = ext.to_ascii_lowercase();
    
    // Find matches and ensure they span the entire extension
    for mat in ac.find_iter(&ext_lower) {
        // Only accept matches that span the entire extension
        if mat.start() == 0 && mat.end() == ext_lower.len() {
            let pattern_id = mat.pattern().as_usize();
            return Some(pattern_to_lang[pattern_id]);
        }
    }
    
    None
}

/// Hybrid extension matching combining character bucketing and aho-corasick.
///
/// This function first tries character-based bucketing for optimal performance
/// on common cases, then falls back to aho-corasick for comprehensive matching.
///
/// # Arguments
/// * `ext` - The file extension to match (case-insensitive)
///
/// # Returns
/// * `Some(SupportLang)` if a matching language is found
/// * `None` if no language matches the extension
pub fn match_extension_optimized(ext: &str) -> Option<SupportLang> {
    // Try character bucketing first (fastest for most cases)
    if let Some(lang) = match_by_char_bucket(ext) {
        return Some(lang);
    }
    
    // Fallback to aho-corasick for comprehensive matching
    match_by_aho_corasick(ext)
}

/// Get statistics about the optimization structures for debugging/profiling.
pub fn get_optimization_stats() -> OptimizationStats {
    let char_buckets = &*CHAR_BUCKETS;
    let (ref ac, ref patterns) = *AHO_CORASICK;
    
    let total_buckets = char_buckets.len();
    let total_bucket_entries: usize = char_buckets.values().map(|v| v.len()).sum();
    let avg_bucket_size = if total_buckets > 0 {
        total_bucket_entries as f64 / total_buckets as f64
    } else {
        0.0
    };
    
    let single_lang_buckets = char_buckets.values().filter(|v| v.len() == 1).count();
    let multi_lang_buckets = char_buckets.values().filter(|v| v.len() > 1).count();
    
    OptimizationStats {
        total_extensions: patterns.len(),
        total_char_buckets: total_buckets,
        single_language_buckets: single_lang_buckets,
        multi_language_buckets: multi_lang_buckets,
        average_bucket_size: avg_bucket_size,
        aho_corasick_patterns: ac.patterns_len(),
    }
}

/// Statistics about the optimization structures.
#[derive(Debug, Clone)]
pub struct OptimizationStats {
    pub total_extensions: usize,
    pub total_char_buckets: usize,
    pub single_language_buckets: usize,
    pub multi_language_buckets: usize,
    pub average_bucket_size: f64,
    pub aho_corasick_patterns: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_char_bucket_matching() {
        // Test single-language buckets (perfect disambiguation)
        assert_eq!(match_by_char_bucket("rs"), Some(SupportLang::Rust));
        assert_eq!(match_by_char_bucket("go"), Some(SupportLang::Go));
        assert_eq!(match_by_char_bucket("lua"), Some(SupportLang::Lua));
        
        // Test case insensitivity
        assert_eq!(match_by_char_bucket("RS"), Some(SupportLang::Rust));
        assert_eq!(match_by_char_bucket("Go"), Some(SupportLang::Go));
        
        // Test multi-language buckets (should still work)
        assert_eq!(match_by_char_bucket("js"), Some(SupportLang::JavaScript));
        assert_eq!(match_by_char_bucket("java"), Some(SupportLang::Java));
        
        // Test non-existent extensions
        assert_eq!(match_by_char_bucket("xyz"), None);
        assert_eq!(match_by_char_bucket(""), None);
    }
    
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
    
    #[test]
    fn test_hybrid_matching() {
        // Test that hybrid matching works for all known extensions
        let test_cases = [
            ("rs", SupportLang::Rust),
            ("py", SupportLang::Python),
            ("js", SupportLang::JavaScript),
            ("tsx", SupportLang::Tsx),
            ("cpp", SupportLang::Cpp),
            ("go", SupportLang::Go),
            ("java", SupportLang::Java),
            ("json", SupportLang::Json),
        ];
        
        for (ext, expected_lang) in test_cases {
            assert_eq!(match_extension_optimized(ext), Some(expected_lang));
            // Test case insensitivity
            assert_eq!(match_extension_optimized(&ext.to_uppercase()), Some(expected_lang));
        }
        
        // Test non-existent extensions
        assert_eq!(match_extension_optimized("xyz"), None);
        assert_eq!(match_extension_optimized(""), None);
    }
    
    #[test]
    fn test_optimization_stats() {
        let stats = get_optimization_stats();
        
        // Verify basic statistics make sense
        assert!(stats.total_extensions > 0);
        assert!(stats.total_char_buckets > 0);
        assert!(stats.aho_corasick_patterns > 0);
        assert_eq!(stats.total_extensions, stats.aho_corasick_patterns);
        
        // Verify bucket distribution
        assert!(stats.single_language_buckets > 0);
        assert!(stats.multi_language_buckets > 0);
        assert_eq!(
            stats.single_language_buckets + stats.multi_language_buckets,
            stats.total_char_buckets
        );
        
        println!("Optimization stats: {:#?}", stats);
    }
}
