// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Concurrency abstraction layer for incremental analysis.
//!
//! Provides unified interface for parallel execution across different deployment targets:
//! - **RayonExecutor**: CPU-bound parallelism for CLI (multi-core)
//! - **TokioExecutor**: Async I/O concurrency for all deployments
//! - **SequentialExecutor**: Fallback for single-threaded execution
//!
//! ## Architecture
//!
//! The concurrency layer adapts to deployment context via feature flags:
//! - CLI with `parallel` feature: Rayon for CPU-bound work
//! - All deployments: tokio for async I/O operations
//! - Fallback: Sequential execution when parallelism unavailable
//!
//! ## Examples
//!
//! ### Basic Usage
//!
//! ```rust
//! use thread_flow::incremental::concurrency::{
//!     create_executor, ConcurrencyMode, ExecutionError,
//! };
//!
//! # async fn example() -> Result<(), ExecutionError> {
//! // Create executor for current deployment
//! let executor = create_executor(ConcurrencyMode::Tokio { max_concurrent: 10 });
//!
//! // Process batch of items
//! let items = vec![1, 2, 3, 4, 5];
//! let results = executor.execute_batch(items, |n| {
//!     // Your work here
//!     Ok(())
//! }).await?;
//!
//! assert_eq!(results.len(), 5);
//! # Ok(())
//! # }
//! ```
//!
//! ### Feature-Aware Execution
//!
//! ```rust
//! use thread_flow::incremental::concurrency::{
//!     create_executor, ConcurrencyMode,
//! };
//!
//! # async fn example() {
//! // Automatically uses best executor for current build
//! #[cfg(feature = "parallel")]
//! let executor = create_executor(ConcurrencyMode::Rayon { num_threads: None });
//!
//! #[cfg(not(feature = "parallel"))]
//! let executor = create_executor(ConcurrencyMode::Tokio { max_concurrent: 10 });
//! # }
//! ```

use async_trait::async_trait;
use std::sync::Arc;
use thiserror::Error;

/// Errors that can occur during batch execution.
#[derive(Debug, Error)]
pub enum ExecutionError {
    /// Generic execution failure with description.
    #[error("Execution failed: {0}")]
    Failed(String),

    /// Thread pool creation or management error.
    #[error("Thread pool error: {0}")]
    ThreadPool(String),

    /// Task join or coordination error.
    #[error("Task join error: {0}")]
    Join(String),
}

/// Unified interface for concurrent batch execution.
///
/// Implementations provide different parallelism strategies:
/// - **Rayon**: CPU-bound parallelism (multi-threaded)
/// - **Tokio**: I/O-bound concurrency (async tasks)
/// - **Sequential**: Single-threaded fallback
#[async_trait]
pub trait ConcurrencyExecutor: Send + Sync {
    /// Execute operation on batch of items concurrently.
    ///
    /// Returns vector of results in same order as input items.
    /// Individual item failures don't stop processing of other items.
    ///
    /// # Arguments
    ///
    /// * `items` - Batch of items to process
    /// * `op` - Operation to apply to each item
    ///
    /// # Returns
    ///
    /// Vector of results for each item. Length matches input items.
    ///
    /// # Errors
    ///
    /// Returns error if batch execution infrastructure fails.
    /// Individual item failures are captured in result vector.
    async fn execute_batch<F, T>(
        &self,
        items: Vec<T>,
        op: F,
    ) -> Result<Vec<Result<(), ExecutionError>>, ExecutionError>
    where
        F: Fn(T) -> Result<(), ExecutionError> + Send + Sync + 'static,
        T: Send + 'static;

    /// Get executor implementation name for debugging.
    fn name(&self) -> &str;
}

// ============================================================================
// Rayon Executor (CPU-bound parallelism, CLI only)
// ============================================================================

#[cfg(feature = "parallel")]
/// CPU-bound parallel executor using Rayon thread pool.
///
/// Optimized for multi-core CLI deployments processing independent items.
/// Not available in edge deployments (no `parallel` feature).
#[derive(Debug)]
pub struct RayonExecutor {
    thread_pool: rayon::ThreadPool,
}

#[cfg(feature = "parallel")]
impl RayonExecutor {
    /// Create new Rayon executor with optional thread count.
    ///
    /// # Arguments
    ///
    /// * `num_threads` - Optional thread count (None = use all cores)
    ///
    /// # Errors
    ///
    /// Returns [`ExecutionError::ThreadPool`] if pool creation fails.
    pub fn new(num_threads: Option<usize>) -> Result<Self, ExecutionError> {
        let mut builder = rayon::ThreadPoolBuilder::new();

        if let Some(threads) = num_threads {
            if threads == 0 {
                return Err(ExecutionError::ThreadPool(
                    "Thread count must be > 0".to_string(),
                ));
            }
            builder = builder.num_threads(threads);
        }

        let thread_pool = builder.build().map_err(|e| {
            ExecutionError::ThreadPool(format!("Failed to create thread pool: {}", e))
        })?;

        Ok(Self { thread_pool })
    }
}

