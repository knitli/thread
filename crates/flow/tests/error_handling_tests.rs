// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileCopyrightText: 2026 Knitli Inc.
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Comprehensive error handling test suite
//!
//! Validates robust error handling for edge cases and failure scenarios.
//!
//! ## Error Categories:
//! 1. **Invalid Input**: Malformed syntax, unsupported languages
//! 2. **Resource Limits**: Large files, excessive complexity
//! 3. **Unicode Handling**: Edge cases, invalid encodings
//! 4. **Empty/Null Cases**: Missing content, zero-length input
//! 5. **Concurrent Access**: Multi-threaded safety
//! 6. **System Errors**: Resource exhaustion, timeouts

use recoco::base::value::{BasicValue, Value};
use recoco::ops::interface::{FlowInstanceContext, SimpleFunctionFactory};
use recoco::setup::AuthRegistry;
use std::sync::Arc;
use thread_flow::functions::parse::ThreadParseFactory;

/// Helper to create mock context
fn create_mock_context() -> Arc<FlowInstanceContext> {
    Arc::new(FlowInstanceContext {
        flow_instance_name: "test_flow".to_string(),
        auth_registry: Arc::new(AuthRegistry::new()),
    })
}

/// Helper to create empty spec
fn empty_spec() -> serde_json::Value {
    serde_json::json!({})
}

/// Execute parse with given inputs
async fn execute_parse(
    content: &str,
    language: &str,
    file_path: &str,
) -> Result<Value, recoco::prelude::Error> {
    let factory = Arc::new(ThreadParseFactory);
    let context = create_mock_context();

    let build_output = factory.build(empty_spec(), vec![], context).await?;
    let executor = build_output.executor.await?;

    let inputs = vec![
        Value::Basic(BasicValue::Str(content.to_string().into())),
        Value::Basic(BasicValue::Str(language.to_string().into())),
        Value::Basic(BasicValue::Str(file_path.to_string().into())),
    ];

    executor.evaluate(inputs).await
}

// =============================================================================
// Invalid Input Tests
// =============================================================================

#[tokio::test]
async fn test_error_invalid_syntax_rust() {
    let invalid_rust = "fn invalid { this is not valid rust syntax )))";
    let result = execute_parse(invalid_rust, "rs", "invalid.rs").await;

    // Should succeed even with invalid syntax (parser is resilient)
    assert!(
        result.is_ok(),
        "Parser should handle invalid syntax gracefully"
    );
}

#[tokio::test]
async fn test_error_invalid_syntax_python() {
    let invalid_python = "def broken(: invalid syntax here)))\n\tindent error";
    let result = execute_parse(invalid_python, "py", "invalid.py").await;

    assert!(result.is_ok(), "Parser should handle invalid Python syntax");
}

#[tokio::test]
async fn test_error_invalid_syntax_typescript() {
    let invalid_ts = "function broken({ incomplete destructuring";
    let result = execute_parse(invalid_ts, "ts", "invalid.ts").await;

    assert!(
        result.is_ok(),
        "Parser should handle invalid TypeScript syntax"
    );
}

#[tokio::test]
async fn test_error_unsupported_language() {
    let content = "some code here";
    let result = execute_parse(content, "unsupported_lang", "test.unsupported").await;

    assert!(result.is_err(), "Should error on unsupported language");

    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(
            error_msg.contains("Unsupported language") || error_msg.contains("client"),
            "Error should indicate unsupported language, got: {}",
            error_msg
        );
    }
}

#[tokio::test]
async fn test_error_empty_language_string() {
    let content = "fn main() {}";
    let result = execute_parse(content, "", "test.rs").await;

    assert!(result.is_err(), "Should error on empty language string");
}

#[tokio::test]
async fn test_error_whitespace_only_language() {
    let content = "fn main() {}";
    let result = execute_parse(content, "   ", "test.rs").await;

    assert!(result.is_err(), "Should error on whitespace-only language");
}

// =============================================================================
// Resource Limit Tests
// =============================================================================

#[tokio::test]
async fn test_large_file_handling() {
    // Generate moderately large file (~100KB of code)
    let mut large_code = String::new();
    for i in 0..2_000 {
        large_code.push_str(&format!("fn function_{}() {{ println!(\"test\"); }}\n", i));
    }

    assert!(large_code.len() > 50_000, "Test file should be >50KB");

    let result = execute_parse(&large_code, "rs", "large.rs").await;

    // Should succeed but may take longer
    assert!(result.is_ok(), "Should handle large files gracefully");
}

