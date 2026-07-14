---
# ───── Machine-readable frontmatter ─────
id: TASK-AI-004
title: "Cost-hold expiry cleanup job — refund unsettled holds + emit audit"
module: AI
priority: MUST
status: done
accepted_at: 2026-05-15
accepted_by: Stephen Cheng
verify: T
phase: P0
milestone: P0 · slice 1
slice: 1
owner: Stephen Cheng
created: 2026-05-15
shipped: null
memory_chain_hash: null
related_tasks: [TASK-AI-001, TASK-AI-002, TASK-AI-003]
depends_on: [TASK-AI-001, TASK-AI-003]
blocks: [TASK-AI-021]

# ───── Source contracts ─────
source_pages:
  - website/docs/modules/ai.html#cost-gate
source_decisions:
  - docs/tasks/ai/TASK-AI-001-cost-ledger-precheck/spec.md §3 (cost_ledger_hold TTL = 60s)
  - docs/tasks/ai/TASK-AI-002-cost-ledger-postcall-reconcile/spec.md §3 (state machine: held → expired)
  - archive/2026-05-14/AUDIT_AND_PLAN.md §3.3 (P0 · slice 1 build placement)

# ───── Build envelope ─────
language: rust 1.81
service: cyberos/services/ai-gateway/
new_files:
  - services/ai-gateway/src/bin/cost_hold_expiry.rs    # standalone tokio binary
  - services/ai-gateway/src/cost_hold_expiry.rs        # library code (testable)
  - services/ai-gateway/tests/cost_hold_expiry_test.rs
  - deploy/systemd/cyberos-ai-gateway-expiry.service   # systemd unit
modified_files:
  - services/ai-gateway/Cargo.toml   # add binary target
