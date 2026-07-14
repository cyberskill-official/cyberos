---
task_id: TASK-INV-002
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 7
template: engineering-spec@1
---

INV multi-currency with daily SBV/ECB FX snapshot + per-invoice currency lock + cross-currency reporting. 440 lines, 11 §1 clauses, 22 ACs, 3 tests, 13 failure modes, 6 notes. 7 issues resolved (SBV scrape failure → ECB fallback, manual override CFO-gated with 7-day TTL, snapshot idempotency via UNIQUE(snapshot_date, currency_pair), invoice currency immutability after issue, FX missing date → nearest-prior with sev-2 audit, report determinism via as-of date locking, memory-111 PII scrub on amounts). **Score = 10/10.**

*End of TASK-INV-002 audit.*
