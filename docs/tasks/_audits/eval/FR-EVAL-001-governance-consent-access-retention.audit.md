---
task_id: TASK-EVAL-001
audited: 2026-06-29
verdict: PASS
score: 10/10
issues_resolved: 12
template: engineering-spec@1
eu_ai_act_risk_class: high
authoring_md_compliance: 2026-06-29 (rule 36 — ≥6 canonical ISSes verified; task-audit skill §3.12 compliant; high-risk AI Risk Assessment section present)
governance_first_invariant: verified (capture + evaluation gated behind acknowledgment; covert collection out of scope; access-restricted + contract-disclosed)
---

## §1 — Verdict summary

TASK-EVAL-001 is the Phase-0 governance, consent, access-control, and retention gate for the whole BRAIN/EVAL workstream, authored 2026-06-29 to the engineering-spec@1 house style and Stephen's decisions DEC-2520..2525. Current scope: 18 §1 normative clauses (versioned per-tenant monitoring notice, per-subject acknowledgment, the `is_gated` capture/evaluation gate, data-category/purpose/lawful-basis registry, scope-minimisation rejection of out-of-scope categories, per-category retention policy + sweeper, founder/manager-of/self/explicit access resolution, access-grant table, read-audit-row-per-cross-person-read, data-subject-rights endpoints, human-in-the-loop for consequential actions, one chained audit row per governance mutation, the out-of-scope-covert-collection statement, tenant RLS, append-only REVOKE, OTel metrics, notice-bump re-gating, governance-status endpoint). 10 §2 rationale paragraphs plus a "not legal advice" closing note. §3 contains: the full `0001_governance.sql` migration (6 tables + RLS policies on `app.current_tenant_id` + append-only REVOKEs), the acknowledgment-gate Rust (`gate_reason` / `is_gated`), the access resolution + fail-closed read-audit (`may_read` / `guard_evaluation_read`), the shared-chain governance-audit emit (reusing `cyberos-audit-chain`), and the L2-only retention sweeper. 25 ACs. §10 lists 24 failure rows. §11 lists 13 implementation notes. The `## AI Risk Assessment` section (Data Sources / Human Oversight / Failure Modes) is present per the high EU-AI-Act risk class. depends_on [TASK-AUTH-003]; blocks [TASK-MEMORY-121, TASK-EVAL-002]. Verdict: PASS, 10/10.

## §2 — Findings (all resolved)

### ISS-001 — Wide day-1 capture has no lawful basis without a gate
Stephen's directive is monitoring the moment a person logs in. Wide capture with no notice is the covert-surveillance posture DEC-2525 forbids. Resolved: §1 #1-#3 — a versioned per-tenant notice + per-subject acknowledgment + `is_gated` precondition that BLOCKS capture and evaluation for any subject without a current-version ack; capture is *armed* day-1 but records a subject only after acknowledgment. AC #4 #5 #6.

### ISS-002 — The acknowledged text could move under the employee
A consent record is worthless if the operator can edit the notice after the fact. Resolved: §1 #1 versioning + `body_sha256`, §1 #2 ack pins `notice_sha256` to the acknowledged text, §1 #15 notice table append-only (a correction is a new version). AC #2 #3.

### ISS-003 — Collection scope could silently exceed work-interactions
Without an enforced boundary, "capture everything" drifts into keystroke/screen/private-life surveillance. Resolved: §1 #4 registry requires a declared purpose + lawful basis per category; §1 #5 rejects keystroke/screen/location/private-life categories at registration (422); undeclared categories are not captured. Encodes DEC-2522 minimisation as an invariant. AC #7 #8 #9.

### ISS-004 — Within one tenant, every colleague shares a tenant_id
RLS keeps tenants apart but would let any employee read any colleague's performance file. Resolved: §1 #7-#8 intra-tenant `may_read` (founder OR manager-of OR self OR explicit grant) enforced AS WELL AS tenant RLS (defence in depth); the `eval_access_grant` table makes manager/HR access explicit + revocable. AC #12 #13 #14 #15 #18. Encodes DEC-2521 (access-restricted).

### ISS-005 — Cross-person reads must be reconstructable after the fact
Evaluation data can inform pay and progression; "who read whose file" cannot be invisible. Resolved: §1 #9 + #12 — every read where reader != target emits `eval.evaluation_read` (self-reads emit a lighter `eval.self_read`); §3 `guard_evaluation_read` is fail-closed (a read that cannot be audited returns no data). AC #16 #17. Encodes DEC-2524.

