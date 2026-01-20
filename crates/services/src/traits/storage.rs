// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # Storage Service Traits - Commercial Boundary
//!
//! Defines storage service interfaces that create clear commercial boundaries.
//! These traits are available for trait definitions in open source but
//! implementations are commercial-only features.

use async_trait::async_trait;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use crate::error::{ServiceResult, StorageError};
use crate::types::{AnalysisContext, CrossFileRelationship, ParsedDocument};
use thread_ast_engine::source::Doc;

/// Storage service trait for persisting analysis results and enabling advanced features.
///
/// This trait defines the commercial boundary for Thread. The trait definition
/// is available in open source for interface compatibility, but implementations
/// are commercial-only features that enable:
///
/// - Persistent analysis result caching
/// - Cross-session analysis state
/// - Advanced analytics and reporting
/// - Enterprise-scale data management
///
/// # Commercial Features
///
/// Implementations of this trait enable:
/// - **Analysis Persistence**: Store parsed documents and analysis results
/// - **Advanced Caching**: Intelligent caching strategies for large codebases
/// - **Analytics**: Usage tracking, performance metrics, and insights
/// - **Collaboration**: Share analysis results across team members
/// - **Compliance**: Audit trails and data governance features
///
/// # Usage Pattern
///
/// ```rust,no_run
/// // Open source: trait available for interface compatibility
/// use thread_services::traits::StorageService;
///
/// // Commercial: actual implementations available with license
/// #[cfg(feature = "commercial")]
/// use thread_commercial::PostgresStorageService;
///
/// async fn example() {
///     #[cfg(feature = "commercial")]
///     {
///         let storage: Box<dyn StorageService> = Box::new(
///             PostgresStorageService::new("connection_string").await.unwrap()
///         );
///         
///         // Store analysis results persistently
///         // storage.store_analysis_result(...).await.unwrap();
///     }
/// }
/// ```
#[async_trait]
pub trait StorageService: Send + Sync {
    /// Store analysis results persistently.
    ///
    /// Enables caching of expensive analysis operations across sessions
    /// and sharing results across team members.
    async fn store_analysis_result<D: Doc>(
        &self,
        key: &AnalysisKey,
        result: &AnalysisResult<D>,
        context: &AnalysisContext,
    ) -> ServiceResult<()>;

    /// Load cached analysis results.
    ///
    /// Retrieves previously stored analysis results to avoid recomputation
    /// and enable incremental analysis workflows.
    async fn load_analysis_result<D: Doc>(
        &self,
        key: &AnalysisKey,
        context: &AnalysisContext,
    ) -> ServiceResult<Option<AnalysisResult<D>>>;

    /// Store parsed document for caching.
    ///
    /// Enables persistent caching of expensive parsing operations,
    /// particularly valuable for large codebases.
    async fn store_parsed_document<D: Doc>(
        &self,
        document: &ParsedDocument<D>,
        context: &AnalysisContext,
    ) -> ServiceResult<StorageKey>;

    /// Load cached parsed document.
    ///
    /// Retrieves previously parsed and cached documents to avoid
    /// redundant parsing operations.
    async fn load_parsed_document<D: Doc>(
        &self,
        key: &StorageKey,
        context: &AnalysisContext,
    ) -> ServiceResult<Option<ParsedDocument<D>>>;

    /// Store cross-file relationships.
    ///
    /// Persists codebase-level graph intelligence for advanced analytics
    /// and cross-session analysis continuation.
    async fn store_relationships(
        &self,
        relationships: &[CrossFileRelationship],
        context: &AnalysisContext,
    ) -> ServiceResult<()>;

    /// Load cross-file relationships.
    ///
    /// Retrieves previously analyzed relationships to build on existing
    /// codebase intelligence and enable incremental updates.
    async fn load_relationships(
        &self,
        context: &AnalysisContext,
    ) -> ServiceResult<Vec<CrossFileRelationship>>;

    /// Get storage capabilities and configuration.
    fn capabilities(&self) -> StorageCapabilities;

    /// Perform storage maintenance operations.
    ///
    /// Includes cleanup, optimization, and health monitoring tasks
    /// for enterprise storage management.
    async fn maintenance(
        &self,
        operation: MaintenanceOperation,
    ) -> ServiceResult<MaintenanceResult>;

    /// Get storage statistics and metrics.
    ///
    /// Provides insights into storage usage, performance, and health
    /// for enterprise monitoring and analytics.
    async fn get_statistics(&self) -> ServiceResult<StorageStatistics>;
}

