---
id: TASK-PROJ-003
title: "Yjs CRDT for issue description + comment-body fields; LWW for scalar metadata; reconnection state recovery; conflict-free multi-cursor editing"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-16T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: PROJ
priority: p0
status: done
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_tasks: [TASK-PROJ-001, TASK-PROJ-002, TASK-PROJ-017, TASK-AUTH-003, TASK-AUTH-004]
depends_on: [TASK-PROJ-002]
blocks: [TASK-PROJ-017, TASK-PROJ-011]

source_pages:
  - website/docs/modules/proj.html#collaborative-editing
  - website/docs/runbooks/proj-yjs-runbook.html
source_decisions:
  - DEC-240 (Yjs is the CRDT library; battle-tested; ecosystem-proven; WebSocket Y-Provider canonical transport)
  - DEC-241 (CRDT for rich-text fields only; scalars use LWW with explicit conflict resolution)
  - DEC-242 (Y.Doc state snapshot every 60s to Postgres for reconnection recovery)
  - DEC-243 (server-side merge is non-authoritative; clients converge via CRDT; server is replication + persistence)

language: typescript 5.4 + rust 1.81
service: cyberos/services/proj-sync/  (rust) + cyberos/web/proj-client/ (ts)
new_files:
  - services/proj-sync/src/yjs_relay.rs
  - services/proj-sync/src/snapshot.rs
  - services/proj-sync/src/state_persistence.rs
  - services/proj-sync/migrations/0003_yjs_state_snapshots.sql
  - services/proj-sync/tests/yjs_relay_test.rs
  - services/proj/tests/audit_row_test.rs
  - web/proj-client/src/collab/YjsProvider.ts
  - web/proj-client/src/collab/CursorTracker.ts
  - web/proj-client/src/collab/ConnectionStatus.ts
  - web/proj-client/src/collab/ScalarLWW.ts
  - web/proj-client/tests/yjs_provider_test.ts
modified_files:
  # add /yjs/<doc_id> WebSocket route
  - services/proj-sync/src/server.rs
  # yrs (Rust Yjs port), y-sync@0.4
  - services/proj-sync/Cargo.toml
  # yjs@13, y-websocket@2, y-protocols@1
  - web/proj-client/package.json
  # wire Y.Text to TipTap editor
  - web/proj-client/src/components/IssueEditor.tsx
