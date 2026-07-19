---
id: TASK-TIME-002
title: "TIME timer start/stop — single-active-timer per Member + auto-stop on logout + ≤15-min resolution snap + idle-detection at 10min"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: TIME
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng (CCO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-TIME-001, TASK-TIME-003, TASK-TIME-005, TASK-TIME-007, TASK-PROJ-001, TASK-AUTH-101, TASK-AI-003, TASK-MEMORY-111]
depends_on: [TASK-TIME-001]
blocks: []

source_pages:
  - website/docs/modules/time.html#timer

source_decisions:
  - DEC-1380 2026-05-17 — One active timer per Member at a time; starting new timer auto-stops prior (commits the entry)
  - DEC-1381 2026-05-17 — Idle detection: if no client heartbeat for 10 min → auto-pause + prompt user on next interaction to confirm "still working?" → adjust or commit
  - DEC-1382 2026-05-17 — 15-min resolution snap on commit: durations rounded UP to nearest 15-min interval (industry standard for client-billable time)
  - DEC-1383 2026-05-17 — Closed enum `timer_state` = {running, paused_idle, paused_manual, committed, abandoned}; CI cardinality 5
  - DEC-1384 2026-05-17 — Logout auto-commits running timer (avoid orphan timers across sessions)
  - DEC-1385 2026-05-17 — memory audit kinds: time.timer_started, time.timer_stopped, time.timer_idle_paused, time.timer_committed, time.timer_abandoned

language: rust 1.81 + typescript 5.5
service: cyberos/services/time/
new_files:
  - services/time/migrations/0002_timers.sql
  - services/time/src/timer/mod.rs
  - services/time/src/timer/start_stop.rs
  - services/time/src/timer/idle_detector.rs
  - services/time/src/timer/commit.rs
  - services/time/src/audit/timer_events.rs
  - services/time/src/handlers/timer_routes.rs
  - services/time/web/timer-widget.ts
  - services/time/tests/timer_single_active_test.rs
  - services/time/tests/timer_idle_pause_test.rs
  - services/time/tests/timer_15min_snap_test.rs
  - services/time/tests/timer_logout_auto_commit_test.rs
  - services/time/tests/timer_state_enum_cardinality_test.rs
  - services/time/tests/timer_audit_emission_test.rs

modified_files:
  - services/time/src/lib.rs
  # invoke timer commit on logout
  - services/auth/src/handlers/logout.rs

