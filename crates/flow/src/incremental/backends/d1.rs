// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Cloudflare D1 storage backend for the incremental update system.
//!
//! Provides a SQLite-compatible backend for edge deployment on Cloudflare Workers.
//! Communicates with D1 via the Cloudflare REST API using HTTP.
//!
//! # Architecture
//!
//! The D1 backend follows the same HTTP API pattern as the existing D1 target
//! (`crates/flow/src/targets/d1.rs`). All queries are executed via the
//! Cloudflare D1 REST API using `reqwest`.
//!
//! # Performance Targets
//!
//! - Single operations: <50ms p95 latency (Constitutional Principle VI)
//! - Full graph load (1000 nodes): <200ms p95 latency
//!
//! # D1 API Pattern
//!
//! All queries are sent as JSON payloads to:
//! ```text
//! POST https://api.cloudflare.com/client/v4/accounts/{account_id}/d1/database/{database_id}/query
//! ```
//!
//! # Feature Gating
//!
//! This module is gated behind the `d1-backend` feature flag.
//!
//! # Example
//!
//! ```rust,ignore
//! use thread_flow::incremental::backends::d1::D1IncrementalBackend;
//!
//! let backend = D1IncrementalBackend::new(
//!     "account-id".to_string(),
//!     "database-id".to_string(),
//!     "api-token".to_string(),
//! ).expect("Failed to create D1 backend");
//!
//! backend.run_migrations().await.expect("Migration failed");
//! ```

use crate::incremental::graph::DependencyGraph;
use crate::incremental::storage::{StorageBackend, StorageError};
use crate::incremental::types::{
    AnalysisDefFingerprint, DependencyEdge, DependencyStrength, DependencyType, SymbolDependency,
    SymbolKind,
};
use async_trait::async_trait;
use recoco::utils::fingerprint::Fingerprint;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Cloudflare D1 storage backend for the incremental update system.
///
/// Uses the Cloudflare REST API to execute SQL queries against a D1 database.
/// All queries use parameterized statements for safety and performance.
///
/// # Connection Management
///
/// The backend uses a shared `reqwest::Client` with connection pooling
/// for efficient HTTP/2 multiplexing to the Cloudflare API.
///
/// # Thread Safety
///
/// This type is `Send + Sync` and can be shared across async tasks.
pub struct D1IncrementalBackend {
    /// Cloudflare account ID.
    account_id: String,
    /// D1 database identifier.
    database_id: String,
    /// Cloudflare API bearer token.
    api_token: String,
    /// Shared HTTP client with connection pooling.
    http_client: Arc<reqwest::Client>,
}

/// Response from the D1 REST API.
#[derive(serde::Deserialize)]
struct D1Response {
    success: bool,
    #[serde(default)]
    errors: Vec<D1Error>,
    #[serde(default)]
    result: Vec<D1QueryResult>,
}

/// A single error from the D1 API.
#[derive(serde::Deserialize)]
struct D1Error {
    message: String,
}

/// Result of a single query within a D1 response.
#[derive(serde::Deserialize)]
struct D1QueryResult {
    #[serde(default)]
    results: Vec<serde_json::Value>,
    #[serde(default)]
    meta: D1QueryMeta,
}

/// Metadata about a query execution.
#[derive(serde::Deserialize, Default)]
struct D1QueryMeta {
    #[serde(default)]
    changes: u64,
}

impl D1IncrementalBackend {
    /// Creates a new D1 backend with the given Cloudflare credentials.
    ///
    /// Initializes a shared HTTP client with connection pooling optimized
    /// for Cloudflare API communication.
    ///
    /// # Arguments
    ///
    /// * `account_id` - Cloudflare account ID.
    /// * `database_id` - D1 database identifier.
    /// * `api_token` - Cloudflare API bearer token.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::Backend`] if the HTTP client cannot be created.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let backend = D1IncrementalBackend::new(
    ///     "account-123".to_string(),
    ///     "db-456".to_string(),
    ///     "token-789".to_string(),
    /// )?;
    /// ```
    pub fn new(
        account_id: String,
        database_id: String,
        api_token: String,
    ) -> Result<Self, StorageError> {
        use std::time::Duration;

        let http_client = Arc::new(
            reqwest::Client::builder()
                .pool_max_idle_per_host(10)
                .pool_idle_timeout(Some(Duration::from_secs(90)))
                .tcp_keepalive(Some(Duration::from_secs(60)))
                .timeout(Duration::from_secs(30))
                .build()
                .map_err(|e| {
                    StorageError::Backend(format!("Failed to create HTTP client: {e}"))
                })?,
        );

        Ok(Self {
            account_id,
            database_id,
            api_token,
            http_client,
        })
    }