allowed_tools:
  - file_read: services/proj-sync/**, web/proj-client/**
  - file_write: services/proj-sync/{src,tests,migrations}/**, web/proj-client/src/**, web/proj-client/tests/**
  - bash: cd services/proj-sync && cargo test
  - bash: cd web/proj-client && npm test
disallowed_tools:
  - merge CRDT updates server-side (per DEC-243 — server is relay, not authority)
  - use Y.Map for scalar fields (per DEC-241 — overkill; LWW with version vector suffices)
  - skip snapshot persistence (per DEC-242 — clients reconnecting must converge to last server-known state)

effort_hours: 10
subtasks:
  - "0.5h: 0003_yjs_state_snapshots.sql migration (table per doc; (doc_id, version, snapshot_bytes, created_at))"
  - "1.0h: yjs_relay.rs — Rust yrs (y-rust) WebSocket relay; forwards CRDT update messages between connected clients"
  - "1.0h: snapshot.rs — periodic (every 60s) snapshot of Y.Doc state to Postgres; bound size to 1 MB per doc"
  - "0.5h: state_persistence.rs — load latest snapshot on doc open; replay any binlog updates since snapshot timestamp"
  - "0.5h: server.rs — WebSocket route `/yjs/:doc_id` with auth (JWT via TASK-AUTH-004); RLS check via TASK-AUTH-003"
  - "1.0h: web/YjsProvider.ts — wraps y-websocket-provider; reconnection + JWT refresh + offline-edit buffer"
  - "1.0h: web/CursorTracker.ts — per-user awareness state (cursor position, selection, presence indicator)"
  - "0.5h: web/ConnectionStatus.ts — exposes online/offline/syncing state for UI indicator"
  - "1.0h: web/ScalarLWW.ts — last-writer-wins for issue.title, status, priority, assignee_id (NOT in Y.Doc; separate Postgres column with version vector)"
  - "0.5h: IssueEditor.tsx — Y.Text bound to TipTap rich-text editor; cursor decoration"
  - "1.0h: yjs_relay_test.rs — two clients edit concurrently; relay forwards; both converge"
  - "0.5h: snapshot_test.rs — snapshot every 60s; reconnect loads latest"
  - "1.0h: yjs_provider_test.ts — offline edit then reconnect; reconciliation correct"
risk_if_skipped: "Without CRDT, concurrent editing of issue.description (the bread-and-butter PROJ field) produces lost updates: Alice and Bob both edit, Bob saves last, Alice's changes vanish. With CRDT, every edit converges deterministically — no lost work. Without LWW on scalars, two users assigning the same issue to different people produces flapping state — UI never settles. Without snapshot persistence, a 4-hour offline editor reconnects and replays an unbounded binlog (Postgres queries 50k updates) → 10s page load. Without server-side relay, every client maintains an N×(N-1) full mesh — doesn't scale beyond 4 users per issue."
---

## §1 — Description (BCP-14 normative)

The collaborative-editing layer **MUST** use Yjs CRDT for rich-text fields and LWW for scalar metadata. The contract:

1. **MUST** use Y.Doc to represent each issue. Specifically:
    - `Y.Text("description")` — issue description.
    - `Y.Array("comments")` of `Y.Map { id: string, author_id: uuid, body: Y.Text, created_at: number }`.
    - Scalars (title, status, priority, assignee_id, cycle_id, estimate, labels) are NOT in Y.Doc; they're plain Postgres columns with LWW.
2. **MUST** persist Y.Doc state as snapshots in Postgres `yjs_state_snapshots` (one row per snapshot version per issue):
    - Snapshot every 60s OR on graceful disconnect of last connected client.
    - Snapshot bytes = `Y.encodeStateAsUpdate(doc)`; compressed with `zstd` (level 6); typical < 10 KB after 1000 edits.
    - Retain last 50 snapshots OR last 7 days (whichever is fewer per doc).
3. **MUST** restore Y.Doc on doc open:
    - Read latest snapshot from `yjs_state_snapshots` → `Y.applyUpdate(doc, snapshot)`.
    - Read all binlog deltas since snapshot's `created_at` from `yjs_update_log` → apply.
    - Send initial sync to connecting client.
4. **MUST** be a transparent relay: the Rust server's role is forwarding `update` messages between connected clients of the same `doc_id`. It does NOT merge, does NOT authoritatively interpret CRDT operations. CRDT correctness is client-side per Yjs semantics.
5. **MUST** authenticate WebSocket connections via JWT (TASK-AUTH-004); enforce RLS on doc_id (TASK-AUTH-003 — only tenant members can subscribe). Invalid auth → close with code 4001; tenant mismatch → close with 4003.
6. **MUST** apply scalar LWW per field:
    - Each scalar field has companion columns `<field>_updated_at_ns` (i64, unix ns) + `<field>_updated_by_subject_id` (uuid).
    - On write: if `incoming_updated_at_ns > stored_updated_at_ns`, accept; else reject with `409 STALE_WRITE` and return current state for client to reconcile.
    - Tie-break on equal timestamps: lexicographic on `updated_by_subject_id` (deterministic).
7. **MUST** track awareness state (per-user cursor + selection + presence):
    - Sent via Yjs Awareness Protocol on a separate channel within the same WebSocket.
    - Awareness state expires 30s after last heartbeat (user disconnected without close frame).
    - Sent at most 30 Hz (33ms throttle) per user per doc.
8. **MUST** buffer offline edits client-side:
    - When WebSocket disconnected, accumulate updates in IndexedDB-backed Y.Doc.
    - On reconnect: send buffered updates first; server forwards; clients converge.
    - Buffer size capped at 5 MB per doc; overflow → emit `proj.yjs_buffer_overflow` audit row + UI banner ("editing offline; some changes may be lost on reconnect").
9. **MUST** emit memory audit rows for significant CRDT events:
    - `proj.issue_collab_session_started` on first client connection (when prior was 0).
    - `proj.issue_collab_session_ended` on last disconnect.
    - `proj.issue_snapshot_persisted` per snapshot, with `{doc_id, version, bytes_compressed, applies_since_last_snapshot}`.
    - `proj.yjs_buffer_overflow` on client-side buffer overflow (sent on reconnect).
    - `proj.scalar_stale_write_rejected` on LWW reject with `{field, incoming_ts, stored_ts, attempted_by}`.
10. **MUST** propagate W3C TraceContext: WebSocket handshake carries `traceparent` header; relay includes trace_id in every forwarded message metadata for OBS correlation.
11. **MUST** emit OTel metrics:
    - `proj_yjs_active_connections{doc_id_bucket}` (gauge; doc_id bucketed to prevent cardinality blow-up).
    - `proj_yjs_messages_forwarded_total{kind}` (counter; kind ∈ update | awareness | sync-step1 | sync-step2).
    - `proj_yjs_snapshot_duration_seconds` (histogram).
    - `proj_yjs_snapshot_bytes` (histogram; alert if p99 > 1 MB).
    - `proj_yjs_lwww_conflicts_total{field}` (counter — operator visibility into hot conflict fields).
12. **MUST** support graceful degradation when Postgres unavailable:
    - Snapshot writes buffered in memory (cap 100 MB across all docs); replayed on Postgres recovery.
    - Reads of `yjs_state_snapshots` failure → start fresh Y.Doc (no history); UI shows "history unavailable" banner.
13. **MUST** be the only path for issue.description writes. Direct Postgres updates of `issues.description` are forbidden (enforced by absence of `description` column on the canonical issue table — it's a write-through view of latest Y.Doc snapshot).

---

## §2 — Why this design (rationale for humans)

**Why Yjs over alternatives (§1 #1, DEC-240)?** Three reasons. First, MATURITY: Yjs has been production-deployed at scale (Notion, Linear, Figma's earlier versions). Second, ECOSYSTEM: TipTap (our chosen editor) has first-class Yjs binding. Third, PERFORMANCE: Yjs's compact binary format and incremental updates outperform Automerge / Loro for our scale (issue descriptions ≤ 50 KB, ≤ 10 concurrent editors).

**Why Y.Text only for body fields, NOT scalars (§1 #1, DEC-241)?** CRDT is expensive for scalar fields — a Y.Map for `status: "in_progress"` carries 50+ bytes of metadata for 12 bytes of value. LWW with version vectors is simpler (one i64 + one uuid per field) and correctness is acceptable: "the last person to click `In Progress` wins" matches user mental model.

**Why server is relay-only (§1 #4, DEC-243)?** Yjs's correctness is CLIENT-defined: clients converge regardless of message order. A server that tries to merge introduces a second authoritative interpretation that may diverge. Relay-only server is simpler (forward bytes) and matches Yjs's design intent. Trade-off: server can't enforce content validation; we accept this since rich-text fields are user content anyway.

**Why snapshot every 60s (§1 #2, DEC-242)?** Linear scaling: snapshot every-edit = Postgres write storm; snapshot once-per-day = 24h replay on reconnect (slow). 60s is the calibrated balance: ~3 MB/day write volume per doc, < 1s replay budget.

**Why scalar LWW with `_updated_at_ns` (§1 #6)?** Three properties: (a) DETERMINISTIC: same timestamps + same tie-breaker = same outcome on every replica; (b) STALE DETECTION: clients can refuse stale local writes against server's view; (c) AUDITABLE: companion column `_updated_by_subject_id` is the "who" — clear in memory audit.

**Why 30 Hz awareness throttle (§1 #7)?** Mouse-cursor at 60 Hz overwhelms WebSocket → bursty traffic. 30 Hz = ~33ms granularity; human eye perceives 30 Hz as smooth. Heartbeat at 30 Hz keeps presence "alive" without flooding.

**Why offline buffer cap 5 MB (§1 #8)?** A user editing offline for an hour accumulates ~500 KB of CRDT operations on a heavy document. 5 MB covers ~10 hours of offline work — generous. Beyond that, the buffer overflow signals "you really should reconnect"; we degrade gracefully (newer edits drop) and emit an audit row.

**Why audit start/end + snapshots (§1 #9)?** Operators investigating "who edited issue X last week" need a session trail. Snapshot rows give time-series of edit volume per issue (forensics + product analytics).

**Why W3C trace_id in WebSocket (§1 #10)?** Operators tracing "user clicked Save → server processed → memory row appeared" need correlation. WebSocket sessions are long-lived; without trace propagation, every CRDT operation is orphaned in OBS.

**Why Postgres-down degrades to memory buffer (§1 #12)?** Editing must not block on database availability. 100 MB in-memory cap covers ~5 minutes of typical edit volume across all active docs; longer outages emit sev-1.

**Why direct description writes forbidden (§1 #13)?** Two paths = drift risk. A naive REST endpoint that writes `issues.description = '...'` would bypass CRDT, lose concurrent edits, and corrupt the Y.Doc state. The canonical issue table has no `description` column; description is materialised from latest snapshot via view.

---

## §3 — API contract

### Schema

```sql
-- services/proj-sync/migrations/0003_yjs_state_snapshots.sql

CREATE TABLE yjs_state_snapshots (
    doc_id          TEXT NOT NULL,
    version         BIGINT NOT NULL,
    snapshot_bytes  BYTEA NOT NULL,
    applies_count   INT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    tenant_id       UUID NOT NULL,
    PRIMARY KEY (doc_id, version)
);
CREATE INDEX idx_yjs_state_snapshots_recent ON yjs_state_snapshots (doc_id, version DESC);

CREATE TABLE yjs_update_log (
    doc_id          TEXT NOT NULL,
    seq             BIGSERIAL NOT NULL,
    update_bytes    BYTEA NOT NULL,
    applied_by_subject_id UUID NOT NULL,
    applied_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    tenant_id       UUID NOT NULL,
    PRIMARY KEY (doc_id, seq)
);
CREATE INDEX idx_yjs_update_log_replay ON yjs_update_log (doc_id, applied_at);

-- LWW companion columns on issues table
ALTER TABLE issues
    ADD COLUMN title_updated_at_ns BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN title_updated_by_subject_id UUID,
    ADD COLUMN status_updated_at_ns BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN status_updated_by_subject_id UUID,
    ADD COLUMN assignee_updated_at_ns BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN assignee_updated_by_subject_id UUID;

-- RLS policy
CREATE POLICY yjs_state_tenant_isolation ON yjs_state_snapshots
    USING (tenant_id = current_setting('app.tenant_id')::uuid);
```

### Rust relay

```rust
// services/proj-sync/src/yjs_relay.rs
use yrs::sync::{Awareness, AwarenessUpdate};
use yrs::Doc;
use tokio::sync::broadcast;
use std::sync::Arc;

pub struct DocRoom {
    pub doc_id:    String,
    pub doc:       Arc<tokio::sync::RwLock<Doc>>,
    pub awareness: Arc<tokio::sync::RwLock<Awareness>>,
    /// broadcast channel: forwards every received message to all connected clients
    pub broadcast: broadcast::Sender<Message>,
}

#[derive(Clone, Debug)]
pub enum Message {
    SyncStep1(Vec<u8>),                  // client requests state vector
    SyncStep2(Vec<u8>),                  // server / peer sends missing updates
    Update(Vec<u8>),                     // CRDT delta
    Awareness(AwarenessUpdate),
}

pub async fn handle_ws_connection(
    ws: axum::extract::ws::WebSocket,
    doc_id: String,
    subject_id: uuid::Uuid,
    tenant_id: uuid::Uuid,
    room: Arc<DocRoom>,
) {
    let (mut sender, mut receiver) = ws.split();
    let mut rx = room.broadcast.subscribe();

    // 1. Send initial state to new client
    let doc = room.doc.read().await;
    let state_vec = yrs::encode_state_as_update_v1(&doc, &yrs::StateVector::default());
    drop(doc);
    let _ = sender.send(axum::extract::ws::Message::Binary(state_vec.into())).await;

    // 2. Forward incoming + broadcast outgoing
    loop {
        tokio::select! {
            // Incoming from this client
            Some(Ok(msg)) = receiver.next() => {
                if let axum::extract::ws::Message::Binary(bytes) = msg {
                    let parsed = parse_y_protocol(&bytes);
                    match parsed {
                        Some(Message::Update(update_bytes)) => {
                            // Apply to room's Doc (so reconnections see latest)
                            let mut doc = room.doc.write().await;
                            let _ = yrs::apply_update_v1(&doc, &update_bytes);
                            drop(doc);
                            // Persist to update_log (async background task)
                            let _ = state_persistence::log_update(&room.doc_id, subject_id, tenant_id, &update_bytes).await;
                            // Broadcast to all other clients
                            let _ = room.broadcast.send(Message::Update(update_bytes));
                        }
                        Some(Message::Awareness(aw)) => {
                            let mut awareness = room.awareness.write().await;
                            awareness.apply_update(aw.clone());
                            drop(awareness);
                            let _ = room.broadcast.send(Message::Awareness(aw));
                        }
                        Some(Message::SyncStep1(state_vec)) => {
                            // client asking for missing updates
                            let doc = room.doc.read().await;
                            let missing = yrs::encode_state_as_update_v1(&doc, &state_vec);
                            drop(doc);
                            let _ = sender.send(axum::extract::ws::Message::Binary(missing.into())).await;
                        }
                        _ => {}
                    }
                }
            }
            // Outgoing from broadcast
            Ok(msg) = rx.recv() => {
                let bytes = serialise_message(&msg);
                let _ = sender.send(axum::extract::ws::Message::Binary(bytes.into())).await;
            }
        }
    }
}
```

### Snapshot scheduler

```rust
// services/proj-sync/src/snapshot.rs
use std::time::Duration;
use tokio::time::interval;

pub async fn snapshot_loop(rooms: Arc<RoomRegistry>) {
    let mut ticker = interval(Duration::from_secs(60));
    loop {
        ticker.tick().await;
        for room in rooms.iter().await {
            let start = std::time::Instant::now();
            let doc = room.doc.read().await;
            let snapshot = yrs::encode_state_as_update_v1(&doc, &yrs::StateVector::default());
            drop(doc);
            let compressed = zstd::encode_all(snapshot.as_slice(), 6).expect("zstd");
            let applies_count = room.applies_since_snapshot.swap(0, std::sync::atomic::Ordering::Relaxed);

            let res = sqlx::query!(
                "INSERT INTO yjs_state_snapshots (doc_id, version, snapshot_bytes, applies_count, tenant_id)
                 VALUES ($1, $2, $3, $4, $5)",
                room.doc_id, room.next_version().await, compressed, applies_count, room.tenant_id,
            ).execute(&pool).await;

            metrics::histogram!("proj_yjs_snapshot_duration_seconds").record(start.elapsed().as_secs_f64());
            metrics::histogram!("proj_yjs_snapshot_bytes").record(compressed.len() as f64);

            match res {
                Ok(_) => {
                    emit_memory_row("proj.issue_snapshot_persisted", serde_json::json!({
                        "doc_id": room.doc_id,
                        "version": room.current_version().await,
                        "bytes_compressed": compressed.len(),
                        "applies_since_last_snapshot": applies_count,
                    })).await;
                    prune_old_snapshots(&room.doc_id).await;
                }
                Err(e) => {
                    tracing::error!(doc_id = %room.doc_id, ?e, "snapshot persist failed; buffering");
                    buffer::push(room.doc_id.clone(), compressed).await;
                }
            }
        }
    }
}
```

### TypeScript client

```typescript
// web/proj-client/src/collab/YjsProvider.ts
import * as Y from 'yjs';
import { WebsocketProvider } from 'y-websocket';
import { IndexeddbPersistence } from 'y-indexeddb';

export interface YjsConfig {
  docId:    string;
  wsUrl:    string;          // wss://proj-sync.cyberos.world
  jwt:      string;          // TASK-AUTH-004 access token
  onStatus: (s: 'connecting'|'connected'|'syncing'|'offline') => void;
}

export class YjsProvider {
  doc:       Y.Doc;
  awareness: any;
  private wsProvider: WebsocketProvider;
  private idb:        IndexeddbPersistence;

  constructor(cfg: YjsConfig) {
    this.doc = new Y.Doc();
    this.idb = new IndexeddbPersistence(`proj-${cfg.docId}`, this.doc);   // offline buffer
    this.wsProvider = new WebsocketProvider(cfg.wsUrl, cfg.docId, this.doc, {
      params: { token: cfg.jwt },                 // server reads token from URL query
      connect: true,
      maxBackoffTime: 5000,
    });
    this.awareness = this.wsProvider.awareness;

    this.wsProvider.on('status', ({ status }: any) => {
      cfg.onStatus(status === 'connected' ? 'connected' : 'connecting');
    });
    this.wsProvider.on('sync', (synced: boolean) => {
      cfg.onStatus(synced ? 'connected' : 'syncing');
    });

    // Offline detection
    window.addEventListener('online',  () => this.wsProvider.connect());
    window.addEventListener('offline', () => cfg.onStatus('offline'));
  }

  getText(field: string): Y.Text {
    return this.doc.getText(field);
  }

  destroy() {
    this.wsProvider.destroy();
    this.idb.destroy();
    this.doc.destroy();
  }
}
```

```typescript
// web/proj-client/src/collab/ScalarLWW.ts
export interface ScalarFieldUpdate<T> {
  field:        string;
  value:        T;
  updated_at_ns: bigint;
  updated_by_subject_id: string;
}

export async function writeScalarLWW<T>(
  issueId: string,
  field:   string,
  value:   T,
  jwt:     string,
): Promise<{ accepted: true } | { accepted: false; current: T; reason: 'stale_write' }>
{
  const now = BigInt(Date.now()) * 1_000_000n;  // ns
  const subject = decodeJwtSubject(jwt);
  const resp = await fetch(`/api/proj/issues/${issueId}/${field}`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${jwt}` },
    body: JSON.stringify({ value, updated_at_ns: String(now), updated_by_subject_id: subject }),
  });
  if (resp.status === 409) {
    const body = await resp.json();
    return { accepted: false, current: body.current, reason: 'stale_write' };
  }
  return { accepted: true };
}
```

### LWW backend handler

```rust
// services/proj-sync/src/scalar_handlers.rs
pub async fn patch_scalar(
    Path((issue_id, field)): Path<(uuid::Uuid, String)>,
    State(pool): State<sqlx::PgPool>,
    Json(req): Json<ScalarPatchReq>,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    // Whitelist allowed fields
    let allowed = ["title", "status", "priority", "assignee_id", "cycle_id", "estimate"];
    if !allowed.contains(&field.as_str()) {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "unknown_field"}))));
    }
    // LWW: compare against stored timestamp
    let stored: (i64, Option<uuid::Uuid>) = sqlx::query_as(&format!(
        "SELECT {field}_updated_at_ns, {field}_updated_by_subject_id FROM issues WHERE id = $1"
    ))
    .bind(issue_id).fetch_one(&pool).await.unwrap();

    if req.updated_at_ns < stored.0
       || (req.updated_at_ns == stored.0 && req.updated_by_subject_id < stored.1.unwrap_or_default())
    {
        // Stale write
        let current: serde_json::Value = fetch_current(&pool, issue_id, &field).await;
        emit_memory_row("proj.scalar_stale_write_rejected", json!({
            "issue_id": issue_id, "field": field,
            "incoming_ts": req.updated_at_ns, "stored_ts": stored.0,
            "attempted_by": req.updated_by_subject_id,
        })).await;
        metrics::counter!("proj_yjs_lwww_conflicts_total", "field" => field.clone()).increment(1);
        return Err((StatusCode::CONFLICT, Json(json!({
            "error": "stale_write", "current": current
        }))));
    }
    // Accept
    sqlx::query(&format!(
        "UPDATE issues SET {field} = $1, {field}_updated_at_ns = $2, {field}_updated_by_subject_id = $3 WHERE id = $4"
    ))
    .bind(req.value).bind(req.updated_at_ns).bind(req.updated_by_subject_id).bind(issue_id)
    .execute(&pool).await.unwrap();
    Ok(())
}
```

---

## §4 — Acceptance criteria

1. **Two clients edit description concurrently → both converge** — Alice types "X" at offset 0; Bob types "Y" at offset 0; both clients end up with same text (e.g. "XY" or "YX" deterministically per Yjs).
2. **Comments are CRDT** — Alice adds comment, Bob adds comment simultaneously → both clients see both comments in eventually-consistent order.
3. **Scalar LWW: later timestamp wins** — Alice sets status=done at T=100; Bob sets status=in_progress at T=200 → final status=in_progress.
4. **Scalar LWW: stale write rejected** — Bob still has status=todo (from T=50); writes status=cancelled at T=50 (his clock is behind) → server returns 409 with current state.
5. **Scalar LWW: tie-break deterministic** — Alice + Bob both write at T=100; Alice subject_id "a..." < Bob subject_id "b..." → Bob wins (lexicographically greater).
6. **Snapshot every 60s** — fixture: 60 updates over 60s; assert exactly 1 snapshot row added to `yjs_state_snapshots`.
7. **Snapshot bytes compressed** — empirical: snapshot < 10 KB for issue with 1000 edits.
8. **Reconnect loads latest snapshot + binlog deltas** — disconnect; add 50 binlog updates; reconnect → client sees state == server's current Y.Doc.
9. **Old snapshots pruned** — > 50 snapshots OR > 7 days → oldest deleted; `proj_yjs_snapshots_pruned_total` increments.
10. **JWT-authenticated WebSocket** — connect without `?token=` → close code 4001.
11. **Tenant mismatch → 4003** — JWT for tenant A connects to doc owned by tenant B → close code 4003.
12. **Awareness state expires 30s** — disconnect without close frame → other clients see presence indicator vanish after 30s.
13. **Awareness throttled at 30 Hz** — fixture sends 100 cursor updates in 1s; server forwards ≤ 30.
14. **Offline edit + reconnect** — disconnect; edit description; reconnect → server applies the buffered updates; all clients converge.
15. **Buffer overflow** — > 5 MB of offline edits → client emits `proj.yjs_buffer_overflow` on reconnect; UI banner shown.
16. **memory audit: session_started** — first client connects → `proj.issue_collab_session_started` row.
17. **memory audit: session_ended** — last client disconnects → `proj.issue_collab_session_ended` row.
18. **memory audit: snapshot_persisted** — snapshot fires → row with `bytes_compressed`, `applies_since_last_snapshot`.
19. **memory audit: scalar_stale_write_rejected** — LWW reject → row with `incoming_ts`, `stored_ts`.
20. **OTel: messages_forwarded counter** — relay forwards 100 updates → counter at 100.
21. **OTel: snapshot_bytes histogram** — large doc (50 KB) → snapshot recorded; alert at > 1 MB.
22. **OTel: lww_conflicts_total per field** — frequent assignee conflicts → operator sees `field="assignee_id"` is the hottest.
23. **W3C trace propagation** — handshake carries `traceparent`; relay logs include trace_id; OBS dashboard correlates.
24. **Postgres down: in-memory buffer** — kill Postgres; clients continue editing; snapshots buffer; Postgres up → buffered snapshots flushed.
25. **Direct description writes forbidden** — REST endpoint `PATCH /issues/:id description=X` → 404 / 405 (field not exposed).

---

## §5 — Verification

```rust
// services/proj-sync/tests/yjs_relay_test.rs

#[tokio::test]
async fn two_clients_converge_concurrent_edits() {
    let env = TestEnv::new().await;
    let alice = env.connect_as("alice").await;
    let bob   = env.connect_as("bob").await;

    // Alice + Bob both edit "description" at the same offset
    alice.send_update(insert_text("description", 0, "Alice")).await;
    bob.send_update(insert_text("description", 0, "Bob")).await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    let a_text = alice.read_text("description").await;
    let b_text = bob.read_text("description").await;
    assert_eq!(a_text, b_text);    // converged
}

#[tokio::test]
async fn snapshot_every_60s() {
    let env = TestEnv::with_paused_time().await;
    let alice = env.connect_as("alice").await;
    for i in 0..60 { alice.send_update(insert_text("description", 0, &format!("{i}"))).await; }
    tokio::time::advance(Duration::from_secs(61)).await;
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM yjs_state_snapshots")
        .fetch_one(&env.pool).await.unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn reconnect_loads_latest_snapshot() {
    let env = TestEnv::new().await;
    let alice = env.connect_as("alice").await;
    alice.send_update(insert_text("description", 0, "before disconnect")).await;
    env.force_snapshot().await;
    alice.send_update(insert_text("description", 17, " ; more updates")).await;
    alice.disconnect().await;

    let alice2 = env.connect_as("alice").await;
    let text = alice2.read_text("description").await;
    assert_eq!(text, "before disconnect ; more updates");
}

#[tokio::test]
async fn jwt_required() {
    let env = TestEnv::new().await;
    let result = env.try_connect_without_jwt().await;
    assert!(matches!(result, Err(WsCloseCode(4001))));
}

#[tokio::test]
async fn tenant_mismatch_rejected() {
    let env = TestEnv::new().await;
    let bob_other_tenant = env.connect_as_other_tenant("bob").await;
    assert!(matches!(bob_other_tenant, Err(WsCloseCode(4003))));
}
```

```typescript
// web/proj-client/tests/yjs_provider_test.ts

test('offline edit then reconnect converges', async () => {
  const provider = new YjsProvider({ docId: 'iss-123', wsUrl, jwt, onStatus: noop });
  await provider.waitConnected();
  provider.getText('description').insert(0, 'before');
  await provider.waitSync();

  // Simulate offline
  provider.wsProvider.disconnect();
  provider.getText('description').insert(6, ' offline');
  expect(provider.getText('description').toString()).toBe('before offline');

  // Reconnect
  provider.wsProvider.connect();
  await provider.waitSync();
  // Server should now have the offline edit
  const serverDoc = await fetchServerState('iss-123');
  expect(Y.encodeStateAsUpdate(serverDoc)).toContain(/* delta with " offline" */);
});

