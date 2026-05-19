---
id: FR-REW-002
title: "REW parameter versioning — immutable versioned formula parameters with 100% replay-equivalence on prior payslips"
module: REW
priority: MUST
status: draft
verify: T
phase: P2
milestone: P2 · slice 1
slice: 1
owner: Stephen Cheng (CFO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-REW-001, FR-REW-005, FR-HR-005, FR-MEMORY-111]
depends_on: [FR-REW-001]
blocks: []

source_pages:
  - website/docs/modules/rew.html#param-versioning

source_decisions:
  - DEC-2160 2026-05-17 — All payroll formula parameters (tax brackets, SI rates, overtime multipliers, etc.) version-pinned; payslip replay must produce byte-identical output
  - DEC-2161 2026-05-17 — Closed enum `param_kind` = {tax_bracket, si_rate, overtime_multiplier, allowance_cap, bonus_formula}; cardinality 5
  - DEC-2162 2026-05-17 — Per-version snapshot IMMUTABLE; effective_from + effective_to dates; payslip records version_id at compute time
  - DEC-2163 2026-05-17 — Replay-equivalence test: monthly CI job replays last 12 months of payslips with current params; expect 100% match for prior periods (using their version), 0% match if forced to current
  - DEC-2164 2026-05-17 — memory audit kinds: rew.param_version_added, rew.param_lookup_executed, rew.replay_test_passed, rew.replay_test_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/rew/
  new_files:
    - services/rew/migrations/0002_param_versions.sql
    - services/rew/src/params/mod.rs
    - services/rew/src/params/loader.rs
    - services/rew/src/params/replay_test.rs
    - services/rew/src/audit/params_events.rs
    - services/rew/tests/param_kind_enum_cardinality_test.rs
    - services/rew/tests/param_immutability_test.rs
    - services/rew/tests/param_version_lookup_test.rs
    - services/rew/tests/replay_equivalence_test.rs
    - services/rew/tests/param_audit_emission_test.rs

  modified_files:
    - services/rew/src/lib.rs

  allowed_tools:
    - file_read: services/{rew,hr}/**
    - file_write: services/rew/{src,tests,migrations}/**
    - bash: cd services/rew && cargo test params

  disallowed_tools:
    - mutate prior version (per DEC-2162)
    - non-deterministic params (per DEC-2163)

effort_hours: 6
sub_tasks:
  - "0.4h: 0002_param_versions.sql"
  - "0.4h: params/mod.rs"
  - "0.5h: loader.rs"
  - "0.7h: replay_test.rs"
  - "0.3h: audit/params_events.rs"
  - "2.5h: tests — 5 test files"
  - "0.7h: docs + CI integration"
  - "0.5h: monthly replay cron"

risk_if_skipped: "Without versioning, FR-REW-005 payroll varies by run time → audit failure. Without DEC-2163 replay test, drift detected only at audit (too late)."
---

## §1 — Description (BCP-14 normative)

The REW service **MUST** ship parameter versioning at `services/rew/src/params/` with immutable snapshots + version-pinned payslip computation + monthly replay-equivalence CI, 4 memory audit kinds.

1. **MUST** validate `param_kind` against closed enum per DEC-2161.

2. **MUST** define tables at migration `0002`:
   ```sql
   CREATE TABLE rew_param_versions (
     version_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     kind TEXT NOT NULL
       CHECK (kind IN ('tax_bracket','si_rate','overtime_multiplier','allowance_cap','bonus_formula')),
     value_jsonb JSONB NOT NULL,
     effective_from DATE NOT NULL,
     effective_to DATE,
     source_law_reference TEXT,
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     created_by UUID NOT NULL
   );
   CREATE INDEX param_kind_effective_idx ON rew_param_versions(tenant_id, kind, effective_from DESC);
   ALTER TABLE rew_param_versions ENABLE ROW LEVEL SECURITY;
   CREATE POLICY param_rls ON rew_param_versions
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON rew_param_versions FROM cyberos_app;
   ```

3. **MUST** lookup at `loader.rs::get(tenant_id, kind, effective_at)` per DEC-2162:
   - Returns version effective at the date.
   - Deterministic — same params → same result (critical for FR-REW-005).

4. **MUST** run monthly replay-equivalence test via FR-MCP-007 per DEC-2163:
   - For last 12 months of payslips, re-run compute with original version_id.
   - Expect byte-identical output.
   - Failure → sev-1 alert + CI block.

5. **MUST** expose endpoints:
   ```text
   POST /v1/rew/params               (CFO; new version)
   GET  /v1/rew/params/{kind}?at=... (lookup at date)
   POST /v1/rew/replay-test/trigger  (CFO manual)
   ```

6. **MUST** emit 4 memory audit kinds per DEC-2164. PII per FR-MEMORY-111: param values (mostly rates/brackets) ok in memory as they're public; member-specific compute hashes only.

7. **MUST** thread trace_id from lookup → audit.

8. **MUST NOT** mutate prior version per DEC-2162 (REVOKE UPDATE/DELETE).

9. **MUST NOT** use `now()` or random in compute path per DEC-2163.

---

## §2 — Why this design

**Why versioning (DEC-2160)?** FR-REW-005 payroll must reproduce historical payslips exactly; mutable params break replay.

**Why replay test (DEC-2163)?** Without automated check, drift creeps in unnoticed; monthly CI catches early.

**Why immutable (DEC-2162)?** Audit lineage requires unmutable history; corrections via new version with new effective_from.

---

## §3 — API contract

Sample param lookup:
```json
GET /v1/rew/params/tax_bracket?at=2026-06-01

{
  "version_id": "uuid",
  "kind": "tax_bracket",
  "value_jsonb": [
    {"min_vnd": 0, "max_vnd": 5000000, "rate": 0.05},
    {"min_vnd": 5000000, "max_vnd": 10000000, "rate": 0.10}
  ],
  "effective_from": "2025-01-01",
  "source_law_reference": "Decree 152/2020 Art. 7"
}
```

---

## §4 — Acceptance criteria
1. **param_kind enum cardinality 5**. 2. **Immutable rows (REVOKE)**. 3. **Lookup by effective_at**. 4. **Replay test monthly via cron**. 5. **100% match for prior periods**. 6. **Failure → sev-1 + CI block**. 7. **4 memory audit kinds emitted**. 8. **PII: param values public; compute hashed**. 9. **RLS denies cross-tenant**. 10. **CFO-only write**. 11. **Trace_id preserved**. 12. **Append-only via REVOKE**. 13. **JSONB schema validated per kind**. 14. **Source law reference recommended**. 15. **Effective_to NULL = current**. 16. **Index on (kind, effective_from)**. 17. **Lookup performance < 5ms**. 18. **Annual refresh runbook**. 19. **Replay test produces diff report on failure**. 20. **Deterministic (no now/random)**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn version_lookup_at_date() {
    let ctx = TestContext::with_two_tax_bracket_versions("2024-01", "2025-01").await;
    let v_2024 = ctx.get_param("tax_bracket", "2024-06-01").await;
    let v_2025 = ctx.get_param("tax_bracket", "2025-06-01").await;
    assert_ne!(v_2024.version_id, v_2025.version_id);
}

#[tokio::test]
async fn immutability_enforced() {
    let ctx = TestContext::with_param_version().await;
    let r = ctx.try_update_param(ctx.version_id).await;
    assert!(r.is_err());
}

#[tokio::test]
async fn replay_equivalence_100pct() {
    let ctx = TestContext::with_12_months_payslips().await;
    let result = ctx.run_replay_test().await;
    assert_eq!(result.match_pct, dec!(100.0));
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-REW-001.
**Downstream:** FR-REW-005 (payroll compute uses versioned params).
**Cross-module:** FR-HR-005 (compares versioning approach), FR-MCP-007 (replay cron), FR-MEMORY-111 (audit).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Replay test fail | sev-1 alert | CI block | investigate diff |
| Lookup at date with no version | error | 404 | seed gap |
| Two versions same date | UNIQUE on (kind, effective_from) | reject | use later date |
| JSONB schema invalid | validator | 400 | fix shape |
| Cross-tenant query | RLS | 0 rows | inherent |
| Non-CFO write | role check | 403 | request CFO |
| Decimal precision drift | rust_decimal | inherent | inherent |
| Cron skipped | catch-up | inherent | inherent |
| Test data corrupt | sev-1 | manual review | data fix |
| Param value range invalid | range check | 400 | fix |

## §11 — Implementation notes
- §11.1 Replay test reruns FR-REW-005 compute with member context at original period; compares to stored payslip blob hash.
- §11.2 Cron via FR-MCP-007 `kind: 'rew.replay_equivalence_test'`, monthly 1st at 02:00.
- §11.3 Failure produces diff report uploaded to FR-DOC-001 for CFO review.
- §11.4 memory audit body: tenant_id, kind, version_id; param values ok (not PII).
- §11.5 JSONB schema per kind documented in code constants + tested.

---

*End of FR-REW-002 spec.*
