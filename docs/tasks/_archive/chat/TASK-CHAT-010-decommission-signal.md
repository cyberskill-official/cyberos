---
id: TASK-CHAT-010
title: "Decommission signal — (chat msgs) / (chat + slack + zalo msgs) ≥ 0.95 over 14-day rolling window with per-tenant trigger"
module: CHAT
priority: MUST
status: superseded
superseded_by: TASK-CHAT-101 (first-party native chat replaced the Mattermost fork wholesale; still-wanted intents re-homed as TASK-CHAT-102..106)
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_tasks: [TASK-CHAT-006, TASK-CHAT-007, TASK-OBS-007]
depends_on: [TASK-CHAT-006, TASK-CHAT-007]
blocks: []

source_pages:
  - website/docs/modules/chat.html#decommission
source_decisions:
  - DEC-510 (signal fires when CyberOS-CHAT carries ≥95% of message volume vs. legacy sources for 14 days)
  - DEC-511 (signal is INFORMATIONAL only; never auto-shuts off imports)

language: rust 1.81
service: cyberos/services/chat-importer/
new_files:
  - services/chat-importer/src/decommission/mod.rs
  - services/chat-importer/src/decommission/signal.rs
  - services/chat-importer/tests/decommission_test.rs
modified_files:
  - services/chat-importer/src/main.rs               # add `decommission check` subcommand