    /// Creates a new D1 backend with a pre-configured HTTP client.
    ///
    /// Useful for testing or when you want to share a client across
    /// multiple backends.
    ///
    /// # Arguments
    ///
    /// * `account_id` - Cloudflare account ID.
    /// * `database_id` - D1 database identifier.
    /// * `api_token` - Cloudflare API bearer token.
    /// * `http_client` - Pre-configured HTTP client.
    pub fn with_client(
        account_id: String,
        database_id: String,
        api_token: String,
        http_client: Arc<reqwest::Client>,
    ) -> Self {
        Self {
            account_id,
            database_id,
            api_token,
            http_client,
        }
    }

    /// Runs the D1 schema migration to create required tables and indexes.
    ///
    /// This is idempotent: running it multiple times has no effect if the
    /// schema already exists (uses `CREATE TABLE IF NOT EXISTS`).
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::Backend`] if the migration SQL fails to execute.
    pub async fn run_migrations(&self) -> Result<(), StorageError> {
        let migration_sql = include_str!("../../../migrations/d1_incremental_v1.sql");

        // D1 requires executing statements individually (no batch_execute).
        // Split on semicolons and execute each statement.
        for statement in migration_sql.split(';') {
            let trimmed = statement.trim();
            if trimmed.is_empty() || trimmed.starts_with("--") {
                continue;
            }
            self.execute_sql(trimmed, vec![]).await?;
        }

        Ok(())
    }

    /// Saves multiple dependency edges in a batch.
    ///
    /// More efficient than calling [`save_edge`](StorageBackend::save_edge)
    /// individually for each edge, as it reduces HTTP round-trips.
    ///
    /// # Arguments
    ///
    /// * `edges` - Slice of dependency edges to persist.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::Backend`] if any operation fails.
    pub async fn save_edges_batch(&self, edges: &[DependencyEdge]) -> Result<(), StorageError> {
        if edges.is_empty() {
            return Ok(());
        }

        let mut statements = Vec::with_capacity(edges.len());

        for edge in edges {
            let (sym_from, sym_to, sym_kind, strength) = extract_symbol_fields(&edge.symbol);

            let params = vec![
                serde_json::Value::String(edge.from.to_string_lossy().to_string()),
                serde_json::Value::String(edge.to.to_string_lossy().to_string()),
                serde_json::Value::String(edge.dep_type.to_string()),
                opt_string_to_json(sym_from),
                opt_string_to_json(sym_to),
                opt_string_to_json(sym_kind.as_deref()),
                opt_string_to_json(strength.as_deref()),
            ];

            statements.push((UPSERT_EDGE_SQL.to_string(), params));
        }

        self.execute_batch(statements).await
    }

    /// Returns the D1 API URL for this database.
    fn api_url(&self) -> String {
        format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/d1/database/{}/query",
            self.account_id, self.database_id
        )
    }

    /// Executes a single SQL statement against D1.
    async fn execute_sql(
        &self,
        sql: &str,
        params: Vec<serde_json::Value>,
    ) -> Result<D1QueryResult, StorageError> {
        let request_body = serde_json::json!({
            "sql": sql,
            "params": params
        });

        let response = self
            .http_client
            .post(self.api_url())
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| StorageError::Backend(format!("D1 API request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(StorageError::Backend(format!(
                "D1 API error ({status}): {error_text}"
            )));
        }

        let body: D1Response = response
            .json()
            .await
            .map_err(|e| StorageError::Backend(format!("Failed to parse D1 response: {e}")))?;

        if !body.success {
            let error_msgs: Vec<_> = body.errors.iter().map(|e| e.message.as_str()).collect();
            return Err(StorageError::Backend(format!(
                "D1 execution failed: {}",
                error_msgs.join("; ")
            )));
        }

        body.result
            .into_iter()
            .next()
            .ok_or_else(|| StorageError::Backend("D1 returned no result set".to_string()))
    }

    /// Executes multiple SQL statements sequentially.
    async fn execute_batch(
        &self,
        statements: Vec<(String, Vec<serde_json::Value>)>,
    ) -> Result<(), StorageError> {
        for (sql, params) in statements {
            self.execute_sql(&sql, params).await?;
        }
        Ok(())
    }
}

