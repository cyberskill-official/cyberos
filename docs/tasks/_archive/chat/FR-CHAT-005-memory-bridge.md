---
id: TASK-CHAT-005
title: "memory bridge — Postgres logical replication from chat to memory Layer-3 ingest with p95 ≤ 5s latency"
module: CHAT
priority: MUST
status: superseded
superseded_by: TASK-CHAT-101 (first-party native chat replaced the Mattermost fork wholesale; still-wanted intents re-homed as TASK-CHAT-102..106)
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_tasks: [TASK-CHAT-003, TASK-CHAT-006, TASK-CHAT-008, TASK-CHAT-012, TASK-MEMORY-101, TASK-MEMORY-107, TASK-MEMORY-111]
depends_on: [TASK-CHAT-003]
blocks: [TASK-CHAT-006, TASK-CHAT-008, TASK-CHAT-012, TASK-PORTAL-006]

source_pages:
  - website/docs/modules/chat.html#memory-bridge
source_decisions:
  - DEC-460 (Postgres logical replication; chat → bridge process → memory; never direct app-side write)
  - DEC-461 (sync_class derived from channel privacy; private channel = sync_class private)
  - DEC-462 (p95 chat-msg-to-memory ≤ 5s; sev-2 alarm above)

language: rust 1.81
service: cyberos/services/chat-memory-bridge/
new_files:
  - services/chat-memory-bridge/Cargo.toml
  - services/chat-memory-bridge/src/main.rs
  - services/chat-memory-bridge/src/replication.rs
  - services/chat-memory-bridge/src/map.rs
  - services/chat-memory-bridge/tests/bridge_test.rs
modified_files:
  - infra/terraform/modules/tenant_chat/rds.tf      # enable wal_level=logical
  - services/chat/sql/init-bridge-publication.sql
