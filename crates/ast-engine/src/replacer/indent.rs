#![allow(clippy::doc_overindented_list_items)]

// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

//! # Indentation-Preserving Code Replacement
//!
//! Handles automatic indentation adjustment during code replacement to maintain
//! proper formatting when inserting multi-line code snippets.
//!
//! ## The Challenge
//!
//! When replacing AST nodes with new code that contains meta-variables, we need to:
//! 1. Preserve the relative indentation within captured variables
//! 2. Adjust indentation to match the replacement context
//! 3. Maintain overall source code formatting
//!
//! ## Algorithm Overview
//!
//! The indentation algorithm works in three phases:
//!
//! ### 1. Extract with De-indent
//! Extract captured meta-variables and normalize their indentation by removing
//! the original context indentation (except from the first line).
//!
//! ### 2. Insert with Re-indent
//! Insert the normalized meta-variable content into the replacement template,
//! applying the replacement context's indentation.
//!
//! ### 3. Final Re-indent
//! Adjust the entire replacement to match the original matched node's indentation
//! in the source code.
//!
//! ## Example Walkthrough
//!
//! **Original Code:**
//! ```ignore
//! if (true) {
//!   a(
//!     1
//!       + 2
//!       + 3
//!   )
//! }
//! ```
//!
//! **Pattern:** `a($B)`
//! **Replacement:** `c(\n  $B\n)`
//!
//! **Step 1 - Extract `$B` (indented at 4 spaces):**
//! ```ignore
//! 1
//!   + 2    // Relative indent preserved
//!   + 3
//! ```
//!
//! **Step 2 - Insert into replacement (2 space context):**
//! ```ignore
//! c(
//!   1
//!     + 2  // 2 + 2 = 4 spaces total
//!     + 3
//! )
//! ```
//!
//! **Step 3 - Final indent (match original 2 space context):**
//! ```ignore
//! if (true) {
//!   c(
//!     1
//!       + 2
//!       + 3
//!   )
//! }
//! ```
//!
//! ## Key Types
//!
//! - [`DeindentedExtract`] - Represents extracted content with indentation info
//! - [`extract_with_deindent`] - Extracts and normalizes meta-variable content
//! - [`indent_lines`] - Applies indentation to multi-line content
//!
//! ## Limitations
//!
//! - Only supports space-based indentation (tabs not fully supported)
//! - Assumes well-formed input indentation
//! - Performance overhead for large code blocks
//! - Complex algorithm with edge cases
use crate::source::Content;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::ops::Range;

/// We assume `NEW_LINE`, `TAB`, `SPACE` is only one code unit.
/// This is sufficiently true for utf8, utf16 and char.
fn get_new_line<C: Content>() -> C::Underlying {
    C::decode_str("\n")[0].clone()
}
fn get_space<C: Content>() -> C::Underlying {
    C::decode_str(" ")[0].clone()
}

const MAX_LOOK_AHEAD: usize = 512;

/// Extracted content with indentation information for later re-indentation.
///
/// Represents the result of extracting a meta-variable's content from source code,
/// along with the indentation context needed for proper re-insertion.
pub enum DeindentedExtract<'a, C: Content> {
    /// Single-line content that doesn't require indentation adjustment.
    ///
    /// Contains just the raw content bytes since there are no line breaks
    /// to worry about for indentation purposes.
    SingleLine(&'a [C::Underlying]),

    /// Multi-line content with original indentation level recorded.
    ///
    /// Contains the content bytes and the number of spaces that were used
    /// for indentation in the original context. The first line's indentation
    /// is not included in the content.
    ///
    /// # Fields
    /// - Content bytes with relative indentation preserved
    /// - Original indentation level (number of spaces)
    MultiLine(&'a [C::Underlying], usize),
}

/// Extract content from source code and prepare it for indentation-aware replacement.
///
/// Analyzes the content at the given range and determines whether it needs
/// indentation processing. For multi-line content, calculates the original
/// indentation level for later re-indentation.
///
/// # Parameters
///
/// - `content` - Source content to extract from
/// - `range` - Byte range of the content to extract
///
/// # Returns
///
/// [`DeindentedExtract`] containing the content and indentation information
///
/// # Example
///
/// ```rust,ignore
/// let source = "  if (true) {\n    console.log('test');\n  }";
/// let extract = extract_with_deindent(&source, 2..source.len());
/// // Returns MultiLine with 2-space indentation context
/// ```
pub fn extract_with_deindent<C: Content>(
    content: &C,
    range: Range<usize>,
) -> DeindentedExtract<'_, C> {
    let extract_slice = content.get_range(range.clone());
    // no need to compute indentation for single line
    if !extract_slice.contains(&get_new_line::<C>()) {
        return DeindentedExtract::SingleLine(extract_slice);
    }
    let indent = get_indent_at_offset::<C>(content.get_range(0..range.start));
    DeindentedExtract::MultiLine(extract_slice, indent)
}

