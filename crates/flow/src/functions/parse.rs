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
        _context: Arc<FlowInstanceContext>,
    ) -> Result<SimpleFunctionBuildOutput, cocoindex::error::Error> {
        Ok(SimpleFunctionBuildOutput {
            executor: Arc::new(ThreadParseExecutor),
            output_value_type: crate::conversion::build_output_schema(),
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
        // Input: [content, language, file_path]
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

        let path_str = input
            .get(2)
            .map(|v| v.as_str().unwrap_or("unknown"))
            .unwrap_or("unknown");

        // Resolve language
        // We assume lang_str is an extension or can be resolved by from_extension_str
        // If it's a full name, this might need adjustment, but usually extensions are passed.
        use thread_language::SupportLang;
        let lang = thread_language::from_extension_str(lang_str)
            .or_else(|| {
                // Try from_extension with a constructed path if lang_str is just extension
                let p = std::path::PathBuf::from(format!("dummy.{}", lang_str));
                thread_language::from_extension(&p)
            })
            .ok_or_else(|| {
                cocoindex::error::Error::msg(format!("Unsupported language: {}", lang_str))
            })?;

        // Parse with Thread
        use thread_ast_engine::tree_sitter::LanguageExt;
        let root = lang.ast_grep(content);

        // Compute hash
        let hash = thread_services::conversion::compute_content_hash(content, None);

        // Convert to ParsedDocument
        let path = std::path::PathBuf::from(path_str);
        let mut doc = thread_services::conversion::root_to_parsed_document(root, path, lang, hash);

        // Extract metadata
        thread_services::conversion::extract_basic_metadata(&doc)
            .map(|metadata| {
                doc.metadata = metadata;
            })
            .map_err(|e| cocoindex::error::Error::msg(format!("Extraction error: {}", e)))?;

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
