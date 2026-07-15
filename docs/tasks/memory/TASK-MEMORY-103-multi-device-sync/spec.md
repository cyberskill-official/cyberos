---
id: TASK-MEMORY-103
title: "memory-sync daemon — laptop A ↔ Cloud memory ↔ laptop B with sync_class gating + CRDT conflict + 10K offline buffer + device-id stamp"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-15T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: memory
priority: p0
status: done
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng (CDO)
created: 2026-05-15
shipped: 2026-05-23
memory_chain_hash: null
related_tasks: [TASK-MEMORY-101, TASK-MEMORY-104, TASK-MEMORY-105, TASK-MEMORY-106]
depends_on: [TASK-MEMORY-101]
blocks: [TASK-MEMORY-104, TASK-MEMORY-106, TASK-MEMORY-105]

source_pages:
  - website/docs/modules/memory.html#multi-device-sync
  - memory/docs/AUTOSYNC_DESIGN.md
source_decisions:
  - DEC-036 (compensation/equity rows excluded from sync at any boundary)
  - DEC-070 (Layer 1 source of truth; sync preserves chain integrity per AGENTS.md §14.2)
  - DEC-100 (sync_class gating at sync boundary; private never leaves device)
  - DEC-101 (CRDT triple `(content_hash, ts_ns, originator_device)` for conflict detection)

language: rust 1.81
service: cyberos/services/memory-sync/
new_files:
  - services/memory-sync/Cargo.toml
  - services/memory-sync/src/main.rs
  - services/memory-sync/src/sync.rs
  - services/memory-sync/src/crdt.rs
  - services/memory-sync/src/protocol/mod.rs
  - services/memory-sync/src/protocol/grpc.rs
  - services/memory-sync/src/buffer.rs
  - services/memory-sync/src/sync_class_filter.rs
  - services/memory-sync/src/compensation_guard.rs
  - services/memory-sync/proto/memory_sync.proto
  - services/memory/tests/ingest_test.rs
  - services/memory/tests/ingest_test.rs
  - services/memory/tests/ingest_test.rs
  - services/memory/tests/ingest_test.rs
  - services/memory/tests/chain_anchor_test.rs
