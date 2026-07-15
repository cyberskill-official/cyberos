---
id: NFR-OKR-002
title: "OKR custom-SQL co-sign — custom-SQL KRs MUST require CTO + objective-owner co-sign"
module: OKR
category: security
priority: MUST
verification: T
phase: P0
slo: "100% of custom-SQL KRs carry both signatures + readonly DB role"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-OKR-003]
---

## §1 — Statement (BCP-14 normative)

1. KRs using `custom-sql` progress source **MUST** be co-signed by CTO + the objective owner before activation; the SQL is reviewed for safety + correctness.
2. The SQL **MUST** execute against a read-only DB role with row-level isolation to the tenant; cross-tenant queries are impossible by construction.
3. Query results **MUST** be cached for ≥ 1 hour to limit DB load from frequent OKR refresh.
4. SQL execution timeout **MUST** be 30s; longer-running queries are rejected.
5. Custom-SQL KRs **MUST** include a sample expected-value or test fixture; the test runs at activation time to verify the SQL produces sensible output.

## §2 — Why this constraint

Custom SQL is an extension surface that lets tenants measure things the platform doesn't natively expose — powerful, but a risk vector. The co-sign + readonly + tenant-isolation triad makes it safe. Cache + timeout protect the DB. Sample-fixture test ensures the SQL actually returns a number.

## §3 — Measurement

- Counter `okr_custom_sql_activation_total{has_both_signatures}`.
- Histogram `okr_custom_sql_execution_seconds`.
- Counter `okr_custom_sql_timeout_total`.

## §4 — Verification

- Integration test (T) — single-signer → reject.
- Pen test (T) — cross-tenant probe; assert blocked.
- Timeout test (T) — slow SQL → reject.

## §5 — Failure handling

- Single-signer attempt → block.
- Cross-tenant query bypassed → sev-1; halt; investigate.
- Timeout rate > 5% → sev-3; tenant SQL needs review.

---

*End of NFR-OKR-002.*
