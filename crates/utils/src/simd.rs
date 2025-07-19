// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later
//! SIMD optimized utilities for string processing.
//!
//! This module provides a series of SIMD optimized functions for string processing.
//! Its operations use the `simdeez` crate, along with `memchr` for strong SIMD support.
//! Both libraries provide SIMD support for wasm32, `x86_64/x86`, and aarch64 and can find
//! optimal instruction sets at runtime.
//! If no SIMD support is available, they will fall back to scalar operations.

use memchr::memmem::FinderRev;
use simdeez::{prelude::*, simd_runtime_generate};
use std::sync::OnceLock;

static REV_LINE_FINDER: OnceLock<FinderRev> = OnceLock::new();

// Checks if a string is all ascii.
simd_runtime_generate!(
    pub fn is_ascii_simd(text: &str) -> bool {
        let bytes = text.as_bytes();
        let len = bytes.len();

        // reinterpret u8 as i8 slice (safe because underlying bits match)
        let bytes_i8 = unsafe { std::slice::from_raw_parts(bytes.as_ptr().cast::<i8>(), len) };

        let mut remainder = bytes_i8;

        // Process in vector-width chunks
        while remainder.len() >= S::Vi8::WIDTH {
            let chunk = &remainder[..S::Vi8::WIDTH];
            let v = S::Vi8::load_from_slice(chunk);

            // For ASCII, all values must be >= 0 (since ASCII is 0..127)
            let mask = v.cmp_lt(S::Vi8::set1(0));
            // Check if any lane is negative (non-ASCII)
            // get_mask() returns a bitmask, if any bit is set, it means non-ASCII was found
            if mask.get_mask() != 0 {
                return false;
            }

            remainder = &remainder[S::Vi8::WIDTH..];
        }

        // Handle remaining bytes
        remainder.iter().all(|&b| b >= 0)
    }
);

// Find the last occurrence of a byte value in a slice, searching backwards
// Returns the index of the last occurrence, or None if not found
simd_runtime_generate!(
    fn find_last_byte_simd(haystack: &[u8], needle: u8, is_eol: bool) -> Option<usize> {
        if haystack.is_empty() {
            return None;
        }
        if is_eol {
            // Special case for newline, use cached finder
            // Use into_owned() to ensure the FinderRev outlives the reference to its needle (it doesn't need after it's constructed)
            let line_finder =
                REV_LINE_FINDER.get_or_init(|| FinderRev::new(&[needle]).into_owned());
            return line_finder.rfind(haystack);
        }
        let bound_needle = &[needle];
        let finder = FinderRev::new(bound_needle);

        finder.rfind(haystack)
    }
);

// Count UTF-8 characters in a byte slice using SIMD
// This counts by identifying non-continuation bytes
simd_runtime_generate!(
    fn count_utf8_chars_simd(bytes: &[u8]) -> usize {
        let len = bytes.len();
        if len == 0 {
            return 0;
        }

        // Convert to i8 for SIMD operations
        let bytes_i8 = unsafe { std::slice::from_raw_parts(bytes.as_ptr().cast::<i8>(), len) };

        let mut remainder = bytes_i8;
        let mut char_count = 0;

        // UTF-8 continuation bytes have pattern 10xxxxxx (0x80-0xBF)
        // We want to count bytes that are NOT continuation bytes
        let continuation_pattern = S::Vi8::set1(0b1000_0000_u8 as i8);
        let mask_pattern = S::Vi8::set1(0b1100_0000_u8 as i8);

        // Process in SIMD chunks
        while remainder.len() >= S::Vi8::WIDTH {
            let chunk = &remainder[..S::Vi8::WIDTH];
            let v = S::Vi8::load_from_slice(chunk);

            // Check which bytes are NOT continuation bytes
            // Continuation bytes: (byte & 0b11000000) == 0b10000000
            let masked = v & mask_pattern;
            let is_continuation = masked.cmp_eq(continuation_pattern);

            // Count non-continuation bytes
            let mask = is_continuation.get_mask();
            // Count zeros in the mask (non-continuation bytes)
            char_count += S::Vi8::WIDTH - mask.count_ones() as usize;

            remainder = &remainder[S::Vi8::WIDTH..];
        }

        // Handle remaining bytes
        for &byte in remainder {
            if (byte as u8) & 0b1100_0000 != 0b1000_0000 {
                char_count += 1;
            }
        }

        char_count
    }
);

