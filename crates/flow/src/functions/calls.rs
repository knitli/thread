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

/// Factory for creating the ExtractCallsExecutor
pub struct ExtractCallsFactory;

/// Spec for extract_calls operator (empty - uses default args)
#[derive(Debug, Clone, Deserialize)]
pub struct ExtractCallsSpec {}

#[async_trait]
impl SimpleFunctionFactoryBase for ExtractCallsFactory {
    type Spec = ExtractCallsSpec;
    type ResolvedArgs = ();

    fn name(&self) -> &str {
        "extract_calls"
    }

    async fn analyze<'a>(
        &'a self,
        _spec: &'a Self::Spec,
        _args_resolver: &mut OpArgsResolver<'a>,
        _context: &FlowInstanceContext,
    ) -> Result<SimpleFunctionAnalysisOutput<Self::ResolvedArgs>, recoco::prelude::Error> {
        Ok(SimpleFunctionAnalysisOutput {
            resolved_args: (),
            output_schema: get_calls_output_schema(),
            behavior_version: Some(1),
        })
    }

    async fn build_executor(
        self: Arc<Self>,
        _spec: Self::Spec,
        _resolved_args: Self::ResolvedArgs,
        _context: Arc<FlowInstanceContext>,
    ) -> Result<impl SimpleFunctionExecutor, recoco::prelude::Error> {
        Ok(ExtractCallsExecutor)
    }
}

/// Executor that extracts the calls table from a parsed document
pub struct ExtractCallsExecutor;

#[async_trait]
impl SimpleFunctionExecutor for ExtractCallsExecutor {
    async fn evaluate(&self, input: Vec<Value>) -> Result<Value, recoco::prelude::Error> {
        // Input: parsed_document (Struct with fields: symbols, imports, calls)
        let parsed_doc = input
            .first()
            .ok_or_else(|| recoco::prelude::Error::client("Missing parsed_document input"))?;

        // Extract the third field (calls table)
        match parsed_doc {
            Value::Struct(field_values) => {
                let calls = field_values
                    .fields
                    .get(2)
                    .ok_or_else(|| {
                        recoco::prelude::Error::client("Missing calls field in parsed_document")
                    })?
                    .clone();

                Ok(calls)
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

/// Build the schema for the output of ExtractCalls (just the calls table)
fn get_calls_output_schema() -> EnrichedValueType {
    EnrichedValueType {
        typ: ValueType::Table(TableSchema {
            kind: TableKind::LTable,
            row: match crate::conversion::call_type() {
                ValueType::Struct(s) => s,
                _ => unreachable!(),
            },
        }),
        nullable: false,
        attrs: Default::default(),
    }
}
