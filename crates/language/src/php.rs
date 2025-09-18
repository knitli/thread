#![cfg(test)]

// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

use super::*;

fn test_match(query: &str, source: &str) {
    use crate::test::test_match_lang;
    test_match_lang(query, source, Php);
}

#[test]
fn test_php_pattern() {
    // dummy example, php pattern actually does not work
    test_match("123", "123");
}

// https://github.com/ast-grep/ast-grep/issues/639#issuecomment-1876622828
// TODO: better php support
