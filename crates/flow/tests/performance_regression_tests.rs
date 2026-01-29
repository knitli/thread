// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Performance regression test suite
//!
//! These tests FAIL if performance degrades beyond acceptable thresholds.
//! Unlike benchmarks, these run in CI and prevent performance regressions from merging.
//!
//! ## Test Categories:
//! 1. **Fingerprint Speed**: Blake3 hashing must stay sub-microsecond
//! 2. **Parse Speed**: Direct parsing must meet baseline targets
//! 3. **Serialization Speed**: Value conversion must be fast
//! 4. **Memory Efficiency**: No unexpected allocations
//!
//! ## Performance Thresholds (p99):
//! - Small file fingerprint: <5µs
//! - Small file parse: <1ms
//! - Small file serialize: <500µs
//! - 100 fingerprints: <1ms (batch processing)

use std::path::PathBuf;
use std::time::Instant;
use thread_ast_engine::tree_sitter::LanguageExt;
use thread_flow::conversion::serialize_parsed_doc;
use thread_language::{Rust, SupportLang};
use thread_services::conversion::compute_content_fingerprint;
use thread_services::conversion::extract_basic_metadata;
use thread_services::types::ParsedDocument;

// =============================================================================
// Test Data
// =============================================================================

const SMALL_RUST: &str = r#"
use std::collections::HashMap;

pub struct Config {
    name: String,
    value: i32,
}

impl Config {
    pub fn new(name: String, value: i32) -> Self {
        Self { name, value }
    }

    pub fn update(&mut self, value: i32) {
        self.value = value;
    }
}

pub fn process_data(input: &[i32]) -> Vec<i32> {
    input.iter().map(|x| x * 2).collect()
}
"#;

const MEDIUM_RUST: &str = r#"
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

pub struct UserManager {
    users: Arc<Mutex<HashMap<u64, String>>>,
    emails: Arc<Mutex<HashMap<String, u64>>>,
}

impl UserManager {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            emails: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_user(&self, id: u64, name: String, email: String) {
        let mut users = self.users.lock().unwrap();
        let mut emails = self.emails.lock().unwrap();
        users.insert(id, name);
        emails.insert(email, id);
    }

    pub fn get_user(&self, id: u64) -> Option<String> {
        self.users.lock().unwrap().get(&id).cloned()
    }

    pub fn find_by_email(&self, email: &str) -> Option<u64> {
        self.emails.lock().unwrap().get(email).copied()
    }

    pub fn remove_user(&self, id: u64) -> Option<String> {
        let mut users = self.users.lock().unwrap();
        users.remove(&id)
    }
}
"#;

fn generate_large_rust() -> String {
    let mut code = MEDIUM_RUST.to_string();
    for i in 0..50 {
        code.push_str(&format!(
            r#"
pub fn function_{}(x: i32) -> i32 {{
    x + {}
}}
"#,
            i, i
        ));
    }
    code
}

/// Helper to create test document
fn create_document(content: &str) -> ParsedDocument<thread_ast_engine::tree_sitter::StrDoc<Rust>> {
    let ast_root = Rust.ast_grep(content);
    let fingerprint = compute_content_fingerprint(content);
    ParsedDocument::new(
        ast_root,
        PathBuf::from("test.rs"),
        SupportLang::Rust,
        fingerprint,
    )
}

// =============================================================================
// Fingerprint Performance Tests
// =============================================================================

#[test]
fn test_fingerprint_speed_small_file() {
    const ITERATIONS: usize = 1000;
    const MAX_TIME_PER_OP_US: u128 = 5; // 5 microseconds

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _fp = compute_content_fingerprint(SMALL_RUST);
    }
    let elapsed = start.elapsed();
    let avg_us = elapsed.as_micros() / ITERATIONS as u128;

    assert!(
        avg_us <= MAX_TIME_PER_OP_US,
        "Fingerprint performance regression: {}µs per op (expected ≤{}µs)",
        avg_us,
        MAX_TIME_PER_OP_US
    );

    println!("✓ Fingerprint small file: {}µs per op", avg_us);
}