// ─── SQL Constants ──────────────────────────────────────────────────────────

const UPSERT_FINGERPRINT_SQL: &str = "\
    INSERT INTO analysis_fingerprints (file_path, content_fingerprint, last_analyzed, updated_at) \
    VALUES (?1, ?2, ?3, strftime('%s', 'now')) \
    ON CONFLICT (file_path) DO UPDATE SET \
        content_fingerprint = excluded.content_fingerprint, \
        last_analyzed = excluded.last_analyzed, \
        updated_at = strftime('%s', 'now')";

const SELECT_FINGERPRINT_SQL: &str = "\
    SELECT content_fingerprint, last_analyzed \
    FROM analysis_fingerprints WHERE file_path = ?1";

const DELETE_FINGERPRINT_SQL: &str = "\
    DELETE FROM analysis_fingerprints WHERE file_path = ?1";

const DELETE_SOURCE_FILES_SQL: &str = "\
    DELETE FROM source_files WHERE fingerprint_path = ?1";

const INSERT_SOURCE_FILE_SQL: &str = "\
    INSERT INTO source_files (fingerprint_path, source_path) VALUES (?1, ?2)";

const SELECT_SOURCE_FILES_SQL: &str = "\
    SELECT source_path FROM source_files WHERE fingerprint_path = ?1";

const UPSERT_EDGE_SQL: &str = "\
    INSERT INTO dependency_edges \
    (from_path, to_path, dep_type, symbol_from, symbol_to, symbol_kind, dependency_strength) \
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) \
    ON CONFLICT (from_path, to_path, dep_type) DO UPDATE SET \
        symbol_from = excluded.symbol_from, \
        symbol_to = excluded.symbol_to, \
        symbol_kind = excluded.symbol_kind, \
        dependency_strength = excluded.dependency_strength";

const SELECT_EDGES_FROM_SQL: &str = "\
    SELECT from_path, to_path, dep_type, \
           symbol_from, symbol_to, symbol_kind, dependency_strength \
    FROM dependency_edges WHERE from_path = ?1";

const SELECT_EDGES_TO_SQL: &str = "\
    SELECT from_path, to_path, dep_type, \
           symbol_from, symbol_to, symbol_kind, dependency_strength \
    FROM dependency_edges WHERE to_path = ?1";

const DELETE_EDGES_FOR_SQL: &str = "\
    DELETE FROM dependency_edges WHERE from_path = ?1 OR to_path = ?1";

const SELECT_ALL_FINGERPRINTS_SQL: &str = "\
    SELECT file_path, content_fingerprint, last_analyzed \
    FROM analysis_fingerprints";

const SELECT_ALL_SOURCE_FILES_SQL: &str = "\
    SELECT fingerprint_path, source_path \
    FROM source_files ORDER BY fingerprint_path";

const SELECT_ALL_EDGES_SQL: &str = "\
    SELECT from_path, to_path, dep_type, \
           symbol_from, symbol_to, symbol_kind, dependency_strength \
    FROM dependency_edges";

// ─── StorageBackend Implementation ──────────────────────────────────────────

#[async_trait]
impl StorageBackend for D1IncrementalBackend {
    async fn save_fingerprint(
        &self,
        file_path: &Path,
        fingerprint: &AnalysisDefFingerprint,
    ) -> Result<(), StorageError> {
        let fp_path = file_path.to_string_lossy().to_string();

        // Encode fingerprint bytes as base64 for JSON transport (D1 BLOB handling).
        let fp_b64 = bytes_to_b64(fingerprint.fingerprint.as_slice());

        // Upsert the fingerprint record.
        self.execute_sql(
            UPSERT_FINGERPRINT_SQL,
            vec![
                serde_json::Value::String(fp_path.clone()),
                serde_json::Value::String(fp_b64),
                match fingerprint.last_analyzed {
                    Some(ts) => serde_json::Value::Number(ts.into()),
                    None => serde_json::Value::Null,
                },
            ],
        )
        .await?;

        // Replace source files: delete existing, then insert new.
        self.execute_sql(
            DELETE_SOURCE_FILES_SQL,
            vec![serde_json::Value::String(fp_path.clone())],
        )
        .await?;

        for source in &fingerprint.source_files {
            let src_path = source.to_string_lossy().to_string();
            self.execute_sql(
                INSERT_SOURCE_FILE_SQL,
                vec![
                    serde_json::Value::String(fp_path.clone()),
                    serde_json::Value::String(src_path),
                ],
            )
            .await?;
        }

        Ok(())
    }

