//! This module defines the `Doc` and `Content` traits to abstract away source code encoding issues.
//!
//! ast-grep supports three kinds of encoding: utf-8 for CLI, utf-16 for nodeJS napi and `Vec<char>` for wasm.
//! Different encoding will produce different tree-sitter Node's range and position.
//!
//! The `Content` trait is defined to abstract different encoding.
//! It is used as associated type bound `Source` in the `Doc` trait.
//! Its associated type `Underlying`  represents the underlying type of the content, e.g. `Vec<u8>`, `Vec<u16>`.
//!
//! `Doc` is a trait that defines a document that can be parsed by Tree-sitter.
//! It has a `Source` associated type bounded by `Content` that represents the source code of the document,
//! and a `Lang` associated type that represents the language of the document.

pub use ag_service_types::Content;
use std::borrow::Cow;
use std::ops::Range;

impl Content for String {
  type Underlying = u8;
  fn get_range(&self, range: Range<usize>) -> &[Self::Underlying] {
    &self.as_bytes()[range]
  }
  fn decode_str(src: &str) -> Cow<[Self::Underlying]> {
    Cow::Borrowed(src.as_bytes())
  }
  fn encode_bytes(bytes: &[Self::Underlying]) -> Cow<str> {
    String::from_utf8_lossy(bytes)
  }

  /// This is an O(n) operation. We assume the col will not be a
  /// huge number in reality. This may be problematic for special
  /// files like compressed js
  fn get_char_column(&self, _col: usize, offset: usize) -> usize {
    let src = self.as_bytes();
    let mut col = 0;
    // TODO: is it possible to use SIMD here???
    for &b in src[..offset].iter().rev() {
      if b == b'\n' {
        break;
      }
      // https://en.wikipedia.org/wiki/UTF-8#Description
      if b & 0b1100_0000 != 0b1000_0000 {
        col += 1;
      }
    }
    col
  }
}
