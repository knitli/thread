<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: AGPL-3.0-or-later
-->

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
- Missed-update replay on reconnect (`RequestMissedUpdates`)

**Message Types**: CodeChangeDetected (S→C), ConflictUpdate (S→C), SessionProgress (S→C), GraphUpdate (S→C), Ping/Pong (keepalive), Error (S→C), RequestMissedUpdates (C→S)

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
  "status": "Complete",
  "is_final": false,
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
    status: ConflictUpdateStatus::Complete,
    is_final: false, // Tier 2 and 3 still pending
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
    status: ConflictUpdateStatus::Complete,
    is_final: false, // Tier 3 still pending
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
    status: ConflictUpdateStatus::Complete,
    is_final: true, // Final tier — no further updates for this conflict_id
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

#### Tier Failure / Timeout

If a tier fails to complete (analysis engine timeout, circuit breaker OPEN, engine crash), the server sends a `ConflictUpdate` with `status: Timeout`. Only `Timeout` paired with `is_final: true` is terminal (no further updates for this `conflict_id`). `Timeout` paired with `is_final: false` means a retry is queued and a follow-up `Complete` message will arrive:

```rust
// Case 1: Timeout with no retry queued (terminal — is_final: true)
WebSocketMessage::ConflictUpdate {
    conflict_id: "conflict:abc123".to_string(),
    tier: DetectionTier::Tier2Semantic,
    status: ConflictUpdateStatus::Timeout,
    is_final: true,   // No retry queued; this is the definitive result for this conflict_id
    conflicts: vec![/* last known state from Tier 1 */],
    timestamp: 1704988802,
}

// Case 2: Timeout with retry queued (is_final: false — expect a follow-up Complete message)
WebSocketMessage::ConflictUpdate {
    conflict_id: "conflict:abc123".to_string(),
    tier: DetectionTier::Tier2Semantic,
    status: ConflictUpdateStatus::Timeout,
    is_final: false,  // Retry queued; a subsequent Complete message will arrive for this (conflict_id, tier)
    conflicts: vec![/* last known state from Tier 1 */],
    timestamp: 1704988802,
}
// ... later, when the retry completes:
WebSocketMessage::ConflictUpdate {
    conflict_id: "conflict:abc123".to_string(),
    tier: DetectionTier::Tier2Semantic,
    status: ConflictUpdateStatus::Complete,
    is_final: false,  // Tier 3 may still follow
    conflicts: vec![/* full Tier 2 result */],
    timestamp: 1704988815,
}
```

`is_final: true` signals the client that no further `ConflictUpdate` messages will follow for this `conflict_id`. The client should display the last known tier result as the definitive analysis. `is_final: false` with `status: Timeout` means a retry is queued; the client should keep the current result and apply any subsequent update for the same `(conflict_id, tier)` when it arrives (last-received-wins).

**Tier failure does NOT generate a generic `Error` message.** Error messages are reserved for connection-level or session-level failures, not analysis tier failures.

#### ConflictUpdate Status and Finality Fields

The `ConflictUpdate` message includes two new fields:

```rust
pub enum ConflictUpdateStatus {
    Complete, // Tier analysis completed successfully
    Timeout,  // Tier timed out. `is_final: true` if no retry is queued; `is_final: false` if retry is pending.
}
```

- `status: ConflictUpdateStatus` — indicates outcome of the tier analysis
- `is_final: bool` — when `true`, no further `ConflictUpdate` messages will be sent for this `conflict_id`

**Deferred-completion (retry) pattern**: When a tier times out, the server sends `status: Timeout`. If a retry is queued, `is_final: false` is set — the client should keep the last known result and expect a follow-up. When the retry completes, a new `ConflictUpdate` arrives for the same `conflict_id` and `tier` with `status: Complete`. Clients always apply the most recently received message for a given `(conflict_id, tier)` pair — newer messages implicitly supersede older ones for the same pair. There is no explicit `Superseded` status: last-received-wins is sufficient for conflict update merging.

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

**Timeout**: If no Pong received within 90 seconds (3 × ping interval, configurable), server closes connection. The default of 3 × ping interval provides resilience against single dropped packets and high-latency edge clients while remaining responsive to genuine disconnections.

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

### 7. Request Missed Updates (Client → Server)

**Direction**: Client → Server
**Trigger**: Client reconnects after disconnection and requests replay of messages missed during the outage
**Use Case**: Ensures no conflict updates, graph changes, or session progress events are silently lost during network interruption

```rust
WebSocketMessage::RequestMissedUpdates {
    since_timestamp: 1704988750, // Unix timestamp of last received message
}
```

**Server Response**: Server replays all messages with `timestamp > since_timestamp` from the replay buffer, in chronological order, followed by a synthetic `SessionProgress` message indicating replay is complete.

**Replay Buffer Limits**:
- Retention period: 5 minutes of messages retained per repository connection
- Maximum replay batch: 500 messages per reconnect request
- If `since_timestamp` is older than the retention window, server responds with `Error { code: "REPLAY_EXPIRED", message: "Reconnect gap exceeds 5-minute replay window; full re-sync required" }`

**Deployment behavior**:
- **Commercial edge** (Durable Objects): Replay buffer maintained in DO storage. Full replay semantics as specified above.
- **OSS edge**: No replay buffer. Clients that reconnect receive `Error { code: "REPLAY_NOT_SUPPORTED", message: "Replay requires commercial deployment" }`.
- **OSS CLI**: No replay buffer currently. `RequestMissedUpdates` returns `Error { code: "REPLAY_NOT_SUPPORTED", message: "CLI replay buffer is a backlog item" }`. Clients should treat reconnect as a fresh connection.

OSS CLI replay buffer is tracked as a backlog item. When implemented, it will maintain an in-memory buffer with configurable retention (default: 5 minutes).

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
  |--- RequestMissedUpdates -> | (since last_timestamp; see Message Type 7)
  |<-- [replayed messages] --- | (all messages since last_timestamp, chronological)
  |<-- SessionProgress ------- | (synthetic replay-complete marker)
  |                            |
```

**Reconnect Backoff**: 1s, 2s, 4s, 8s, 16s, 30s (max)

**Replay Protocol**: After reconnecting, clients SHOULD send `RequestMissedUpdates` with the Unix timestamp of the last message they received. On commercial edge (Durable Objects), the server replays all buffered messages newer than that timestamp; if the gap exceeds the 5-minute replay window, it returns `Error { code: "REPLAY_EXPIRED" }` and the client must perform a full re-sync. On OSS deployments (both CLI and edge), `RequestMissedUpdates` returns `Error { code: "REPLAY_NOT_SUPPORTED" }` and clients should treat the reconnect as a fresh connection. See Message Type 7 for full replay buffer limits and deployment-specific behavior.

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
    connections: thread_utilities::RapidMap<String, WebSocket>,
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

1. **Authentication**: WebSocket connections require a valid API token in the `Authorization` header. **Exception**: CLI local-mode deployment (single-user, localhost-bound) does not require authentication per SC-AUTH-001. All authentication requirements here apply to service-mode and all edge deployments.
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
- **Heartbeat Timeout**: 90 seconds default (3 × 30s ping interval); configurable via server configuration
- **Message Propagation**: <50ms (WebSocket), <100ms (SSE), 100-500ms (Polling)
- **Heartbeat Overhead**: <100 bytes/minute per connection
- **Binary vs JSON Size**: ~60% reduction (postcard vs JSON)
