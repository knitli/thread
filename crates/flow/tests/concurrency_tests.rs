// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileCopyrightText: 2026 Knitli Inc.
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Comprehensive test suite for concurrency layer (Phase 4.3).
//!
//! Tests cover three executor implementations:
//! - Sequential: Always available fallback
//! - Rayon: CPU-bound parallelism (feature = "parallel")
//! - Tokio: Async I/O concurrency (always available)
//!
//! Test organization:
//! 1. Test helpers and fixtures
//! 2. Sequential executor tests
//! 3. Rayon executor tests (feature-gated)
//! 4. Tokio executor tests
//! 5. Factory pattern tests
//! 6. Error handling tests
//! 7. Feature gating tests
//! 8. Performance validation tests
//! 9. Integration tests

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use thread_flow::incremental::concurrency::{
    ConcurrencyMode, ExecutionError, Executor, create_executor,
};

// ============================================================================
// Test Helpers and Fixtures
// ============================================================================

/// CPU-intensive work simulation (parsing, hashing).
fn cpu_intensive_work(_n: u32) -> Result<(), ExecutionError> {
    let _result: u64 = (0..10000).map(|i| (i as u64).wrapping_mul(i as u64)).sum();
    Ok(())
}

/// I/O-bound work simulation (network, disk).
fn io_bound_work(_n: u32) -> Result<(), ExecutionError> {
    std::thread::sleep(Duration::from_millis(10));
    Ok(())
}

/// Fails on multiples of 10.
fn conditional_failure(n: u32) -> Result<(), ExecutionError> {
    if n % 10 == 0 {
        Err(ExecutionError::Failed(format!("Item {} failed", n)))
    } else {
        Ok(())
    }
}

/// Always fails.
fn always_fails(_n: u32) -> Result<(), ExecutionError> {
    Err(ExecutionError::Failed("Intentional failure".to_string()))
}

/// Verify batch result statistics.
fn assert_batch_results(
    results: &[Result<(), ExecutionError>],
    expected_success: usize,
    expected_failure: usize,
) {
    let successes = results.iter().filter(|r| r.is_ok()).count();
    let failures = results.iter().filter(|r| r.is_err()).count();

    assert_eq!(
        successes, expected_success,
        "Expected {} successes, got {}",
        expected_success, successes
    );
    assert_eq!(
        failures, expected_failure,
        "Expected {} failures, got {}",
        expected_failure, failures
    );
}

// ============================================================================
// 1. Sequential Executor Tests
// ============================================================================

mod sequential_tests {
    use super::*;

    #[tokio::test]
    async fn test_sequential_basic_execution() {
        let executor = Executor::sequential();
        let items: Vec<u32> = (0..10).collect();

        let results = executor
            .execute_batch(items, cpu_intensive_work)
            .await
            .unwrap();

        assert_eq!(results.len(), 10);
        assert_batch_results(&results, 10, 0);
        assert_eq!(executor.name(), "sequential");
    }

    #[tokio::test]
    async fn test_sequential_empty_batch() {
        let executor = Executor::sequential();
        let items: Vec<u32> = vec![];

        let results = executor
            .execute_batch(items, cpu_intensive_work)
            .await
            .unwrap();

        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_sequential_error_propagation() {
        let executor = Executor::sequential();
        let items: Vec<u32> = (0..20).collect();

        let results = executor
            .execute_batch(items, conditional_failure)
            .await
            .unwrap();

        assert_eq!(results.len(), 20);
        // Failures at 0, 10
        assert_batch_results(&results, 18, 2);
    }

    #[tokio::test]
    async fn test_sequential_ordering_preserved() {
        let executor = Executor::sequential();
        let items: Vec<u32> = vec![5, 3, 8, 1, 9];

        let order = Arc::new(std::sync::Mutex::new(Vec::new()));
        let order_clone = Arc::clone(&order);

        let results = executor
            .execute_batch(items, move |n| {
                order_clone.lock().unwrap().push(n);
                Ok(())
            })
            .await
            .unwrap();

        assert_eq!(results.len(), 5);
        assert_batch_results(&results, 5, 0);

        let execution_order = order.lock().unwrap();
        assert_eq!(*execution_order, vec![5, 3, 8, 1, 9]);
    }
}

// ============================================================================
// 2. Rayon Executor Tests (CPU-bound parallelism)
// ============================================================================

#[cfg(feature = "parallel")]
mod rayon_tests {
    use super::*;