    async fn load_fingerprint(
        &self,
        file_path: &Path,
    ) -> Result<Option<AnalysisDefFingerprint>, StorageError> {
        let fp_path = file_path.to_string_lossy().to_string();

        // Load the fingerprint record.
        let result = self
            .execute_sql(
                SELECT_FINGERPRINT_SQL,
                vec![serde_json::Value::String(fp_path.clone())],
            )
            .await?;

        let Some(row) = result.results.into_iter().next() else {
            return Ok(None);
        };

        let fp_b64 = row["content_fingerprint"]
            .as_str()
            .ok_or_else(|| StorageError::Corruption("Missing content_fingerprint".to_string()))?;

        let fp_bytes = b64_to_bytes(fp_b64)?;
        let fingerprint = bytes_to_fingerprint(&fp_bytes)?;

        let last_analyzed = row["last_analyzed"].as_i64();

        // Load associated source files.
        let src_result = self
            .execute_sql(
                SELECT_SOURCE_FILES_SQL,
                vec![serde_json::Value::String(fp_path)],
            )
            .await?;

        let source_files: HashSet<PathBuf> = src_result
            .results
            .iter()
            .filter_map(|r| r["source_path"].as_str().map(|s| PathBuf::from(s)))
            .collect();

        Ok(Some(AnalysisDefFingerprint {
            source_files,
            fingerprint,
            last_analyzed,
        }))
    }

    async fn delete_fingerprint(&self, file_path: &Path) -> Result<bool, StorageError> {
        let fp_path = file_path.to_string_lossy().to_string();

        // CASCADE via foreign key will delete source_files entries.
        let result = self
            .execute_sql(
                DELETE_FINGERPRINT_SQL,
                vec![serde_json::Value::String(fp_path)],
            )
            .await?;

        Ok(result.meta.changes > 0)
    }

    async fn save_edge(&self, edge: &DependencyEdge) -> Result<(), StorageError> {
        let (sym_from, sym_to, sym_kind, strength) = extract_symbol_fields(&edge.symbol);

        self.execute_sql(
            UPSERT_EDGE_SQL,
            vec![
                serde_json::Value::String(edge.from.to_string_lossy().to_string()),
                serde_json::Value::String(edge.to.to_string_lossy().to_string()),
                serde_json::Value::String(edge.dep_type.to_string()),
                opt_string_to_json(sym_from),
                opt_string_to_json(sym_to),
                opt_string_to_json(sym_kind.as_deref()),
                opt_string_to_json(strength.as_deref()),
            ],
        )
        .await?;

        Ok(())
    }

    async fn load_edges_from(&self, file_path: &Path) -> Result<Vec<DependencyEdge>, StorageError> {
        let fp = file_path.to_string_lossy().to_string();

        let result = self
            .execute_sql(
                SELECT_EDGES_FROM_SQL,
                vec![serde_json::Value::String(fp)],
            )
            .await?;

        result.results.iter().map(json_to_edge).collect()
    }

    async fn load_edges_to(&self, file_path: &Path) -> Result<Vec<DependencyEdge>, StorageError> {
        let fp = file_path.to_string_lossy().to_string();

        let result = self
            .execute_sql(
                SELECT_EDGES_TO_SQL,
                vec![serde_json::Value::String(fp)],
            )
            .await?;

        result.results.iter().map(json_to_edge).collect()
    }

    async fn delete_edges_for(&self, file_path: &Path) -> Result<usize, StorageError> {
        let fp = file_path.to_string_lossy().to_string();

        let result = self
            .execute_sql(
                DELETE_EDGES_FOR_SQL,
                vec![serde_json::Value::String(fp)],
            )
            .await?;

        Ok(result.meta.changes as usize)
    }