allowed_tools:
  - file_read: services/chat-importer/**
  - file_write: services/chat-importer/{src,tests}/**
  - bash: cd services/chat-importer && cargo test decommission
disallowed_tools:
  - auto-stop imports based on signal (per DEC-511 — informational)
  - compute over window < 14 days (sample-size protection)

effort_hours: 5
subtasks:
  - "0.5h: decommission/mod.rs — DecommissionSignal struct"
  - "1.0h: signal.rs — query message counts per source over rolling 14d"
  - "1.0h: ratio computation + threshold check"
  - "0.5h: memory audit 'chat.decommission_signal'"
  - "0.5h: CLI subcommand `decommission check --tenant <id>`"
  - "0.5h: scheduled nightly run via cron"
  - "1.0h: decommission_test.rs — ratio math + threshold + non-fire when insufficient data"
risk_if_skipped: "Without signal, tenants pay for two chat tools indefinitely. With it, ops can confidently retire legacy Slack/Zalo subscriptions ('CHAT has carried 96% of traffic for 14 days; safe to cancel'). Without 14-day window, single-day spikes trigger false positive."
---

## §1 — Description (BCP-14 normative)

The decommission signal **MUST** compute the share of CyberOS-CHAT vs. legacy sources and fire when threshold met. The contract:

1. **MUST** count messages over a rolling 14-day window, partitioned by source:
   - CyberOS-CHAT: count of `posts` rows where `create_at >= now() - 14d` AND `props.cyberos_source IS NULL OR props.cyberos_source != 'imported_*'`.
   - Slack import: count from `posts WHERE props.cyberos_source = 'imported_slack' AND props.original_ts >= now() - 14d`.
   - Zalo import: count from `posts WHERE props.cyberos_source = 'imported_zalo' AND props.original_ts >= now() - 14d`.
2. **MUST** compute ratio = `chat / (chat + slack + zalo)`. If denominator < 100 messages → insufficient data; return `Status::InsufficientData`.
3. **MUST** mark `Status::Ready` when ratio ≥ 0.95 AND window is full 14 days AND total ≥ 100.
4. **MUST** mark `Status::NotReady` otherwise; payload includes current ratio + gap to threshold.
5. **MUST** be INFORMATIONAL only: never modify import workflows, never block writes, never disable legacy paths.
6. **MUST** emit memory audit `chat.decommission_signal` per check with payload `{tenant_id, ratio, chat_count, slack_count, zalo_count, status, window_start, window_end, trace_id}`.
7. **MUST** run nightly at 02:30 (after TASK-PROJ-010 drift sweep, before TASK-PROJ-013 calibration).
8. **MUST** support on-demand CLI: `cyberos-chat decommission check --tenant <id>` → prints status JSON.
9. **MUST** notify via TASK-OBS-007 when status transitions from NotReady → Ready (sev-3 info).
10. **MUST** emit OTel metrics:
    - `chat_decommission_ratio{tenant_id}` (gauge).
    - `chat_decommission_status_total{status}` (counter).
11. **MUST** track per-source breakdown so operators can answer "which legacy source is still active": payload contains `{chat_count, slack_count, zalo_count, slack_active_users, zalo_active_users}` (active = posted ≥ 1 message in window).
12. **MUST** require ratio to be ≥ 0.95 for THREE consecutive nightly checks (not just one) before transitioning to Ready. Single-check transitions are too noisy; consecutive-check requirement smooths over single-day anomalies. Status enum gains `Ready` (after 3 consecutive) + `Approaching` (1 or 2 consecutive ≥ 0.95).
13. **MUST** persist a per-tenant `decommission_state` row capturing `{tenant_id, current_status, ready_streak_days, first_ready_at, last_check_at, last_ratio}`. Updated atomically with each check.
14. **MUST** support tenant-level threshold override: `cyberos_chat_tenant_settings.decommission_threshold` (float 0..1, default 0.95). Some tenants (e.g. strict compliance) want a higher bar (0.98); some prove out at 0.90.
15. **MUST** detect and signal `Regression` status when a previously-Ready tenant drops below threshold for ≥ 2 consecutive checks. Payload includes `{prior_status, drop_from_ratio, drop_to_ratio, contributing_source}`. SEV-2 alert (regression > novel info).
16. **MUST** offer per-tenant "snooze" via CLI: `cyberos-chat decommission snooze --tenant <id> --until <date>`. Snoozed tenants skip checks until date; emit `chat.decommission_snoozed` audit at snooze time.
17. **MUST** include `recommended_action` field in payload:
    - InsufficientData → "Insufficient data; continue normal operation"
    - NotReady (ratio < 0.5) → "Substantial legacy traffic; do not decommission"
    - NotReady (ratio 0.5–0.95) → "Migration in progress; monitor"
    - Approaching → "Near decommission threshold; review with stakeholders"
    - Ready → "Safe to decommission legacy sources; coordinate with operations"
    - Regression → "Legacy traffic returned; investigate cause before decommissioning"
18. **MUST** include a `last_legacy_message_at` timestamp in the payload (max of legacy sources' last message); operators want "the last Slack message was 9 days ago" as a human-meaningful signal.
19. **MUST** support per-source weights (slice-extension): config `decommission_source_weights = {slack: 1.0, zalo: 1.5}` allows operators to count Zalo more heavily (more SMB-critical in VN market). Default is all 1.0.
20. **MUST** emit a `chat.decommission_state_changed` memory row WHENEVER status changes (not just on transition to Ready). Operators tracking adoption see every state change.
21. **MUST** provide a 30-day trend in the payload as `ratio_history: [{date, ratio}, ...]` (last 30 days of nightly checks); operators see the curve, not just the point.

---

## §2 — Why this design

**Why 14-day window (DEC-510)?** Single-week may catch spike (conference, launch); 14 days smooths. Empirical: 95% threshold over 14 days correlates with operator-confidence to cancel legacy subscription.

**Why 0.95 not 1.0 (§1 #3)?** Long-tail of stragglers (one user still on Slack). 95% = "everyone who matters has moved."

**Why informational only (DEC-511)?** Auto-disabling imports would corrupt the import_jobs checkpoint table (mid-import abort). Operator decides; signal is the input.

**Why min 100 messages (§1 #2)?** Below 100, ratio is noisy (5/4 = 56% but meaningless). Threshold protects from "your trial tenant just barely passes."

**Why sev-3 on transition (§1 #9)?** Informational; not actionable urgently. Surfaces in daily digest.

**Why active-users breakdown (§1 #11)?** Operators investigating "should we decommission Slack?" want to know whether the remaining 5% is concentrated (5 users still using Slack heavily) or distributed (50 users each occasionally). Active-user count answers that.

**Why 3 consecutive checks (§1 #12)?** Single-day spikes (vacation, conference, downtime) shouldn't flip status. 3 consecutive = 3 days of stable ratio = enough evidence.

**Why per-tenant override (§1 #14)?** Default 95% suits most; compliance-heavy tenants want stricter (98%); SMB tenants migrating from heavy Slack may accept 90% as "good enough." Operator-set threshold respects per-customer reality.

**Why detect Regression (§1 #15)?** A tenant that crossed Ready then dropped is a SIGNAL — usually means legacy was re-activated (a team rejoined, a vendor sent messages). Surface as SEV-2 because operators may have already begun decommissioning steps.

**Why snooze (§1 #16)?** Some tenants are in a known "we're not decommissioning anytime soon" state (M&A pending, vendor renewal); nightly Ready alerts are noise. Snooze respects that.

**Why recommended_action (§1 #17)?** Operators reading the JSON shouldn't need to memorise the decision tree. The field encodes the contextual advice based on numeric state.

**Why last_legacy_message_at (§1 #18)?** Concrete date is more compelling than ratio. "The last Slack message was 9 days ago" is what operators actually communicate to stakeholders.

**Why per-source weights (§1 #19)?** Vietnam-market tenants find Zalo more business-critical than Slack; an operator may want to weight Zalo migration heavier. The default doesn't impose; the knob respects context.

**Why state-changed audit (§1 #20)?** Operators reviewing tenant lifecycle answer "when did they cross 80%?" via memory query. Without state-change rows, only the final transition is visible.

**Why 30-day trend (§1 #21)?** Operators visually inspecting "is migration accelerating or stalling?" need the curve, not just the point. 30 days = one operator-decision window.

---

## §3 — API contract

```rust
// services/chat-importer/src/decommission/mod.rs
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status { Ready, NotReady, InsufficientData }

#[derive(Clone, Debug, Serialize)]
pub struct DecommissionSignal {
    pub tenant_id: uuid::Uuid,
    pub status: Status,
    pub ratio: f64,
    pub chat_count: i64,
    pub slack_count: i64,
    pub zalo_count: i64,
    pub total: i64,
    pub window_start: chrono::DateTime<chrono::Utc>,
    pub window_end: chrono::DateTime<chrono::Utc>,
}
```

```rust
// services/chat-importer/src/decommission/signal.rs
use chrono::{DateTime, Utc, Duration};

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Ready,             // ≥ threshold for 3 consecutive days
    Approaching,       // ≥ threshold for 1–2 consecutive days
    NotReady,          // < threshold
    Regression,        // was Ready, dropped < threshold for ≥ 2 consecutive days
    InsufficientData,  // total < 100 messages
    Snoozed,           // operator snoozed via CLI
}

#[derive(Clone, Debug, Serialize)]
pub struct DecommissionSignal {
    pub tenant_id:                uuid::Uuid,
    pub status:                   Status,
    pub ratio:                    f64,
    pub threshold:                f64,
    pub chat_count:               i64,
    pub slack_count:              i64,
    pub zalo_count:               i64,
    pub slack_active_users:       i64,
    pub zalo_active_users:        i64,
    pub total:                    i64,
    pub window_start:             DateTime<Utc>,
    pub window_end:               DateTime<Utc>,
    pub ready_streak_days:        i32,
    pub regression_streak_days:   i32,
    pub last_legacy_message_at:   Option<DateTime<Utc>>,
    pub recommended_action:       &'static str,
    pub ratio_history:            Vec<HistoryPoint>,
}

#[derive(Clone, Debug, Serialize)]
pub struct HistoryPoint { pub date: chrono::NaiveDate, pub ratio: f64 }

pub async fn check_tenant(
    pool: &sqlx::PgPool,
    tenant_id: uuid::Uuid,
) -> anyhow::Result<DecommissionSignal> {
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant_id.to_string()).execute(pool).await?;

    // Check snooze first.
    if let Some(until) = sqlx::query_scalar::<_, DateTime<Utc>>(
        "SELECT snoozed_until FROM cyberos_chat_tenant_settings WHERE tenant_id = $1
            AND snoozed_until > NOW()"
    ).bind(tenant_id).fetch_optional(pool).await? {
        return build_snoozed(tenant_id, until).await;
    }

    let threshold = sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(decommission_threshold, 0.95)
           FROM cyberos_chat_tenant_settings WHERE tenant_id = $1"
    ).bind(tenant_id).fetch_optional(pool).await?.unwrap_or(0.95);

    let now = Utc::now();
    let window_start = now - Duration::days(14);

    // Per-source counts + active-user counts + last legacy ts in one query.
    let row = sqlx::query!(r#"
        SELECT
          COUNT(*) FILTER (WHERE props->>'cyberos_source' IS NULL
                               OR props->>'cyberos_source' NOT LIKE 'imported_%')   AS chat_count,
          COUNT(*) FILTER (WHERE props->>'cyberos_source' = 'imported_slack')       AS slack_count,
          COUNT(*) FILTER (WHERE props->>'cyberos_source' = 'imported_zalo')        AS zalo_count,
          COUNT(DISTINCT user_id) FILTER (WHERE props->>'cyberos_source' = 'imported_slack') AS slack_users,
          COUNT(DISTINCT user_id) FILTER (WHERE props->>'cyberos_source' = 'imported_zalo')  AS zalo_users,
          MAX(create_at) FILTER (WHERE props->>'cyberos_source' LIKE 'imported_%')  AS last_legacy_ts
        FROM posts
        WHERE create_at >= $1 AND delete_at IS NULL
    "#, window_start).fetch_one(pool).await?;

    let chat   = row.chat_count.unwrap_or(0);
    let slack  = row.slack_count.unwrap_or(0);
    let zalo   = row.zalo_count.unwrap_or(0);
    let total  = chat + slack + zalo;
    let active_slack = row.slack_users.unwrap_or(0);
    let active_zalo  = row.zalo_users.unwrap_or(0);
    let last_legacy_ts = row.last_legacy_ts.map(|ts| Utc.timestamp_opt(ts, 0).unwrap());

    let prior_state = fetch_state(pool, tenant_id).await?;
    let raw_ratio = if total > 0 { chat as f64 / total as f64 } else { 0.0 };

    let (status, streak_ready, streak_regression) = derive_status(
        total, raw_ratio, threshold, &prior_state,
    );

    let recommended_action = recommend(status, raw_ratio);
    let history = fetch_ratio_history(pool, tenant_id, 30).await?;

    let signal = DecommissionSignal {
        tenant_id, status, ratio: raw_ratio, threshold,
        chat_count: chat, slack_count: slack, zalo_count: zalo,
        slack_active_users: active_slack, zalo_active_users: active_zalo,
        total, window_start, window_end: now,
        ready_streak_days: streak_ready, regression_streak_days: streak_regression,
        last_legacy_message_at: last_legacy_ts,
        recommended_action,
        ratio_history: history,
    };

    persist_state(pool, &signal).await?;
    emit_memory_row("chat.decommission_signal", serde_json::to_value(&signal)?).await;

    if status != prior_state.as_ref().map(|s| s.current_status).unwrap_or(Status::InsufficientData) {
        emit_memory_row("chat.decommission_state_changed", serde_json::json!({
            "tenant_id": tenant_id,
            "from": prior_state.as_ref().map(|s| s.current_status),
            "to":   status,
            "ratio": raw_ratio,
        })).await;
    }

    metrics::gauge!("chat_decommission_ratio", "tenant_id" => tenant_id.to_string()).set(raw_ratio);
    metrics::counter!("chat_decommission_status_total",
        "status" => status_label(status).to_string()).increment(1);

    // Alerts on transitions.
    match (prior_state.as_ref().map(|s| s.current_status), status) {
        (Some(p), Status::Ready) if p != Status::Ready =>
            obs::alert(obs::Severity::Sev3, "chat_decommission_ready",
                serde_json::json!({"tenant_id": tenant_id, "ratio": raw_ratio})).await,
        (Some(Status::Ready), Status::Regression) =>
            obs::alert(obs::Severity::Sev2, "chat_decommission_regression",
                serde_json::json!({"tenant_id": tenant_id, "ratio": raw_ratio,
                                  "from_ready_to": status_label(status)})).await,
        _ => {}
    }

    Ok(signal)
}

fn derive_status(
    total: i64,
    ratio: f64,
    threshold: f64,
    prior: &Option<DecommissionState>,
) -> (Status, i32, i32) {
    if total < 100 { return (Status::InsufficientData, 0, 0); }
    let prior_streak_ready = prior.as_ref().map(|s| s.ready_streak_days).unwrap_or(0);
    let prior_streak_reg   = prior.as_ref().map(|s| s.regression_streak_days).unwrap_or(0);
    let was_ready = matches!(prior.as_ref().map(|s| s.current_status), Some(Status::Ready));

    if ratio >= threshold {
        let next_streak = prior_streak_ready + 1;
        let status = if next_streak >= 3 { Status::Ready }
                     else if next_streak >= 1 { Status::Approaching }
                     else { Status::NotReady };
        (status, next_streak, 0)
    } else {
        let next_reg = if was_ready { prior_streak_reg + 1 } else { 0 };
        let status = if next_reg >= 2 { Status::Regression } else { Status::NotReady };
        (status, 0, next_reg)
    }
}

fn recommend(status: Status, ratio: f64) -> &'static str {
    match (status, ratio) {
        (Status::InsufficientData, _) => "Insufficient data; continue normal operation",
        (Status::Snoozed,          _) => "Snoozed by operator; checks skipped",
        (Status::Ready,            _) => "Safe to decommission legacy sources; coordinate with operations",
        (Status::Regression,       _) => "Legacy traffic returned; investigate cause before decommissioning",
        (Status::Approaching,      _) => "Near decommission threshold; review with stakeholders",
        (Status::NotReady, r) if r < 0.5  => "Substantial legacy traffic; do not decommission",
        (Status::NotReady, _) => "Migration in progress; monitor",
    }
}

fn status_label(s: Status) -> &'static str {
    match s {
        Status::Ready             => "ready",
        Status::Approaching       => "approaching",
        Status::NotReady          => "not_ready",
        Status::Regression        => "regression",
        Status::InsufficientData  => "insufficient_data",
        Status::Snoozed           => "snoozed",
    }
}
```

### state.rs — persist + fetch decommission state

```rust
// services/chat-importer/src/decommission/state.rs
pub struct DecommissionState {
    pub tenant_id:               uuid::Uuid,
    pub current_status:          Status,
    pub ready_streak_days:       i32,
    pub regression_streak_days:  i32,
    pub first_ready_at:          Option<DateTime<Utc>>,
    pub last_check_at:           DateTime<Utc>,
    pub last_ratio:              f64,
}

pub async fn fetch_state(pool: &sqlx::PgPool, tenant_id: uuid::Uuid)
    -> sqlx::Result<Option<DecommissionState>>
{
    sqlx::query_as!(DecommissionState,
        "SELECT * FROM cyberos_chat_decommission_state WHERE tenant_id = $1",
        tenant_id
    ).fetch_optional(pool).await
}

pub async fn persist_state(pool: &sqlx::PgPool, sig: &DecommissionSignal) -> sqlx::Result<()> {
    sqlx::query!(
        r#"INSERT INTO cyberos_chat_decommission_state
              (tenant_id, current_status, ready_streak_days,
               regression_streak_days, first_ready_at, last_check_at, last_ratio)
           VALUES ($1, $2, $3, $4,
              CASE WHEN $2 = 'ready' AND
                        (SELECT first_ready_at FROM cyberos_chat_decommission_state WHERE tenant_id = $1) IS NULL
                   THEN NOW()
                   ELSE (SELECT first_ready_at FROM cyberos_chat_decommission_state WHERE tenant_id = $1)
              END,
              NOW(), $5)
           ON CONFLICT (tenant_id) DO UPDATE SET
              current_status = EXCLUDED.current_status,
              ready_streak_days = EXCLUDED.ready_streak_days,
              regression_streak_days = EXCLUDED.regression_streak_days,
              first_ready_at = EXCLUDED.first_ready_at,
              last_check_at = NOW(),
              last_ratio = $5"#,
        sig.tenant_id, status_label(sig.status), sig.ready_streak_days,
        sig.regression_streak_days, sig.ratio
    ).execute(pool).await?;
    Ok(())
}

pub async fn fetch_ratio_history(pool: &sqlx::PgPool, tenant_id: uuid::Uuid, days: i32)
    -> sqlx::Result<Vec<HistoryPoint>>
{
    sqlx::query_as!(HistoryPoint,
        "SELECT DISTINCT ON (ts_ns::date)
                ts_ns::date AS date,
                (payload->>'ratio')::float8 AS ratio
           FROM memory_audit
           WHERE kind = 'chat.decommission_signal'
             AND tenant_id = $1
             AND ts_ns > NOW() - INTERVAL '1 day' * $2
           ORDER BY ts_ns::date DESC, ts_ns ASC",
        tenant_id, days
    ).fetch_all(pool).await
}
```

### snooze.rs — operator snooze

```rust
pub async fn snooze(pool: &sqlx::PgPool, tenant_id: uuid::Uuid, until: DateTime<Utc>) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE cyberos_chat_tenant_settings SET snoozed_until = $1 WHERE tenant_id = $2"
    ).bind(until).bind(tenant_id).execute(pool).await?;
    emit_memory_row("chat.decommission_snoozed", serde_json::json!({
        "tenant_id": tenant_id, "snoozed_until": until,
    })).await;
    Ok(())
}

pub async fn unsnooze(pool: &sqlx::PgPool, tenant_id: uuid::Uuid) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE cyberos_chat_tenant_settings SET snoozed_until = NULL WHERE tenant_id = $1"
    ).bind(tenant_id).execute(pool).await?;
    emit_memory_row("chat.decommission_unsnoozed", serde_json::json!({
        "tenant_id": tenant_id,
    })).await;
    Ok(())
}
```

### Schema additions

```sql
-- services/chat/sql/init-decommission.sql
CREATE TABLE IF NOT EXISTS cyberos_chat_decommission_state (
    tenant_id              UUID PRIMARY KEY,
    current_status         TEXT NOT NULL,
    ready_streak_days      INT NOT NULL DEFAULT 0,
    regression_streak_days INT NOT NULL DEFAULT 0,
    first_ready_at         TIMESTAMPTZ,
    last_check_at          TIMESTAMPTZ,
    last_ratio             FLOAT8
);

ALTER TABLE cyberos_chat_tenant_settings ADD COLUMN IF NOT EXISTS
    decommission_threshold FLOAT8 NOT NULL DEFAULT 0.95;
ALTER TABLE cyberos_chat_tenant_settings ADD COLUMN IF NOT EXISTS
    snoozed_until TIMESTAMPTZ;
ALTER TABLE cyberos_chat_tenant_settings ADD COLUMN IF NOT EXISTS
    decommission_source_weights JSONB NOT NULL DEFAULT '{"slack": 1.0, "zalo": 1.0}';
```

---

## §4 — Acceptance criteria

1. **Ready status at 95% ratio** — fixture: 950 chat, 50 slack → status=Ready, ratio=0.95.
2. **NotReady at 80%** — fixture: 800 chat, 200 slack → status=NotReady.
3. **InsufficientData at total < 100** — fixture: 50 chat, 30 slack → status=InsufficientData.
4. **14-day window applied** — messages older than 14d excluded.
5. **Soft-deleted messages excluded** — `delete_at IS NOT NULL` not counted.
6. **Per-source counting via props** — `cyberos_source` discriminates.
7. **memory audit chat.decommission_signal emitted per check**.
8. **OTel gauge `chat_decommission_ratio` set**.
9. **Sev-3 alert on NotReady → Ready transition**.
10. **No alert on first check if Ready** — initial check has no prior; informational only.
11. **CLI prints JSON status**.
12. **Cron runs nightly at 02:30**.
13. **RLS isolates per-tenant**.
14. **InsufficientData → ratio=0** — no division by zero.
15. **Active-user counts populated** — payload `slack_active_users` + `zalo_active_users` are non-zero when those sources have user activity (AC for §1 #11).
16. **3-consecutive-checks gate for Ready** — fixture: 1st check 0.96 → Approaching (streak=1); 2nd check 0.97 → Approaching (streak=2); 3rd check 0.96 → Ready (streak=3) (AC for §1 #12).
17. **State persists across checks** — observe `cyberos_chat_decommission_state` row updated atomically with ready_streak_days (AC for §1 #13).
18. **Per-tenant threshold override honoured** — set `decommission_threshold=0.98`; fixture ratio 0.96 → NotReady, not Ready (AC for §1 #14).
19. **Regression detected** — fixture: ratio 0.96 then 0.96 then 0.80 then 0.78 → status=Regression at 4th check; SEV-2 alert (AC for §1 #15).
20. **Snooze skips checks** — `snooze --until 2026-06-01`; checks return Status::Snoozed; no `chat.decommission_signal` row (AC for §1 #16).
21. **recommended_action populated per status** — observe each status produces the right human-readable action (AC for §1 #17).
22. **last_legacy_message_at present** — fixture with legacy posts → field populated; absent → null (AC for §1 #18).
23. **chat.decommission_state_changed audit on every state change** — fixture: NotReady → Approaching → Ready → 3 state-change rows (AC for §1 #20).
24. **30-day trend in payload** — fixture: 30 prior nightly signals → payload `ratio_history.len() == 30` (AC for §1 #21).
25. **No spam on stable status** — fixture: ratio stable at 0.85 for 5 checks → 1 transition row only (NotReady is initial; no change since) (AC for §1 #20).

---

## §5 — Verification

### AC #1 — Approaching at first ≥-threshold check

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac1_approaching_on_first_check_at_threshold() {
    let env = TestEnv::new().await;
    env.seed_messages(env.tenant_id(), 950, 50, 0).await;
    let s = check_tenant(&env.pool, env.tenant_id()).await.unwrap();
    assert_eq!(s.status, Status::Approaching);
    assert!((s.ratio - 0.95).abs() < 0.01);
    assert_eq!(s.ready_streak_days, 1);
}
```

### AC #16 — 3-consecutive gate for Ready

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac16_three_consecutive_gates_ready() {
    let env = TestEnv::new().await;
    env.seed_messages(env.tenant_id(), 960, 40, 0).await;

    let s1 = check_tenant(&env.pool, env.tenant_id()).await.unwrap();
    assert_eq!(s1.status, Status::Approaching);
    assert_eq!(s1.ready_streak_days, 1);

    let s2 = check_tenant(&env.pool, env.tenant_id()).await.unwrap();
    assert_eq!(s2.status, Status::Approaching);
    assert_eq!(s2.ready_streak_days, 2);

    let s3 = check_tenant(&env.pool, env.tenant_id()).await.unwrap();
    assert_eq!(s3.status, Status::Ready);
    assert_eq!(s3.ready_streak_days, 3);
}
```

### AC #3 — insufficient data

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac3_insufficient_data() {
    let env = TestEnv::new().await;
    env.seed_messages(env.tenant_id(), 50, 30, 0).await;
    let s = check_tenant(&env.pool, env.tenant_id()).await.unwrap();
    assert_eq!(s.status, Status::InsufficientData);
    assert_eq!(s.ratio, 0.0);
    assert_eq!(s.recommended_action, "Insufficient data; continue normal operation");
}
```

### AC #15 — active-user counts

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac15_active_user_counts() {
    let env = TestEnv::new().await;
    env.seed_messages_by_users(env.tenant_id(), 950, 50, 0, /* slack_users */ 3, /* zalo_users */ 0).await;
    let s = check_tenant(&env.pool, env.tenant_id()).await.unwrap();
    assert_eq!(s.slack_active_users, 3);
    assert_eq!(s.zalo_active_users, 0);
}
```

### AC #18 — per-tenant threshold override

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac18_per_tenant_threshold() {
    let env = TestEnv::new().await;
    env.set_tenant_decommission_threshold(env.tenant_id(), 0.98).await;
    env.seed_messages(env.tenant_id(), 960, 40, 0).await; // 0.96 ratio
    let s = check_tenant(&env.pool, env.tenant_id()).await.unwrap();
    assert_eq!(s.status, Status::NotReady);
    assert_eq!(s.threshold, 0.98);
}
```