#[allow(dead_code)]
fn deindent_slice<'a, C: Content>(
    slice: &'a [C::Underlying],
    content: &'a C,
    start: usize,
) -> DeindentedExtract<'a, C> {
    if !slice.contains(&get_new_line::<C>()) {
        return DeindentedExtract::SingleLine(slice);
    }
    let indent = get_indent_at_offset::<C>(content.get_range(0..start));
    DeindentedExtract::MultiLine(slice, indent)
}

pub fn formatted_slice<'a, C: Content>(
    slice: &'a [C::Underlying],
    content: &'a C,
    start: usize,
) -> Cow<'a, [C::Underlying]> {
    if !slice.contains(&get_new_line::<C>()) {
        return Cow::Borrowed(slice);
    }
    Cow::Owned(indent_lines::<C>(0, &DeindentedExtract::MultiLine(slice, get_indent_at_offset::<C>(content.get_range(0..start)))).into_owned())
}

pub fn indent_lines<'a, C: Content>(
    indent: usize,
    extract: &'a DeindentedExtract<'a, C>,
) -> Cow<'a, [C::Underlying]> {
    use DeindentedExtract::{MultiLine, SingleLine};
    let (lines, original_indent) = match extract {
        SingleLine(line) => return Cow::Borrowed(line),
        MultiLine(lines, ind) => (lines, ind),
    };
    match original_indent.cmp(&indent) {
        // if old and new indent match, just return old lines
        Ordering::Equal => Cow::Borrowed(lines),
        // need strip old indent
        Ordering::Greater => Cow::Owned(remove_indent::<C>(original_indent - indent, lines)),
        // need add missing indent
        Ordering::Less => Cow::Owned(indent_lines_impl::<C, _>(
            indent - original_indent,
            lines.split(|b| *b == get_new_line::<C>()),
        )),
    }
}

fn indent_lines_impl<'a, C, Lines>(indent: usize, mut lines: Lines) -> Vec<C::Underlying>
where
    C: Content + 'a,
    Lines: Iterator<Item = &'a [C::Underlying]>,
{
    let mut ret = vec![];
    let space = get_space::<C>();
    let leading: Vec<_> = std::iter::repeat_n(space, indent).collect();
    // first line wasn't indented, so we don't add leading spaces
    if let Some(line) = lines.next() {
        ret.extend(line.iter().cloned());
    }
    let new_line = get_new_line::<C>();
    for line in lines {
        ret.push(new_line.clone());
        ret.extend(leading.clone());
        ret.extend(line.iter().cloned());
    }
    ret
}

/// returns 0 if no indent is found before the offset
/// either truly no indent exists, or the offset is in a long line
pub fn get_indent_at_offset<C: Content>(src: &[C::Underlying]) -> usize {
    let lookahead = src.len().max(MAX_LOOK_AHEAD) - MAX_LOOK_AHEAD;

    let mut indent = 0;
    let new_line = get_new_line::<C>();
    let space = get_space::<C>();
    // TODO: support TAB. only whitespace is supported now
    for c in src[lookahead..].iter().rev() {
        if *c == new_line {
            return indent;
        }
        if *c == space {
            indent += 1;
        } else {
            indent = 0;
        }
    }
    // lookahead == 0 means we have indentation at first line.
    if lookahead == 0 && indent != 0 {
        indent
    } else {
        0
    }
}

