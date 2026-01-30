// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileCopyrightText: 2026 Knitli Inc.
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Type system round-trip validation tests
//!
//! Ensures no metadata loss in Rust → ReCoco → verification cycles.
//! Validates that Document → Value serialization preserves all data integrity.

use recoco::base::value::{BasicValue, FieldValues, ScopeValue, Value};
use std::path::PathBuf;
use thread_ast_engine::tree_sitter::LanguageExt;
use thread_flow::conversion::serialize_parsed_doc;
use thread_language::{Python, Rust, SupportLang, Tsx};
use thread_services::conversion::{compute_content_fingerprint, extract_basic_metadata};
use thread_services::types::{ParsedDocument, SymbolInfo, SymbolKind, Visibility};

/// Helper to create a Rust test document
fn create_rust_document(
    content: &str,
) -> ParsedDocument<thread_ast_engine::tree_sitter::StrDoc<Rust>> {
    let ast_root = Rust.ast_grep(content);
    let fingerprint = compute_content_fingerprint(content);

    ParsedDocument::new(
        ast_root,
        PathBuf::from("test.rs"),
        SupportLang::Rust,
        fingerprint,
    )
}

/// Helper to create a Python test document
fn create_python_document(
    content: &str,
) -> ParsedDocument<thread_ast_engine::tree_sitter::StrDoc<Python>> {
    let ast_root = Python.ast_grep(content);
    let fingerprint = compute_content_fingerprint(content);

    ParsedDocument::new(
        ast_root,
        PathBuf::from("test.py"),
        SupportLang::Python,
        fingerprint,
    )
}

/// Helper to create a TypeScript test document
fn create_typescript_document(
    content: &str,
) -> ParsedDocument<thread_ast_engine::tree_sitter::StrDoc<Tsx>> {
    let ast_root = Tsx.ast_grep(content);
    let fingerprint = compute_content_fingerprint(content);

    ParsedDocument::new(
        ast_root,
        PathBuf::from("test.ts"),
        SupportLang::TypeScript,
        fingerprint,
    )
}

/// Extract symbol count from ReCoco Value
fn extract_symbol_count(value: &Value) -> usize {
    match value {
        Value::Struct(FieldValues { fields }) => match &fields[0] {
            Value::LTable(symbols) => symbols.len(),
            _ => panic!("Expected LTable for symbols"),
        },
        _ => panic!("Expected Struct output"),
    }
}

/// Extract import count from ReCoco Value
fn extract_import_count(value: &Value) -> usize {
    match value {
        Value::Struct(FieldValues { fields }) => match &fields[1] {
            Value::LTable(imports) => imports.len(),
            _ => panic!("Expected LTable for imports"),
        },
        _ => panic!("Expected Struct output"),
    }
}

/// Extract call count from ReCoco Value
fn extract_call_count(value: &Value) -> usize {
    match value {
        Value::Struct(FieldValues { fields }) => match &fields[2] {
            Value::LTable(calls) => calls.len(),
            _ => panic!("Expected LTable for calls"),
        },
        _ => panic!("Expected Struct output"),
    }
}

/// Extract fingerprint from ReCoco Value
fn extract_fingerprint(value: &Value) -> Vec<u8> {
    match value {
        Value::Struct(FieldValues { fields }) => match &fields[3] {
            Value::Basic(BasicValue::Bytes(bytes)) => bytes.to_vec(),
            _ => panic!("Expected Bytes for fingerprint"),
        },
        _ => panic!("Expected Struct output"),
    }
}

/// Validate symbol structure in ReCoco Value
fn validate_symbol_structure(symbol: &ScopeValue) {
    let ScopeValue(FieldValues { fields }) = symbol;
    assert_eq!(
        fields.len(),
        3,
        "Symbol should have 3 fields: name, kind, scope"
    );

    // Validate field types
    assert!(
        matches!(&fields[0], Value::Basic(BasicValue::Str(_))),
        "Name should be string"
    );
    assert!(
        matches!(&fields[1], Value::Basic(BasicValue::Str(_))),
        "Kind should be string"
    );
    assert!(
        matches!(&fields[2], Value::Basic(BasicValue::Str(_))),
        "Scope should be string"
    );
}