allowed_tools:
  - file_read: services/chat-memory-bridge/**
  - file_write: services/chat-memory-bridge/{src,tests}/**, services/chat/sql/**
  - bash: cd services/chat-memory-bridge && cargo test
disallowed_tools:
  - skip PII redaction before memory write (per TASK-MEMORY-111)
  - write to memory bypassing TASK-MEMORY-107 capture daemon (per AGENTS.md §14.1)

effort_hours: 10
subtasks:
  - "0.5h: rds.tf — wal_level=logical + replication slot params"
  - "0.5h: init-bridge-publication.sql — CREATE PUBLICATION chat_bridge FOR TABLE posts, channels"
  - "0.5h: Cargo.toml deps (tokio-postgres, pgreplication)"
  - "1.5h: main.rs — bridge daemon entry; signal handlers; reconnect loop"
  - "2.0h: replication.rs — consume logical replication stream; parse INSERT/UPDATE events"
  - "1.5h: map.rs — map MM post → memory row {kind: chat.message, payload: {channel_id, user_id, body_redacted, sync_class, trace_id}}"
  - "0.5h: integration with TASK-MEMORY-107 capture daemon via Unix socket"
  - "0.5h: memory audit kinds (chat.message | chat.channel_created | chat.channel_archived)"
  - "0.5h: latency tracking: msg.create_at vs memory seq emit time"
  - "1.5h: bridge_test.rs — happy + lag detection + PII redaction"
  - "0.5h: OTel histogram chat_memory_bridge_latency_seconds"
risk_if_skipped: "Chat messages stay in chat DB forever; never indexed for cross-chat search / cross-module retrieval. Without async bridge, app-side dual-write = transaction complexity. Without redaction, customer PII lands in memory. Without 5s SLA, search is stale at the surface. Logical replication keeps the chat app's write path fast (no synchronous external call)."
---

## §1 — Description (BCP-14 normative)

The bridge **MUST** consume Postgres logical replication from chat DB and emit memory rows via TASK-MEMORY-107 socket. The contract:

1. **MUST** enable PostgreSQL logical replication on tenant_chat RDS: `wal_level=logical`, `max_replication_slots=4`, `max_wal_senders=4`.
2. **MUST** create publication `chat_bridge FOR TABLE posts, channels`; bridge subscribes via `CREATE_REPLICATION_SLOT cyberos_bridge LOGICAL pgoutput`.
3. **MUST** run as a separate process per tenant (Fargate task, sidecar to chat service); auto-restart on crash.
4. **MUST** parse incoming WAL events:
    - `INSERT posts` → emit `chat.message` memory row.
    - `UPDATE posts SET deleted_at = ...` → emit `chat.message_deleted`.
    - `INSERT channels` → emit `chat.channel_created`.
    - `UPDATE channels SET delete_at = ...` → emit `chat.channel_archived`.
5. **MUST** derive `sync_class` from channel's privacy:
    - Private channel → `sync_class: private`.
    - Public channel (team-wide) → `sync_class: shareable` with `acl: []` (any tenant member).
    - DM (direct message) → `sync_class: private`.
6. **MUST** redact PII via TASK-MEMORY-111 ruleset BEFORE emit.
7. **MUST** propagate W3C trace_id: read from post's `props.cyberos_trace_id` if present; else generate new at bridge.
8. **MUST** track latency: measure `now() - post.create_at` at emit time; emit histogram `chat_memory_bridge_latency_seconds`.
9. **MUST** emit memory audit `chat.bridge_lag_alert` when p95 latency > 5s sustained 60s (sev-2 routing via TASK-OBS-007).
10. **MUST** persist replication slot LSN; on restart, resume from last-committed LSN (no message loss).
11. **MUST** be tenant-scoped: bridge process knows its tenant_id; emits with that tenant_id only.
12. **MUST** emit OTel metrics:
    - `chat_memory_bridge_messages_total{kind, outcome}` (counter; outcome ∈ emitted | redaction_failed | memory_write_failed).
    - `chat_memory_bridge_latency_seconds`.
    - `chat_memory_bridge_lag_lsn` (gauge; chat WAL LSN minus consumed LSN).
13. **MUST** handle memory socket unavailable: bridge pauses (does NOT advance LSN); resumes when socket up; alarm sev-2 if down > 60s.
14. **MUST** be idempotent at the message level: re-emit of the same `(tenant_id, post_id, version)` triple MUST produce a memory row whose `dedup_key` field equals the prior row's `dedup_key`. TASK-MEMORY-107 uses `dedup_key` to drop duplicates that arise from restart-mid-LSN-advance.
15. **MUST NOT** advance the replication slot past an LSN whose memory emit ack has not been received. The bridge maintains a `pending_acks: BTreeMap<Lsn, AckHandle>` and only calls `pg_replication_slot_advance` for LSNs whose ack has resolved. This is the source of the exactly-once-modulo-dedup guarantee.
16. **MUST** treat `UPDATE posts SET message = ...` (Mattermost message edit) as a separate row kind `chat.message_edited` carrying both the new body and a pointer to the prior version's `dedup_key`. The original row is not retracted; downstream consumers see the edit as a new event.
17. **MUST** detect and emit `chat.message_reacted` rows for `INSERT INTO reactions` events; payload `{post_id, user_id, emoji_name, create_at_ns}`. Reactions are a separate publication target.
18. **MUST** emit `chat.user_joined_channel` and `chat.user_left_channel` rows from `channelmembers` table INSERT / DELETE events. These are required by TASK-CHAT-008 mention resolution and TASK-CHAT-012 DSAR export.
19. **MUST** include a `chat_db_replica_safety` check at startup: query `SELECT pg_is_in_recovery()` and refuse to start if the bridge is connected to a read replica (replicas don't have logical replication slots). Emits SEV-1 `chat.bridge_misconfigured` audit on refusal.
20. **MUST** publish a heartbeat row `chat.bridge_heartbeat` every 30s with `{lsn_consumed, lsn_max, lag_bytes, lag_seconds, slot_active}` so operators can monitor liveness without inspecting metrics.
21. **MUST** support graceful shutdown via SIGTERM:
    - Stop accepting new replication events.
    - Drain in-flight memory emits with a 10s grace window.
    - Persist final LSN to memory as `chat.bridge_shutdown` row.
    - Exit 0.
    SIGKILL is allowed but loses up to one in-flight message (recovered via dedup on restart).
22. **MUST** scrub Mattermost's special characters that look like PII but aren't: Mattermost's `@all`, `@channel`, `@here` mentions; emoji codes like `:smile:`; markdown link references `[text](url)`. The PII scrubber MUST run AFTER these are normalised so the regex windows match accurately.
23. **MUST** include a `replay_safety_check` on first start after a major version bump: scan the publication's `relfilenode` to detect a pgoutput message-format change, and refuse to start if the format is incompatible. Operator runs `cyberos chat-bridge force-replay --from-lsn <lsn>` after coordinating with TASK-MEMORY-107 schema update.
24. **MUST** emit a `chat.message_attachment` row separate from `chat.message` when a post has `file_ids`. Payload `{post_id, file_id, filename_redacted, mime, size_bytes, sync_class}`. Filenames may carry PII (e.g. `Resume - Trinh Thai Anh.pdf`); the redactor runs on filenames too.
25. **MUST NOT** read the chat DB's `Sessions` table — login events are owned by TASK-CHAT-002, not by the bridge. The publication scope is explicitly `posts, channels, reactions, channelmembers, fileinfo`; no other tables.

---

## §2 — Why this design (rationale for humans)

**Why logical replication (DEC-460)?** Dual-write from app = transaction complexity + latency. Logical replication = async, decoupled, native Postgres. Bridge can crash + restart without losing messages.

**Why per-tenant bridge (§1 #3)?** Single bridge = noisy neighbour + tenant_id ambiguity. Per-tenant = bounded scope + independent failure domains.

**Why sync_class from channel privacy (DEC-461)?** Operator intuition: private channels are private; public channels are shareable. Channel privacy is the natural primitive.

**Why p95 ≤ 5s (DEC-462)?** Users searching memory expect "I just said it in chat" to be findable within seconds. 5s is the calibrated SLA; alarm above forces investigation.

**Why persist LSN (§1 #10)?** Restart-loss = silent data loss. LSN persistence = exactly-once delivery (modulo memory-write idempotency).

**Why pause on memory down (§1 #13)?** Advancing LSN without successful memory write = data loss on permanent failure. Pause = back-pressure + data integrity.

**Why dedup_key (§1 #14)?** Even with pause-on-failure, a bridge crash AFTER memory ack but BEFORE LSN advance causes re-emit on restart. TASK-MEMORY-107 needs a deterministic key to dedup — we compute it as `sha256(tenant_id || post_id || version)` so the same logical event always produces the same key.

**Why pending_acks BTreeMap (§1 #15)?** memory emits are async; if we advance the LSN past an unacked message, a crash loses it. The BTreeMap keeps LSNs in order so we can only advance to the highest-acked contiguous LSN.

**Why message_edited as new row, not overwrite (§1 #16)?** Append-only audit. Operators investigating "what was said before the edit" must see the original. Edit metadata (pointer to prior dedup_key) lets downstream UI render "edited from..." without losing history.

**Why reactions as separate rows (§1 #17)?** A message + 20 reactions = 21 events. Bundling them produces fragile derived state ("the reaction count changed from 19 to 20 — was that an add or a delete?"). Separate rows mirror the source-of-truth shape.

**Why channelmember events (§1 #18)?** TASK-CHAT-008 mentions resolve from channelmembers (who can see a mention?). TASK-CHAT-012 DSAR needs the timeline of "what channels did user X access." Without these events, those tasks would have to query the chat DB directly — coupling we want to avoid.

**Why refuse to start on read replica (§1 #19)?** Postgres logical replication only works on the primary. A bridge connected to a replica would silently consume nothing. Loud failure at startup is far better than silent no-op.

**Why heartbeat row (§1 #20)?** Operators answering "is the bridge running" without OBS dashboard access need a memory-side signal. A heartbeat every 30s is far cheaper than tailing logs.

**Why SIGTERM graceful drain (§1 #21)?** Fargate scale-in sends SIGTERM with a 30s grace; we use 10s for drain so the remaining 20s covers TCP cleanup. Without drain, every scale-in event loses up to one message per task.

**Why normalise Mattermost specials before PII scan (§1 #22)?** Real example: `@channel — please send to alice@cyberskill.world`. The PII scrubber regex `\w+@\w+\.\w+` would match `@channel` if not normalised first. False-positive scrubs are not security failures but they're noisy and operator-confusing.

**Why replay_safety_check (§1 #23)?** Postgres pgoutput format has changed between major versions. A bridge that consumed v15-format messages would mis-parse v16 messages on the same publication. Explicit version-pinning prevents silent corruption.

**Why scope publication explicitly (§1 #25)?** Default `ALTER PUBLICATION chat_bridge FOR ALL TABLES` would include Mattermost's `Sessions` table — leaking login events to memory that should be owned by TASK-CHAT-002. Explicit table list keeps the contract surface narrow.

---

## §3 — API contract (key sketches)

### Cargo.toml — pinned crate set

```toml
[package]
name    = "cyberos-chat-memory-bridge"
version = "0.1.0"
edition = "2021"
rust-version = "1.81"

[dependencies]
tokio              = { version = "1.40", features = ["full"] }
tokio-postgres     = { version = "0.7", features = ["with-uuid-1"] }
postgres-protocol  = "0.6"                   # raw pgoutput parsing
bytes              = "1.7"
anyhow             = "1.0"
thiserror          = "1.0"
serde              = { version = "1.0", features = ["derive"] }
serde_json         = "1.0"
uuid               = { version = "1.10", features = ["serde", "v7"] }
sha2               = "0.10"
hex                = "0.4"
tracing            = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
metrics            = "0.23"
metrics-exporter-prometheus = "0.15"
opentelemetry      = "0.24"
opentelemetry_sdk  = "0.24"
opentelemetry-otlp = "0.17"
cyberos-memory-client = { path = "../../crates/cyberos-memory-client" }
cyberos-memory-pii    = { path = "../../crates/cyberos-memory-pii" }
clap               = { version = "4.5", features = ["derive", "env"] }

[dev-dependencies]
testcontainers     = "0.21"
pretty_assertions  = "1.4"
rstest             = "0.22"
proptest           = "1.5"
```

### init-bridge-publication.sql

```sql
-- services/chat/sql/init-bridge-publication.sql
-- Owned by TASK-CHAT-005; executed once per tenant at first deploy
-- via cyberos-chat-bridge --init-publication.

BEGIN;

-- Publication scope is EXPLICIT — not FOR ALL TABLES.
-- Per TASK-CHAT-005 §1 #25, Sessions table is NOT in this set.
DROP PUBLICATION IF EXISTS chat_bridge;

CREATE PUBLICATION chat_bridge
  FOR TABLE
      posts,
      channels,
      channelmembers,
      reactions,
      fileinfo
  WITH (publish = 'insert,update,delete', publish_via_partition_root = true);

-- Helper view: post + channel + user joined in one selectable for downstream
-- recovery scripts that need post context without re-joining.
CREATE OR REPLACE VIEW chat_bridge_post_context AS
  SELECT p.id        AS post_id,
         p.channel_id,
         c.type      AS channel_type,
         c.team_id,
         p.user_id,
         p.create_at,
         p.update_at,
         p.delete_at,
         p.props,
         p.file_ids,
         p.message,
         (p.update_at > p.create_at) AS was_edited
  FROM   posts  p
  JOIN   channels c ON c.id = p.channel_id;

-- Role for the bridge — least-privilege: SELECT on publication tables,
-- REPLICATION attribute, USAGE on schema only.
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'cyberos_chat_bridge') THEN
    CREATE ROLE cyberos_chat_bridge LOGIN REPLICATION
      PASSWORD '<set-via-secretsmanager>';
  END IF;
END $$;

GRANT USAGE  ON SCHEMA public TO cyberos_chat_bridge;
GRANT SELECT ON posts, channels, channelmembers, reactions, fileinfo, chat_bridge_post_context
       TO cyberos_chat_bridge;

COMMIT;
```

### main.rs — process entry, signal handling, reconnect

```rust
// services/chat-memory-bridge/src/main.rs
use anyhow::Result;
use clap::Parser;
use std::time::Duration;
use tokio::signal::unix::{signal, SignalKind};
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(name = "cyberos-chat-memory-bridge")]
struct Args {
    /// Tenant UUID this bridge serves.
    #[arg(long, env = "CYBEROS_TENANT_ID")]
    tenant_id: uuid::Uuid,

    /// Postgres DSN — must point at the chat RDS primary, not a replica.
    #[arg(long, env = "CYBEROS_CHAT_DSN")]
    chat_dsn: String,

    /// memory writer Unix socket / TCP endpoint (TASK-MEMORY-107).
    #[arg(long, env = "CYBEROS_MEMORY_WRITER_SOCK")]
    memory_socket: String,

    /// If set, run the init-publication SQL and exit.
    #[arg(long)]
    init_publication: bool,

    /// Reconnect backoff cap (seconds).
    #[arg(long, default_value_t = 30, env = "CYBEROS_RECONNECT_MAX_BACKOFF")]
    reconnect_max_backoff_secs: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let args = Args::parse();

    if args.init_publication {
        return init_publication(&args.chat_dsn).await;
    }

    let memory = cyberos_memory_client::connect(&args.memory_socket).await?;
    let shutdown_rx = install_signal_handlers().await;

    let mut backoff = Duration::from_millis(500);
    loop {
        match run_bridge(&args, &memory, shutdown_rx.clone()).await {
            Ok(ShutdownReason::Signal) => {
                info!("graceful shutdown complete");
                return Ok(());
            }
            Ok(ShutdownReason::ReplicationEnded) => {
                warn!("replication stream ended; reconnecting");
            }
            Err(e) => {
                error!(?e, ?backoff, "bridge error; backing off");
                metrics::counter!("chat_memory_bridge_reconnects_total",
                    "reason" => classify_error(&e)).increment(1);
                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(Duration::from_secs(args.reconnect_max_backoff_secs));
                continue;
            }
        }
        backoff = Duration::from_millis(500); // reset on clean reconnect
    }
}

async fn install_signal_handlers() -> tokio::sync::watch::Receiver<bool> {
    let (tx, rx) = tokio::sync::watch::channel(false);
    tokio::spawn(async move {
        let mut sigterm = signal(SignalKind::terminate()).unwrap();
        let mut sigint  = signal(SignalKind::interrupt()).unwrap();
        tokio::select! {
            _ = sigterm.recv() => info!("SIGTERM received; initiating drain"),
            _ = sigint.recv()  => info!("SIGINT received; initiating drain"),
        }
        let _ = tx.send(true);
    });
    rx
}

async fn replica_safety_check(client: &tokio_postgres::Client) -> Result<()> {
    let row = client.query_one("SELECT pg_is_in_recovery()", &[]).await?;
    let in_recovery: bool = row.get(0);
    if in_recovery {
        cyberos_memory_client::emit(MemoryRow {
            kind: "chat.bridge_misconfigured".into(),
            severity: Some("SEV-1".into()),
            payload: serde_json::json!({
                "reason": "connected to read replica; logical replication unavailable",
                "remediation": "point CYBEROS_CHAT_DSN at the primary endpoint",
            }),
            ..Default::default()
        }).await?;
        anyhow::bail!("connected to read replica; refusing to start");
    }
    Ok(())
}
```

### replication.rs — exactly-once-modulo-dedup loop

```rust
// services/chat-memory-bridge/src/replication.rs
use postgres_protocol::message::backend::{LogicalReplicationMessage, ReplicationMessage};
use std::collections::BTreeMap;
use tokio::sync::oneshot;

pub async fn run_replication_loop(
    client: tokio_postgres::Client,
    tenant_id: uuid::Uuid,
    memory: &cyberos_memory_client::Client,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) -> Result<ShutdownReason> {
    replica_safety_check(&client).await?;

    let slot_name = format!("cyberos_bridge_{}", tenant_id.simple());
    ensure_slot_exists(&client, &slot_name).await?;

    let mut stream = client.copy_both_simple::<bytes::Bytes>(&format!(
        "START_REPLICATION SLOT {} LOGICAL 0/0 (proto_version '1', publication_names 'chat_bridge')",
        slot_name
    )).await?;

    // pending_acks: outstanding emits keyed by LSN. We can only advance the
    // slot past the highest contiguous LSN whose ack has resolved.
    let mut pending_acks: BTreeMap<u64, oneshot::Receiver<bool>> = BTreeMap::new();
    let mut highest_committed_lsn: u64 = 0;
    let mut heartbeat_ticker = tokio::time::interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            biased;

            // Shutdown: drain pending, persist final LSN, exit.
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    drain_pending(&mut pending_acks, Duration::from_secs(10)).await;
                    advance_slot(&client, &slot_name, highest_committed_lsn).await?;
                    memory.emit(memory_row_shutdown(tenant_id, highest_committed_lsn)).await?;
                    return Ok(ShutdownReason::Signal);
                }
            }

            // Heartbeat.
            _ = heartbeat_ticker.tick() => {
                let lag = compute_lag(&client, &slot_name).await?;
                memory.emit(memory_row_heartbeat(tenant_id, highest_committed_lsn, lag)).await?;
                metrics::gauge!("chat_memory_bridge_lag_lsn").set(lag.bytes as f64);
            }

            // Next replication message.
            msg = stream.next() => {
                let Some(msg) = msg else { return Ok(ShutdownReason::ReplicationEnded); };
                let parsed = ReplicationMessage::parse(&msg?)?;
                match parsed {
                    ReplicationMessage::XLogData(xlog) => {
                        let event = decode_pgoutput(xlog.data())?;
                        let lsn   = xlog.wal_start();
                        let row   = map_event_to_memory_row(event.clone(), tenant_id)?;

                        // PII redaction BEFORE emit. Failure = drop + audit.
                        let normalised = normalise_mattermost_specials(&row.body);
                        let redacted = match cyberos_memory_pii::scan_and_redact(&normalised, &[]).await {
                            Ok(r) => r,
                            Err(e) => {
                                error!(?e, post_id = ?event.post_id(), "pii redaction failed; dropping");
                                memory.emit(memory_row_redaction_failed(tenant_id, event.post_id(), e.to_string())).await?;
                                metrics::counter!("chat_memory_bridge_messages_total",
                                    "kind" => row.kind.clone(),
                                    "outcome" => "redaction_failed").increment(1);
                                // We advance LSN — the source-of-truth data
                                // is in chat; we're not blocking on broken redaction.
                                advance_slot(&client, &slot_name, lsn).await?;
                                highest_committed_lsn = lsn;
                                continue;
                            }
                        };

                        let final_row = row.with_body(redacted.redacted_body)
                            .with_dedup_key(compute_dedup_key(tenant_id, &event));
                        let (ack_tx, ack_rx) = oneshot::channel();
                        memory.emit_with_ack(final_row, ack_tx).await?;
                        pending_acks.insert(lsn, ack_rx);
                    }
                    ReplicationMessage::PrimaryKeepAlive(ka) => {
                        if ka.reply() == 1 {
                            send_standby_status(&mut stream, highest_committed_lsn).await?;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Drain finished acks into highest_committed_lsn.
        let mut to_drop = Vec::new();
        for (lsn, rx) in pending_acks.iter_mut() {
            match rx.try_recv() {
                Ok(true)  => to_drop.push(*lsn),
                Ok(false) => break, // ack arrived false — emit failed; stop advancing
                Err(tokio::sync::oneshot::error::TryRecvError::Empty) => break, // not yet
                Err(_)    => break,
            }
        }
        for lsn in to_drop {
            pending_acks.remove(&lsn);
            highest_committed_lsn = lsn;
        }
        if highest_committed_lsn > 0 {
            advance_slot(&client, &slot_name, highest_committed_lsn).await?;
        }
    }
}

fn compute_dedup_key(tenant_id: uuid::Uuid, event: &ReplicationEvent) -> String {
    use sha2::{Sha256, Digest};
    let mut h = Sha256::new();
    h.update(tenant_id.as_bytes());
    h.update(event.post_id().as_bytes());
    h.update(event.version().to_string().as_bytes());
    hex::encode(h.finalize())
}
```

### map.rs — pgoutput → memory row

```rust
// services/chat-memory-bridge/src/map.rs

#[derive(Clone, Debug)]
pub enum ReplicationEvent {
    PostInsert(Post),
    PostUpdate { old: Post, new: Post },
    PostDelete(Post),
    ChannelInsert(Channel),
    ChannelArchive(Channel),
    ReactionInsert(Reaction),
    ReactionDelete(Reaction),
    ChannelMemberInsert(ChannelMember),
    ChannelMemberDelete(ChannelMember),
    FileInfoInsert(FileInfo),
}

pub fn map_event_to_memory_row(
    event: ReplicationEvent,
    tenant_id: uuid::Uuid,
) -> Result<MemoryRow> {
    let row = match event {
        ReplicationEvent::PostInsert(p) => {
            let sync_class = derive_sync_class(&p);
            MemoryRow {
                kind: "chat.message".into(),
                tenant_id,
                payload: serde_json::json!({
                    "post_id":     p.id,
                    "channel_id":  p.channel_id,
                    "user_id":     p.user_id,
                    "body":        p.message,
                    "sync_class":  sync_class,
                    "create_at_ns": p.create_at,
                    "trace_id":    extract_trace_id(&p.props),
                    "has_attachments": !p.file_ids.is_empty(),
                }),
                ..Default::default()
            }
        }
        ReplicationEvent::PostUpdate { old, new } if old.message != new.message => {
            MemoryRow {
                kind: "chat.message_edited".into(),
                tenant_id,
                payload: serde_json::json!({
                    "post_id":         new.id,
                    "channel_id":      new.channel_id,
                    "user_id":         new.user_id,
                    "body":            new.message,
                    "sync_class":      derive_sync_class(&new),
                    "prior_dedup_key": compute_dedup_key(tenant_id, &ReplicationEvent::PostInsert(old.clone())),
                    "edit_at_ns":      new.update_at,
                    "trace_id":        extract_trace_id(&new.props),
                }),
                ..Default::default()
            }
        }
        ReplicationEvent::PostUpdate { new, .. } if new.delete_at > 0 => {
            MemoryRow {
                kind: "chat.message_deleted".into(),
                tenant_id,
                payload: serde_json::json!({
                    "post_id":       new.id,
                    "channel_id":    new.channel_id,
                    "deleted_at_ns": new.delete_at,
                }),
                ..Default::default()
            }
        }
        ReplicationEvent::ChannelInsert(c) => MemoryRow {
            kind: "chat.channel_created".into(),
            tenant_id,
            payload: serde_json::json!({
                "channel_id":  c.id,
                "channel_type": c.type_,
                "team_id":     c.team_id,
                "display_name": c.display_name,
            }),
            ..Default::default()
        },
        ReplicationEvent::ChannelArchive(c) => MemoryRow {
            kind: "chat.channel_archived".into(),
            tenant_id,
            payload: serde_json::json!({
                "channel_id":    c.id,
                "archived_at_ns": c.delete_at,
            }),
            ..Default::default()
        },
        ReplicationEvent::ReactionInsert(r) => MemoryRow {
            kind: "chat.message_reacted".into(),
            tenant_id,
            payload: serde_json::json!({
                "post_id":     r.post_id,
                "user_id":     r.user_id,
                "emoji_name":  r.emoji_name,
                "create_at_ns": r.create_at,
            }),
            ..Default::default()
        },
        ReplicationEvent::ReactionDelete(r) => MemoryRow {
            kind: "chat.message_unreacted".into(),
            tenant_id,
            payload: serde_json::json!({
                "post_id":    r.post_id,
                "user_id":    r.user_id,
                "emoji_name": r.emoji_name,
            }),
            ..Default::default()
        },
        ReplicationEvent::ChannelMemberInsert(m) => MemoryRow {
            kind: "chat.user_joined_channel".into(),
            tenant_id,
            payload: serde_json::json!({
                "channel_id": m.channel_id,
                "user_id":    m.user_id,
                "joined_at_ns": m.msg_count, // Mattermost reuses MsgCount as join marker
            }),
            ..Default::default()
        },
        ReplicationEvent::ChannelMemberDelete(m) => MemoryRow {
            kind: "chat.user_left_channel".into(),
            tenant_id,
            payload: serde_json::json!({
                "channel_id": m.channel_id,
                "user_id":    m.user_id,
            }),
            ..Default::default()
        },
        ReplicationEvent::FileInfoInsert(f) => MemoryRow {
            kind: "chat.message_attachment".into(),
            tenant_id,
            payload: serde_json::json!({
                "post_id":      f.post_id,
                "file_id":      f.id,
                "filename":     f.name, // redacted at caller
                "mime":         f.mime_type,
                "size_bytes":   f.size,
                "sync_class":   derive_sync_class_from_post(&f.post_id),
            }),
            ..Default::default()
        },
        // No-op cases (post update with no semantic change).
        _ => return Err(anyhow::anyhow!("event not mapped")),
    };
    Ok(row)
}

pub fn derive_sync_class(post: &Post) -> &'static str {
    match post.channel_type.as_str() {
        "O" => "shareable",      // open / public
        "P" => "private",        // private channel
        "D" => "private",        // direct message
        "G" => "private",        // group DM
        _   => "private",        // fail-secure on unknown
    }
}

pub fn extract_trace_id(props: &serde_json::Value) -> String {
    props.get("cyberos_trace_id")
         .and_then(|v| v.as_str())
         .map(|s| s.to_owned())
         .unwrap_or_else(generate_trace_id)
}

pub fn normalise_mattermost_specials(body: &str) -> String {
    // Strip @all, @channel, @here, emoji codes, markdown links before PII scan.
    let s = body
        .replace("@all",     "[mention_all]")
        .replace("@channel", "[mention_channel]")
        .replace("@here",    "[mention_here]");
    let emoji_re = regex::Regex::new(r":[a-z_]+:").unwrap();
    let s = emoji_re.replace_all(&s, "[emoji]").into_owned();
    let md_link_re = regex::Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
    md_link_re.replace_all(&s, "$1 [link]").into_owned()
}
```

### lsn.rs — slot advancement

```rust
// services/chat-memory-bridge/src/lsn.rs
use postgres_types::Type;

pub async fn advance_slot(
    client: &tokio_postgres::Client,
    slot: &str,
    lsn: u64,
) -> Result<()> {
    // pg_replication_slot_advance is the recommended primitive in PG 16+.
    let lsn_str = format!("{:X}/{:X}", lsn >> 32, lsn & 0xFFFF_FFFF);
    client.execute(
        "SELECT pg_replication_slot_advance($1, $2::pg_lsn)",
        &[&slot, &lsn_str]
    ).await?;
    Ok(())
}

pub async fn ensure_slot_exists(client: &tokio_postgres::Client, slot: &str) -> Result<()> {
    let row = client.query_opt(
        "SELECT 1 FROM pg_replication_slots WHERE slot_name = $1", &[&slot]
    ).await?;
    if row.is_none() {
        client.execute(&format!(
            "SELECT pg_create_logical_replication_slot('{}', 'pgoutput')", slot
        ), &[]).await?;
    }
    Ok(())
}

pub async fn compute_lag(client: &tokio_postgres::Client, slot: &str) -> Result<Lag> {
    let row = client.query_one(
        "SELECT
            pg_wal_lsn_diff(pg_current_wal_lsn(), confirmed_flush_lsn) AS lag_bytes,
            EXTRACT(EPOCH FROM (now() - last_msg_send_time))           AS lag_seconds,
            active
         FROM   pg_replication_slots
         WHERE  slot_name = $1",
        &[&slot]
    ).await?;
    Ok(Lag {
        bytes:   row.get(0),
        seconds: row.get(1),
        active:  row.get(2),
    })
}

pub struct Lag { pub bytes: i64, pub seconds: f64, pub active: bool }
```

---

## §4 — Acceptance criteria

1. **Publication created on RDS init**.
2. **Slot created on bridge first start**.
3. **INSERT post → chat.message row in memory within 5s**.
4. **UPDATE post deleted_at → chat.message_deleted**.
5. **INSERT channel → chat.channel_created**.
6. **UPDATE channel delete_at → chat.channel_archived**.
7. **Private channel → sync_class: private**.
8. **Public channel → sync_class: shareable**.
9. **DM → sync_class: private**.
10. **PII redacted (email in message → <EMAIL> in memory row)**.
11. **trace_id from props.cyberos_trace_id when present**.
12. **trace_id generated when absent**.
13. **LSN advances on successful memory write**.
14. **LSN NOT advanced on memory failure** — fixture: kill memory socket; insert message; restart; replay.
15. **Latency histogram populated**.
16. **Bridge_lag_alert fires at p95 > 5s sustained 60s**.
17. **Per-tenant isolation: cross-tenant messages never emitted under wrong tenant_id**.
18. **Reconnect after RDS restart** — bridge resumes from last LSN.
19. **dedup_key is deterministic** — same `(tenant_id, post_id, version)` → same `dedup_key` across runs and pods (AC for §1 #14).
20. **LSN advance only past acked emits** — fixture: emit two messages, ack second-only, observe slot at first-message LSN, not second (AC for §1 #15).
21. **Edit emits chat.message_edited with prior_dedup_key** — fixture: insert post; update message; observe memory row whose payload `prior_dedup_key` equals dedup_key of original (AC for §1 #16).
22. **Reaction insert → chat.message_reacted** — fixture: insert reaction; observe memory row with `emoji_name` (AC for §1 #17).
23. **channelmember insert → chat.user_joined_channel** — fixture: add user to channel; observe memory row (AC for §1 #18).
24. **channelmember delete → chat.user_left_channel** — fixture: remove user from channel; observe memory row (AC for §1 #18).
25. **Read-replica connection refused at startup** — fixture: point DSN at replica; observe SEV-1 `chat.bridge_misconfigured` memory row + process exit non-zero (AC for §1 #19).
26. **Heartbeat row every 30s** — fixture: idle bridge; observe `chat.bridge_heartbeat` row at t≈30s (AC for §1 #20).
27. **SIGTERM drains in 10s** — fixture: send 100 messages then SIGTERM; observe all 100 emitted to memory before process exits; observe `chat.bridge_shutdown` row (AC for §1 #21).
28. **@channel doesn't trigger email regex** — fixture: send `@channel — alice@x.com`; observe redacted body retains `[mention_channel]` and `<EMAIL>` (AC for §1 #22).
29. **pgoutput version mismatch refused at startup** — fixture: stub pgoutput to return unknown protocol version; observe SEV-1 audit + exit (AC for §1 #23).
30. **Attachment filename redacted** — fixture: post with file `Resume - Trinh Thai Anh.pdf`; observe `chat.message_attachment` row with redacted filename (AC for §1 #24).
31. **Sessions table NOT in publication** — fixture: insert into Sessions; observe no memory row (AC for §1 #25).
32. **Crash mid-LSN-advance produces duplicate emit, dedup_key identical** — fixture: kill bridge after memory ack but before slot advance; restart; observe two emits with same dedup_key; TASK-MEMORY-107 dedup retains one.
33. **Per-tenant slot naming prevents cross-tenant collision** — fixture: two bridges with tenant_id A and B against same DB; observe two slots `cyberos_bridge_<A>` and `cyberos_bridge_<B>`.
34. **chat.bridge_lag_alert payload includes lag_bytes + lag_seconds** — fixture: stall memory; wait for alarm; observe payload completeness.
35. **No table outside publication scope is read** — `pg_stat_user_tables` for the bridge role shows reads only on `posts, channels, channelmembers, reactions, fileinfo`; verified by `tests/scope-isolation.py`.
36. **Replication slot doesn't bloat on idle** — fixture: idle bridge for 1h; observe `pg_wal` size stable; bridge sends standby-status keepalives.

---

## §5 — Verification

`tests/bridge_test.rs` spins up a real Postgres + fake memory socket via `testcontainers`. Helpers are in `tests/common/mod.rs`.

### Test harness

```rust
// tests/common/mod.rs
use testcontainers::*;
pub struct TestEnv {
    pub pg:      testcontainers::Container<images::postgres::Postgres>,
    pub memory:   FakeMemorySocket,
    pub bridge:  tokio::task::JoinHandle<anyhow::Result<()>>,
    pub tenant:  uuid::Uuid,
}
impl TestEnv {
    pub async fn new() -> Self {
        let pg = images::postgres::Postgres::default()
            .with_env_var("POSTGRES_PASSWORD", "test")
            .with_cmd(vec!["postgres", "-c", "wal_level=logical",
                                       "-c", "max_replication_slots=4",
                                       "-c", "max_wal_senders=4",
                                       "-c", "shared_preload_libraries=pg_stat_statements"]);
        let pg = pg.start().await.unwrap();
        // Run init-bridge-publication.sql
        run_sql_file(&pg, "services/chat/sql/init-bridge-publication.sql").await;
        // Stub posts/channels tables for Mattermost shape
        run_sql_file(&pg, "tests/fixtures/mm-schema.sql").await;

        let memory = FakeMemorySocket::start().await;
        let tenant = uuid::Uuid::now_v7();
        let bridge_args = bridge::Args {
            tenant_id: tenant,
            chat_dsn: pg.dsn(),
            memory_socket: memory.endpoint(),
            ..Default::default()
        };
        let bridge = tokio::spawn(run_bridge_for_tests(bridge_args));
        Self { pg, memory, bridge, tenant }
    }
    pub async fn insert_chat_post(&self, channel_type: &str, body: &str) -> String {
        let post_id = uuid::Uuid::now_v7().to_string();
        let channel_id = self.ensure_channel(channel_type).await;
        let user_id    = self.ensure_user().await;
        self.pg.client().await.execute(
            "INSERT INTO posts(id, create_at, update_at, channel_id, user_id, message, props, file_ids)
                  VALUES ($1, $2, $2, $3, $4, $5, $6, $7)",
            &[&post_id, &now_ns(), &channel_id, &user_id, &body,
              &serde_json::json!({}), &serde_json::Value::Array(vec![])]
        ).await.unwrap();
        post_id
    }
    // ... ensure_channel, ensure_user, kill_memory_socket, restart_memory_socket, ...
}
```

### AC #3 — happy path

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac3_happy_message_to_memory() {
    let env = TestEnv::new().await;
    let post_id = env.insert_chat_post("O", "Hello, world").await;
    let row = env.memory.wait_for_kind_predicate("chat.message",
        |r| r["payload"]["post_id"] == post_id,
        Duration::from_secs(6)).await.unwrap();
    assert_eq!(row["payload"]["sync_class"], "shareable");
}
```

### AC #4 — message deleted

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac4_message_deleted() {
    let env = TestEnv::new().await;
    let post_id = env.insert_chat_post("O", "delete me").await;
    env.memory.wait_for_kind("chat.message", Duration::from_secs(6)).await.unwrap();
    env.update_post(&post_id, "delete_at", now_ns()).await;
    env.memory.wait_for_kind_predicate("chat.message_deleted",
        |r| r["payload"]["post_id"] == post_id,
        Duration::from_secs(6)).await.unwrap();
}
```

### AC #7/#8/#9 — sync_class derivation

```rust
#[rstest]
#[case("O", "shareable")]
#[case("P", "private")]
#[case("D", "private")]
#[case("G", "private")]
#[tokio::test(flavor = "multi_thread")]
async fn ac7_8_9_sync_class(#[case] channel_type: &str, #[case] expected: &str) {
    let env = TestEnv::new().await;
    env.insert_chat_post(channel_type, "x").await;
    let row = env.memory.wait_for_kind("chat.message", Duration::from_secs(6)).await.unwrap();
    assert_eq!(row["payload"]["sync_class"], expected);
}
```

### AC #10 — PII redaction

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac10_pii_redacted() {
    let env = TestEnv::new().await;
    env.insert_chat_post("O", "contact alice@cyberskill.world or 0901234567").await;
    let row = env.memory.wait_for_kind("chat.message", Duration::from_secs(6)).await.unwrap();
    let body = row["payload"]["body"].as_str().unwrap();
    assert!(!body.contains("alice@cyberskill.world"));
    assert!(!body.contains("0901234567"));
    assert!(body.contains("<EMAIL>"));
    assert!(body.contains("<PHONE>"));
}
```

### AC #14 — LSN pause on memory failure

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac14_lsn_not_advanced_on_memory_failure() {
    let env = TestEnv::new().await;
    env.memory.stop().await;
    let initial_lsn = env.bridge_consumed_lsn().await;
    env.insert_chat_post("O", "lost?").await;
    tokio::time::sleep(Duration::from_secs(3)).await;
    assert_eq!(env.bridge_consumed_lsn().await, initial_lsn,
        "LSN advanced while memory socket was down");
    env.memory.start().await;
    let row = env.memory.wait_for_kind("chat.message", Duration::from_secs(15)).await.unwrap();
    assert_eq!(row["payload"]["body"], "lost?");
}
```

### AC #19 — dedup_key determinism

```rust
#[test]
fn ac19_dedup_key_is_deterministic() {
    let tenant = uuid::uuid!("00000000-0000-0000-0000-000000000001");
    let event = ReplicationEvent::PostInsert(Post {
        id: "post-123".into(), update_at: 1747407137,
        ..Default::default()
    });
    let k1 = compute_dedup_key(tenant, &event);
    let k2 = compute_dedup_key(tenant, &event);
    assert_eq!(k1, k2);

    // Different version → different key
    let event2 = ReplicationEvent::PostInsert(Post {
        id: "post-123".into(), update_at: 1747407138,
        ..Default::default()
    });
    let k3 = compute_dedup_key(tenant, &event2);
    assert_ne!(k1, k3);
}
```

### AC #20 — LSN advance gated on contiguous acks

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac20_lsn_advance_gated_on_contiguous_acks() {
    let env = TestEnv::new().await;
    env.memory.set_ack_policy(AckPolicy::AckOnly(vec![/* second emit only */]));
    let p1 = env.insert_chat_post("O", "first").await;
    let p2 = env.insert_chat_post("O", "second").await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    let consumed = env.bridge_consumed_lsn().await;
    let lsn_of_p1 = env.lsn_of_post(&p1).await;
    assert_eq!(consumed, lsn_of_p1,
        "LSN advanced past unacked first message");
}
```

### AC #21 — chat.message_edited carries prior_dedup_key

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac21_message_edited_prior_dedup_key() {
    let env = TestEnv::new().await;
    let post_id = env.insert_chat_post("O", "v1").await;
    let row_v1 = env.memory.wait_for_kind("chat.message", Duration::from_secs(6)).await.unwrap();
    let dk1 = row_v1["payload"]["dedup_key"].as_str().unwrap().to_owned();

    env.update_post_message(&post_id, "v2").await;
    let row_edit = env.memory.wait_for_kind("chat.message_edited", Duration::from_secs(6)).await.unwrap();
    assert_eq!(row_edit["payload"]["prior_dedup_key"], dk1);
}
```