#[cfg(feature = "parallel")]
#[async_trait]
impl ConcurrencyExecutor for RayonExecutor {
    async fn execute_batch<F, T>(
        &self,
        items: Vec<T>,
        op: F,
    ) -> Result<Vec<Result<(), ExecutionError>>, ExecutionError>
    where
        F: Fn(T) -> Result<(), ExecutionError> + Send + Sync + 'static,
        T: Send + 'static,
    {
        // Wrap operation for thread safety
        let op = Arc::new(op);

        // Process items in parallel using Rayon
        let results = self.thread_pool.install(|| {
            use rayon::prelude::*;
            items
                .into_par_iter()
                .map(|item| op(item))
                .collect::<Vec<_>>()
        });

        Ok(results)
    }

    fn name(&self) -> &str {
        "rayon"
    }
}

// ============================================================================
// Tokio Executor (I/O-bound concurrency, always available)
// ============================================================================

/// Async I/O executor using tokio tasks with concurrency limit.
///
/// Optimized for I/O-bound operations (network, disk, async operations).
/// Available in all deployments (tokio is standard dependency).
#[derive(Debug)]
pub struct TokioExecutor {
    max_concurrent: usize,
}

impl TokioExecutor {
    /// Create new Tokio executor with concurrency limit.
    ///
    /// # Arguments
    ///
    /// * `max_concurrent` - Maximum number of concurrent async tasks
    pub fn new(max_concurrent: usize) -> Self {
        Self { max_concurrent }
    }
}

#[async_trait]
impl ConcurrencyExecutor for TokioExecutor {
    async fn execute_batch<F, T>(
        &self,
        items: Vec<T>,
        op: F,
    ) -> Result<Vec<Result<(), ExecutionError>>, ExecutionError>
    where
        F: Fn(T) -> Result<(), ExecutionError> + Send + Sync + 'static,
        T: Send + 'static,
    {
        use tokio::sync::Semaphore;
        use tokio::task;

        // Semaphore for concurrency control
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let op = Arc::new(op);

        // Spawn tasks with concurrency limit
        let mut handles = Vec::with_capacity(items.len());
        for item in items {
            let permit = semaphore.clone().acquire_owned().await.map_err(|e| {
                ExecutionError::Join(format!("Semaphore acquisition failed: {}", e))
            })?;

            let op = Arc::clone(&op);
            let handle = task::spawn_blocking(move || {
                let result = op(item);
                drop(permit); // Release permit
                result
            });

            handles.push(handle);
        }

        // Collect results in order
        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            let result = handle
                .await
                .map_err(|e| ExecutionError::Join(format!("Task join failed: {}", e)))?;
            results.push(result);
        }

        Ok(results)
    }

    fn name(&self) -> &str {
        "tokio"
    }
}

// ============================================================================
// Sequential Executor (Single-threaded fallback)
// ============================================================================

/// Sequential executor processing items one at a time.
///
/// Fallback executor when parallelism is unavailable or undesired.
/// Always available regardless of feature flags.
#[derive(Debug)]
pub struct SequentialExecutor;

#[async_trait]
impl ConcurrencyExecutor for SequentialExecutor {
    async fn execute_batch<F, T>(
        &self,
        items: Vec<T>,
        op: F,
    ) -> Result<Vec<Result<(), ExecutionError>>, ExecutionError>
    where
        F: Fn(T) -> Result<(), ExecutionError> + Send + Sync + 'static,
        T: Send + 'static,
    {
        // Process items sequentially
        let results = items.into_iter().map(op).collect();
        Ok(results)
    }

    fn name(&self) -> &str {
        "sequential"
    }
}

// ============================================================================
// Factory Pattern
// ============================================================================

/// Unified executor enum combining all concurrency strategies.
///
/// Wraps different executor implementations in a single enum for type-safe usage.
/// Automatically routes to appropriate implementation based on configuration.
#[derive(Debug)]
pub enum Executor {
    /// Sequential executor (always available).
    Sequential(SequentialExecutor),

    /// Tokio async executor (always available).
    Tokio(TokioExecutor),

    /// Rayon parallel executor (requires `parallel` feature).
    #[cfg(feature = "parallel")]
    Rayon(RayonExecutor),
}

impl Executor {
    /// Create Sequential executor.
    pub fn sequential() -> Self {
        Self::Sequential(SequentialExecutor)
    }

    /// Create Tokio executor with concurrency limit.
    pub fn tokio(max_concurrent: usize) -> Self {
        Self::Tokio(TokioExecutor::new(max_concurrent))
    }

