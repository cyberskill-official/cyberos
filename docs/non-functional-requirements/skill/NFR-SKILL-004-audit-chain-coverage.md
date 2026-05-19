---
id: NFR-SKILL-004
title: "SKILL audit-chain coverage — every skill invocation MUST emit ≥ 1 memory row"
module: SKILL
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% of skill invocations produce a Layer-1 row within 1s; reconciliation drift < 0.001%"
owner: CTO
created: 2026-05-18
related_frs: [FR-SKILL-101, FR-SKILL-105, FR-SKILL-106]
---

## §1 — Statement (BCP-14 normative)

1. Every skill invocation (success, denial, error) **MUST** result in at least one Layer-1 memory row carrying `{tenant_id, actor_id, skill_name, skill_version, capability_used, started_at, ended_at, outcome, output_digest?}`.
2. The audit row **MUST** be emitted within 1s of skill completion; emission **MUST** be at-least-once with idempotent dedup at Layer-2.
3. If the audit emit fails (memory unreachable, disk full), the skill runtime **MUST** retain the audit row on local disk in `skill/audit-pending/<uuid>.json` and replay on next memory healthcheck.
4. Reconciliation: number of invocations (per `cyberos-skill stats`) **MUST** match number of audit rows (per memory `SELECT count(*) FROM l2_memory WHERE kind='skill.invoke'`) within 0.001% over a 7-day window.
5. Audit rows **MUST NOT** be mutated post-emit; corrections take the form of a new compensating row with `kind=skill.invoke.correction`.

## §2 — Why this constraint

The audit chain is the platform's legally-attestable record of who invoked what. A single missed row is a hole in evidence; a class of missed rows is regulatory disqualification. The at-least-once + idempotent + local-spool design accepts a small replay cost in exchange for the strong guarantee. The 0.001% reconciliation budget is the tolerance for known-and-acceptable race conditions (e.g., process killed mid-write before spool is durable); anything beyond that is a real bug.

## §3 — Measurement

- Counter `skill_audit_emit_attempt_total{outcome}` and `skill_audit_emit_success_total`.
- Daily reconciliation job emits gauge `skill_audit_reconciliation_drift_ratio` — alarmed at > 0.001%.
- Counter `skill_audit_spool_replay_total` — surfaces how often local-spool was used.

## §4 — Verification

- Integration test `modules/skill/tests/audit_coverage_test.py` (T) — invokes 10k skills; asserts 100% audit-row coverage post-test.
- Chaos test (T) — kills memory connection mid-invocation; asserts skill completes, row spooled, replayed within 60s.
- Daily reconciliation job (T) auto-runs in production; alerts on drift.

## §5 — Failure handling

- Drift > 0.001% for 24 hours → sev-2; CTO + compliance lead engage; root-cause + backfill.
- Spool replay rate > 1% sustained → sev-3; memory connectivity degraded; investigate.
- Local spool > 10MB or > 1000 rows → sev-2; skill runtime starts shedding new invocations to protect data integrity.

---

*End of NFR-SKILL-004.*