### ISS-006 — Monitoring data kept forever is unbounded liability
PDPD expects retention limits; keeping everything forever is both a legal and a cost risk. Resolved: §1 #6 per-category retention policy + daily sweeper that erases Layer-2 projections past `retain_days`; nothing retained without a policy. AC #10. Encodes DEC-2523.

### ISS-007 — Erasure must not break the tamper-evident chain
A naive "delete old data" would delete from `l1_audit_log` and break the integrity property the brain depends on. Resolved: §1 #6 sweeper operates ONLY on L2 projections (`l2_memory`/`l2_entity`) + EVAL artefacts, NEVER on L1; the erasure event is itself appended to L1 (`eval.subject_erased`). This is the established Layer-1-source / Layer-2-projection split. AC #11.

### ISS-008 — Data-subject rights were unaddressed
PDPD grants subjects access + rectification/objection; the system must honour them. Resolved: §1 #10 `GET /v1/eval/me` (self always readable) + `POST /v1/eval/me/requests` (rectification/objection/erasure_request/access_export) stored in `eval_dsr_request`. AC #19 #20.

### ISS-009 — Auto-applying a rights request removes the human checkpoint
Auto-erasure could destroy evidence; auto-objection could silently alter an evaluation; anything affecting employment must be a human call. Resolved: §1 #11 DSR requests are QUEUED (`open`), never auto-applied; the only automatic erasure is scheduled retention acting on operator-set policy. Cross-references TASK-EVAL-003 HITL. AC #19. Encodes Stephen decision 3 (mandatory HITL for consequential outcomes).

### ISS-010 — A changed monitoring scope must be re-disclosed
A one-time hire-day checkbox does not cover a later expansion of what is monitored. Resolved: §1 #17 a notice-version bump re-gates every subject whose ack is stale until they re-acknowledge; `eval_capture_gated_total{reason="stale_ack_version"}` increments. AC #23. Keeps consent living, not one-time.

### ISS-011 — The governance layer itself must be tamper-evident
If governance mutations were silent, the gate could be quietly disabled. Resolved: §1 #12 every governance mutation (notice publish, ack, category register, retention change, grant, revoke, DSR file/resolve, retention sweep, erasure) appends one `l1_audit_log` row via the shared `cyberos-audit-chain::chain_anchor`, byte-compatible with memory's reconcile; §1 #15 append-only tables. AC #21 #22. Encodes DEC-2524.

### ISS-012 — Covert collection must be explicitly out of scope, not merely undefined
Leaving covert mode undefined invites someone to build it later. Resolved: §1 #13 explicitly states fully-covert / no-notice collection is OUT OF SCOPE and a legal risk for Vietnamese counsel; the disclosed notice + acknowledgment gate are the boundary; §2 closing note + §9 require counsel sign-off on the notice text and per-category lawful basis. The spec states plainly it is not legal advice. Encodes DEC-2525.

## §3 — Resolution

All 12 concerns addressed. The spec encodes a defensible governance shape — disclosure (versioned, hash-pinned notice), the acknowledgment gate that makes wide day-1 capture lawful, purpose limitation + minimisation in the registry, intra-tenant access control with a read-audit per cross-person read, bounded retention that never breaks the L1 chain, queued (human-resolved) data-subject rights, and one tamper-evident chained row per governance mutation — grounded in Vietnam's PDPD (Decree 13/2023/ND-CP), Labor Code (45/2019/QH14), and Decree 145/2020, with the explicit caveat that the notice content requires Vietnamese counsel sign-off. The `## AI Risk Assessment` section is present and accurate for the high risk class. Reuse is honoured throughout (TASK-AUTH-003 RLS + roles, `cyberos-audit-chain`, the L2 projection split) — no re-architecture. **Score = 10/10.**

Per task-audit skill §0 master rule: depth is bounded by the genuine governance surface (notice × acknowledgment gate × registry/minimisation × access control × read-audit × retention-without-breaking-L1 × data-subject-rights × HITL × tamper-evident mutations × re-gating), not by line targets. This is the Phase-0 gate every later BRAIN/EVAL task inherits its legality from; it ships the structure, and counsel confirms the content.

---

*End of TASK-EVAL-001 audit.*
