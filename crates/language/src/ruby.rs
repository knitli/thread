#![cfg(test)]

// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

use super::*;
use thread_ast_engine::Pattern;

fn test_match(query: &str, source: &str) {
    use crate::test::test_match_lang;
    test_match_lang(query, source, Ruby);
}

#[test]
fn test_ruby_pattern() {
    test_match("Foo::bar", "Foo::bar");
}

// https://github.com/ast-grep/ast-grep/issues/713
#[test]
fn test_ruby_tree_sitter_panic() {
    let pattern = Pattern::new("Foo::barbaz", &Ruby);
    assert_eq!(pattern.fixed_string(), "barbaz");
}

fn test_replace(src: &str, pattern: &str, replacer: &str) -> String {
    use crate::test::test_replace_lang;
    test_replace_lang(src, pattern, replacer, Ruby)
}

#[test]
fn test_ruby_replace() {
    let ret = test_replace("Foo::bar()", "Foo::$METHOD()", "$METHOD()");
    assert_eq!(ret, "bar()");
}
