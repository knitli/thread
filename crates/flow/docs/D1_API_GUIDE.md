# Cloudflare D1 API Integration Guide

**Purpose**: Comprehensive guide for implementing D1 target factory for Thread code analysis storage

**Date**: January 27, 2026
**D1 Version**: Latest (2025-2026)

---

## Overview

Cloudflare D1 is a distributed SQLite database built for edge deployment with global replication. This guide covers the API patterns needed to implement our D1 target factory.

## Two API Approaches

### 1. Workers Binding API (Recommended for Edge)
- **Use Case**: Production edge deployment with Cloudflare Workers
- **Access**: Via environment binding (`env.DB`)
- **Performance**: Optimal latency (edge-local)
- **Rate Limits**: No global API limits (per-Worker limits apply)

### 2. REST API (Administrative/External)
- **Use Case**: External access, bulk operations, admin tasks
- **Access**: HTTP POST to Cloudflare API
- **Performance**: Subject to global API rate limits
- **Limitation**: Best for admin use, not production queries

**Our Choice**: Workers Binding API for production, REST API for bulk imports/testing

---

## Workers Binding API Details

### Accessing the Database

Workers access D1 via environment binding. The binding type is `D1Database` with methods for database interaction.

### Query Methods

#### Method 1: Prepared Statements (Primary Method)

**Characteristics**:
- ✅ Prevents SQL injection via parameter binding
- ✅ Reusable query objects
- ✅ Best performance for repeated queries
- ✅ Type-safe parameter binding

**Result Format**:
```json
{
  "success": true,
  "results": [
    { "file_path": "/path/to/file.rs", "name": "main", "kind": "function" }
  ],
  "meta": {
    "duration": 0.123,
    "rows_read": 1,
    "rows_written": 0
  }
}
```

#### Method 2: Batch Operations (Critical for Performance)

**Characteristics**:
- ✅ **Huge performance impact** - reduces network round trips
- ✅ Atomic transactions - all succeed or all fail
- ✅ Sequential execution (not concurrent)
- ✅ Error reporting per statement
- ❌ Rollback on any failure

**Batch Limits**:
- **Recommended**: 100-500 statements per batch for optimal performance
- **Maximum**: No hard limit, but keep under 1000 for reliability
- **Payload size**: Constrained by Worker request size (10MB)

#### Method 3: Direct Execution (Administrative Use)

**Characteristics**:
- ⚠️ Less secure (no parameter binding)
- ⚠️ Less performant
- ✅ Useful for schema management
- ✅ Supports multi-statement SQL

**Use Cases**: Schema creation, database migration, admin tasks

---

## UPSERT Pattern (Critical for Content-Addressed Updates)

SQLite (D1's underlying database) supports `ON CONFLICT` clause for UPSERT:

### Insert or Update Pattern

```sql
INSERT INTO code_symbols (file_path, name, kind, scope, content_hash)
VALUES (?, ?, ?, ?, ?)
ON CONFLICT(file_path, name)
DO UPDATE SET
  kind = excluded.kind,
  scope = excluded.scope,
  content_hash = excluded.content_hash,
  indexed_at = CURRENT_TIMESTAMP;
```

### Batch UPSERT Pattern

Combine multiple UPSERT operations in batch for optimal performance. Each statement follows the same ON CONFLICT pattern.

---

## Deletion Patterns

### Delete by File (Cascade)

Foreign key cascades handle symbols/imports/calls automatically when deleting from file_metadata.

### Conditional Delete (Content Hash Check)

Delete only if content hash matches expected value, enabling safe concurrent updates.

---

## Transaction Support

D1 batch operations are **atomic transactions**:

**Key Points**:
- Batch operations execute sequentially (not concurrent)
- First failure aborts entire sequence
- Rollback is automatic on error
- No explicit BEGIN/COMMIT needed

---

## Error Handling Patterns

### Statement-Level Errors

Wrap queries in try-catch and check result.success field.

### Batch Error Handling

Filter results for errors and handle batch-level rollback.

### Retry Logic

Implement exponential backoff for transient failures (3-5 retries recommended).

---

## Rate Limits & Performance

### Workers Binding Limits

**CPU Time**:
- Free: 10ms per request
- Paid: 50ms per request

**Memory**:
- 128 MB per Worker

**D1 Query Limits**:
- Free: 100,000 rows read/day
- Paid: 25M rows read/day (first 25M free)

**Batch Recommendations**:
- Optimal: 100-500 statements per batch
- Maximum: Keep under 1000 for reliability
- Monitor: Use result.meta.duration for profiling

### Performance Tips

1. **Use Batch Operations**: 10-50x faster than individual queries
2. **Prepared Statements**: Reuse for repeated queries
3. **Index Strategy**: Create indexes on frequently queried columns
4. **Limit Result Sets**: Use LIMIT clause, avoid SELECT *
5. **Monitor Metrics**: Track rows_read and duration in result.meta

---

## REST API (For External Access)

### Endpoint

```
POST https://api.cloudflare.com/client/v4/accounts/{account_id}/d1/database/{database_id}/query
```

### Authentication

```
Authorization: Bearer {api_token}
Content-Type: application/json
```

### Request Format

```json
{
  "sql": "INSERT INTO code_symbols (file_path, name, kind) VALUES (?, ?, ?)",
  "params": ["src/lib.rs", "main", "function"]
}
```

### Response Format

```json
{
  "result": [
    {
      "results": [],
      "success": true,
      "meta": {
        "served_by": "v3-prod",
        "duration": 0.123,
        "changes": 1,
        "last_row_id": 42,
        "changed_db": true,
        "size_after": 8192,
        "rows_read": 0,
        "rows_written": 1
      }
    }
  ],
  "success": true,
  "errors": [],
  "messages": []
}
```

### REST API Limitations

⚠️ **Known Issues** (as of 2024):
- No batch mode with parameters (SQL injection risk)
- Global API rate limits apply
- Higher latency than Workers binding

**Recommendation**: Use REST API only for:
- Bulk imports during setup
- Administrative tasks
- External integrations

---

## Implementation Checklist for D1 Target Factory

### Required Functionality

- [ ] HTTP client for D1 REST API (for external access)
- [ ] Workers binding support (for edge deployment)
- [ ] Prepared statement creation with parameter binding
- [ ] Batch operation support (100-1000 statements)
- [ ] UPSERT logic using ON CONFLICT
- [ ] DELETE with cascading foreign keys
- [ ] Transaction error handling
- [ ] Retry logic with exponential backoff
- [ ] Content-hash deduplication
- [ ] Query result parsing

### Performance Optimizations

- [ ] Batch operations (target: 500 statements/batch)
- [ ] Prepared statement reuse
- [ ] Connection pooling (if using REST API)
- [ ] Metrics tracking (rows_read, duration)
- [ ] Index utilization validation

### Error Scenarios

- [ ] Network timeout handling
- [ ] SQL constraint violations (primary key, foreign key)
- [ ] Transaction rollback
- [ ] Rate limit exceeded
- [ ] Database full (10 GB limit)

---

## Sources

- [Workers Binding API](https://developers.cloudflare.com/d1/worker-api/d1-database/)
- [Build an API to access D1](https://developers.cloudflare.com/d1/tutorials/build-an-api-to-access-d1/)
- [Bulk import tutorial](https://developers.cloudflare.com/d1/tutorials/import-to-d1-with-rest-api/)
- [D1 Overview](https://developers.cloudflare.com/d1/)
- [Cloudflare API Reference](https://developers.cloudflare.com/api/resources/d1/)