    /// Create Rayon executor with optional thread count (requires `parallel` feature).
    #[cfg(feature = "parallel")]
    pub fn rayon(num_threads: Option<usize>) -> Result<Self, ExecutionError> {
        RayonExecutor::new(num_threads).map(Self::Rayon)
    }

    /// Get executor implementation name for debugging.
    pub fn name(&self) -> &str {
        match self {
            Self::Sequential(_) => "sequential",
            Self::Tokio(_) => "tokio",
            #[cfg(feature = "parallel")]
            Self::Rayon(_) => "rayon",
        }
    }

    /// Execute operation on batch of items concurrently.
    ///
    /// Returns vector of results in same order as input items.
    /// Individual item failures don't stop processing of other items.
    pub async fn execute_batch<F, T>(
        &self,
        items: Vec<T>,
        op: F,
    ) -> Result<Vec<Result<(), ExecutionError>>, ExecutionError>
    where
        F: Fn(T) -> Result<(), ExecutionError> + Send + Sync + 'static,
        T: Send + 'static,
    {
        match self {
            Self::Sequential(exec) => exec.execute_batch(items, op).await,
            Self::Tokio(exec) => exec.execute_batch(items, op).await,
            #[cfg(feature = "parallel")]
            Self::Rayon(exec) => exec.execute_batch(items, op).await,
        }
    }
}

/// Concurrency mode selection for executor factory.
#[derive(Debug, Clone)]
pub enum ConcurrencyMode {
    /// Rayon parallel executor (requires `parallel` feature).
    Rayon { num_threads: Option<usize> },

    /// Tokio async executor (always available).
    Tokio { max_concurrent: usize },

    /// Sequential fallback executor.
    Sequential,
}

/// Create executor instance based on mode and available features.
///
/// Automatically falls back to Sequential when requested mode unavailable.
///
/// # Arguments
///
/// * `mode` - Desired concurrency mode
///
/// # Returns
///
/// Executor enum instance ready for use.
///
/// # Examples
///
/// ```rust
/// use thread_flow::incremental::concurrency::{
///     create_executor, ConcurrencyMode,
/// };
///
/// # async fn example() {
/// // Request Rayon (falls back to Sequential if `parallel` feature disabled)
/// let executor = create_executor(ConcurrencyMode::Rayon { num_threads: Some(4) });
///
/// // Tokio always available
/// let executor = create_executor(ConcurrencyMode::Tokio { max_concurrent: 10 });
/// # }
/// ```
pub fn create_executor(mode: ConcurrencyMode) -> Executor {
    match mode {
        #[cfg(feature = "parallel")]
        ConcurrencyMode::Rayon { num_threads } => {
            match Executor::rayon(num_threads) {
                Ok(executor) => executor,
                Err(_) => {
                    // Fall back to Sequential on Rayon initialization failure
                    Executor::sequential()
                }
            }
        }

        #[cfg(not(feature = "parallel"))]
        ConcurrencyMode::Rayon { .. } => {
            // Graceful degradation when `parallel` feature disabled
            Executor::sequential()
        }

        ConcurrencyMode::Tokio { max_concurrent } => Executor::tokio(max_concurrent),

        ConcurrencyMode::Sequential => Executor::sequential(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sequential_basic() {
        let executor = SequentialExecutor;
        let items = vec![1, 2, 3];
        let results = executor.execute_batch(items, |_| Ok(())).await.unwrap();

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[tokio::test]
    async fn test_tokio_basic() {
        let executor = TokioExecutor::new(2);
        let items = vec![1, 2, 3];
        let results = executor.execute_batch(items, |_| Ok(())).await.unwrap();

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[cfg(feature = "parallel")]
    #[tokio::test]
    async fn test_rayon_basic() {
        let executor = RayonExecutor::new(None).unwrap();
        let items = vec![1, 2, 3];
        let results = executor.execute_batch(items, |_| Ok(())).await.unwrap();

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[test]
    fn test_factory_sequential() {
        let executor = create_executor(ConcurrencyMode::Sequential);
        assert_eq!(executor.name(), "sequential");
    }

    #[test]
    fn test_factory_tokio() {
        let executor = create_executor(ConcurrencyMode::Tokio { max_concurrent: 5 });
        assert_eq!(executor.name(), "tokio");
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn test_factory_rayon() {
        let executor = create_executor(ConcurrencyMode::Rayon { num_threads: None });
        assert_eq!(executor.name(), "rayon");
    }

    #[cfg(not(feature = "parallel"))]
    #[test]
    fn test_factory_rayon_fallback() {
        let executor = create_executor(ConcurrencyMode::Rayon { num_threads: None });
        // Falls back to sequential when parallel feature disabled
        assert_eq!(executor.name(), "sequential");
    }
}
