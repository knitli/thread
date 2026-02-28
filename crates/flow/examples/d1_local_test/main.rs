// SPDX-FileCopyrightText: 2026 Knitli Inc.
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use recoco::base::schema::{BasicValueType, EnrichedValueType, FieldSchema, ValueType};
use recoco::base::value::{BasicValue, FieldValues, KeyValue, Value};
use recoco::ops::factory_bases::TargetFactoryBase;
use recoco::ops::interface::{
    ExportTargetDeleteEntry, ExportTargetMutationWithContext, ExportTargetUpsertEntry,
};
use thread_flow::targets::d1::{D1ExportContext, D1Spec, D1TargetFactory};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Thread D1 Target Factory Test\n");

    // This example tests the D1 target factory directly without a full flow
    // In production, this would be integrated into ThreadFlowBuilder

    // 1. Create D1 specification
    let d1_spec = D1Spec {
        account_id: "test-account".to_string(),
        database_id: "thread_test".to_string(),
        api_token: "test-token".to_string(),
        table_name: Some("code_symbols".to_string()),
    };

    println!("📋 Configuration:");
    println!("   Database: {}", d1_spec.database_id);
    println!("   Table: {}\n", d1_spec.table_name.as_ref().unwrap());

    // 2. Create target factory
    let factory = D1TargetFactory;
    println!("✅ Target factory: {}", factory.name());

    // 3. Create export context (this would normally be done by FlowBuilder)
    let key_fields_schema = vec![FieldSchema::new(
        "content_hash",
        EnrichedValueType {
            typ: ValueType::Basic(BasicValueType::Str),
            nullable: false,
            attrs: Default::default(),
        },
    )];

    let value_fields_schema = vec![
        FieldSchema::new(
            "file_path",
            EnrichedValueType {
                typ: ValueType::Basic(BasicValueType::Str),
                nullable: false,
                attrs: Default::default(),
            },
        ),
        FieldSchema::new(
            "symbol_name",
            EnrichedValueType {
                typ: ValueType::Basic(BasicValueType::Str),
                nullable: false,
                attrs: Default::default(),
            },
        ),
        FieldSchema::new(
            "symbol_type",
            EnrichedValueType {
                typ: ValueType::Basic(BasicValueType::Str),
                nullable: false,
                attrs: Default::default(),
            },
        ),
        FieldSchema::new(
            "start_line",
            EnrichedValueType {
                typ: ValueType::Basic(BasicValueType::Int64),
                nullable: false,
                attrs: Default::default(),
            },
        ),
        FieldSchema::new(
            "end_line",
            EnrichedValueType {
                typ: ValueType::Basic(BasicValueType::Int64),
                nullable: false,
                attrs: Default::default(),
            },
        ),
        FieldSchema::new(
            "source_code",
            EnrichedValueType {
                typ: ValueType::Basic(BasicValueType::Str),
                nullable: false,
                attrs: Default::default(),
            },
        ),
        FieldSchema::new(
            "language",
            EnrichedValueType {
                typ: ValueType::Basic(BasicValueType::Str),
                nullable: false,
                attrs: Default::default(),
            },
        ),
    ];

    let metrics = thread_flow::monitoring::performance::PerformanceMetrics::new();

    let export_context = D1ExportContext::new_with_default_client(
        d1_spec.database_id.clone(),
        d1_spec.table_name.clone().unwrap(),
        d1_spec.account_id.clone(),
        d1_spec.api_token.clone(),
        key_fields_schema,
        value_fields_schema,
        metrics,
    )
    .expect("Failed to create D1 export context");

    println!("🔧 Export context created");
    println!(
        "   Key fields: {:?}",
        export_context
            .key_fields_schema
            .iter()
            .map(|f| &f.name)
            .collect::<Vec<_>>()
    );
    println!(
        "   Value fields: {:?}\n",
        export_context
            .value_fields_schema
            .iter()
            .map(|f| &f.name)
            .collect::<Vec<_>>()
    );

    // 4. Create sample data (simulating parsed code symbols)
    let sample_entries = vec![
        create_symbol_entry(
            "abc123",
            "src/main.rs",
            "main",
            "function",
            1,
            10,
            "fn main() { ... }",
            "rust",
        ),
        create_symbol_entry(
            "def456",
            "src/lib.rs",
            "Calculator",
            "struct",
            15,
            50,
            "pub struct Calculator { ... }",
            "rust",
        ),
        create_symbol_entry(
            "ghi789",
            "src/utils.ts",
            "capitalize",
            "function",
            5,
            8,
            "export function capitalize(str: string) { ... }",
            "typescript",
        ),
    ];

    println!("📊 Sample Data:");
    for (i, entry) in sample_entries.iter().enumerate() {
        println!("   {}. {:?}", i + 1, get_symbol_name(&entry.value));
    }
    println!();

    // 5. Test UPSERT operation
    println!("🔄 Testing UPSERT operation...");

    // Note: This will fail with actual HTTP calls since we're using test credentials
    // In real usage, you would:
    // 1. Set up local D1 with: wrangler d1 execute thread_test --local --file=schema.sql
    // 2. Use real account_id and api_token from Cloudflare
    // 3. Point to localhost:8787 for local D1 API

    // Clone a key for later delete test
    let first_key = sample_entries[0].key.clone();

    let mutation = recoco::ops::interface::ExportTargetMutation {
        upserts: sample_entries,
        deletes: vec![],
    };

    let _mutation_with_context = ExportTargetMutationWithContext {
        mutation,
        export_context: &export_context,
    };

    // This would execute the actual upsert:
    // factory.apply_mutation(vec![mutation_with_context]).await?;

    println!("   ⚠️  Skipping actual HTTP call (test credentials)");
    println!("   In production, this would:");
    println!("      1. Convert ReCoco values to JSON");
    println!("      2. Build UPSERT SQL statements");
    println!("      3. Execute batch via D1 HTTP API");
    println!("      4. Handle response and errors\n");

    // 6. Test DELETE operation
    println!("🗑️  Testing DELETE operation...");

    let delete_entries = vec![ExportTargetDeleteEntry {
        key: first_key,
        additional_key: serde_json::Value::Null,
    }];

    let delete_mutation = recoco::ops::interface::ExportTargetMutation {
        upserts: vec![],
        deletes: delete_entries,
    };

    let _delete_mutation_with_context = ExportTargetMutationWithContext {
        mutation: delete_mutation,
        export_context: &export_context,
    };

    println!("   ⚠️  Skipping actual HTTP call (test credentials)");
    println!("   In production, this would:");
    println!("      1. Extract key from KeyValue");
    println!("      2. Build DELETE SQL statement");
    println!("      3. Execute via D1 HTTP API\n");

    // 7. Show what SQL would be generated
    println!("📝 Example SQL that would be generated:\n");
    println!("   UPSERT:");
    println!(
        "   INSERT INTO code_symbols (content_hash, file_path, symbol_name, symbol_type, start_line, end_line, source_code, language)"
    );
    println!("   VALUES (?, ?, ?, ?, ?, ?, ?, ?)");
    println!("   ON CONFLICT DO UPDATE SET");
    println!("     file_path = excluded.file_path,");
    println!("     symbol_name = excluded.symbol_name,");
    println!("     symbol_type = excluded.symbol_type,");
    println!("     start_line = excluded.start_line,");
    println!("     end_line = excluded.end_line,");
    println!("     source_code = excluded.source_code,");
    println!("     language = excluded.language;\n");

    println!("   DELETE:");
    println!("   DELETE FROM code_symbols WHERE content_hash = ?;\n");

    println!("✅ D1 Target Factory Test Complete!\n");
    println!("💡 Next Steps:");
    println!("   1. Set up local D1: wrangler d1 execute thread_test --local --file=schema.sql");
    println!("   2. Update credentials to use real Cloudflare account");
    println!("   3. Integrate into ThreadFlowBuilder for full pipeline");
    println!("   4. Test with real D1 database (local or production)");

    Ok(())
}