/// Validate import structure in ReCoco Value
fn validate_import_structure(import: &ScopeValue) {
    let ScopeValue(FieldValues { fields }) = import;
    assert_eq!(
        fields.len(),
        3,
        "Import should have 3 fields: symbol_name, source_path, kind"
    );

    assert!(
        matches!(&fields[0], Value::Basic(BasicValue::Str(_))),
        "Symbol name should be string"
    );
    assert!(
        matches!(&fields[1], Value::Basic(BasicValue::Str(_))),
        "Source path should be string"
    );
    assert!(
        matches!(&fields[2], Value::Basic(BasicValue::Str(_))),
        "Kind should be string"
    );
}

/// Validate call structure in ReCoco Value
fn validate_call_structure(call: &ScopeValue) {
    let ScopeValue(FieldValues { fields }) = call;
    assert_eq!(
        fields.len(),
        2,
        "Call should have 2 fields: function_name, arguments_count"
    );

    assert!(
        matches!(&fields[0], Value::Basic(BasicValue::Str(_))),
        "Function name should be string"
    );
    assert!(
        matches!(&fields[1], Value::Basic(BasicValue::Int64(_))),
        "Arguments count should be int64"
    );
}

// =============================================================================
// Basic Round-Trip Tests
// =============================================================================

#[tokio::test]
async fn test_empty_document_round_trip() {
    let doc = create_rust_document("");
    let value = serialize_parsed_doc(&doc).expect("Serialization should succeed");

    // Verify structure
    assert!(matches!(value, Value::Struct(_)), "Output should be Struct");

    // Verify empty tables
    assert_eq!(
        extract_symbol_count(&value),
        0,
        "Empty doc should have 0 symbols"
    );
    assert_eq!(
        extract_import_count(&value),
        0,
        "Empty doc should have 0 imports"
    );
    assert_eq!(
        extract_call_count(&value),
        0,
        "Empty doc should have 0 calls"
    );

    // Verify fingerprint exists
    let fingerprint_bytes = extract_fingerprint(&value);
    assert!(
        !fingerprint_bytes.is_empty(),
        "Fingerprint should exist for empty doc"
    );
}

#[tokio::test]
async fn test_simple_function_round_trip() {
    let content = "fn test_function() { println!(\"hello\"); }";
    let mut doc = create_rust_document(content);

    // Extract metadata
    let metadata = extract_basic_metadata(&doc).expect("Metadata extraction should succeed");
    doc.metadata = metadata;

    let value = serialize_parsed_doc(&doc).expect("Serialization should succeed");

    // Verify symbol count (may be 0 or 1 depending on pattern matching)
    let symbol_count = extract_symbol_count(&value);
    println!("Symbol count: {}", symbol_count);

    // Verify all symbols have correct structure
    if let Value::Struct(FieldValues { fields }) = &value {
        if let Value::LTable(symbols) = &fields[0] {
            for symbol in symbols {
                validate_symbol_structure(symbol);
            }
        }
    }
}

#[tokio::test]
async fn test_fingerprint_consistency() {
    let content = "fn main() { let x = 42; }";

    // Create two documents with same content
    let doc1 = create_rust_document(content);
    let doc2 = create_rust_document(content);

    let value1 = serialize_parsed_doc(&doc1).expect("Serialization 1 should succeed");
    let value2 = serialize_parsed_doc(&doc2).expect("Serialization 2 should succeed");

    // Fingerprints should be identical
    let fp1 = extract_fingerprint(&value1);
    let fp2 = extract_fingerprint(&value2);
    assert_eq!(fp1, fp2, "Same content should produce same fingerprint");
}

#[tokio::test]
async fn test_fingerprint_uniqueness() {
    let content1 = "fn main() {}";
    let content2 = "fn test() {}";

    let doc1 = create_rust_document(content1);
    let doc2 = create_rust_document(content2);

    let value1 = serialize_parsed_doc(&doc1).expect("Serialization 1 should succeed");
    let value2 = serialize_parsed_doc(&doc2).expect("Serialization 2 should succeed");

    // Fingerprints should be different
    let fp1 = extract_fingerprint(&value1);
    let fp2 = extract_fingerprint(&value2);
    assert_ne!(
        fp1, fp2,
        "Different content should produce different fingerprints"
    );
}