### AC #19 — Regression detected

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac19_regression_detected() {
    let env = TestEnv::new().await;
    // 3 days at 0.96 → Ready
    env.seed_messages(env.tenant_id(), 960, 40, 0).await;
    check_tenant(&env.pool, env.tenant_id()).await.unwrap();
    check_tenant(&env.pool, env.tenant_id()).await.unwrap();
    let s_ready = check_tenant(&env.pool, env.tenant_id()).await.unwrap();
    assert_eq!(s_ready.status, Status::Ready);

    // Drop below threshold for 2 checks → Regression
    env.add_legacy_messages(env.tenant_id(), 300).await; // ratio drops
    let s_first_drop = check_tenant(&env.pool, env.tenant_id()).await.unwrap();
    assert_eq!(s_first_drop.status, Status::NotReady);
    let s_regression = check_tenant(&env.pool, env.tenant_id()).await.unwrap();
    assert_eq!(s_regression.status, Status::Regression);
    let alert = env.obs.latest_alert().await;
    assert_eq!(alert.severity, "SEV-2");
    assert_eq!(alert.kind, "chat_decommission_regression");
}
```

### AC #20 — Snooze

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac20_snooze_skips_checks() {
    let env = TestEnv::new().await;
    let until = chrono::Utc::now() + chrono::Duration::days(30);
    snooze::snooze(&env.pool, env.tenant_id(), until).await.unwrap();
    let s = check_tenant(&env.pool, env.tenant_id()).await.unwrap();
    assert_eq!(s.status, Status::Snoozed);
    let snooze_audit = env.memory.last_of_kind("chat.decommission_snoozed").await.unwrap();
    assert!(snooze_audit["payload"]["snoozed_until"].is_string());
}
```