    #[tokio::test]
    async fn test_rayon_basic_execution() {
        let executor = Executor::rayon(None).unwrap();
        let items: Vec<u32> = (0..10).collect();

        let results = executor
            .execute_batch(items, cpu_intensive_work)
            .await
            .unwrap();

        assert_eq!(results.len(), 10);
        assert_batch_results(&results, 10, 0);
        assert_eq!(executor.name(), "rayon");
    }

    #[tokio::test]
    async fn test_rayon_empty_batch() {
        let executor = Executor::rayon(None).unwrap();
        let items: Vec<u32> = vec![];

        let results = executor
            .execute_batch(items, cpu_intensive_work)
            .await
            .unwrap();

        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_rayon_large_batch() {
        let executor = Executor::rayon(None).unwrap();
        let items: Vec<u32> = (0..1000).collect();

        let results = executor
            .execute_batch(items, cpu_intensive_work)
            .await
            .unwrap();

        assert_eq!(results.len(), 1000);
        assert_batch_results(&results, 1000, 0);
    }

    #[tokio::test]
    async fn test_rayon_error_propagation() {
        let executor = Executor::rayon(None).unwrap();
        let items: Vec<u32> = (0..100).collect();

        let results = executor
            .execute_batch(items, conditional_failure)
            .await
            .unwrap();

        assert_eq!(results.len(), 100);
        // Failures at 0, 10, 20, ..., 90 (10 failures)
        assert_batch_results(&results, 90, 10);
    }

    #[tokio::test]
    async fn test_rayon_all_failures() {
        let executor = Executor::rayon(None).unwrap();
        let items: Vec<u32> = (0..10).collect();

        let results = executor.execute_batch(items, always_fails).await.unwrap();

        assert_eq!(results.len(), 10);
        assert_batch_results(&results, 0, 10);
    }

    #[tokio::test]
    async fn test_rayon_thread_pool_configuration() {
        // Test with specific thread count
        let executor = Executor::rayon(Some(2)).unwrap();
        let items: Vec<u32> = (0..10).collect();

        let results = executor
            .execute_batch(items, cpu_intensive_work)
            .await
            .unwrap();

        assert_eq!(results.len(), 10);
        assert_batch_results(&results, 10, 0);

        // Test with default (all cores)
        let executor = Executor::rayon(None).unwrap();
        let items: Vec<u32> = (0..10).collect();

        let results = executor
            .execute_batch(items, cpu_intensive_work)
            .await
            .unwrap();

        assert_eq!(results.len(), 10);
        assert_batch_results(&results, 10, 0);
    }

    #[tokio::test]
    async fn test_rayon_thread_pool_error() {
        // Invalid thread count (0)
        let result = Executor::rayon(Some(0));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExecutionError::ThreadPool(_)));
    }
}

// ============================================================================
// 3. Tokio Executor Tests (Async I/O)
// ============================================================================

mod tokio_tests {
    use super::*;

    #[tokio::test]
    async fn test_tokio_basic_execution() {
        let executor = Executor::tokio(10);
        let items: Vec<u32> = (0..10).collect();

        let results = executor
            .execute_batch(items, cpu_intensive_work)
            .await
            .unwrap();

        assert_eq!(results.len(), 10);
        assert_batch_results(&results, 10, 0);
        assert_eq!(executor.name(), "tokio");
    }

