// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # Thread Flow Monitoring
//!
//! Production-ready monitoring and observability infrastructure for Thread Flow.
//!
//! ## Features
//!
//! - **Metrics Collection**: Prometheus-compatible metrics for cache, latency, throughput
//! - **Structured Logging**: JSON and human-readable logging with tracing
//! - **Performance Tracking**: Real-time performance metrics and alerts
//! - **Error Tracking**: Error rates and error type categorization
//!
//! ## Usage
//!
//! ```rust,ignore
//! use thread_flow::monitoring::{Metrics, init_logging};
//!
//! // Initialize logging
//! init_logging(LogLevel::Info, LogFormat::Json)?;
//!
//! // Create metrics collector
//! let metrics = Metrics::new();
//!
//! // Track operations
//! metrics.record_cache_hit();
//! metrics.record_query_latency(15);  // 15ms
//! metrics.record_fingerprint_time(425);  // 425ns
//!
//! // Get statistics
//! let stats = metrics.snapshot();
//! println!("Cache hit rate: {:.2}%", stats.cache_hit_rate());
//! ```
//!
//! ## Metrics Tracked
//!
//! ### Cache Metrics
//! - `cache_hits` - Total cache hits
//! - `cache_misses` - Total cache misses
//! - `cache_hit_rate` - Hit rate percentage (target: >90%)
//!
//! ### Latency Metrics (in milliseconds)
//! - `query_latency_p50` - Median query latency
//! - `query_latency_p95` - 95th percentile query latency
//! - `query_latency_p99` - 99th percentile query latency
//!
//! ### Performance Metrics
//! - `fingerprint_time_ns` - Blake3 fingerprinting time in nanoseconds
//! - `parse_time_us` - Tree-sitter parsing time in microseconds
//! - `extract_time_us` - Symbol extraction time in microseconds
//!
//! ### Throughput Metrics
//! - `files_processed_total` - Total files processed
//! - `symbols_extracted_total` - Total symbols extracted
//! - `throughput_files_per_sec` - Files processed per second
//!
//! ### Error Metrics
//! - `errors_total` - Total errors by type
//! - `error_rate` - Error rate percentage

pub mod logging;
pub mod performance;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use thread_utils::RapidMap;

/// Metrics collector for Thread Flow operations
#[derive(Clone)]
pub struct Metrics {
    inner: Arc<MetricsInner>,
}

struct MetricsInner {
    // Cache metrics
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,

    // Latency tracking (microseconds)
    query_latencies: RwLock<Vec<u64>>,
    fingerprint_times: RwLock<Vec<u64>>,
    parse_times: RwLock<Vec<u64>>,

    // Throughput tracking
    files_processed: AtomicU64,
    symbols_extracted: AtomicU64,
    start_time: Instant,

    // Error tracking
    errors_by_type: RwLock<RapidMap<String, u64>>,
}

