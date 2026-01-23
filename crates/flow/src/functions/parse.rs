// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

use async_trait::async_trait;
use cocoindex::base::value::Value;
use cocoindex::context::FlowInstanceContext;
use cocoindex::ops::interface::{
    SimpleFunctionBuildOutput, SimpleFunctionExecutor, SimpleFunctionFactory,
};
use std::sync::Arc;

/// Factory for creating the ThreadParseExecutor
pub struct ThreadParseFactory;

#[async_trait]
impl SimpleFunctionFactory for ThreadParseFactory {
    async fn build(
        self: Arc<Self>,
        _spec: serde_json::Value,
        _args: Vec<cocoindex::base::schema::OpArgSchema>,
        _context: Arc<FlowInstanceContext>,
    ) -> Result<SimpleFunctionBuildOutput, cocoindex::error::Error> {
        Ok(SimpleFunctionBuildOutput {
            executor: Box::pin(async {
                Ok(Box::new(ThreadParseExecutor) as Box<dyn SimpleFunctionExecutor>)
            }),
            output_type: crate::conversion::get_thread_parse_output_schema(),
            behavior_version: Some(1),
        })
    }
}

/// Adapter: Wraps Thread's imperative parsing in a CocoIndex executor
pub struct ThreadParseExecutor;

#[async_trait]
impl SimpleFunctionExecutor for ThreadParseExecutor {
    async fn evaluate(&self, input: Vec<Value>) -> Result<Value, cocoindex::error::Error> {
        // Input: [content, language, file_path]
        let content = input
            .get(0)
            .ok_or_else(|| cocoindex::error::Error::client("Missing content"))?
            .as_str()
            .map_err(|e| cocoindex::error::Error::client(e.to_string()))?;

        let lang_str = input
            .get(1)
            .ok_or_else(|| cocoindex::error::Error::client("Missing language"))?
            .as_str()
            .map_err(|e| cocoindex::error::Error::client(e.to_string()))?;

        let path_str = input
            .get(2)
            .and_then(|v| v.as_str().ok())
            .map(|v| v.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Resolve language
        // We assume lang_str is an extension or can be resolved by from_extension_str
        // If it's a full name, this might need adjustment, but usually extensions are passed.

        let lang = thread_language::from_extension_str(lang_str)
            .or_else(|| {
                // Try from_extension with a constructed path if lang_str is just extension
                let p = std::path::PathBuf::from(format!("dummy.{}", lang_str));
                thread_language::from_extension(&p)
            })
            .ok_or_else(|| {
                cocoindex::error::Error::client(format!("Unsupported language: {}", lang_str))
            })?;

        // Parse with Thread
        use thread_ast_engine::tree_sitter::LanguageExt;
        let root = lang.ast_grep(content);

        // Compute hash
        let hash = thread_services::conversion::compute_content_hash(content, None);

        // Convert to ParsedDocument
        let path = std::path::PathBuf::from(&path_str);
        let mut doc = thread_services::conversion::root_to_parsed_document(root, path, lang, hash);

        // Extract metadata
        thread_services::conversion::extract_basic_metadata(&doc)
            .map(|metadata| {
                doc.metadata = metadata;
            })
            .map_err(|e| {
                cocoindex::error::Error::internal_msg(format!("Extraction error: {}", e))
            })?;

        // Extract symbols (CodeAnalyzer::extract_symbols is what the plan mentioned, but conversion::extract_basic_metadata does it)

        // Serialize
        use crate::conversion::serialize_parsed_doc;
        serialize_parsed_doc(&doc)
    }

    fn enable_cache(&self) -> bool {
        true
    }

    fn timeout(&self) -> Option<std::time::Duration> {
        Some(std::time::Duration::from_secs(30))
    }
}
