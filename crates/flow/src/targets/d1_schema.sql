-- SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
-- SPDX-License-Identifier: AGPL-3.0-or-later

-- D1 Database Schema for Thread Code Analysis
-- SQLite schema for Cloudflare D1 distributed edge database

-- ============================================================================
-- FILE METADATA TABLE
-- ============================================================================
-- Tracks analyzed files with content hashing for incremental updates

CREATE TABLE IF NOT EXISTS file_metadata (
    -- Primary identifier
    file_path TEXT PRIMARY KEY,

    -- Content addressing for incremental updates
    content_hash TEXT NOT NULL,

    -- Language detection
    language TEXT NOT NULL,

    -- Analysis tracking
    last_analyzed DATETIME DEFAULT CURRENT_TIMESTAMP,
    analysis_version INTEGER DEFAULT 1,

    -- File statistics
    line_count INTEGER,
    char_count INTEGER
);

-- Index for content-addressed lookups
CREATE INDEX IF NOT EXISTS idx_metadata_hash
    ON file_metadata(content_hash);

-- Index for language-based queries
CREATE INDEX IF NOT EXISTS idx_metadata_language
    ON file_metadata(language);

-- ============================================================================
-- CODE SYMBOLS TABLE
-- ============================================================================
-- Stores extracted symbols: functions, classes, variables, etc.

CREATE TABLE IF NOT EXISTS code_symbols (
    -- Composite primary key (file + symbol name)
    file_path TEXT NOT NULL,
    name TEXT NOT NULL,

    -- Symbol classification
    kind TEXT NOT NULL,          -- function, class, variable, constant, etc.
    scope TEXT,                   -- namespace/module/class scope

    -- Location information
    line_start INTEGER,
    line_end INTEGER,

    -- Content addressing
    content_hash TEXT NOT NULL,   -- For detecting symbol changes

    -- Metadata
    indexed_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    -- Primary key prevents duplicate symbols per file
    PRIMARY KEY (file_path, name),

    -- Foreign key to file metadata
    FOREIGN KEY (file_path) REFERENCES file_metadata(file_path)
        ON DELETE CASCADE
);

-- Indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_symbols_kind
    ON code_symbols(kind);

CREATE INDEX IF NOT EXISTS idx_symbols_name
    ON code_symbols(name);

CREATE INDEX IF NOT EXISTS idx_symbols_scope
    ON code_symbols(scope);

CREATE INDEX IF NOT EXISTS idx_symbols_file
    ON code_symbols(file_path);

-- ============================================================================
-- CODE IMPORTS TABLE
-- ============================================================================
-- Tracks import statements for dependency analysis

CREATE TABLE IF NOT EXISTS code_imports (
    -- Composite primary key (file + symbol + source)
    file_path TEXT NOT NULL,
    symbol_name TEXT NOT NULL,
    source_path TEXT NOT NULL,

    -- Import classification
    kind TEXT,                    -- named, default, namespace, wildcard

    -- Content addressing
    content_hash TEXT NOT NULL,

    -- Metadata
    indexed_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    -- Primary key prevents duplicate imports
    PRIMARY KEY (file_path, symbol_name, source_path),

    -- Foreign key to file metadata
    FOREIGN KEY (file_path) REFERENCES file_metadata(file_path)
        ON DELETE CASCADE
);

-- Indexes for dependency graph queries
CREATE INDEX IF NOT EXISTS idx_imports_source
    ON code_imports(source_path);

CREATE INDEX IF NOT EXISTS idx_imports_symbol
    ON code_imports(symbol_name);

CREATE INDEX IF NOT EXISTS idx_imports_file
    ON code_imports(file_path);

-- ============================================================================
-- FUNCTION CALLS TABLE
-- ============================================================================
-- Tracks function calls for call graph analysis

CREATE TABLE IF NOT EXISTS code_calls (
    -- Composite primary key (file + function + line)
    file_path TEXT NOT NULL,
    function_name TEXT NOT NULL,
    line_number INTEGER NOT NULL,

    -- Call details
    arguments_count INTEGER,

    -- Content addressing
    content_hash TEXT NOT NULL,

    -- Metadata
    indexed_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    -- Primary key prevents duplicate calls at same location
    PRIMARY KEY (file_path, function_name, line_number),

    -- Foreign key to file metadata
    FOREIGN KEY (file_path) REFERENCES file_metadata(file_path)
        ON DELETE CASCADE
);

-- Indexes for call graph queries
CREATE INDEX IF NOT EXISTS idx_calls_function
    ON code_calls(function_name);

CREATE INDEX IF NOT EXISTS idx_calls_file
    ON code_calls(file_path);

-- ============================================================================
-- ANALYSIS STATISTICS TABLE (Optional)
-- ============================================================================
-- Tracks analysis runs for monitoring and debugging

CREATE TABLE IF NOT EXISTS analysis_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- Execution metrics
    started_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME,
    duration_ms INTEGER,

    -- Analysis scope
    files_analyzed INTEGER DEFAULT 0,
    symbols_extracted INTEGER DEFAULT 0,
    imports_extracted INTEGER DEFAULT 0,
    calls_extracted INTEGER DEFAULT 0,

    -- Cache effectiveness
    cache_hits INTEGER DEFAULT 0,
    cache_misses INTEGER DEFAULT 0,

    -- Error tracking
    errors_count INTEGER DEFAULT 0,
    error_summary TEXT
);

-- ============================================================================
-- VIEWS FOR COMMON QUERIES
-- ============================================================================

-- View: All symbols with file metadata
CREATE VIEW IF NOT EXISTS v_symbols_with_files AS
SELECT
    s.file_path,
    s.name,
    s.kind,
    s.scope,
    s.line_start,
    s.line_end,
    f.language,
    f.content_hash AS file_hash,
    s.content_hash AS symbol_hash
FROM code_symbols s
JOIN file_metadata f ON s.file_path = f.file_path;

-- View: Import dependency graph
CREATE VIEW IF NOT EXISTS v_import_graph AS
SELECT
    i.file_path AS importer,
    i.source_path AS imported,
    i.symbol_name,
    i.kind,
    f.language
FROM code_imports i
JOIN file_metadata f ON i.file_path = f.file_path;

-- View: Function call graph
CREATE VIEW IF NOT EXISTS v_call_graph AS
SELECT
    c.file_path AS caller_file,
    c.function_name AS called_function,
    c.line_number,
    c.arguments_count,
    f.language
FROM code_calls c
JOIN file_metadata f ON c.file_path = f.file_path;

-- ============================================================================
-- NOTES ON D1 USAGE
-- ============================================================================

-- Content-Addressed Updates:
--   1. Hash file content before analysis
--   2. Check file_metadata.content_hash
--   3. Skip analysis if hash unchanged
--   4. On change: DELETE old symbols/imports/calls (cascades), INSERT new

-- UPSERT Pattern (SQLite ON CONFLICT):
--   INSERT INTO code_symbols (file_path, name, kind, ...)
--   VALUES (?, ?, ?, ...)
--   ON CONFLICT(file_path, name)
--   DO UPDATE SET kind = excluded.kind, ...

-- Batch Operations:
--   D1 supports multiple statements in single request
--   Limit: ~1000 rows per batch for performance

-- Query Limits:
--   D1 free tier: 100,000 rows read/day
--   Design queries to be selective (use indexes!)

-- Storage Limits:
--   D1 free tier: 10 GB per database
--   Monitor growth with analysis_stats table