test('stale scalar write returns 409', async () => {
  // Alice writes at T=100
  const a1 = await writeScalarLWW('iss-1', 'status', 'in_progress', aliceJwt);
  expect(a1.accepted).toBe(true);

  // Bob writes at T=50 (clock skew)
  const b1 = await writeScalarLWW_atTime('iss-1', 'status', 'todo', 50n, bobJwt);
  expect(b1.accepted).toBe(false);
  expect(b1.reason).toBe('stale_write');
  expect(b1.current).toBe('in_progress');
});
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton.)

---

## §7 — Dependencies

- **TASK-PROJ-001** — issue schema; we add LWW companion columns + new tables.
- **TASK-PROJ-002 (upstream)** — WebSocket sync engine; Yjs relay is a route within it.
- **TASK-PROJ-017 (downstream)** — Brief modal uses Yjs description binding.
- **TASK-AUTH-003** — RLS on `yjs_state_snapshots` + `yjs_update_log`.
- **TASK-AUTH-004** — JWT validation on WebSocket connect; tenant_id from claims.
- **TASK-MEMORY-101** — audit row emission.

---

## §8 — Example payloads

### `proj.issue_collab_session_started`

```json
{
  "kind": "proj.issue_collab_session_started",
  "payload": {
    "doc_id":            "iss-01HZK9R8M3X5C8Q4",
    "first_subject_id":  "7e57c0de-1234-5678-9abc-def012345678",
    "started_at_ns":     1747407137483000000,
    "trace_id":          "0af7651916cd43dd8448eb211c80319c"
  }
}
```

