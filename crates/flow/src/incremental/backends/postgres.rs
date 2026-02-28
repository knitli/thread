// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! PostgreSQL storage backend for the incremental update system.
//!
//! Provides a full-featured SQL backend for CLI deployment with:
//!
//! - **Connection pooling** via `deadpool-postgres` for concurrent access
//! - **Prepared statements** for query plan caching and performance
//! - **Batch operations** with transactional atomicity
//! - **Upsert semantics** for idempotent fingerprint and edge updates
//!
//! # Performance Targets
//!
//! - Single operations: <10ms p95 latency (Constitutional Principle VI)
//! - Full graph load (1000 nodes): <50ms p95 latency
//!
//! # Example
//!
//! ```rust,ignore
//! use thread_flow::incremental::backends::postgres::PostgresIncrementalBackend;
//!
//! let backend = PostgresIncrementalBackend::new("postgresql://localhost/thread")
//!     .await
//!     .expect("Failed to connect to Postgres");
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
use deadpool_postgres::{Config, Pool, Runtime};
use recoco::utils::fingerprint::Fingerprint;
use std::path::{Path, PathBuf};
use thread_utils::RapidSet;
use tokio_postgres::NoTls;

/// PostgreSQL storage backend for the incremental update system.
///
/// Uses `deadpool-postgres` for connection pooling and `tokio-postgres` for
/// async query execution. All queries use prepared statements for optimal
/// query plan caching.
///
/// # Connection Management
///
/// The backend manages a pool of connections. The default pool size is 16
/// connections, configurable via the connection URL or pool configuration.
///
/// # Thread Safety
///
/// This type is `Send + Sync` and can be shared across async tasks.
#[derive(Debug)]
pub struct PostgresIncrementalBackend {
    pool: Pool,
}

impl PostgresIncrementalBackend {
    /// Creates a new Postgres backend connected to the given database URL.
    ///
    /// The URL should be a standard PostgreSQL connection string:
    /// `postgresql://user:password@host:port/database`
    ///
    /// # Arguments
    ///
    /// * `database_url` - PostgreSQL connection string.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::Backend`] if the connection pool cannot be created.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let backend = PostgresIncrementalBackend::new("postgresql://localhost/thread").await?;
    /// ```
    pub async fn new(database_url: &str) -> Result<Self, StorageError> {
        let pg_config = database_url
            .parse::<tokio_postgres::Config>()
            .map_err(|e| StorageError::Backend(format!("Invalid database URL: {e}")))?;

        let mut cfg = Config::new();
        // Extract config from parsed URL
        if let Some(hosts) = pg_config.get_hosts().first() {
            match hosts {
                tokio_postgres::config::Host::Tcp(h) => cfg.host = Some(h.clone()),
                #[cfg(unix)]
                tokio_postgres::config::Host::Unix(p) => {
                    cfg.host = Some(p.to_string_lossy().to_string());
                }
            }
        }
        if let Some(ports) = pg_config.get_ports().first() {
            cfg.port = Some(*ports);
        }
        if let Some(user) = pg_config.get_user() {
            cfg.user = Some(user.to_string());
        }
        if let Some(password) = pg_config.get_password() {
            cfg.password = Some(String::from_utf8_lossy(password).to_string());
        }
        if let Some(dbname) = pg_config.get_dbname() {
            cfg.dbname = Some(dbname.to_string());
        }

        let pool = cfg
            .create_pool(Some(Runtime::Tokio1), NoTls)
            .map_err(|e| StorageError::Backend(format!("Failed to create connection pool: {e}")))?;

        // Verify connectivity by acquiring and releasing a connection
        let _conn = pool
            .get()
            .await
            .map_err(|e| StorageError::Backend(format!("Failed to connect to database: {e}")))?;

        Ok(Self { pool })
    }

    /// Creates a new Postgres backend from an existing connection pool.
    ///
    /// Useful for testing or when you want to configure the pool externally.
    ///
    /// # Arguments
    ///
    /// * `pool` - A pre-configured `deadpool-postgres` connection pool.
    pub fn from_pool(pool: Pool) -> Self {
        Self { pool }
    }

