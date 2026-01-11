# WebSocket Protocol Specification

**Feature**: Real-Time Code Graph Intelligence  
**Protocol Version**: 1.0  
**Last Updated**: 2026-01-11

## Overview

The WebSocket protocol enables real-time bidirectional communication between clients (developers) and the Thread code intelligence service. It supports:

- Real-time code change notifications (<100ms propagation from FR-013)
- Progressive conflict detection updates (Tier 1 → Tier 2 → Tier 3)
- Live analysis session progress
- Graph update streaming

**Fallback Strategy**: WebSocket primary, Server-Sent Events (SSE) secondary, Long-Polling last resort

## Connection Establishment

### CLI Deployment (Native)

```
Client                      Server
  |                            |
  |--- HTTP GET /ws/subscribe -|
  |    Upgrade: websocket      |
  |    Sec-WebSocket-Version:13|
  |                            |
  |<-- 101 Switching Protocols-|
  |    WebSocket established   |
  |                            |
  |<==== Binary Messages =====>|
```

**Endpoint**: `ws://localhost:8080/ws/subscribe?repo_id={repository_id}`

### Edge Deployment (Cloudflare Workers)

```
Client                      Cloudflare Worker
  |                            |
  |--- HTTP GET /ws/subscribe -|
  |                            |
  |<-- WebSocketPair created --|
  |                            |
  |<==== Binary Messages =====>|
  |                            |
 [Durable Object manages connection state]
```

**Endpoint**: `wss://api.thread.dev/ws/subscribe?repo_id={repository_id}`

**Durable Object**: `AnalysisSessionDO` manages WebSocket connections per repository

## Message Format

### Binary Serialization (Production)

Messages use `postcard` binary serialization for ~60% size reduction vs JSON:

```rust
// Serialize
let msg = WebSocketMessage::ConflictUpdate { ... };
let bytes = postcard::to_allocvec(&msg)?;
ws.send_binary(bytes).await?;

// Deserialize
let msg: WebSocketMessage = postcard::from_bytes(&bytes)?;
```

### JSON Serialization (Debugging)

For development/debugging, JSON serialization is supported:

```json
{
  "type": "ConflictUpdate",
  "conflict_id": "conflict:abc123",
  "tier": "Tier1AST",
  "conflicts": [...],
  "timestamp": 1704988800
}
```

## Message Types

### 1. Code Change Detected

**Direction**: Server → Client  
**Trigger**: File change detected by indexer (file watcher or git poll)  
**Latency Target**: <100ms from code change to client notification (FR-013)

```rust
WebSocketMessage::CodeChangeDetected {
    repository_id: "repo:xyz789".to_string(),
    changed_files: vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("src/utils.rs"),
    ],
    timestamp: 1704988800, // Unix timestamp
}
```

**Client Action**: Trigger incremental analysis if desired, or wait for conflict update

---

### 2. Conflict Update (Progressive)

**Direction**: Server → Client  
**Trigger**: Conflict detection tier completes  
**Progressive Delivery**: Tier 1 (100ms) → Tier 2 (1s) → Tier 3 (5s)