### AC #25 — read-replica refusal

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac25_read_replica_refused() {
    let env = TestEnv::new_with_replica().await;
    let result = bridge::main_with_args(bridge::Args {
        chat_dsn: env.replica_dsn(),
        ..env.default_args()
    }).await;
    assert!(result.is_err());
    let row = env.memory.wait_for_kind("chat.bridge_misconfigured",
        Duration::from_secs(5)).await.unwrap();
    assert_eq!(row["severity"], "SEV-1");
}
```

### AC #26 — heartbeat

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac26_heartbeat_every_30s() {
    let env = TestEnv::new().await;
    let t1 = env.memory.wait_for_kind("chat.bridge_heartbeat",
        Duration::from_secs(35)).await.unwrap();
    let t2 = env.memory.wait_for_kind_after(t1.ts_ns, "chat.bridge_heartbeat",
        Duration::from_secs(35)).await.unwrap();
    let delta_ms = (t2.ts_ns - t1.ts_ns) / 1_000_000;
    assert!((28_000..32_000).contains(&delta_ms),
        "heartbeat cadence drift: {}ms", delta_ms);
}
```

### AC #27 — SIGTERM drain

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac27_sigterm_drains_in_10s() {
    let env = TestEnv::new().await;
    for i in 0..100 {
        env.insert_chat_post("O", &format!("msg-{}", i)).await;
    }
    env.send_sigterm().await;
    env.wait_for_exit(Duration::from_secs(12)).await.unwrap();
    let rows = env.memory.collected_rows_of_kind("chat.message").await;
    assert_eq!(rows.len(), 100, "drained {} of 100 messages", rows.len());
    let shutdown = env.memory.collected_rows_of_kind("chat.bridge_shutdown").await;
    assert_eq!(shutdown.len(), 1);
}
```

### AC #28 — `@channel` doesn't trigger email regex

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac28_mention_specials_normalised_before_pii() {
    let env = TestEnv::new().await;
    env.insert_chat_post("O", "@channel — please email alice@x.com").await;
    let row = env.memory.wait_for_kind("chat.message", Duration::from_secs(6)).await.unwrap();
    let body = row["payload"]["body"].as_str().unwrap();
    assert!(body.contains("[mention_channel]"), "got: {}", body);
    assert!(body.contains("<EMAIL>"));
    assert!(!body.contains("@channel"));
}
```

