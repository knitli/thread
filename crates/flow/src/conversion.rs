// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

use cocoindex::base::schema::{EnrichedValueType, FieldSchema, StructType, ValueType};
use cocoindex::base::value::Value;
use std::collections::HashMap;
use thread_services::types::{CallInfo, ImportInfo, ParsedDocument, SymbolInfo};

/// Convert a ParsedDocument to a CocoIndex Value
pub fn serialize_parsed_doc<D: thread_services::types::Doc>(
    doc: &ParsedDocument<D>,
) -> Result<Value, cocoindex::error::Error> {
    let mut fields = HashMap::new();

    // Serialize AST (as source representation for now, or S-expr)
    // Note: A full AST serialization would be very large.
    // We'll store the generated source or S-expression.
    // For now, let's store metadata provided by ParsedDocument.

    // fields.insert("ast".to_string(), Value::String(doc.root().to_sexp().to_string()));
    // Actually, let's stick to what's practical: extracted metadata.

    // Serialize symbols
    let symbols = doc
        .metadata
        .defined_symbols
        .values()
        .map(serialize_symbol)
        .collect::<Result<Vec<_>, _>>()?;
    fields.insert("symbols".to_string(), Value::LTable(symbols));

    // Serialize imports
    let imports = doc
        .metadata
        .imported_symbols
        .values()
        .map(serialize_import)
        .collect::<Result<Vec<_>, _>>()?;
    fields.insert("imports".to_string(), Value::LTable(imports));

    // Serialize calls
    let calls = doc
        .metadata
        .function_calls
        .iter()
        .map(serialize_call)
        .collect::<Result<Vec<_>, _>>()?;
    fields.insert("calls".to_string(), Value::LTable(calls));

    Ok(Value::Struct(FieldValues {
        fields: Arc::new(vec![
            fields.remove("symbols").unwrap_or(Value::Null),
            fields.remove("imports").unwrap_or(Value::Null),
            fields.remove("calls").unwrap_or(Value::Null),
        ]),
    }))
}

fn serialize_symbol(info: &SymbolInfo) -> Result<Value, cocoindex::error::Error> {
    let mut fields = HashMap::new();
    fields.insert("name".to_string(), Value::Basic(BasicValue::Str(info.name.clone().into())));
    fields.insert(
        "kind".to_string(),
        Value::Basic(BasicValue::Str(format!("{:?}", info.kind).into())),
    ); // SymbolKind doesn't impl Display/Serialize yet
    fields.insert("scope".to_string(), Value::Basic(BasicValue::Str(info.scope.clone().into())));
    // Position can be added if needed
    Ok(Value::Struct(fields))
}

fn serialize_import(info: &ImportInfo) -> Result<Value, cocoindex::error::Error> {
    let mut fields = HashMap::new();
    fields.insert(
        "symbol_name".to_string(),
        Value::Basic(BasicValue::Str(info.symbol_name.clone().into())),
    );
    fields.insert(
        "source_path".to_string(),
        Value::Basic(BasicValue::Str(info.source_path.clone().into())),
    );
    fields.insert(
        "kind".to_string(),
        Value::Basic(BasicValue::Str(format!("{:?}", info.import_kind).into())),
    );
    Ok(Value::Struct(fields))
}

fn serialize_call(info: &CallInfo) -> Result<Value, cocoindex::error::Error> {
    let mut fields = HashMap::new();
    fields.insert(
        "function_name".to_string(),
        Value::Basic(BasicValue::Str(info.function_name.clone().into())),
    );
    fields.insert(
        "arguments_count".to_string(),
        Value::Basic(BasicValue::Int64(info.arguments_count as i64)),
    );
    Ok(Value::Struct(fields))
}

/// Build the schema for the output of ThreadParse
    EnrichedValueType {
        typ: ValueType::Struct(StructType {
            fields: Arc::new(vec![
                FieldSchema::new(
                    "symbols".to_string(),
                    EnrichedValueType {
                        typ: ValueType::Table(TableSchema {
                            kind: TableKind::LTable,
                            row: match symbol_type() {
                                ValueType::Struct(s) => s,
                                _ => unreachable!(),
                            },
                        }),
                        nullable: false,
                        attrs: Default::default(),
                    },
                ),
                FieldSchema::new(
                    "imports".to_string(),
                    EnrichedValueType {
                        typ: ValueType::Table(TableSchema {
                            kind: TableKind::LTable,
                            row: match import_type() {
                                ValueType::Struct(s) => s,
                                _ => unreachable!(),
                            },
                        }),
                        nullable: false,
                        attrs: Default::default(),
                    },
                ),
                FieldSchema::new(
                    "calls".to_string(),
                    EnrichedValueType {
                        typ: ValueType::Table(TableSchema {
                            kind: TableKind::LTable,
                            row: match call_type() {
                                ValueType::Struct(s) => s,
                                _ => unreachable!(),
                            },
                        }),
                        nullable: false,
                        attrs: Default::default(),
                    },
                ),
            ]),
            description: None,
        }),
        nullable: false,
        attrs: Default::default(),
    }
}

fn symbol_type() -> ValueType {
    ValueType::Struct(StructType {
        fields: vec![
            FieldSchema::new("name".to_string(), ValueType::String),
            FieldSchema::new("kind".to_string(), ValueType::String),
            FieldSchema::new("scope".to_string(), ValueType::String),
        ],
    })
}

fn import_type() -> ValueType {
    ValueType::Struct(StructType {
        fields: vec![
            FieldSchema::new("symbol_name".to_string(), ValueType::String),
            FieldSchema::new("source_path".to_string(), ValueType::String),
            FieldSchema::new("kind".to_string(), ValueType::String),
        ],
    })
}

fn call_type() -> ValueType {
    ValueType::Struct(StructType {
        fields: vec![
            FieldSchema::new("function_name".to_string(), ValueType::String),
            FieldSchema::new("arguments_count".to_string(), ValueType::Int),
        ],
    })
}
