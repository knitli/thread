// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Concrete storage backend implementations for the incremental update system.
//!
//! This module provides database-specific implementations of the
//! [`StorageBackend`](super::storage::StorageBackend) trait:
//!
//! - **Postgres** (`postgres-backend` feature): Full SQL backend for CLI deployment
//!   with connection pooling, prepared statements, and batch operations.
//! - **D1** (`d1-backend` feature): Cloudflare D1 backend for edge deployment
//!   via the Cloudflare REST API.
//! - **InMemory**: Simple in-memory backend for testing (always available).
//!
//! ## Backend Factory Pattern
//!
//! The [`create_backend`] factory function provides runtime backend selection
//! based on deployment environment and feature flags:
//!
//! ```rust
//! use thread_flow::incremental::backends::{BackendType, BackendConfig, create_backend};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // CLI deployment with Postgres
//! # #[cfg(feature = "postgres-backend")]
//! let backend = create_backend(
//!     BackendType::Postgres,
//!     BackendConfig::Postgres {
//!         database_url: "postgresql://localhost/thread".to_string(),
//!     },
//! ).await?;
//!
//! // Edge deployment with D1
//! # #[cfg(feature = "d1-backend")]
//! let backend = create_backend(
//!     BackendType::D1,
//!     BackendConfig::D1 {
//!         account_id: "your-account-id".to_string(),
//!         database_id: "your-db-id".to_string(),
//!         api_token: "your-token".to_string(),
//!     },
//! ).await?;
//!
//! // Testing with in-memory storage (always available)
//! let backend = create_backend(
//!     BackendType::InMemory,
//!     BackendConfig::InMemory,
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Feature Gating
//!
//! Backend availability depends on cargo features:
//!
//! - `postgres-backend`: Enables [`PostgresIncrementalBackend`]
//! - `d1-backend`: Enables [`D1IncrementalBackend`]
//! - No features required: [`InMemoryStorage`] always available
//!
//! Attempting to use a disabled backend returns [`IncrementalError::UnsupportedBackend`].
//!
//! ## Deployment Scenarios
//!
//! ### CLI Deployment (Postgres)
//!
//! ```toml
//! [dependencies]
//! thread-flow = { version = "*", features = ["postgres-backend"] }
//! ```
//!
//! ```rust
//! # #[cfg(feature = "postgres-backend")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use thread_flow::incremental::backends::{BackendType, BackendConfig, create_backend};
//!
//! let backend = create_backend(
//!     BackendType::Postgres,
//!     BackendConfig::Postgres {
//!         database_url: std::env::var("DATABASE_URL")?,
//!     },
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Edge Deployment (D1)
//!
//! ```toml
//! [dependencies]
//! thread-flow = { version = "*", features = ["d1-backend", "worker"] }
//! ```
//!
//! ```rust
//! # #[cfg(feature = "d1-backend")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use thread_flow::incremental::backends::{BackendType, BackendConfig, create_backend};
//!
//! let backend = create_backend(
//!     BackendType::D1,
//!     BackendConfig::D1 {
//!         account_id: std::env::var("CF_ACCOUNT_ID")?,
//!         database_id: std::env::var("CF_DATABASE_ID")?,
//!         api_token: std::env::var("CF_API_TOKEN")?,
//!     },
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Testing (InMemory)
//!
//! ```rust
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use thread_flow::incremental::backends::{BackendType, BackendConfig, create_backend};
//!
//! let backend = create_backend(
//!     BackendType::InMemory,
//!     BackendConfig::InMemory,
//! ).await?;
//! # Ok(())
//! # }
//! ```

use super::storage::{InMemoryStorage, StorageBackend};
use std::error::Error;
use std::fmt;

#[cfg(feature = "postgres-backend")]
pub mod postgres;

#[cfg(feature = "d1-backend")]
pub mod d1;

#[cfg(feature = "postgres-backend")]
pub use postgres::PostgresIncrementalBackend;

