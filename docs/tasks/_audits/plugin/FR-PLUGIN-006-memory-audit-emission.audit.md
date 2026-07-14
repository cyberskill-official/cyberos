---
task_id: TASK-PLUGIN-006
audited: 2026-05-19
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 7
template: engineering-spec@1
---

## §1 — Verdict summary

memory audit emission — 6 plugin.* kinds, audit-then-respond ordering, Postgres-backed durable outbox with exponential backoff up to 24h, idempotency via SHA-256 hashed key, body scrubbing to prevent user-data leak. 470 lines, 14 §1 clauses, 22 ACs, 5 test files, 16 failure modes, 10 implementation notes. 7 issues resolved (audit-then-respond closes the lost-row window; durable outbox survives restart; hashed idempotency key dedups retries safely; locked schema body scrubs prevent user-data leak into the chain; OTel emission on emission-failure prevents silent audit-outage; subject_id-not-plugin_id as actor matches Strategy §2 attribution model; install-before-invoked ordering keeps audit chain narratively coherent). **Score = 10/10.**

## §2 — Findings (all resolved)

### ISS-001 — Lost audit rows on process crash
Audit-after-respond means a crash between response and emit loses the row. Resolved: §1 clause 2 + DEC-2451 — audit-then-respond; AC #21.

### ISS-002 — In-memory queue lost on restart
Process restart loses queued audit emits. Resolved: §1 clause 3 + DEC-2452 + Postgres outbox; AC #8.

### ISS-003 — Retry duplicates cause audit chain noise
Without idempotency, every retry produces a new memory row. Resolved: §1 clause 4 + DEC-2453 — SHA-256 composed key; memory dedups; AC #7, #22.

### ISS-004 — Tool input/output leaks into audit chain
Audit chains are read by admins + DSAR fulfillers; user content leak violates privacy. Resolved: §1 clause 6 + DEC-2455 — body strictly limited to 6-8 fields; tool input/output excluded; AC #11-13.

### ISS-005 — Silent audit-emission outages
Audit failures are the highest-stakes failure; need visibility. Resolved: §1 clause 7 + DEC-2456 — required OTel emission on success/retry/failure; AC #14-16.

### ISS-006 — Actor attribution wrong (plugin instead of subject)
Audit rows attributed to "plugin" lose human accountability. Resolved: §1 clause 8 — actor_id = JWT sub claim; plugin_id surfaces in body; AC #20.

### ISS-007 — 4xx retry storms waste resources
Retrying programming errors indefinitely never succeeds. Resolved: §1 clause 11 + clause 14 — 4xx (non-429) fails immediately; AC #17-18.

## §3 — Resolution

All 7 ISS findings resolved by extending §1 with clauses 2, 6, 7, 8, 11, 14, defining the Postgres outbox schema with RLS, locking the 6 audit kinds + 6-field body shape, and writing 5 integration tests including the leak-detection test (#11-12).

Final score: **10/10.**

*End of TASK-PLUGIN-006 audit.*