```rust
// Tier 1: Fast AST diff
WebSocketMessage::ConflictUpdate {
    conflict_id: "conflict:abc123".to_string(),
    tier: DetectionTier::Tier1AST,
    conflicts: vec![
        Conflict {
            id: "conflict:abc123".to_string(),
            conflict_type: "SignatureChange".to_string(),
            severity: Severity::Medium,
            confidence: 0.6, // Low confidence from AST only
            tier: DetectionTier::Tier1AST,
            affected_symbols: vec!["processPayment".to_string()],
            description: "Function signature changed".to_string(),
            suggested_resolution: None, // Not yet analyzed
        },
    ],
    timestamp: 1704988800,
}

// Tier 2: Semantic refinement (1 second later)
WebSocketMessage::ConflictUpdate {
    conflict_id: "conflict:abc123".to_string(),
    tier: DetectionTier::Tier2Semantic,
    conflicts: vec![
        Conflict {
            id: "conflict:abc123".to_string(),
            conflict_type: "BreakingAPIChange".to_string(),
            severity: Severity::High, // Upgraded from Medium
            confidence: 0.9, // High confidence from semantic analysis
            tier: DetectionTier::Tier2Semantic,
            affected_symbols: vec!["processPayment".to_string(), "validatePayment".to_string()],
            description: "Breaking change - 15 callers affected".to_string(),
            suggested_resolution: Some("Update all call sites to use new signature".to_string()),
        },
    ],
    timestamp: 1704988801,
}

// Tier 3: Graph impact (5 seconds later)
WebSocketMessage::ConflictUpdate {
    conflict_id: "conflict:abc123".to_string(),
    tier: DetectionTier::Tier3GraphImpact,
    conflicts: vec![
        Conflict {
            id: "conflict:abc123".to_string(),
            conflict_type: "BreakingAPIChange".to_string(),
            severity: Severity::Critical, // Upgraded to Critical
            confidence: 0.95, // Very high confidence
            tier: DetectionTier::Tier3GraphImpact,
            affected_symbols: vec!["processPayment".to_string(), "validatePayment".to_string(), "checkoutFlow".to_string()],
            description: "Critical path affected - checkout flow broken".to_string(),
            suggested_resolution: Some("Refactor in 3 steps: 1) Add adapter layer, 2) Migrate callers, 3) Remove old API".to_string()),
        },
    ],
    timestamp: 1704988805,
}
```

**Client UI Update**: 
1. Show initial conflict immediately (Tier 1)
2. Refine details as Tier 2 completes (update confidence, severity)
3. Show comprehensive analysis when Tier 3 completes (final recommendation)

---

### 3. Session Progress

**Direction**: Server → Client  
**Trigger**: Analysis session makes progress  
**Frequency**: Every 10% of files processed, or every 5 seconds

```rust
WebSocketMessage::SessionProgress {
    session_id: "session:20260111120000:abc".to_string(),
    files_processed: 1000,
    total_files: 10000,
    timestamp: 1704988800,
}
```

**Client Action**: Update progress bar, show "10% complete (1000/10000 files)"

---

### 4. Graph Update

**Direction**: Server → Client  
**Trigger**: Incremental graph update completes (CocoIndex diff applied)  
**Latency Target**: <100ms from code change to graph update notification

```rust
WebSocketMessage::GraphUpdate {
    repository_id: "repo:xyz789".to_string(),
    added_nodes: vec!["node:def456".to_string()], // New function added
    removed_nodes: vec!["node:abc123".to_string()], // Old function deleted
    added_edges: vec!["edge:ghi789".to_string()],  // New call relationship
    removed_edges: vec!["edge:jkl012".to_string()], // Old relationship broken
    timestamp: 1704988800,
}
```

**Client Action**: Update local graph visualization, invalidate cached queries

---

### 5. Heartbeat (Keep-Alive)

**Direction**: Server → Client (Ping), Client → Server (Pong)  
**Frequency**: Every 30 seconds  
**Purpose**: Keep WebSocket connection alive, detect disconnections

```rust
// Server sends
WebSocketMessage::Ping { timestamp: 1704988800 }

// Client responds
WebSocketMessage::Pong { timestamp: 1704988800 }
```

**Timeout**: If no Pong received within 60 seconds, server closes connection

---

### 6. Error Notification

**Direction**: Server → Client  
**Trigger**: Error during analysis, storage, or processing

```rust
WebSocketMessage::Error {
    code: "ANALYSIS_TIMEOUT".to_string(),
    message: "File analysis exceeded 30s timeout".to_string(),
}
```

**Client Action**: Display error notification, optionally retry

---

## Connection Lifecycle

### Successful Connection

```
Client                      Server
  |                            |
  |--- HTTP Upgrade ---------> |
  |                            |
  |<-- 101 Switching --------- |
  |                            |
  |<-- Ping ------------------ | (every 30s)
  |--- Pong -----------------> |
  |                            |
  |<-- CodeChangeDetected ---- | (on code change)
  |<-- ConflictUpdate -------- | (progressive tiers)
  |                            |
```

### Disconnection and Reconnect