allowed_tools:
  - file_read: services/ai-gateway/**
  - file_write: services/ai-gateway/{src,tests}/**
  - file_write: deploy/systemd/**
  - bash: cargo test -p cyberos-ai-gateway cost_hold_expiry
  - bash: cargo build --bin cost_hold_expiry --release
disallowed_tools:
  - in-place edit of cost_ledger_hold schema (TASK-AI-001/002 own it)
  - bypass memory_writer when emitting ai.hold_expired rows
  - touch services/ai-gateway/src/cost_ledger.rs (this FR adds a sibling, not a fork)

# ───── Estimated work ─────
effort_hours: 3
subtasks:
  - "0.5h: scan_expired_holds() query with FOR UPDATE SKIP LOCKED"
  - "0.5h: per-hold expiry transition (held → expired) inside transaction"
  - "0.5h: memory ai.hold_expired audit row emission via memory_writer (TASK-AI-003)"
  - "0.5h: standalone tokio binary loop (every 30s, with graceful SIGTERM)"
  - "0.5h: systemd unit + Cargo.toml binary target"
  - "0.5h: integration test (seed N expired holds, run one tick, verify state)"
risk_if_skipped: "Expired holds remain in 'held' state forever. The hold count grows monotonically (one per call). The cost_ledger_hold table eventually slows down precheck (index scan over millions of stale rows). The chain has no audit record that a tenant's call timed out without ever reconciling — a compliance gap (no evidence the gateway exhausted its cost gate)."
---

## §1 — Description (BCP-14 normative)

The AI Gateway service **MUST** ship a standalone Rust binary `cost_hold_expiry` that runs as a long-lived process and, on a 30-second tick, scans `cost_ledger_hold` for rows whose `state = 'held' AND expires_at < NOW()` and transitions each to `state = 'expired'` with a chained memory audit row per row.

The cleanup tick:

1. **MUST** select expired holds with `FOR UPDATE SKIP LOCKED LIMIT 500` — bounded per tick to prevent a million-row spike from monopolising the database.
2. **MUST** process each hold inside its own Postgres transaction; one hold's failure MUST NOT roll back the others. Per-hold atomicity replaces per-batch atomicity.
3. **MUST** emit one `ai.hold_expired` memory audit row per expired hold via `memory_writer::emit()` (TASK-AI-003) before committing the row's state transition.
4. **MUST** leave `cost_ledger.spent_usd` UNCHANGED — an expired hold is functionally a refund (the call never settled, so no value was delivered to the tenant).
5. **MUST** sleep 30 seconds between ticks via `tokio::time::sleep`; the tick interval is configurable via the `CYBEROS_AI_EXPIRY_TICK_SECONDS` env var (default 30, min 5, max 300).
6. **MUST** handle SIGTERM gracefully — finish the current tick (or current hold inside the tick), emit a `process_shutdown` log line, exit 0 within 5 seconds of receiving SIGTERM.
7. **MUST** handle Postgres connection drops by reconnecting with exponential backoff (1s, 2s, 4s, 8s, capped at 30s). Connection failures MUST NOT crash the binary; the process is expected to outlive transient DB outages.
8. **MUST** emit OTel metrics: `ai_expiry_holds_processed_total` (counter), `ai_expiry_tick_duration_seconds` (histogram), `ai_expiry_consecutive_failures_total` (counter, resets on success).
9. **SHOULD** emit a sev-2 OBS alert when `ai_expiry_consecutive_failures_total ≥ 10` (5 minutes of total failure). TASK-OBS-007 routes this when it lands; until then, structured `tracing::error!` lines suffice.
10. **MUST** be idempotent under crash — if the binary crashes mid-tick after emitting some memory rows but before committing some hold transitions, the next tick MUST re-scan and either complete the transitions or surface a chain inconsistency. The `FOR UPDATE SKIP LOCKED` semantics ensure no double-emission *within a single binary instance*; cross-restart crash recovery is the slice-1 documented limitation per AC #9.
11. **MUST** allocate one `tick_id` (ULID-26) per tick at tick start. Every `ai.hold_expired` audit row emitted during the tick MUST carry `extra.tick_id`. This enables OBS correlation: "show all expirations from tick X".
12. **MUST** emit OTel metrics on every tick: `ai_expiry_ticks_total` (counter), `ai_expiry_tick_duration_seconds` (histogram), `ai_expiry_holds_processed_total` (counter), `ai_expiry_holds_succeeded_total` (counter), `ai_expiry_holds_failed_total` (counter), `ai_expiry_consecutive_failures` (gauge; resets to 0 on success).
13. **MUST NOT** implement leader election for multi-instance deployments. One instance is sufficient at any P0/P1 scale. If two are deployed by accident, Postgres `FOR UPDATE SKIP LOCKED` keeps them safe — one will simply do no work per tick. TASK-AI-021 (operator CLI) surfaces a warning if `ps -ef \| grep cost_hold_expiry \| wc -l > 1`.
14. **MUST** emit OBS metrics enumerated above (#12) on every tick AND publish the per-tick `cleanup_holds_pending_gauge` as a Prometheus gauge that operators can dashboard. Long-lived `pending` backlog (>30 minutes) is the canary signal for a stuck cleanup loop or a Postgres lock conflict.
15. **MUST** emit `ai.hold_expired_started` BEFORE applying `UPDATE cost_ledger SET state='expired'` for each batched hold per task-audit skill §3.8 rule 25 (audit-before-action). The Postgres transaction wraps both the memory-emit AND the UPDATE in one atomic unit; rollback on either failure. `ai.hold_expired_completed` follows post-commit per task-audit skill §3.8 rule 26 (pair-write). The OBS lint flags any standalone `ai.hold_expired_started` over a 5-minute window. AC #15 added with captured-events ordering test.
16. **MUST** order the per-sweep `SELECT ... FOR UPDATE SKIP LOCKED LIMIT 1000` by `ORDER BY hold_id ASC` per task-audit skill §3.9 rule 27 (determinism), so two runs on the same backlog produce byte-identical expire-order. The per-sweep `ai.cleanup_run_completed` audit row's `extra.expired_hold_ids: Vec<Uuid>` MUST be recorded in the same sorted order; operator diffing across runs is reliable. AC #16 added with deterministic-order test.

A single instance of this binary services the AI Gateway deployment. Running multiple instances is safe (Postgres row-level locking ensures correctness) but provides no throughput benefit at slice-1 scale; TASK-AI-021 (operator CLI) adds the multi-instance leader-election story if it's ever needed.

---

## §2 — Why this design (rationale for humans)

**Why a separate binary and not a tokio task inside the main gateway service?** Two reasons. (1) Lifecycle isolation: if the cleanup job hangs (e.g., a single bad row blocks the whole binary), the gateway's request path keeps serving. The opposite is also true — gateway crashes don't take down cleanup. (2) Operational visibility: a separate systemd unit gives ops a clean signal ("cleanup is up", "cleanup is down") rather than burying the status behind a gateway health-check.

**Why 30 seconds tick interval?** Holds expire after 60s (TASK-AI-001's TTL). A 30s tick guarantees an expired hold gets processed within at most 90s of its original creation (60s TTL + 30s tick lag). Tighter intervals (5–15s) just burn CPU; slacker intervals (60–300s) let the hold table grow under traffic spikes. 30s is the goldilocks zone for slice-1 traffic shapes.

**Why `FOR UPDATE SKIP LOCKED`?** Two pieces of value. (1) `FOR UPDATE` prevents a second cleanup instance (or a stray re-reconcile call from the main gateway) from racing to transition the same row. (2) `SKIP LOCKED` means cleanup never blocks on a hold currently being reconciled by the main gateway — if a row is locked elsewhere, just skip it; it'll be processed in 30s. The default `FOR UPDATE` (without SKIP LOCKED) would queue cleanup behind every in-flight reconcile and lengthen ticks under load.

**Why per-hold transactions and not per-batch?** Per-batch atomicity feels safer ("all 500 or none"), but it amplifies blast radius — one bad row (e.g., a hold pointing to a deleted tenant) rolls back all 500. Per-hold atomicity isolates the failure. The cost is N transaction commits per tick instead of 1, but with `synchronous_commit = on` and a local Postgres, that's ~5ms × 500 = 2.5s of commit overhead — well inside the 30s tick budget.

**Why emit `ai.hold_expired` and not just delete the row?** Three reasons. (1) Compliance: the EU AI Act Art. 12 audit trail wants evidence of every cost-gate decision, including the ones where the gate held money that never got spent. (2) Debugging: when a tenant reports "I got charged for a call that never returned", the `ai.hold_expired` row is the canonical proof that we *didn't* charge them. (3) memory's audit-before-action invariant says state transitions must hit the chain; this is no exception.

**Why is `cost_ledger.spent_usd` unchanged on expiry?** An expired hold means the provider call never completed (or completed so late that the reconcile path was abandoned). The tenant got no value. Charging the budget would be a soft fraud. The few cases where the provider *did* complete but the gateway lost track are explicit data-quality bugs to fix in the gateway, not silent leaks to swallow.

---

## §3 — API contract

### Library function signature (testable)

```rust
// services/ai-gateway/src/cost_hold_expiry.rs

pub async fn run_tick(pool: &PgPool) -> Result<TickReport, TickError>;

pub struct TickReport {
    pub holds_processed: u32,
    pub holds_succeeded: u32,
    pub holds_failed: u32,
    pub duration_ms: u32,
}

pub enum TickError {
    DbError(sqlx::Error),
    // Note: per-hold failures roll up into TickReport.holds_failed; TickError is reserved
    // for tick-level failures (DB unreachable, query syntax error, etc.)
}
```

### Binary entrypoint

```rust
// services/ai-gateway/src/bin/cost_hold_expiry.rs

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let tick_seconds = env::var("CYBEROS_AI_EXPIRY_TICK_SECONDS")
        .unwrap_or_else(|_| "30".into())
        .parse::<u64>()?
        .clamp(5, 300);

    let pool = build_pool().await?;
    let mut shutdown = signal::unix::signal(SignalKind::terminate())?;
    let mut tick = tokio::time::interval(Duration::from_secs(tick_seconds));
    let mut consecutive_failures: u32 = 0;

    loop {
        tokio::select! {
            _ = tick.tick() => {
                match cost_hold_expiry::run_tick(&pool).await {
                    Ok(report) => {
                        consecutive_failures = 0;
                        metrics::observe_tick(&report);
                        tracing::info!(?report, "expiry_tick_complete");
                    }
                    Err(e) => {
                        consecutive_failures += 1;
                        tracing::error!(?e, consecutive_failures, "expiry_tick_failed");
                        if consecutive_failures >= 10 {
                            tracing::error!("expiry_consecutive_failures_threshold");
                        }
                    }
                }
            }
            _ = shutdown.recv() => {
                tracing::info!("process_shutdown");
                break;
            }
        }
    }
    Ok(())
}
```

### SQL — scan + transition

```sql
-- One per-hold transaction:
BEGIN;
  SELECT id, tenant_id, estimated_usd, expires_at
    FROM cost_ledger_hold
    WHERE state = 'held' AND expires_at < NOW()
    ORDER BY expires_at ASC
    FOR UPDATE SKIP LOCKED
    LIMIT 1;

  -- (in Rust: per row, emit memory ai.hold_expired before this update)

  UPDATE cost_ledger_hold
    SET state = 'expired', refunded_at = NOW(), refund_reason = 'tick_expired'
    WHERE id = $hold_id AND state = 'held';
COMMIT;
```

The outer Rust loop fetches up to 500 hold IDs in one query (`LIMIT 500`), then processes each in its own transaction. The per-row transaction holds the row lock for only the duration of one memory emission + one UPDATE (~50ms).

### Systemd unit

```ini
# deploy/systemd/cyberos-ai-gateway-expiry.service
[Unit]
Description=CyberOS AI Gateway cost-hold expiry cleanup
After=network-online.target postgresql.service
Wants=network-online.target

[Service]
Type=simple
User=cyberos
ExecStart=/usr/local/bin/cost_hold_expiry
EnvironmentFile=/etc/cyberos/ai-gateway.env
Restart=always
RestartSec=5
TimeoutStopSec=10
KillSignal=SIGTERM
StandardOutput=journal
StandardError=journal
SyslogIdentifier=cyberos-ai-expiry

[Install]
WantedBy=multi-user.target
```

---

## §4 — Acceptance criteria

1. **Happy path** — Seed Postgres with 3 holds whose `expires_at = NOW() - INTERVAL '10 seconds'` and `state = 'held'`. Call `run_tick()`. MUST return `TickReport { holds_processed: 3, holds_succeeded: 3, holds_failed: 0 }`. All 3 holds MUST transition to `state = 'expired'`. Exactly 3 `ai.hold_expired` memory rows MUST exist on the chain, each chained to the previous.
2. **Non-expired holds skipped** — Seed 5 holds, 2 expired and 3 not. `run_tick()` MUST process only the 2 expired ones; the 3 non-expired MUST remain `state = 'held'` untouched.
3. **Already-reconciled holds skipped** — Seed a hold with `state = 'reconciled'` but `expires_at < NOW()` (an edge case where reconcile happened just before tick). `run_tick()` MUST NOT touch it (state filter is `state = 'held'`).
4. **`FOR UPDATE SKIP LOCKED` works** — Spawn a separate transaction that holds `SELECT ... FOR UPDATE` on hold A; call `run_tick()` against the same pool. Tick MUST skip hold A and process the other expired holds normally. Hold A MUST be processed in the next tick after the blocking transaction commits.
5. **memory failure rolls back row transition** — Inject a `memory_writer` failure for one specific hold. The transaction for that hold MUST roll back (state stays `'held'`); the other holds in the tick MUST process normally; `TickReport.holds_failed = 1`.
6. **Bounded batch size** — Seed 5,000 expired holds. `run_tick()` MUST process at most 500 (the LIMIT). The remaining 4,500 MUST be picked up on subsequent ticks.
7. **Graceful shutdown** — Send SIGTERM to the running binary mid-tick. The current hold's transaction MUST complete (commit or roll back); the binary MUST exit 0 within 5 seconds; no partial state visible on the next process start.
8. **Reconnect on DB drop** — Kill the Postgres container mid-binary-run. Wait 10 seconds, restart Postgres. The binary MUST reconnect (visible via the `consecutive_failures` counter resetting to 0 on next successful tick); MUST NOT crash.
9. **Crash recovery (weakened for slice 1)** — Run `run_tick()`, kill the process mid-iteration (between memory emit and UPDATE commit) via SIGKILL. On restart, the next tick MUST handle the un-transitioned hold by emitting a *second* `ai.hold_expired` memory row (slice 1 lacks dedup_key per TASK-AI-008 deferral) and then transitioning the hold. Both memory rows MUST chain correctly. The operator MUST run `cyberos-ai expiry repair` (TASK-AI-021) to dedupe after large crash events. This is a documented slice-1 limitation; TASK-AI-008 (slice 2 PyO3 spike) adds `dedup_key` to fix it natively.
10. **Metrics emitted** — After 3 successful ticks processing 10 holds total, `ai_expiry_holds_processed_total` MUST equal 10, `ai_expiry_tick_duration_seconds` histogram MUST have 3 observations, `ai_expiry_consecutive_failures_total` MUST equal 0.

---

## §5 — Verification method

**Integration test:** `services/ai-gateway/tests/cost_hold_expiry_test.rs`

```rust
#[tokio::test]
async fn tick_processes_expired_holds() {
    let env = TestEnv::new().await;
    env.seed_tenant("org:test-a", monthly_cap = 100, spent_usd = 50).await;
    for i in 0..3 {
        env.seed_hold(
            "org:test-a",
            estimated_usd = dec!(0.0085),
            expires_at = Utc::now() - Duration::seconds(10),
            state = "held",
        ).await;
    }

    let report = cost_hold_expiry::run_tick(&env.pool).await.unwrap();
    assert_eq!(report.holds_processed, 3);
    assert_eq!(report.holds_succeeded, 3);
    assert_eq!(report.holds_failed, 0);

    let holds = env.read_all_holds_for_tenant("org:test-a").await;
    assert_eq!(holds.iter().filter(|h| h.state == "expired").count(), 3);

    let ledger = env.read_ledger("org:test-a").await;
    assert_eq!(ledger.spent_usd, dec!(50));  // unchanged on expiry

    let chain_rows = env.memory_count_rows_by_kind("ai.hold_expired").await;
    assert_eq!(chain_rows, 3);
    assert!(env.verify_chain_from_genesis().await);
}

#[tokio::test]
async fn tick_skips_locked_rows() {
    let env = TestEnv::new().await;
    let lock_holder_id = env.seed_hold(/* expired */).await;

    // Lock the row from another transaction
    let mut blocking_tx = env.pool.begin().await.unwrap();
    sqlx::query("SELECT * FROM cost_ledger_hold WHERE id = $1 FOR UPDATE")
        .bind(lock_holder_id).fetch_one(&mut *blocking_tx).await.unwrap();

    let other_id = env.seed_hold(/* expired */).await;

    let report = cost_hold_expiry::run_tick(&env.pool).await.unwrap();
    assert_eq!(report.holds_processed, 1);  // skipped lock_holder_id

    let lock_holder = env.read_hold(lock_holder_id).await;
    assert_eq!(lock_holder.state, "held");  // still held

    blocking_tx.rollback().await.unwrap();

    // Next tick picks up the previously locked one
    let report2 = cost_hold_expiry::run_tick(&env.pool).await.unwrap();
    assert_eq!(report2.holds_processed, 1);
}

