#![cfg(test)]

// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

use super::*;

fn test_match(query: &str, source: &str) {
    use crate::test::test_match_lang;
    test_match_lang(query, source, Json);
}

fn test_non_match(query: &str, source: &str) {
    use crate::test::test_non_match_lang;
    test_non_match_lang(query, source, Json);
}

#[test]
fn test_json_str() {
    test_match("123", "123");
    test_match("{\"d\": 123}", "{\"d\": 123}");
    test_non_match("344", "123");
    test_non_match("{\"key\": 123}", "{}");
}

#[test]
fn test_json_pattern() {
    test_match("$A", "123");
    test_match(r#"[$A]"#, r#"[123]"#);
    test_match(r#"{ $$$ }"#, r#"{"abc": 123}"#);
}

fn test_replace(src: &str, pattern: &str, replacer: &str) -> String {
    use crate::test::test_replace_lang;
    test_replace_lang(src, pattern, replacer, Json)
}

#[test]
fn test_json_replace() {
    let ret = test_replace(r#"{ "a": 123 }"#, r#"123"#, r#"456"#);
    assert_eq!(ret, r#"{ "a": 456 }"#);
}