### `proj.issue_snapshot_persisted`

```json
{
  "kind": "proj.issue_snapshot_persisted",
  "payload": {
    "doc_id":                       "iss-01HZK9R8M3X5C8Q4",
    "version":                      42,
    "bytes_compressed":             8421,
    "applies_since_last_snapshot":  37,
    "snapshot_duration_ms":         12
  }
}
```

### `proj.scalar_stale_write_rejected`

```json
{
  "kind": "proj.scalar_stale_write_rejected",
  "payload": {
    "issue_id":      "iss-01HZK9R8M3X5C8Q4",
    "field":         "status",
    "incoming_ts":   1747407100000000000,
    "stored_ts":     1747407137000000000,
    "attempted_by":  "7e57c0de-..."
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- CRDT for `labels` array (currently LWW; may be desirable as Y.Array later) — slice 4+.
- CRDT for cross-issue moves (drag from one cycle to another) — slice 4+.
- Selective sync (subscribe to subset of fields to save bandwidth) — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Postgres unavailable | sqlx Err | In-memory buffer (cap 100 MB); sev-2 alarm | Postgres restored; buffer flushed |
| Buffer overflow (Postgres long-down) | size check | Oldest snapshots dropped; sev-1 alarm | Operator extends Postgres downtime tolerance OR scales |
| WebSocket connection refused | `accept` Err | Client retries with backoff | Network restored |
| JWT expired mid-session | server periodic check | Close code 4002; client refreshes + reconnects | Client transparently re-auths |
| Tenant changed mid-session (impossible by design) | RLS catches | Updates rejected at write time | None |
| Y.Doc memory leak (long-running room with no clients) | RoomRegistry GC after 10 min idle | Room dropped; reload from snapshot on next connect | None |
| Client clock far in the future | LWW accepts | Other client's writes rejected as stale | Operator reviews; clock-sync recommended |
| Client clock far in the past | LWW rejects all writes | Frequent 409s; UI banner "your clock is behind"; instruct to sync | None automatic |
| CRDT update corrupted in transit (network bit-flip) | Yjs internal checksum | Update dropped silently; client may need re-sync | Yjs handles |
| Awareness state churn (1000 cursor updates/sec) | 30 Hz throttle | Client rate-limited; messages_forwarded counter shows skipped | None |
| Snapshot bytes > 1 MB | size cap check | sev-2 alarm; investigate doc size | Operator considers splitting doc |
| Two snapshots same version (race) | unique constraint | Second insert fails; logged; original kept | None |
| Old snapshots not pruned (job crashed) | row count > 50 | sev-2 alarm; manual prune | Operator runs maintenance |
| `applies_since_last_snapshot` overflow | i32 capacity (2.1B) | Never in practice | None |
| Client-side IndexedDB quota exceeded | y-indexeddb Err | Offline buffer disabled; banner shown | User clears storage |
| Multiple tabs of same user same issue | Yjs handles | All tabs converge; presence shows multiple cursors | By design |
| Awareness state of a non-existent user | server validates JWT first | Cannot happen; auth gates | None |
| LWW field name typo in PATCH URL | whitelist check | 400 with `unknown_field` | Client fixes |
| LWW value type mismatch (e.g. string for integer estimate) | serde reject | 400 with details | Client fixes |
| Description binary update without proper Y-protocol framing | parse fails silently | Message dropped; client desyncs | Client reconnects |
| WebSocket pong missed | y-websocket auto-reconnects | < 30s gap; awareness expires + re-establishes | Built-in |
| RLS policy bug (cross-tenant leak) | property test detects | CI blocked | Author fixes RLS |

---

## §11 — Implementation notes

- `yrs` (the Rust Yjs port) is API-compatible with JavaScript Yjs; same wire format. We use `yrs::sync::Awareness` for awareness protocol.
- The `RoomRegistry` is in-memory; on server restart, rooms are rebuilt lazily as clients reconnect. Snapshots from Postgres provide continuity.
- `zstd` level 6 is the empirical sweet spot: level 3 = 30% larger, level 9 = 5% smaller (not worth 4× CPU).
- The 5 MB IndexedDB cap is enforced by y-indexeddb's `gcFilter`; we configure it explicitly.
- WebSocket close codes 4001/4002/4003 are app-defined (4xxx range is application-specific per RFC 6455 §7.4.2).
- Snapshot prune query: `DELETE FROM yjs_state_snapshots WHERE doc_id = $1 AND version < (SELECT version FROM yjs_state_snapshots WHERE doc_id = $1 ORDER BY version DESC LIMIT 1 OFFSET 49)` — keeps 50 most recent.
- The `applies_count` metric in snapshot rows is a sanity check: very low values (1-2) suggest snapshots too frequent for doc; very high (10000+) suggest snapshots too rare.
- `current_setting('app.tenant_id')::uuid` is set by the connection pool's `SET LOCAL` before each request, per TASK-AUTH-003.
- Awareness state heartbeat is sent by y-protocols automatically every ~15s; expiry is configured to 30s in the server.

---

*End of TASK-PROJ-003.*
