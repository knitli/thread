//! RPC Type Definitions for Real-Time Code Graph Intelligence
//!
//! These types are shared across CLI and Edge deployments for API consistency.
//! Serialization uses `serde` + `postcard` for binary efficiency (~40% size reduction vs JSON).
//!
//! **Protocol**: Custom RPC over HTTP + WebSockets (gRPC not viable for Cloudflare Workers)
//! **Transport**: HTTP POST for request/response, WebSocket for real-time streaming
//! **Serialization**: postcard (binary) for production, JSON for debugging

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// ============================================================================
// Core RPC Trait
// ============================================================================

/// RPC interface for code analysis operations
///
/// Implemented by both `NativeService` (CLI with tokio) and `EdgeService` (Cloudflare Workers)
#[async_trait::async_trait]
pub trait CodeAnalysisRpc {
    /// Analyze a single file and return graph nodes
    async fn analyze_file(&self, req: AnalyzeFileRequest) -> Result<AnalyzeFileResponse, RpcError>;

    /// Query the code graph for dependencies, callers, etc.
    async fn query_graph(&self, req: GraphQueryRequest) -> Result<GraphQueryResponse, RpcError>;

    /// Search for similar code patterns (semantic search)
    async fn search_similar(&self, req: SimilaritySearchRequest) -> Result<SimilaritySearchResponse, RpcError>;

    /// Detect conflicts between code changes
    async fn detect_conflicts(&self, req: ConflictDetectionRequest) -> Result<ConflictDetectionResponse, RpcError>;

    /// Get analysis session status
    async fn get_session_status(&self, session_id: String) -> Result<SessionStatus, RpcError>;

    /// Stream real-time updates (returns WebSocket stream)
    /// Note: Implemented via separate WebSocket endpoint, not direct RPC
    async fn subscribe_updates(&self, repo_id: String) -> Result<UpdateSubscription, RpcError>;
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Analyze a file and extract graph nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeFileRequest {
    pub file_path: PathBuf,
    pub content: String,
    pub language: String,
    pub repository_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeFileResponse {
    pub file_id: String,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub analysis_time_ms: u64,
}

/// Query the code graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQueryRequest {
    pub query_type: GraphQueryType,
    pub node_id: String,
    pub max_depth: Option<u32>,
    pub edge_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphQueryType {
    Dependencies,       // What does this symbol depend on?
    Callers,            // Who calls this function?
    Callees,            // What does this function call?
    ReverseDependencies, // Who depends on this?
    PathBetween { target_id: String }, // Find path between two symbols
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQueryResponse {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub query_time_ms: u64,
    pub cache_hit: bool,
}

/// Semantic similarity search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilaritySearchRequest {
    pub query_node_id: Option<String>, // Search similar to this node
    pub query_text: Option<String>,    // Or search by code snippet
    pub language: String,
    pub top_k: usize,
    pub min_similarity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilaritySearchResponse {
    pub results: Vec<SimilarityResult>,
    pub search_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityResult {
    pub node_id: String,
    pub similarity_score: f32,
    pub node: GraphNode,
}

/// Conflict detection (multi-tier progressive)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictDetectionRequest {
    pub old_content: String,
    pub new_content: String,
    pub file_path: PathBuf,
    pub language: String,
    pub repository_id: String,
    pub tiers: Vec<DetectionTier>, // Which tiers to run
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DetectionTier {
    Tier1AST,           // Fast AST diff (<100ms)
    Tier2Semantic,      // Semantic analysis (<1s)
    Tier3GraphImpact,   // Graph impact (<5s)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictDetectionResponse {
    pub conflicts: Vec<Conflict>,
    pub total_time_ms: u64,
    pub tier_timings: HashMap<String, u64>, // Tier name -> time in ms
}

/// Session status query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatus {
    pub session_id: String,
    pub status: SessionState,
    pub files_analyzed: u32,
    pub nodes_created: u32,
    pub conflicts_detected: u32,
    pub cache_hit_rate: f32,
    pub elapsed_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionState {
    Running,
    Completed,
    Failed { error: String },
}

/// Real-time update subscription (WebSocket)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSubscription {
    pub subscription_id: String,
    pub websocket_url: String, // ws:// or wss:// URL for WebSocket connection
}

// ============================================================================
// Data Types (Shared Entities)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub node_type: String,
    pub name: String,
    pub qualified_name: String,
    pub file_path: PathBuf,
    pub line: u32,
    pub column: u32,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source_id: String,
    pub target_id: String,
    pub edge_type: String,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub id: String,
    pub conflict_type: String,
    pub severity: Severity,
    pub confidence: f32,
    pub tier: DetectionTier,
    pub affected_symbols: Vec<String>,
    pub description: String,
    pub suggested_resolution: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

// ============================================================================
// WebSocket Message Types (Real-Time Updates)
// ============================================================================

/// Messages sent over WebSocket for real-time updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebSocketMessage {
    /// Code change detected, incremental update triggered
    CodeChangeDetected {
        repository_id: String,
        changed_files: Vec<PathBuf>,
        timestamp: i64,
    },

    /// Conflict prediction update (progressive tiers)
    ConflictUpdate {
        conflict_id: String,
        tier: DetectionTier,
        conflicts: Vec<Conflict>,
        timestamp: i64,
    },

    /// Analysis session progress update
    SessionProgress {
        session_id: String,
        files_processed: u32,
        total_files: u32,
        timestamp: i64,
    },

    /// Graph update notification (nodes/edges added/removed)
    GraphUpdate {
        repository_id: String,
        added_nodes: Vec<String>,
        removed_nodes: Vec<String>,
        added_edges: Vec<String>,
        removed_edges: Vec<String>,
        timestamp: i64,
    },

    /// Heartbeat (keep-alive)
    Ping { timestamp: i64 },

    /// Heartbeat response
    Pong { timestamp: i64 },

    /// Error notification
    Error { code: String, message: String },
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: ErrorCode,
    pub message: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ErrorCode {
    InvalidRequest,
    ParseError,
    AnalysisError,
    StorageError,
    NotFound,
    Timeout,
    InternalError,
}

impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for RpcError {}

// ============================================================================
// Serialization Helpers
// ============================================================================

/// Serialize RPC request/response to binary (postcard)
pub fn serialize_binary<T: Serialize>(value: &T) -> Result<Vec<u8>, postcard::Error> {
    postcard::to_allocvec(value)
}

/// Deserialize RPC request/response from binary (postcard)
pub fn deserialize_binary<'a, T: Deserialize<'a>>(bytes: &'a [u8]) -> Result<T, postcard::Error> {
    postcard::from_bytes(bytes)
}

/// Serialize to JSON (for debugging, not production)
pub fn serialize_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}

/// Deserialize from JSON (for debugging, not production)
pub fn deserialize_json<'a, T: Deserialize<'a>>(json: &'a str) -> Result<T, serde_json::Error> {
    serde_json::from_str(json)
}