#[tokio::test]
async fn graceful_shutdown_finishes_current_hold() { /* AC #7 */ }

#[tokio::test]
async fn bounded_batch_500() { /* AC #6 */ }
```

Run via:

```bash
cd /Users/stephencheng/Projects/CyberSkill/cyberos
cargo test -p cyberos-ai-gateway cost_hold_expiry
```

**Smoke test for the binary:**

```bash
# Terminal 1: run binary
cargo run --bin cost_hold_expiry

# Terminal 2: seed expired holds, watch them transition
psql -c "INSERT INTO cost_ledger_hold (...) VALUES (...)"
psql -c "SELECT state, refunded_at FROM cost_ledger_hold WHERE expires_at < NOW();"

# Terminal 1: SIGTERM, observe clean exit
kill -SIGTERM $(pidof cost_hold_expiry)
```

---

## §6 — Implementation skeleton

```rust
// services/ai-gateway/src/cost_hold_expiry.rs

use sqlx::PgPool;
use uuid::Uuid;
use std::time::Instant;

const BATCH_SIZE: i64 = 500;

pub async fn run_tick(pool: &PgPool) -> Result<TickReport, TickError> {
    let started = Instant::now();
    let mut processed = 0u32;
    let mut succeeded = 0u32;
    let mut failed = 0u32;

    loop {
        // Fetch a batch of expired hold IDs in one read-only query
        let ids: Vec<Uuid> = sqlx::query_scalar(
            "SELECT id FROM cost_ledger_hold \
             WHERE state = 'held' AND expires_at < NOW() \
             ORDER BY expires_at ASC \
             LIMIT $1"
        )
        .bind(BATCH_SIZE)
        .fetch_all(pool)
        .await?;

        if ids.is_empty() { break; }

        let mut batch_attempted = 0u32;
        for id in ids {
            batch_attempted += 1;
            match process_one_hold(pool, id).await {
                Ok(HoldDisposition::Transitioned)        => { processed += 1; succeeded += 1; }
                Ok(HoldDisposition::AlreadyTransitioned) => { /* skip count */ }
                Err(e) => {
                    processed += 1;
                    failed += 1;
                    tracing::warn!(?id, ?e, "expiry_hold_failed");
                }
            }
        }

        // If we just processed a partial batch, we're done; otherwise loop for the next batch
        if batch_attempted < BATCH_SIZE as u32 { break; }
    }

    Ok(TickReport {
        holds_processed: processed,
        holds_succeeded: succeeded,
        holds_failed: failed,
        duration_ms: started.elapsed().as_millis() as u32,
    })
}