### AC #31 — Sessions table not in publication

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac31_sessions_not_in_publication() {
    let env = TestEnv::new().await;
    env.pg.client().await.execute(
        "INSERT INTO Sessions(id, token, user_id, create_at)
              VALUES ('s1', 'tok', 'u1', $1)",
        &[&now_ns()]).await.unwrap();
    let result = env.memory.wait_for_any(Duration::from_secs(5)).await;
    assert!(matches!(result, Err(_)) || result.unwrap().kind != "chat.session_started",
        "bridge leaked Sessions table to memory");
}
```

### AC #32 — duplicate emit collapses via dedup_key

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac32_crash_mid_advance_dedups_on_restart() {
    let env = TestEnv::new().await;
    let post_id = env.insert_chat_post("O", "exactly-once").await;
    let row1 = env.memory.wait_for_kind("chat.message", Duration::from_secs(6)).await.unwrap();
    let dk = row1["payload"]["dedup_key"].as_str().unwrap().to_owned();

    env.kill_bridge_after_ack_before_advance().await;
    env.restart_bridge().await;

    let row2 = env.memory.wait_for_kind_after(row1.ts_ns, "chat.message",
        Duration::from_secs(10)).await.unwrap();
    assert_eq!(row2["payload"]["dedup_key"], dk,
        "dedup_key non-deterministic across restart");
    // TASK-MEMORY-107 will dedup these on the read side; we just verify the key matches.
}
```