modified_files: []
allowed_tools:
  - file_read: services/memory-sync/**
  - file_write: services/memory-sync/**
  - bash: cd services/memory-sync && cargo test
disallowed_tools:
  - sync compensation/equity data (per DEC-036 — compensation_guard rejects at sync boundary)
  - sync without sync_class permission check (per §1 #3)
  - bypass CRDT conflict detection (per §1 #4 — silent overwrite forbidden)
  - drop offline-buffered rows without sev-2 alert (per §1 #7)

effort_hours: 18
subtasks:
  - "0.5h: Cargo.toml + protocol/grpc.rs (tonic-generated from .proto)"
  - "1.0h: sync.rs main loop (push + pull + apply per AGENTS.md §14.2)"
  - "1.0h: sync_class_filter.rs — only `shareable` rows push"
  - "0.5h: compensation_guard.rs — refuse compensation/equity rows at sync boundary (defense in depth)"
  - "1.0h: crdt.rs — content_hash + ts_ns + originator_device triple"
  - "1.0h: Conflict detection (concurrent append from A + B with same target id)"
  - "0.5h: disputed_pair row creation on conflict (TASK-MEMORY-105 surfaces UI)"
  - "1.0h: buffer.rs — 10K-row offline buffer; FIFO eviction with sev-2 alert"
  - "0.5h: device_id stamping (per-device UUID in extra.originator_device_id)"
  - "0.5h: Foreign-chain dedup (don't re-import already-imported chains)"
  - "1.0h: gRPC TLS + bearer auth"
  - "0.5h: Online/offline state detection + reconnect resume"
  - "0.5h: OTel metrics emission"
  - "1.5h: Tests — happy + offline-buffer + conflict + sync_class + compensation-rejected + 5s round-trip + 10K buffer"
  - "1.0h: Tests — concurrent A/B writes + private-stays-local + cloud-down + reconnect"
  - "1.0h: Integration with Cloud memory (mocked or real)"
  - "1.0h: Tauri-app integration (TASK-MEMORY-104) hooks"
  - "0.5h: Cleanup + observability"
risk_if_skipped: "Each Member's laptop has an isolated memory. Founder switching from MBP to iMac loses context. Multi-device daily-use scenario fails. Without sync_class gating, private memories leak to Cloud. Without compensation_guard, DEC-036 is unenforced — comp data can sync. Without CRDT, concurrent writes silently overwrite (whichever wins the race). Without offline buffer, network blips lose data."
---

## §1 — Description (BCP-14 normative)

A `memory-sync` Rust daemon **MUST** run on every device, syncing the personal memory to a designated Cloud memory per `memory/docs/AUTOSYNC_DESIGN.md`:

1. **MUST** push every local chain row to Cloud memory via gRPC stream (TLS + bearer auth). Push is unidirectional; one row per stream message; ack-per-row.
2. **MUST** pull foreign-origin rows from Cloud memory; apply via local-import per AGENTS.md §14.2 — each foreign row becomes a fresh `put` on the LOCAL chain whose `extra.imported_from` identifies the source device fingerprint AND `extra.foreign_chain` records the source chain hash.
3. **MUST** respect `meta.sync_class`: only rows with `sync_class == "shareable"` (or the v1 transitional values `publishable | shared | client-visible`) push to Cloud. Rows with `private | local-only` stay on the device. The filter is applied at push-time AT the sync boundary.
4. **MUST** detect conflicts via CRDT triple `(content_hash, ts_ns, originator_device)`. Conflict = same target memory id with two different writes from different devices within overlapping wall-clock windows. On conflict, write a `disputed_pair` row with both versions; TASK-MEMORY-105 surfaces the resolution UI.
5. **MUST** complete sync round-trip within 5 seconds p95 on healthy network (push + cloud ack + pull + local apply). Above 5s, ops investigates.
6. **MUST** be device-id-stamped: every sync-emitted row carries `extra.originator_device_id` (UUID generated at first daemon start; persisted to `~/.cyberos/device_id`).
7. **MUST** maintain offline buffer of up to 10,000 rows when Cloud memory is unreachable. Buffer is FIFO; oldest rows evicted on overflow with sev-2 alert + metric `memory_sync_buffer_overflow_total`. Evicted rows are NOT lost (they're in the local Layer 1 chain) — they just don't propagate until manual reconciliation.
8. **MUST** emit OTel metrics:
    - `memory_sync_lag_seconds{device_id}` (histogram; round-trip).
    - `memory_sync_conflicts_total` (counter).
    - `memory_sync_buffered_rows{device_id}` (gauge).
    - `memory_sync_buffer_overflow_total{device_id}` (counter; sev-2 alarm).
    - `memory_sync_pushed_rows_total{device_id, sync_class}` (counter).
    - `memory_sync_compensation_rejected_total` (counter; sev-1 alarm — DEC-036 violation attempt).
9. **MUST** apply `compensation_guard` at the sync boundary: ANY row whose path matches `meta/people/*/compensation*` OR `meta/people/*/equity*` OR `meta/finance/payroll*` is REJECTED at push time AND NOT added to offline buffer (compensation rows that arrive at sync are a sev-1 — investigate the upstream code that emitted them).
10. **MUST** not re-import already-imported foreign chains (idempotency on `extra.foreign_chain` value).
11. **MUST** detect online/offline state via TCP probe to Cloud memory every 30s; transition online→offline → start buffering; offline→online → flush buffer + resume.
12. **MUST** authenticate to Cloud memory via per-device bearer token (rotated quarterly; managed via TASK-AUTH-006-style sweeper). Compromised token → revoke at Cloud memory; device cannot sync until new token issued.
13. **MUST** apply CRDT conflict resolution deterministically across devices: given `(A_content_hash, A_ts, A_device)` vs `(B_content_hash, B_ts, B_device)`, both devices independently arrive at the SAME resolution — write disputed_pair (both visible until human decides).
14. **MUST** preserve chain ordering on import: per AGENTS.md §14.2, the import block is bracketed by `session.start` and `session.end` audit row on the local chain — the imports are clearly demarcated.

---

## §2 — Why this design (rationale for humans)

**Why Cloud memory as star-topology hub (DEC-100)?** Peer-to-peer sync (every device → every device) scales as O(N²) connections; centralised hub is O(N). Cloud memory is the trust root + delivery hub. Devices online intermittently get all updates from the hub on reconnect.

**Why sync_class gating (§1 #3)?** Some memories are device-local (working notes, drafts). Others are shareable across devices (published decisions, project history). The `sync_class` field is the user's privacy primitive; respecting it at sync boundary is the implementation.

**Why compensation_guard at sync (DEC-036)?** Compensation/equity is in scope for AI assistance ON device but NEVER leaves device. The guard at sync boundary is defense-in-depth — even if a row is mistakenly tagged `shareable`, the path-based check catches compensation rows. Sev-1 metric flags the attempt.

**Why CRDT triple for conflict (DEC-101)?** `(content_hash, ts_ns, originator_device)` identifies any version uniquely. Two devices independently producing different content for the same target id at overlapping times = conflict. The triple is the conflict-detection primitive; deterministic across devices means both A and B independently reach "this is a conflict" without coordination.

**Why import-as-fresh-put per §14.2 (§1 #2)?** AGENTS.md §14.2 mandates: foreign chain doesn't merge directly into local chain (would corrupt local chain integrity). Each foreign row becomes a NEW local put with `extra.foreign_chain` referencing the source. The local chain stays internally consistent.

**Why 10K offline buffer (§1 #7)?** Sizing math: typical user appends ~100 rows/day; 10K = 100 days offline. Beyond 100 days, the user is effectively a different device — manual reconciliation makes sense. The cap protects against unbounded local growth during prolonged outages.

**Why 5s p95 round-trip (§1 #5)?** Multi-device users expect "I added a note on laptop A; let me find it on laptop B" to work near-real-time. 5s is below human attention; longer windows produce confusion ("did it sync?").

**Why per-device bearer token (§1 #12)?** Compromised device could sync false data to Cloud, corrupting other devices. Per-device token revocable at Cloud — compromised device's sync stops; legitimate devices unaffected.

**Why deterministic CRDT resolution (§1 #13)?** Both devices A and B independently see the conflict; both must agree on the resolution outcome (disputed_pair). Without deterministic resolution, A might apply one version + B applies the other → silent divergence. Determinism = both arrive at SAME state without coordination.

**Why session.start / session.end brackets (§1 #14, §14.2)?** Without brackets, imported rows interleave with normal local activity, making the import boundary invisible. Brackets give compliance audits a clear "imported from X at time Y to Y+N" demarcation.

---

## §3 — API contract

```proto
// services/memory-sync/proto/memory_sync.proto
syntax = "proto3";
package cyberos.memory_sync.v1;

service MemorySync {
    rpc Push(stream PushRequest) returns (stream PushAck);
    rpc Pull(PullRequest) returns (stream PullResponse);
    rpc Health(HealthRequest) returns (HealthResponse);
}

message PushRequest {
    string device_id = 1;
    int64 seq = 2;
    int64 ts_ns = 3;
    bytes body = 4;
    bytes meta_json = 5;
    bytes prev_chain = 6;
    bytes chain = 7;
    string sync_class = 8;       // shareable | private | ...
}

message PushAck { int64 seq = 1; bool accepted = 2; string reason = 3; }

message PullRequest { string device_id = 1; bytes since_chain = 2; }
message PullResponse {
    string origin_device_id = 1;
    int64 origin_seq = 2;
    bytes body = 3;
    bytes meta_json = 4;
    bytes foreign_chain = 5;
}
```

```rust
// services/memory-sync/src/sync.rs
pub async fn sync_loop(local: &LocalMemory, cloud: &CloudMemoryClient, config: &SyncConfig) -> anyhow::Result<()> {
    loop {
        let online = cloud.health_check().await.is_ok();

        if online {
            // §1 #11 — flush buffer if just transitioned to online
            if let Some(buffered) = buffer::take_all().await? {
                push_batch(buffered, cloud).await?;
            }

            // Push: shareable rows only
            let pending = local.pending_for_sync().await?;
            for row in pending {
                if !sync_class_filter::is_shareable(&row) { continue; }
                if compensation_guard::is_compensation(&row) {
                    metrics::compensation_rejected();
                    tracing::error!(seq = row.seq, path = %row.path, "compensation row at sync boundary; sev-1");
                    continue;
                }
                push_one(row, cloud).await?;
            }

            // Pull: foreign rows
            let cursor = local.last_pulled_chain().await?;
            for remote in cloud.pull_since(cursor).await? {
                if local.has_foreign_chain(&remote.foreign_chain).await? {
                    continue;   // §1 #10 dedup
                }
                let new_local_seq = local.put_with_origin(remote.clone()).await?;
                if let Some(conflict) = crdt::detect_conflict(&new_local_seq, local).await? {
                    local.write_disputed_pair(&conflict).await?;
                    metrics::conflict();
                }
            }
        } else {
            // §1 #7 buffer
            let pending = local.pending_for_sync().await?;
            for row in pending {
                if !sync_class_filter::is_shareable(&row) { continue; }
                if compensation_guard::is_compensation(&row) { /* sev-1 */; continue; }
                buffer::push(row).await?;
            }
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
```

```rust
// services/memory-sync/src/crdt.rs
pub struct CrdtTriple {
    pub content_hash: [u8; 32],
    pub ts_ns: i64,
    pub originator_device: Uuid,
}