impl Metrics {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            inner: Arc::new(MetricsInner {
                cache_hits: AtomicU64::new(0),
                cache_misses: AtomicU64::new(0),
                query_latencies: RwLock::new(Vec::new()),
                fingerprint_times: RwLock::new(Vec::new()),
                parse_times: RwLock::new(Vec::new()),
                files_processed: AtomicU64::new(0),
                symbols_extracted: AtomicU64::new(0),
                start_time: Instant::now(),
                errors_by_type: RwLock::new(thread_utils::get_map()),
            }),
        }
    }

    /// Record a cache hit
    pub fn record_cache_hit(&self) {
        self.inner.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss
    pub fn record_cache_miss(&self) {
        self.inner.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record query latency in milliseconds
    pub fn record_query_latency(&self, latency_ms: u64) {
        if let Ok(mut latencies) = self.inner.query_latencies.write() {
            latencies.push(latency_ms);
            // Keep only last 10,000 samples to prevent unbounded growth
            if latencies.len() > 10_000 {
                latencies.drain(0..5_000);
            }
        }
    }

    /// Record fingerprint computation time in nanoseconds
    pub fn record_fingerprint_time(&self, time_ns: u64) {
        if let Ok(mut times) = self.inner.fingerprint_times.write() {
            times.push(time_ns);
            if times.len() > 10_000 {
                times.drain(0..5_000);
            }
        }
    }

    /// Record parse time in microseconds
    pub fn record_parse_time(&self, time_us: u64) {
        if let Ok(mut times) = self.inner.parse_times.write() {
            times.push(time_us);
            if times.len() > 10_000 {
                times.drain(0..5_000);
            }
        }
    }

    /// Record files processed
    pub fn record_files_processed(&self, count: u64) {
        self.inner
            .files_processed
            .fetch_add(count, Ordering::Relaxed);
    }

    /// Record symbols extracted
    pub fn record_symbols_extracted(&self, count: u64) {
        self.inner
            .symbols_extracted
            .fetch_add(count, Ordering::Relaxed);
    }

    /// Record an error by type
    pub fn record_error(&self, error_type: impl Into<String>) {
        if let Ok(mut errors) = self.inner.errors_by_type.write() {
            *errors.entry(error_type.into()).or_insert(0) += 1;
        }
    }

    /// Get a snapshot of current metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        let cache_hits = self.inner.cache_hits.load(Ordering::Relaxed);
        let cache_misses = self.inner.cache_misses.load(Ordering::Relaxed);
        let total_cache_lookups = cache_hits + cache_misses;

        let cache_hit_rate = if total_cache_lookups > 0 {
            (cache_hits as f64 / total_cache_lookups as f64) * 100.0
        } else {
            0.0
        };

        // Calculate percentiles
        let query_latencies = self
            .inner
            .query_latencies
            .read()
            .ok()
            .map(|l| calculate_percentiles(&l))
            .unwrap_or_default();

        let fingerprint_times = self
            .inner
            .fingerprint_times
            .read()
            .ok()
            .map(|t| calculate_percentiles(&t))
            .unwrap_or_default();

        let parse_times = self
            .inner
            .parse_times
            .read()
            .ok()
            .map(|t| calculate_percentiles(&t))
            .unwrap_or_default();

        let files_processed = self.inner.files_processed.load(Ordering::Relaxed);
        let symbols_extracted = self.inner.symbols_extracted.load(Ordering::Relaxed);
        let elapsed = self.inner.start_time.elapsed();

        let throughput_files_per_sec = if elapsed.as_secs() > 0 {
            files_processed as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        let errors_by_type = self
            .inner
            .errors_by_type
            .read()
            .ok()
            .map(|e| e.clone())
            .unwrap_or_default();

        let total_errors: u64 = errors_by_type.values().sum();
        let error_rate = if files_processed > 0 {
            (total_errors as f64 / files_processed as f64) * 100.0
        } else {
            0.0
        };

        MetricsSnapshot {
            cache_hits,
            cache_misses,
            cache_hit_rate,
            query_latency_p50: query_latencies.p50,
            query_latency_p95: query_latencies.p95,
            query_latency_p99: query_latencies.p99,
            fingerprint_time_p50: fingerprint_times.p50,
            fingerprint_time_p95: fingerprint_times.p95,
            parse_time_p50: parse_times.p50,
            parse_time_p95: parse_times.p95,
            files_processed,
            symbols_extracted,
            throughput_files_per_sec,
            errors_by_type,
            error_rate,
            uptime: elapsed,
        }
    }

    /// Export metrics in Prometheus format
    pub fn export_prometheus(&self) -> String {
        let snapshot = self.snapshot();
        format!(
            r#"# HELP thread_cache_hits_total Total number of cache hits
# TYPE thread_cache_hits_total counter
thread_cache_hits_total {}

# HELP thread_cache_misses_total Total number of cache misses
# TYPE thread_cache_misses_total counter
thread_cache_misses_total {}

# HELP thread_cache_hit_rate Cache hit rate percentage
# TYPE thread_cache_hit_rate gauge
thread_cache_hit_rate {:.2}

# HELP thread_query_latency_milliseconds Query latency in milliseconds
# TYPE thread_query_latency_milliseconds summary
thread_query_latency_milliseconds{{quantile="0.5"}} {}
thread_query_latency_milliseconds{{quantile="0.95"}} {}
thread_query_latency_milliseconds{{quantile="0.99"}} {}

# HELP thread_fingerprint_time_nanoseconds Fingerprint computation time in nanoseconds
# TYPE thread_fingerprint_time_nanoseconds summary
thread_fingerprint_time_nanoseconds{{quantile="0.5"}} {}
thread_fingerprint_time_nanoseconds{{quantile="0.95"}} {}

# HELP thread_parse_time_microseconds Parse time in microseconds
# TYPE thread_parse_time_microseconds summary
thread_parse_time_microseconds{{quantile="0.5"}} {}
thread_parse_time_microseconds{{quantile="0.95"}} {}

# HELP thread_files_processed_total Total files processed
# TYPE thread_files_processed_total counter
thread_files_processed_total {}

# HELP thread_symbols_extracted_total Total symbols extracted
# TYPE thread_symbols_extracted_total counter
thread_symbols_extracted_total {}

# HELP thread_throughput_files_per_second Files processed per second
# TYPE thread_throughput_files_per_second gauge
thread_throughput_files_per_second {:.2}

# HELP thread_error_rate Error rate percentage
# TYPE thread_error_rate gauge
thread_error_rate {:.2}
"#,
            snapshot.cache_hits,
            snapshot.cache_misses,
            snapshot.cache_hit_rate,
            snapshot.query_latency_p50,
            snapshot.query_latency_p95,
            snapshot.query_latency_p99,
            snapshot.fingerprint_time_p50,
            snapshot.fingerprint_time_p95,
            snapshot.parse_time_p50,
            snapshot.parse_time_p95,
            snapshot.files_processed,
            snapshot.symbols_extracted,
            snapshot.throughput_files_per_sec,
            snapshot.error_rate,
        )
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.inner.cache_hits.store(0, Ordering::Relaxed);
        self.inner.cache_misses.store(0, Ordering::Relaxed);
        self.inner.files_processed.store(0, Ordering::Relaxed);
        self.inner.symbols_extracted.store(0, Ordering::Relaxed);

        if let Ok(mut latencies) = self.inner.query_latencies.write() {
            latencies.clear();
        }
        if let Ok(mut times) = self.inner.fingerprint_times.write() {
            times.clear();
        }
        if let Ok(mut times) = self.inner.parse_times.write() {
            times.clear();
        }
        if let Ok(mut errors) = self.inner.errors_by_type.write() {
            errors.clear();
        }
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of metrics at a point in time
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    // Cache metrics
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_hit_rate: f64,

    // Latency metrics (milliseconds)
    pub query_latency_p50: u64,
    pub query_latency_p95: u64,
    pub query_latency_p99: u64,

    // Performance metrics
    pub fingerprint_time_p50: u64, // nanoseconds
    pub fingerprint_time_p95: u64, // nanoseconds
    pub parse_time_p50: u64,       // microseconds
    pub parse_time_p95: u64,       // microseconds

    // Throughput metrics
    pub files_processed: u64,
    pub symbols_extracted: u64,
    pub throughput_files_per_sec: f64,

    // Error metrics
    pub errors_by_type: RapidMap<String, u64>,
    pub error_rate: f64,

    // System metrics
    pub uptime: Duration,
}

impl MetricsSnapshot {
    /// Check if metrics meet production SLOs
    pub fn meets_slo(&self) -> SLOStatus {
        let mut violations = Vec::new();

        // Cache hit rate SLO: >90%
        if self.cache_hit_rate < 90.0 {
            violations.push(format!(
                "Cache hit rate {:.2}% below SLO (90%)",
                self.cache_hit_rate
            ));
        }

        // Query latency SLO: p95 <10ms (CLI), <50ms (Edge)
        // Assume CLI for now - could make this configurable
        if self.query_latency_p95 > 50 {
            violations.push(format!(
                "Query p95 latency {}ms above SLO (50ms)",
                self.query_latency_p95
            ));
        }

        // Error rate SLO: <1%
        if self.error_rate > 1.0 {
            violations.push(format!("Error rate {:.2}% above SLO (1%)", self.error_rate));
        }

        if violations.is_empty() {
            SLOStatus::Healthy
        } else {
            SLOStatus::Violated(violations)
        }
    }

    /// Format metrics as human-readable text
    pub fn format_text(&self) -> String {
        format!(
            r#"Thread Flow Metrics
==================

Cache Performance:
  Hits: {} | Misses: {} | Hit Rate: {:.2}%

Query Latency (ms):
  p50: {} | p95: {} | p99: {}

Performance (Blake3 fingerprint in ns, parse in µs):
  Fingerprint p50: {}ns | p95: {}ns
  Parse p50: {}µs | p95: {}µs

Throughput:
  Files Processed: {}
  Symbols Extracted: {}
  Files/sec: {:.2}

Errors:
  Total Errors: {} ({:.2}% rate)
  By Type: {:?}

Uptime: {:.2}s
"#,
            self.cache_hits,
            self.cache_misses,
            self.cache_hit_rate,
            self.query_latency_p50,
            self.query_latency_p95,
            self.query_latency_p99,
            self.fingerprint_time_p50,
            self.fingerprint_time_p95,
            self.parse_time_p50,
            self.parse_time_p95,
            self.files_processed,
            self.symbols_extracted,
            self.throughput_files_per_sec,
            self.errors_by_type.values().sum::<u64>(),
            self.error_rate,
            self.errors_by_type,
            self.uptime.as_secs_f64(),
        )
    }
}

/// SLO compliance status
#[derive(Debug, Clone, PartialEq)]
pub enum SLOStatus {
    /// All SLOs are met
    Healthy,
    /// One or more SLOs are violated
    Violated(Vec<String>),
}

/// Helper struct for percentile calculations
#[derive(Debug, Default)]
struct Percentiles {
    p50: u64,
    p95: u64,
    p99: u64,
}

/// Calculate percentiles from a sorted list
fn calculate_percentiles(values: &[u64]) -> Percentiles {
    if values.is_empty() {
        return Percentiles::default();
    }

    let mut sorted = values.to_vec();
    sorted.sort_unstable();

    let p50_idx = (sorted.len() as f64 * 0.50) as usize;
    let p95_idx = (sorted.len() as f64 * 0.95) as usize;
    let p99_idx = (sorted.len() as f64 * 0.99) as usize;

    Percentiles {
        p50: sorted.get(p50_idx).copied().unwrap_or(0),
        p95: sorted.get(p95_idx).copied().unwrap_or(0),
        p99: sorted.get(p99_idx).copied().unwrap_or(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_cache_tracking() {
        let metrics = Metrics::new();

        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_miss();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.cache_hits, 2);
        assert_eq!(snapshot.cache_misses, 1);
        assert_eq!(snapshot.cache_hit_rate, 66.66666666666666);
    }

    #[test]
    fn test_metrics_latency_percentiles() {
        let metrics = Metrics::new();

        // Record latencies: 10, 20, 30, 40, 50, 60, 70, 80, 90, 100
        for i in 1..=10 {
            metrics.record_query_latency(i * 10);
        }

        let snapshot = metrics.snapshot();
        // With 10 values, p50_idx = (10 * 0.50) as usize = 5, sorted[5] = 60
        assert_eq!(snapshot.query_latency_p50, 60);
        assert_eq!(snapshot.query_latency_p95, 100);
        assert_eq!(snapshot.query_latency_p99, 100);
    }

    #[test]
    fn test_metrics_slo_compliance() {
        let metrics = Metrics::new();

        // Good metrics (meet SLO)
        for _ in 0..95 {
            metrics.record_cache_hit();
        }
        for _ in 0..5 {
            metrics.record_cache_miss();
        }
        metrics.record_query_latency(5);
        metrics.record_files_processed(100);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.meets_slo(), SLOStatus::Healthy);

        // Bad metrics (violate SLO)
        metrics.reset();
        for _ in 0..50 {
            metrics.record_cache_hit();
        }
        for _ in 0..50 {
            metrics.record_cache_miss();
        }

        let snapshot = metrics.snapshot();
        assert!(matches!(snapshot.meets_slo(), SLOStatus::Violated(_)));
    }

    #[test]
    fn test_prometheus_export() {
        let metrics = Metrics::new();
        metrics.record_cache_hit();
        metrics.record_files_processed(10);

        let prometheus = metrics.export_prometheus();
        assert!(prometheus.contains("thread_cache_hits_total 1"));
        assert!(prometheus.contains("thread_files_processed_total 10"));
    }

    #[test]
    fn test_metrics_reset() {
        let metrics = Metrics::new();
        metrics.record_cache_hit();
        metrics.record_files_processed(10);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.cache_hits, 1);
        assert_eq!(snapshot.files_processed, 10);

        metrics.reset();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.cache_hits, 0);
        assert_eq!(snapshot.files_processed, 0);
    }
}