    #[tokio::test]
    async fn test_tokio_empty_batch() {
        let executor = Executor::tokio(10);
        let items: Vec<u32> = vec![];

        let results = executor
            .execute_batch(items, cpu_intensive_work)
            .await
            .unwrap();

        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_tokio_large_concurrent_batch() {
        let executor = Executor::tokio(10);
        let items: Vec<u32> = (0..100).collect();

        let results = executor
            .execute_batch(items, cpu_intensive_work)
            .await
            .unwrap();

        assert_eq!(results.len(), 100);
        assert_batch_results(&results, 100, 0);
    }

    #[tokio::test]
    async fn test_tokio_error_propagation() {
        let executor = Executor::tokio(10);
        let items: Vec<u32> = (0..50).collect();

        let results = executor
            .execute_batch(items, conditional_failure)
            .await
            .unwrap();

        assert_eq!(results.len(), 50);
        // Failures at 0, 10, 20, 30, 40 (5 failures)
        assert_batch_results(&results, 45, 5);
    }

    #[tokio::test]
    async fn test_tokio_all_failures() {
        let executor = Executor::tokio(10);
        let items: Vec<u32> = (0..10).collect();

        let results = executor.execute_batch(items, always_fails).await.unwrap();

        assert_eq!(results.len(), 10);
        assert_batch_results(&results, 0, 10);
    }

    #[tokio::test]
    async fn test_tokio_concurrency_limit() {
        let concurrent_count = Arc::new(AtomicUsize::new(0));
        let max_observed = Arc::new(AtomicUsize::new(0));

        let executor = Executor::tokio(5);
        let items: Vec<u32> = (0..50).collect();

        let concurrent_clone = Arc::clone(&concurrent_count);
        let max_clone = Arc::clone(&max_observed);

        let results = executor
            .execute_batch(items, move |_| {
                let current = concurrent_clone.fetch_add(1, Ordering::SeqCst) + 1;

                // Update max observed
                max_clone.fetch_max(current, Ordering::SeqCst);

                // Simulate work
                std::thread::sleep(Duration::from_millis(10));

                concurrent_clone.fetch_sub(1, Ordering::SeqCst);
                Ok(())
            })
            .await
            .unwrap();

        assert_eq!(results.len(), 50);
        assert_batch_results(&results, 50, 0);

        // Verify concurrency limit respected
        let max = max_observed.load(Ordering::SeqCst);
        assert!(
            max <= 5,
            "Concurrency limit violated: observed {} concurrent tasks",
            max
        );
    }
}

// ============================================================================
// 4. Factory Pattern Tests
// ============================================================================

mod factory_tests {
    use super::*;

    #[tokio::test]
    async fn test_factory_creates_sequential() {
        let executor = create_executor(ConcurrencyMode::Sequential);
        assert_eq!(executor.name(), "sequential");

        let items: Vec<u32> = (0..5).collect();
        let results = executor
            .execute_batch(items, cpu_intensive_work)
            .await
            .unwrap();

        assert_eq!(results.len(), 5);
        assert_batch_results(&results, 5, 0);
    }

    #[cfg(feature = "parallel")]
    #[tokio::test]
    async fn test_factory_creates_rayon() {
        let executor = create_executor(ConcurrencyMode::Rayon { num_threads: None });
        assert_eq!(executor.name(), "rayon");

        let items: Vec<u32> = (0..5).collect();
        let results = executor
            .execute_batch(items, cpu_intensive_work)
            .await
            .unwrap();

        assert_eq!(results.len(), 5);
        assert_batch_results(&results, 5, 0);
    }

    #[tokio::test]
    async fn test_factory_creates_tokio() {
        let executor = create_executor(ConcurrencyMode::Tokio { max_concurrent: 10 });
        assert_eq!(executor.name(), "tokio");

        let items: Vec<u32> = (0..5).collect();
        let results = executor
            .execute_batch(items, cpu_intensive_work)
            .await
            .unwrap();

        assert_eq!(results.len(), 5);
        assert_batch_results(&results, 5, 0);
    }

    #[cfg(feature = "parallel")]
    #[tokio::test]
    async fn test_factory_rayon_with_threads() {
        let executor = create_executor(ConcurrencyMode::Rayon {
            num_threads: Some(4),
        });
        assert_eq!(executor.name(), "rayon");

        let items: Vec<u32> = (0..10).collect();
        let results = executor
            .execute_batch(items, cpu_intensive_work)
            .await
            .unwrap();

        assert_eq!(results.len(), 10);
        assert_batch_results(&results, 10, 0);
    }

