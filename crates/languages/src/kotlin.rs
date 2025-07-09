#![cfg(test)]

// SPDX-FileCopyrightText: 2025 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: MIT


use super::*;

fn test_match(query: &str, source: &str) {
  use crate::test::test_match_lang;
  test_match_lang(query, source, Kotlin);
}

fn test_non_match(query: &str, source: &str) {
  use crate::test::test_non_match_lang;
  test_non_match_lang(query, source, Kotlin);
}

#[test]
fn test_kotlin_str() {
  test_match("println($A)", "println(123)");
  test_match("println('123')", "println('123')");
  test_non_match("println('123')", "println('456')");
  test_non_match("'123'", "'456'");
}

#[test]
fn test_kotlin_pattern() {
  test_match("$A = 0", "a = 0");
  test_match(
    r#"fun $A($$$): $B { $$$ }"#,
    r#"
fun plus(a: Int, b: Int): Int {
  return a + b
}"#,
  );
}

fn test_replace(src: &str, pattern: &str, replacer: &str) -> String {
  use crate::test::test_replace_lang;
  test_replace_lang(src, pattern, replacer, Kotlin)
}

#[test]
fn test_kotlin_replace() {
  let ret = test_replace(
    r#"
fun plus(a: Int, b: Int): Int {
  return a + b
}"#,
    r#"fun $F($$$): $R { $$$BODY }"#,
    r#"fun $F() { $$$BODY }"#,
  );
  assert_eq!(
    ret,
    r#"
fun plus() { return a + b }"#
  );
}