#[cfg(feature = "d1-backend")]
pub use d1::D1IncrementalBackend;

// ─── Error Types ──────────────────────────────────────────────────────────────

/// Errors that can occur during backend initialization and operation.
#[derive(Debug)]
pub enum IncrementalError {
    /// The requested backend is not available (feature flag disabled).
    UnsupportedBackend(&'static str),

    /// Backend initialization failed (connection error, invalid config, etc.).
    InitializationFailed(String),

    /// Propagated storage error from backend operations.
    Storage(super::storage::StorageError),
}

impl fmt::Display for IncrementalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IncrementalError::UnsupportedBackend(backend) => {
                write!(
                    f,
                    "Backend '{}' is not available. Enable the corresponding feature flag.",
                    backend
                )
            }
            IncrementalError::InitializationFailed(msg) => {
                write!(f, "Backend initialization failed: {}", msg)
            }
            IncrementalError::Storage(err) => write!(f, "Storage error: {}", err),
        }
    }
}

impl Error for IncrementalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            IncrementalError::Storage(err) => Some(err),
            _ => None,
        }
    }
}

impl From<super::storage::StorageError> for IncrementalError {
    fn from(err: super::storage::StorageError) -> Self {
        IncrementalError::Storage(err)
    }
}

// ─── Backend Configuration ────────────────────────────────────────────────────

/// Backend type selector for runtime backend selection.
///
/// Use this enum with [`create_backend`] to instantiate the appropriate
/// storage backend based on deployment environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    /// PostgreSQL backend (requires `postgres-backend` feature).
    ///
    /// Primary backend for CLI deployment with connection pooling
    /// and batch operations.
    Postgres,

    /// Cloudflare D1 backend (requires `d1-backend` feature).
    ///
    /// Primary backend for edge deployment via Cloudflare Workers.
    D1,

    /// In-memory backend (always available).
    ///
    /// Used for testing and development. Data is not persisted.
    InMemory,
}

/// Configuration for backend initialization.
///
/// Each variant contains the connection parameters needed to initialize
/// the corresponding backend type.
#[derive(Debug, Clone)]
pub enum BackendConfig {
    /// PostgreSQL connection configuration.
    Postgres {
        /// PostgreSQL connection URL (e.g., `postgresql://localhost/thread`).
        database_url: String,
    },

    /// Cloudflare D1 connection configuration.
    D1 {
        /// Cloudflare account ID.
        account_id: String,
        /// D1 database ID.
        database_id: String,
        /// Cloudflare API token with D1 read/write permissions.
        api_token: String,
    },

    /// In-memory storage (no configuration needed).
    InMemory,
}

// ─── Backend Factory ──────────────────────────────────────────────────────────

