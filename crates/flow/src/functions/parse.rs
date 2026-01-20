// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

use async_trait::async_trait;
use cocoindex::base::value::Value;
use cocoindex::context::FlowInstanceContext;
use cocoindex::ops::interface::{
    SimpleFunctionBuildOutput, SimpleFunctionExecutor, SimpleFunctionFactory,
};
use std::sync::Arc;
use thread_ast_engine::{Language, parse};
use thread_services::error::ServiceResult;

/// Factory for creating the ThreadParseExecutor
pub struct ThreadParseFactory;

#[async_trait]
impl SimpleFunctionFactory for ThreadParseFactory {
    async fn build(
        self: Arc<Self>,
        _spec: serde_json::Value,
        _context: Arc<FlowInstanceContext>,
    ) -> Result<SimpleFunctionBuildOutput, cocoindex::error::Error> {
        Ok(SimpleFunctionBuildOutput {
            executor: Arc::new(ThreadParseExecutor),
            // TODO: Define output schema
            output_value_type: cocoindex::base::schema::EnrichedValueType::Json,
            enable_cache: true,
            timeout: None,
        })
    }
}

/// Adapter: Wraps Thread's imperative parsing in a CocoIndex executor
pub struct ThreadParseExecutor;

#[async_trait]
impl SimpleFunctionExecutor for ThreadParseExecutor {
    async fn evaluate(&self, input: Vec<Value>) -> Result<Value, cocoindex::error::Error> {
        // Input: [content, language]
        let content = input
            .get(0)
            .ok_or_else(|| cocoindex::error::Error::msg("Missing content"))?
            .as_str()
            .map_err(|e| cocoindex::error::Error::msg(e.to_string()))?;

        let lang_str = input
            .get(1)
            .ok_or_else(|| cocoindex::error::Error::msg("Missing language"))?
            .as_str()
            .map_err(|e| cocoindex::error::Error::msg(e.to_string()))?;

        // Adapt: Call Thread's internal logic
        // Note: Real implementation needs strict error mapping
        // let lang = Language::from_str(lang_str).map_err(...)

        // Placeholder for actual parsing logic integration
        // let doc = thread_ast_engine::parse(content, lang)?;

        // Adapt: Convert Thread Doc -> CocoIndex Value
        // serialize_doc(doc)

        Ok(Value::Json(serde_json::json!({
            "status": "parsed",
            "language": lang_str,
            "length": content.len()
        })))
    }

    fn enable_cache(&self) -> bool {
        true
    }

    fn timeout(&self) -> Option<std::time::Duration> {
        Some(std::time::Duration::from_secs(30))
    }
}