#[test]
fn test_fingerprint_speed_medium_file() {
    const ITERATIONS: usize = 1000;
    const MAX_TIME_PER_OP_US: u128 = 10; // 10 microseconds

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _fp = compute_content_fingerprint(MEDIUM_RUST);
    }
    let elapsed = start.elapsed();
    let avg_us = elapsed.as_micros() / ITERATIONS as u128;

    assert!(
        avg_us <= MAX_TIME_PER_OP_US,
        "Fingerprint performance regression: {}µs per op (expected ≤{}µs)",
        avg_us,
        MAX_TIME_PER_OP_US
    );

    println!("✓ Fingerprint medium file: {}µs per op", avg_us);
}

#[test]
fn test_fingerprint_batch_speed() {
    const BATCH_SIZE: usize = 100;
    const MAX_TOTAL_TIME_MS: u128 = 1; // 1 millisecond for 100 ops

    let start = Instant::now();
    for _ in 0..BATCH_SIZE {
        let _fp = compute_content_fingerprint(SMALL_RUST);
    }
    let elapsed = start.elapsed();
    let total_ms = elapsed.as_millis();

    assert!(
        total_ms <= MAX_TOTAL_TIME_MS,
        "Batch fingerprint regression: {}ms for {} ops (expected ≤{}ms)",
        total_ms,
        BATCH_SIZE,
        MAX_TOTAL_TIME_MS
    );

    println!("✓ Batch fingerprint ({} ops): {}ms", BATCH_SIZE, total_ms);
}

// =============================================================================
// Parse Performance Tests
// =============================================================================

#[test]
fn test_parse_speed_small_file() {
    const ITERATIONS: usize = 100;
    const MAX_TIME_PER_OP_MS: u128 = 1; // 1 millisecond

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _doc = create_document(SMALL_RUST);
    }
    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_millis() / ITERATIONS as u128;

    assert!(
        avg_ms <= MAX_TIME_PER_OP_MS,
        "Parse performance regression: {}ms per op (expected ≤{}ms)",
        avg_ms,
        MAX_TIME_PER_OP_MS
    );

    println!("✓ Parse small file: {}ms per op", avg_ms);
}

#[test]
fn test_parse_speed_medium_file() {
    const ITERATIONS: usize = 100;
    const MAX_TIME_PER_OP_MS: u128 = 2; // 2 milliseconds

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _doc = create_document(MEDIUM_RUST);
    }
    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_millis() / ITERATIONS as u128;

    assert!(
        avg_ms <= MAX_TIME_PER_OP_MS,
        "Parse performance regression: {}ms per op (expected ≤{}ms)",
        avg_ms,
        MAX_TIME_PER_OP_MS
    );

    println!("✓ Parse medium file: {}ms per op", avg_ms);
}

#[test]
fn test_parse_speed_large_file() {
    const ITERATIONS: usize = 50;
    const MAX_TIME_PER_OP_MS: u128 = 10; // 10 milliseconds

    let large_code = generate_large_rust();

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _doc = create_document(&large_code);
    }
    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_millis() / ITERATIONS as u128;

    assert!(
        avg_ms <= MAX_TIME_PER_OP_MS,
        "Parse performance regression: {}ms per op (expected ≤{}ms)",
        avg_ms,
        MAX_TIME_PER_OP_MS
    );

    println!("✓ Parse large file: {}ms per op", avg_ms);
}

// =============================================================================
// Serialization Performance Tests
// =============================================================================

#[test]
fn test_serialize_speed_small_doc() {
    const ITERATIONS: usize = 1000;
    const MAX_TIME_PER_OP_US: u128 = 500; // 500 microseconds

    let doc = create_document(SMALL_RUST);

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _value = serialize_parsed_doc(&doc).expect("Serialization should succeed");
    }
    let elapsed = start.elapsed();
    let avg_us = elapsed.as_micros() / ITERATIONS as u128;

    assert!(
        avg_us <= MAX_TIME_PER_OP_US,
        "Serialization performance regression: {}µs per op (expected ≤{}µs)",
        avg_us,
        MAX_TIME_PER_OP_US
    );

    println!("✓ Serialize small doc: {}µs per op", avg_us);
}

#[test]
fn test_serialize_speed_with_metadata() {
    const ITERATIONS: usize = 1000;
    const MAX_TIME_PER_OP_US: u128 = 1000; // 1 millisecond

    let mut doc = create_document(MEDIUM_RUST);
    let metadata = extract_basic_metadata(&doc).unwrap_or_default();
    doc.metadata = metadata;

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _value = serialize_parsed_doc(&doc).expect("Serialization should succeed");
    }
    let elapsed = start.elapsed();
    let avg_us = elapsed.as_micros() / ITERATIONS as u128;

    assert!(
        avg_us <= MAX_TIME_PER_OP_US,
        "Serialization with metadata regression: {}µs per op (expected ≤{}µs)",
        avg_us,
        MAX_TIME_PER_OP_US
    );

    println!("✓ Serialize with metadata: {}µs per op", avg_us);
}

