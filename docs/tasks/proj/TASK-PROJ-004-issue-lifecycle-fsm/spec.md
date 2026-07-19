---
id: TASK-PROJ-004
title: "Issue lifecycle FSM — backlog → todo → in-progress → in-review → done | cancelled with TASK-PROJ-002 audit trail, validation, and forward-only enforcement"
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
related_tasks: [TASK-PROJ-001, TASK-PROJ-002, TASK-PROJ-003, TASK-PROJ-008, TASK-MEMORY-101]
depends_on: [TASK-PROJ-001, TASK-PROJ-002]
blocks: [TASK-PROJ-008, TASK-PROJ-012]

source_pages:
  - website/docs/modules/proj.html#lifecycle
  - website/docs/runbooks/proj-lifecycle-runbook.html
source_decisions:
  - DEC-250 (issue lifecycle is a strict FSM; arbitrary status mutations forbidden by construction)
  - DEC-251 (backlog → done is the canonical 5-state pattern; teams using extended states must subclass via state_groups)
  #6; companion columns track when/who)
  - DEC-252 (transitions are LWW-scalar via TASK-PROJ-003 §1
  - DEC-253 (re-open from done/cancelled is explicit transition with reason; not implicit)

language: rust 1.81 + typescript 5.4
service: cyberos/services/proj-sync/
new_files:
  - services/proj-sync/src/lifecycle/mod.rs
  - services/proj-sync/src/lifecycle/fsm.rs
  - services/proj-sync/src/lifecycle/transitions.rs
  - services/proj-sync/migrations/0004_issue_status_history.sql
  - services/proj/tests/status_fsm_test.rs
  - services/proj/tests/audit_row_test.rs
  - web/proj-client/src/lifecycle/StatusPicker.tsx
  - web/proj-client/src/lifecycle/allowed_transitions.ts
modified_files:
  # status PATCH uses FSM validator
  - services/proj-sync/src/scalar_handlers.rs
  # IssueStatus enum
  - services/proj-sync/src/types.rs
  # status field uses StatusPicker
  - web/proj-client/src/components/IssueEditor.tsx
allowed_tools:
  - file_read: services/proj-sync/**, web/proj-client/**
  - file_write: services/proj-sync/{src,tests,migrations}/**, web/proj-client/src/**
  - bash: cd services/proj-sync && cargo test lifecycle
disallowed_tools:
  - skip FSM validation on status PATCH (per DEC-250)
  - allow re-open without explicit `reason` field (per DEC-253)
  - hardcode state names in TS or Rust without sharing single source (would drift)

effort_hours: 5
subtasks:
  - "0.5h: 0004_issue_status_history.sql migration (status transitions log table)"
  - "0.5h: IssueStatus enum (backlog | todo | in_progress | in_review | done | cancelled)"
  - "0.5h: fsm.rs — allowed_transitions table; is_legal_transition(from, to) -> bool"
  - "1.0h: transitions.rs — apply_transition(issue, from, to, by, reason?) -> Result<TransitionApplied, FsmError>"
  - "0.5h: scalar_handlers.rs integration — status PATCH calls FSM validator before LWW write"
  - "0.5h: memory audit row 'proj.issue_status_changed' on every transition"
  - "0.5h: TS allowed_transitions.ts mirror (generated from Rust via build.rs)"
  - "0.5h: StatusPicker.tsx — disables illegal options based on current status"
  - "0.5h: fsm_test.rs — every legal pair + every illegal pair (5×5 grid)"
  - "0.5h: transition_e2e_test.rs — PATCH → audit row → companion columns updated"
risk_if_skipped: "Without FSM, users can jump backlog → done (skipping in_progress), which breaks downstream rollups (no in-progress time recorded), velocity tracking (Δstate count wrong), and audit trail (no review evidence). Re-opens without reason silently revive done tasks; auditors investigating 'why was this re-opened' have no answer. Drift between client + server FSM definitions = UX breakage (UI offers option server rejects). Hardcoded state names in TS and Rust separately → adding a new state requires synchronised PRs across two languages; typos are easy."
---

## §1 — Description (BCP-14 normative)

The issue lifecycle **MUST** be a strict FSM with the following 5 states and 13 legal transitions:

```
   ┌────────┐   ┌──────┐   ┌─────────────┐   ┌───────────┐   ┌─────┐
   │backlog ├──▶│ todo ├──▶│ in_progress ├──▶│ in_review ├──▶│done │
   └────────┘   └──────┘   └─────────────┘   └───────────┘   └─────┘
       │           │              │                 │            ▲
       │           │              │                 │            │
       │           │              ▼                 ▼            │
       │           │      ┌─────────────┐    ┌─────────────┐     │
       └──────────────────┤  cancelled  │◀───┤             │     │
                          └─────────────┘    └─────────────┘     │
                                                  │              │
                                  (re-open with reason; ──┐      │
                                   from done|cancelled    │      │
                                   to in_progress)        └──────┘
```

1. **MUST** define `IssueStatus` enum with exactly 6 variants: `Backlog`, `Todo`, `InProgress`, `InReview`, `Done`, `Cancelled`. No other variants permitted in v1.
2. **MUST** allow ONLY these transitions:
    - **Forward**: `Backlog → Todo`, `Backlog → InProgress`, `Todo → InProgress`, `Todo → Cancelled`, `InProgress → InReview`, `InProgress → Cancelled`, `InProgress → Done`, `InReview → Done`, `InReview → InProgress` (revisions), `InReview → Cancelled`.
    - **Re-open**: `Done → InProgress`, `Cancelled → InProgress`. BOTH require non-empty `reason` field (per DEC-253).
    - **No-op**: same-state → same-state (allowed; treated as `Idempotent` not Transition).
   All other pairs MUST be rejected with `FsmError::IllegalTransition { from, to, allowed_from }` where `allowed_from` is the list of legal targets from current state.
3. **MUST** validate transitions server-side BEFORE the LWW scalar write. Validation order:
    1. FSM check (this task).
    2. LWW timestamp check (TASK-PROJ-003 §1 #6).
    3. Apply.
   FSM failure → 422 UNPROCESSABLE_ENTITY with body `{"error":"illegal_transition","from":<s>,"to":<s>,"allowed_to":[...]}`.
4. **MUST** persist transition history in `issue_status_history` (per-tenant, append-only):
    - `(issue_id, transition_seq, from_status, to_status, by_subject_id, reason, transitioned_at_ns)`.
    - `transition_seq` is monotonic per issue.
    - `reason` REQUIRED for re-open transitions (Done|Cancelled → InProgress); SHOULD-be-empty otherwise (operators MAY annotate non-re-open transitions for context).
5. **MUST** emit `proj.issue_status_changed` memory audit row per accepted transition with payload `{issue_id, from, to, by_subject_id, reason, transition_seq, transitioned_at_ns, trace_id}`.
6. **MUST** support `cyberos issue history <id>` CLI returning the full transition log for an issue (operator + auditor view).
7. **MUST** expose REST endpoint `POST /api/proj/issues/:id/transition` with body `{to: IssueStatus, reason?: string}`. Returns 200 with `{transition_seq, applied_at_ns, current_status}` or 422 with FSM error.
8. **MUST** share the FSM definition between Rust + TypeScript via build-time codegen:
    - `build.rs` writes `web/proj-client/src/lifecycle/allowed_transitions.ts` from the Rust source-of-truth.
    - CI gate (`ts-fsm-fresh`) asserts the generated file matches `cargo build`'s output (no drift).
9. **MUST** track per-status time-in-state for analytics (downstream of TASK-PROJ-013 estimate calibration):
    - On transition out of status X, compute `elapsed_ns = transitioned_at_ns - prior_transitioned_at_ns_for_this_issue`.
    - Emit metric `proj_issue_time_in_status_seconds{from_status}` (histogram).
10. **MUST** emit OTel span `proj.issue.transition` with attributes `issue_id`, `from`, `to`, `had_reason`, `transition_seq`, `duration_ms`.
11. **MUST** emit OTel metrics:
    - `proj_issue_transitions_total{from, to}` (counter; bounded cardinality 5×6=30).
    - `proj_issue_transitions_rejected_total{reason}` (counter; reason ∈ illegal_transition | reason_required | stale_write | unauthorised).
    - `proj_issue_reopens_total` (counter — operator visibility into re-open frequency per tenant).
12. **MUST** emit `proj.issue_reopened` row (separate kind from generic status_changed) on every Done|Cancelled → InProgress transition, with payload `{issue_id, from, reason, by_subject_id, reopen_seq, trace_id}` so dashboards can pivot on re-open patterns.
13. **SHOULD** support per-engagement extended states via `state_groups[]` config (slice-3+; placeholder). v1 only ships the canonical 6.

---

## §2 — Why this design (rationale for humans)

**Why FSM not free-form (§1 #2, DEC-250)?** Free-form status = chaos. Velocity dashboards assume `backlog → todo → in_progress → done` progression; arbitrary jumps break them. Audit trails need to answer "did this issue go through review?" — if jumps are allowed, the answer is non-deterministic. FSM enforces the workflow at the data layer.

**Why these 6 states specifically (§1 #1)?** Linear/Notion/Jira all converge on this pattern (renamed: `unstarted → started → completed`, etc.). Our names are operator-readable. 6 states is the empirical sweet spot — 4 is too coarse (where does "ready for review" go?); 8+ is over-engineered for most teams.

**Why `cancelled` from todo OR in_progress, but not backlog (§1 #2)?** Backlog items haven't been committed-to yet; deleting > cancelling. Cancellation is a meaningful operator decision once work has started.

**Why re-open requires reason (§1 #2 + #4, DEC-253)?** Re-opens are exception cases — the team thought they were done; something changed. Auditors and managers ask "why" immediately. Capturing the reason at transition time = no later interview required.

**Why `InReview → InProgress` is legal (§1 #2)?** Reviewer requests changes; work returns to in_progress. Without this transition, reviewers have no way to send feedback; they'd flip Cancelled (wrong semantic).

**Why pre-validate before LWW (§1 #3)?** A client sending `from: todo, to: done` should fail FAST (semantic error). Going through LWW first means an old client with stale timestamp gets a 409 (stale_write) instead of the more informative 422 (illegal_transition). Order matters for UX.

**Why per-issue transition_seq (§1 #4)?** Monotonic seq lets dashboards query "give me the 5th transition of this issue" without timestamp math. Also: detects gaps (audit trail corruption).

**Why separate `proj.issue_reopened` row (§1 #12)?** Re-open frequency is a leading indicator of estimate quality + reviewer-completeness. Dashboarding it as a distinct event (vs filtering generic status_changed) makes the metric trivially queryable.

**Why TS↔Rust codegen (§1 #8)?** Two-language hardcoding drifts. Adding a state to Rust + forgetting to update TS = UI shows option, server rejects. Codegen + CI gate makes drift impossible.

**Why time-in-status metric (§1 #9)?** "Average time in `in_review`" is the leading indicator of review-team throughput. Captured at transition-out time = exact + cheap. Downstream of TASK-PROJ-013 calibration.

---

## §3 — API contract

### IssueStatus enum

```rust
// services/proj-sync/src/lifecycle/mod.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueStatus {
    Backlog,
    Todo,
    InProgress,
    InReview,
    Done,
    Cancelled,
}

impl IssueStatus {
    pub fn all() -> [Self; 6] {
        use IssueStatus::*;
        [Backlog, Todo, InProgress, InReview, Done, Cancelled]
    }
    pub fn is_terminal(self) -> bool {
        matches!(self, IssueStatus::Done | IssueStatus::Cancelled)
    }
}
```

### FSM

```rust
// services/proj-sync/src/lifecycle/fsm.rs
use crate::lifecycle::IssueStatus;
use IssueStatus::*;

/// Source of truth for legal transitions. CI gate generates TS mirror from this.
pub const LEGAL_TRANSITIONS: &[(IssueStatus, IssueStatus)] = &[
    // Forward
    (Backlog,    Todo),
    (Backlog,    InProgress),       // skip Todo (urgent tasks)
    (Todo,       InProgress),
    (Todo,       Cancelled),
    (InProgress, InReview),
    (InProgress, Cancelled),
    (InProgress, Done),              // skip review (small tasks)
    (InReview,   Done),
    (InReview,   InProgress),        // reviewer requests changes
    (InReview,   Cancelled),
    // Re-open (requires reason)
    (Done,       InProgress),
    (Cancelled,  InProgress),
];

pub fn is_legal(from: IssueStatus, to: IssueStatus) -> bool {
    if from == to { return true; }   // no-op
    LEGAL_TRANSITIONS.contains(&(from, to))
}

pub fn requires_reason(from: IssueStatus, to: IssueStatus) -> bool {
    from.is_terminal() && to == IssueStatus::InProgress
}

pub fn allowed_targets(from: IssueStatus) -> Vec<IssueStatus> {
    LEGAL_TRANSITIONS.iter()
        .filter_map(|&(f, t)| if f == from { Some(t) } else { None })
        .collect()
}
```

### Transition handler

```rust
// services/proj-sync/src/lifecycle/transitions.rs
use crate::lifecycle::{IssueStatus, fsm};

#[derive(Debug, thiserror::Error)]
pub enum FsmError {
    #[error("illegal transition: {from:?} → {to:?}; allowed: {allowed_to:?}")]
    IllegalTransition { from: IssueStatus, to: IssueStatus, allowed_to: Vec<IssueStatus> },
    #[error("reason required for re-open ({from:?} → {to:?})")]
    ReasonRequired { from: IssueStatus, to: IssueStatus },
    #[error("stale write (LWW)")]
    StaleWrite,
    #[error("database error: {0}")]
    Db(String),
}

#[derive(Clone, Debug, Serialize)]
pub struct TransitionApplied {
    pub transition_seq:        i64,
    pub applied_at_ns:         i64,
    pub current_status:        IssueStatus,
}

pub async fn apply_transition(
    pool: &sqlx::PgPool,
    issue_id: uuid::Uuid,
    requested_to: IssueStatus,
    by_subject_id: uuid::Uuid,
    reason: Option<String>,
) -> Result<TransitionApplied, FsmError> {
    // 1. Read current state (with row lock for the duration of transaction)
    let mut tx = pool.begin().await.map_err(|e| FsmError::Db(e.to_string()))?;
    let row = sqlx::query!(
        "SELECT status as \"status: IssueStatus\",
                status_updated_at_ns,
                status_updated_by_subject_id
         FROM issues WHERE id = $1 FOR UPDATE",
        issue_id
    ).fetch_one(&mut *tx).await.map_err(|e| FsmError::Db(e.to_string()))?;

    let from = row.status;

    // 2. FSM check
    if !fsm::is_legal(from, requested_to) {
        return Err(FsmError::IllegalTransition {
            from, to: requested_to,
            allowed_to: fsm::allowed_targets(from),
        });
    }

    // 3. Reason check
    if fsm::requires_reason(from, requested_to) && reason.as_deref().map(str::trim).unwrap_or("").is_empty() {
        return Err(FsmError::ReasonRequired { from, to: requested_to });
    }

    // 4. No-op short-circuit
    if from == requested_to {
        return Ok(TransitionApplied {
            transition_seq:  row.transition_seq.unwrap_or(0),
            applied_at_ns:   row.status_updated_at_ns,
            current_status:  from,
        });
    }

    // 5. Compute time-in-state BEFORE update
    let now_ns = chrono::Utc::now().timestamp_nanos_opt().unwrap();
    let time_in_state_ns = now_ns - row.status_updated_at_ns;
    metrics::histogram!("proj_issue_time_in_status_seconds", "from_status" => format!("{from:?}"))
        .record(time_in_state_ns as f64 / 1e9);

    // 6. Insert into history; update issue
    let transition_seq: i64 = sqlx::query_scalar!(
        "INSERT INTO issue_status_history
            (issue_id, transition_seq, from_status, to_status, by_subject_id, reason, transitioned_at_ns, tenant_id)
         SELECT $1,
                COALESCE((SELECT MAX(transition_seq) FROM issue_status_history WHERE issue_id = $1), 0) + 1,
                $2, $3, $4, $5, $6, current_setting('app.tenant_id')::uuid
         RETURNING transition_seq",
        issue_id, from as IssueStatus, requested_to as IssueStatus,
        by_subject_id, reason, now_ns
    ).fetch_one(&mut *tx).await.map_err(|e| FsmError::Db(e.to_string()))?;

    sqlx::query!(
        "UPDATE issues SET status = $1, status_updated_at_ns = $2, status_updated_by_subject_id = $3 WHERE id = $4",
        requested_to as IssueStatus, now_ns, by_subject_id, issue_id
    ).execute(&mut *tx).await.map_err(|e| FsmError::Db(e.to_string()))?;

    tx.commit().await.map_err(|e| FsmError::Db(e.to_string()))?;

    // 7. Emit audit rows
    emit_memory_row("proj.issue_status_changed", serde_json::json!({
        "issue_id": issue_id, "from": from, "to": requested_to,
        "by_subject_id": by_subject_id, "reason": reason,
        "transition_seq": transition_seq,
        "transitioned_at_ns": now_ns,
        "trace_id": current_trace_id(),
    })).await;
    if from.is_terminal() && requested_to == IssueStatus::InProgress {
        emit_memory_row("proj.issue_reopened", serde_json::json!({
            "issue_id": issue_id, "from": from, "reason": reason,
            "by_subject_id": by_subject_id,
            "reopen_seq": transition_seq, "trace_id": current_trace_id(),
        })).await;
        metrics::counter!("proj_issue_reopens_total").increment(1);
    }
    metrics::counter!("proj_issue_transitions_total",
        "from" => format!("{from:?}"), "to" => format!("{requested_to:?}")).increment(1);

    Ok(TransitionApplied {
        transition_seq, applied_at_ns: now_ns, current_status: requested_to,
    })
}
```

### Migration

```sql
-- services/proj-sync/migrations/0004_issue_status_history.sql

CREATE TABLE issue_status_history (
    issue_id            UUID NOT NULL,
    transition_seq      BIGINT NOT NULL,
    from_status         TEXT NOT NULL,
    to_status           TEXT NOT NULL,
    by_subject_id       UUID NOT NULL,
    reason              TEXT,
    transitioned_at_ns  BIGINT NOT NULL,
    tenant_id           UUID NOT NULL,
    PRIMARY KEY (issue_id, transition_seq)
);
CREATE INDEX idx_issue_status_history_recent ON issue_status_history (issue_id, transition_seq DESC);

CREATE POLICY issue_status_history_tenant_isolation ON issue_status_history
    USING (tenant_id = current_setting('app.tenant_id')::uuid);
```

### TypeScript mirror (codegen output)

```typescript
// web/proj-client/src/lifecycle/allowed_transitions.ts (GENERATED — do not edit)
// regenerate via `cd services/proj-sync && cargo build`

export type IssueStatus = 'backlog' | 'todo' | 'in_progress' | 'in_review' | 'done' | 'cancelled';

export const LEGAL_TRANSITIONS: readonly [IssueStatus, IssueStatus][] = [
  ['backlog',    'todo'],
  ['backlog',    'in_progress'],
  ['todo',       'in_progress'],
  ['todo',       'cancelled'],
  ['in_progress','in_review'],
  ['in_progress','cancelled'],
  ['in_progress','done'],
  ['in_review',  'done'],
  ['in_review',  'in_progress'],
  ['in_review',  'cancelled'],
  ['done',       'in_progress'],
  ['cancelled',  'in_progress'],
] as const;

export function isLegal(from: IssueStatus, to: IssueStatus): boolean {
  if (from === to) return true;
  return LEGAL_TRANSITIONS.some(([f, t]) => f === from && t === to);
}

export function requiresReason(from: IssueStatus, to: IssueStatus): boolean {
  return (from === 'done' || from === 'cancelled') && to === 'in_progress';
}

export function allowedTargets(from: IssueStatus): readonly IssueStatus[] {
  return LEGAL_TRANSITIONS.filter(([f]) => f === from).map(([, t]) => t);
}
```

### Status picker UI

```typescript
// web/proj-client/src/lifecycle/StatusPicker.tsx
import * as React from 'react';
import { IssueStatus, allowedTargets, requiresReason, isLegal } from './allowed_transitions';

interface Props {
  issueId:    string;
  current:    IssueStatus;
  onChange:   (to: IssueStatus, reason?: string) => Promise<void>;
}

export function StatusPicker({ issueId, current, onChange }: Props) {
  const [pending, setPending] = React.useState<IssueStatus | null>(null);
  const [reasonText, setReasonText] = React.useState('');

  const handleClick = async (to: IssueStatus) => {
    if (!isLegal(current, to)) return;
    if (requiresReason(current, to)) {
      setPending(to);
      return;
    }
    await onChange(to);
  };

  return (
    <div className="status-picker">
      {(['backlog','todo','in_progress','in_review','done','cancelled'] as IssueStatus[]).map(s => (
        <button key={s}
                disabled={!isLegal(current, s) && s !== current}
                aria-current={s === current}
                onClick={() => handleClick(s)}>
          {s}
        </button>
      ))}
      {pending && (
        <div className="reason-prompt">
          <label>Re-open reason (required):</label>
          <textarea value={reasonText} onChange={e => setReasonText(e.target.value)} />
          <button disabled={!reasonText.trim()} onClick={async () => {
            await onChange(pending, reasonText.trim());
            setPending(null); setReasonText('');
          }}>Re-open</button>
          <button onClick={() => { setPending(null); setReasonText(''); }}>Cancel</button>
        </div>
      )}
    </div>
  );
}
```

---

## §4 — Acceptance criteria

1. **Every legal transition succeeds** — fixture: each of 12 forward + 2 re-open transitions → 200; transition_seq increments.
2. **Every illegal transition rejected** — fixture: all 18 illegal pairs (out of 30 total) → 422 with `allowed_to` list.
3. **No-op same-state allowed** — current=todo, request todo → 200; `transition_seq` unchanged.
4. **Re-open requires reason** — done → in_progress with `reason: null` → 422 `reason_required`; with `reason: "scope creep"` → 200.
5. **Re-open with empty reason rejected** — `reason: "   "` → 422 (trimmed empty).
6. **history table populated** — after 5 transitions on issue X → 5 rows in `issue_status_history` with consecutive `transition_seq` 1..5.
7. **Companion columns updated** — `status_updated_at_ns` + `status_updated_by_subject_id` match the transition row.
8. **memory audit row per transition** — every legal transition → `proj.issue_status_changed` row.
9. **memory audit `proj.issue_reopened` on re-open** — done → in_progress → both `status_changed` AND `issue_reopened` rows.
10. **time_in_status metric recorded** — issue X transitions todo → in_progress at T+10s → histogram records 10s for `from_status="todo"`.
11. **TS mirror matches Rust** — CI `ts-fsm-fresh` runs codegen + diffs; PR rejected if drift.
12. **StatusPicker disables illegal options** — current=backlog → in_review button is disabled.
13. **StatusPicker prompts for reason on re-open** — current=done, click in_progress → reason textarea appears; submit button disabled until non-empty.
14. **`cyberos issue history <id>` CLI** — returns transition log JSON; ordered ascending by transition_seq.
15. **Concurrent transitions serialised** — two clients send transitions on same issue concurrently → second client gets stale_write (TASK-PROJ-003 LWW); history reflects single transition.
16. **OTel span emitted** — span `proj.issue.transition` with `from`, `to`, `had_reason`, `transition_seq`.
17. **OTel metric `proj_issue_transitions_total`** — labels `from` + `to` populated correctly per call.
18. **OTel metric `proj_issue_transitions_rejected_total{reason="illegal_transition"}`** — increments on illegal transition.
19. **OTel metric `proj_issue_reopens_total`** — increments on re-open only.
20. **RLS enforces tenant isolation** — tenant A cannot read/write tenant B's `issue_status_history`.

---

## §5 — Verification

```rust
// services/proj/tests/status_fsm_test.rs

#[test]
fn legal_transitions_table() {
    use IssueStatus::*;
    let expected_legal: HashSet<_> = [
        (Backlog, Todo), (Backlog, InProgress),
        (Todo, InProgress), (Todo, Cancelled),
        (InProgress, InReview), (InProgress, Cancelled), (InProgress, Done),
        (InReview, Done), (InReview, InProgress), (InReview, Cancelled),
        (Done, InProgress), (Cancelled, InProgress),
    ].into_iter().collect();
    for &(f, t) in expected_legal.iter() {
        assert!(fsm::is_legal(f, t), "expected legal: {f:?} → {t:?}");
    }
}

#[test]
fn all_illegal_pairs_rejected() {
    use IssueStatus::*;
    let mut count_illegal = 0;
    for f in IssueStatus::all() {
        for t in IssueStatus::all() {
            if f == t { continue; }
            let in_legal = matches!((f, t),
                (Backlog, Todo)        | (Backlog, InProgress)  |
                (Todo, InProgress)     | (Todo, Cancelled)      |
                (InProgress, InReview) | (InProgress, Cancelled)| (InProgress, Done) |
                (InReview, Done)       | (InReview, InProgress) | (InReview, Cancelled) |
                (Done, InProgress)     | (Cancelled, InProgress));
            if !in_legal {
                assert!(!fsm::is_legal(f, t), "expected illegal: {f:?} → {t:?}");
                count_illegal += 1;
            }
        }
    }
    assert_eq!(count_illegal, 18);
}

#[test]
fn no_op_same_state_legal() {
    for s in IssueStatus::all() {
        assert!(fsm::is_legal(s, s));
    }
}

#[test]
fn requires_reason_only_for_terminal_to_inprogress() {
    use IssueStatus::*;
    assert!(fsm::requires_reason(Done, InProgress));
    assert!(fsm::requires_reason(Cancelled, InProgress));
    assert!(!fsm::requires_reason(Done, Cancelled));   // not allowed; would short-circuit at fsm::is_legal
    assert!(!fsm::requires_reason(Backlog, Todo));
}

#[tokio::test]
async fn re_open_without_reason_rejected() {
    let env = TestEnv::new().await;
    let issue = env.create_issue_in_status(IssueStatus::Done).await;
    let res = apply_transition(&env.pool, issue, IssueStatus::InProgress, env.alice(), None).await;
    assert!(matches!(res, Err(FsmError::ReasonRequired { .. })));
}

#[tokio::test]
async fn re_open_with_reason_succeeds_and_emits_reopen_row() {
    let env = TestEnv::new().await;
    let issue = env.create_issue_in_status(IssueStatus::Done).await;
    let res = apply_transition(&env.pool, issue, IssueStatus::InProgress, env.alice(), Some("scope creep".into())).await.unwrap();
    assert_eq!(res.current_status, IssueStatus::InProgress);
    let reopen_row = env.memory.latest("proj.issue_reopened").await;
    assert_eq!(reopen_row["payload"]["reason"], "scope creep");
}

#[tokio::test]
async fn time_in_status_metric_recorded() {
    let env = TestEnv::with_paused_time().await;
    let issue = env.create_issue_in_status(IssueStatus::Todo).await;
    tokio::time::advance(Duration::from_secs(10)).await;
    let _ = apply_transition(&env.pool, issue, IssueStatus::InProgress, env.alice(), None).await.unwrap();
    let metric = env.metrics.histogram_values("proj_issue_time_in_status_seconds", &[("from_status", "Todo")]).await;
    assert!(metric.iter().any(|&v| (v - 10.0).abs() < 0.5));
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton.)

---

## §7 — Dependencies

- **TASK-PROJ-001** — `issues` table; we add `status_updated_at_ns` + `status_updated_by_subject_id` columns + new `issue_status_history` table.
- **TASK-PROJ-002** — WebSocket sync; transitions broadcast as scalar LWW updates per TASK-PROJ-003.
- **TASK-PROJ-003** — LWW companion-column pattern.
- **TASK-PROJ-008 (downstream)** — memory audit row pattern reused.
- **TASK-AUTH-003** — RLS on `issue_status_history`.
- **TASK-MEMORY-101** — audit emission.

---

## §8 — Example payloads

### `proj.issue_status_changed`

```json
{
  "kind": "proj.issue_status_changed",
  "payload": {
    "issue_id":             "iss-01HZK9R8M3X5C8Q4",
    "from":                 "in_progress",
    "to":                   "in_review",
    "by_subject_id":        "7e57c0de-1234-...",
    "reason":               null,
    "transition_seq":       4,
    "transitioned_at_ns":   1747407137483000000,
    "trace_id":             "0af7651916cd43dd8448eb211c80319c"
  }
}
```

### `proj.issue_reopened`

```json
{
  "kind": "proj.issue_reopened",
  "payload": {
    "issue_id":       "iss-01HZK9R8M3X5C8Q4",
    "from":           "done",
    "reason":         "scope expanded by stakeholder; need additional logging",
    "by_subject_id":  "7e57c0de-...",
    "reopen_seq":     7,
    "trace_id":       "0af7651916cd43dd8448eb211c80319c"
  }
}
```

### 422 illegal transition response

```json
{
  "error":     "illegal_transition",
  "from":      "backlog",
  "to":        "in_review",
  "allowed_to": ["todo", "in_progress"]
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Per-engagement extended states (state_groups) — slice 3+ (per §1 #13).
- Status presets per project template — slice 4+.
- Bulk transition API (move 50 issues at once) — slice 3+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Illegal transition | fsm::is_legal | 422 `illegal_transition` with allowed_to | Client retries with legal target |
| Reason missing on re-open | trim check | 422 `reason_required` | Client prompts user |
| Concurrent transition race | row-level lock + LWW | Second caller gets stale_write OR transition_seq collision (PK) | Client retries |
| Postgres tx commit fails | sqlx Err | 500; no audit row | Operator restores DB; client retries |
| Audit row emit fails | MemoryWriter Err | Transition succeeded; audit lost; sev-2 alarm | Operator restores memory |
| Codegen drift (TS missing new state) | CI `ts-fsm-fresh` | PR blocked | Author re-runs `cargo build` |
| Operator manually inserts row with skip | RLS doesn't prevent; FSM only checked at handler | Skips audit trail; out-of-band edit | Operator avoids manual SQL |
| Issue deleted mid-transition | SELECT … FOR UPDATE returns 0 rows | sqlx Err → 500 | Client retries (will get 404) |
| Subject_id not in tenant | RLS prevents at issue read | 404 | Caller fixes auth |
| Time-in-state computed across DB clock skew | should not happen on single node | Slight imprecision in histogram | Operator monitors |
| Transition_seq gap (manual delete from history) | analytic query detects | Auditor flag; operator investigates | Manual reconciliation |
| Same-state no-op produces transition_seq | code short-circuits | No new row; correct | None |
| Re-open from non-terminal state | fsm rejects | 422 | None — not a legal transition |
| `reason` is 1 GB | DB column TEXT (no length limit) | Stored; UI may truncate | Slice-3+ cap at 4 KB |
| RLS bypass attempt (cross-tenant transition) | RLS catches | RLS returns 0 rows → 500 / 404 | None automatic |
| Time-in-state metric cardinality | `from_status` has 6 values | Bounded; safe | None |
| Issue created with status != backlog | TASK-PROJ-001 default; bypassed by direct SQL | Allowed; history starts mid-FSM | Operator audits |
| TS picker shows transitions the server later rejects | only happens if drift; CI prevents | UX-degrading | CI catches |

---

## §11 — Implementation notes

- The Rust enum is `IssueStatus`; sqlx maps to text via `#[derive(sqlx::Type)] #[sqlx(type_name = "TEXT")]`. We use TEXT not Postgres enum because we may need to add states without migration (per §1 #13 placeholder).
- `LEGAL_TRANSITIONS` is a `&[(IssueStatus, IssueStatus)]` slice — compile-time constant; no allocation.
- The `build.rs` script reads `lifecycle/fsm.rs`, parses the LEGAL_TRANSITIONS table, and writes the TS mirror with a `// GENERATED` header. The CI gate diffs the file in repo against fresh codegen output.
- `requires_reason` is intentionally narrow: only for re-opens. Non-re-open transitions MAY carry `reason` (operator annotation) but it's not required.
- The `apply_transition` function uses `SELECT … FOR UPDATE` to serialise concurrent transitions on the same issue — prevents the FSM check from being read-skewed.
- The audit row's `reason` field is `null` for non-re-open transitions (rather than empty string) so consumers can distinguish "no annotation" from "explicit empty annotation."
- The time-in-state histogram has bounded label cardinality (6 from-statuses) — Prometheus-safe.
- The TS picker's `disabled={!isLegal(current, s) && s !== current}` keeps the current state button enabled but inert (good UX — visible self-reference).
- Re-open reason prompt is a modal/textarea in the picker; TASK-PROJ-017 (Brief modal) provides the more polished version.

---

*End of TASK-PROJ-004.*
