// SPDX-FileCopyrightText: 2025 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

// Sample Rust code for benchmarking
fn test_function() {
    println!("Hello World");
    println!("test string");
    println!("template {}", variable);
}

struct TestStruct {
    value: i32,
}

impl TestStruct {
    fn new() -> Self {
        Self { value: 42 }
    }

    fn method(&self) {
        println!("{}", self.value);
    }
}

use std::collections::HashMap;
use std::fs::File;

static VARIABLE: &str = "test";
const CONSTANT: i32 = 123;

async fn async_function() -> Result<String, Box<dyn std::error::Error>> {
    let result = fetch_data().await?;
    Ok(result)
}

fn recursion() {
    recursion();
}

fn main() {
    test_function();
}
