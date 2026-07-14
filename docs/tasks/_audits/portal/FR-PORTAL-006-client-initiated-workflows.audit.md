---
task_id: TASK-PORTAL-006
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

## §1 — Verdict summary

The spec lands client-initiated workflows bridging PORTAL submissions → CHAT threads with auto-routing + SLA monitoring + security-keyword auto-escalation. 730 lines, 20 §1 normative clauses, 20 ACs, 6 verification tests, 22 failure-mode rows, 10 implementation notes. 2 migrations, 6 endpoints, 6 memory audit kinds.

6 issues resolved.

## §2 — Findings (all resolved)

### ISS-001 — Status state machine invalid-transition guard missing

§10 row mentions "Status state machine invalid transition" but the spec didn't define valid transitions. Resolved: §11.3 defines: `submitted → acknowledged → in_progress → resolved → closed`; `escalated` from any non-terminal; reopen → awaiting_client.

### ISS-002 — Security keyword review cadence

§11.2 says "review quarterly". But who reviews? Operator process. Resolved: documented as CCO ops responsibility; slice 3 may add config UI.

### ISS-003 — Concurrent status updates optimistic lock

§10 row mentions but no implementation detail. Resolved: `updated_at` column used as optimistic lock token; UPDATE includes `AND updated_at = $expected`.

### ISS-004 — Bridge failure during CHAT thread creation leaves orphan workflow

§10 row "Workflow with no chat_thread_id" notes. Resolved: background job retries thread creation; workflow visible in degraded state to engagement_admin who can manually create thread.

### ISS-005 — Submitter loses access mid-workflow via SCIM deprovision

§10 row covers. Engagement team continues internally; submitter audit history preserved. Documented as acceptable behaviour.

### ISS-006 — Escalation reason free-text — PII risk

§3 schema has `escalation_reason TEXT`. If it includes user-mentioned details, PII leaks to audit. Resolved: §11 — escalation_reason captured at sev-1 audit but hashed via TASK-MEMORY-111 before chain commit; raw retained in DB only.

## §3 — Resolution

All 6 mechanical concerns addressed.

The 730-line length is appropriate for a 6h-effort FR with focused but bidirectional scope.

**Score = 10/10.**

---

*End of TASK-PORTAL-006 audit.*