### AC #21 — recommended_action

```rust
#[rstest]
#[case(Status::InsufficientData, 0.0,  "Insufficient data; continue normal operation")]
#[case(Status::NotReady,         0.3,  "Substantial legacy traffic; do not decommission")]
#[case(Status::NotReady,         0.8,  "Migration in progress; monitor")]
#[case(Status::Approaching,      0.96, "Near decommission threshold; review with stakeholders")]
#[case(Status::Ready,            0.96, "Safe to decommission legacy sources; coordinate with operations")]
#[case(Status::Regression,       0.78, "Legacy traffic returned; investigate cause before decommissioning")]
fn ac21_recommended_action(#[case] s: Status, #[case] r: f64, #[case] expected: &str) {
    assert_eq!(recommend(s, r), expected);
}
```

### AC #22 — last_legacy_message_at

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac22_last_legacy_ts_populated() {
    let env = TestEnv::new().await;
    env.seed_messages(env.tenant_id(), 1000, 5, 0).await; // 5 slack
    let s = check_tenant(&env.pool, env.tenant_id()).await.unwrap();
    assert!(s.last_legacy_message_at.is_some());

    // Pure chat tenant.
    env.purge_legacy_messages(env.tenant_id()).await;
    env.add_chat_messages(env.tenant_id(), 100).await;
    let s2 = check_tenant(&env.pool, env.tenant_id()).await.unwrap();
    assert_eq!(s2.last_legacy_message_at, None);
}
```

### AC #23 — state-changed audit

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac23_state_change_audits() {
    let env = TestEnv::new().await;
    env.seed_messages(env.tenant_id(), 800, 200, 0).await; // 0.80 NotReady
    check_tenant(&env.pool, env.tenant_id()).await.unwrap();
    env.purge_legacy_messages(env.tenant_id()).await;
    env.add_messages_with_ratio(env.tenant_id(), 0.97, 1000).await;
    check_tenant(&env.pool, env.tenant_id()).await.unwrap();
    let changes = env.memory.count_rows("chat.decommission_state_changed").await;
    assert!(changes >= 1, "expected ≥1 state-changed row, got {}", changes);
}
```