/// Cache service trait for high-performance caching strategies.
///
/// Provides advanced caching capabilities that are commercial features,
/// including intelligent cache invalidation, distributed caching,
/// and performance optimization strategies.
#[async_trait]
pub trait CacheService: Send + Sync {
    /// Store item in cache with TTL.
    async fn store<T: CacheableItem>(
        &self,
        key: &CacheKey,
        item: &T,
        ttl: Option<Duration>,
    ) -> ServiceResult<()>;

    /// Load item from cache.
    async fn load<T: CacheableItem>(&self, key: &CacheKey) -> ServiceResult<Option<T>>;

    /// Invalidate cache entries.
    async fn invalidate(&self, pattern: &CachePattern) -> ServiceResult<usize>;

    /// Get cache statistics.
    async fn get_cache_stats(&self) -> ServiceResult<CacheStatistics>;

    /// Perform cache maintenance.
    async fn maintenance(&self) -> ServiceResult<()>;
}

/// Analytics service trait for usage tracking and insights.
///
/// Commercial feature that provides detailed analytics, usage tracking,
/// and performance insights for enterprise deployments.
#[async_trait]
pub trait AnalyticsService: Send + Sync {
    /// Record analysis operation for tracking.
    async fn record_operation(
        &self,
        operation: &OperationRecord,
        context: &AnalysisContext,
    ) -> ServiceResult<()>;

    /// Get usage analytics.
    async fn get_analytics(&self, query: &AnalyticsQuery) -> ServiceResult<AnalyticsResult>;

    /// Get performance metrics.
    async fn get_performance_metrics(
        &self,
        period: &TimePeriod,
    ) -> ServiceResult<PerformanceMetrics>;

    /// Generate insights and recommendations.
    async fn generate_insights(&self, context: &AnalysisContext) -> ServiceResult<Vec<Insight>>;
}

// Storage-related types and configurations

/// Key for storing analysis results
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct AnalysisKey {
    pub operation_type: String,
    pub content_hash: u64,
    pub configuration_hash: u64,
    pub version: String,
}

/// Stored analysis result
#[derive(Debug, Clone)]
pub struct AnalysisResult<D: Doc> {
    pub documents: Vec<ParsedDocument<D>>,
    pub relationships: Vec<CrossFileRelationship>,
    pub metadata: HashMap<String, String>,
    pub timestamp: SystemTime,
    pub version: String,
}

/// Storage key for individual items
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct StorageKey {
    pub namespace: String,
    pub identifier: String,
    pub version: Option<String>,
}

/// Storage service capabilities
#[derive(Debug, Clone)]
pub struct StorageCapabilities {
    /// Maximum storage size per tenant
    pub max_storage_size: Option<u64>,

    /// Supported storage backends
    pub supported_backends: Vec<StorageBackend>,

    /// Whether distributed storage is supported
    pub supports_distributed: bool,

    /// Whether encryption at rest is supported
    pub supports_encryption: bool,

    /// Whether backup/restore is supported
    pub supports_backup: bool,

    /// Whether multi-tenancy is supported
    pub supports_multi_tenancy: bool,

    /// Performance characteristics
    pub performance_profile: StoragePerformanceProfile,
}

/// Storage backend types
#[derive(Debug, Clone, PartialEq)]
pub enum StorageBackend {
    PostgreSQL,
    Redis,
    S3,
    FileSystem,
    InMemory,
    Custom(String),
}

/// Storage performance profile
#[derive(Debug, Clone, PartialEq)]
pub enum StoragePerformanceProfile {
    HighThroughput,
    LowLatency,
    Balanced,
    CostOptimized,
}

/// Maintenance operations
#[derive(Debug, Clone)]
pub enum MaintenanceOperation {
    Cleanup { older_than: Duration },
    Optimize,
    Backup { destination: String },
    Restore { source: String },
    HealthCheck,
    Vacuum,
}

