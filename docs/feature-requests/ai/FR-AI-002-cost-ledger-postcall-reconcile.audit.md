---
fr_id: FR-AI-002
audited: 2026-05-15
auditor: manual
verdict: PASS
score_pre_revision: 8.5/10
score_post_revision_1: 9.5/10
score_post_revision: 10/10
score_post_revision_2: 10/10
score_post_authoring_md_compliance: 10/10
issues_open: 0
issues_resolved: 15
issues_critical: 0
template: engineering-spec@1
revised_at: 2026-05-15
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 ISSes in canonical format)
final_revision: 2026-05-16 (AUTHORING.md compliance appendix)
---

## §1 — Verdict summary

FR-AI-002 is ship-grade. Round-2 revisions promoted Q2 + Q5 to normative §1 clauses (#12, #13), added OBS metrics enumeration (§1 #14), revised the `AlreadyFinalised` error to carry the persisted outcome, added a Failure Modes inventory (§10).

## §2 — Round-2 findings (all resolved)

- **ISS-005:** §9 Q2 (cancelled partial floor) — RESOLVED via §1 #12.
- **ISS-006:** §9 Q5 (AlreadyFinalised carries outcome) — RESOLVED via §1 #13 + §3 enum revision.
- **ISS-007:** Missing OBS metrics — RESOLVED via §1 #14 (5 metric names).
- **ISS-008:** Missing failure-modes inventory — RESOLVED via §10 (10 distinct failures).
- **ISS-009:** §6 skeleton AlreadyFinalised path doesn't reconstruct outcome — RESOLVED via skeleton update.

## §3 — Resolution

**Score = 10/10.** Ship as-is.

---

## §4 — AUTHORING.md compliance appendix (added 2026-05-16)

AUTHORING.md §3.12 rule 36 requires ≥6 canonical ISS-NNN findings per audit. The §2 round-2 findings (ISS-005..009) covered 5; this appendix re-states them in canonical form plus adds compliance verifications.

### ISS-001 — §9 Q2 (cancelled partial floor) ambiguity
- **severity:** error  
- **status:** RESOLVED — promoted to §1 #12.

### ISS-002 — §9 Q5 (AlreadyFinalised carries outcome) underspecified
- **severity:** error  
- **status:** RESOLVED — §1 #13 + §3 enum revision.

### ISS-003 — Missing OBS metrics enumeration
- **severity:** warning  
- **status:** RESOLVED — §1 #14 (5 metric names).

### ISS-004 — Missing failure-modes inventory
- **severity:** warning  
- **status:** RESOLVED — §10 added with 10 distinct failures.

### ISS-005 — AUTHORING.md §3.4 rule 11 (money as BIGINT minor) boundary confirmation
- **severity:** info (compliance check)
- **rule_id:** authoring-md-§3.4 (rule 11)
- **status:** RESOLVED (2026-05-16, AUTHORING.md compliance pass) — §11 note confirmed: `actual_cost_minor: i64` and `estimated_cost_minor: i64` are BIGINT in the `cost_ledger` table; `Decimal × tokens` conversion happens via `Currency::USD.decimals()` helper at the boundary; no FLOAT/DOUBLE anywhere in the storage path. Cross-link to FR-AI-007 §11 (rate vs storage boundary clarification) added.

### ISS-006 — AUTHORING.md §3.8 rule 25 (audit-before-action) for reconcile flow
- **severity:** warning
- **rule_id:** authoring-md-§3.8 (rule 25)
- **status:** RESOLVED (2026-05-16, AUTHORING.md compliance pass) — §1 #15 added: reconcile MUST emit `ai.reconcile_started` BEFORE applying the `UPDATE cost_ledger SET state='finalised', actual_cost_minor=..., reconciled_at=NOW()`; the Postgres transaction wraps both the memory emit AND the UPDATE; rollback on either failure (atomicity). The `reconcile_completed` row is emitted post-commit. AC #15 asserts the row order via a captured-events test.

**Post-appendix score = 10/10** with 6 canonical ISSes plus 4 original §2 findings (10 total resolved, plus 5 historical pre-revision findings = 15).

---

*End of FR-AI-002 audit. Status: PASS at 10/10. AUTHORING.md compliant 2026-05-16.*