pub async fn detect_conflict(new_row: &Row, local: &LocalMemory) -> Result<Option<DisputedPair>, anyhow::Error> {
    let target_id = new_row.target_memory_id();
    let recent_writes = local.writes_for_target_in_window(
        target_id, new_row.ts_ns - 5_000_000_000, new_row.ts_ns + 5_000_000_000,
    ).await?;

    for other in recent_writes {
        if other.originator_device != new_row.originator_device
           && other.content_hash != new_row.content_hash
        {
            return Ok(Some(DisputedPair {
                target: target_id,
                version_a: row_to_triple(new_row),
                version_b: row_to_triple(&other),
            }));
        }
    }
    Ok(None)
}

pub fn row_to_triple(row: &Row) -> CrdtTriple {
    CrdtTriple {
        content_hash: sha256(&row.body),
        ts_ns: row.ts_ns,
        originator_device: row.originator_device.unwrap_or(Uuid::nil()),
    }
}
```

```rust
// services/memory-sync/src/sync_class_filter.rs
const SHAREABLE_VALUES: &[&str] = &["shareable", "publishable", "shared", "client-visible"];

pub fn is_shareable(row: &Row) -> bool {
    let class = row.meta.get("sync_class").and_then(|v| v.as_str()).unwrap_or("private");
    SHAREABLE_VALUES.contains(&class)
}
```

```rust
// services/memory-sync/src/compensation_guard.rs
const FORBIDDEN_PATTERNS: &[&str] = &[
    "meta/people/*/compensation*",
    "meta/people/*/equity*",
    "meta/finance/payroll*",
    "meta/finance/comp*",
];

