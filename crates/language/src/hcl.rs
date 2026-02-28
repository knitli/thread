// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT
#![cfg(test)]
use super::*;
use crate::test::{test_match_lang, test_replace_lang};

fn test_match(s1: &str, s2: &str) {
    test_match_lang(s1, s2, Hcl)
}

#[test]
fn test_hcl_pattern() {
    test_match("$A = $B", r#"foo = "bar""#);
    test_match(
        "resource $TYPE $NAME $BODY",
        r#"resource "aws_instance" "example" { ami = "ami-123" }"#,
    );
    test_match(
        "$BLOCK $BODY",
        r#"terraform { required_providers { aws = { source = "hashicorp/aws" } } }"#,
    );
    test_match(
        "variable $NAME $CONFIG",
        r#"variable "region" { default = "us-west-2" }"#,
    );
    test_match(
        "output $NAME $VALUE",
        r#"output "instance_ip" { value = aws_instance.example.public_ip }"#,
    );
    test_match("$VAR = [$$$ITEMS]", r#"tags = ["production", "web"]"#);
    test_match(
        "$VAR = { $$$PAIRS }",
        r#"labels = { environment = "prod", team = "backend" }"#,
    );
    test_match(r#"$VAR = "$CONTENT""#, r#"name = "instance""#);
}

fn test_replace(src: &str, pattern: &str, replacer: &str) -> String {
    test_replace_lang(src, pattern, replacer, Hcl)
}

#[test]
fn test_hcl_replace() {
    let ret = test_replace(r#"foo = "bar""#, r#"$A = $B"#, r#"$B = $A"#);
    assert_eq!(ret, r#""bar" = foo"#);

    let ret = test_replace(
        r#"resource "aws_instance" "example" { ami = "ami-123" }"#,
        r#"resource $TYPE $NAME $BODY"#,
        r#"resource $NAME $TYPE $BODY"#,
    );
    assert_eq!(
        ret,
        r#"resource "example" "aws_instance" { ami = "ami-123" }"#
    );

    let ret = test_replace(
        r#"variable "region" { default = "us-west-2" }"#,
        r#"variable "region" { default = $DEFAULT }"#,
        r#"variable "region" { default = "eu-west-1" }"#,
    );
    assert_eq!(ret, r#"variable "region" { default = "eu-west-1" }"#);
}
