# Thread D1 Target Factory Test

This example demonstrates the D1 target factory implementation for exporting Thread code analysis results to Cloudflare D1 databases.

## What This Tests

This is a **direct test of the D1 target factory** without a full dataflow pipeline. It validates:

- ✅ D1Spec configuration
- ✅ D1ExportContext creation with schema definitions
- ✅ ExportTargetUpsertEntry and ExportTargetDeleteEntry construction
- ✅ ReCoco Value → JSON type conversions
- ✅ UPSERT and DELETE SQL statement generation patterns

## Prerequisites

```bash
# 1. Wrangler CLI (for local D1 testing)
npm install -g wrangler

# 2. Thread flow crate built
cd /home/knitli/thread
cargo build -p thread-flow
```

## Quick Start

### 1. Run the Test

```bash
cd /home/knitli/thread

# Build and run the example
cargo run --example d1_local_test
```

**Expected Output:**

```
🚀 Thread D1 Target Factory Test

📋 Configuration:
   Database: thread_test
   Table: code_symbols

✅ Target factory: d1

🔧 Export context created
   Key fields: ["content_hash"]
   Value fields: ["file_path", "symbol_name", "symbol_type", "start_line", "end_line", "source_code", "language"]

📊 Sample Data:
   1. "main"
   2. "Calculator"
   3. "capitalize"

🔄 Testing UPSERT operation...
   ⚠️  Skipping actual HTTP call (test credentials)
   In production, this would:
      1. Convert ReCoco values to JSON
      2. Build UPSERT SQL statements
      3. Execute batch via D1 HTTP API
      4. Handle response and errors

🗑️  Testing DELETE operation...
   ⚠️  Skipping actual HTTP call (test credentials)
   In production, this would:
      1. Extract key from KeyValue
      2. Build DELETE SQL statement
      3. Execute via D1 HTTP API

📝 Example SQL that would be generated:

   UPSERT:
   INSERT INTO code_symbols (content_hash, file_path, symbol_name, symbol_type, start_line, end_line, source_code, language)
   VALUES (?, ?, ?, ?, ?, ?, ?, ?)
   ON CONFLICT DO UPDATE SET
     file_path = excluded.file_path,
     symbol_name = excluded.symbol_name,
     symbol_type = excluded.symbol_type,
     start_line = excluded.start_line,
     end_line = excluded.end_line,
     source_code = excluded.source_code,
     language = excluded.language;

   DELETE:
   DELETE FROM code_symbols WHERE content_hash = ?;

✅ D1 Target Factory Test Complete!

💡 Next Steps:
   1. Set up local D1: wrangler d1 execute thread_test --local --file=schema.sql
   2. Update credentials to use real Cloudflare account
   3. Integrate into ThreadFlowBuilder for full pipeline
   4. Test with real D1 database (local or production)
```

### 2. (Optional) Set Up Local D1 for Real Testing

If you want to test with actual D1 HTTP calls:

```bash
cd crates/flow/examples/d1_local_test

# Create local D1 database
wrangler d1 execute thread_test --local --file=schema.sql

# Start Wrangler in local mode (runs D1 HTTP API on localhost:8787)
wrangler dev --local

# In another terminal, update main.rs to use localhost D1 endpoint
# Then run: cargo run --example d1_local_test
```

## What Gets Tested

### 1. **Schema Definition**

The example creates a realistic schema with:
- Primary key: `content_hash` (for content-addressed deduplication)
- Value fields: file_path, symbol_name, symbol_type, line numbers, source code, language

### 2. **Type Conversions**

Tests ReCoco type system integration:
```rust
// String values
Value::Basic(BasicValue::Str("example".to_string()))

// Integer values
Value::Basic(BasicValue::Int64(42))

// Key parts
KeyValue(Box::new([KeyPart::Str("hash123".to_string())]))
```

### 3. **Mutation Operations**

Creates sample mutations:
- **UPSERT**: 3 symbol entries (main function, Calculator struct, capitalize function)
- **DELETE**: 1 entry removal by content hash

### 4. **SQL Generation Pattern**

Shows what SQL the D1 target factory generates:
- SQLite INSERT ... ON CONFLICT DO UPDATE SET (UPSERT)
- Batch statement grouping for efficiency
- Primary key-based deduplication

## Integration Points

This example validates the **D1 target factory in isolation**. In production:

1. **ThreadFlowBuilder** would orchestrate the full pipeline:
   ```rust
   let mut builder = ThreadFlowBuilder::new("code_analysis")
       .source_local("src/", &["*.rs", "*.ts"], &[])
       .parse()
       .extract_symbols()
       .target_d1(d1_spec);  // <-- D1 target integration point
   ```

2. **ReCoco FlowBuilder** would:
   - Call `D1TargetFactory::build()` to create export contexts
   - Execute the flow and collect mutations
   - Call `D1TargetFactory::apply_mutation()` with batched data

3. **Real D1 API** would:
   - Receive HTTP POST to `/database/<database_id>/query`
   - Execute batch SQL statements in transaction
   - Return success/error responses

## File Structure

```
d1_local_test/
├── main.rs              # Test program
├── schema.sql           # D1 table schema
├── wrangler.toml        # Wrangler configuration
├── README.md            # This file
└── sample_code/         # Sample files (for future full integration)
    ├── calculator.rs
    └── utils.ts
```

## Known Limitations

1. **No Actual HTTP Calls**: Example uses test credentials and skips HTTP calls
   - To test HTTP: Set up local Wrangler and update credentials

2. **No Full Flow**: Tests D1 target factory directly, not via ThreadFlowBuilder
   - Full integration requires ThreadFlowBuilder.target_d1() implementation

3. **Schema Changes Not Tested**: `apply_setup_changes()` requires manual execution
   - Use: `wrangler d1 execute thread_test --local --file=schema.sql`

## Next Steps for Production

### 1. ThreadFlowBuilder Integration

Add D1 target support to ThreadFlowBuilder:
```rust
impl ThreadFlowBuilder {
    pub fn target_d1(mut self, spec: D1Spec) -> Self {
        self.target = Some(Target::D1(spec));
        self
    }
}
```

### 2. Real D1 Testing

Test with Cloudflare D1 (local or production):
```bash
# Local D1
wrangler dev --local
# Update main.rs with localhost:8787 endpoint

# Production D1
wrangler d1 create thread-prod
# Update main.rs with production credentials
```

### 3. Content-Addressed Incremental Analysis

Implement hash-based change detection:
```rust
// Only re-analyze files where content hash changed
let hash = calculate_content_hash(&file_content);
if hash != db_hash {
    analyze_and_upsert(file, hash);
}
```

### 4. Edge Deployment

Deploy to Cloudflare Workers:
```rust
// Worker uses D1 binding (not HTTP API)
#[event(fetch)]
pub async fn main(req: Request, env: Env) -> Result<Response> {
    let db = env.d1("DB")?;
    // Direct D1 access without HTTP overhead
}
```

## Validation Checklist

- ✅ D1TargetFactory compiles without errors
- ✅ Type conversions (ReCoco Value → JSON) tested
- ✅ UPSERT and DELETE SQL patterns validated
- ✅ Schema definition complete with indexes
- ✅ Example runs and shows expected output
- ⏳ HTTP API integration (requires real D1 setup)
- ⏳ ThreadFlowBuilder integration (future work)
- ⏳ End-to-end flow testing (future work)

## Troubleshooting

### Example won't compile
```bash
# Ensure recoco dependency is available
cargo build -p thread-flow

# Check imports match local recoco source
ls /home/knitli/recoco/crates/recoco-core/src/
```

### Want to test real HTTP calls
```bash
# 1. Set up local D1
cd crates/flow/examples/d1_local_test
wrangler d1 execute thread_test --local --file=schema.sql

# 2. Start Wrangler dev server
wrangler dev --local

# 3. Update main.rs:
# - Use real account_id from Cloudflare dashboard
# - Use api_token from Cloudflare API tokens
# - Point to localhost:8787 for local testing

# 4. Run example
cargo run --example d1_local_test
```

### SQL generation issues
Check the D1 target factory implementation at:
`/home/knitli/thread/crates/flow/src/targets/d1.rs`

Key methods:
- `build_upsert_stmt()` - Generates INSERT ... ON CONFLICT SQL
- `build_delete_stmt()` - Generates DELETE WHERE key = ? SQL
- `key_part_to_json()` - Converts ReCoco KeyPart to JSON
- `value_to_json()` - Converts ReCoco Value to JSON

## References

- **D1 Documentation**: https://developers.cloudflare.com/d1/
- **ReCoco Target Pattern**: `/home/knitli/thread/crates/flow/docs/RECOCO_TARGET_PATTERN.md`
- **D1 Target Factory**: `/home/knitli/thread/crates/flow/src/targets/d1.rs`
- **Wrangler CLI**: https://developers.cloudflare.com/workers/wrangler/