pub fn is_compensation(row: &Row) -> bool {
    let path = &row.path;
    FORBIDDEN_PATTERNS.iter().any(|p| glob_match(p, path))
}
```

```rust
// services/memory-sync/src/buffer.rs
const MAX_BUFFER_ROWS: usize = 10_000;

pub async fn push(row: Row) -> anyhow::Result<()> {
    let mut buffer = BUFFER.lock().await;
    if buffer.len() >= MAX_BUFFER_ROWS {
        let evicted = buffer.pop_front().unwrap();
        tracing::warn!(seq = evicted.seq, "buffer overflow; oldest evicted");
        metrics::buffer_overflow();
    }
    buffer.push_back(row);
    metrics::buffer_size(buffer.len());
    Ok(())
}

pub async fn take_all() -> anyhow::Result<Option<Vec<Row>>> {
    let mut buffer = BUFFER.lock().await;
    if buffer.is_empty() { return Ok(None); }
    Ok(Some(std::mem::take(&mut *buffer).into_iter().collect()))
}
```

---

## §4 — Acceptance criteria

1. Append row on device A → within 5s, row visible on device B (via Cloud memory round-trip).
2. Offline: device A appends 50 rows → reconnect → all 50 sync.
3. Concurrent append (A + B same target id, overlapping ts_ns) → CRDT detects conflict; both stored as `disputed_pair`.
4. Private rows (`sync_class: private`) never leave device.
5. Compensation rows refused at sync boundary; sev-1 metric `memory_sync_compensation_rejected_total` increments.
6. Sync round-trip < 5s p95 (push + pull + apply).
7. 10K offline buffer handled without crash; 10001st entry → oldest evicted + sev-2 alert.
8. Foreign chain dedup — re-importing same row is no-op (no duplicate local row).
9. Online/offline detection: TCP probe every 30s; state transitions trigger buffer flush.
10. Per-device bearer auth — wrong token → Cloud rejects with 401.
11. Deterministic CRDT — A and B independently arrive at same disputed_pair state.
12. Import block bracketed by session.start / session.end audit rows on local chain.
13. shareable values include v1 transitional (publishable, shared, client-visible).
14. Device-id auto-generated at first daemon start; persisted to `~/.cyberos/device_id`.
15. Buffered rows survive daemon restart (persisted to disk).
16. Push ack per row — sender doesn't proceed until ack received.

---

## §5 — Verification

```rust
#[tokio::test]
async fn append_on_a_visible_on_b_within_5s() {
    let mock_cloud = MockCloud::start();
    let device_a = test_device("a", &mock_cloud).await;
    let device_b = test_device("b", &mock_cloud).await;

    let row = test_helper::shareable_row("test_body");
    device_a.append(row).await.unwrap();

    let t0 = std::time::Instant::now();
    while t0.elapsed() < Duration::from_secs(6) {
        if device_b.has_imported(&row).await { return; }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("row not synced within 5s");
}

#[tokio::test]
async fn offline_buffers_50_rows_then_syncs_on_reconnect() {
    let mock_cloud = MockCloud::start();
    let device = test_device("a", &mock_cloud).await;
    mock_cloud.go_offline();
    for i in 0..50 {
        device.append(test_helper::shareable_row(&format!("body_{i}"))).await.unwrap();
    }
    let buffered: u64 = otel_test_helper::gauge_value("memory_sync_buffered_rows", &[]);
    assert_eq!(buffered, 50);
    mock_cloud.go_online();
    tokio::time::sleep(Duration::from_secs(3)).await;
    let final_buffered: u64 = otel_test_helper::gauge_value("memory_sync_buffered_rows", &[]);
    assert_eq!(final_buffered, 0);
    let received = mock_cloud.received_count();
    assert_eq!(received, 50);
}

#[tokio::test]
async fn private_rows_never_leave_device() {
    let mock_cloud = MockCloud::start();
    let device = test_device("a", &mock_cloud).await;
    device.append(test_helper::row_with_sync_class("private", "secret")).await.unwrap();
    tokio::time::sleep(Duration::from_secs(2)).await;
    assert_eq!(mock_cloud.received_count(), 0);
}

#[tokio::test]
async fn compensation_row_rejected_at_sync_boundary() {
    let mock_cloud = MockCloud::start();
    let device = test_device("a", &mock_cloud).await;
    let comp_row = test_helper::row_at_path("meta/people/alice/compensation.md", "shareable", "body");
    device.append(comp_row).await.unwrap();
    tokio::time::sleep(Duration::from_secs(2)).await;
    assert_eq!(mock_cloud.received_count(), 0);
    let metric: u64 = otel_test_helper::counter_value("memory_sync_compensation_rejected_total", &[]);
    assert_eq!(metric, 1);
}

#[tokio::test]
async fn concurrent_append_creates_disputed_pair() {
    let mock_cloud = MockCloud::start();
    let device_a = test_device("a", &mock_cloud).await;
    let device_b = test_device("b", &mock_cloud).await;

    let target_id = Uuid::new_v4();
    device_a.append_at_target(target_id, "version_a", now_ns()).await.unwrap();
    device_b.append_at_target(target_id, "version_b", now_ns()).await.unwrap();

    tokio::time::sleep(Duration::from_secs(3)).await;
    let pair_a = device_a.find_disputed_pair(target_id).await;
    let pair_b = device_b.find_disputed_pair(target_id).await;
    assert!(pair_a.is_some());
    assert!(pair_b.is_some());
    // Deterministic: both devices see same disputed_pair contents
    assert_eq!(pair_a.unwrap().version_a.content_hash, pair_b.unwrap().version_a.content_hash);
}

#[tokio::test]
async fn buffer_overflow_evicts_oldest() {
    let mock_cloud = MockCloud::start();
    let device = test_device("a", &mock_cloud).await;
    mock_cloud.go_offline();
    for i in 0..10_001 {
        device.append(test_helper::shareable_row(&format!("body_{i}"))).await.unwrap();
    }
    let metric: u64 = otel_test_helper::counter_value("memory_sync_buffer_overflow_total", &[]);
    assert_eq!(metric, 1);   // exactly one eviction
    let buffer_size = device.buffer_size().await;
    assert_eq!(buffer_size, 10_000);
}

#[tokio::test]
async fn foreign_chain_dedup() {
    let mock_cloud = MockCloud::start();
    let device = test_device("a", &mock_cloud).await;
    let foreign_row = test_helper::foreign_row_with_chain("chain_xyz");
    device.import_from_cloud(foreign_row.clone()).await.unwrap();
    device.import_from_cloud(foreign_row.clone()).await.unwrap();   // re-import same chain
    let count = device.local_rows_with_foreign_chain("chain_xyz").await;
    assert_eq!(count, 1);
}

#[tokio::test]
async fn import_brackets_with_session_start_end() {
    let mock_cloud = MockCloud::start();
    let device = test_device("a", &mock_cloud).await;
    mock_cloud.publish_rows(vec![test_helper::foreign_row("a"), test_helper::foreign_row("b")]).await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    let chain = device.local_chain().await;
    assert!(chain.iter().any(|r| r.kind == "session.start" && r.extra.imported_from.is_some()));
    assert!(chain.iter().any(|r| r.kind == "session.end" && r.extra.imported_from.is_some()));
}
```

---

## §6 — Implementation skeleton

See §3.

---

## §7 — Dependencies

- **TASK-MEMORY-101** — Layer 2 sees synced rows downstream.
- **TASK-MEMORY-104** — Tauri app embeds memory-sync.
- **TASK-MEMORY-105** — Disputed-pair UI (slice 2+).
- **TASK-MEMORY-106** — sync-class enforcement at write time (this task enforces at sync boundary).
- Cloud memory service (separate, P1+).
- Crates: `tonic@0.12`, `prost@0.13`, `tokio`, `sqlx`, `serde`, `glob`.

---

## §8 — Example payloads

### Push request (gRPC)

```proto
PushRequest {
    device_id: "device-laptop-mbp"
    seq: 12345
    ts_ns: 1747526400000000000
    body: <bytes>
    meta_json: '{"sync_class": "shareable", "kind": "decisions"}'
    prev_chain: <hash>
    chain: <hash>
    sync_class: "shareable"
}
```

### Disputed pair audit row

```json
{
  "kind": "memory.disputed_pair",
  "payload": {
    "target_memory_id": "550e...",
    "version_a": {
      "content_hash": "a3f9c8d7...",
      "ts_ns": 1747526400000000000,
      "originator_device": "device-mbp"
    },
    "version_b": {
      "content_hash": "9d6e3a2b...",
      "ts_ns": 1747526400500000000,
      "originator_device": "device-imac"
    }
  }
}
```

### Compensation rejection log

```text
ERROR seq=42 path="meta/people/alice/compensation.md" device_id="laptop-a"
      compensation row at sync boundary; sev-1 rejected
sev-1 memory_sync_compensation_rejected_total incremented
```

### Buffer overflow alert

```text
WARN seq=12345 device_id="laptop-a"
     buffer overflow; oldest evicted (was offline > 100 days)
sev-2 memory_sync_buffer_overflow_total incremented
```

---

## §9 — Open questions

All resolved. Deferred:
- E2E encryption of body in Cloud (slice 3+).
- Conflict auto-resolution heuristics — slice 4+.
- Cross-Cloud-memory federation (multi-region) — slice 5+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Network drop | health_check fails | Buffer locally; resume on reconnect | Self-heals |
| Cloud memory down | Same | Same | Self-heals |
| Conflict | CRDT detect | disputed_pair row created | User picks via TASK-MEMORY-105 UI |
| Buffer overflow (>10K) | Sev-2 alert; oldest rows dropped | Operator investigates extended outage | Standard ops |
| Foreign chain not anchored to local | per §14.2 fresh put | Each foreign import is a fresh put | By design |
| Compensation row at sync | path-based guard | Rejected; sev-1 metric | Investigate upstream code that emitted shareable comp |
| Per-device token revoked | Cloud 401 | Sync stops; sev-1 alarm | Operator issues new token |
| Token leaked | unauthorized syncs | sev-1; rotate | Standard incident |
| Concurrent push race | gRPC server handles | One succeeds; other gets ack | By design |
| Disk full on offline buffer | persist fails | Sev-1 alarm | Operator extends disk |
| Daemon crash mid-buffer | buffer persisted to disk | Recover on restart | By design (slice 2+ implements persistence; slice 1 in-memory) |
| Clock skew between devices | CRDT ts_ns may differ slightly | Conflict window 5s tolerates | By design |
| Import duplicates a chain | foreign_chain dedup | Skipped | By design (§1 #10) |
| sync_class missing | defaults to "private" | Stays local | By design (safe default) |
| Push of row > 1MB | gRPC max-message check | Rejected with PushAck.accepted=false | Caller chunks |
| Pull cursor lost | recompute from local chain | Re-imports happen; dedup catches | Self-heals |
| TLS cert expired | gRPC fails | Sync stops; sev-1 | Operator rotates cert |
| Disputed_pair already exists | UPSERT | No duplicate | By design |
| Metric collector down | local OTel buffer | Telemetry delayed | Self-heals |
| Daemon restart loses 5s buffer (in-memory) | known limitation slice 1 | Few rows lost | Slice 2+ persists buffer |

---

## §11 — Notes

- Cloud memory is the centralised hub (star topology); per-device sync via gRPC stream.
- The `sync_class` field is the user's privacy primitive. Default `private` (safe default).
- compensation_guard at sync boundary is defense-in-depth — even if a row is mistakenly tagged shareable, the path-based check catches comp rows.
- CRDT triple deterministic resolution means two devices independently produce the same disputed_pair state.
- 10K offline buffer covers ~100 days of typical usage (100 rows/day). Beyond that, manual reconciliation is expected.
- Per-device bearer token rotated quarterly via TASK-AUTH-006-style sweeper.
- Import-as-fresh-put per §14.2 preserves local chain integrity; foreign chain doesn't merge directly.
- session.start / session.end brackets give compliance audits clear "imported from X at time Y" demarcation.
- Slice 1 buffer is in-memory; slice 2+ persists to disk for daemon-restart recovery.

---

*End of TASK-MEMORY-103. Status: done (implemented 2026-05-23).*

## As built (2026-07-02)

Shipped in python (modules/memory), not the Rust service path named above.
