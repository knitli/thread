// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileCopyrightText: 2026 Knitli Inc.
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Integration tests for the D1 incremental backend.
//!
//! Since D1 is accessed via HTTP REST API, these tests use `rusqlite` (in-memory
//! SQLite) to validate the SQL schema, query correctness, and data integrity.
//! The SQL statements match those used by the D1 backend exactly.
//!
//! This approach ensures:
//! - Schema migration SQL is valid SQLite
//! - All queries execute correctly against SQLite
//! - Upsert/conflict handling works as expected
//! - BLOB/INTEGER type conversions are correct
//! - Performance characteristics are validated locally

use recoco::utils::fingerprint::{Fingerprint, Fingerprinter};
use rusqlite::{Connection, params};
use std::time::Instant;

/// Creates an in-memory SQLite database with the D1 schema applied.
fn setup_db() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to open in-memory SQLite");
    // Enable foreign keys (required for CASCADE behavior).
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .expect("Failed to set PRAGMA");

    // Strip SQL comments and execute the full migration.
    // rusqlite's execute_batch handles multi-statement SQL with semicolons.
    let migration_sql = include_str!("../migrations/d1_incremental_v1.sql");
    let cleaned = strip_sql_comments(migration_sql);
    conn.execute_batch(&cleaned)
        .unwrap_or_else(|e| panic!("Migration failed: {e}\nSQL:\n{cleaned}"));

    conn
}

