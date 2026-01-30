<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# D1 Integration API Reference

**Version**: 1.0.0
**Last Updated**: 2025-01-28
**Status**: Production Ready

---

## Table of Contents

1. [Overview](#overview)
2. [Core Types](#core-types)
3. [Setup State Management](#setup-state-management)
4. [Query Building](#query-building)
5. [Type Conversions](#type-conversions)
6. [Configuration](#configuration)
7. [Error Handling](#error-handling)
8. [Usage Examples](#usage-examples)
9. [Best Practices](#best-practices)

---

## Overview

The **D1 Integration** enables Thread Flow to export code analysis results to **Cloudflare D1**, a distributed SQLite database running at the edge. This integration provides:

- ✅ **Content-Addressed Storage**: Automatic deduplication via content hashing
- ✅ **Schema Management**: Automatic table creation and migration
- ✅ **Type System Integration**: Seamless conversion between ReCoco and D1 types
- ✅ **UPSERT Operations**: Efficient incremental updates
- ✅ **Edge-Native**: <50ms p95 latency worldwide

### Quick Start

```rust
use thread_flow::ThreadFlowBuilder;

let flow = ThreadFlowBuilder::new("my_analysis")
    .source_local("src/", &["**/*.rs"], &[])
    .parse()
    .extract_symbols()
    .target_d1(
        "your-cloudflare-account-id",
        "your-d1-database-id",
        "your-api-token",
        "code_symbols",             // table name
        &["content_hash"],          // primary key for deduplication
    )
    .build()
    .await?;
```

---

## Core Types

### D1Spec

Connection specification for D1 database.

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct D1Spec {
    /// Cloudflare account ID
    pub account_id: String,

    /// D1 database ID
    pub database_id: String,

    /// API token for authentication
    pub api_token: String,

    /// Optional table name override
    pub table_name: Option<String>,
}
```

**Usage:**
```rust
let spec = D1Spec {
    account_id: env::var("CLOUDFLARE_ACCOUNT_ID")?,
    database_id: env::var("D1_DATABASE_ID")?,
    api_token: env::var("CLOUDFLARE_API_TOKEN")?,
    table_name: Some("my_table".to_string()),
};
```

### D1TableId

Unique identifier for a D1 table (used as SetupKey).

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct D1TableId {
    pub database_id: String,
    pub table_name: String,
}
```

**Usage:**
```rust
let table_id = D1TableId {
    database_id: "my-database-id".to_string(),
    table_name: "code_symbols".to_string(),
};
```

### D1SetupState

Represents the current schema state of a D1 table.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D1SetupState {
    pub table_id: D1TableId,
    pub key_columns: Vec<ColumnSchema>,
    pub value_columns: Vec<ColumnSchema>,
    pub indexes: Vec<IndexSchema>,
}
```

**Fields:**
- `table_id`: Identifies the table (database + table name)
- `key_columns`: Primary key columns (for content addressing)
- `value_columns`: Value columns (data being stored)
- `indexes`: Secondary indexes for queries

**Usage:**
```rust
let state = D1SetupState {
    table_id: D1TableId {
        database_id: "my-db".to_string(),
        table_name: "symbols".to_string(),
    },
    key_columns: vec![
        ColumnSchema {
            name: "content_hash".to_string(),
            sql_type: "TEXT".to_string(),
            nullable: false,
            primary_key: true,
        },
    ],
    value_columns: vec![
        ColumnSchema {
            name: "symbol_name".to_string(),
            sql_type: "TEXT".to_string(),
            nullable: false,
            primary_key: false,
        },
        ColumnSchema {
            name: "file_path".to_string(),
            sql_type: "TEXT".to_string(),
            nullable: false,
            primary_key: false,
        },
    ],
    indexes: vec![
        IndexSchema {
            name: "idx_symbol_name".to_string(),
            columns: vec!["symbol_name".to_string()],
            unique: false,
        },
    ],
};
```

### ColumnSchema

Defines a single column in the D1 table.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColumnSchema {
    pub name: String,
    pub sql_type: String,
    pub nullable: bool,
    pub primary_key: bool,
}
```

**SQL Type Mappings:**
| ReCoco Type | D1 SQL Type | Notes |
|-------------|-------------|-------|
| `BasicValueType::Str` | `TEXT` | UTF-8 strings |
| `BasicValueType::Bytes` | `BLOB` | Binary data (base64 encoded) |
| `BasicValueType::Int64` | `INTEGER` | 64-bit integers |
| `BasicValueType::Float64` | `REAL` | Floating point |
| `BasicValueType::Bool` | `INTEGER` | 0 or 1 |
| `BasicValueType::Json` | `TEXT` | JSON serialized |
| `BasicValueType::Vector` | `TEXT` | JSON array |

**Example:**
```rust
let content_hash_column = ColumnSchema {
    name: "content_hash".to_string(),
    sql_type: "TEXT".to_string(),
    nullable: false,
    primary_key: true,
};
```

### IndexSchema

Defines a secondary index on the table.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IndexSchema {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
}
```

**Example:**
```rust
// Composite index on (file_path, symbol_name)
let composite_index = IndexSchema {
    name: "idx_file_symbol".to_string(),
    columns: vec![
        "file_path".to_string(),
        "symbol_name".to_string(),
    ],
    unique: false,
};

// Unique index on content_hash
let unique_index = IndexSchema {
    name: "idx_unique_hash".to_string(),
    columns: vec!["content_hash".to_string()],
    unique: true,
};
```

### D1SetupChange

Describes schema migrations to apply to the database.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D1SetupChange {
    pub table_id: D1TableId,
    pub create_table_sql: Option<String>,
    pub create_indexes_sql: Vec<String>,
    pub alter_table_sql: Vec<String>,
}
```

**Fields:**
- `create_table_sql`: SQL for creating new table (if needed)
- `create_indexes_sql`: SQL for creating indexes
- `alter_table_sql`: SQL for altering existing table schema

**Example:**
```rust
let change = D1SetupChange {
    table_id: D1TableId {
        database_id: "my-db".to_string(),
        table_name: "symbols".to_string(),
    },
    create_table_sql: Some(
        "CREATE TABLE symbols (content_hash TEXT PRIMARY KEY, symbol_name TEXT, file_path TEXT)".to_string()
    ),
    create_indexes_sql: vec![
        "CREATE INDEX idx_symbol_name ON symbols(symbol_name)".to_string(),
    ],
    alter_table_sql: vec![],
};
```

### D1ExportContext

Runtime context for D1 export operations (internal use).

```rust
pub struct D1ExportContext {
    pub database_id: String,
    pub table_name: String,
    pub account_id: String,
    pub api_token: String,
    pub http_client: reqwest::Client,
    pub key_fields_schema: Vec<FieldSchema>,
    pub value_fields_schema: Vec<FieldSchema>,
}
```

**Creation:**
```rust
let context = D1ExportContext::new(
    "my-database-id".to_string(),
    "code_symbols".to_string(),
    "my-account-id".to_string(),
    "my-api-token".to_string(),
    key_fields_schema,
    value_fields_schema,
)?;
```

**API URL:**
```rust
let url = context.api_url();
// Returns: "https://api.cloudflare.com/client/v4/accounts/{account_id}/d1/database/{database_id}/query"
```

---

## Setup State Management

D1 integration uses ReCoco's setup state system for automatic schema management.

### Setup State Lifecycle

```
┌─────────────────────────────────────────────┐
│  1. Define Desired State (D1SetupState)    │
│     - Table schema                          │
│     - Column types                          │
│     - Indexes                               │
└──────────────┬──────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────┐
│  2. Check Current State (if exists)         │
│     - Query D1 for existing schema          │
│     - Compare with desired state            │
└──────────────┬──────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────┐
│  3. Calculate Diff (SetupStateCompatibility)│
│     - Compatible → No changes needed        │
│     - Incompatible → Generate migration     │
└──────────────┬──────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────┐
│  4. Generate Migration (D1SetupChange)      │
│     - CREATE TABLE (if new)                 │
│     - ALTER TABLE (if schema changed)       │
│     - CREATE INDEX (if new indexes)         │
└──────────────┬──────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────┐
│  5. Apply Migration                         │
│     - Execute SQL via D1 HTTP API           │
│     - Store new setup state                 │
└─────────────────────────────────────────────┘
```

### Creating Setup State

```rust
use thread_flow::targets::d1::{D1SetupState, D1TableId, ColumnSchema, IndexSchema};

let setup_state = D1SetupState {
    table_id: D1TableId {
        database_id: env::var("D1_DATABASE_ID")?,
        table_name: "code_symbols".to_string(),
    },
    key_columns: vec![
        ColumnSchema {
            name: "content_hash".to_string(),
            sql_type: "TEXT".to_string(),
            nullable: false,
            primary_key: true,
        },
    ],
    value_columns: vec![
        ColumnSchema {
            name: "symbol_name".to_string(),
            sql_type: "TEXT".to_string(),
            nullable: false,
            primary_key: false,
        },
        ColumnSchema {
            name: "file_path".to_string(),
            sql_type: "TEXT".to_string(),
            nullable: false,
            primary_key: false,
        },
        ColumnSchema {
            name: "line_number".to_string(),
            sql_type: "INTEGER".to_string(),
            nullable: true,
            primary_key: false,
        },
    ],
    indexes: vec![
        IndexSchema {
            name: "idx_symbol_name".to_string(),
            columns: vec!["symbol_name".to_string()],
            unique: false,
        },
        IndexSchema {
            name: "idx_file_path".to_string(),
            columns: vec!["file_path".to_string()],
            unique: false,
        },
    ],
};
```

### Schema Compatibility

ReCoco's `SetupStateCompatibility` enum indicates compatibility status:

```rust
pub enum SetupStateCompatibility {
    /// Schemas are identical, no changes needed
    Compatible,

    /// Schemas are incompatible, migration required
    Incompatible(SetupChange),
}
```

**Compatibility Rules:**
- **Compatible** if:
  - All key columns match (name, type, nullability)
  - All value columns match (name, type, nullability)
  - All indexes match (name, columns, uniqueness)

- **Incompatible** if:
  - Key columns differ (requires table recreation)
  - Value columns added/removed/changed
  - Indexes added/removed/changed

### Generating Migrations

```rust
// Compare desired vs current state
let compatibility = current_state.is_compatible_with(&desired_state);

match compatibility {
    SetupStateCompatibility::Compatible => {
        println!("Schema up to date, no migration needed");
    }
    SetupStateCompatibility::Incompatible(change) => {
        println!("Migration required:");
        for description in change.describe_changes() {
            println!("  - {}", description);
        }
        // Apply migration
        apply_migration(&change).await?;
    }
}
```

---

## Query Building

D1ExportContext provides methods for building SQL queries.

### UPSERT Operations

```rust
pub fn build_upsert_stmt(
    &self,
    key: &KeyValue,
    values: &FieldValues,
) -> Result<(String, Vec<serde_json::Value>), RecocoError>
```

**Generated SQL:**
```sql
INSERT INTO {table} ({columns})
VALUES ({placeholders})
ON CONFLICT DO UPDATE SET
    {value_column_1} = excluded.{value_column_1},
    {value_column_2} = excluded.{value_column_2},
    ...
```

**Example:**
```rust
use recoco::base::value::{KeyValue, KeyPart, FieldValues, BasicValue};

// Create key: content_hash = "abc123"
let key = KeyValue(Box::new([
    KeyPart::Str("abc123".into()),
]));

// Create values: symbol_name = "MyClass", file_path = "src/main.rs"
let values = FieldValues {
    fields: vec![
        BasicValue::Str("MyClass".into()).into(),
        BasicValue::Str("src/main.rs".into()).into(),
    ].into(),
};

let (sql, params) = context.build_upsert_stmt(&key, &values)?;

// sql = "INSERT INTO code_symbols (content_hash, symbol_name, file_path)
//        VALUES (?, ?, ?)
//        ON CONFLICT DO UPDATE SET
//        symbol_name = excluded.symbol_name,
//        file_path = excluded.file_path"
// params = ["abc123", "MyClass", "src/main.rs"]
```

### DELETE Operations

```rust
pub fn build_delete_stmt(
    &self,
    key: &KeyValue,
) -> Result<(String, Vec<serde_json::Value>), RecocoError>
```

**Generated SQL:**
```sql
DELETE FROM {table}
WHERE {key_column_1} = ? AND {key_column_2} = ? ...
```

**Example:**
```rust
let key = KeyValue(Box::new([
    KeyPart::Str("abc123".into()),
]));

let (sql, params) = context.build_delete_stmt(&key)?;

// sql = "DELETE FROM code_symbols WHERE content_hash = ?"
// params = ["abc123"]
```

### Batch Operations

```rust
// Batch UPSERT
pub async fn upsert(
    &self,
    upserts: &[ExportTargetUpsertEntry],
) -> Result<(), RecocoError>

// Batch DELETE
pub async fn delete(
    &self,
    deletes: &[ExportTargetDeleteEntry],
) -> Result<(), RecocoError>
```

**Example:**
```rust
let upserts = vec![
    ExportTargetUpsertEntry {
        key: key1,
        value: value1,
    },
    ExportTargetUpsertEntry {
        key: key2,
        value: value2,
    },
];

context.upsert(&upserts).await?;
```

---

## Type Conversions

### KeyPart to JSON

```rust
pub fn key_part_to_json(
    key_part: &recoco::base::value::KeyPart
) -> Result<serde_json::Value, RecocoError>
```

**Type Mappings:**
| KeyPart Type | JSON Type | Example |
|--------------|-----------|---------|
| `Str(s)` | String | `"hello"` |
| `Bytes(b)` | String (base64) | `"SGVsbG8="` |
| `Bool(b)` | Boolean | `true` |
| `Int64(i)` | Number | `42` |
| `Range(r)` | Array | `[10, 20]` |
| `Uuid(u)` | String | `"550e8400-e29b-41d4-a716-446655440000"` |
| `Date(d)` | String (ISO 8601) | `"2025-01-28"` |
| `Struct(parts)` | Array | `["part1", "part2"]` |

**Example:**
```rust
use recoco::base::value::{KeyPart, RangeValue};

// String key
let str_part = KeyPart::Str("my_key".into());
let json = key_part_to_json(&str_part)?;
// json = "my_key"

// Bytes key (base64 encoded)
let bytes_part = KeyPart::Bytes(vec![1, 2, 3, 4, 5].into());
let json = key_part_to_json(&bytes_part)?;
// json = "AQIDBAU="

// Range key
let range_part = KeyPart::Range(RangeValue::new(10, 20));
let json = key_part_to_json(&range_part)?;
// json = [10, 20]
```

### Value to JSON

```rust
pub fn value_to_json(
    value: &Value
) -> Result<serde_json::Value, RecocoError>
```

**Type Mappings:**
| Value Type | JSON Type | Example |
|------------|-----------|---------|
| `Null` | Null | `null` |
| `Basic(Str)` | String | `"text"` |
| `Basic(Int64)` | Number | `123` |
| `Basic(Float64)` | Number | `3.14` |
| `Basic(Bool)` | Boolean | `true` |
| `Basic(Bytes)` | String (base64) | `"SGVsbG8="` |
| `Basic(Json)` | Object | `{"key": "value"}` |
| `Basic(Vector)` | Array | `[1, 2, 3]` |
| `Struct(fields)` | Array | `["field1", "field2"]` |
| `UTable/LTable` | Array of Arrays | `[[...], [...]]` |
| `KTable` | Object | `{"key1": [...], "key2": [...]}` |

**Example:**
```rust
use recoco::base::value::{Value, BasicValue};
use std::sync::Arc;

// String value
let str_val = Value::Basic(BasicValue::Str("hello".into()));
let json = value_to_json(&str_val)?;
// json = "hello"

// JSON object
let json_val = Value::Basic(BasicValue::Json(Arc::new(
    serde_json::json!({"name": "Alice", "age": 30})
)));
let json = value_to_json(&json_val)?;
// json = {"name": "Alice", "age": 30}

// Vector
let vec_val = Value::Basic(BasicValue::Vector(vec![
    BasicValue::Int64(1),
    BasicValue::Int64(2),
    BasicValue::Int64(3),
].into()));
let json = value_to_json(&vec_val)?;
// json = [1, 2, 3]
```

### BasicValue to JSON

```rust
pub fn basic_value_to_json(
    basic: &BasicValue
) -> Result<serde_json::Value, RecocoError>
```

**Example:**
```rust
use recoco::base::value::BasicValue;

let val = BasicValue::Int64(42);
let json = basic_value_to_json(&val)?;
// json = 42
```

---

## Configuration

### Environment Variables

```bash
# Required for D1 integration
export CLOUDFLARE_ACCOUNT_ID="your-account-id"
export D1_DATABASE_ID="your-database-id"
export CLOUDFLARE_API_TOKEN="your-api-token"

# Optional
export D1_TABLE_NAME="code_symbols"  # Default: from builder
```

### Cloudflare Setup

1. **Create D1 Database:**
   ```bash
   wrangler d1 create thread-analysis
   ```

2. **Get Database ID:**
   ```bash
   wrangler d1 list
   ```

3. **Create API Token:**
   - Go to Cloudflare Dashboard → My Profile → API Tokens
   - Create Token with D1 read/write permissions

4. **Initialize Schema:**
   ```bash
   wrangler d1 execute thread-analysis --local --file=schema.sql
   ```

### ThreadFlowBuilder Configuration

```rust
use thread_flow::ThreadFlowBuilder;
use std::env;

let flow = ThreadFlowBuilder::new("my_analysis")
    .source_local("src/", &["**/*.rs"], &["target/**"])
    .parse()
    .extract_symbols()
    .target_d1(
        env::var("CLOUDFLARE_ACCOUNT_ID")?,  // Account ID
        env::var("D1_DATABASE_ID")?,         // Database ID
        env::var("CLOUDFLARE_API_TOKEN")?,   // API Token
        "code_symbols",                       // Table name
        &["content_hash"],                    // Primary key fields
    )
    .build()
    .await?;
```

---

## Error Handling

### Common Errors

```rust
use thread_services::error::{ServiceError, ServiceResult};

// D1 API connection errors
Err(ServiceError::Connection { ... })

// Invalid schema configuration
Err(ServiceError::Config { ... })

// Type conversion errors
Err(ServiceError::Conversion { ... })

// D1 query execution errors
Err(ServiceError::Execution { ... })
```

### Error Recovery

```rust
use recoco::utils::prelude::Error as RecocoError;

match context.upsert(&upserts).await {
    Ok(_) => println!("UPSERT successful"),
    Err(RecocoError::Internal { message }) => {
        eprintln!("D1 API error: {}", message);
        // Retry logic here
    }
    Err(e) => {
        eprintln!("Unexpected error: {:?}", e);
        return Err(e);
    }
}
```

---

## Usage Examples

### Basic Code Symbol Export

```rust
use thread_flow::ThreadFlowBuilder;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build analysis flow
    let flow = ThreadFlowBuilder::new("rust_symbols")
        .source_local("src/", &["**/*.rs"], &["target/**"])
        .parse()
        .extract_symbols()
        .target_d1(
            env::var("CLOUDFLARE_ACCOUNT_ID")?,
            env::var("D1_DATABASE_ID")?,
            env::var("CLOUDFLARE_API_TOKEN")?,
            "code_symbols",
            &["content_hash"],
        )
        .build()
        .await?;

    // Execute flow
    flow.execute().await?;

    println!("✅ Symbols exported to D1");
    Ok(())
}
```

### Multi-Language Analysis

```rust
// Analyze both Rust and TypeScript files
let flow = ThreadFlowBuilder::new("multi_lang_analysis")
    .source_local(".", &["**/*.rs", "**/*.ts"], &["node_modules/**", "target/**"])
    .parse()
    .extract_symbols()
    .extract_imports()
    .target_d1(
        env::var("CLOUDFLARE_ACCOUNT_ID")?,
        env::var("D1_DATABASE_ID")?,
        env::var("CLOUDFLARE_API_TOKEN")?,
        "code_analysis",
        &["content_hash", "file_path"],
    )
    .build()
    .await?;
```

### Custom Schema

```rust
use thread_flow::targets::d1::{D1SetupState, D1TableId, ColumnSchema};

// Define custom schema
let custom_schema = D1SetupState {
    table_id: D1TableId {
        database_id: env::var("D1_DATABASE_ID")?,
        table_name: "custom_symbols".to_string(),
    },
    key_columns: vec![
        ColumnSchema {
            name: "file_hash".to_string(),
            sql_type: "TEXT".to_string(),
            nullable: false,
            primary_key: true,
        },
        ColumnSchema {
            name: "symbol_hash".to_string(),
            sql_type: "TEXT".to_string(),
            nullable: false,
            primary_key: true,
        },
    ],
    value_columns: vec![
        ColumnSchema {
            name: "symbol_type".to_string(),
            sql_type: "TEXT".to_string(),
            nullable: false,
            primary_key: false,
        },
        ColumnSchema {
            name: "metadata".to_string(),
            sql_type: "TEXT".to_string(),  // JSON
            nullable: true,
            primary_key: false,
        },
    ],
    indexes: vec![],
};
```

---

## Best Practices

### 1. **Use Content-Addressed Primary Keys**

Always include a content hash in your primary key for automatic deduplication:

```rust
.target_d1(
    account_id,
    database_id,
    api_token,
    "symbols",
    &["content_hash"],  // ✅ Enables deduplication
)
```

### 2. **Index Frequently Queried Columns**

Add indexes for columns you'll query:

```rust
indexes: vec![
    IndexSchema {
        name: "idx_symbol_name".to_string(),
        columns: vec!["symbol_name".to_string()],
        unique: false,
    },
    IndexSchema {
        name: "idx_file_path".to_string(),
        columns: vec!["file_path".to_string()],
        unique: false,
    },
],
```

### 3. **Batch Operations**

Use batch UPSERT/DELETE for efficiency:

```rust
// ✅ Good: Batch operation
context.upsert(&upserts).await?;

// ❌ Bad: Individual operations in loop
for entry in &upserts {
    context.upsert(&[entry.clone()]).await?;  // Slow!
}
```

### 4. **Handle Nullable Columns**

Set `nullable: true` for optional fields:

```rust
ColumnSchema {
    name: "description".to_string(),
    sql_type: "TEXT".to_string(),
    nullable: true,  // ✅ Optional field
    primary_key: false,
},
```

### 5. **Monitor API Rate Limits**

D1 has rate limits; implement retry logic:

```rust
use tokio::time::{sleep, Duration};

let mut retries = 3;
while retries > 0 {
    match context.upsert(&upserts).await {
        Ok(_) => break,
        Err(e) if e.to_string().contains("rate limit") => {
            retries -= 1;
            sleep(Duration::from_secs(2)).await;
        }
        Err(e) => return Err(e),
    }
}
```

### 6. **Use Appropriate SQL Types**

Choose SQL types based on data:

| Data Type | SQL Type | Notes |
|-----------|----------|-------|
| Small text | `TEXT` | < 1MB |
| Large text | `TEXT` | D1 has no TEXT size limit |
| Small integers | `INTEGER` | -2^63 to 2^63-1 |
| Decimals | `REAL` | Floating point |
| Binary data | `BLOB` | Raw bytes |
| JSON | `TEXT` | Use JSON functions |
| Booleans | `INTEGER` | 0 or 1 |

### 7. **Test Schema Migrations**

Always test migrations in local D1 first:

```bash
# Local D1
wrangler d1 execute my-db --local --file=migration.sql

# Verify schema
wrangler d1 execute my-db --local --command="SELECT * FROM sqlite_master WHERE type='table'"
```

---

## Next Steps

- **Deployment Guide**: See `crates/cloudflare/docs/EDGE_DEPLOYMENT.md` for Cloudflare Workers setup (segregated in cloudflare directory)
- **Performance Tuning**: See `docs/operations/PERFORMANCE_TUNING.md` for optimization strategies
- **Troubleshooting**: See `docs/operations/TROUBLESHOOTING.md` for common issues

---

**Last Updated**: 2025-01-28
**Maintainers**: Thread Team
**License**: AGPL-3.0-or-later