### AC #24 — 30-day trend

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac24_ratio_history_in_payload() {
    let env = TestEnv::new().await;
    for i in 0..30 {
        env.set_simulated_date(chrono::Utc::now() - chrono::Duration::days(30 - i)).await;
        env.seed_messages_with_ratio(env.tenant_id(), 0.5 + (i as f64) * 0.015).await;
        check_tenant(&env.pool, env.tenant_id()).await.unwrap();
    }
    let s = check_tenant(&env.pool, env.tenant_id()).await.unwrap();
    assert_eq!(s.ratio_history.len(), 30);
    assert!(s.ratio_history.first().unwrap().ratio < s.ratio_history.last().unwrap().ratio,
        "ratio trend should be increasing");
}
```

### AC #25 — no spam on stable status

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac25_no_spam_on_stable_status() {
    let env = TestEnv::new().await;
    env.seed_messages(env.tenant_id(), 800, 200, 0).await; // 0.80 NotReady
    for _ in 0..5 { check_tenant(&env.pool, env.tenant_id()).await.unwrap(); }
    let changes = env.memory.count_rows("chat.decommission_state_changed").await;
    assert_eq!(changes, 1, "should be 1 initial state-change, not 5");
}
```

### derive_status pure-function

```rust
#[test]
fn derive_status_table() {
    use Status::*;
    let no_prior: Option<DecommissionState> = None;
    assert_eq!(derive_status(50,  0.95, 0.95, &no_prior), (InsufficientData, 0, 0));
    assert_eq!(derive_status(100, 0.95, 0.95, &no_prior), (Approaching, 1, 0));
    assert_eq!(derive_status(100, 0.80, 0.95, &no_prior), (NotReady, 0, 0));

    let prior_ready_2 = Some(DecommissionState {
        tenant_id: uuid::Uuid::nil(), current_status: Approaching,
        ready_streak_days: 2, regression_streak_days: 0,
        first_ready_at: None, last_check_at: chrono::Utc::now(), last_ratio: 0.96,
    });
    assert_eq!(derive_status(100, 0.96, 0.95, &prior_ready_2), (Ready, 3, 0));

    let prior_ready = Some(DecommissionState {
        tenant_id: uuid::Uuid::nil(), current_status: Ready,
        ready_streak_days: 5, regression_streak_days: 0,
        first_ready_at: Some(chrono::Utc::now()),
        last_check_at: chrono::Utc::now(), last_ratio: 0.96,
    });
    assert_eq!(derive_status(100, 0.80, 0.95, &prior_ready), (NotReady, 0, 1));
    let prior_reg_1 = Some(DecommissionState {
        current_status: NotReady, regression_streak_days: 1,
        ..prior_ready.clone().unwrap()
    });
    assert_eq!(derive_status(100, 0.80, 0.95, &prior_reg_1), (Regression, 0, 2));
}
```