    /// Runs the schema migration to create required tables and indexes.
    ///
    /// This is idempotent: running it multiple times has no effect if the
    /// schema already exists (uses `CREATE TABLE IF NOT EXISTS`).
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::Backend`] if the migration SQL fails to execute.
    pub async fn run_migrations(&self) -> Result<(), StorageError> {
        let client = self.pool.get().await.map_err(pg_pool_error)?;

        let migration_sql = include_str!("../../../migrations/incremental_system_v1.sql");
        client
            .batch_execute(migration_sql)
            .await
            .map_err(|e| StorageError::Backend(format!("Migration failed: {e}")))?;

        Ok(())
    }

    /// Saves multiple dependency edges in a single transaction.
    ///
    /// This is more efficient than calling [`save_edge`](StorageBackend::save_edge)
    /// individually for each edge, as it reduces round-trips to the database.
    ///
    /// # Arguments
    ///
    /// * `edges` - Slice of dependency edges to persist.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::Backend`] if the transaction fails.
    /// The transaction is rolled back on any error.
    pub async fn save_edges_batch(&self, edges: &[DependencyEdge]) -> Result<(), StorageError> {
        if edges.is_empty() {
            return Ok(());
        }

        let mut client = self.pool.get().await.map_err(pg_pool_error)?;

        // Execute in a transaction for atomicity
        let txn = client.transaction().await.map_err(pg_error)?;

        let stmt = txn
            .prepare(
                "INSERT INTO dependency_edges \
                 (from_path, to_path, dep_type, symbol_from, symbol_to, symbol_kind, dependency_strength) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7) \
                 ON CONFLICT (from_path, to_path, dep_type) DO UPDATE SET \
                     symbol_from = EXCLUDED.symbol_from, \
                     symbol_to = EXCLUDED.symbol_to, \
                     symbol_kind = EXCLUDED.symbol_kind, \
                     dependency_strength = EXCLUDED.dependency_strength",
            )
            .await
            .map_err(pg_error)?;

        for edge in edges {
            let (sym_from, sym_to, sym_kind, strength) = match &edge.symbol {
                Some(sym) => (
                    Some(sym.from_symbol.as_str()),
                    Some(sym.to_symbol.as_str()),
                    Some(sym.kind.to_string()),
                    Some(sym.strength.to_string()),
                ),
                None => (None, None, None, None),
            };

            txn.execute(
                &stmt,
                &[
                    &edge.from.to_string_lossy().as_ref(),
                    &edge.to.to_string_lossy().as_ref(),
                    &edge.dep_type.to_string(),
                    &sym_from,
                    &sym_to,
                    &sym_kind.as_deref(),
                    &strength.as_deref(),
                ],
            )
            .await
            .map_err(pg_error)?;
        }

        txn.commit().await.map_err(pg_error)?;

        Ok(())
    }
}

#[async_trait]
impl StorageBackend for PostgresIncrementalBackend {
    async fn save_fingerprint(
        &self,
        file_path: &Path,
        fingerprint: &AnalysisDefFingerprint,
    ) -> Result<(), StorageError> {
        let mut client = self.pool.get().await.map_err(pg_pool_error)?;

        let txn = client.transaction().await.map_err(pg_error)?;

        // Upsert the fingerprint record
        let stmt = txn
            .prepare(
                "INSERT INTO analysis_fingerprints (file_path, content_fingerprint, last_analyzed) \
                 VALUES ($1, $2, $3) \
                 ON CONFLICT (file_path) DO UPDATE SET \
                     content_fingerprint = EXCLUDED.content_fingerprint, \
                     last_analyzed = EXCLUDED.last_analyzed",
            )
            .await
            .map_err(pg_error)?;

        let fp_path = file_path.to_string_lossy();
        let fp_bytes = fingerprint.fingerprint.as_slice();

        txn.execute(
            &stmt,
            &[&fp_path.as_ref(), &fp_bytes, &fingerprint.last_analyzed],
        )
        .await
        .map_err(pg_error)?;

        // Replace source files: delete existing, then insert new
        let del_stmt = txn
            .prepare("DELETE FROM source_files WHERE fingerprint_path = $1")
            .await
            .map_err(pg_error)?;

        txn.execute(&del_stmt, &[&fp_path.as_ref()])
            .await
            .map_err(pg_error)?;

        if !fingerprint.source_files.is_empty() {
            let ins_stmt = txn
                .prepare("INSERT INTO source_files (fingerprint_path, source_path) VALUES ($1, $2)")
                .await
                .map_err(pg_error)?;

            for source in &fingerprint.source_files {
                let src_path = source.to_string_lossy();
                txn.execute(&ins_stmt, &[&fp_path.as_ref(), &src_path.as_ref()])
                    .await
                    .map_err(pg_error)?;
            }
        }

        txn.commit().await.map_err(pg_error)?;

        Ok(())
    }

