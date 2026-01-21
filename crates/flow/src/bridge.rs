// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

use async_trait::async_trait;
use thread_services::error::ServiceResult;
use thread_services::traits::{AnalyzerCapabilities, CodeAnalyzer};
use thread_services::types::{AnalysisContext, CrossFileRelationship, ParsedDocument};

/// Bridge: Implements thread-services traits using CocoIndex internals.
///
/// This struct decouples the service abstraction from the CocoIndex implementation.
pub struct CocoIndexAnalyzer {
    // Encapsulated CocoIndex internals
    // flow_ctx: Arc<FlowContext>,
}

impl CocoIndexAnalyzer {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl CodeAnalyzer for CocoIndexAnalyzer {
    fn capabilities(&self) -> AnalyzerCapabilities {
        AnalyzerCapabilities {
            supports_incremental: true,
            supports_cross_file: true,
            supports_deep_analysis: true,
            supported_languages: vec![], // TODO: Fill from available parsers
        }
    }

    async fn analyze_document(
        &self,
        document: &ParsedDocument<impl thread_ast_engine::source::Doc>,
        context: &AnalysisContext,
    ) -> ServiceResult<ParsedDocument<impl thread_ast_engine::source::Doc>> {
        // Bridge: Trigger a CocoIndex flow execution for single document
        Ok(ParsedDocument::new(
            document.ast_root.clone(),
            document.file_path.clone(),
            document.language,
            document.content_hash,
        ))
    }

    async fn analyze_cross_file_relationships(
        &self,
        _documents: &[ParsedDocument<impl thread_ast_engine::source::Doc>],
        _context: &AnalysisContext,
    ) -> ServiceResult<Vec<CrossFileRelationship>> {
        // Bridge: Query CocoIndex graph for relationships
        Ok(vec![])
    }
}