/// Optimized character column calculation with SIMD, finding the last newline character's index
///
/// It first checks if the line is entirely `ascii` with [`is_ascii_simd`],
/// and if so, uses a faster search strategy with [`find_last_byte_simd`].
/// If there are utf-8 characters present, it still uses the same approach but then
/// must use [`count_utf8_chars_simd`] to count non-continuation bytes.
/// All operations are highly optimized with full SIMD support.
#[inline]
#[must_use] pub fn get_char_column_simd(text: &str, offset: usize) -> usize {
    if offset == 0 {
        return 0;
    }

    let bytes = text.as_bytes();
    if offset > bytes.len() {
        return 0;
    }

    let search_slice = &bytes[..offset];

    // Check if the text is ASCII for fast path
    if is_ascii_simd(text) {
        // ASCII fast path: find last newline and count bytes
        match find_last_byte_simd(search_slice, b'\n', true) {
            Some(newline_pos) => offset - newline_pos - 1,
            None => offset, // No newline found, entire offset is the column
        }
    } else {
        // UTF-8 path: find last newline then count UTF-8 characters
        match find_last_byte_simd(search_slice, b'\n', true) {
            Some(newline_pos) => {
                let line_start = newline_pos + 1;
                let line_bytes = &search_slice[line_start..];
                count_utf8_chars_simd(line_bytes)
            }
            None => {
                // No newline found, count characters from start
                count_utf8_chars_simd(search_slice)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_string() {
        assert!(is_ascii_simd(""));
    }

    #[test]
    fn test_pure_ascii() {
        assert!(is_ascii_simd("Hello, World!"));
        assert!(is_ascii_simd("123456789"));
        assert!(is_ascii_simd("ABCDEFGHIJKLMNOPQRSTUVWXYZ"));
        assert!(is_ascii_simd("abcdefghijklmnopqrstuvwxyz"));
        assert!(is_ascii_simd("!@#$%^&*()_+-=[]{}|;':\",./<>?"));
    }

    #[test]
    fn test_ascii_with_newlines_and_tabs() {
        assert!(is_ascii_simd("Hello\nWorld\t!"));
        assert!(is_ascii_simd("\t\n\r"));
    }

    #[test]
    fn test_ascii_control_characters() {
        // Test ASCII control characters (0-31, 127)
        assert!(is_ascii_simd("\x00\x01\x02\x03\x04\x05\x06\x07"));
        assert!(is_ascii_simd("\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F"));
        assert!(is_ascii_simd("\x10\x11\x12\x13\x14\x15\x16\x17"));
        assert!(is_ascii_simd("\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F"));
        assert!(is_ascii_simd("\x7F")); // DEL character
    }

    #[test]
    fn test_non_ascii_characters() {
        // UTF-8 encoded non-ASCII characters
        assert!(!is_ascii_simd("café")); // contains é
        assert!(!is_ascii_simd("naïve")); // contains ï
        assert!(!is_ascii_simd("résumé")); // contains é
        assert!(!is_ascii_simd("🚀")); // emoji
        assert!(!is_ascii_simd("こんにちは")); // Japanese
        assert!(!is_ascii_simd("Привет")); // Russian
        assert!(!is_ascii_simd("مرحبا")); // Arabic
        // all together for fun
        assert!(!is_ascii_simd(
            "café مرحبا こんにちは 🚀 Привет résumé naïve"
        ));
    }

    #[test]
    fn test_mixed_ascii_non_ascii() {
        assert!(!is_ascii_simd("Hello café"));
        assert!(!is_ascii_simd("ASCII and 🚀"));
        assert!(!is_ascii_simd("test\u{200B}")); // zero-width space
    }

    #[test]
    fn test_long_ascii_strings() {
        // Test strings longer than typical SIMD vector width
        let long_ascii = "a".repeat(1000);
        assert!(is_ascii_simd(&long_ascii));

        let long_ascii_mixed = "ABC123!@#".repeat(100);
        assert!(is_ascii_simd(&long_ascii_mixed));
    }

    #[test]
    fn test_long_non_ascii_strings() {
        let long_non_ascii = "café".repeat(100);
        assert!(!is_ascii_simd(&long_non_ascii));
    }

    #[test]
    fn test_ascii_boundary_values() {
        // Test characters at ASCII boundaries
        assert!(is_ascii_simd("\x00")); // NULL (0)
        assert!(is_ascii_simd("\x7F")); // DEL (127)

        // Test non-ASCII characters (properly encoded UTF-8)
        assert!(!is_ascii_simd("ü")); // UTF-8 encoded ü (first byte is 0xC3)
        assert!(!is_ascii_simd("€")); // UTF-8 encoded € (first byte is 0xE2)
    }

    #[test]
    fn test_various_lengths() {
        // Test strings of various lengths to exercise both SIMD and scalar paths
        for i in 1..=100 {
            let ascii_string = "a".repeat(i);
            assert!(is_ascii_simd(&ascii_string), "Failed for length {}", i);
        }
    }

    #[test]
    fn test_non_ascii_at_different_positions() {
        // Non-ASCII at the beginning
        assert!(!is_ascii_simd("éabc"));

        // Non-ASCII in the middle
        assert!(!is_ascii_simd("abéc"));

        // Non-ASCII at the end
        assert!(!is_ascii_simd("abcé"));

        // Multiple non-ASCII characters
        assert!(!is_ascii_simd("éabcé"));
    }

    #[test]
    fn test_consistency_with_str_is_ascii() {
        let test_strings = vec![
            "",
            "Hello",
            "café",
            "🚀",
            "ASCII123!@#",
            "test\u{200B}",
            "\x00\x7F",
        ];

        // Test regular strings
        for test_str in &test_strings {
            assert_eq!(
                is_ascii_simd(test_str),
                test_str.is_ascii(),
                "Mismatch for string: {:?}",
                test_str
            );
        }

        // Test long string separately
        let long_string = "a".repeat(1000);
        assert_eq!(
            is_ascii_simd(&long_string),
            long_string.is_ascii(),
            "Mismatch for long string"
        );

        // Test additional non-ASCII characters
        let non_ascii_chars = ["ü", "€", "漢", "🎉"];
        for ch in &non_ascii_chars {
            assert_eq!(
                is_ascii_simd(ch),
                ch.is_ascii(),
                "Mismatch for non-ASCII character: {:?}",
                ch
            );
        }
    }

    #[test]
    fn test_simd_vector_width_boundaries() {
        // Test strings that are exactly SIMD vector width and around those boundaries
        // Common SIMD widths are 16, 32, 64 bytes
        for width in [16, 32, 64] {
            // Exactly vector width
            let exact = "a".repeat(width);
            assert!(is_ascii_simd(&exact));

            // One less than vector width
            let one_less = "a".repeat(width - 1);
            assert!(is_ascii_simd(&one_less));

            // One more than vector width
            let one_more = "a".repeat(width + 1);
            assert!(is_ascii_simd(&one_more));

            // Non-ASCII at exact boundary
            let mut boundary_test = "a".repeat(width - 1);
            boundary_test.push('é');
            assert!(!is_ascii_simd(&boundary_test));
        }
    }

    #[test]
    fn test_all_ascii_characters() {
        // Test all valid ASCII characters (0-127)
        let mut all_ascii = String::new();
        for i in 0u8..=127 {
            all_ascii.push(i as char);
        }
        assert!(is_ascii_simd(&all_ascii));
    }

    #[test]
    fn debug_simple_case() {
        // Test with simple ASCII first
        assert!(is_ascii_simd("a"));
        assert!(is_ascii_simd("aa"));
        assert!(is_ascii_simd("aaa"));

        // Test with simple non-ASCII
        assert!(!is_ascii_simd("é"));

        println!("Simple cases work");
    }

    // Tests for find_last_byte_simd
    #[test]
    fn test_find_last_byte_empty() {
        assert_eq!(find_last_byte_simd(&[], b'a', false), None);
    }

    #[test]
    fn test_find_last_byte_single() {
        assert_eq!(find_last_byte_simd(&[b'a'], b'a', false), Some(0));
        assert_eq!(find_last_byte_simd(&[b'a'], b'b', false), None);
    }

    #[test]
    fn test_find_last_byte_multiple() {
        let haystack = b"hello world hello";
        assert_eq!(find_last_byte_simd(haystack, b'l', false), Some(15)); // Last 'l'
        assert_eq!(find_last_byte_simd(haystack, b'h', false), Some(12)); // Last 'h'
        assert_eq!(find_last_byte_simd(haystack, b'o', false), Some(16)); // Last 'o'
        assert_eq!(find_last_byte_simd(haystack, b'x', false), None); // Not found
    }

    #[test]
    fn test_find_last_byte_newlines() {
        let text = b"line1\nline2\nline3";
        assert_eq!(find_last_byte_simd(text, b'\n', true), Some(11)); // Last newline

        let single_line = b"no newlines here";
        assert_eq!(find_last_byte_simd(single_line, b'\n', true), None);
    }

    #[test]
    fn test_find_last_byte_long() {
        // Test with strings longer than SIMD width
        let long_text = "a".repeat(100) + "b" + &"a".repeat(100);
        let bytes = long_text.as_bytes();
        assert_eq!(find_last_byte_simd(bytes, b'b', false), Some(100));
    }

    // Tests for count_utf8_chars_simd
    #[test]
    fn test_count_utf8_chars_empty() {
        assert_eq!(count_utf8_chars_simd(&[]), 0);
    }

    #[test]
    fn test_count_utf8_chars_ascii() {
        assert_eq!(count_utf8_chars_simd(b"hello"), 5);
        assert_eq!(count_utf8_chars_simd(b"Hello, World!"), 13);
        assert_eq!(count_utf8_chars_simd(b"123"), 3);
    }

    #[test]
    fn test_count_utf8_chars_utf8() {
        // "café" in UTF-8: c(1) a(1) f(1) é(2 bytes: 0xC3 0xA9)
        assert_eq!(count_utf8_chars_simd("café".as_bytes()), 4);

        // "🚀" in UTF-8: 4 bytes (0xF0 0x9F 0x9A 0x80)
        assert_eq!(count_utf8_chars_simd("🚀".as_bytes()), 1);

        // Mixed: "Hello🚀" = 5 ASCII + 1 emoji = 6 chars
        assert_eq!(count_utf8_chars_simd("Hello🚀".as_bytes()), 6);
    }

    #[test]
    fn test_count_utf8_chars_consistency() {
        let test_strings = vec!["Hello", "café", "🚀", "Hello, 世界!", "résumé", "测试", ""];

        for test_str in test_strings {
            let simd_count = count_utf8_chars_simd(test_str.as_bytes());
            let std_count = test_str.chars().count();
            assert_eq!(simd_count, std_count, "Mismatch for string: {:?}", test_str);
        }
    }

    // Tests for get_char_column_simd
    #[test]
    fn test_get_char_column_simple() {
        // Simple case: no newlines
        assert_eq!(get_char_column_simd("hello", 5), 5);
        assert_eq!(get_char_column_simd("hello", 3), 3);
        assert_eq!(get_char_column_simd("hello", 0), 0);
    }

    #[test]
    fn test_get_char_column_with_newlines() {
        let text = "line1\nline2\nline3";

        // Position at start of each line
        assert_eq!(get_char_column_simd(text, 0), 0); // Start of "line1"
        assert_eq!(get_char_column_simd(text, 6), 0); // Start of "line2"
        assert_eq!(get_char_column_simd(text, 12), 0); // Start of "line3"

        // Positions within lines
        assert_eq!(get_char_column_simd(text, 3), 3); // "lin|e1"
        assert_eq!(get_char_column_simd(text, 9), 3); // "lin|e2"
        assert_eq!(get_char_column_simd(text, 15), 3); // "lin|e3"
    }

    #[test]
    fn test_get_char_column_utf8() {
        // Test with UTF-8 characters
        let text = "café\nnaïve";

        // Position within first line: "ca|fé" = position 2
        assert_eq!(get_char_column_simd(text, 2), 2);

        // Position at start of second line after newline
        assert_eq!(get_char_column_simd(text, 6), 0); // Start of "naïve"

        // Position within second line: "na|ïve" = position 2 (after 'n', 'a')
        assert_eq!(get_char_column_simd(text, 8), 2);
    }

    #[test]
    fn test_get_char_column_consistency_with_original() {
        fn original_get_char_column(text: &str, offset: usize) -> usize {
            let src = text.as_bytes();
            let mut col = 0;
            for &b in src[..offset].iter().rev() {
                if b == b'\n' {
                    break;
                }
                if b & 0b1100_0000 != 0b1000_0000 {
                    col += 1;
                }
            }
            col
        }

        let test_cases = vec![
            ("hello", vec![0, 1, 3, 5]),
            ("line1\nline2", vec![0, 3, 5, 6, 9]),
            ("café\nworld", vec![0, 2, 5, 6, 8]),
            ("🚀test\nnew", vec![0, 1, 3, 6, 7]),
            ("", vec![0]),
            ("a", vec![0, 1]),
        ];

        for (text, offsets) in test_cases {
            for offset in offsets {
                if offset <= text.len() {
                    let original = original_get_char_column(text, offset);
                    let simd = get_char_column_simd(text, offset);
                    assert_eq!(
                        original, simd,
                        "Mismatch for text: {:?}, offset: {}",
                        text, offset
                    );
                }
            }
        }
    }

    #[test]
    fn test_get_char_column_edge_cases() {
        // Test edge cases
        assert_eq!(get_char_column_simd("", 0), 0);
        assert_eq!(get_char_column_simd("test", 0), 0);
        assert_eq!(get_char_column_simd("test", 100), 0); // Offset beyond length

        // Test with only newlines
        assert_eq!(get_char_column_simd("\n\n\n", 1), 0);
        assert_eq!(get_char_column_simd("\n\n\n", 2), 0);

        // Test long lines
        let long_line = "a".repeat(1000);
        assert_eq!(get_char_column_simd(&long_line, 500), 500);

        let long_with_newline = "a".repeat(500) + "\n" + &"b".repeat(300);
        assert_eq!(get_char_column_simd(&long_with_newline, 800), 299);
    }
}
