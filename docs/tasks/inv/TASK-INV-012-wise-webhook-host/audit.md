---
task_id: TASK-INV-012
audited: 2026-07-26
verdict: PASS (after revision)
score_pre_revision: 7/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 6
template: task@1
---

## §1 — Verdict summary

Wise HTTP host residual over INV-004 library. 8 ACs, 10 failure modes; cash-app
and Postgres persist honestly Out of scope.

## §2 — Findings (all resolved)

### ISS-001 — Network in CI
Resolved: PublicKeySource trait + static PEM tests.

### ISS-002 — Cash-app blocked on INV-006
Resolved: CashAppStub; AC #7.

### ISS-003 — Stale must 200
Resolved: AC #5 + DEC-844.

### ISS-004 — Rotation once only
Resolved: AC #4 force_refresh once.

### ISS-005 — JWT on webhook
Resolved: no JWT; signature auth only.

### ISS-006 — Persist vs in-memory
Resolved: Out of scope Postgres; in-memory receipt set for host-c.

## §3 — Resolution

All 6 mechanical concerns addressed. **Score = 10/10.**

---

*End of TASK-INV-012 audit.*