pub enum HoldDisposition {
    /// We touched the hold and transitioned it from 'held' to 'expired'
    Transitioned,
    /// Hold was already reconciled / expired / locked by another transaction;
    /// nothing for us to do.
    AlreadyTransitioned,
}

async fn process_one_hold(pool: &PgPool, hold_id: Uuid) -> Result<HoldDisposition, HoldError> {
    let mut tx = pool.begin().await?;

    // Lock + load the hold (SKIP LOCKED here means we just bail if someone else took it)
    let hold: Option<HoldRow> = sqlx::query_as(
        "SELECT id, tenant_id, idempotency_key, estimated_usd, expires_at \
         FROM cost_ledger_hold \
         WHERE id = $1 AND state = 'held' AND expires_at < NOW() \
         FOR UPDATE SKIP LOCKED"
    )
    .bind(hold_id)
    .fetch_optional(&mut *tx)
    .await?;

    let hold = match hold {
        Some(h) => h,
        None => return Ok(HoldDisposition::AlreadyTransitioned),
    };

    // Emit memory audit row INSIDE the transaction's lifetime
    memory_writer::emit(canonical::hold_expired(
        &hold.tenant_id,
        hold.id,
        hold.expires_at,
        hold.estimated_usd,
    ))
    .await
    .map_err(|e| HoldError::MemoryEmitFailed(e))?;

    // Transition the hold
    sqlx::query(
        "UPDATE cost_ledger_hold \
         SET state = 'expired', refunded_at = NOW(), refund_reason = 'tick_expired' \
         WHERE id = $1"
    )
    .bind(hold.id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(HoldDisposition::Transitioned)
}
```

*Scaffold above is suggestive. AC §4 is the contract.*

---

## §7 — Dependencies

**Code dependencies:**
- TASK-AI-001 — `cost_ledger_hold` table exists.
- TASK-AI-002 — `cost_ledger_hold.refunded_at` + `refund_reason` columns exist.
- TASK-AI-003 — `memory_writer::emit()` + `canonical::hold_expired()` are available.

**Concept dependencies:**
- The state machine for `cost_ledger_hold.state`: `held → reconciled | refunded | expired`. This FR is the only path that creates `'expired'` rows.

**Operational dependencies:**
- Postgres at `DATABASE_URL`.
- A systemd-equivalent supervisor (Docker `restart: always`, Kubernetes Deployment, or systemd directly). The slice-1 deploy target is systemd on bare metal.

---

## §8 — Example payloads

### Postgres state — before tick

```
| id           | tenant_id      | state | expires_at                | estimated_usd |
|--------------|----------------|-------|---------------------------|---------------|
| 01HZK9R7A2B4 | org:cyberskill | held  | 2026-05-15 09:30:00+00    | 0.0085        |
| 01HZK9R7A2B5 | org:cyberskill | held  | 2026-05-15 09:29:50+00    | 0.0120        |
| 01HZK9R8M3X5 | org:test-a     | held  | 2026-05-15 09:31:30+00    | 0.0050        |
```

(NOW() = 2026-05-15 09:30:30+00; rows 1 and 2 are expired)

### `run_tick()` returns

```rust
TickReport {
    holds_processed: 2,
    holds_succeeded: 2,
    holds_failed: 0,
    duration_ms: 120,
}
```

### Postgres state — after tick

```
| id           | tenant_id      | state    | expires_at                | refunded_at           | refund_reason  |
|--------------|----------------|----------|---------------------------|-----------------------|----------------|
| 01HZK9R7A2B4 | org:cyberskill | expired  | 2026-05-15 09:30:00+00    | 2026-05-15 09:30:30   | tick_expired   |
| 01HZK9R7A2B5 | org:cyberskill | expired  | 2026-05-15 09:29:50+00    | 2026-05-15 09:30:30   | tick_expired   |
| 01HZK9R8M3X5 | org:test-a     | held     | 2026-05-15 09:31:30+00    | NULL                  | NULL           |
```

### memory row written (per expired hold)

```json
{
  "seq": 18450,
  "ts_ns": 1763112630000000000,
  "op": "put",
  "path": "memories/decisions/ai-invocations/1763112630000_org-cyberskill_hold-expired_01HZK9R7A2B4.md",
  "extra": {
    "kind": "ai.hold_expired",
    "tenant_id": "org:cyberskill",
    "hold_id": "01HZK9R7A2B4",
    "expires_at": "2026-05-15T09:30:00Z",
    "refund_amount_usd": 0.0085,
    "tick_id": "01HZKHA2B4C8D6E1F3G5"
  },
  "prev_chain": "...",
  "chain": "..."
}
```

### Tick log line

```
{"ts":"2026-05-15T09:30:30.120Z","level":"info","event":"expiry_tick_complete","holds_processed":2,"holds_succeeded":2,"holds_failed":0,"duration_ms":120}
```

---

## §9 — Open questions

All resolved 2026-05-15 (round 2). Promoted to §1 normative clauses or deferred with explicit owner:

1. **~~Idempotent memory re-emit on crash recovery~~** → deferred to TASK-AI-008 + TASK-AI-021 (cleanup CLI). AC #9 weakened with explicit note. Was Q1.
2. **~~DB pool sizing~~** → pool size 2 (one bulk-fetch, one per-hold tx); revisit if profiling shows contention. Was Q2.
3. **~~Empty hold-table tick~~** → no-op SELECT, ~5ms overhead, no harm; do nothing. Was Q3.
4. **~~Leader election~~** → §1 #13 (NOT implemented; SKIP LOCKED makes multi-instance safe). Was Q4.
5. **~~tick_id correlation~~** → §1 #11 (ULID-26 per tick on every audit row). Was Q5.
2. **Should the binary do its own DB pool sizing?** — Tick's max in-flight transactions are bounded by the per-hold loop (sequential, not parallel). One DB connection is sufficient. Proposal: pool size 2 (one for the bulk-fetch query, one for the per-hold transaction); revisit if profiling shows contention.
3. **What if the cost_ledger_hold table is *empty* but ticks keep running?** — No-op tick, ~5ms of SELECT overhead, no harm. We could short-circuit by tracking last-non-zero tick time, but the optimisation buys nothing meaningful at any scale we care about. Proposal: don't bother.
4. **Leader election for multiple instances** — One instance is enough for the foreseeable future. If someone deploys two by accident, Postgres's `SKIP LOCKED` keeps them safe (one of them just does no work per tick). Proposal: do nothing; TASK-AI-021 surfaces a warning if `ps -ef | grep cost_hold_expiry | wc -l > 1` in the operator-CLI health check.
5. **Tick-id correlation across rows** — Each tick's `ai.hold_expired` rows should carry the same `tick_id` so OBS investigators can correlate them. Proposed format: `01<26-char-ULID>` allocated at tick start. Adds 1 field to the audit row's `extra`. Worth it.

---

## §10 — Failure modes inventory

| Failure | Detection | Action | Recovery |
|---|---|---|---|
| Postgres unreachable | sqlx::Error on initial connect | Reconnect with exponential backoff (1s, 2s, 4s, 8s, capped 30s) | Self-healing; `ai_expiry_consecutive_failures_total` counter tracks |
| Postgres connection drops mid-tick | sqlx::Error on per-hold transaction | Per-hold failure incremented; tick continues with next hold | Self-healing; no per-tick rollback needed |
| memory Writer fails for a specific hold | `memory_writer::emit` returns Err | Hold transaction rolls back (state stays `'held'`); `holds_failed++` | Hold gets processed next tick (likely succeeds); if persistent, OBS sev-2 |
| memory Writer hangs > 5s | Writer's own timeout | Same as above (Writer's timeout becomes our error) | Tick continues |
| Bulk-fetch query returns 0 expired holds | Normal | Tick exits the inner loop; sleeps 30s | Normal idle state |
| Bulk-fetch returns 500+ holds | Normal under load | Process 500; remaining picked up next tick | Normal; consider lowering precheck TTL if persistent |
| Hold lock contention (another process holding the row) | `FOR UPDATE SKIP LOCKED` returns None | Skip the row; process the next | Normal; row processed next tick when lock releases |
| Crash mid-tick after memory emit but before UPDATE commit | Process dies | Next tick may emit a duplicate memory row + complete transition | Slice-1 limitation; TASK-AI-021 `cyberos-ai expiry repair` dedupes |
| SIGTERM during tick | tokio::select catches it | Finish current hold's transaction; exit 0 within 5s | Clean shutdown |
| Consecutive failure count exceeds 10 (5 minutes total failure) | counter check at end of each failed tick | Sev-2 log; TASK-OBS-007 routes alarm (when shipped) | Operator investigates |

---

## §11 — Notes

- This is the smallest FR in slice 1 (3h effort) but probably the highest-leverage at runtime — it's the only thing standing between "holds that got reconciled" and "holds that didn't ever settle". Without it, the hold table is unbounded growth and the chain is missing a class of rows.
- The systemd unit lives in `deploy/systemd/` not `services/ai-gateway/deploy/`; that path convention is set by the platform team (deploy assets centralised, code repos clean).
- The `ai_expiry_consecutive_failures_total` counter resets on each successful tick — its purpose is to drive an alert when cleanup is *completely* broken, not to count cumulative lifetime failures. The latter is provided by `ai_expiry_holds_processed_total` minus the lifetime `holds_succeeded` rollup in the OBS dashboard.
- Once TASK-AI-021 (operator CLI) ships, the `cyberos-ai expiry-status` subcommand will surface the binary's live tick state without needing OBS — useful for low-environment debugging.
- Future enhancement (TASK-AI-104, post-P0): when the same tenant has many expired holds in a window, fold them into a single batch memory row (`ai.hold_expired_batch`) to reduce chain noise. Deferred until production traffic justifies the optimisation.

---

*End of TASK-AI-004. Run `task-audit` next: `cargo run -p cyberos-skill-cli -- run task-audit --input '{"fr_path": "docs/tasks/ai/TASK-AI-004-cost-hold-expiry-cleanup/spec.md"}'`*