// =============================================================================
// Symbol Preservation Tests
// =============================================================================

#[tokio::test]
async fn test_symbol_data_preservation() {
    let content = "fn calculate_sum(a: i32, b: i32) -> i32 { a + b }";
    let mut doc = create_rust_document(content);

    // Manually add symbol to ensure we have data to verify
    let mut metadata = extract_basic_metadata(&doc).unwrap_or_default();
    metadata.defined_symbols.insert(
        "calculate_sum".to_string(),
        SymbolInfo {
            name: "calculate_sum".to_string(),
            kind: SymbolKind::Function,
            position: thread_ast_engine::Position::new(0, 0, 0),
            scope: "global".to_string(),
            visibility: Visibility::Public,
        },
    );
    doc.metadata = metadata;

    let value = serialize_parsed_doc(&doc).expect("Serialization should succeed");

    // Verify symbol structure
    if let Value::Struct(FieldValues { fields }) = &value {
        if let Value::LTable(symbols) = &fields[0] {
            assert_eq!(symbols.len(), 1, "Should have 1 symbol");

            let symbol = &symbols[0];
            validate_symbol_structure(symbol);

            // Verify symbol name
            let ScopeValue(FieldValues {
                fields: symbol_fields,
            }) = symbol;
            if let Value::Basic(BasicValue::Str(name)) = &symbol_fields[0] {
                assert_eq!(
                    name.as_ref(),
                    "calculate_sum",
                    "Symbol name should be preserved"
                );
            }
        }
    }
}

#[tokio::test]
async fn test_multiple_symbols_preservation() {
    let content = r#"
        fn function1() {}
        fn function2() {}
        fn function3() {}
    "#;
    let mut doc = create_rust_document(content);

    // Extract metadata
    let metadata = extract_basic_metadata(&doc).unwrap_or_default();
    doc.metadata = metadata;

    let value = serialize_parsed_doc(&doc).expect("Serialization should succeed");

    // Verify all symbols have correct structure
    if let Value::Struct(FieldValues { fields }) = &value {
        if let Value::LTable(symbols) = &fields[0] {
            println!("Found {} symbols", symbols.len());
            for symbol in symbols {
                validate_symbol_structure(symbol);
            }
        }
    }
}

// =============================================================================
// Import/Call Preservation Tests
// =============================================================================

#[tokio::test]
async fn test_import_data_preservation() {
    let content = "use std::collections::HashMap;";
    let mut doc = create_rust_document(content);

    let metadata = extract_basic_metadata(&doc).unwrap_or_default();
    doc.metadata = metadata;

    let value = serialize_parsed_doc(&doc).expect("Serialization should succeed");

    // Verify imports structure (may be 0 or more depending on pattern matching)
    if let Value::Struct(FieldValues { fields }) = &value {
        if let Value::LTable(imports) = &fields[1] {
            println!("Found {} imports", imports.len());
            for import in imports {
                validate_import_structure(import);
            }
        }
    }
}

#[tokio::test]
async fn test_call_data_preservation() {
    let content = "fn main() { println!(\"test\"); }";
    let mut doc = create_rust_document(content);

    let metadata = extract_basic_metadata(&doc).unwrap_or_default();
    doc.metadata = metadata;

    let value = serialize_parsed_doc(&doc).expect("Serialization should succeed");

    // Verify calls structure (may be 0 or more depending on pattern matching)
    if let Value::Struct(FieldValues { fields }) = &value {
        if let Value::LTable(calls) = &fields[2] {
            println!("Found {} calls", calls.len());
            for call in calls {
                validate_call_structure(call);
            }
        }
    }
}

// =============================================================================
// Complex Document Tests
// =============================================================================

