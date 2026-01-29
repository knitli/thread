-- SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
-- SPDX-License-Identifier: AGPL-3.0-or-later
--
-- Thread Incremental Update System - D1 (SQLite) Schema v1
--
-- This migration creates the storage tables for the incremental update system
-- on Cloudflare D1 (SQLite dialect). Mirrors the Postgres schema with
-- SQLite-compatible types and syntax.
--
-- Compatible with: SQLite 3.x / Cloudflare D1
-- Performance target: <50ms p95 for single operations (Constitutional Principle VI)
--
-- Key differences from Postgres schema:
--   - INTEGER instead of BIGINT/SERIAL
--   - BLOB instead of BYTEA
--   - strftime('%s','now') instead of NOW()/TIMESTAMPTZ
--   - No triggers or stored functions (SQLite limitation)
--   - INTEGER PRIMARY KEY AUTOINCREMENT instead of SERIAL

-- ── Fingerprint Tracking ────────────────────────────────────────────────────

-- Stores content-addressed fingerprints for analyzed files.
-- Uses Blake3 hashing (16 bytes) for change detection.
CREATE TABLE IF NOT EXISTS analysis_fingerprints (
    file_path           TEXT PRIMARY KEY,
    content_fingerprint BLOB NOT NULL,      -- blake3 hash (16 bytes)
    last_analyzed       INTEGER,            -- Unix timestamp in microseconds, NULL if never persisted
    created_at          INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at          INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- ── Source File Tracking ────────────────────────────────────────────────────

-- Tracks which source files contribute to each fingerprinted analysis result.
-- Many-to-many: one fingerprint can have multiple source files,
-- and one source file can contribute to multiple fingerprints.
CREATE TABLE IF NOT EXISTS source_files (
    fingerprint_path TEXT NOT NULL,
    source_path      TEXT NOT NULL,
    PRIMARY KEY (fingerprint_path, source_path),
    FOREIGN KEY (fingerprint_path) REFERENCES analysis_fingerprints(file_path) ON DELETE CASCADE
);

-- ── Dependency Graph Edges ──────────────────────────────────────────────────

-- Stores dependency edges between files in the code graph.
-- Supports both file-level and symbol-level dependency tracking.
CREATE TABLE IF NOT EXISTS dependency_edges (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    from_path           TEXT NOT NULL,          -- source file (dependent)
    to_path             TEXT NOT NULL,          -- target file (dependency)
    dep_type            TEXT NOT NULL,          -- 'import', 'export', 'macro', 'type', 'trait'
    symbol_from         TEXT,                   -- source symbol name (optional)
    symbol_to           TEXT,                   -- target symbol name (optional)
    symbol_kind         TEXT,                   -- 'function', 'class', etc. (optional)
    dependency_strength TEXT,                   -- 'strong' or 'weak' (optional, from symbol)
    created_at          INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    UNIQUE(from_path, to_path, dep_type)       -- prevent duplicate edges
);

-- ── Performance Indexes ─────────────────────────────────────────────────────

-- Index for querying edges originating from a file (forward traversal).
CREATE INDEX IF NOT EXISTS idx_edges_from ON dependency_edges(from_path);

-- Index for querying edges targeting a file (reverse traversal / dependents).
CREATE INDEX IF NOT EXISTS idx_edges_to ON dependency_edges(to_path);

-- Index for joining source_files back to fingerprints.
CREATE INDEX IF NOT EXISTS idx_source_files_fp ON source_files(fingerprint_path);

-- Index for querying source files by source path (reverse lookup).
CREATE INDEX IF NOT EXISTS idx_source_files_src ON source_files(source_path);
