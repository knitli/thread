#![cfg(test)]
// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

use super::*;

fn test_match(query: &str, source: &str) {
    use crate::test::test_match_lang;
    test_match_lang(query, source, Solidity);
}

fn test_non_match(query: &str, source: &str) {
    use crate::test::test_non_match_lang;
    test_non_match_lang(query, source, Solidity);
}

#[test]
fn test_solidity_str() {
    test_match("pragma solidity 0.8.28;", "pragma solidity 0.8.28;");
    test_match(
        r#"import { Test } from "forge-std/Test.sol";"#,
        r#"import { Test } from "forge-std/Test.sol";"#,
    );
    test_non_match("pragma solidity 0.8.28;", "pragma solidity 0.8.26;");
    test_non_match(
        r#"import { Test } from "forge-std/Test.sol";"#,
        r#"import { console } from "forge-std/Test.sol";"#,
    );
}

#[test]
fn test_solidity_pattern() {
    test_match(
        r#"import { $A } from "forge-std/Test.sol";"#,
        r#"import { Test } from "forge-std/Test.sol";"#,
    );
    test_match(
        r#"import { $$$ } from "forge-std/Test.sol";"#,
        r#"import { Test, console } from "forge-std/Test.sol";"#,
    );
}

fn test_replace(src: &str, pattern: &str, replacer: &str) -> String {
    use crate::test::test_replace_lang;
    test_replace_lang(src, pattern, replacer, Solidity)
}

#[test]
fn test_solidity_replace() {
    let ret = test_replace(
        r#"import { Test } from "forge-std/Test.sol";"#,
        "Test",
        "console",
    );
    assert_eq!(ret, r#"import { console } from "forge-std/Test.sol";"#);
}