    async fn load_full_graph(&self) -> Result<DependencyGraph, StorageError> {
        let mut graph = DependencyGraph::new();

        // Load all fingerprints.
        let fp_result = self.execute_sql(SELECT_ALL_FINGERPRINTS_SQL, vec![]).await?;

        // Load all source files.
        let src_result = self.execute_sql(SELECT_ALL_SOURCE_FILES_SQL, vec![]).await?;

        // Build source files map grouped by fingerprint_path.
        let mut source_map: std::collections::HashMap<String, HashSet<PathBuf>> =
            std::collections::HashMap::new();
        for row in &src_result.results {
            if let (Some(fp_path), Some(src_path)) =
                (row["fingerprint_path"].as_str(), row["source_path"].as_str())
            {
                source_map
                    .entry(fp_path.to_string())
                    .or_default()
                    .insert(PathBuf::from(src_path));
            }
        }

        // Reconstruct fingerprint nodes.
        for row in &fp_result.results {
            let file_path = row["file_path"]
                .as_str()
                .ok_or_else(|| StorageError::Corruption("Missing file_path".to_string()))?;

            let fp_b64 = row["content_fingerprint"]
                .as_str()
                .ok_or_else(|| {
                    StorageError::Corruption("Missing content_fingerprint".to_string())
                })?;

            let fp_bytes = b64_to_bytes(fp_b64)?;
            let fingerprint = bytes_to_fingerprint(&fp_bytes)?;
            let last_analyzed = row["last_analyzed"].as_i64();

            let source_files = source_map.remove(file_path).unwrap_or_default();

            let fp = AnalysisDefFingerprint {
                source_files,
                fingerprint,
                last_analyzed,
            };

            graph.nodes.insert(PathBuf::from(file_path), fp);
        }

        // Load all edges.
        let edge_result = self.execute_sql(SELECT_ALL_EDGES_SQL, vec![]).await?;

        for row in &edge_result.results {
            let edge = json_to_edge(row)?;
            graph.add_edge(edge);
        }

        Ok(graph)
    }

    async fn save_full_graph(&self, graph: &DependencyGraph) -> Result<(), StorageError> {
        // Clear existing data (order matters due to foreign keys).
        // D1 does not support TRUNCATE; use DELETE instead.
        self.execute_sql("DELETE FROM source_files", vec![]).await?;
        self.execute_sql("DELETE FROM dependency_edges", vec![])
            .await?;
        self.execute_sql("DELETE FROM analysis_fingerprints", vec![])
            .await?;

        // Save all fingerprints and their source files.
        for (path, fp) in &graph.nodes {
            let fp_path = path.to_string_lossy().to_string();
            let fp_b64 = bytes_to_b64(fp.fingerprint.as_slice());

            self.execute_sql(
                "INSERT INTO analysis_fingerprints \
                 (file_path, content_fingerprint, last_analyzed) \
                 VALUES (?1, ?2, ?3)",
                vec![
                    serde_json::Value::String(fp_path.clone()),
                    serde_json::Value::String(fp_b64),
                    match fp.last_analyzed {
                        Some(ts) => serde_json::Value::Number(ts.into()),
                        None => serde_json::Value::Null,
                    },
                ],
            )
            .await?;

            for source in &fp.source_files {
                let src_path = source.to_string_lossy().to_string();
                self.execute_sql(
                    INSERT_SOURCE_FILE_SQL,
                    vec![
                        serde_json::Value::String(fp_path.clone()),
                        serde_json::Value::String(src_path),
                    ],
                )
                .await?;
            }
        }

        // Save all edges.
        for edge in &graph.edges {
            let (sym_from, sym_to, sym_kind, strength) = extract_symbol_fields(&edge.symbol);

            self.execute_sql(
                UPSERT_EDGE_SQL,
                vec![
                    serde_json::Value::String(edge.from.to_string_lossy().to_string()),
                    serde_json::Value::String(edge.to.to_string_lossy().to_string()),
                    serde_json::Value::String(edge.dep_type.to_string()),
                    opt_string_to_json(sym_from),
                    opt_string_to_json(sym_to),
                    opt_string_to_json(sym_kind.as_deref()),
                    opt_string_to_json(strength.as_deref()),
                ],
            )
            .await?;
        }

        Ok(())
    }
}

// ─── Helper Functions ───────────────────────────────────────────────────────

