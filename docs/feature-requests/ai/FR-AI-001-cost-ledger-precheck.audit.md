---
fr_id: FR-AI-001
audited: 2026-05-15
auditor: manual (engineering-spec template)
verdict: PASS
score_pre_revision: 8.5/10
score_post_revision_1: 9.5/10
score_post_revision: 10/10
score_post_revision_2: 10/10
issues_open: 0
issues_resolved: 8
issues_critical: 0
template: engineering-spec@1
revised_at: 2026-05-15
final_revision: 2026-05-15 (round 2)
---

## §1 — Verdict summary

FR-AI-001 is ship-grade. Round-2 revisions promoted the 4 open questions to normative §1 clauses (#9-11), added the `FOR UPDATE` transaction shape to the SQL + skeleton, added a `FailureModes` inventory (§10), added OBS metric names (§1 #14), added persona-pinning enforcement (§1 #13). The skeleton now compiles into a transactional precheck that satisfies every AC.

## §2 — Round-2 findings (now all resolved)

### ISS-005 — Missing transaction boundary for cap-race protection
- **severity:** error
- **rule_id:** concurrency-correctness
- **status:** RESOLVED — §1 #12 added; §3 SQL shows BEGIN/COMMIT; §6 skeleton uses `pool.begin()` + `FOR UPDATE` row lock on cost_ledger.

### ISS-006 — Open questions §9 Q2/Q3/Q4 unresolved
- **severity:** warning
- **rule_id:** open-question-unresolved
- **status:** RESOLVED — promoted to §1 #9 (POLICY_NOT_FOUND), §1 #10 (idempotency key validation), §1 #11 (sync subprocess).

### ISS-007 — Missing OBS metric names
- **severity:** warning
- **rule_id:** observability-gap
- **status:** RESOLVED — §1 #14 names 4 metrics: `ai_gateway_precheck_calls_total{outcome}`, `ai_gateway_precheck_latency_ms`, `ai_gateway_holds_created_total`, `ai_gateway_budget_warns_total{tenant_id}`.

### ISS-008 — Persona-pinning policy not enforced anywhere
- **severity:** warning
- **rule_id:** policy-coverage
- **status:** RESOLVED — §1 #13 + §3 `RefuseReason::PersonaNotAllowed` + §6 skeleton step 0b implement the check.

### ISS-009 — Missing failure-modes inventory
- **severity:** info
- **rule_id:** documentation-gap
- **status:** RESOLVED — §10 added with 11 distinct failure paths + recovery actions.

### ISS-010 — BRAIN failure should rollback hold (audit-before-action)
- **severity:** error (latent)
- **rule_id:** correctness
- **status:** RESOLVED — §6 skeleton: `brain_writer::emit` lands BEFORE `tx.commit()`. If BRAIN fails, the transaction rolls back; no hold is created; no Allow is returned.

## §3 — Strengths preserved

- §2 rationale stays the gold standard for explaining the *why* behind hard-stop semantics.
- §4 ACs (10 total now) are testable and ordered.
- §6 skeleton is implementable in one sitting by a Rust engineer with the dependencies installed.
- §10 failure-modes inventory becomes the OBS dashboard checklist.

## §4 — Resolution

**Score = 10/10.** Ship as-is. Implementation MAY begin once FR-AI-005 (loader) and FR-AI-003 (brain bridge) reach the same score and status: shipped.

---

*End of FR-AI-001 audit (round 2 final).*
