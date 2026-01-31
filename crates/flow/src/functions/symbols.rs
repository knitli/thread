// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

use async_trait::async_trait;
use recoco::base::schema::{EnrichedValueType, TableKind, TableSchema, ValueType};
use recoco::base::value::Value;
use recoco::ops::factory_bases::SimpleFunctionFactoryBase;
use recoco::ops::interface::{FlowInstanceContext, SimpleFunctionExecutor};
use recoco::ops::sdk::{OpArgsResolver, SimpleFunctionAnalysisOutput};
use serde::Deserialize;
use std::sync::Arc;

/// Factory for creating the ExtractSymbolsExecutor
pub struct ExtractSymbolsFactory;

/// Spec for extract_symbols operator (empty - uses default args)
#[derive(Debug, Clone, Deserialize)]
pub struct ExtractSymbolsSpec {}

#[async_trait]
impl SimpleFunctionFactoryBase for ExtractSymbolsFactory {
    type Spec = ExtractSymbolsSpec;
    type ResolvedArgs = ();

    fn name(&self) -> &str {
        "extract_symbols"
    }

    async fn analyze<'a>(
        &'a self,
        _spec: &'a Self::Spec,
        _args_resolver: &mut OpArgsResolver<'a>,
        _context: &FlowInstanceContext,
    ) -> Result<SimpleFunctionAnalysisOutput<Self::ResolvedArgs>, recoco::prelude::Error> {
        Ok(SimpleFunctionAnalysisOutput {
            resolved_args: (),
            output_schema: get_symbols_output_schema(),
            behavior_version: Some(1),
        })
    }

    async fn build_executor(
        self: Arc<Self>,
        _spec: Self::Spec,
        _resolved_args: Self::ResolvedArgs,
        _context: Arc<FlowInstanceContext>,
    ) -> Result<impl SimpleFunctionExecutor, recoco::prelude::Error> {
        Ok(ExtractSymbolsExecutor)
    }
}

/// Executor that extracts the symbols table from a parsed document
pub struct ExtractSymbolsExecutor;

#[async_trait]
impl SimpleFunctionExecutor for ExtractSymbolsExecutor {
    async fn evaluate(&self, input: Vec<Value>) -> Result<Value, recoco::prelude::Error> {
        // Input: parsed_document (Struct with fields: symbols, imports, calls)
        let parsed_doc = input
            .first()
            .ok_or_else(|| recoco::prelude::Error::client("Missing parsed_document input"))?;

        // Extract the first field (symbols table)
        match parsed_doc {
            Value::Struct(field_values) => {
                let symbols = field_values
                    .fields
                    .first()
                    .ok_or_else(|| {
                        recoco::prelude::Error::client("Missing symbols field in parsed_document")
                    })?
                    .clone();

                Ok(symbols)
            }
            _ => Err(recoco::prelude::Error::client(
                "Expected Struct for parsed_document",
            )),
        }
    }

    fn enable_cache(&self) -> bool {
        true
    }

    fn timeout(&self) -> Option<std::time::Duration> {
        Some(std::time::Duration::from_secs(30))
    }
}

/// Build the schema for the output of ExtractSymbols (just the symbols table)
fn get_symbols_output_schema() -> EnrichedValueType {
    EnrichedValueType {
        typ: ValueType::Table(TableSchema {
            kind: TableKind::LTable,
            row: match crate::conversion::symbol_type() {
                ValueType::Struct(s) => s,
                _ => unreachable!(),
            },
        }),
        nullable: false,
        attrs: Default::default(),
    }
}