### Scope isolation script — AC #35

```python
#!/usr/bin/env python3
# tests/scope-isolation.py
import psycopg2, os
conn = psycopg2.connect(os.environ['CHAT_DSN'])
cur = conn.cursor()
cur.execute("""
    SELECT relname FROM pg_stat_user_tables
    WHERE schemaname='public' AND relname NOT IN
        ('posts','channels','channelmembers','reactions','fileinfo')
        AND (select sum(seq_scan)+sum(idx_scan)
             from pg_stat_user_tables s
             where s.relname=pg_stat_user_tables.relname) > 0
""")
unauthorised = [r[0] for r in cur.fetchall()]
assert not unauthorised, f"bridge read tables outside scope: {unauthorised}"
print("PASS: bridge stayed within publication scope")
```

### Slot bloat test — AC #36

```rust
#[tokio::test(flavor = "multi_thread")]
#[ignore = "long-running"] // 1h test; runs in nightly
async fn ac36_idle_slot_no_wal_bloat() {
    let env = TestEnv::new().await;
    let initial = env.pg_wal_size_bytes().await;
    tokio::time::sleep(Duration::from_secs(3600)).await;
    let after = env.pg_wal_size_bytes().await;
    let delta = after.saturating_sub(initial);
    assert!(delta < 100_000_000, "WAL grew by {} bytes in idle hour", delta);
}
```