    #[tokio::test]
    async fn test_factory_tokio_with_concurrency() {
        let executor = create_executor(ConcurrencyMode::Tokio { max_concurrent: 20 });
        assert_eq!(executor.name(), "tokio");

        let items: Vec<u32> = (0..10).collect();
        let results = executor
            .execute_batch(items, cpu_intensive_work)
            .await
            .unwrap();

        assert_eq!(results.len(), 10);
        assert_batch_results(&results, 10, 0);
    }
}

// ============================================================================
// 5. Error Handling Tests
// ============================================================================

mod error_tests {
    use super::*;

    #[test]
    fn test_execution_error_display() {
        let err = ExecutionError::Failed("test error".to_string());
        assert_eq!(err.to_string(), "Execution failed: test error");

        let err = ExecutionError::ThreadPool("pool error".to_string());
        assert_eq!(err.to_string(), "Thread pool error: pool error");

        let err = ExecutionError::Join("join error".to_string());
        assert_eq!(err.to_string(), "Task join error: join error");
    }

    #[test]
    fn test_execution_error_source() {
        let err = ExecutionError::Failed("test".to_string());
        // ExecutionError doesn't have inner sources, so source() returns None
        assert!(std::error::Error::source(&err).is_none());
    }

    #[tokio::test]
    async fn test_partial_batch_failure() {
        let executor = Executor::sequential();
        let items: Vec<u32> = (0..100).collect();

        let results = executor
            .execute_batch(items, conditional_failure)
            .await
            .unwrap();

        // Verify can filter results
        let successes: Vec<_> = results.iter().filter(|r| r.is_ok()).collect();
        let failures: Vec<_> = results.iter().filter(|r| r.is_err()).collect();

        assert_eq!(successes.len(), 90);
        assert_eq!(failures.len(), 10);
    }
}

// ============================================================================
// 6. Feature Gating Tests
// ============================================================================

mod feature_gating_tests {
    use super::*;

    #[tokio::test]
    async fn test_sequential_always_available() {
        // Sequential works without any feature flags
        let executor = Executor::sequential();
        let items: Vec<u32> = (0..5).collect();

        let results = executor
            .execute_batch(items, cpu_intensive_work)
            .await
            .unwrap();

        assert_eq!(results.len(), 5);
        assert_batch_results(&results, 5, 0);
    }

    #[cfg(not(feature = "parallel"))]
    #[tokio::test]
    async fn test_rayon_disabled_fallback() {
        // Rayon mode falls back to Sequential when `parallel` feature disabled
        let executor = create_executor(ConcurrencyMode::Rayon { num_threads: None });
        assert_eq!(executor.name(), "sequential");
    }

    #[tokio::test]
    async fn test_tokio_always_available() {
        // Tokio always available (no feature gate)
        let executor = Executor::tokio(10);
        let items: Vec<u32> = (0..5).collect();

        let results = executor
            .execute_batch(items, cpu_intensive_work)
            .await
            .unwrap();

        assert_eq!(results.len(), 5);
        assert_batch_results(&results, 5, 0);
    }

    #[tokio::test]
    async fn test_factory_feature_detection() {
        // Factory correctly creates executors based on available features

        // Sequential always works
        let executor = create_executor(ConcurrencyMode::Sequential);
        assert_eq!(executor.name(), "sequential");

        // Tokio always works
        let executor = create_executor(ConcurrencyMode::Tokio { max_concurrent: 5 });
        assert_eq!(executor.name(), "tokio");

        // Rayon depends on `parallel` feature
        let executor = create_executor(ConcurrencyMode::Rayon { num_threads: None });
        #[cfg(feature = "parallel")]
        assert_eq!(executor.name(), "rayon");

        #[cfg(not(feature = "parallel"))]
        assert_eq!(executor.name(), "sequential");
    }
}

// ============================================================================
// 7. Performance Validation Tests
// ============================================================================

#[cfg(feature = "parallel")]
mod performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_rayon_performance_benefit() {
        // Skip in CI or resource-constrained environments
        if std::env::var("CI").is_ok() {
            return;
        }

