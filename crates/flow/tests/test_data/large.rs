// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Large file for performance testing

use std::collections::HashMap;

pub struct LargeStruct {
    field1: String,
    field2: i32,
    field3: bool,
}

impl LargeStruct {
    pub fn new() -> Self {
        Self {
            field1: String::new(),
            field2: 0,
            field3: false,
        }
    }

    pub fn method1(&self) -> String {
        self.field1.clone()
    }

    pub fn method2(&self) -> i32 {
        self.field2
    }

    pub fn method3(&self) -> bool {
        self.field3
    }
}

pub fn function1() -> i32 { 1 }
pub fn function2() -> i32 { 2 }
pub fn function3() -> i32 { 3 }
pub fn function4() -> i32 { 4 }
pub fn function5() -> i32 { 5 }
pub fn function6() -> i32 { 6 }
pub fn function7() -> i32 { 7 }
pub fn function8() -> i32 { 8 }
pub fn function9() -> i32 { 9 }
pub fn function10() -> i32 { 10 }

pub fn caller() {
    function1();
    function2();
    function3();
    function4();
    function5();
    function6();
    function7();
    function8();
    function9();
    function10();
}

pub struct Config {
    pub settings: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            settings: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn set(&mut self, key: String, value: String) {
        self.settings.insert(key, value);
    }
}

#[derive(Debug, Clone)]
pub enum Status {
    Active,
    Inactive,
    Pending,
}

pub trait Processor {
    fn process(&self) -> Result<(), String>;
}

impl Processor for LargeStruct {
    fn process(&self) -> Result<(), String> {
        Ok(())
    }
}

pub mod nested {
    pub fn nested_function() -> String {
        "nested".to_string()
    }

    pub struct NestedStruct;
}