/// Converts a JSON row from D1 to a [`DependencyEdge`].
fn json_to_edge(row: &serde_json::Value) -> Result<DependencyEdge, StorageError> {
    let from_path = row["from_path"]
        .as_str()
        .ok_or_else(|| StorageError::Corruption("Missing from_path".to_string()))?;

    let to_path = row["to_path"]
        .as_str()
        .ok_or_else(|| StorageError::Corruption("Missing to_path".to_string()))?;

    let dep_type_str = row["dep_type"]
        .as_str()
        .ok_or_else(|| StorageError::Corruption("Missing dep_type".to_string()))?;

    let dep_type = parse_dependency_type(dep_type_str)?;

    let symbol_from = row["symbol_from"].as_str().map(String::from);
    let symbol_to = row["symbol_to"].as_str().map(String::from);
    let symbol_kind = row["symbol_kind"].as_str().map(String::from);
    let strength = row["dependency_strength"].as_str().map(String::from);

    let symbol = match (symbol_from, symbol_to, symbol_kind, strength) {
        (Some(from), Some(to), Some(kind), Some(str_val)) => Some(SymbolDependency {
            from_symbol: from,
            to_symbol: to,
            kind: parse_symbol_kind(&kind)?,
            strength: parse_dependency_strength(&str_val)?,
        }),
        _ => None,
    };

    Ok(DependencyEdge {
        from: PathBuf::from(from_path),
        to: PathBuf::from(to_path),
        dep_type,
        symbol,
    })
}

/// Extracts symbol fields from an optional [`SymbolDependency`] for SQL binding.
fn extract_symbol_fields(
    symbol: &Option<SymbolDependency>,
) -> (Option<&str>, Option<&str>, Option<String>, Option<String>) {
    match symbol {
        Some(sym) => (
            Some(sym.from_symbol.as_str()),
            Some(sym.to_symbol.as_str()),
            Some(sym.kind.to_string()),
            Some(sym.strength.to_string()),
        ),
        None => (None, None, None, None),
    }
}

/// Converts an `Option<&str>` to a JSON value (String or Null).
fn opt_string_to_json(opt: Option<&str>) -> serde_json::Value {
    match opt {
        Some(s) => serde_json::Value::String(s.to_string()),
        None => serde_json::Value::Null,
    }
}

/// Encodes raw bytes as base64 for D1 BLOB transport.
fn bytes_to_b64(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// Decodes base64-encoded bytes from D1 BLOB transport.
fn b64_to_bytes(b64: &str) -> Result<Vec<u8>, StorageError> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(b64)
        .map_err(|e| StorageError::Corruption(format!("Invalid base64 fingerprint: {e}")))
}

/// Converts raw bytes to a [`Fingerprint`].
fn bytes_to_fingerprint(bytes: &[u8]) -> Result<Fingerprint, StorageError> {
    let arr: [u8; 16] = bytes.try_into().map_err(|_| {
        StorageError::Corruption(format!(
            "Fingerprint has invalid length: expected 16, got {}",
            bytes.len()
        ))
    })?;
    Ok(Fingerprint(arr))
}

/// Parses a string representation of [`DependencyType`].
fn parse_dependency_type(s: &str) -> Result<DependencyType, StorageError> {
    match s {
        "import" | "Import" => Ok(DependencyType::Import),
        "export" | "Export" => Ok(DependencyType::Export),
        "macro" | "Macro" => Ok(DependencyType::Macro),
        "type" | "Type" => Ok(DependencyType::Type),
        "trait" | "Trait" => Ok(DependencyType::Trait),
        other => Err(StorageError::Corruption(format!(
            "Unknown dependency type: {other}"
        ))),
    }
}

/// Parses a string representation of [`SymbolKind`].
fn parse_symbol_kind(s: &str) -> Result<SymbolKind, StorageError> {
    match s {
        "function" | "Function" => Ok(SymbolKind::Function),
        "class" | "Class" => Ok(SymbolKind::Class),
        "interface" | "Interface" => Ok(SymbolKind::Interface),
        "type_alias" | "TypeAlias" => Ok(SymbolKind::TypeAlias),
        "constant" | "Constant" => Ok(SymbolKind::Constant),
        "enum" | "Enum" => Ok(SymbolKind::Enum),
        "module" | "Module" => Ok(SymbolKind::Module),
        "macro" | "Macro" => Ok(SymbolKind::Macro),
        other => Err(StorageError::Corruption(format!(
            "Unknown symbol kind: {other}"
        ))),
    }
}

/// Parses a string representation of [`DependencyStrength`].
fn parse_dependency_strength(s: &str) -> Result<DependencyStrength, StorageError> {
    match s {
        "strong" | "Strong" => Ok(DependencyStrength::Strong),
        "weak" | "Weak" => Ok(DependencyStrength::Weak),
        other => Err(StorageError::Corruption(format!(
            "Unknown dependency strength: {other}"
        ))),
    }
}
