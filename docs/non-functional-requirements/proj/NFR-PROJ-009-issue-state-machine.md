---
id: NFR-PROJ-009
title: "PROJ issue state machine — transitions MUST conform to the declared FSM"
module: PROJ
category: reliability
priority: MUST
verification: T
phase: P0
slo: "100% of issue state transitions match the FSM; 0 illegal transitions in production"
owner: CTO
created: 2026-05-18
related_frs: [FR-PROJ-004, FR-PROJ-001]
---

## §1 — Statement (BCP-14 normative)

1. Issue state transitions **MUST** conform to the FSM declared in `modules/proj/state-machine.yaml`: states + allowed transitions + required roles.
2. Illegal transition attempts **MUST** return `E_ILLEGAL_TRANSITION` with `data.allowed_next_states = [...]`.
3. Every state transition **MUST** emit a BRAIN audit row capturing `{from, to, actor_id, transition_at, reason?}`.
4. The FSM definition **MUST** be the source of truth; UI buttons + API endpoints derive their behaviour from it.
5. FSM changes **MUST** be reviewed via PR; deployment requires migration plan for in-flight issues.

## §2 — Why this constraint

Issue states drive every dashboard, every notification, every reporting. A silently-bypassed transition pollutes all downstream views. The FSM-as-truth + UI-derives + API-rejects pattern means the rule is enforced at every layer; bypass requires lying to the FSM file itself, which is reviewed in PR. The audit row makes transitions forensically reconstructable.

## §3 — Measurement

- Counter `proj_issue_illegal_transition_total{from, to}` — must be 0.
- Counter `proj_issue_transition_total{from, to}` — full transition matrix observable.
- Audit-row count = transition counter (reconciliation).

## §4 — Verification

- Unit test (T) — drive every legal transition; assert allowed.
- Unit test (T) — illegal transitions; assert rejected.
- CI gate (T) — UI + API endpoint set matches FSM-allowed transitions.

## §5 — Failure handling

- Illegal transition observed → sev-3; FSM gate may be bypassed at one layer.
- Counter mismatch with audit row count → sev-2 audit drop; investigate.
- FSM migration during in-flight issues → sev-3; manual reconciliation may be needed.

---

*End of NFR-PROJ-009.*