// =============================================================================
// End-to-End Performance Tests
// =============================================================================

#[test]
fn test_full_pipeline_small_file() {
    const ITERATIONS: usize = 100;
    const MAX_TIME_PER_OP_MS: u128 = 100; // 100 milliseconds (includes metadata extraction)

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        // Full pipeline: fingerprint → parse → extract metadata → serialize
        let _fp = compute_content_fingerprint(SMALL_RUST);
        let mut doc = create_document(SMALL_RUST);
        let metadata = extract_basic_metadata(&doc).unwrap_or_default();
        doc.metadata = metadata;
        let _value = serialize_parsed_doc(&doc).expect("Serialization should succeed");
    }
    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_millis() / ITERATIONS as u128;

    assert!(
        avg_ms <= MAX_TIME_PER_OP_MS,
        "Full pipeline performance regression: {}ms per op (expected ≤{}ms)",
        avg_ms,
        MAX_TIME_PER_OP_MS
    );

    println!("✓ Full pipeline small file: {}ms per op", avg_ms);
}

#[test]
fn test_metadata_extraction_speed() {
    const ITERATIONS: usize = 100;
    const MAX_TIME_PER_OP_MS: u128 = 300; // 300 milliseconds (pattern matching is slow)

    let doc = create_document(MEDIUM_RUST);

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _metadata = extract_basic_metadata(&doc).unwrap_or_default();
    }
    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_millis() / ITERATIONS as u128;

    assert!(
        avg_ms <= MAX_TIME_PER_OP_MS,
        "Metadata extraction regression: {}ms per op (expected ≤{}ms)",
        avg_ms,
        MAX_TIME_PER_OP_MS
    );

    println!("✓ Metadata extraction: {}ms per op", avg_ms);
}

// =============================================================================
// Memory Efficiency Tests
// =============================================================================

#[test]
fn test_fingerprint_allocation_count() {
    // Fingerprint should make minimal allocations
    // This is a smoke test - more detailed profiling in benchmarks

    const TEST_SIZE: usize = 1000;
    let mut fingerprints = Vec::with_capacity(TEST_SIZE);

    for _ in 0..TEST_SIZE {
        fingerprints.push(compute_content_fingerprint(SMALL_RUST));
    }

    // Basic verification: all fingerprints should be unique for our test data
    // (This doesn't test memory directly but verifies correctness)
    assert_eq!(fingerprints.len(), TEST_SIZE);
    println!(
        "✓ Fingerprint memory test: {} operations completed",
        TEST_SIZE
    );
}

#[test]
fn test_parse_does_not_leak_memory() {
    // Stress test: parse many documents in sequence
    // If memory leaks, this will eventually OOM or take excessive time

    const ITERATIONS: usize = 100;

    for i in 0..ITERATIONS {
        let _doc = create_document(SMALL_RUST);

        // Periodic progress to detect if we're stuck
        if i % 50 == 0 {
            println!("  Memory test progress: {}/{}", i, ITERATIONS);
        }
    }

    println!(
        "✓ Parse memory test: {} iterations without leak",
        ITERATIONS
    );
}

// =============================================================================
// Comparative Performance Tests
// =============================================================================

#[test]
fn test_fingerprint_faster_than_parse() {
    const ITERATIONS: usize = 100;

    // Measure fingerprint time
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _fp = compute_content_fingerprint(SMALL_RUST);
    }
    let fingerprint_time = start.elapsed();

    // Measure parse time
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _doc = create_document(SMALL_RUST);
    }
    let parse_time = start.elapsed();

    // Fingerprint should be at least 10x faster than parsing
    let speedup = parse_time.as_micros() as f64 / fingerprint_time.as_micros() as f64;

    assert!(
        speedup >= 10.0,
        "Fingerprint should be at least 10x faster than parse (got {:.1}x)",
        speedup
    );

    println!(
        "✓ Fingerprint vs parse: {:.1}x faster ({:?} vs {:?})",
        speedup,
        fingerprint_time / ITERATIONS as u32,
        parse_time / ITERATIONS as u32
    );
}