---

## §6 — Implementation skeleton

The Rust modules above are the skeleton. Operational wiring:

### §6.1 — Nightly cron

Scheduler runs at 02:30 ICT (Vietnam business time + post overnight maintenance). Implementation: `tokio-cron-scheduler` inside the chat-importer service process. Each tenant gets one check per night. Failed checks are logged + retried next night (no in-day retry).

### §6.2 — Check ordering across tenants

When multiple tenants need checks, they run sequentially (not parallel) to avoid load spikes on the chat DB. Each check is fast (~100ms) so 1000 tenants = ~2min total runtime.

### §6.3 — State table vs derive-from-history

We persist `cyberos_chat_decommission_state` to avoid recomputing streak from the audit row history on every check. Streak math depends on prior status; reading the prior row from memory would couple read latency to memory availability. Local state keeps the check independent.

### §6.4 — Transition detection

Status transitions are computed locally (compare current vs. prior persisted state). The state-changed audit row fires AFTER the new state is persisted, so the memory trail and DB state stay consistent.

### §6.5 — Snooze interaction with checks

Snoozed tenants short-circuit at the top of `check_tenant` — no DB query, no audit row, no metric. The CLI command `decommission status --tenant <id>` still works, returning the persisted Snoozed status.

### §6.6 — `recommended_action` evolution

The recommendation strings are pure functions of `(status, ratio)`. New statuses or thresholds require recomputing `recommend`. We keep it small and human-curated (not LLM-generated) because operators expect deterministic text.

### §6.7 — Ratio history performance

