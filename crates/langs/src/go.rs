#![cfg(test)]

// SPDX-FileCopyrightText: 2025 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: MIT

use super::*;

fn test_match(query: &str, source: &str) {
  use crate::test::test_match_lang;
  test_match_lang(query, source, Go);
}

fn test_non_match(query: &str, source: &str) {
  use crate::test::test_non_match_lang;
  test_non_match_lang(query, source, Go);
}

#[test]
fn test_go_str() {
  test_match("print($A)", "print(123)");
  test_match("print('123')", "print('123')");
  test_non_match("print('123')", "print('456')");
  test_non_match("'123'", "'456'");
}

#[test]
fn test_go_pattern() {
  test_match("$A = 0", "a = 0");
  test_match(
    r#"func $A($$$) $B { $$$ }"#,
    r#"
func plus(a int, b int) int {
  return a + b
}"#,
  );
}

fn test_replace(src: &str, pattern: &str, replacer: &str) -> String {
  use crate::test::test_replace_lang;
  test_replace_lang(src, pattern, replacer, Go)
}

#[test]
fn test_go_replace() {
  let ret = test_replace(
    r#"
func intSeq() {
  defer func()  {
      i++
  }()
}"#,
    r#"defer func() {
$$$BODY }()"#,
    r#"func b() { $$$BODY }"#,
  );
  assert_eq!(
    ret,
    r#"
func intSeq() {
  func b() { i++ }
}"#
  );
}