/// Strips SQL line comments (-- ...) from a SQL string.
/// Preserves the rest of the SQL including semicolons.
fn strip_sql_comments(sql: &str) -> String {
    sql.lines()
        .map(|line| {
            // Remove everything after `--` (line comment)
            if let Some(pos) = line.find("--") {
                &line[..pos]
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Creates a test fingerprint from content bytes.
fn make_fingerprint(content: &[u8]) -> Vec<u8> {
    let mut fp = Fingerprinter::default();
    fp.write_raw_bytes(content);
    let fingerprint = fp.into_fingerprint();
    fingerprint.as_slice().to_vec()
}

// ═══════════════════════════════════════════════════════════════════════════
// Schema Migration Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_d1_migration_creates_all_tables() {
    let conn = setup_db();

    // Verify all three tables exist.
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(
        tables.contains(&"analysis_fingerprints".to_string()),
        "Missing analysis_fingerprints table"
    );
    assert!(
        tables.contains(&"dependency_edges".to_string()),
        "Missing dependency_edges table"
    );
    assert!(
        tables.contains(&"source_files".to_string()),
        "Missing source_files table"
    );
}

#[test]
fn test_d1_migration_creates_indexes() {
    let conn = setup_db();

    let indexes: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(indexes.contains(&"idx_edges_from".to_string()));
    assert!(indexes.contains(&"idx_edges_to".to_string()));
    assert!(indexes.contains(&"idx_source_files_fp".to_string()));
    assert!(indexes.contains(&"idx_source_files_src".to_string()));
}

#[test]
fn test_d1_migration_is_idempotent() {
    let conn = setup_db();

    // Run migrations again - should not fail (IF NOT EXISTS).
    let migration_sql = include_str!("../migrations/d1_incremental_v1.sql");
    let cleaned = strip_sql_comments(migration_sql);
    conn.execute_batch(&cleaned)
        .unwrap_or_else(|e| panic!("Re-migration failed: {e}"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Fingerprint CRUD Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_d1_save_and_load_fingerprint() {
    let conn = setup_db();
    let fp_bytes = make_fingerprint(b"fn main() {}");

    // Insert fingerprint.
    conn.execute(
        "INSERT INTO analysis_fingerprints (file_path, content_fingerprint, last_analyzed) \
         VALUES (?1, ?2, ?3)",
        params!["src/main.rs", fp_bytes, 1706400000_000_000i64],
    )
    .unwrap();

    // Load it back.
    let (loaded_fp, loaded_ts): (Vec<u8>, Option<i64>) = conn
        .query_row(
            "SELECT content_fingerprint, last_analyzed FROM analysis_fingerprints WHERE file_path = ?1",
            params!["src/main.rs"],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();

    assert_eq!(loaded_fp, fp_bytes);
    assert_eq!(loaded_ts, Some(1706400000_000_000i64));
}

#[test]
fn test_d1_fingerprint_upsert() {
    let conn = setup_db();
    let fp_v1 = make_fingerprint(b"version 1");
    let fp_v2 = make_fingerprint(b"version 2");

    // Insert v1.
    conn.execute(
        "INSERT INTO analysis_fingerprints (file_path, content_fingerprint, last_analyzed, updated_at) \
         VALUES (?1, ?2, ?3, strftime('%s', 'now')) \
         ON CONFLICT (file_path) DO UPDATE SET \
             content_fingerprint = excluded.content_fingerprint, \
             last_analyzed = excluded.last_analyzed, \
             updated_at = strftime('%s', 'now')",
        params!["file.rs", fp_v1, 100i64],
    )
    .unwrap();

    // Upsert v2 on the same path.
    conn.execute(
        "INSERT INTO analysis_fingerprints (file_path, content_fingerprint, last_analyzed, updated_at) \
         VALUES (?1, ?2, ?3, strftime('%s', 'now')) \
         ON CONFLICT (file_path) DO UPDATE SET \
             content_fingerprint = excluded.content_fingerprint, \
             last_analyzed = excluded.last_analyzed, \
             updated_at = strftime('%s', 'now')",
        params!["file.rs", fp_v2, 200i64],
    )
    .unwrap();

    // Verify v2 is stored (only 1 row).
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM analysis_fingerprints", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(count, 1);

    let loaded_fp: Vec<u8> = conn
        .query_row(
            "SELECT content_fingerprint FROM analysis_fingerprints WHERE file_path = ?1",
            params!["file.rs"],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(loaded_fp, fp_v2);
}

#[test]
fn test_d1_fingerprint_delete() {
    let conn = setup_db();
    let fp = make_fingerprint(b"content");

    conn.execute(
        "INSERT INTO analysis_fingerprints (file_path, content_fingerprint) VALUES (?1, ?2)",
        params!["to_delete.rs", fp],
    )
    .unwrap();

    let changes = conn
        .execute(
            "DELETE FROM analysis_fingerprints WHERE file_path = ?1",
            params!["to_delete.rs"],
        )
        .unwrap();

    assert_eq!(changes, 1);

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM analysis_fingerprints WHERE file_path = ?1",
            params!["to_delete.rs"],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_d1_fingerprint_load_nonexistent() {
    let conn = setup_db();

    let result = conn.query_row(
        "SELECT content_fingerprint FROM analysis_fingerprints WHERE file_path = ?1",
        params!["nonexistent.rs"],
        |row| row.get::<_, Vec<u8>>(0),
    );

    assert!(result.is_err()); // QueryReturnedNoRows
}

// ═══════════════════════════════════════════════════════════════════════════
// Source File Tracking Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_d1_source_files_tracking() {
    let conn = setup_db();
    let fp = make_fingerprint(b"analysis result");

    // Insert fingerprint.
    conn.execute(
        "INSERT INTO analysis_fingerprints (file_path, content_fingerprint) VALUES (?1, ?2)",
        params!["main.rs", fp],
    )
    .unwrap();

    // Add source files.
    conn.execute(
        "INSERT INTO source_files (fingerprint_path, source_path) VALUES (?1, ?2)",
        params!["main.rs", "utils.rs"],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO source_files (fingerprint_path, source_path) VALUES (?1, ?2)",
        params!["main.rs", "config.rs"],
    )
    .unwrap();

    // Load source files.
    let sources: Vec<String> = conn
        .prepare("SELECT source_path FROM source_files WHERE fingerprint_path = ?1")
        .unwrap()
        .query_map(params!["main.rs"], |row| row.get(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert_eq!(sources.len(), 2);
    assert!(sources.contains(&"utils.rs".to_string()));
    assert!(sources.contains(&"config.rs".to_string()));
}

#[test]
fn test_d1_source_files_cascade_delete() {
    let conn = setup_db();
    let fp = make_fingerprint(b"content");

    conn.execute(
        "INSERT INTO analysis_fingerprints (file_path, content_fingerprint) VALUES (?1, ?2)",
        params!["main.rs", fp],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO source_files (fingerprint_path, source_path) VALUES (?1, ?2)",
        params!["main.rs", "dep.rs"],
    )
    .unwrap();

    // Delete the fingerprint - should cascade to source_files.
    conn.execute(
        "DELETE FROM analysis_fingerprints WHERE file_path = ?1",
        params!["main.rs"],
    )
    .unwrap();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM source_files WHERE fingerprint_path = ?1",
            params!["main.rs"],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        count, 0,
        "CASCADE delete should remove source_files entries"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Dependency Edge CRUD Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_d1_save_and_load_edge() {
    let conn = setup_db();

    conn.execute(
        "INSERT INTO dependency_edges \
         (from_path, to_path, dep_type, symbol_from, symbol_to, symbol_kind, dependency_strength) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            "main.rs",
            "utils.rs",
            "import",
            None::<String>,
            None::<String>,
            None::<String>,
            None::<String>
        ],
    )
    .unwrap();

    let rows: Vec<(String, String, String)> = conn
        .prepare("SELECT from_path, to_path, dep_type FROM dependency_edges WHERE from_path = ?1")
        .unwrap()
        .query_map(params!["main.rs"], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0],
        (
            "main.rs".to_string(),
            "utils.rs".to_string(),
            "import".to_string()
        )
    );
}

#[test]
fn test_d1_edge_upsert_on_conflict() {
    let conn = setup_db();

    // Insert edge without symbol.
    conn.execute(
        "INSERT INTO dependency_edges \
         (from_path, to_path, dep_type, symbol_from, symbol_to, symbol_kind, dependency_strength) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) \
         ON CONFLICT (from_path, to_path, dep_type) DO UPDATE SET \
             symbol_from = excluded.symbol_from, \
             symbol_to = excluded.symbol_to, \
             symbol_kind = excluded.symbol_kind, \
             dependency_strength = excluded.dependency_strength",
        params![
            "a.rs",
            "b.rs",
            "import",
            None::<String>,
            None::<String>,
            None::<String>,
            None::<String>
        ],
    )
    .unwrap();

    // Upsert same edge with symbol info.
    conn.execute(
        "INSERT INTO dependency_edges \
         (from_path, to_path, dep_type, symbol_from, symbol_to, symbol_kind, dependency_strength) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) \
         ON CONFLICT (from_path, to_path, dep_type) DO UPDATE SET \
             symbol_from = excluded.symbol_from, \
             symbol_to = excluded.symbol_to, \
             symbol_kind = excluded.symbol_kind, \
             dependency_strength = excluded.dependency_strength",
        params![
            "a.rs", "b.rs", "import", "main", "helper", "function", "strong"
        ],
    )
    .unwrap();

    // Should be 1 row with updated symbol info.
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM dependency_edges", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(count, 1);

    let sym_from: Option<String> = conn
        .query_row(
            "SELECT symbol_from FROM dependency_edges WHERE from_path = ?1",
            params!["a.rs"],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(sym_from, Some("main".to_string()));
}

#[test]
fn test_d1_edge_with_symbol_data() {
    let conn = setup_db();

    conn.execute(
        "INSERT INTO dependency_edges \
         (from_path, to_path, dep_type, symbol_from, symbol_to, symbol_kind, dependency_strength) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            "api.rs",
            "router.rs",
            "import",
            "handler",
            "Router",
            "class",
            "strong"
        ],
    )
    .unwrap();

    let (sym_from, sym_to, sym_kind, strength): (String, String, String, String) = conn
        .query_row(
            "SELECT symbol_from, symbol_to, symbol_kind, dependency_strength \
             FROM dependency_edges WHERE from_path = ?1",
            params!["api.rs"],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap();

    assert_eq!(sym_from, "handler");
    assert_eq!(sym_to, "Router");
    assert_eq!(sym_kind, "class");
    assert_eq!(strength, "strong");
}

#[test]
fn test_d1_load_edges_to() {
    let conn = setup_db();

    // Two files depend on utils.rs.
    conn.execute(
        "INSERT INTO dependency_edges (from_path, to_path, dep_type) VALUES (?1, ?2, ?3)",
        params!["main.rs", "utils.rs", "import"],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO dependency_edges (from_path, to_path, dep_type) VALUES (?1, ?2, ?3)",
        params!["lib.rs", "utils.rs", "import"],
    )
    .unwrap();

    let dependents: Vec<String> = conn
        .prepare("SELECT from_path FROM dependency_edges WHERE to_path = ?1")
        .unwrap()
        .query_map(params!["utils.rs"], |row| row.get(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert_eq!(dependents.len(), 2);
    assert!(dependents.contains(&"main.rs".to_string()));
    assert!(dependents.contains(&"lib.rs".to_string()));
}

#[test]
fn test_d1_delete_edges_for_file() {
    let conn = setup_db();

    // a.rs -> b.rs, c.rs -> a.rs, d.rs -> e.rs
    conn.execute(
        "INSERT INTO dependency_edges (from_path, to_path, dep_type) VALUES (?1, ?2, ?3)",
        params!["a.rs", "b.rs", "import"],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO dependency_edges (from_path, to_path, dep_type) VALUES (?1, ?2, ?3)",
        params!["c.rs", "a.rs", "import"],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO dependency_edges (from_path, to_path, dep_type) VALUES (?1, ?2, ?3)",
        params!["d.rs", "e.rs", "import"],
    )
    .unwrap();

    let changes = conn
        .execute(
            "DELETE FROM dependency_edges WHERE from_path = ?1 OR to_path = ?1",
            params!["a.rs"],
        )
        .unwrap();

    assert_eq!(changes, 2); // Both edges involving a.rs

    let remaining: i64 = conn
        .query_row("SELECT COUNT(*) FROM dependency_edges", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(remaining, 1); // Only d.rs -> e.rs remains
}

// ═══════════════════════════════════════════════════════════════════════════
// Full Graph Save/Load Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_d1_full_graph_roundtrip() {
    let conn = setup_db();
    let fp_a = make_fingerprint(b"file a");
    let fp_b = make_fingerprint(b"file b");
    let fp_c = make_fingerprint(b"file c");

    // Save fingerprints.
    for (path, fp) in [("a.rs", &fp_a), ("b.rs", &fp_b), ("c.rs", &fp_c)] {
        conn.execute(
            "INSERT INTO analysis_fingerprints (file_path, content_fingerprint, last_analyzed) \
             VALUES (?1, ?2, ?3)",
            params![path, fp, 1000i64],
        )
        .unwrap();
    }

    // Save source files.
    conn.execute(
        "INSERT INTO source_files (fingerprint_path, source_path) VALUES (?1, ?2)",
        params!["a.rs", "dep1.rs"],
    )
    .unwrap();

    // Save edges.
    conn.execute(
        "INSERT INTO dependency_edges (from_path, to_path, dep_type) VALUES (?1, ?2, ?3)",
        params!["a.rs", "b.rs", "import"],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO dependency_edges (from_path, to_path, dep_type) VALUES (?1, ?2, ?3)",
        params!["b.rs", "c.rs", "import"],
    )
    .unwrap();

    // Load and verify.
    let fp_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM analysis_fingerprints", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(fp_count, 3);

    let edge_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM dependency_edges", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(edge_count, 2);

    let src_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM source_files WHERE fingerprint_path = ?1",
            params!["a.rs"],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(src_count, 1);
}

#[test]
fn test_d1_full_graph_clear_and_replace() {
    let conn = setup_db();
    let fp = make_fingerprint(b"old data");

    // Insert initial data.
    conn.execute(
        "INSERT INTO analysis_fingerprints (file_path, content_fingerprint) VALUES (?1, ?2)",
        params!["old.rs", fp],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO dependency_edges (from_path, to_path, dep_type) VALUES (?1, ?2, ?3)",
        params!["old.rs", "dep.rs", "import"],
    )
    .unwrap();

    // Clear all data (D1 uses DELETE, not TRUNCATE).
    conn.execute_batch(
        "DELETE FROM source_files; \
         DELETE FROM dependency_edges; \
         DELETE FROM analysis_fingerprints;",
    )
    .unwrap();

    let fp_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM analysis_fingerprints", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(fp_count, 0);

    let edge_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM dependency_edges", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(edge_count, 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// BLOB/INTEGER Conversion Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_d1_blob_fingerprint_roundtrip() {
    let conn = setup_db();
    let fp_bytes = make_fingerprint(b"test content for blob roundtrip");

    // Insert as BLOB.
    conn.execute(
        "INSERT INTO analysis_fingerprints (file_path, content_fingerprint) VALUES (?1, ?2)",
        params!["blob_test.rs", fp_bytes],
    )
    .unwrap();

    // Read back as BLOB.
    let loaded: Vec<u8> = conn
        .query_row(
            "SELECT content_fingerprint FROM analysis_fingerprints WHERE file_path = ?1",
            params!["blob_test.rs"],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(loaded.len(), 16, "Fingerprint must be 16 bytes");
    assert_eq!(loaded, fp_bytes);

    // Verify it can be converted back to a Fingerprint.
    let arr: [u8; 16] = loaded.try_into().unwrap();
    let restored = Fingerprint(arr);
    assert_eq!(restored.as_slice(), &fp_bytes[..]);
}

#[test]
fn test_d1_integer_timestamp_handling() {
    let conn = setup_db();
    let fp = make_fingerprint(b"timestamp test");

    // Test with large Unix microsecond timestamp.
    let timestamp: i64 = 1706400000_000_000; // 2024-01-28 in microseconds

    conn.execute(
        "INSERT INTO analysis_fingerprints (file_path, content_fingerprint, last_analyzed) \
         VALUES (?1, ?2, ?3)",
        params!["ts_test.rs", fp, timestamp],
    )
    .unwrap();

    let loaded: i64 = conn
        .query_row(
            "SELECT last_analyzed FROM analysis_fingerprints WHERE file_path = ?1",
            params!["ts_test.rs"],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(loaded, timestamp);
}

#[test]
fn test_d1_null_timestamp() {
    let conn = setup_db();
    let fp = make_fingerprint(b"null ts");

    conn.execute(
        "INSERT INTO analysis_fingerprints (file_path, content_fingerprint, last_analyzed) \
         VALUES (?1, ?2, NULL)",
        params!["null_ts.rs", fp],
    )
    .unwrap();

    let loaded: Option<i64> = conn
        .query_row(
            "SELECT last_analyzed FROM analysis_fingerprints WHERE file_path = ?1",
            params!["null_ts.rs"],
            |row| row.get(0),
        )
        .unwrap();

    assert!(loaded.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// Performance Validation Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_d1_performance_single_fingerprint_op() {
    let conn = setup_db();

    // Insert 100 fingerprints.
    let start = Instant::now();
    for i in 0..100 {
        let fp = make_fingerprint(format!("content {i}").as_bytes());
        conn.execute(
            "INSERT INTO analysis_fingerprints (file_path, content_fingerprint, last_analyzed) \
             VALUES (?1, ?2, ?3)",
            params![format!("file_{i}.rs"), fp, i as i64],
        )
        .unwrap();
    }
    let insert_duration = start.elapsed();

    // Lookup performance.
    let start = Instant::now();
    for i in 0..100 {
        let _: Vec<u8> = conn
            .query_row(
                "SELECT content_fingerprint FROM analysis_fingerprints WHERE file_path = ?1",
                params![format!("file_{i}.rs")],
                |row| row.get(0),
            )
            .unwrap();
    }
    let lookup_duration = start.elapsed();

    // SQLite in-memory should be much faster than the 50ms D1 target.
    // This validates the query structure is efficient.
    assert!(
        insert_duration.as_millis() < 500,
        "100 inserts took {}ms (should be <500ms even on SQLite)",
        insert_duration.as_millis()
    );
    assert!(
        lookup_duration.as_millis() < 100,
        "100 lookups took {}ms (should be <100ms on SQLite)",
        lookup_duration.as_millis()
    );
}

#[test]
fn test_d1_performance_edge_traversal() {
    let conn = setup_db();

    // Create a graph with 100 edges.
    for i in 0..100 {
        conn.execute(
            "INSERT INTO dependency_edges (from_path, to_path, dep_type) VALUES (?1, ?2, ?3)",
            params![
                format!("file_{i}.rs"),
                format!("dep_{}.rs", i % 10),
                "import"
            ],
        )
        .unwrap();
    }

    // Measure forward traversal.
    let start = Instant::now();
    for i in 0..100 {
        let _: Vec<String> = conn
            .prepare("SELECT to_path FROM dependency_edges WHERE from_path = ?1")
            .unwrap()
            .query_map(params![format!("file_{i}.rs")], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
    }
    let forward_duration = start.elapsed();

    // Measure reverse traversal (dependents lookup).
    let start = Instant::now();
    for i in 0..10 {
        let _: Vec<String> = conn
            .prepare("SELECT from_path FROM dependency_edges WHERE to_path = ?1")
            .unwrap()
            .query_map(params![format!("dep_{i}.rs")], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
    }
    let reverse_duration = start.elapsed();

    assert!(
        forward_duration.as_millis() < 200,
        "100 forward lookups took {}ms",
        forward_duration.as_millis()
    );
    assert!(
        reverse_duration.as_millis() < 50,
        "10 reverse lookups took {}ms",
        reverse_duration.as_millis()
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Batch Operation Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_d1_batch_edge_insertion() {
    let conn = setup_db();

    // Simulate batch insertion (D1 sends multiple individual statements).
    let edges = vec![
        ("a.rs", "b.rs", "import"),
        ("a.rs", "c.rs", "import"),
        ("b.rs", "c.rs", "trait"),
        ("c.rs", "d.rs", "type"),
    ];

    for (from, to, dep_type) in &edges {
        conn.execute(
            "INSERT INTO dependency_edges \
             (from_path, to_path, dep_type) \
             VALUES (?1, ?2, ?3) \
             ON CONFLICT (from_path, to_path, dep_type) DO NOTHING",
            params![from, to, dep_type],
        )
        .unwrap();
    }

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM dependency_edges", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(count, 4);
}

#[test]
fn test_d1_unique_constraint_prevents_duplicate_edges() {
    let conn = setup_db();

    conn.execute(
        "INSERT INTO dependency_edges (from_path, to_path, dep_type) VALUES (?1, ?2, ?3)",
        params!["a.rs", "b.rs", "import"],
    )
    .unwrap();

    // Same edge should fail (UNIQUE constraint).
    let result = conn.execute(
        "INSERT INTO dependency_edges (from_path, to_path, dep_type) VALUES (?1, ?2, ?3)",
        params!["a.rs", "b.rs", "import"],
    );
    assert!(
        result.is_err(),
        "Duplicate edge should violate UNIQUE constraint"
    );

    // But same files with different dep_type should succeed.
    conn.execute(
        "INSERT INTO dependency_edges (from_path, to_path, dep_type) VALUES (?1, ?2, ?3)",
        params!["a.rs", "b.rs", "type"],
    )
    .unwrap();

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM dependency_edges", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(count, 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// Edge Case Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_d1_empty_fingerprint_content() {
    let conn = setup_db();
    let fp = make_fingerprint(b"");

    conn.execute(
        "INSERT INTO analysis_fingerprints (file_path, content_fingerprint) VALUES (?1, ?2)",
        params!["empty.rs", fp],
    )
    .unwrap();

    let loaded: Vec<u8> = conn
        .query_row(
            "SELECT content_fingerprint FROM analysis_fingerprints WHERE file_path = ?1",
            params!["empty.rs"],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(loaded.len(), 16);
    assert_eq!(loaded, fp);
}

#[test]
fn test_d1_path_with_special_characters() {
    let conn = setup_db();
    let fp = make_fingerprint(b"special path content");

    // Paths with spaces, dots, and non-ASCII.
    let paths = [
        "src/my module/file.rs",
        "src/../lib.rs",
        "src/unicode\u{00e9}.rs",
    ];

    for path in &paths {
        conn.execute(
            "INSERT INTO analysis_fingerprints (file_path, content_fingerprint) VALUES (?1, ?2)",
            params![path, fp],
        )
        .unwrap();
    }

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM analysis_fingerprints", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(count, 3);
}
