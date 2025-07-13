#![cfg(test)]

// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

use super::*;

fn test_match(query: &str, source: &str) {
  use crate::test::test_match_lang;
  test_match_lang(query, source, Css);
}

#[test]
fn test_css_pattern() {
  test_match("$A { color: red; }", ".a { color: red; }");
  test_match(".a { color: $COLOR; }", ".a { color: red; }");
  test_match(".a { $PROP: red; }", ".a { color: red; }");
}

fn test_replace(src: &str, pattern: &str, replacer: &str) -> String {
  use crate::test::test_replace_lang;
  test_replace_lang(src, pattern, replacer, Css)
}

#[test]
fn test_css_replace() {
  let ret = test_replace(
    ".a {color: red; }",
    ".a { color: $COLOR}",
    ".a {background: $COLOR}",
  );
  assert_eq!(ret, ".a {background: red}");
}
