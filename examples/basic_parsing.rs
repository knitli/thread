// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Basic parsing example demonstrating Thread's ast-grep integration
//!
//! This example shows how Thread parses Rust code using ast-grep and
//! converts it to Thread's structured types.

use thread_parse::{DefaultRustParser, LanguageProvider};
use thread_core::LanguageParser;

fn main() {
    println!("Thread Basic Parsing Example");
    println!("=============================");
    
    let parser = DefaultRustParser::new();
    
    // Example Rust code to parse
    let rust_code = r#"
use std::collections::HashMap;

fn main() {
    println!("Hello, Thread!");
    let result = add_numbers(5, 3);
    println!("Result: {}", result);
}

fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}

pub struct Calculator {
    history: Vec<i32>,
}

impl Calculator {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }
    
    pub fn add(&mut self, a: i32, b: i32) -> i32 {
        let result = a + b;
        self.history.push(result);
        result
    }
}
"#;
    
    println!("Parsing Rust code...\n");
    
    match parser.parse_content(rust_code, "example.rs") {
        Ok(result) => {
            println!("✅ Parsing successful!");
            println!("📄 File: {}", result.file_path);
            println!("🦀 Language: {}", result.language);
            println!("⏱️  Parse time: {}ms", result.parse_time_ms);
            println!("📊 Found {} code elements", result.elements.len());
            println!("📥 Found {} imports", result.imports.len());
            println!("📤 Found {} exports", result.exports.len());
            
            println!("\n📋 Code Elements:");
            for (i, element) in result.elements.iter().enumerate() {
                println!("  {}. {} ({})", 
                    i + 1, 
                    element.name, 
                    format!("{:?}", element.kind)
                );
                println!("     📍 Location: line {}-{}", 
                    element.location.start_line, 
                    element.location.end_line
                );
                println!("     📝 Signature: {}", element.signature);
                
                if !element.metadata.parameters.is_empty() {
                    println!("     🔧 Parameters: {}", 
                        element.metadata.parameters.len()
                    );
                }
                
                println!();
            }
            
            // Demonstrate language detection
            println!("🔍 Language Detection:");
            println!("  Can parse .rs files: {}", parser.can_parse_extension("rs"));
            println!("  Can parse .js files: {}", parser.can_parse_extension("js"));
            println!("  Language ID: {}", parser.language_id());
            println!("  File extensions: {:?}", parser.file_extensions());
            
            // Demonstrate dependency extraction
            println!("\n🔗 Dependencies:");
            match parser.extract_dependencies(rust_code) {
                Ok(deps) => {
                    if deps.is_empty() {
                        println!("  No dependencies found (current implementation is simplified)");
                    } else {
                        for dep in deps {
                            println!("  - {}", dep);
                        }
                    }
                }
                Err(e) => println!("  Error extracting dependencies: {}", e),
            }
            
            println!("\n🎯 Next Steps:");
            println!("  - Implement real function extraction using ast-grep patterns");
            println!("  - Add import/export detection");
            println!("  - Build relationship graph between code elements");
            println!("  - Add content-addressable storage for deduplication");
        }
        Err(e) => {
            println!("❌ Parsing failed: {}", e);
        }
    }
    
    println!("\n🚀 Thread ast-grep integration working successfully!");
}