/// Creates a storage backend based on the specified type and configuration.
///
/// This factory function provides runtime backend selection with compile-time
/// feature gating. If a backend is requested but its feature flag is disabled,
/// returns [`IncrementalError::UnsupportedBackend`].
///
/// # Arguments
///
/// * `backend_type` - The type of backend to instantiate.
/// * `config` - Configuration parameters for the backend.
///
/// # Returns
///
/// A boxed trait object implementing [`StorageBackend`], or an error if:
/// - The backend feature is disabled ([`IncrementalError::UnsupportedBackend`])
/// - Backend initialization fails ([`IncrementalError::InitializationFailed`])
/// - Configuration mismatch between `backend_type` and `config`
///
/// # Examples
///
/// ```rust
/// use thread_flow::incremental::backends::{BackendType, BackendConfig, create_backend};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create in-memory backend (always available)
/// let backend = create_backend(
///     BackendType::InMemory,
///     BackendConfig::InMemory,
/// ).await?;
///
/// // Create Postgres backend (requires postgres-backend feature)
/// # #[cfg(feature = "postgres-backend")]
/// let backend = create_backend(
///     BackendType::Postgres,
///     BackendConfig::Postgres {
///         database_url: "postgresql://localhost/thread".to_string(),
///     },
/// ).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// - [`IncrementalError::UnsupportedBackend`]: Feature flag disabled for requested backend
/// - [`IncrementalError::InitializationFailed`]: Connection failed, invalid config, or initialization error
pub async fn create_backend(
    backend_type: BackendType,
    config: BackendConfig,
) -> Result<Box<dyn StorageBackend>, IncrementalError> {
    match (backend_type, config) {
        // ── Postgres Backend ──────────────────────────────────────────────
        (BackendType::Postgres, BackendConfig::Postgres { database_url }) => {
            #[cfg(feature = "postgres-backend")]
            {
                PostgresIncrementalBackend::new(&database_url)
                    .await
                    .map(|b| Box::new(b) as Box<dyn StorageBackend>)
                    .map_err(|e| {
                        IncrementalError::InitializationFailed(format!("Postgres init failed: {}", e))
                    })
            }
            #[cfg(not(feature = "postgres-backend"))]
            {
                let _ = database_url; // Suppress unused warning
                Err(IncrementalError::UnsupportedBackend("postgres"))
            }
        }

        // ── D1 Backend ────────────────────────────────────────────────────
        (
            BackendType::D1,
            BackendConfig::D1 {
                account_id,
                database_id,
                api_token,
            },
        ) => {
            #[cfg(feature = "d1-backend")]
            {
                D1IncrementalBackend::new(account_id, database_id, api_token)
                    .map(|b| Box::new(b) as Box<dyn StorageBackend>)
                    .map_err(|e| {
                        IncrementalError::InitializationFailed(format!("D1 init failed: {}", e))
                    })
            }
            #[cfg(not(feature = "d1-backend"))]
            {
                let _ = (account_id, database_id, api_token); // Suppress unused warnings
                Err(IncrementalError::UnsupportedBackend("d1"))
            }
        }

        // ── InMemory Backend ──────────────────────────────────────────────
        (BackendType::InMemory, BackendConfig::InMemory) => {
            Ok(Box::new(InMemoryStorage::new()) as Box<dyn StorageBackend>)
        }

        // ── Configuration Mismatch ────────────────────────────────────────
        _ => Err(IncrementalError::InitializationFailed(
            "Backend type and configuration mismatch".to_string(),
        )),
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_in_memory_backend() {
        let result = create_backend(BackendType::InMemory, BackendConfig::InMemory).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_configuration_mismatch() {
        let result = create_backend(
            BackendType::InMemory,
            BackendConfig::Postgres {
                database_url: "test".to_string(),
            },
        )
        .await;
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(matches!(
                err,
                IncrementalError::InitializationFailed(_)
            ));
        }
    }

    #[cfg(not(feature = "postgres-backend"))]
    #[tokio::test]
    async fn test_postgres_backend_unavailable() {
        let result = create_backend(
            BackendType::Postgres,
            BackendConfig::Postgres {
                database_url: "postgresql://localhost/test".to_string(),
            },
        )
        .await;
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(matches!(
                err,
                IncrementalError::UnsupportedBackend("postgres")
            ));
        }
    }

    #[cfg(not(feature = "d1-backend"))]
    #[tokio::test]
    async fn test_d1_backend_unavailable() {
        let result = create_backend(
            BackendType::D1,
            BackendConfig::D1 {
                account_id: "test".to_string(),
                database_id: "test".to_string(),
                api_token: "test".to_string(),
            },
        )
        .await;
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(matches!(
                err,
                IncrementalError::UnsupportedBackend("d1")
            ));
        }
    }

    #[test]
    fn test_incremental_error_display() {
        let err = IncrementalError::UnsupportedBackend("test");
        assert!(format!("{}", err).contains("not available"));

        let err = IncrementalError::InitializationFailed("connection failed".to_string());
        assert!(format!("{}", err).contains("connection failed"));
    }

    #[test]
    fn test_backend_type_equality() {
        assert_eq!(BackendType::InMemory, BackendType::InMemory);
        assert_ne!(BackendType::Postgres, BackendType::D1);
    }
}
