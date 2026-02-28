// SPDX-FileCopyrightText: 2026 Knitli Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Performance monitoring and metrics collection
//!
//! Integrates with Prometheus to track:
//! - Fingerprint computation latency
//! - Cache hit/miss rates
//! - Query execution times
//! - Memory usage
//! - Throughput metrics

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Performance metrics collector
#[derive(Clone)]
pub struct PerformanceMetrics {
    // Fingerprint metrics
    fingerprint_total: Arc<AtomicU64>,
    fingerprint_duration_ns: Arc<AtomicU64>,

    // Cache metrics
    cache_hits: Arc<AtomicU64>,
    cache_misses: Arc<AtomicU64>,
    cache_evictions: Arc<AtomicU64>,

    // Query metrics
    query_count: Arc<AtomicU64>,
    query_duration_ns: Arc<AtomicU64>,
    query_errors: Arc<AtomicU64>,

    // Memory metrics
    bytes_processed: Arc<AtomicU64>,
    allocations: Arc<AtomicU64>,

    // Throughput metrics
    files_processed: Arc<AtomicU64>,
    batch_count: Arc<AtomicU64>,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMetrics {
    /// Create new performance metrics collector
    pub fn new() -> Self {
        Self {
            fingerprint_total: Arc::new(AtomicU64::new(0)),
            fingerprint_duration_ns: Arc::new(AtomicU64::new(0)),
            cache_hits: Arc::new(AtomicU64::new(0)),
            cache_misses: Arc::new(AtomicU64::new(0)),
            cache_evictions: Arc::new(AtomicU64::new(0)),
            query_count: Arc::new(AtomicU64::new(0)),
            query_duration_ns: Arc::new(AtomicU64::new(0)),
            query_errors: Arc::new(AtomicU64::new(0)),
            bytes_processed: Arc::new(AtomicU64::new(0)),
            allocations: Arc::new(AtomicU64::new(0)),
            files_processed: Arc::new(AtomicU64::new(0)),
            batch_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Record fingerprint computation
    pub fn record_fingerprint(&self, duration: Duration) {
        self.fingerprint_total.fetch_add(1, Ordering::Relaxed);
        self.fingerprint_duration_ns
            .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
    }

    /// Record cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record cache eviction
    pub fn record_cache_eviction(&self) {
        self.cache_evictions.fetch_add(1, Ordering::Relaxed);
    }

    /// Record query execution
    pub fn record_query(&self, duration: Duration, success: bool) {
        self.query_count.fetch_add(1, Ordering::Relaxed);
        self.query_duration_ns
            .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
        if !success {
            self.query_errors.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Record bytes processed
    pub fn record_bytes(&self, bytes: u64) {
        self.bytes_processed.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Record memory allocation
    pub fn record_allocation(&self) {
        self.allocations.fetch_add(1, Ordering::Relaxed);
    }

    /// Record file processed
    pub fn record_file_processed(&self) {
        self.files_processed.fetch_add(1, Ordering::Relaxed);
    }

    /// Record batch processing
    pub fn record_batch(&self, file_count: u64) {
        self.batch_count.fetch_add(1, Ordering::Relaxed);
        self.files_processed
            .fetch_add(file_count, Ordering::Relaxed);
    }

    /// Get fingerprint statistics
    pub fn fingerprint_stats(&self) -> FingerprintStats {
        let total = self.fingerprint_total.load(Ordering::Relaxed);
        let duration_ns = self.fingerprint_duration_ns.load(Ordering::Relaxed);

        let avg_ns = duration_ns.checked_div(total).unwrap_or(0);

        FingerprintStats {
            total_count: total,
            total_duration_ns: duration_ns,
            avg_duration_ns: avg_ns,
        }
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;

        let hit_rate = if total > 0 {
            (hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        CacheStats {
            hits,
            misses,
            evictions: self.cache_evictions.load(Ordering::Relaxed),
            hit_rate_percent: hit_rate,
        }
    }

    /// Get query statistics
    pub fn query_stats(&self) -> QueryStats {
        let count = self.query_count.load(Ordering::Relaxed);
        let duration_ns = self.query_duration_ns.load(Ordering::Relaxed);
        let errors = self.query_errors.load(Ordering::Relaxed);

        let avg_ns = duration_ns.checked_div(count).unwrap_or(0);
        let error_rate = if count > 0 {
            (errors as f64 / count as f64) * 100.0
        } else {
            0.0
        };

        QueryStats {
            total_count: count,
            total_duration_ns: duration_ns,
            avg_duration_ns: avg_ns,
            errors,
            error_rate_percent: error_rate,
        }
    }

    /// Get throughput statistics
    pub fn throughput_stats(&self) -> ThroughputStats {
        ThroughputStats {
            bytes_processed: self.bytes_processed.load(Ordering::Relaxed),
            files_processed: self.files_processed.load(Ordering::Relaxed),
            batches_processed: self.batch_count.load(Ordering::Relaxed),
        }
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.fingerprint_total.store(0, Ordering::Relaxed);
        self.fingerprint_duration_ns.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
        self.cache_evictions.store(0, Ordering::Relaxed);
        self.query_count.store(0, Ordering::Relaxed);
        self.query_duration_ns.store(0, Ordering::Relaxed);
        self.query_errors.store(0, Ordering::Relaxed);
        self.bytes_processed.store(0, Ordering::Relaxed);
        self.allocations.store(0, Ordering::Relaxed);
        self.files_processed.store(0, Ordering::Relaxed);
        self.batch_count.store(0, Ordering::Relaxed);
    }

    /// Export metrics in Prometheus format
    pub fn export_prometheus(&self) -> String {
        let fingerprint = self.fingerprint_stats();
        let cache = self.cache_stats();
        let query = self.query_stats();
        let throughput = self.throughput_stats();

        format!(
            r#"# HELP thread_fingerprint_total Total fingerprint computations
# TYPE thread_fingerprint_total counter
thread_fingerprint_total {}

# HELP thread_fingerprint_duration_seconds Total fingerprint computation time
# TYPE thread_fingerprint_duration_seconds counter
thread_fingerprint_duration_seconds {}

# HELP thread_fingerprint_avg_duration_seconds Average fingerprint computation time
# TYPE thread_fingerprint_avg_duration_seconds gauge
thread_fingerprint_avg_duration_seconds {}

# HELP thread_cache_hits_total Total cache hits
# TYPE thread_cache_hits_total counter
thread_cache_hits_total {}

# HELP thread_cache_misses_total Total cache misses
# TYPE thread_cache_misses_total counter
thread_cache_misses_total {}

# HELP thread_cache_evictions_total Total cache evictions
# TYPE thread_cache_evictions_total counter
thread_cache_evictions_total {}

# HELP thread_cache_hit_rate_percent Cache hit rate percentage
# TYPE thread_cache_hit_rate_percent gauge
thread_cache_hit_rate_percent {}

# HELP thread_query_total Total queries executed
# TYPE thread_query_total counter
thread_query_total {}

# HELP thread_query_duration_seconds Total query execution time
# TYPE thread_query_duration_seconds counter
thread_query_duration_seconds {}

# HELP thread_query_avg_duration_seconds Average query execution time
# TYPE thread_query_avg_duration_seconds gauge
thread_query_avg_duration_seconds {}

# HELP thread_query_errors_total Total query errors
# TYPE thread_query_errors_total counter
thread_query_errors_total {}

# HELP thread_query_error_rate_percent Query error rate percentage
# TYPE thread_query_error_rate_percent gauge
thread_query_error_rate_percent {}

# HELP thread_bytes_processed_total Total bytes processed
# TYPE thread_bytes_processed_total counter
thread_bytes_processed_total {}

# HELP thread_files_processed_total Total files processed
# TYPE thread_files_processed_total counter
thread_files_processed_total {}

# HELP thread_batches_processed_total Total batches processed
# TYPE thread_batches_processed_total counter
thread_batches_processed_total {}
"#,
            fingerprint.total_count,
            fingerprint.total_duration_ns as f64 / 1_000_000_000.0,
            fingerprint.avg_duration_ns as f64 / 1_000_000_000.0,
            cache.hits,
            cache.misses,
            cache.evictions,
            cache.hit_rate_percent,
            query.total_count,
            query.total_duration_ns as f64 / 1_000_000_000.0,
            query.avg_duration_ns as f64 / 1_000_000_000.0,
            query.errors,
            query.error_rate_percent,
            throughput.bytes_processed,
            throughput.files_processed,
            throughput.batches_processed,
        )
    }
}

/// Fingerprint computation statistics
#[derive(Debug, Clone)]
pub struct FingerprintStats {
    pub total_count: u64,
    pub total_duration_ns: u64,
    pub avg_duration_ns: u64,
}

/// Cache performance statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub hit_rate_percent: f64,
}

/// Query execution statistics
#[derive(Debug, Clone)]
pub struct QueryStats {
    pub total_count: u64,
    pub total_duration_ns: u64,
    pub avg_duration_ns: u64,
    pub errors: u64,
    pub error_rate_percent: f64,
}

/// Throughput statistics
#[derive(Debug, Clone)]
pub struct ThroughputStats {
    pub bytes_processed: u64,
    pub files_processed: u64,
    pub batches_processed: u64,
}

/// Performance timer for automatic metric recording
pub struct PerformanceTimer<'a> {
    metrics: &'a PerformanceMetrics,
    metric_type: MetricType,
    start: Instant,
}

/// Type of metric being timed
pub enum MetricType {
    Fingerprint,
    Query,
}

impl<'a> PerformanceTimer<'a> {
    /// Start a new performance timer
    pub fn start(metrics: &'a PerformanceMetrics, metric_type: MetricType) -> Self {
        Self {
            metrics,
            metric_type,
            start: Instant::now(),
        }
    }

    /// Stop the timer and record the duration (success)
    pub fn stop_success(self) {
        let duration = self.start.elapsed();
        match self.metric_type {
            MetricType::Fingerprint => self.metrics.record_fingerprint(duration),
            MetricType::Query => self.metrics.record_query(duration, true),
        }
    }

    /// Stop the timer and record the duration (error)
    pub fn stop_error(self) {
        let duration = self.start.elapsed();
        if let MetricType::Query = self.metric_type {
            self.metrics.record_query(duration, false)
        }
    }
}

impl<'a> Drop for PerformanceTimer<'a> {
    fn drop(&mut self) {
        // Auto-record on drop (assumes success)
        let duration = self.start.elapsed();
        match self.metric_type {
            MetricType::Fingerprint => self.metrics.record_fingerprint(duration),
            MetricType::Query => self.metrics.record_query(duration, true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_fingerprint_metrics() {
        let metrics = PerformanceMetrics::new();

        // Record some fingerprints
        metrics.record_fingerprint(Duration::from_nanos(500));
        metrics.record_fingerprint(Duration::from_nanos(1000));
        metrics.record_fingerprint(Duration::from_nanos(1500));

        let stats = metrics.fingerprint_stats();
        assert_eq!(stats.total_count, 3);
        assert_eq!(stats.total_duration_ns, 3000);
        assert_eq!(stats.avg_duration_ns, 1000);
    }

    #[test]
    fn test_cache_metrics() {
        let metrics = PerformanceMetrics::new();

        // Record cache activity
        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_miss();
        metrics.record_cache_eviction();

        let stats = metrics.cache_stats();
        assert_eq!(stats.hits, 3);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.evictions, 1);
        assert_eq!(stats.hit_rate_percent, 75.0);
    }

    #[test]
    fn test_query_metrics() {
        let metrics = PerformanceMetrics::new();

        // Record queries
        metrics.record_query(Duration::from_millis(10), true);
        metrics.record_query(Duration::from_millis(20), true);
        metrics.record_query(Duration::from_millis(15), false);

        let stats = metrics.query_stats();
        assert_eq!(stats.total_count, 3);
        assert_eq!(stats.errors, 1);
        assert!((stats.error_rate_percent - 33.33).abs() < 0.1);
    }

    #[test]
    fn test_throughput_metrics() {
        let metrics = PerformanceMetrics::new();

        metrics.record_bytes(1024);
        metrics.record_file_processed();
        metrics.record_batch(10);

        let stats = metrics.throughput_stats();
        assert_eq!(stats.bytes_processed, 1024);
        assert_eq!(stats.files_processed, 11); // 1 + 10 from batch
        assert_eq!(stats.batches_processed, 1);
    }

    #[test]
    fn test_performance_timer() {
        let metrics = PerformanceMetrics::new();

        {
            let _timer = PerformanceTimer::start(&metrics, MetricType::Fingerprint);
            thread::sleep(Duration::from_millis(1));
        }

        let stats = metrics.fingerprint_stats();
        assert_eq!(stats.total_count, 1);
        assert!(stats.avg_duration_ns >= 1_000_000); // At least 1ms
    }

    #[test]
    fn test_metrics_reset() {
        let metrics = PerformanceMetrics::new();

        metrics.record_fingerprint(Duration::from_nanos(500));
        metrics.record_cache_hit();
        metrics.record_query(Duration::from_millis(10), true);

        metrics.reset();

        let fp_stats = metrics.fingerprint_stats();
        let cache_stats = metrics.cache_stats();
        let query_stats = metrics.query_stats();

        assert_eq!(fp_stats.total_count, 0);
        assert_eq!(cache_stats.hits, 0);
        assert_eq!(query_stats.total_count, 0);
    }

    #[test]
    fn test_prometheus_export() {
        let metrics = PerformanceMetrics::new();

        metrics.record_fingerprint(Duration::from_nanos(500));
        metrics.record_cache_hit();

        let export = metrics.export_prometheus();

        assert!(export.contains("thread_fingerprint_total 1"));
        assert!(export.contains("thread_cache_hits_total 1"));
        assert!(export.contains("# HELP"));
        assert!(export.contains("# TYPE"));
    }
}