The 30-day history is fetched from memory audit rows via a daily-aggregate query. We use `DISTINCT ON (date)` to deduplicate intra-day checks (rare, but possible if CLI is invoked manually). Performance: < 50ms for a tenant with 30d of nightly checks.

### §6.8 — CLI surface

```text
$ cyberos-chat decommission check --tenant <id>
{
  "tenant_id": "...", "status": "ready", "ratio": 0.962, ...
}

$ cyberos-chat decommission status --tenant <id>
ratio:            0.962  (threshold: 0.95)
streak:           14 days at ready
first ready at:   2026-05-02T02:30Z
last legacy msg:  2026-05-15T18:42Z
recommended:      Safe to decommission legacy sources

$ cyberos-chat decommission snooze --tenant <id> --until 2026-06-01
✓ snoozed until 2026-06-01

$ cyberos-chat decommission unsnooze --tenant <id>
✓ unsnoozed
```

### §6.9 — Failure routing

| Failure | Audit | Operator action |
|---|---|---|
| DB query fail | chat.decommission_check_failed (SEV-3) | Investigate DB |
| memory audit emit fail | logged + counter | Restore memory |
| State persist fail | logged; next check overwrites | Investigate |
| OBS alert fail | logged | Restore OBS |
| Snooze date in past | reject at CLI | Operator fixes |

### §6.10 — Source-weight extension (slice 4+)

The `decommission_source_weights` JSONB column is reserved for slice-4+ implementation. Current code reads but ignores it; future implementation would multiply per-source counts by weights before computing ratio. Documented in §1 #19 as a contract; behaviour wired in current code is `weight=1.0` for all sources.

---

## §7 — Dependencies

- **TASK-CHAT-006** — `imported_slack` source flag.
- **TASK-CHAT-007** — `imported_zalo` source flag.
- **TASK-OBS-007** — sev-3 routing.

---

## §8 — Example payloads

### `chat.decommission_signal` — Ready

```json
{
  "kind": "chat.decommission_signal",
  "ts_ns": 1747407137000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "tenant_id":              "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
    "status":                 "ready",
    "ratio":                  0.962,
    "threshold":              0.95,
    "chat_count":             9620,
    "slack_count":            360,
    "zalo_count":              20,
    "slack_active_users":     2,
    "zalo_active_users":      1,
    "total":                  10000,
    "window_start":           "2026-05-02T02:30:00Z",
    "window_end":             "2026-05-16T02:30:00Z",
    "ready_streak_days":      3,
    "regression_streak_days": 0,
    "last_legacy_message_at": "2026-05-15T18:42:00Z",
    "recommended_action":     "Safe to decommission legacy sources; coordinate with operations",
    "ratio_history": [
      {"date": "2026-04-17", "ratio": 0.82},
      {"date": "2026-04-18", "ratio": 0.83},
      {"date": "2026-05-15", "ratio": 0.96},
      {"date": "2026-05-16", "ratio": 0.962}
    ]
  }
}
```

### `chat.decommission_signal` — Regression

```json
{
  "kind": "chat.decommission_signal",
  "ts_ns": 1747839337000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "status":                 "regression",
    "ratio":                  0.78,
    "threshold":              0.95,
    "chat_count":             7800,
    "slack_count":            2200,
    "zalo_count":             0,
    "regression_streak_days": 2,
    "ready_streak_days":      0,
    "recommended_action":     "Legacy traffic returned; investigate cause before decommissioning"
  }
}
```

### `chat.decommission_state_changed`

```json
{
  "kind": "chat.decommission_state_changed",
  "ts_ns": 1747407137100000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "from":  "approaching",
    "to":    "ready",
    "ratio": 0.962
  }
}
```

### `chat.decommission_snoozed`

```json
{
  "kind": "chat.decommission_snoozed",
  "ts_ns": 1747407100000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "snoozed_until": "2026-06-15T00:00:00Z",
    "snoozed_by":    "ops@cyberskill.world",
    "reason":        "tenant in M&A; deferring decommission decision"
  }
}
```

### SEV-2 alert — Regression

