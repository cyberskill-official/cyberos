---
id: NFR-OKR-006
title: "OKR cycle-phase advance correctness — cycle phases MUST advance per declared schedule"
module: OKR
category: reliability
priority: MUST
verification: T
phase: P0
slo: "100% of cycles advance phase per schedule; 0 stuck or auto-jumped phases"
owner: CEO
created: 2026-05-18
related_tasks: [TASK-OKR-007]
---

## §1 — Statement (BCP-14 normative)

1. OKR cycles **MUST** advance through declared phases: `draft → active → mid-cycle-check → final-check → retro-draft → closed` on the declared schedule.
2. Phase transitions **MUST** be either time-triggered (per schedule) or operator-triggered (early advance); auto-jumping multiple phases is forbidden.
3. Each transition **MUST** emit an audit row.
4. Stuck phase (no advance > schedule + 7 days) **MUST** trigger sev-3 alert to objective owners.
5. The retro draft (`TASK-OKR-007`) **MUST** be available at `final-check → retro-draft` transition.

## §2 — Why this constraint

Cycle phases drive expectations (when do we check in, when do we close out). Skipping or stalling silently fragments the cadence. The audit row + alert combination makes the cycle observable. The single-step advance rule prevents bugs from sending a cycle straight from draft to closed.

## §3 — Measurement

- Counter `okr_cycle_phase_transition_total{from, to}`.
- Counter `okr_cycle_stuck_phase_alert_total`.
- Histogram `okr_cycle_phase_duration_days{phase}`.

## §4 — Verification

- Integration test (T) — drive cycle through phases; assert audit rows + correct ordering.
- Time-trigger test (T) — clock advance triggers transition.
- Property test (T) — multi-step jump → reject.

## §5 — Failure handling

- Stuck phase → sev-3 alert.
- Multi-step jump → sev-2; investigate.
- Phase missed transition → operator manual advance.

---

*End of NFR-OKR-006.*