---

## §6 — Implementation skeleton

The Rust modules above are the skeleton. This section names the operational wiring decisions that are not local to any single file:

### §6.1 — Process model per tenant

Each tenant runs ONE bridge process. The process is a Fargate task launched by TASK-CHAT-003's ECS service definition (a second task in the same task definition as the Mattermost server, so they scale together). The bridge depends on the chat DB but is independent of the Mattermost process — Mattermost can be killed/restarted without losing replication progress, because the slot LSN lives in Postgres, not in Mattermost state.

### §6.2 — Slot naming convention

`cyberos_bridge_<tenant_simple>` where `<tenant_simple>` is the UUID-v7 with hyphens stripped (32 hex chars). Postgres allows up to 63-byte identifiers, so the full name fits. Two bridges with the same tenant_id MUST NOT run concurrently — they would race on slot ownership. The orchestrator (TASK-CHAT-003 ECS) enforces this via `desired_count=1` on the bridge service.

### §6.3 — Reconnect backoff

Exponential, starting at 500ms, doubling, capped at 30s (configurable via `--reconnect-max-backoff-secs`). Reset to 500ms on a clean reconnect (one full minute of no error). The reset rule prevents fast-flap from staying at max backoff forever.

Reasons classified for the `chat_memory_bridge_reconnects_total` counter:
- `pg_disconnect` — Postgres connection lost.
- `memory_disconnect` — memory socket lost.
- `slot_lost` — slot dropped by external operator.
- `parse_error` — pgoutput message couldn't be parsed (schema drift).
- `auth_error` — replication user credentials invalid.
- `unknown` — fallback.

### §6.4 — Standby status messages

Every 10s the bridge sends a `StandbyStatusUpdate` message to Postgres carrying the most-recently-confirmed LSN (`highest_committed_lsn`). This is the primary mechanism for slot advancement under sustained traffic; explicit `pg_replication_slot_advance` is for catch-up after a stall.

If Postgres requests an immediate reply (PrimaryKeepAlive with `reply=1`), the bridge sends the standby status immediately rather than waiting for the 10s tick. This is important because Postgres can request a reply when it's near its `max_slot_wal_keep_size` and needs the slot to advance.

### §6.5 — memory client interface

The bridge consumes `cyberos-memory-client::emit_with_ack(row, ack_tx)` (a workspace crate, owned by TASK-MEMORY-107). The client serialises the row, frames it per TASK-MEMORY-107 wire format, sends, and resolves the `ack_tx` when the memory writer acks. Ack timeout is 5s; on timeout the ack resolves `false` (which causes the bridge to stop advancing the LSN past that emit).

### §6.6 — PII allowlist propagation

Task-AGENTS.md §3.6 allows tenant-scoped PII allowlists. The bridge reads `<memory-root>/manifest.json::tenants[tenant_id].pii_allowlist` at startup and passes it to `cyberos_memory_pii::scan_and_redact(body, &allowlist)`. Reload on SIGHUP (or memory-side hot-reload event).

### §6.7 — Init publication command

`cyberos-chat-bridge --init-publication --chat-dsn <dsn>` runs `init-bridge-publication.sql` and exits. Invoked once by TASK-CHAT-003 Terraform via `null_resource` after RDS is `available`. Idempotent (DROP IF EXISTS + CREATE).

### §6.8 — Schema drift handling

When Mattermost adds a column to `posts`, the pgoutput protocol delivers it but the bridge's `Post` struct doesn't know the field. Two cases:
1. **Additive change (new nullable column):** bridge ignores the field; no behaviour change. Tracking: nightly CI runs `tests/schema-drift.sh` that compares Mattermost's migration output against the bridge's struct definition and emits a SEV-3 audit if drift is detected.
2. **Breaking change (rename, type change):** bridge fails at decode_pgoutput; reconnect counter increments with `reason=parse_error`; SEV-1 alarm fires.

The fix path is: ship a bridge image with updated structs, redeploy. The replication slot retains LSN, so no message loss across the upgrade.

### §6.9 — Drain semantics

`drain_pending(map, timeout)` waits for every entry in the `pending_acks` BTreeMap to resolve, up to the timeout. If timeout expires with pending entries:
1. Persist the highest contiguous-acked LSN.
2. Emit `chat.bridge_shutdown` with `pending_count: N`.
3. Exit 0 (graceful) anyway — restart will replay the unacked messages, dedup_key collapses duplicates.

We don't exit 1 because Fargate would mark the task as failed and the SCALE-IN would treat it as a crash, blocking the scale event. The clean exit lets the SCALE proceed.

### §6.10 — Heartbeat semantics

Heartbeat emits regardless of replication activity. The payload always carries:
- `lsn_consumed`: highest_committed_lsn
- `lsn_max`: `pg_current_wal_lsn()`
- `lag_bytes`: `pg_wal_lsn_diff(lsn_max, lsn_consumed)`
- `lag_seconds`: time since most recent emitted message
- `slot_active`: `pg_replication_slots.active`
- `bridge_uptime_seconds`
- `pending_ack_count`
- `pii_redaction_failures_in_last_30s`

