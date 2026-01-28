// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

use recoco::base::spec::{ExecutionOptions, FlowInstanceSpec, IndexOptions, SourceRefreshOptions};
use recoco::builder::flow_builder::FlowBuilder;
use recoco::prelude::Error as RecocoError;
use serde_json::json;
use thread_services::error::{ServiceError, ServiceResult};

#[derive(Clone)]
struct SourceConfig {
    path: String,
    included: Vec<String>,
    excluded: Vec<String>,
}

#[derive(Clone)]
enum Step {
    Parse,
    ExtractSymbols,
    ExtractImports,
    ExtractCalls,
}

#[derive(Clone)]
enum Target {
    Postgres {
        table: String,
        primary_key: Vec<String>,
    },
    D1 {
        account_id: String,
        database_id: String,
        api_token: String,
        table: String,
        primary_key: Vec<String>,
    },
}

/// Builder for constructing standard Thread analysis pipelines.
///
/// This implements the Builder pattern to simplify the complexity of
/// constructing CocoIndex flows with multiple operators.
pub struct ThreadFlowBuilder {
    name: String,
    source: Option<SourceConfig>,
    steps: Vec<Step>,
    target: Option<Target>,
}

impl ThreadFlowBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            source: None,
            steps: Vec::new(),
            target: None,
        }
    }

    pub fn source_local(
        mut self,
        path: impl Into<String>,
        included: &[&str],
        excluded: &[&str],
    ) -> Self {
        self.source = Some(SourceConfig {
            path: path.into(),
            included: included.iter().map(|s| s.to_string()).collect(),
            excluded: excluded.iter().map(|s| s.to_string()).collect(),
        });
        self
    }

    pub fn parse(mut self) -> Self {
        self.steps.push(Step::Parse);
        self
    }

    pub fn extract_symbols(mut self) -> Self {
        self.steps.push(Step::ExtractSymbols);
        self
    }

    pub fn extract_imports(mut self) -> Self {
        self.steps.push(Step::ExtractImports);
        self
    }

    pub fn extract_calls(mut self) -> Self {
        self.steps.push(Step::ExtractCalls);
        self
    }

    pub fn target_postgres(mut self, table: impl Into<String>, primary_key: &[&str]) -> Self {
        self.target = Some(Target::Postgres {
            table: table.into(),
            primary_key: primary_key.iter().map(|s| s.to_string()).collect(),
        });
        self
    }

    /// Configure D1 as the export target
    ///
    /// # Arguments
    /// * `account_id` - Cloudflare account ID
    /// * `database_id` - D1 database ID
    /// * `api_token` - Cloudflare API token
    /// * `table` - Table name to export to
    /// * `primary_key` - Primary key field names for content-addressed deduplication
    pub fn target_d1(
        mut self,
        account_id: impl Into<String>,
        database_id: impl Into<String>,
        api_token: impl Into<String>,
        table: impl Into<String>,
        primary_key: &[&str],
    ) -> Self {
        self.target = Some(Target::D1 {
            account_id: account_id.into(),
            database_id: database_id.into(),
            api_token: api_token.into(),
            table: table.into(),
            primary_key: primary_key.iter().map(|s| s.to_string()).collect(),
        });
        self
    }

    pub async fn build(self) -> ServiceResult<FlowInstanceSpec> {
        let mut builder = FlowBuilder::new(&self.name)
            .await
            .map_err(|e: RecocoError| {
                ServiceError::execution_dynamic(format!("Failed to create builder: {}", e))
            })?;

        let source_cfg = self
            .source
            .ok_or_else(|| ServiceError::config_static("Missing source configuration"))?;

        // 1. SOURCE
        let source_node = builder
            .add_source(
                "local_file".to_string(),
                json!({
                    "path": source_cfg.path,
                    "included_patterns": source_cfg.included,
                    "excluded_patterns": source_cfg.excluded
                })
                .as_object()
                .ok_or_else(|| ServiceError::config_static("Invalid source spec"))?
                .clone(),
                None,
                "source".to_string(),
                Some(SourceRefreshOptions::default()),
                Some(ExecutionOptions::default()),
            )
            .await
            .map_err(|e: RecocoError| {
                ServiceError::execution_dynamic(format!("Failed to add source: {}", e))
            })?;

        let current_node = source_node;
        let mut parsed_node = None;

        for step in self.steps {
            match step {
                Step::Parse => {
                    // 2. TRANSFORM: Parse with Thread
                    let content_field = current_node
                        .field("content")
                        .map_err(|e| {
                            ServiceError::config_dynamic(format!("Missing content field: {}", e))
                        })?
                        .ok_or_else(|| ServiceError::config_static("Content field not found"))?;

                    // Attempt to get language field, fallback to path if needed or error
                    let language_field = current_node
                        .field("language")
                        .or_else(|_| current_node.field("path"))
                        .map_err(|e| {
                            ServiceError::config_dynamic(format!(
                                "Missing language/path field: {}",
                                e
                            ))
                        })?
                        .ok_or_else(|| {
                            ServiceError::config_static("Language/Path field not found")
                        })?;

                    let parsed = builder
                        .transform(
                            "thread_parse".to_string(),
                            serde_json::Map::new(),
                            vec![
                                (content_field, Some("content".to_string())),
                                (language_field, Some("language".to_string())),
                            ],
                            None,
                            "parsed".to_string(),
                        )
                        .await
                        .map_err(|e: RecocoError| {
                            ServiceError::execution_dynamic(format!(
                                "Failed to add parse step: {}",
                                e
                            ))
                        })?;

                    parsed_node = Some(parsed);
                }
                Step::ExtractSymbols => {
                    // 3. COLLECT: Symbols
                    let parsed = parsed_node.as_ref().ok_or_else(|| {
                        ServiceError::config_static("Extract symbols requires parse step first")
                    })?;

                    let mut root_scope = builder.root_scope();
                    let symbols_collector = root_scope
                        .add_collector("symbols".to_string())
                        .map_err(|e: RecocoError| {
                            ServiceError::execution_dynamic(format!(
                                "Failed to add collector: {}",
                                e
                            ))
                        })?;

                    // We need source node for file_path
                    let path_field = current_node
                        .field("path")
                        .map_err(|e| {
                            ServiceError::config_dynamic(format!("Missing path field: {}", e))
                        })?
                        .ok_or_else(|| ServiceError::config_static("Path field not found"))?;

                    let symbols = parsed
                        .field("symbols")
                        .map_err(|e| {
                            ServiceError::config_dynamic(format!(
                                "Missing symbols field in parsed output: {}",
                                e
                            ))
                        })?
                        .ok_or_else(|| ServiceError::config_static("Symbols field not found"))?;

                    // Get content_fingerprint field for content-addressed deduplication
                    let content_fingerprint = parsed
                        .field("content_fingerprint")
                        .map_err(|e| {
                            ServiceError::config_dynamic(format!(
                                "Missing content_fingerprint field in parsed output: {}",
                                e
                            ))
                        })?
                        .ok_or_else(|| {
                            ServiceError::config_static("Content fingerprint field not found")
                        })?;

                    builder
                        .collect(
                            &symbols_collector,
                            vec![
                                ("file_path".to_string(), path_field),
                                ("content_fingerprint".to_string(), content_fingerprint),
                                (
                                    "name".to_string(),
                                    symbols
                                        .field("name")
                                        .map_err(|e: RecocoError| {
                                            ServiceError::config_dynamic(e.to_string())
                                        })?
                                        .ok_or_else(|| {
                                            ServiceError::config_static(
                                                "Symbol Name field not found",
                                            )
                                        })?,
                                ),
                                (
                                    "kind".to_string(),
                                    symbols
                                        .field("kind")
                                        .map_err(|e: RecocoError| {
                                            ServiceError::config_dynamic(e.to_string())
                                        })?
                                        .ok_or_else(|| {
                                            ServiceError::config_static(
                                                "Symbol Kind field not found",
                                            )
                                        })?,
                                ),
                                (
                                    "signature".to_string(),
                                    symbols
                                        .field("scope")
                                        .map_err(|e: RecocoError| {
                                            ServiceError::config_dynamic(e.to_string())
                                        })?
                                        .ok_or_else(|| {
                                            ServiceError::config_static(
                                                "Symbol Scope field not found",
                                            )
                                        })?,
                                ),
                            ],
                            None,
                        )
                        .await
                        .map_err(|e: RecocoError| {
                            ServiceError::execution_dynamic(format!(
                                "Failed to configure collector: {}",
                                e
                            ))
                        })?;

                    // 4. EXPORT
                    if let Some(target_cfg) = &self.target {
                        match target_cfg {
                            Target::Postgres { table, primary_key } => {
                                builder
                                    .export(
                                        "symbols_table".to_string(),
                                        "postgres".to_string(), // target type name
                                        json!({
                                            "table": table,
                                            "primary_key": primary_key
                                        })
                                        .as_object()
                                        .ok_or_else(|| {
                                            ServiceError::config_static("Invalid target spec")
                                        })?
                                        .clone(),
                                        vec![],
                                        IndexOptions {
                                            primary_key_fields: Some(
                                                primary_key.iter().map(|s| s.to_string()).collect(),
                                            ),
                                            vector_indexes: vec![],
                                            fts_indexes: vec![],
                                        },
                                        &symbols_collector,
                                        false, // setup_by_user
                                    )
                                    .map_err(|e: RecocoError| {
                                        ServiceError::execution_dynamic(format!(
                                            "Failed to add export: {}",
                                            e
                                        ))
                                    })?;
                            }
                            Target::D1 {
                                account_id,
                                database_id,
                                api_token,
                                table,
                                primary_key,
                            } => {
                                builder
                                    .export(
                                        "symbols_table".to_string(),
                                        "d1".to_string(), // target type name matching D1TargetFactory::name()
                                        json!({
                                            "account_id": account_id,
                                            "database_id": database_id,
                                            "api_token": api_token,
                                            "table_name": table
                                        })
                                        .as_object()
                                        .ok_or_else(|| {
                                            ServiceError::config_static("Invalid target spec")
                                        })?
                                        .clone(),
                                        vec![],
                                        IndexOptions {
                                            primary_key_fields: Some(
                                                primary_key.iter().map(|s| s.to_string()).collect(),
                                            ),
                                            vector_indexes: vec![],
                                            fts_indexes: vec![],
                                        },
                                        &symbols_collector,
                                        false, // setup_by_user
                                    )
                                    .map_err(|e: RecocoError| {
                                        ServiceError::execution_dynamic(format!(
                                            "Failed to add export: {}",
                                            e
                                        ))
                                    })?;
                            }
                        }
                    }
                }
                Step::ExtractImports => {
                    // Similar to ExtractSymbols but for imports
                    let parsed = parsed_node.as_ref().ok_or_else(|| {
                        ServiceError::config_static("Extract imports requires parse step first")
                    })?;

                    let mut root_scope = builder.root_scope();
                    let imports_collector = root_scope
                        .add_collector("imports".to_string())
                        .map_err(|e: RecocoError| {
                            ServiceError::execution_dynamic(format!(
                                "Failed to add collector: {}",
                                e
                            ))
                        })?;

                    let path_field = current_node
                        .field("path")
                        .map_err(|e| {
                            ServiceError::config_dynamic(format!("Missing path field: {}", e))
                        })?
                        .ok_or_else(|| ServiceError::config_static("Path field not found"))?;

                    let imports = parsed
                        .field("imports")
                        .map_err(|e| {
                            ServiceError::config_dynamic(format!(
                                "Missing imports field in parsed output: {}",
                                e
                            ))
                        })?
                        .ok_or_else(|| ServiceError::config_static("Imports field not found"))?;

                    // Get content_fingerprint field for content-addressed deduplication
                    let content_fingerprint = parsed
                        .field("content_fingerprint")
                        .map_err(|e| {
                            ServiceError::config_dynamic(format!(
                                "Missing content_fingerprint field in parsed output: {}",
                                e
                            ))
                        })?
                        .ok_or_else(|| {
                            ServiceError::config_static("Content fingerprint field not found")
                        })?;

                    builder
                        .collect(
                            &imports_collector,
                            vec![
                                ("file_path".to_string(), path_field),
                                ("content_fingerprint".to_string(), content_fingerprint),
                                (
                                    "symbol_name".to_string(),
                                    imports
                                        .field("symbol_name")
                                        .map_err(|e: RecocoError| {
                                            ServiceError::config_dynamic(e.to_string())
                                        })?
                                        .ok_or_else(|| {
                                            ServiceError::config_static(
                                                "Import symbol_name field not found",
                                            )
                                        })?,
                                ),
                                (
                                    "source_path".to_string(),
                                    imports
                                        .field("source_path")
                                        .map_err(|e: RecocoError| {
                                            ServiceError::config_dynamic(e.to_string())
                                        })?
                                        .ok_or_else(|| {
                                            ServiceError::config_static(
                                                "Import source_path field not found",
                                            )
                                        })?,
                                ),
                                (
                                    "kind".to_string(),
                                    imports
                                        .field("kind")
                                        .map_err(|e: RecocoError| {
                                            ServiceError::config_dynamic(e.to_string())
                                        })?
                                        .ok_or_else(|| {
                                            ServiceError::config_static(
                                                "Import kind field not found",
                                            )
                                        })?,
                                ),
                            ],
                            None,
                        )
                        .await
                        .map_err(|e: RecocoError| {
                            ServiceError::execution_dynamic(format!(
                                "Failed to configure collector: {}",
                                e
                            ))
                        })?;

                    // Export if target configured
                    if let Some(target_cfg) = &self.target {
                        match target_cfg {
                            Target::Postgres { table, primary_key } => {
                                builder
                                    .export(
                                        "imports_table".to_string(),
                                        "postgres".to_string(),
                                        json!({
                                            "table": format!("{}_imports", table),
                                            "primary_key": primary_key
                                        })
                                        .as_object()
                                        .ok_or_else(|| {
                                            ServiceError::config_static("Invalid target spec")
                                        })?
                                        .clone(),
                                        vec![],
                                        IndexOptions {
                                            primary_key_fields: Some(
                                                primary_key.iter().map(|s| s.to_string()).collect(),
                                            ),
                                            vector_indexes: vec![],
                                            fts_indexes: vec![],
                                        },
                                        &imports_collector,
                                        false,
                                    )
                                    .map_err(|e: RecocoError| {
                                        ServiceError::execution_dynamic(format!(
                                            "Failed to add export: {}",
                                            e
                                        ))
                                    })?;
                            }
                            Target::D1 {
                                account_id,
                                database_id,
                                api_token,
                                table,
                                primary_key,
                            } => {
                                builder
                                    .export(
                                        "imports_table".to_string(),
                                        "d1".to_string(),
                                        json!({
                                            "account_id": account_id,
                                            "database_id": database_id,
                                            "api_token": api_token,
                                            "table_name": format!("{}_imports", table)
                                        })
                                        .as_object()
                                        .ok_or_else(|| {
                                            ServiceError::config_static("Invalid target spec")
                                        })?
                                        .clone(),
                                        vec![],
                                        IndexOptions {
                                            primary_key_fields: Some(
                                                primary_key.iter().map(|s| s.to_string()).collect(),
                                            ),
                                            vector_indexes: vec![],
                                            fts_indexes: vec![],
                                        },
                                        &imports_collector,
                                        false,
                                    )
                                    .map_err(|e: RecocoError| {
                                        ServiceError::execution_dynamic(format!(
                                            "Failed to add export: {}",
                                            e
                                        ))
                                    })?;
                            }
                        }
                    }
                }
                Step::ExtractCalls => {
                    // Similar to ExtractSymbols but for function calls
                    let parsed = parsed_node.as_ref().ok_or_else(|| {
                        ServiceError::config_static("Extract calls requires parse step first")
                    })?;

                    let mut root_scope = builder.root_scope();
                    let calls_collector = root_scope
                        .add_collector("calls".to_string())
                        .map_err(|e: RecocoError| {
                            ServiceError::execution_dynamic(format!(
                                "Failed to add collector: {}",
                                e
                            ))
                        })?;

                    let path_field = current_node
                        .field("path")
                        .map_err(|e| {
                            ServiceError::config_dynamic(format!("Missing path field: {}", e))
                        })?
                        .ok_or_else(|| ServiceError::config_static("Path field not found"))?;

                    let calls = parsed
                        .field("calls")
                        .map_err(|e| {
                            ServiceError::config_dynamic(format!(
                                "Missing calls field in parsed output: {}",
                                e
                            ))
                        })?
                        .ok_or_else(|| ServiceError::config_static("Calls field not found"))?;

                    // Get content_fingerprint field for content-addressed deduplication
                    let content_fingerprint = parsed
                        .field("content_fingerprint")
                        .map_err(|e| {
                            ServiceError::config_dynamic(format!(
                                "Missing content_fingerprint field in parsed output: {}",
                                e
                            ))
                        })?
                        .ok_or_else(|| {
                            ServiceError::config_static("Content fingerprint field not found")
                        })?;

                    builder
                        .collect(
                            &calls_collector,
                            vec![
                                ("file_path".to_string(), path_field),
                                ("content_fingerprint".to_string(), content_fingerprint),
                                (
                                    "function_name".to_string(),
                                    calls
                                        .field("function_name")
                                        .map_err(|e: RecocoError| {
                                            ServiceError::config_dynamic(e.to_string())
                                        })?
                                        .ok_or_else(|| {
                                            ServiceError::config_static(
                                                "Call function_name field not found",
                                            )
                                        })?,
                                ),
                                (
                                    "arguments_count".to_string(),
                                    calls
                                        .field("arguments_count")
                                        .map_err(|e: RecocoError| {
                                            ServiceError::config_dynamic(e.to_string())
                                        })?
                                        .ok_or_else(|| {
                                            ServiceError::config_static(
                                                "Call arguments_count field not found",
                                            )
                                        })?,
                                ),
                            ],
                            None,
                        )
                        .await
                        .map_err(|e: RecocoError| {
                            ServiceError::execution_dynamic(format!(
                                "Failed to configure collector: {}",
                                e
                            ))
                        })?;

                    // Export if target configured
                    if let Some(target_cfg) = &self.target {
                        match target_cfg {
                            Target::Postgres { table, primary_key } => {
                                builder
                                    .export(
                                        "calls_table".to_string(),
                                        "postgres".to_string(),
                                        json!({
                                            "table": format!("{}_calls", table),
                                            "primary_key": primary_key
                                        })
                                        .as_object()
                                        .ok_or_else(|| {
                                            ServiceError::config_static("Invalid target spec")
                                        })?
                                        .clone(),
                                        vec![],
                                        IndexOptions {
                                            primary_key_fields: Some(
                                                primary_key.iter().map(|s| s.to_string()).collect(),
                                            ),
                                            vector_indexes: vec![],
                                            fts_indexes: vec![],
                                        },
                                        &calls_collector,
                                        false,
                                    )
                                    .map_err(|e: RecocoError| {
                                        ServiceError::execution_dynamic(format!(
                                            "Failed to add export: {}",
                                            e
                                        ))
                                    })?;
                            }
                            Target::D1 {
                                account_id,
                                database_id,
                                api_token,
                                table,
                                primary_key,
                            } => {
                                builder
                                    .export(
                                        "calls_table".to_string(),
                                        "d1".to_string(),
                                        json!({
                                            "account_id": account_id,
                                            "database_id": database_id,
                                            "api_token": api_token,
                                            "table_name": format!("{}_calls", table)
                                        })
                                        .as_object()
                                        .ok_or_else(|| {
                                            ServiceError::config_static("Invalid target spec")
                                        })?
                                        .clone(),
                                        vec![],
                                        IndexOptions {
                                            primary_key_fields: Some(
                                                primary_key.iter().map(|s| s.to_string()).collect(),
                                            ),
                                            vector_indexes: vec![],
                                            fts_indexes: vec![],
                                        },
                                        &calls_collector,
                                        false,
                                    )
                                    .map_err(|e: RecocoError| {
                                        ServiceError::execution_dynamic(format!(
                                            "Failed to add export: {}",
                                            e
                                        ))
                                    })?;
                            }
                        }
                    }
                }
            }
        }

        let ctx = builder.build_flow().await.map_err(|e: RecocoError| {
            ServiceError::execution_dynamic(format!("Failed to build flow: {}", e))
        })?;

        Ok(ctx.0.flow.flow_instance.clone())
    }
}