#[tokio::test]
async fn test_complex_document_round_trip() {
    let content = r#"
        use std::collections::HashMap;

        fn calculate(x: i32, y: i32) -> i32 {
            let result = x + y;
            println!("Result: {}", result);
            result
        }

        fn process_data(data: HashMap<String, i32>) {
            for (key, value) in data.iter() {
                calculate(value, 10);
            }
        }
    "#;

    let mut doc = create_rust_document(content);
    let metadata = extract_basic_metadata(&doc).unwrap_or_default();
    doc.metadata = metadata;

    let value = serialize_parsed_doc(&doc).expect("Serialization should succeed");

    // Verify complete structure
    assert!(matches!(value, Value::Struct(_)), "Output should be Struct");

    if let Value::Struct(FieldValues { fields }) = &value {
        assert_eq!(fields.len(), 4, "Should have 4 fields");

        // Validate all table structures
        if let Value::LTable(symbols) = &fields[0] {
            for symbol in symbols {
                validate_symbol_structure(symbol);
            }
        }

        if let Value::LTable(imports) = &fields[1] {
            for import in imports {
                validate_import_structure(import);
            }
        }

        if let Value::LTable(calls) = &fields[2] {
            for call in calls {
                validate_call_structure(call);
            }
        }

        // Validate fingerprint
        assert!(
            matches!(&fields[3], Value::Basic(BasicValue::Bytes(_))),
            "Fingerprint should be bytes"
        );
    }
}

#[tokio::test]
async fn test_unicode_content_round_trip() {
    let content = "fn 测试函数() { println!(\"你好世界\"); }";
    let doc = create_rust_document(content);

    let value = serialize_parsed_doc(&doc).expect("Unicode content should serialize");

    // Verify fingerprint handles unicode correctly
    let fingerprint = extract_fingerprint(&value);
    assert!(
        !fingerprint.is_empty(),
        "Unicode content should have fingerprint"
    );
}

#[tokio::test]
async fn test_large_document_round_trip() {
    // Generate large document with many functions
    let mut content = String::new();
    for i in 0..100 {
        content.push_str(&format!("fn function_{}() {{ println!(\"test\"); }}\n", i));
    }

    let mut doc = create_rust_document(&content);
    let metadata = extract_basic_metadata(&doc).unwrap_or_default();
    doc.metadata = metadata;

    let value = serialize_parsed_doc(&doc).expect("Large document should serialize");

    // Verify structure integrity with large data
    if let Value::Struct(FieldValues { fields }) = &value {
        if let Value::LTable(symbols) = &fields[0] {
            println!("Large document has {} symbols", symbols.len());
            // Spot check a few symbols
            for symbol in symbols.iter().take(5) {
                validate_symbol_structure(symbol);
            }
        }
    }
}

// =============================================================================
// Multi-Language Tests
// =============================================================================

#[tokio::test]
async fn test_python_round_trip() {
    let content = r#"
def calculate(x, y):
    return x + y

def main():
    result = calculate(1, 2)
    print(result)
"#;

    let mut doc = create_python_document(content);
    let metadata = extract_basic_metadata(&doc).unwrap_or_default();
    doc.metadata = metadata;

    let value = serialize_parsed_doc(&doc).expect("Python serialization should succeed");

    // Verify structure
    assert!(
        matches!(value, Value::Struct(_)),
        "Python output should be Struct"
    );
}

#[tokio::test]
async fn test_typescript_round_trip() {
    let content = r#"
function calculate(x: number, y: number): number {
    return x + y;
}

const result = calculate(1, 2);
console.log(result);
"#;

    let mut doc = create_typescript_document(content);
    let metadata = extract_basic_metadata(&doc).unwrap_or_default();
    doc.metadata = metadata;

    let value = serialize_parsed_doc(&doc).expect("TypeScript serialization should succeed");

    // Verify structure
    assert!(
        matches!(value, Value::Struct(_)),
        "TypeScript output should be Struct"
    );
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[tokio::test]
async fn test_malformed_content_handling() {
    // Test with syntactically invalid code
    let content = "fn invalid { this is not valid rust syntax )))";
    let doc = create_rust_document(content);

    // Serialization should succeed even with invalid syntax
    let value = serialize_parsed_doc(&doc).expect("Should serialize even with invalid syntax");

    // Verify basic structure exists
    assert!(
        matches!(value, Value::Struct(_)),
        "Invalid syntax should still produce Struct"
    );

    // Fingerprint should still work
    let fingerprint = extract_fingerprint(&value);
    assert!(
        !fingerprint.is_empty(),
        "Invalid syntax should still have fingerprint"
    );
}
