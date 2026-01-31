// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

use async_trait::async_trait;
use recoco::base::value::Value;
use recoco::ops::factory_bases::SimpleFunctionFactoryBase;
use recoco::ops::interface::{FlowInstanceContext, SimpleFunctionExecutor};
use recoco::ops::sdk::{OpArgsResolver, SimpleFunctionAnalysisOutput};
use serde::Deserialize;
use std::sync::Arc;

/// Factory for creating the ThreadParseExecutor
pub struct ThreadParseFactory;

/// Spec for thread_parse operator (empty - uses default args)
#[derive(Debug, Clone, Deserialize)]
pub struct ThreadParseSpec {}

#[async_trait]
impl SimpleFunctionFactoryBase for ThreadParseFactory {
    type Spec = ThreadParseSpec;
    type ResolvedArgs = ();

    fn name(&self) -> &str {
        "thread_parse"
    }

    async fn analyze<'a>(
        &'a self,
        _spec: &'a Self::Spec,
        _args_resolver: &mut OpArgsResolver<'a>,
        _context: &FlowInstanceContext,
    ) -> Result<SimpleFunctionAnalysisOutput<Self::ResolvedArgs>, recoco::prelude::Error> {
        Ok(SimpleFunctionAnalysisOutput {
            resolved_args: (),
            output_schema: crate::conversion::get_thread_parse_output_schema(),
            behavior_version: Some(1),
        })
    }

    async fn build_executor(
        self: Arc<Self>,
        _spec: Self::Spec,
        _resolved_args: Self::ResolvedArgs,
        _context: Arc<FlowInstanceContext>,
    ) -> Result<impl SimpleFunctionExecutor, recoco::prelude::Error> {
        Ok(ThreadParseExecutor)
    }
}

/// Adapter: Wraps Thread's imperative parsing in a ReCoco executor
pub struct ThreadParseExecutor;

#[async_trait]
impl SimpleFunctionExecutor for ThreadParseExecutor {
    async fn evaluate(&self, input: Vec<Value>) -> Result<Value, recoco::prelude::Error> {
        // Input: [content, language, file_path]
        let content = input
            .first()
            .ok_or_else(|| recoco::prelude::Error::client("Missing content"))?
            .as_str()
            .map_err(|e| recoco::prelude::Error::client(e.to_string()))?;

        let lang_str = input
            .get(1)
            .ok_or_else(|| recoco::prelude::Error::client("Missing language"))?
            .as_str()
            .map_err(|e| recoco::prelude::Error::client(e.to_string()))?;

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
                recoco::prelude::Error::client(format!("Unsupported language: {}", lang_str))
            })?;

        // Parse with Thread
        use thread_ast_engine::tree_sitter::LanguageExt;
        let root = lang.ast_grep(content);

        // Compute content fingerprint using ReCoco's blake3-based system
        let fingerprint = thread_services::conversion::compute_content_fingerprint(content);

        // Convert to ParsedDocument
        let path = std::path::PathBuf::from(&path_str);
        let mut doc =
            thread_services::conversion::root_to_parsed_document(root, path, lang, fingerprint);

        // Extract metadata
        thread_services::conversion::extract_basic_metadata(&doc)
            .map(|metadata| {
                doc.metadata = metadata;
            })
            .map_err(|e| {
                recoco::prelude::Error::internal_msg(format!("Extraction error: {}", e))
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
