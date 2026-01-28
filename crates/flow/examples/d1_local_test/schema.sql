-- Thread code analysis results table
-- This schema is created manually via Wrangler CLI
-- Run: wrangler d1 execute thread_test --local --file=schema.sql

CREATE TABLE IF NOT EXISTS code_symbols (
    -- Primary key: content hash for deduplication
    content_hash TEXT PRIMARY KEY,

    -- Source file information
    file_path TEXT NOT NULL,
    symbol_name TEXT NOT NULL,
    symbol_type TEXT NOT NULL,  -- function, class, method, variable, etc.

    -- Location in file
    start_line INTEGER NOT NULL,
    end_line INTEGER NOT NULL,
    start_col INTEGER,
    end_col INTEGER,

    -- Symbol content
    source_code TEXT,

    -- Metadata
    language TEXT NOT NULL,
    last_analyzed TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    -- Indexes for common queries
    INDEX idx_file_path ON code_symbols(file_path),
    INDEX idx_symbol_name ON code_symbols(symbol_name),
    INDEX idx_symbol_type ON code_symbols(symbol_type)
);

-- Example query to verify data
-- SELECT file_path, symbol_name, symbol_type, start_line
-- FROM code_symbols
-- ORDER BY file_path, start_line;