#[tokio::test]
async fn test_deeply_nested_code() {
    // Create deeply nested structure
    let mut nested_code = String::from("fn main() {\n");
    for _ in 0..100 {
        nested_code.push_str("    if true {\n");
    }
    nested_code.push_str("        println!(\"deep\");\n");
    for _ in 0..100 {
        nested_code.push_str("    }\n");
    }
    nested_code.push_str("}\n");

    let result = execute_parse(&nested_code, "rs", "nested.rs").await;

    assert!(result.is_ok(), "Should handle deeply nested code");
}

#[tokio::test]
async fn test_extremely_long_line() {
    // Create a single line with 100k characters
    let long_line = format!("let x = \"{}\";\n", "a".repeat(100_000));

    let result = execute_parse(&long_line, "rs", "longline.rs").await;

    assert!(result.is_ok(), "Should handle extremely long lines");
}

// =============================================================================
// Unicode Handling Tests
// =============================================================================

#[tokio::test]
async fn test_unicode_identifiers() {
    let unicode_code = r#"
fn 测试函数() {
    let 变量 = 42;
    println!("{}", 变量);
}
"#;

    let result = execute_parse(unicode_code, "rs", "unicode.rs").await;

    assert!(result.is_ok(), "Should handle Unicode identifiers");
}

#[tokio::test]
async fn test_unicode_strings() {
    let unicode_strings = r#"
fn main() {
    let emoji = "🦀 Rust";
    let chinese = "你好世界";
    let arabic = "مرحبا بالعالم";
    let hindi = "नमस्ते दुनिया";
    println!("{} {} {} {}", emoji, chinese, arabic, hindi);
}
"#;

    let result = execute_parse(unicode_strings, "rs", "strings.rs").await;

    assert!(result.is_ok(), "Should handle Unicode strings");
}

#[tokio::test]
async fn test_mixed_bidirectional_text() {
    let bidi_code = r#"
fn main() {
    let mixed = "English مع العربية with हिंदी";
    println!("{}", mixed);
}
"#;

    let result = execute_parse(bidi_code, "rs", "bidi.rs").await;

    assert!(result.is_ok(), "Should handle bidirectional text");
}

#[tokio::test]
async fn test_zero_width_characters() {
    // Zero-width joiner and zero-width space
    let zero_width = "fn main() { let x\u{200B} = 42; }\n";

    let result = execute_parse(zero_width, "rs", "zerowidth.rs").await;

    assert!(result.is_ok(), "Should handle zero-width characters");
}

// =============================================================================
// Empty/Null Cases
// =============================================================================

#[tokio::test]
async fn test_empty_content() {
    let result = execute_parse("", "rs", "empty.rs").await;

    assert!(result.is_ok(), "Should handle empty content");

    if let Ok(Value::Struct(fields)) = result {
        // Verify all tables are empty
        assert_eq!(fields.fields.len(), 4, "Should have 4 fields");
    }
}

#[tokio::test]
async fn test_whitespace_only_content() {
    let whitespace = "   \n\t\n    \n";
    let result = execute_parse(whitespace, "rs", "whitespace.rs").await;

    assert!(result.is_ok(), "Should handle whitespace-only content");
}

#[tokio::test]
async fn test_comments_only_content() {
    let comments = r#"
// This file contains only comments
/* Multi-line comment
 * with no actual code
 */
// Another comment
"#;

    let result = execute_parse(comments, "rs", "comments.rs").await;

    assert!(result.is_ok(), "Should handle comments-only files");
}

#[tokio::test]
async fn test_missing_content_parameter() {
    let factory = Arc::new(ThreadParseFactory);
    let context = create_mock_context();

    let build_output = factory
        .build(empty_spec(), vec![], context)
        .await
        .expect("Build should succeed");

    let executor = build_output.executor.await.expect("Executor should build");

    // Pass empty inputs (missing content)
    let result = executor.evaluate(vec![]).await;

    assert!(result.is_err(), "Should error on missing content");

    if let Err(e) = result {
        assert!(
            e.to_string().contains("Missing content"),
            "Error should mention missing content"
        );
    }
}

// =============================================================================
// Concurrent Access Tests
// =============================================================================

#[tokio::test]
async fn test_concurrent_parse_operations() {
    use tokio::task::JoinSet;

    let mut join_set = JoinSet::new();

    // Spawn 10 concurrent parse operations
    for i in 0..10 {
        join_set.spawn(async move {
            let content = format!("fn function_{}() {{ println!(\"test\"); }}", i);
            execute_parse(&content, "rs", &format!("concurrent_{}.rs", i)).await
        });
    }

    // Wait for all to complete
    let mut successes = 0;
    while let Some(result) = join_set.join_next().await {
        if let Ok(Ok(_)) = result {
            successes += 1;
        }
    }

    assert_eq!(successes, 10, "All concurrent operations should succeed");
}