Operators query `chat.bridge_heartbeat` rows to validate liveness without OBS dashboard access.

### §6.11 — Cross-tenant isolation enforcement

The bridge process is launched with `CYBEROS_TENANT_ID=<uuid>` env var. The replication slot is named with that UUID. The memory client connects with a per-tenant credential. There are three layers of enforcement:
1. **Slot scope:** each slot is per-tenant; one bridge consumes one slot.
2. **Publication scope:** the publication `chat_bridge` is one per tenant DB; tenants have separate DBs.
3. **memory write scope:** the memory client refuses to emit a row whose `tenant_id` doesn't match its credential's tenant.

Defense-in-depth: any one layer suffices, but all three are checked.

---

## §7 — Dependencies

- **TASK-CHAT-003** — RDS host for the publication.
- **TASK-MEMORY-107** — capture daemon socket consumer.
- **TASK-MEMORY-111** — PII redaction.
- **TASK-OBS-007** — alarm routing.

---

## §8 — Example payloads

### `chat.message` — happy path

```json
{
  "kind": "chat.message",
  "ts_ns": 1747407137485000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "dedup_key": "f1a2b3c4d5e6f7081920304050607080a1b2c3d4e5f6708192030405060708ab",
  "payload": {
    "post_id":      "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "channel_id":   "01HVQX8ZG2K3R4TVA7P3WV5X8M",
    "user_id":      "01HVQX8ZG2K3R4TVA7P3WV5X8K",
    "body":         "Có ai cần <EMAIL> giúp xử lý không?",
    "sync_class":   "private",
    "create_at_ns": 1747407137483000000,
    "has_attachments": false
  }
}
```

### `chat.message_edited`

```json
{
  "kind": "chat.message_edited",
  "ts_ns": 1747407197485000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "trace_id": "1b8c4d6e1e0c4d7a9d2aad8c4b6bbf22",
  "dedup_key": "a2b3c4d5e6f70819203040506070809a1b2c3d4e5f6708192030405060708abf",
  "payload": {
    "post_id":         "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "channel_id":      "01HVQX8ZG2K3R4TVA7P3WV5X8M",
    "user_id":         "01HVQX8ZG2K3R4TVA7P3WV5X8K",
    "body":            "Có ai cần <EMAIL> hỗ trợ xử lý không?",
    "sync_class":      "private",
    "prior_dedup_key": "f1a2b3c4d5e6f7081920304050607080a1b2c3d4e5f6708192030405060708ab",
    "edit_at_ns":      1747407197480000000
  }
}
```

### `chat.message_deleted`

```json
{
  "kind": "chat.message_deleted",
  "ts_ns": 1747407217485000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "dedup_key": "b3c4d5e6f7081920304050607080a1b2c3d4e5f6708192030405060708abf12c",
  "payload": {
    "post_id":       "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "channel_id":    "01HVQX8ZG2K3R4TVA7P3WV5X8M",
    "deleted_at_ns": 1747407217480000000
  }
}
```

### `chat.channel_created`

```json
{
  "kind": "chat.channel_created",
  "ts_ns": 1747407100000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "channel_id":   "01HVQX8ZG2K3R4TVA7P3WV5X8M",
    "channel_type": "O",
    "team_id":      "01HVQX8ZG2K3R4TVA7P3WV5X8J",
    "display_name": "general"
  }
}
```

### `chat.message_reacted`

```json
{
  "kind": "chat.message_reacted",
  "ts_ns": 1747407140000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "post_id":     "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "user_id":     "01HVQX8ZG2K3R4TVA7P3WV5X8P",
    "emoji_name":  "thumbsup",
    "create_at_ns": 1747407139999000000
  }
}
```

### `chat.user_joined_channel`

```json
{
  "kind": "chat.user_joined_channel",
  "ts_ns": 1747407050000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "channel_id":   "01HVQX8ZG2K3R4TVA7P3WV5X8M",
    "user_id":      "01HVQX8ZG2K3R4TVA7P3WV5X8P",
    "joined_at_ns": 1747407049000000000
  }
}
```

### `chat.message_attachment`

```json
{
  "kind": "chat.message_attachment",
  "ts_ns": 1747407137600000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "post_id":    "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "file_id":    "01HVQX8ZG2K3R4TVA7P3WV5X8Q",
    "filename":   "Resume - <NAME>.pdf",
    "mime":       "application/pdf",
    "size_bytes": 184232,
    "sync_class": "private"
  }
}
```

### `chat.bridge_heartbeat`

```json
{
  "kind": "chat.bridge_heartbeat",
  "ts_ns": 1747407170000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "lsn_consumed":     "1/3A8BC120",
    "lsn_max":          "1/3A8BC1F0",
    "lag_bytes":        208,
    "lag_seconds":      0.42,
    "slot_active":      true,
    "bridge_uptime_seconds": 8327,
    "pending_ack_count": 2,
    "pii_redaction_failures_in_last_30s": 0
  }
}
```

### `chat.bridge_lag_alert`

```json
{
  "kind": "chat.bridge_lag_alert",
  "ts_ns": 1747407300000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "severity": "SEV-2",
  "payload": {
    "p95_latency_seconds": 12.4,
    "sustained_seconds":   62,
    "lag_bytes":           48238,
    "lag_seconds":         12.4,
    "consumed_lsn":        "1/3A8BC120"
  }
}
```

### `chat.bridge_misconfigured`

```json
{
  "kind": "chat.bridge_misconfigured",
  "ts_ns": 1747407100000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "severity": "SEV-1",
  "payload": {
    "reason":      "connected to read replica; logical replication unavailable",
    "remediation": "point CYBEROS_CHAT_DSN at the primary endpoint"
  }
}
```

### `chat.bridge_shutdown`

```json
{
  "kind": "chat.bridge_shutdown",
  "ts_ns": 1747407400000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "final_lsn":         "1/3A8BC1F0",
    "pending_count":     0,
    "drain_duration_ms": 4283,
    "exit_reason":       "sigterm"
  }
}
```

### `chat.bridge_redaction_failed`