/// Maintenance operation result
#[derive(Debug, Clone)]
pub struct MaintenanceResult {
    pub operation: MaintenanceOperation,
    pub success: bool,
    pub message: String,
    pub metrics: HashMap<String, f64>,
    pub duration: Duration,
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStatistics {
    pub total_size: u64,
    pub total_items: u64,
    pub cache_hit_rate: f64,
    pub average_response_time: Duration,
    pub error_rate: f64,
    pub last_updated: SystemTime,
}

// Cache-related types

/// Cache key for items
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CacheKey {
    pub namespace: String,
    pub key: String,
}

/// Cache pattern for bulk operations
#[derive(Debug, Clone)]
pub struct CachePattern {
    pub namespace: Option<String>,
    pub key_pattern: String,
}

/// Trait for items that can be cached
pub trait CacheableItem: Send + Sync {
    fn serialize(&self) -> ServiceResult<Vec<u8>>;
    fn deserialize(data: &[u8]) -> ServiceResult<Self>
    where
        Self: Sized;
    fn cache_key(&self) -> String;
    fn ttl(&self) -> Option<Duration>;
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub total_items: u64,
    pub hit_rate: f64,
    pub miss_rate: f64,
    pub eviction_rate: f64,
    pub memory_usage: u64,
    pub last_updated: SystemTime,
}

// Analytics-related types

/// Record of an analysis operation
#[derive(Debug, Clone)]
pub struct OperationRecord {
    pub operation_type: String,
    pub duration: Duration,
    pub files_processed: usize,
    pub patterns_used: Vec<String>,
    pub success: bool,
    pub error_type: Option<String>,
    pub user_id: Option<String>,
    pub timestamp: SystemTime,
}

/// Analytics query parameters
#[derive(Debug, Clone)]
pub struct AnalyticsQuery {
    pub time_period: TimePeriod,
    pub operation_types: Option<Vec<String>>,
    pub user_ids: Option<Vec<String>>,
    pub aggregation_level: AggregationLevel,
}

/// Time period for queries
#[derive(Debug, Clone)]
pub struct TimePeriod {
    pub start: SystemTime,
    pub end: SystemTime,
}

/// Aggregation level for analytics
#[derive(Debug, Clone, PartialEq)]
pub enum AggregationLevel {
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

/// Analytics query result
#[derive(Debug, Clone)]
pub struct AnalyticsResult {
    pub query: AnalyticsQuery,
    pub data_points: Vec<AnalyticsDataPoint>,
    pub summary: AnalyticsSummary,
}

/// Individual analytics data point
#[derive(Debug, Clone)]
pub struct AnalyticsDataPoint {
    pub timestamp: SystemTime,
    pub operation_count: u64,
    pub success_rate: f64,
    pub average_duration: Duration,
    pub files_processed: u64,
}

/// Analytics summary
#[derive(Debug, Clone)]
pub struct AnalyticsSummary {
    pub total_operations: u64,
    pub overall_success_rate: f64,
    pub average_duration: Duration,
    pub peak_usage: SystemTime,
    pub most_common_operations: Vec<String>,
}

/// Performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub period: TimePeriod,
    pub throughput: f64,                                // operations per second
    pub latency_percentiles: HashMap<String, Duration>, // p50, p95, p99
    pub error_rates: HashMap<String, f64>,
    pub resource_usage: ResourceUsage,
}

/// Resource usage metrics
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub storage_usage: u64,
    pub network_io: u64,
}

/// Generated insight
#[derive(Debug, Clone)]
pub struct Insight {
    pub insight_type: InsightType,
    pub title: String,
    pub description: String,
    pub severity: InsightSeverity,
    pub recommendations: Vec<String>,
    pub confidence: f64,
}

/// Types of insights
#[derive(Debug, Clone, PartialEq)]
pub enum InsightType {
    Performance,
    Usage,
    Optimization,
    Security,
    Maintenance,
}

/// Insight severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum InsightSeverity {
    Info,
    Warning,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_key() {
        let key = AnalysisKey {
            operation_type: "pattern_match".to_string(),
            content_hash: 12345,
            configuration_hash: 67890,
            version: "1.0".to_string(),
        };

        assert_eq!(key.operation_type, "pattern_match");
        assert_eq!(key.content_hash, 12345);
    }

    #[test]
    fn test_storage_capabilities() {
        let caps = StorageCapabilities {
            max_storage_size: Some(1024 * 1024 * 1024), // 1GB
            supported_backends: vec![StorageBackend::PostgreSQL, StorageBackend::Redis],
            supports_distributed: true,
            supports_encryption: true,
            supports_backup: true,
            supports_multi_tenancy: true,
            performance_profile: StoragePerformanceProfile::Balanced,
        };

        assert!(caps.supports_encryption);
        assert!(caps.supports_backup);
        assert_eq!(
            caps.performance_profile,
            StoragePerformanceProfile::Balanced
        );
    }
}
