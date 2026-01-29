-- SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
-- SPDX-License-Identifier: AGPL-3.0-or-later
--
-- Thread Incremental Update System - Postgres Schema v1
--
-- This migration creates the storage tables for the incremental update system.
-- Tables store fingerprints, dependency edges, and source file tracking.
--
-- Compatible with: PostgreSQL 14+
-- Performance target: <10ms p95 for single operations

-- ── Fingerprint Tracking ────────────────────────────────────────────────────

-- Stores content-addressed fingerprints for analyzed files.
-- Uses Blake3 hashing (16 bytes) for change detection.
CREATE TABLE IF NOT EXISTS analysis_fingerprints (
    file_path    TEXT PRIMARY KEY,
    content_fingerprint BYTEA NOT NULL,  -- blake3 hash (16 bytes)
    last_analyzed BIGINT,                -- Unix microseconds, NULL if never persisted
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Source File Tracking ────────────────────────────────────────────────────

-- Tracks which source files contribute to each fingerprinted analysis result.
-- Many-to-many relationship: one fingerprint can have multiple source files,
-- and one source file can contribute to multiple fingerprints.
CREATE TABLE IF NOT EXISTS source_files (
    fingerprint_path TEXT NOT NULL
        REFERENCES analysis_fingerprints(file_path) ON DELETE CASCADE,
    source_path      TEXT NOT NULL,
    PRIMARY KEY (fingerprint_path, source_path)
);

-- ── Dependency Graph Edges ──────────────────────────────────────────────────

-- Stores dependency edges between files in the code graph.
-- Supports both file-level and symbol-level dependency tracking.
CREATE TABLE IF NOT EXISTS dependency_edges (
    id                SERIAL PRIMARY KEY,
    from_path         TEXT NOT NULL,           -- source file (dependent)
    to_path           TEXT NOT NULL,           -- target file (dependency)
    dep_type          TEXT NOT NULL,           -- 'Import', 'Export', 'Macro', 'Type', 'Trait'
    symbol_from       TEXT,                    -- source symbol name (optional)
    symbol_to         TEXT,                    -- target symbol name (optional)
    symbol_kind       TEXT,                    -- 'Function', 'Class', etc. (optional)
    dependency_strength TEXT,                  -- 'Strong' or 'Weak' (optional, from symbol)
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(from_path, to_path, dep_type)      -- prevent duplicate edges
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

-- ── Updated At Trigger ──────────────────────────────────────────────────────

-- Automatically update the updated_at timestamp on fingerprint changes.
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER trigger_fingerprints_updated_at
    BEFORE UPDATE ON analysis_fingerprints
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