```json
{
  "alert_kind": "chat_decommission_regression",
  "severity": "SEV-2",
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "title": "Decommission status regressed for tenant ACME",
  "body":  "Tenant was Ready for 14 days, now NotReady for 2 consecutive checks. Current ratio 0.78. Investigate cause.",
  "url":   "https://obs.cyberos.world/dashboard/chat_decommission?tenant_id=1f8c4d6e-..."
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Per-channel decommission signal — slice 4+; granular.
- User-level decommission ("Alice still on Slack") — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Tenant with 0 messages | InsufficientData | Status: InsufficientData; ratio 0 | None |
| Brand new tenant (< 14d) | window starts before tenant existed | InsufficientData typically | None |
| Tenant created today, 1000 messages | window covers tenant existence | Status computed; possibly Approaching/Ready | None |
| Spike of legacy imports mid-window | NotReady persists | Operator sees ratio drop; expected | None |
| Spike of chat after week of inactivity | ratio jumps | Status updates correctly; no special handling | None |
| memory audit emit fails | row not emitted; counter increments | check still completes; state persisted | Operator restores memory; manual backfill |
| RLS bypass | RLS policy | 0 rows returned; InsufficientData | Operator investigates RLS |
| Concurrent checks (CLI + cron at same time) | DB write race | one wins; second is no-op (idempotent) | None |
| Clock skew on chat DB | window edges shift slightly | minor (≤ 1m) drift in counts | None |
| Source flag missing on imported posts | counted as chat | false-high ratio | Operator audits import pipeline |
| Source flag wrong value | unknown source treated as chat | false-high | Operator fixes import code |
| OBS alert dedup | transition-only firing | No spam during sustained status | None |
| OBS alert burst on multi-tenant transition (50 tenants Ready same day) | 50 SEV-3 alerts | digest grouping (TASK-OBS-007) | None |
| Snooze date passed but state still says Snoozed | check_tenant re-evaluates | next check returns real status | None |
| Snooze date in past at snooze time | reject at CLI | Operator | None |
| Snooze without operator email | log + accept | audit row missing snoozed_by | None |
| Tenant deleted mid-check | check_tenant queries empty | Status: InsufficientData | None |
| 30-day history query returns < 30 rows | array shorter | observed as-is | None |
| Ratio_history fetch fails | logged; payload omits history | check still completes | None |
| State persist race (two checks same instant) | last-writer-wins | ready_streak_days off-by-1 | None |
| Regression streak resets if ratio briefly recovers | reset on ratio ≥ threshold | streak counter restarts | None |
| Per-tenant threshold = 1.0 (unreachable) | always NotReady | as-configured | Operator |
| Per-tenant threshold = 0.0 (always pass) | always Approaching → Ready in 3 days | as-configured | Operator |
| Source weights produce ratio > 1 (edge case) | clamp to 1.0 | Status: Ready | None |
| Active-user count > total messages (corruption) | sanity check warns | Status computed normally | Operator investigates |
| Window includes day-savings-time boundary | window math handles via UTC | None | None |
| Tenant with 1B messages | counts return i64::MAX-safe | ratio computed normally | None |
| chat-importer service restart during cron tick | tick lost; next tick succeeds | one missed check per restart | None |
| State row constraint violation | shouldn't happen; UPSERT | logged + retry | Operator |
| memory history query > 30s (rare) | timeout | history empty | None |
| Two snoozes for same tenant | second overwrites | latest wins | None |
| Snoozed tenant has SEV-2 regression upstream | snooze blocks signal | operator informed via separate channel | None |
| Manual `decommission check` during snooze | returns Snoozed status | informational | None |
| Tenant has only Zalo, no Slack | zalo_active_users populated; slack=0 | works | None |
| Brand-new tenant on day 1: ratio undefined | total=0 → InsufficientData | safe | None |
| Tenant trial expiry while in Approaching | state persists until next check | next check may return InsufficientData if data purged | Operator |
| Time zone confusion (cron 02:30 local) | always UTC for window math | None | None |
| Concurrent snooze + cron check | snooze write commits before check reads | rare interleave; last write wins | None |
| `decommission_threshold` updated mid-check | check uses value at check start | next check uses new value | None |
| Source flag enum extension (e.g. `imported_teams`) | counted as chat (catch-all) | false-high | Update derive_status when adding source |

---

## §11 — Implementation notes

- Source flag `props.cyberos_source` is set by TASK-CHAT-006 + TASK-CHAT-007 at import time. The check treats anything not matching `imported_%` as native chat.
- Nightly cron via `tokio-cron-scheduler` in the chat-importer service. We considered a separate decommissioner service but the importer is already always-on; piggyback is simpler.
- Signal is gauge (current ratio) + counter (status per tick); operator dashboards plot both as time-series.
- Transition detection: persisted in `cyberos_chat_decommission_state` not derived from memory history. Local state is faster and decouples decommission logic from memory availability.
- Window math uses chat-table `create_at` (system insert time); not Slack's `original_ts` — operator decisions are about CURRENT activity, not historical.
- The 3-consecutive-checks gate was calibrated against operator interviews: single-day spikes (vacation, all-hands meeting, conference) trip false-positive Ready signals; 3 days = enough stability to confirm.
- The 2-check Regression threshold (vs. 3 for Ready) is asymmetric: Regression is a stronger signal because it means a previously-stable migration is reversing. Catching it faster matters more than catching Ready faster.
- We chose `Approaching` as a distinct status (rather than "Ready, streak=N") so operator dashboards can show "tenants approaching decommission" as a category — useful for proactive outreach.
- Snooze is per-tenant, not per-status: snoozing applies to all checks regardless of current status. Operators sometimes snooze a brand-new tenant to skip premature signals.
- The `last_legacy_message_at` field is computed via `MAX(create_at) FILTER (... imported_%)`. Postgres handles the FILTER efficiently with a partial index on `(create_at) WHERE props->>'cyberos_source' LIKE 'imported_%'`.
- The 30-day history query uses `DISTINCT ON (date)` to deduplicate multiple same-day checks (CLI invocations).
- `recommend()` is a pure function of `(status, ratio)`; we considered LLM-generated recommendations but operators value determinism — they need to recognise the text and trust the meaning.
- `chat_decommission_status_total` counter is labeled by status; cardinality is bounded (6 statuses) — safe for long-term storage.
- The check is read-only against `posts` (no UPDATE/DELETE); RLS-friendly + safe to run during business hours if needed.
- We chose 14 days (not 7 or 30) for the rolling window because: 7 days catches weekly cycles but not full migration arc; 30 days is so long that recent improvement is masked by old data.
- The CLI's `snooze --until` takes a date, not a duration ("--for 30d"). Date is unambiguous; duration depends on "from when" semantics.
- Per-source weights are documented but not implemented in P1 — slice 4+ adds the weight multiplication in `derive_status`.
- Transition alerts are SEV-3 for Ready (informational) but SEV-2 for Regression (actionable). This asymmetry reflects: "Ready" is a planning trigger ("schedule the decommission"); "Regression" is an investigation trigger ("why did legacy return?").
- The CLI `status` subcommand reads from `cyberos_chat_decommission_state` (the persisted state) rather than running a fresh check — that's faster and operator-friendly (they want the last status, not a new one).
- We considered emitting the signal continuously (every hour) instead of nightly, but nightly is sufficient (decisions are made at human cadence, not hourly) and reduces memory audit volume.
- `recommended_action` strings are intentionally specific (not "consider X" but "do X"). Operators reading the signal want guidance, not options.
- The 30-day history in the payload is bounded; very old tenants don't ship more than 30 entries.
- The `ratio_history` is a snapshot of past checks, not a real-time computation; if checks were missed, the array has gaps — that's intentional (the gaps themselves are signal).
- We don't persist the `recommended_action` in the state row because it's a derived value; recomputing from status is cheap and lets us update the text without DB migration.
- For tenants with multiple workspaces imported (TASK-CHAT-006/007 workspace_id), the signal aggregates across workspaces. Per-workspace decommission is slice-4+.
- The `regression_streak_days` resets when status goes back to Approaching/Ready; doesn't accumulate across non-contiguous regression periods.
- The state machine is: InsufficientData ↔ NotReady ↔ Approaching → Ready → (Regression → NotReady → ...). Snoozed is an overlay that suspends transitions.

---

*End of TASK-CHAT-010.*