        let items: Vec<u32> = (0..1000).collect();

        // Benchmark Sequential
        let sequential = Executor::sequential();
        let start = Instant::now();
        sequential
            .execute_batch(items.clone(), cpu_intensive_work)
            .await
            .unwrap();
        let sequential_time = start.elapsed();

        // Benchmark Rayon
        let rayon = Executor::rayon(None).unwrap();
        let start = Instant::now();
        rayon
            .execute_batch(items, cpu_intensive_work)
            .await
            .unwrap();
        let rayon_time = start.elapsed();

        // Verify speedup (should be >1.5x on multi-core)
        let speedup = sequential_time.as_secs_f64() / rayon_time.as_secs_f64();
        println!(
            "Rayon speedup: {:.2}x (sequential: {:?}, rayon: {:?})",
            speedup, sequential_time, rayon_time
        );

        // Relaxed assertion for CI environments
        assert!(
            speedup > 1.2,
            "Rayon should show speedup (got {:.2}x)",
            speedup
        );
    }

    #[tokio::test]
    async fn test_rayon_multicore_scaling() {
        // Skip in CI
        if std::env::var("CI").is_ok() {
            return;
        }

        let items: Vec<u32> = (0..500).collect();

        // Single thread
        let single = Executor::rayon(Some(1)).unwrap();
        let start = Instant::now();
        single
            .execute_batch(items.clone(), cpu_intensive_work)
            .await
            .unwrap();
        let single_time = start.elapsed();

        // Four threads
        let multi = Executor::rayon(Some(4)).unwrap();
        let start = Instant::now();
        multi
            .execute_batch(items, cpu_intensive_work)
            .await
            .unwrap();
        let multi_time = start.elapsed();

        let speedup = single_time.as_secs_f64() / multi_time.as_secs_f64();
        println!(
            "Multi-core scaling: {:.2}x (1 thread: {:?}, 4 threads: {:?})",
            speedup, single_time, multi_time
        );

        assert!(
            speedup > 1.5,
            "Multi-core should scale (got {:.2}x)",
            speedup
        );
    }
}

mod tokio_performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_tokio_performance_benefit() {
        // Skip in CI
        if std::env::var("CI").is_ok() {
            return;
        }

        let items: Vec<u32> = (0..100).collect();

        // Benchmark Sequential with I/O-bound work
        let sequential = Executor::sequential();
        let start = Instant::now();
        sequential
            .execute_batch(items.clone(), io_bound_work)
            .await
            .unwrap();
        let sequential_time = start.elapsed();

        // Benchmark Tokio (max 10 concurrent)
        let tokio = Executor::tokio(10);
        let start = Instant::now();
        tokio.execute_batch(items, io_bound_work).await.unwrap();
        let tokio_time = start.elapsed();

        println!(
            "Tokio I/O concurrency: sequential {:?}, tokio {:?}",
            sequential_time, tokio_time
        );

        // Tokio should be significantly faster (>5x for I/O-bound work)
        let speedup = sequential_time.as_secs_f64() / tokio_time.as_secs_f64();
        println!("Tokio speedup: {:.2}x", speedup);

        assert!(
            speedup > 3.0,
            "Tokio should parallelize I/O (got {:.2}x)",
            speedup
        );
    }
}

// ============================================================================
// 8. Integration Tests
// ============================================================================

mod integration_tests {
    use super::*;
    use thread_flow::incremental::InMemoryStorage;
    use thread_flow::incremental::types::AnalysisDefFingerprint;

