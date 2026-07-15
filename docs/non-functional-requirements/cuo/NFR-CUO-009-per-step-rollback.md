---
id: NFR-CUO-009
title: "CUO per-step rollback — failed step MUST trigger declared compensations in reverse order"
module: CUO
category: reliability
priority: MUST
verification: T
phase: P0
slo: "100% of step failures with declared compensations execute the rollback in reverse order"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-CUO-105]
---

## §1 — Statement (BCP-14 normative)

1. When a chain step fails AND the chain declares `rollback: <skill_id>` per step, all completed steps' rollback skills **MUST** execute in strict reverse order before the chain exits.
2. Rollback execution itself **MUST** be audited — each rollback skill invocation produces its own audit row with `kind=cuo.rollback`.
3. If a rollback skill itself fails, the chain exits in `outcome=rollback_partial` state. The supervisor does NOT attempt to rollback the rollback (no infinite recursion).
4. Steps without declared rollback are skipped during rollback walk (silent — not an error).
5. Rollback walk **MUST** time-box at 60s total — beyond this, the chain exits `outcome=rollback_timeout` and an operator is paged.

## §2 — Why this constraint

Per-step rollback is the platform's saga semantics. Without strict reverse-order execution, partial side effects can be left in an inconsistent state (e.g., step 3 created a resource, step 5 failed, but step 3's rollback never ran). The "no rollback-of-rollback" rule is the bounded-recursion safety; the time-box is the operational safety. The audit rows for rollbacks ensure forensics — operators can trace exactly what compensations ran.

## §3 — Measurement

- Counter `cuo_rollback_attempt_total{step_index, result=success|failed|timeout}`.
- Histogram `cuo_rollback_walk_duration_seconds`.
- Counter `cuo_rollback_partial_outcomes_total` — surfaces incomplete compensations.

## §4 — Verification

- Integration test `modules/cuo/tests/test_rollback.py` (T) — chain with declared rollback; fail step 3; assert steps 2 + 1 roll back in that order.
- Chaos test (T) — fail the rollback skill itself; assert chain exits `rollback_partial`, no infinite loop.
- Time-box test (T) — slow rollbacks; assert 60s ceiling.

## §5 — Failure handling

- Rollback skipped without declaration → expected behaviour; no alarm.
- Rollback partial → sev-3; operator inspects audit chain to determine cleanup needed.
- Rollback timeout → sev-2; rollback skill is slow or hung; manual cleanup may be required.

---

*End of NFR-CUO-009.*