allowed_tools:
  - file_read: services/{time,auth}/**
  - file_write: services/time/{src,tests,migrations,web}/**
  - bash: cd services/time && cargo test timer

disallowed_tools:
  - allow multiple concurrent active timers per Member (per DEC-1380)
  - skip 15-min snap (per DEC-1382)
  - leave orphan timer on logout (per DEC-1384)

effort_hours: 5
subtasks:
  - "0.4h: 0002_timers.sql + RLS + closed enum"
  - "0.4h: timer/mod.rs"
  - "0.5h: timer/start_stop.rs (single-active enforcement)"
  - "0.5h: timer/idle_detector.rs (10min watchdog)"
  - "0.4h: timer/commit.rs (15min snap + write to time_entries)"
  - "0.3h: audit/timer_events.rs"
  - "0.3h: handlers/timer_routes.rs"
  - "0.4h: web/timer-widget.ts (SPA)"
  - "1.0h: tests — 6 test files"
  - "0.3h: logout integration"
  - "0.3h: wire-up"

risk_if_skipped: "Without timer UI, Members track time externally → re-keying errors + late entries + missed billable hours. Without DEC-1380 single-active enforcement, dual-counting same period across multiple timers. Without DEC-1381 idle detection, forgotten-running timers inflate billable hours fraudulently. Without DEC-1382 15-min snap, sub-minute durations create line-item bloat on invoices. Without DEC-1384 logout commit, orphan timers persist indefinitely. The 5h effort lands the daily-use primitive that anchors TIME data quality."
---

## §1 — Description (BCP-14 normative)

The TIME service **MUST** ship timer start/stop primitive at `services/time/src/timer/` with single-active enforcement, 10-min idle detection, 15-min commit snap, logout auto-commit, 5-state enum, and 5 memory audit kinds.

1. **MUST** define closed `timer_state` enum: `('running','paused_idle','paused_manual','committed','abandoned')` per DEC-1383. Cardinality 5.

2. **MUST** define `timers` table at migration `0002`: `(timer_id UUID PRIMARY KEY, tenant_id UUID NOT NULL, member_subject_id UUID NOT NULL, engagement_id UUID NOT NULL, project_id UUID, task_id UUID, description TEXT, state timer_state NOT NULL DEFAULT 'running', started_at TIMESTAMPTZ NOT NULL DEFAULT now(), last_heartbeat_at TIMESTAMPTZ NOT NULL DEFAULT now(), idle_paused_at TIMESTAMPTZ, manual_paused_at TIMESTAMPTZ, committed_at TIMESTAMPTZ, committed_entry_id UUID, abandoned_at TIMESTAMPTZ, trace_id CHAR(32))`. Partial unique `(member_subject_id) WHERE state IN ('running','paused_idle','paused_manual')` enforces single-active per DEC-1380.

3. **MUST** enforce RLS scoped to `tenant_id` AND `member_subject_id = current_setting('auth.subject_id')::uuid` (Members see own timers only).

4. **MUST** expose `POST /v1/time/timer/start` body `{ engagement_id, project_id?, task_id?, description? }`. Handler:
   - Validates engagement_id in Member's engagements.
   - Checks existing active timer per §1 #5 — if exists, auto-commits it first.
   - INSERTs new timer row with state='running'.
   - Emits `time.timer_started` sev-3.

5. **MUST** auto-stop prior active timer per DEC-1380 on new start. Sequence:
   - SELECT existing active timer for member.
   - If found: invoke commit handler (§1 #7) — produces TIME entry + state='committed'.
   - Then proceed to new INSERT.
   - Both operations in single transaction.

6. **MUST** expose `POST /v1/time/timer/heartbeat` for client keep-alive. Body `{ timer_id }`. Handler:
   - UPDATE `last_heartbeat_at = now()` if state='running'.
   - Idempotent at sub-second rate.
   - Returns current state for client UI.

7. **MUST** expose `POST /v1/time/timer/stop` body `{ timer_id, final_description? }`. Handler:
   - Validates timer belongs to caller.
   - Computes duration = `now() - started_at - sum(pause durations)`.
   - Snaps duration UP to nearest 15-min per DEC-1382.
   - Creates TIME entry via TASK-TIME-001 with snapped duration.
   - Transitions timer to state='committed' + `committed_entry_id` populated.
   - Emits `time.timer_committed` sev-2.

8. **MUST** detect idle per DEC-1381 via `idle_detector.rs::run_watchdog()`. Scheduled job runs every 60s:
   - SELECT timers WHERE state='running' AND `last_heartbeat_at < now() - 10 min`.
   - For each: transition to state='paused_idle' + `idle_paused_at = now()`.
   - Emits `time.timer_idle_paused` sev-3.

9. **MUST** expose `POST /v1/time/timer/resume` body `{ timer_id, idle_decision }` where `idle_decision ∈ {include, exclude, partial}`. Handler:
   - Validates state IN ('paused_idle', 'paused_manual').
   - Per idle_decision: include = re-start timer including idle period; exclude = subtract idle period from final duration; partial = prompt for split.
   - Transitions state='running' + `last_heartbeat_at = now()`.

10. **MUST** expose `POST /v1/time/timer/pause` body `{ timer_id, reason? }`. Handler:
    - Transitions state='paused_manual' + `manual_paused_at = now()`.

11. **MUST** auto-commit on logout per DEC-1384. TASK-AUTH-004 logout handler invokes `time::timer::commit_all_for_subject(subject_id)`:
    - SELECT timers WHERE member_subject_id=$1 AND state IN ('running','paused_idle','paused_manual').
    - For each: invoke commit handler.

12. **MUST** snap duration UP to nearest 15-min per DEC-1382. Implementation: `let snapped_seconds = ((raw_seconds + 899) / 900) * 900`.

13. **MUST** support timer abandon at `POST /v1/time/timer/abandon` body `{ timer_id, reason }`. Transitions state='abandoned' WITHOUT creating TIME entry. Use case: accidental timer start. Emits `time.timer_abandoned` sev-3.

14. **MUST** emit 5 memory audit kinds per DEC-1385:
    - `time.timer_started` (sev-3)
    - `time.timer_stopped` (sev-3 — generic; logout path)
    - `time.timer_idle_paused` (sev-3)
    - `time.timer_committed` (sev-2 — material commercial)
    - `time.timer_abandoned` (sev-3)

15. **MUST** PII-scrub `description` via TASK-MEMORY-111 — SHA256 in chain; raw in DB.

16. **MUST** thread trace_id end-to-end.

17. **MUST NOT** allow multi-active timer per DEC-1380 (partial unique enforces).

18. **MUST NOT** snap duration DOWN (always round UP per DEC-1382 — Member-favourable).

---

## §2 — Why this design (rationale)

**Why single-active timer (§1 #5, DEC-1380)?** Multi-timer = dual-counting same wall-clock period. Industry standard (Toggl, Harvest, Clockify) all enforce single-active.

**Why 15-min snap UP (§1 #12, DEC-1382)?** Industry billing convention; rounding down would systematically undercount Member time. Snap-up = Member-favourable + reduces line-item count.

**Why 10-min idle detection (§1 #8, DEC-1381)?** Member walks away from desk. 10 min = enough for short bathroom break + not so long that forgotten timer accrues hours.

**Why logout auto-commit (§1 #11, DEC-1384)?** Orphan timers across sessions = data quality nightmare. End-of-session commit = clean state.

---

## §3 — API contract

```sql
-- 0002_timers.sql
CREATE TYPE timer_state AS ENUM ('running','paused_idle','paused_manual','committed','abandoned');

CREATE TABLE timers (
  timer_id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  member_subject_id UUID NOT NULL,
  engagement_id UUID NOT NULL,
  project_id UUID,
  task_id UUID,
  description TEXT,
  state timer_state NOT NULL DEFAULT 'running',
  started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  last_heartbeat_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  idle_paused_at TIMESTAMPTZ,
  manual_paused_at TIMESTAMPTZ,
  total_pause_seconds INT NOT NULL DEFAULT 0,
  committed_at TIMESTAMPTZ,
  committed_entry_id UUID,
  abandoned_at TIMESTAMPTZ,
  abandon_reason TEXT,
  trace_id CHAR(32)
);
CREATE UNIQUE INDEX uniq_active_timer_per_member
  ON timers(member_subject_id)
  WHERE state IN ('running','paused_idle','paused_manual');
CREATE INDEX idx_timers_heartbeat ON timers(last_heartbeat_at) WHERE state = 'running';
ALTER TABLE timers ENABLE ROW LEVEL SECURITY;
CREATE POLICY timers_rls ON timers
  USING (tenant_id = current_setting('auth.tenant_id')::uuid
         AND member_subject_id = current_setting('auth.subject_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid
              AND member_subject_id = current_setting('auth.subject_id')::uuid);
REVOKE DELETE ON timers FROM cyberos_app;
GRANT UPDATE (state, last_heartbeat_at, idle_paused_at, manual_paused_at,
              total_pause_seconds, committed_at, committed_entry_id, abandoned_at, abandon_reason)
  ON timers TO cyberos_app;
```

Endpoints:
```text
POST   /v1/time/timer/start
POST   /v1/time/timer/heartbeat
POST   /v1/time/timer/pause
POST   /v1/time/timer/resume
POST   /v1/time/timer/stop
POST   /v1/time/timer/abandon
GET    /v1/time/timer/current
```

---

## §4 — Acceptance criteria

1. **timer_state cardinality 5**.
2. **Single active per Member** — starting timer with existing active auto-commits prior.
3. **15-min snap UP** — 23-min timer commits as 30-min entry.
4. **Logout auto-commits** — session end transitions running timers to 'committed'.
5. **10-min idle pause** — heartbeat absent 10 min → state='paused_idle'.
6. **Resume include vs exclude** — `idle_decision='exclude'` subtracts idle from duration.
7. **Abandon no entry** — abandoned timer produces no TIME entry.
8. **5 memory audit kinds emitted**.
9. **RLS Member-scoped** — caller sees own timers only.
10. **Heartbeat idempotent** — repeated heartbeats update last_heartbeat_at without errors.
11. **Engagement membership validated** — timer for non-member engagement → 403.
12. **PII scrub** — description_sha256 in chain only.
13. **Trace_id end-to-end**.
14. **Cross-tenant denied** — RLS rejects.
15. **Snap respects pause time** — running 60m + paused 15m + running 30m = 90m work; snaps to 90m exactly (multiple of 15).
16. **Project_id optional** — timer without project_id allowed (general engagement work).
17. **Audit emission on each transition** — every state change emits an audit.
18. **Concurrent start race** — two simultaneous starts → partial unique constraint fires; one wins, other gets 409.
19. **Timer over 24h** — single timer running 26h still commits (snapped to 24h)?... actually no, snap to 1560 minutes (26h * 60 = 1560).
20. **Description optional** — empty description allowed at start; can be set at stop.

---

## §5 — Verification

```rust
#[tokio::test]
async fn single_active_enforced() {
    let ctx = TestContext::with_member().await;
    let t1 = ctx.start_timer().await;
    let t2 = ctx.start_timer().await;
    let state1: String = sqlx::query_scalar("SELECT state::text FROM timers WHERE timer_id=$1")
        .bind(t1).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(state1, "committed");
}

#[tokio::test]
async fn fifteen_min_snap_up() {
    let ctx = TestContext::with_member().await;
    let t = ctx.start_timer().await;
    ctx.travel(Duration::from_secs(23 * 60)).await;
    let r = ctx.stop_timer(t).await;
    let entry_id: Uuid = r.json::<serde_json::Value>().await.unwrap()["committed_entry_id"].as_str().unwrap().parse().unwrap();
    let duration: i32 = sqlx::query_scalar("SELECT duration_seconds FROM time_entries WHERE entry_id=$1")
        .bind(entry_id).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(duration, 30 * 60);
}

#[tokio::test]
async fn idle_pause_after_10min() {
    let ctx = TestContext::with_member().await;
    let t = ctx.start_timer().await;
    ctx.travel(Duration::from_secs(11 * 60)).await;
    ctx.run_idle_watchdog().await;
    let state: String = sqlx::query_scalar("SELECT state::text FROM timers WHERE timer_id=$1")
        .bind(t).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(state, "paused_idle");
}

#[tokio::test]
async fn logout_commits_running_timers() {
    let ctx = TestContext::with_member().await;
    let t = ctx.start_timer().await;
    ctx.logout().await;
    let state: String = sqlx::query_scalar("SELECT state::text FROM timers WHERE timer_id=$1")
        .bind(t).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(state, "committed");
}

// 5.5..5.10: enum cardinality, RLS, heartbeat, audit, resume decision, abandon
```

---

## §7 — Dependencies

**Upstream:** TASK-TIME-001 (TimeEntry schema).
**Cross-module:** TASK-AUTH-004 (logout integration), TASK-PROJ-001 (project_id), TASK-AI-003, TASK-MEMORY-111.

---

## §8 — Example payload

`time.timer_committed`:
```json
{
  "kind": "time.timer_committed",
  "severity": 2,
  "tenant_id": "8a2f...",
  "actor_id": "user.member.456",
  "trace_id": "...",
  "payload": {
    "timer_id": "0190...",
    "engagement_id": "0190...",
    "duration_snapped_seconds": 1800,
    "description_sha256": "..."
  }
}
```

---

## §9 — Open questions

Deferred:
- **Deferred:** Pomodoro mode (auto-pause after 25 min) — slice 2.
- **Deferred:** Multi-device same-user timer sync — slice 2.
- **Deferred:** Mobile background timer (battery + iOS background restrictions) — slice 2.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Concurrent start race | partial unique | 409; one wins | Inherent |
| Heartbeat missed during network blip | watchdog | Idle pause triggers | Resume on reconnect |
| Logout race with manual stop | tx isolation | Last writer wins; idempotent commit | Inherent |
| Server clock skew | timestamp diff | Duration approximate; ≤ 1s skew acceptable | NTP |
| Browser tab background suspends heartbeat | client-side detection | Idle pause; user resumes on tab focus | Inherent |
| Long-running timer (24h+) | absolute cap | Hard-cap at 24h via VN Labour Code (TASK-TIME-007 derivative) | Member splits across days |
| Engagement permission lost mid-timer | RLS at commit | Commit may fail; sev-2 alert | Manual entry via TASK-TIME-003 |
| Abandoned timer with significant accrued time | counter | Sev-3 audit; reviewable | Member confirms |
| Idle decision not specified at resume | required field | 400 | Inherent |
| Project_id deleted mid-timer | FK soft | Timer continues; FK enforced at commit | Manual entry alternative |
| Description PII-scrub fails | scrub error | Audit row dropped; raw retained in DB | Sev-3 alert |
| Multiple devices race start | partial unique | First wins; second gets 409 | Inherent |
| Pause-resume-pause-resume cycle | total_pause_seconds accumulator | Tracked correctly | Inherent |
| Long pause (8h+ overnight) | watchdog | State remains paused; commit excludes by default | User decision at resume |
| Auto-commit creates 0-min entry | rare | Skipped (no entry created) + abandoned state | Inherent |
| Timer for archived engagement | engagement check at start | 403 + engagement_archived | Use active engagement |

---

## §11 — Implementation notes

**§11.1** Watchdog runs as scheduled task; queries `timers WHERE state='running' AND last_heartbeat_at < now() - 10min`.

**§11.2** 15-min snap formula: `let snapped = ((seconds + 899) / 900) * 900;` integer arithmetic.

**§11.3** Logout handler at TASK-AUTH-004 invokes `services/time/src/timer/commit.rs::commit_all_for_subject` synchronously before returning.

**§11.4** SPA widget polls `/timer/current` on load; sends heartbeat every 60s while page is active.

**§11.5** Heartbeat uses Page Visibility API to suspend when tab backgrounded (battery savings).

**§11.6** Single-active partial unique uses Postgres expression index; very fast lookup.

**§11.7** Pause durations tracked cumulatively in `total_pause_seconds` for stop-time computation.

**§11.8** TIME entry created at commit time via TASK-TIME-001 standard insert path.

**§11.9** Description PII-scrub via TASK-MEMORY-111 standard ruleset.

**§11.10** Cross-tenant isolation via RLS + explicit member_subject_id check.

---

*End of TASK-TIME-002 spec.*