// NOTE: we assume input is well indented.
// following lines should have fewer indentations than initial line
fn remove_indent<C: Content>(indent: usize, src: &[C::Underlying]) -> Vec<C::Underlying> {
    let indentation: Vec<_> = std::iter::repeat_n(get_space::<C>(), indent)
        .collect();
    let new_line = get_new_line::<C>();
    let lines: Vec<_> = src
        .split(|b| *b == new_line)
        .map(|line| match line.strip_prefix(&*indentation) {
            Some(stripped) => stripped,
            None => line,
        })
        .collect();
    lines.join(&new_line).clone()
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_deindent(source: &str, expected: &str, offset: usize) {
        let source = source.to_string();
        let expected = expected.trim();
        let start = source[offset..]
            .chars()
            .take_while(|n| n.is_whitespace())
            .count()
            + offset;
        let trailing_white = source
            .chars()
            .rev()
            .take_while(|n| n.is_whitespace())
            .count();
        let end = source.chars().count() - trailing_white;
        let extracted = extract_with_deindent(&source, start..end);
        let result_bytes = indent_lines::<String>(0, &extracted);
        let actual = std::str::from_utf8(&result_bytes).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_simple_deindent() {
        let src = r"
  def test():
    pass";
        let expected = r"
def test():
  pass";
        test_deindent(src, expected, 0);
    }

    #[test]
    fn test_first_line_indent_deindent() {
        // note this indentation has no newline
        let src = r"  def test():
    pass";
        let expected = r"
def test():
  pass";
        test_deindent(src, expected, 0);
    }

    #[test]
    fn test_space_in_middle_deindent() {
        let src = r"
a = lambda:
  pass";
        let expected = r"
lambda:
  pass";
        test_deindent(src, expected, 4);
    }

    #[test]
    fn test_middle_deindent() {
        let src = r"
  a = lambda:
    pass";
        let expected = r"
lambda:
  pass";
        test_deindent(src, expected, 6);
    }

    #[test]
    fn test_nested_deindent() {
        let src = r"
def outer():
  def test():
    pass";
        let expected = r"
def test():
  pass";
        test_deindent(src, expected, 13);
    }

    #[test]
    fn test_no_deindent() {
        let src = r"
def test():
  pass
";
        test_deindent(src, src, 0);
    }

    #[test]
    fn test_malformed_deindent() {
        let src = r"
  def test():
pass
";
        let expected = r"
def test():
pass
";
        test_deindent(src, expected, 0);
    }

    #[test]
    fn test_long_line_no_deindent() {
        let src = format!("{}abc\n  def", " ".repeat(MAX_LOOK_AHEAD + 1));
        test_deindent(&src, &src, 0);
    }

    fn test_replace_with_indent(target: &str, start: usize, inserted: &str) -> String {
        let target = target.to_string();
        let replace_lines = DeindentedExtract::MultiLine(inserted.as_bytes(), 0);
        let indent = get_indent_at_offset::<String>(&target.as_bytes()[..start]);
        let ret = indent_lines::<String>(indent, &replace_lines);
        String::from_utf8(ret.to_vec()).unwrap()
    }

    #[test]
    fn test_simple_replace() {
        let target = "";
        let inserted = "def abc(): pass";
        let actual = test_replace_with_indent(target, 0, inserted);
        assert_eq!(actual, inserted);
        let inserted = "def abc():\n  pass";
        let actual = test_replace_with_indent(target, 0, inserted);
        assert_eq!(actual, inserted);
    }

    #[test]
    fn test_indent_replace() {
        let target = "  ";
        let inserted = "def abc(): pass";
        let actual = test_replace_with_indent(target, 2, inserted);
        assert_eq!(actual, "def abc(): pass");
        let inserted = "def abc():\n  pass";
        let actual = test_replace_with_indent(target, 2, inserted);
        assert_eq!(actual, "def abc():\n    pass");
        let target = "    "; // 4 spaces, but insert at 2
        let actual = test_replace_with_indent(target, 2, inserted);
        assert_eq!(actual, "def abc():\n    pass");
        let target = "    "; // 4 spaces, insert at 4
        let actual = test_replace_with_indent(target, 4, inserted);
        assert_eq!(actual, "def abc():\n      pass");
    }

    #[test]
    fn test_leading_text_replace() {
        let target = "a = ";
        let inserted = "def abc(): pass";
        let actual = test_replace_with_indent(target, 4, inserted);
        assert_eq!(actual, "def abc(): pass");
        let inserted = "def abc():\n  pass";
        let actual = test_replace_with_indent(target, 4, inserted);
        assert_eq!(actual, "def abc():\n  pass");
    }

    #[test]
    fn test_leading_text_indent_replace() {
        let target = "  a = ";
        let inserted = "def abc(): pass";
        let actual = test_replace_with_indent(target, 6, inserted);
        assert_eq!(actual, "def abc(): pass");
        let inserted = "def abc():\n  pass";
        let actual = test_replace_with_indent(target, 6, inserted);
        assert_eq!(actual, "def abc():\n    pass");
    }
}
