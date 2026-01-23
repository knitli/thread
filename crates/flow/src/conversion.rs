// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

use cocoindex::base::schema::{
    BasicValueType, EnrichedValueType, FieldSchema, StructType, TableKind, TableSchema, ValueType,
};
use cocoindex::base::value::{BasicValue, FieldValues, ScopeValue, Value};

use std::sync::Arc;
use thread_services::types::{CallInfo, ImportInfo, ParsedDocument, SymbolInfo};

/// Convert a ParsedDocument to a CocoIndex Value
pub fn serialize_parsed_doc<D: thread_services::types::Doc>(
    doc: &ParsedDocument<D>,
) -> Result<Value, cocoindex::error::Error> {
    // Note: serialize_symbol etc now return ScopeValue.
    // Value::LTable takes Vec<ScopeValue>.

    // Serialize symbols
    let symbols = doc
        .metadata
        .defined_symbols
        .values()
        .map(serialize_symbol)
        .collect::<Result<Vec<_>, _>>()?;

    // Serialize imports
    let imports = doc
        .metadata
        .imported_symbols
        .values()
        .map(serialize_import)
        .collect::<Result<Vec<_>, _>>()?;

    // Serialize calls
    let calls = doc
        .metadata
        .function_calls
        .iter()
        .map(serialize_call)
        .collect::<Result<Vec<_>, _>>()?;

    // Output is a Struct containing LTables.
    // Value::Struct takes FieldValues. FieldValues takes fields: Vec<Value>.
    // Value::LTable(symbols) is Value::LTable(Vec<ScopeValue>). This is a Value.
    // So fields is Vec<Value>. Correct.

    Ok(Value::Struct(FieldValues {
        fields: vec![
            Value::LTable(symbols),
            Value::LTable(imports),
            Value::LTable(calls),
        ],
    }))
}

fn serialize_symbol(info: &SymbolInfo) -> Result<ScopeValue, cocoindex::error::Error> {
    Ok(ScopeValue(FieldValues {
        fields: vec![
            Value::Basic(BasicValue::Str(info.name.clone().into())),
            Value::Basic(BasicValue::Str(format!("{:?}", info.kind).into())),
            Value::Basic(BasicValue::Str(info.scope.clone().into())),
        ],
    }))
}

fn serialize_import(info: &ImportInfo) -> Result<ScopeValue, cocoindex::error::Error> {
    Ok(ScopeValue(FieldValues {
        fields: vec![
            Value::Basic(BasicValue::Str(info.symbol_name.clone().into())),
            Value::Basic(BasicValue::Str(info.source_path.clone().into())),
            Value::Basic(BasicValue::Str(format!("{:?}", info.import_kind).into())),
        ],
    }))
}

fn serialize_call(info: &CallInfo) -> Result<ScopeValue, cocoindex::error::Error> {
    Ok(ScopeValue(FieldValues {
        fields: vec![
            Value::Basic(BasicValue::Str(info.function_name.clone().into())),
            Value::Basic(BasicValue::Int64(info.arguments_count as i64)),
        ],
    }))
}

/// Build the schema for the output of ThreadParse
pub fn get_thread_parse_output_schema() -> EnrichedValueType {
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
            FieldSchema::new(
                "name".to_string(),
                EnrichedValueType {
                    typ: ValueType::Basic(BasicValueType::Str),
                    nullable: false,
                    attrs: Default::default(),
                },
            ),
            FieldSchema::new(
                "kind".to_string(),
                EnrichedValueType {
                    typ: ValueType::Basic(BasicValueType::Str),
                    nullable: false,
                    attrs: Default::default(),
                },
            ),
            FieldSchema::new(
                "scope".to_string(),
                EnrichedValueType {
                    typ: ValueType::Basic(BasicValueType::Str),
                    nullable: false,
                    attrs: Default::default(),
                },
            ),
        ]
        .into(),
        description: None,
    })
}

fn import_type() -> ValueType {
    ValueType::Struct(StructType {
        fields: vec![
            FieldSchema::new(
                "symbol_name".to_string(),
                EnrichedValueType {
                    typ: ValueType::Basic(BasicValueType::Str),
                    nullable: false,
                    attrs: Default::default(),
                },
            ),
            FieldSchema::new(
                "source_path".to_string(),
                EnrichedValueType {
                    typ: ValueType::Basic(BasicValueType::Str),
                    nullable: false,
                    attrs: Default::default(),
                },
            ),
            FieldSchema::new(
                "kind".to_string(),
                EnrichedValueType {
                    typ: ValueType::Basic(BasicValueType::Str),
                    nullable: false,
                    attrs: Default::default(),
                },
            ),
        ]
        .into(),
        description: None,
    })
}

fn call_type() -> ValueType {
    ValueType::Struct(StructType {
        fields: vec![
            FieldSchema::new(
                "function_name".to_string(),
                EnrichedValueType {
                    typ: ValueType::Basic(BasicValueType::Str),
                    nullable: false,
                    attrs: Default::default(),
                },
            ),
            FieldSchema::new(
                "arguments_count".to_string(),
                EnrichedValueType {
                    typ: ValueType::Basic(BasicValueType::Int64),
                    nullable: false,
                    attrs: Default::default(),
                },
            ),
        ]
        .into(),
        description: None,
    })
}