```
Client                      Server
  |                            |
  |<==== Connection Lost ===== | (network issue)
  |                            |
  |--- Reconnect ------------> | (exponential backoff)
  |                            |
  |<-- 101 Switching --------- |
  |                            |
  |--- RequestMissedUpdates -> | (since last_timestamp)
  |<-- ConflictUpdate -------- | (replay missed messages)
  |                            |
```

**Reconnect Backoff**: 1s, 2s, 4s, 8s, 16s, 30s (max)

---

## Cloudflare Durable Objects Integration

### AnalysisSessionDO

**Purpose**: Manage WebSocket connections per repository, coordinate real-time updates

```typescript
// Conceptual Durable Object (TypeScript/JavaScript)
export class AnalysisSessionDO {
  constructor(state, env) {
    this.state = state;
    this.env = env;
    this.connections = new Map(); // sessionId -> WebSocket
  }

  async fetch(request) {
    if (request.headers.get("Upgrade") === "websocket") {
      const pair = new WebSocketPair();
      await this.handleSession(pair[1]);
      return new Response(null, { status: 101, webSocket: pair[0] });
    }
    return new Response("Expected WebSocket", { status: 400 });
  }

  async handleSession(webSocket) {
    webSocket.accept();
    const sessionId = crypto.randomUUID();
    this.connections.set(sessionId, webSocket);

    webSocket.addEventListener("message", async (msg) => {
      // Handle client messages
    });

    webSocket.addEventListener("close", () => {
      this.connections.delete(sessionId);
    });
  }

  async broadcast(message) {
    for (const ws of this.connections.values()) {
      ws.send(message);
    }
  }
}
```

**Rust Integration** (workers-rs):

```rust
use worker::*;

#[durable_object]
pub struct AnalysisSession {
    state: State,
    env: Env,
    connections: HashMap<String, WebSocket>,
}

#[durable_object]
impl DurableObject for AnalysisSession {
    async fn fetch(&mut self, req: Request) -> Result<Response> {
        if req.headers().get("Upgrade")?.map(|v| v == "websocket").unwrap_or(false) {
            let pair = WebSocketPair::new()?;
            pair.server.accept()?;
            
            let session_id = uuid::Uuid::new_v4().to_string();
            self.handle_websocket(session_id, pair.server).await?;
            
            Response::ok("")?.websocket(pair.client)
        } else {
            Response::error("Expected WebSocket", 400)
        }
    }
}
```

---

## Fallback Protocols

### Server-Sent Events (SSE)

**Endpoint**: `GET /sse/subscribe?repo_id={repository_id}`  
**Use Case**: One-way server→client streaming, restrictive networks  
**Latency**: <100ms (same as WebSocket)

**Format**:
```
data: {"type": "ConflictUpdate", "conflict_id": "...", ...}

data: {"type": "SessionProgress", "files_processed": 1000, ...}

```

### Long-Polling

**Endpoint**: `GET /poll/updates?repo_id={repository_id}&since={timestamp}`  
**Use Case**: Last resort for networks blocking WebSocket and SSE  
**Latency**: 100-500ms (poll interval configurable)

**Response**:
```json
{
  "messages": [
    {"type": "ConflictUpdate", ...},
    {"type": "SessionProgress", ...}
  ],
  "timestamp": 1704988800
}
```

---

## Security Considerations

1. **Authentication**: WebSocket connections require valid API token in `Authorization` header
2. **Rate Limiting**: Max 1000 messages/second per connection
3. **Message Size**: Max 1MB per message
4. **Connection Limit**: Max 100 concurrent connections per repository
5. **Timeout**: Idle connections closed after 5 minutes of inactivity

---

## Testing Strategy

1. **Unit Tests**: Message serialization/deserialization (postcard + JSON)
2. **Integration Tests**: WebSocket connection lifecycle, reconnect logic
3. **Load Tests**: 1000 concurrent connections, message throughput
4. **Latency Tests**: <100ms propagation for code change notifications
5. **Fallback Tests**: SSE and Long-Polling degradation

---

## Performance Targets

- **Connection Establishment**: <50ms (edge), <10ms (CLI)
- **Message Propagation**: <50ms (WebSocket), <100ms (SSE), 100-500ms (Polling)
- **Heartbeat Overhead**: <100 bytes/minute per connection
- **Binary vs JSON Size**: ~60% reduction (postcard vs JSON)