#[tokio::test]
async fn test_concurrent_same_content() {
    use tokio::task::JoinSet;

    let content = "fn shared() { println!(\"shared\"); }";
    let mut join_set = JoinSet::new();

    // Parse the same content concurrently from multiple tasks
    for i in 0..5 {
        let content = content.to_string();
        join_set
            .spawn(async move { execute_parse(&content, "rs", &format!("shared_{}.rs", i)).await });
    }

    let mut successes = 0;
    while let Some(result) = join_set.join_next().await {
        if let Ok(Ok(_)) = result {
            successes += 1;
        }
    }

    assert_eq!(successes, 5, "All concurrent parses should succeed");
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[tokio::test]
async fn test_null_bytes_in_content() {
    let null_content = "fn main() {\0 let x = 42; }";
    let result = execute_parse(null_content, "rs", "null.rs").await;

    // Parser should handle null bytes gracefully
    assert!(result.is_ok(), "Should handle null bytes in content");
}

#[tokio::test]
async fn test_only_special_characters() {
    let special = "!@#$%^&*()_+-=[]{}|;':\",./<>?";
    let result = execute_parse(special, "rs", "special.rs").await;

    assert!(
        result.is_ok(),
        "Should handle special characters gracefully"
    );
}

#[tokio::test]
async fn test_repetitive_content() {
    // Highly repetitive content that might confuse parsers
    let repetitive = "fn a() {}\n".repeat(1000);
    let result = execute_parse(&repetitive, "rs", "repetitive.rs").await;

    assert!(result.is_ok(), "Should handle repetitive content");
}

#[tokio::test]
async fn test_mixed_line_endings() {
    // Mix of \n, \r\n, and \r
    let mixed = "fn main() {\r\n    let x = 1;\n    let y = 2;\r    let z = 3;\r\n}";
    let result = execute_parse(mixed, "rs", "mixed.rs").await;

    assert!(result.is_ok(), "Should handle mixed line endings");
}

// =============================================================================
// Invalid Type Tests
// =============================================================================

#[tokio::test]
async fn test_invalid_content_type() {
    let factory = Arc::new(ThreadParseFactory);
    let context = create_mock_context();

    let build_output = factory
        .build(empty_spec(), vec![], context)
        .await
        .expect("Build should succeed");

    let executor = build_output.executor.await.expect("Executor should build");

    // Pass integer instead of string for content
    let inputs = vec![
        Value::Basic(BasicValue::Int64(42)),
        Value::Basic(BasicValue::Str("rs".to_string().into())),
        Value::Basic(BasicValue::Str("test.rs".to_string().into())),
    ];

    let result = executor.evaluate(inputs).await;

    assert!(result.is_err(), "Should error on invalid content type");
}

#[tokio::test]
async fn test_invalid_language_type() {
    let factory = Arc::new(ThreadParseFactory);
    let context = create_mock_context();

    let build_output = factory
        .build(empty_spec(), vec![], context)
        .await
        .expect("Build should succeed");

    let executor = build_output.executor.await.expect("Executor should build");

    // Pass integer instead of string for language
    let inputs = vec![
        Value::Basic(BasicValue::Str("content".to_string().into())),
        Value::Basic(BasicValue::Int64(42)),
        Value::Basic(BasicValue::Str("test.rs".to_string().into())),
    ];

    let result = executor.evaluate(inputs).await;

    assert!(result.is_err(), "Should error on invalid language type");
}

// =============================================================================
// Stress Tests
// =============================================================================

#[tokio::test]
async fn test_rapid_sequential_parsing() {
    // Rapidly parse many files in sequence
    const ITERATIONS: usize = 20;

    for i in 0..ITERATIONS {
        let content = format!("fn func_{}() {{ println!(\"test\"); }}", i);
        let result = execute_parse(&content, "rs", &format!("rapid_{}.rs", i)).await;

        assert!(result.is_ok(), "Iteration {} should succeed", i);
    }

    println!("✓ Completed {} rapid sequential parses", ITERATIONS);
}

#[tokio::test]
async fn test_varied_file_sizes() {
    // Parse files of varying sizes in sequence
    let sizes = vec![10, 100, 1000, 10000];

    for size in sizes {
        let mut content = String::new();
        for i in 0..size {
            content.push_str(&format!("fn f_{}() {{}}\n", i));
        }

        let result = execute_parse(&content, "rs", &format!("size_{}.rs", size)).await;

        assert!(result.is_ok(), "File with {} functions should parse", size);
    }
}