    async fn load_fingerprint(
        &self,
        file_path: &Path,
    ) -> Result<Option<AnalysisDefFingerprint>, StorageError> {
        let client = self.pool.get().await.map_err(pg_pool_error)?;

        let fp_path = file_path.to_string_lossy();

        // Load the fingerprint record
        let stmt = client
            .prepare(
                "SELECT content_fingerprint, last_analyzed \
                 FROM analysis_fingerprints WHERE file_path = $1",
            )
            .await
            .map_err(pg_error)?;

        let row = client
            .query_opt(&stmt, &[&fp_path.as_ref()])
            .await
            .map_err(pg_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let fp_bytes: Vec<u8> = row.get(0);
        let last_analyzed: Option<i64> = row.get(1);

        let fingerprint = bytes_to_fingerprint(&fp_bytes)?;

        // Load associated source files
        let src_stmt = client
            .prepare("SELECT source_path FROM source_files WHERE fingerprint_path = $1")
            .await
            .map_err(pg_error)?;

        let src_rows = client
            .query(&src_stmt, &[&fp_path.as_ref()])
            .await
            .map_err(pg_error)?;

        let source_files: RapidSet<PathBuf> = src_rows
            .iter()
            .map(|r| {
                let s: String = r.get(0);
                PathBuf::from(s)
            })
            .collect();

        Ok(Some(AnalysisDefFingerprint {
            source_files,
            fingerprint,
            last_analyzed,
        }))
    }

    async fn delete_fingerprint(&self, file_path: &Path) -> Result<bool, StorageError> {
        let client = self.pool.get().await.map_err(pg_pool_error)?;

        let fp_path = file_path.to_string_lossy();

        // CASCADE will delete source_files entries automatically
        let stmt = client
            .prepare("DELETE FROM analysis_fingerprints WHERE file_path = $1")
            .await
            .map_err(pg_error)?;

        let count = client
            .execute(&stmt, &[&fp_path.as_ref()])
            .await
            .map_err(pg_error)?;

        Ok(count > 0)
    }

    async fn save_edge(&self, edge: &DependencyEdge) -> Result<(), StorageError> {
        let client = self.pool.get().await.map_err(pg_pool_error)?;

        let (sym_from, sym_to, sym_kind, strength) = match &edge.symbol {
            Some(sym) => (
                Some(sym.from_symbol.clone()),
                Some(sym.to_symbol.clone()),
                Some(sym.kind.to_string()),
                Some(sym.strength.to_string()),
            ),
            None => (None, None, None, None),
        };

        let stmt = client
            .prepare(
                "INSERT INTO dependency_edges \
                 (from_path, to_path, dep_type, symbol_from, symbol_to, symbol_kind, dependency_strength) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7) \
                 ON CONFLICT (from_path, to_path, dep_type) DO UPDATE SET \
                     symbol_from = EXCLUDED.symbol_from, \
                     symbol_to = EXCLUDED.symbol_to, \
                     symbol_kind = EXCLUDED.symbol_kind, \
                     dependency_strength = EXCLUDED.dependency_strength",
            )
            .await
            .map_err(pg_error)?;

        client
            .execute(
                &stmt,
                &[
                    &edge.from.to_string_lossy().as_ref(),
                    &edge.to.to_string_lossy().as_ref(),
                    &edge.dep_type.to_string(),
                    &sym_from.as_deref(),
                    &sym_to.as_deref(),
                    &sym_kind.as_deref(),
                    &strength.as_deref(),
                ],
            )
            .await
            .map_err(pg_error)?;

        Ok(())
    }

    async fn load_edges_from(&self, file_path: &Path) -> Result<Vec<DependencyEdge>, StorageError> {
        let client = self.pool.get().await.map_err(pg_pool_error)?;

        let stmt = client
            .prepare(
                "SELECT from_path, to_path, dep_type, \
                        symbol_from, symbol_to, symbol_kind, dependency_strength \
                 FROM dependency_edges WHERE from_path = $1",
            )
            .await
            .map_err(pg_error)?;

        let fp = file_path.to_string_lossy();
        let rows = client
            .query(&stmt, &[&fp.as_ref()])
            .await
            .map_err(pg_error)?;

        rows.iter().map(row_to_edge).collect()
    }

    async fn load_edges_to(&self, file_path: &Path) -> Result<Vec<DependencyEdge>, StorageError> {
        let client = self.pool.get().await.map_err(pg_pool_error)?;

        let stmt = client
            .prepare(
                "SELECT from_path, to_path, dep_type, \
                        symbol_from, symbol_to, symbol_kind, dependency_strength \
                 FROM dependency_edges WHERE to_path = $1",
            )
            .await
            .map_err(pg_error)?;

        let fp = file_path.to_string_lossy();
        let rows = client
            .query(&stmt, &[&fp.as_ref()])
            .await
            .map_err(pg_error)?;

        rows.iter().map(row_to_edge).collect()
    }

    async fn delete_edges_for(&self, file_path: &Path) -> Result<usize, StorageError> {
        let client = self.pool.get().await.map_err(pg_pool_error)?;

        let fp = file_path.to_string_lossy();

        let stmt = client
            .prepare("DELETE FROM dependency_edges WHERE from_path = $1 OR to_path = $1")
            .await
            .map_err(pg_error)?;

        let count = client
            .execute(&stmt, &[&fp.as_ref()])
            .await
            .map_err(pg_error)?;

        Ok(count as usize)
    }

    async fn load_full_graph(&self) -> Result<DependencyGraph, StorageError> {
        let client = self.pool.get().await.map_err(pg_pool_error)?;

        let mut graph = DependencyGraph::new();

        // Load all fingerprints with their source files
        let fp_stmt = client
            .prepare(
                "SELECT f.file_path, f.content_fingerprint, f.last_analyzed \
                 FROM analysis_fingerprints f",
            )
            .await
            .map_err(pg_error)?;

        let fp_rows = client.query(&fp_stmt, &[]).await.map_err(pg_error)?;

        let src_stmt = client
            .prepare(
                "SELECT fingerprint_path, source_path FROM source_files ORDER BY fingerprint_path",
            )
            .await
            .map_err(pg_error)?;

        let src_rows = client.query(&src_stmt, &[]).await.map_err(pg_error)?;

        // Build source files map grouped by fingerprint_path
        let mut source_map: thread_utils::RapidMap<String, RapidSet<PathBuf>> =
            thread_utils::get_map();
        for row in &src_rows {
            let fp_path: String = row.get(0);
            let src_path: String = row.get(1);
            source_map
                .entry(fp_path)
                .or_default()
                .insert(PathBuf::from(src_path));
        }

        // Reconstruct fingerprint nodes
        for row in &fp_rows {
            let file_path: String = row.get(0);
            let fp_bytes: Vec<u8> = row.get(1);
            let last_analyzed: Option<i64> = row.get(2);

            let fingerprint = bytes_to_fingerprint(&fp_bytes)?;
            let source_files = source_map.remove(&file_path).unwrap_or_default();

            let fp = AnalysisDefFingerprint {
                source_files,
                fingerprint,
                last_analyzed,
            };

            graph.nodes.insert(PathBuf::from(&file_path), fp);
        }

        // Load all edges
        let edge_stmt = client
            .prepare(
                "SELECT from_path, to_path, dep_type, \
                        symbol_from, symbol_to, symbol_kind, dependency_strength \
                 FROM dependency_edges",
            )
            .await
            .map_err(pg_error)?;

        let edge_rows = client.query(&edge_stmt, &[]).await.map_err(pg_error)?;

        for row in &edge_rows {
            let edge = row_to_edge(row)?;
            graph.add_edge(edge);
        }

        Ok(graph)
    }

    async fn save_full_graph(&self, graph: &DependencyGraph) -> Result<(), StorageError> {
        let mut client = self.pool.get().await.map_err(pg_pool_error)?;

        let txn = client.transaction().await.map_err(pg_error)?;

        // Clear existing data (order matters due to foreign keys)
        txn.batch_execute(
            "DELETE FROM source_files; \
             DELETE FROM dependency_edges; \
             DELETE FROM analysis_fingerprints;",
        )
        .await
        .map_err(pg_error)?;

        // Save all fingerprints
        let fp_stmt = txn
            .prepare(
                "INSERT INTO analysis_fingerprints (file_path, content_fingerprint, last_analyzed) \
                 VALUES ($1, $2, $3)",
            )
            .await
            .map_err(pg_error)?;

        let src_stmt = txn
            .prepare("INSERT INTO source_files (fingerprint_path, source_path) VALUES ($1, $2)")
            .await
            .map_err(pg_error)?;

        for (path, fp) in &graph.nodes {
            let fp_path = path.to_string_lossy();
            let fp_bytes = fp.fingerprint.as_slice();

            txn.execute(&fp_stmt, &[&fp_path.as_ref(), &fp_bytes, &fp.last_analyzed])
                .await
                .map_err(pg_error)?;

            for source in &fp.source_files {
                let src_path = source.to_string_lossy();
                txn.execute(&src_stmt, &[&fp_path.as_ref(), &src_path.as_ref()])
                    .await
                    .map_err(pg_error)?;
            }
        }

        // Save all edges
        let edge_stmt = txn
            .prepare(
                "INSERT INTO dependency_edges \
                 (from_path, to_path, dep_type, symbol_from, symbol_to, symbol_kind, dependency_strength) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7) \
                 ON CONFLICT (from_path, to_path, dep_type) DO NOTHING",
            )
            .await
            .map_err(pg_error)?;

        for edge in &graph.edges {
            let (sym_from, sym_to, sym_kind, strength) = match &edge.symbol {
                Some(sym) => (
                    Some(sym.from_symbol.clone()),
                    Some(sym.to_symbol.clone()),
                    Some(sym.kind.to_string()),
                    Some(sym.strength.to_string()),
                ),
                None => (None, None, None, None),
            };

            txn.execute(
                &edge_stmt,
                &[
                    &edge.from.to_string_lossy().as_ref(),
                    &edge.to.to_string_lossy().as_ref(),
                    &edge.dep_type.to_string(),
                    &sym_from.as_deref(),
                    &sym_to.as_deref(),
                    &sym_kind.as_deref(),
                    &strength.as_deref(),
                ],
            )
            .await
            .map_err(pg_error)?;
        }

        txn.commit().await.map_err(pg_error)?;

        Ok(())
    }

    fn name(&self) -> &'static str {
        "postgres"
    }
}

// ─── Helper Functions ───────────────────────────────────────────────────────

/// Converts a database row to a [`DependencyEdge`].
fn row_to_edge(row: &tokio_postgres::Row) -> Result<DependencyEdge, StorageError> {
    let from_path: String = row.get(0);
    let to_path: String = row.get(1);
    let dep_type_str: String = row.get(2);
    let symbol_from: Option<String> = row.get(3);
    let symbol_to: Option<String> = row.get(4);
    let symbol_kind: Option<String> = row.get(5);
    let strength: Option<String> = row.get(6);

    let dep_type = parse_dependency_type(&dep_type_str)?;

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

/// Converts raw bytes from Postgres BYTEA to a [`Fingerprint`].
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

/// Converts a `tokio_postgres::Error` to a [`StorageError::Backend`].
fn pg_error(e: tokio_postgres::Error) -> StorageError {
    StorageError::Backend(format!("Postgres error: {e}"))
}

/// Converts a deadpool pool error to a [`StorageError::Backend`].
fn pg_pool_error(e: deadpool_postgres::PoolError) -> StorageError {
    StorageError::Backend(format!("Connection pool error: {e}"))
}
