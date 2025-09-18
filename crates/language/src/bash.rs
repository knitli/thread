#![cfg(test)]

// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

use super::*;

fn test_match(query: &str, source: &str) {
    use crate::test::test_match_lang;
    test_match_lang(query, source, Bash);
}

fn test_non_match(query: &str, source: &str) {
    use crate::test::test_non_match_lang;
    test_non_match_lang(query, source, Bash);
}

fn test_replace(src: &str, pattern: &str, replacer: &str) -> String {
    use crate::test::test_replace_lang;
    test_replace_lang(src, pattern, replacer, Bash)
}

#[test]
fn test_bash_pattern() {
    test_match("123", "123");
    test_match("echo $A", "echo test");
    // TODO
    // test_match("echo { $A }", "echo {1..10}");
    test_match("echo $abc", "echo $abc");
}

#[test]
fn test_bash_pattern_no_match() {
    test_non_match("echo $abc", "echo test");
    test_non_match("echo $abc", "echo $ABC");
}

#[test]
fn test_bash_replace() {
    // TODO: change the replacer to log $A
    let ret = test_replace("echo 123", "echo $A", "log 123");
    assert_eq!(ret, "log 123");
}