/// Helper to create a symbol entry for testing
#[allow(clippy::too_many_arguments)]
fn create_symbol_entry(
    hash: &str,
    file_path: &str,
    symbol_name: &str,
    symbol_type: &str,
    start_line: i64,
    end_line: i64,
    source_code: &str,
    language: &str,
) -> ExportTargetUpsertEntry {
    use recoco::base::value::KeyPart;

    let key = KeyValue(Box::new([KeyPart::Str(hash.into())]));

    // FieldValues is positionally matched to value_fields_schema
    // Order: file_path, symbol_name, symbol_type, start_line, end_line, source_code, language
    let value = FieldValues {
        fields: vec![
            Value::Basic(BasicValue::Str(file_path.into())),
            Value::Basic(BasicValue::Str(symbol_name.into())),
            Value::Basic(BasicValue::Str(symbol_type.into())),
            Value::Basic(BasicValue::Int64(start_line)),
            Value::Basic(BasicValue::Int64(end_line)),
            Value::Basic(BasicValue::Str(source_code.into())),
            Value::Basic(BasicValue::Str(language.into())),
        ],
    };

    ExportTargetUpsertEntry {
        key,
        additional_key: serde_json::Value::Null,
        value,
    }
}

/// Helper to extract symbol name from FieldValues for display
fn get_symbol_name(fields: &FieldValues) -> String {
    // Index 1 is symbol_name in our schema order
    if let Some(Value::Basic(BasicValue::Str(s))) = fields.fields.get(1) {
        s.to_string()
    } else {
        "unknown".to_string()
    }
}