```json
{
  "kind": "chat.bridge_redaction_failed",
  "ts_ns": 1747407220000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "severity": "SEV-2",
  "payload": {
    "post_id":   "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "error":     "regex compilation failed for rule pii.vn-phone-v2",
    "outcome":   "row_dropped"
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Backfill of historical messages on bridge first-start — slice 4+; assumes greenfield.
- Cross-region replication slot redundancy — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| RDS slot already exists | error on CREATE | bridge reuses existing slot at its current LSN | None |
| Slot lag too high (close to `max_slot_wal_keep_size`) | WAL retention pressure metric | SEV-1 alarm; Postgres may drop slot | Scale RDS storage; investigate why bridge stalled |
| memory socket down | emit Err | LSN paused; pending_acks grows; SEV-2 after 60s | Operator restores memory; bridge auto-resumes |
| memory socket slow (ack > 5s) | ack timeout | LSN paused; pending_acks grows | Investigate memory writer load; consider TASK-MEMORY-107 scale-up |
| PII scan failure (catch_unwind) | redact fn returns Err | Message dropped + `chat.bridge_redaction_failed` audit; SEV-2 | Operator fixes ruleset; consider replay via `force-replay --from-lsn` |
| PII scan timeout | timeout wrap | Message dropped + audit; SEV-2 | Investigate ruleset perf regression |
| Schema drift (chat upgrade adds column) | bridge ignores unknown field; nightly CI detects | SEV-3 audit | Update bridge struct; redeploy |
| Schema drift (chat upgrade renames column) | pgoutput parse Err | Reconnect loop; SEV-1 alarm | Ship updated bridge image |
| Duplicate emit | bridge crash mid-LSN-advance | dedup_key identical; TASK-MEMORY-107 dedups | None |
| Tenant_id wrong | startup config | bridge starts but emits with wrong tenant | memory client refuses (defense-in-depth); SEV-1 |
| Message > Mattermost-max (16KB) | n/a; Mattermost rejects at insert | Never reaches bridge | None |
| Channel privacy flip mid-replication | derive at emit time per message | New messages get new sync_class; old retained as-is | None |
| Bridge crash mid-LSN-advance | restart resumes from last confirmed | Duplicate emit collapsed by dedup | None |
| WAL retention exhausted (`max_slot_wal_keep_size`) | RDS drops slot | bridge restart fails; SEV-1 | Operator: bump retention; recreate slot; full replay |
| Replication user permissions wrong | startup query fails | bridge exit non-zero; SEV-1 | Operator runs init-bridge-publication.sql |
| Replication user account locked | auth failure | bridge in reconnect loop | Operator unlocks via Secrets Manager rotation |
| Connected to read replica | replica_safety_check fires | startup fails; SEV-1 `chat.bridge_misconfigured` | Operator fixes DSN |
| pgoutput protocol-version mismatch | parse Err with version code | startup fails; SEV-1 | Coordinate with TASK-MEMORY-107 schema update |
| Concurrent bridges with same tenant_id | second `CREATE_REPLICATION_SLOT` errors | second bridge exits with `slot already active` | ECS desired_count=1 prevents in normal ops |
| Disk-pressure SIGKILL on Fargate | hard kill; no drain | up to one in-flight message; dedup handles | None — automatic |
| Replication slot orphaned (bridge gone, slot stays) | `pg_replication_slots.active=false` for >1h | WAL retention pressure builds | Operator: `SELECT pg_drop_replication_slot('cyberos_bridge_<X>')` |
| RDS minor version upgrade (in-place) | brief connection drop | bridge reconnects within 30s | None |
| RDS major version upgrade | logical slot incompatible | bridge fails at startup | Coordinate slot recreation + full replay |
| Mattermost upgrade adds Sessions-table-related column | publication explicit; ignored | None | None |
| Mattermost upgrade adds a new table consumed by TASK-CHAT-008 | not in current publication | new events not emitted until publication ALTER | Operator runs ALTER PUBLICATION + bridge restart |
| Reactions burst (1000/sec from automation) | bridge keeps up via pipelining | None visible | None |
| Network partition between Fargate task and RDS | connection error | reconnect loop; LSN frozen | Heals automatically |
| Network partition Fargate ↔ memory socket | emit Err | LSN paused; SEV-2 after 60s | Heals automatically |
| Memory leak in tokio-postgres | task OOM kill | Fargate restart | Investigate via heap snapshot; pin tokio-postgres version |
| Heartbeat emit fails | logged warn; metric increment | Next heartbeat retries; not a data-loss event | None |
| Standby status update fails | logged warn | Postgres marks slot lagging; SEV-2 if persistent | Investigate Postgres → Fargate network |
| `pg_replication_slot_advance` race with concurrent writer | error on advance | retry on next loop iteration | None — Postgres serialises |
| Mattermost migration drops a table in the publication | pgoutput emits `Relation` message for unknown OID | parse Err; SEV-1 | Coordinate migration with bridge update |
| Bridge image has wrong arch (amd64 on Graviton) | ECS task exit 139 | crash loop | Operator fixes image tag |
| OTel collector down | metrics buffer | bridge keeps running | Investigate collector |
| Heartbeat row schema rejected by memory (TASK-AI-003 closed set drift) | memory 422 | heartbeat lost; logged warn | Coordinate with TASK-AI-003 to allow `chat.bridge_heartbeat` |
| `pg_replication_slot_advance` not available (Postgres < 11) | function not found at startup | fail; SEV-1 | Upgrade RDS to PG16 |
| Bridge clock skewed > 30s from RDS | latency histogram wrong | None visible to users | Sync clock via ECS task time-sync |
| Mattermost write rate exceeds bridge throughput | lag grows | bridge_lag_alert; SEV-2 | Investigate; possibly partition by team_id (slice 4+) |

---

## §11 — Implementation notes

- `postgres-protocol` (not `pgreplication`) is the canonical pgoutput parser; bundled with `tokio-postgres`. Lower-level than `pgreplication` but maintained alongside the connection lib so version-skew issues are bounded.
- LSN persistence uses the slot itself (`pg_replication_slot_advance`) plus standby status messages. We don't write LSN to a sidecar table because the slot IS the durable LSN store; duplicating it invites drift.
- Bridge runs as separate Fargate task per tenant; ECS launches via TASK-CHAT-003 module. Co-located with the chat server in the same task definition so they scale together (load on chat = load on bridge).
- The 5-second SLA is end-to-end (chat insert → memory row); typical observed ~500ms. The budget breaks down as: Postgres WAL write (1ms) + bridge pgoutput decode (5ms) + PII redact (30ms) + memory socket emit + ack (~100ms typical) = ~140ms median. The 5s headroom absorbs occasional memory GC pauses and PII-rule cold starts.
- Bridge_lag_alert dedup'd by TASK-OBS-007 (no spam during sustained outage). TASK-OBS-007 expects the `chat.bridge_lag_alert` row's payload to carry `sustained_seconds`; this gates re-alarming.
- Channel type codes: `O`=Open, `P`=Private, `D`=Direct, `G`=Group — Mattermost convention. We map `O` to `shareable` and the others to `private`; the unknown-type fail-secure path is `private` (defense against future channel types Mattermost may add).
- `dedup_key = sha256(tenant_id || post_id || version)` where `version` is `update_at`. This means a post and its edit produce different keys, which is intentional — the edit is a new event to TASK-MEMORY-107.
- The `pending_acks: BTreeMap<u64, oneshot::Receiver<bool>>` is bounded implicitly by memory's ack rate. If memory slows to a crawl, the map grows; at 100K entries memory pressure becomes notable (~16MB). We don't set a hard cap because dropping pending entries would lose data; instead, the bridge_lag_alert fires long before memory becomes an issue.
- We chose `tokio-postgres::copy_both_simple` over `tokio-postgres::replication_client` (a higher-level wrapper) because the wrapper's error handling doesn't expose the LSN of the failed message, which we need for pending_acks tracking.
- The `standby_status_update` cadence (10s) trades responsiveness against Postgres write overhead. Faster = lower lag visibility on Postgres side; slower = less WAL accumulation. 10s is the Postgres-docs-recommended default.
- We don't use `wal2json` even though TASK-CHAT-003 preloads it, because `pgoutput` is more efficient (binary protocol vs JSON parsing); `wal2json` is preloaded for other downstream consumers (slice 4+ analytics pipeline).
- The `normalise_mattermost_specials` function runs as a string-replace pass, not a tokeniser. We could've used a proper parser but the savings are small and the regex overhead is < 100µs per message.
- Per-tenant Fargate task per bridge is more expensive than a multi-tenant bridge process, but the operational simplicity (one tenant = one task = one log group) outweighs the cost (~$0.50/mo per tenant for the always-on task). At ≥ 50 tenants we'd revisit; not yet.
- The `replica_safety_check` is the only startup-time fast-fail. We considered checking `wal_level` too, but RDS enforces it via parameter group, and the bridge will fail at `START_REPLICATION` anyway with a clear error.
- `pg_drop_replication_slot` is NEVER called by the bridge — only by operators. Dropping a slot loses LSN progress, which would cause full replay. Operators must coordinate via runbook.
- The `--init-publication` mode runs the SQL once and exits. We chose this over auto-init-at-startup because we want explicit operator awareness of the publication's CREATE (it affects Postgres write performance slightly).
- Heartbeat row size is ~400 bytes per row, 2 rows/min, ~50KB/day per tenant. Negligible storage cost at 100 tenants.
- Why we attach a `severity` field to misconfiguration / lag / shutdown / redaction-failed rows but not to data rows: severity is for operator routing, not consumer semantics. Data rows don't need it; observability rows do.
- The `metrics-exporter-prometheus` is paired with the Cloudwatch agent in TASK-CHAT-003 obs sidecar; metrics live in CloudWatch namespace `CyberOS/Chat/Bridge`.
- We chose `uuid::Uuid::now_v7()` (time-ordered UUIDs) over `v4` for fake-memory socket test IDs so test failure logs are chronologically sortable.
- The 30s heartbeat cadence is calibrated against the 5min sev-2 lag-alarm window: 10 heartbeats fit in the window, so operators get progressive visibility into a developing lag, not just a binary fired/not-fired signal.
- Why `chat.bridge_lag_alert` is emitted by the BRIDGE not OBS: OBS knows the metric but not the LSN values; lag_alert payload requires `consumed_lsn` which is bridge-internal state. Emitting from the bridge keeps the payload self-sufficient.
- Why we emit `chat.bridge_shutdown` instead of just letting the process exit: operators investigating "did the bridge crash or shut down cleanly?" need a memory-side signal. Heartbeat absence is ambiguous (could be network); explicit shutdown row removes ambiguity.
- The cross-tenant isolation defense-in-depth (§6.11) is critical because a misconfigured Fargate task that's pointed at the wrong tenant's DB would have all the credentials needed to emit cross-tenant data without any layer rejecting. Three layers, each owned by a different code path, makes any one bug recoverable.
- `dedup_key` lives in the row body, not as a top-level field, because TASK-MEMORY-107's wire protocol carries it in the payload envelope; the TASK-MEMORY-107 dedup index then promotes it to a top-level filter.
- The `force-replay` runbook (`cyberos-chat-bridge force-replay --from-lsn <lsn>`) is the recovery path for PII redaction-rule corrections: the bridge replays a range of LSNs, producing rows with the same dedup_keys; TASK-MEMORY-107's dedup will collapse to the most recent (replay-replaces-original semantic is owned by TASK-MEMORY-107).
- We don't use SAVEPOINT-style transactions across the WAL stream because Postgres logical replication is intrinsically streamed, not transactional from the consumer's perspective. The exactly-once-modulo-dedup guarantee is established at the LSN-advance boundary.

---

---

*End of TASK-CHAT-005.*