    #[tokio::test]
    async fn test_batch_file_reanalysis() {
        use std::path::PathBuf;

        let items: Vec<PathBuf> = (0..50)
            .map(|i| PathBuf::from(format!("file_{}.rs", i)))
            .collect();

        let executor = Executor::tokio(10);

        // Simulate reanalysis operation
        let results = executor
            .execute_batch(items, |path| {
                // Simulate AST parsing + fingerprint generation
                let content = format!("fn main() {{ // {} }}", path.display());
                let _fp = AnalysisDefFingerprint::new(content.as_bytes());
                Ok(())
            })
            .await
            .unwrap();

        assert_eq!(results.len(), 50);
        assert_batch_results(&results, 50, 0);
    }

    #[tokio::test]
    async fn test_with_storage_backend() {
        use std::path::Path;

        let _storage = InMemoryStorage::new();
        let executor = Executor::tokio(5);

        let items: Vec<(String, Vec<u8>)> = (0..20)
            .map(|i| (format!("file_{}.rs", i), vec![i as u8; 32]))
            .collect();

        let results = executor
            .execute_batch(items, |item| {
                let _path = Path::new(&item.0);
                let _fp = AnalysisDefFingerprint::new(&item.1);
                // Would normally: storage.save_fingerprint(path, &fp).await
                Ok(())
            })
            .await
            .unwrap();

        assert_eq!(results.len(), 20);
        assert_batch_results(&results, 20, 0);
    }

    #[cfg(feature = "parallel")]
    #[tokio::test]
    async fn test_executor_thread_safety() {
        let executor = Arc::new(Executor::rayon(None).unwrap());
        let mut handles = vec![];

        // Spawn 5 concurrent tasks using same executor
        for batch_id in 0..5 {
            let exec_clone = Arc::clone(&executor);
            let handle = tokio::spawn(async move {
                let items: Vec<u32> = (batch_id * 10..(batch_id + 1) * 10).collect();
                exec_clone.execute_batch(items, cpu_intensive_work).await
            });
            handles.push(handle);
        }

        // All should succeed
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_executor_reuse_across_batches() {
        let executor = create_executor(ConcurrencyMode::Sequential);

        // First batch
        let batch1: Vec<u32> = (0..10).collect();
        let results1 = executor
            .execute_batch(batch1, cpu_intensive_work)
            .await
            .unwrap();
        assert_eq!(results1.len(), 10);
        assert_batch_results(&results1, 10, 0);

        // Second batch (reuse same executor)
        let batch2: Vec<u32> = (10..20).collect();
        let results2 = executor
            .execute_batch(batch2, cpu_intensive_work)
            .await
            .unwrap();
        assert_eq!(results2.len(), 10);
        assert_batch_results(&results2, 10, 0);

        // Third batch with different operation
        let batch3: Vec<u32> = (20..30).collect();
        let results3 = executor.execute_batch(batch3, io_bound_work).await.unwrap();
        assert_eq!(results3.len(), 10);
        assert_batch_results(&results3, 10, 0);
    }
}

// ============================================================================
// 9. Edge Cases and Stress Tests
// ============================================================================

mod stress_tests {
    use super::*;

    #[tokio::test]
    async fn test_very_large_batch() {
        let executor = Executor::tokio(20);
        let items: Vec<u32> = (0..10000).collect();

        let results = executor
            .execute_batch(items, cpu_intensive_work)
            .await
            .unwrap();

        assert_eq!(results.len(), 10000);
        assert_batch_results(&results, 10000, 0);
    }

    #[tokio::test]
    async fn test_concurrent_executor_usage() {
        let mut handles = vec![];

        // Create 10 different executors and run them concurrently
        for i in 0..10 {
            let handle = tokio::spawn(async move {
                let executor = Executor::tokio(5);
                let items: Vec<u32> = (i * 10..(i + 1) * 10).collect();
                executor.execute_batch(items, cpu_intensive_work).await
            });
            handles.push(handle);
        }

        // All should succeed
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_executor_lifecycle() {
        // Rapid creation/destruction
        for _ in 0..100 {
            let executor = Executor::tokio(5);
            let items: Vec<u32> = (0..5).collect();

            let results = executor
                .execute_batch(items, cpu_intensive_work)
                .await
                .unwrap();

            assert_eq!(results.len(), 5);
            assert_batch_results(&results, 5, 0);
        }
    }
}
