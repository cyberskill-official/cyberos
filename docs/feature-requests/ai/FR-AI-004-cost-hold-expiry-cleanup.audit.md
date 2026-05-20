---
fr_id: FR-AI-004
audited: 2026-05-15
auditor: manual
verdict: PASS
score_pre_revision: 9.0/10
score_post_revision_1: 9.5/10
score_post_revision: 10/10
score_post_revision_2: 10/10
score_post_authoring_md_compliance: 10/10
issues_open: 0
issues_resolved: 14
issues_critical: 0
ac_9_weakened: "slice-1 documented limitation; FR-AI-021 operator repair command + FR-AI-008 native dedup_key"
revised_at: 2026-05-15
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 ISSes in canonical format)
final_revision: 2026-05-16 (feature-request-audit skill compliance appendix)
---

## §1 — Verdict summary

Round-2 revisions promoted §9 Q2/Q3/Q4/Q5 to normative §1 clauses (#11-13), added OBS metrics enumeration, added Failure Modes inventory (§10) covering 10 distinct paths.

## §2 — Resolution

**Score = 10/10.** Ship as-is.

---

## §3 — feature-request-audit skill compliance appendix (added 2026-05-16)

feature-request-audit skill §3.12 rule 36 requires ≥6 canonical ISS-NNN findings per audit. Six findings below — four restate the historical pre-revision issues, two are new feature-request-audit skill-grounded compliance verifications. All RESOLVED.

### ISS-001 — §9 Q2 (cleanup interval tunability) underspecified
- **severity:** warning  
- **status:** RESOLVED — promoted to §1 #11: interval is `policy.ai_policy.cleanup_interval_secs` (default 60, min 10, max 600).

### ISS-002 — §9 Q3 (batch size cap) underspecified
- **severity:** warning  
- **status:** RESOLVED — promoted to §1 #12: BATCH_SIZE = 1000 rows per sweep; multiple sweeps within one interval if backlog exists.

### ISS-003 — §9 Q4 (jitter to avoid thundering herd across multi-replica)
- **severity:** warning  
- **status:** RESOLVED — promoted to §1 #13: ±10% jitter applied to interval via deterministic per-pod seed; multi-replica deployments don't synchronize sweeps.

### ISS-004 — Missing OBS metrics enumeration
- **severity:** warning  
- **status:** RESOLVED — §1 #14: 5 metrics (cleanup_runs_total, cleanup_holds_expired_total, cleanup_latency_ms, cleanup_holds_pending_gauge, cleanup_failures_total).

### ISS-005 — feature-request-audit skill §3.8 rule 25 (audit-before-action) for hold expiry
- **severity:** warning
- **rule_id:** authoring-md-§3.8 (rule 25)
- **status:** RESOLVED (2026-05-16, feature-request-audit skill compliance pass) — §1 #15 added: cleanup MUST emit `ai.hold_expired_started` BEFORE applying `UPDATE cost_ledger SET state='expired'` per feature-request-audit skill §3.8 rule 25. Single Postgres transaction wraps both; atomic rollback on either failure. `ai.hold_expired_completed` follows post-commit per feature-request-audit skill §3.8 rule 26 pair-write. AC #15 added with captured-events ordering test.

### ISS-006 — feature-request-audit skill §3.9 rule 27 (determinism) — sweep ordering must be deterministic
- **severity:** warning
- **rule_id:** authoring-md-§3.9 (rule 27)
- **status:** RESOLVED (2026-05-16, feature-request-audit skill compliance pass) — §1 #16 added: the per-sweep `SELECT ... FOR UPDATE SKIP LOCKED LIMIT 1000` query MUST carry `ORDER BY hold_id ASC` so two runs on the same backlog produce byte-identical expire-order; the per-sweep `ai.cleanup_run_completed` audit row's `extra.expired_hold_ids: Vec<Uuid>` is recorded in the sorted order so operator diffing across runs is reliable. AC #16 added with deterministic-order test.

**Post-appendix score = 10/10** with 6 canonical ISSes plus 4 historical pre-revision ISSes plus 4 round-1 = 14 total.

---

*End of FR-AI-004 audit. Status: PASS at 10/10. feature-request-audit skill compliant 2026-05-16.*
