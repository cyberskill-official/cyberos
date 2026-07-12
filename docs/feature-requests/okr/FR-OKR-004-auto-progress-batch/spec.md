---
id: FR-OKR-004
title: "OKR auto-progress nightly batch — resolves all KR progress_sources + updates current_value + emits drift alerts"
module: OKR
priority: MUST
status: draft
verify: T
phase: P1
milestone: P1 · slice 3
slice: 3
owner: Stephen Cheng (CEO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-OKR-003, FR-OKR-002, FR-MCP-007, FR-MEMORY-111]
depends_on: [FR-OKR-003]
blocks: []

source_pages:
  - website/docs/modules/okr.html#auto-progress

source_decisions:
  - DEC-1990 2026-05-17 — Nightly batch at 03:00 tenant_tz; resolves all active KR progress_sources; idempotent per (kr_id, run_date)
  - DEC-1991 2026-05-17 — Closed enum `batch_run_status` = {running, completed, partial, failed}; cardinality 4
  - DEC-1992 2026-05-17 — Drift alert: if current_value changes >10% in one run, emit sev-2 audit (likely upstream data issue)
  - DEC-1993 2026-05-17 — Per-KR failure isolated — one failure doesn't halt batch
  - DEC-1994 2026-05-17 — memory audit kinds: okr.batch_started, okr.batch_kr_resolved, okr.batch_kr_drift_alert, okr.batch_kr_failed, okr.batch_completed

build_envelope:
  language: rust 1.81
  service: cyberos/services/okr/
  new_files:
    - services/okr/migrations/0004_auto_progress_runs.sql
    - services/okr/src/auto_progress/mod.rs
    - services/okr/src/auto_progress/batch_runner.rs
    - services/okr/src/auto_progress/drift_detector.rs
    - services/okr/src/handlers/auto_progress_routes.rs
    - services/okr/src/audit/auto_progress_events.rs
    - services/okr/tests/batch_runs_all_active_test.rs
    - services/okr/tests/batch_per_kr_isolation_test.rs
    - services/okr/tests/batch_idempotent_test.rs
    - services/okr/tests/batch_drift_alert_test.rs
    - services/okr/tests/batch_status_enum_cardinality_test.rs
    - services/okr/tests/batch_audit_emission_test.rs

  modified_files:
    - services/okr/src/lib.rs

  allowed_tools:
    - file_read: services/okr/**
    - file_write: services/okr/{src,tests,migrations}/**
    - bash: cd services/okr && cargo test auto_progress

  disallowed_tools:
    - mutate prior batch run (per DEC-1990)
    - halt batch on single KR failure (per DEC-1993)

effort_hours: 5
sub_tasks:
  - "0.3h: 0004_auto_progress_runs.sql"
  - "0.3h: auto_progress/mod.rs"
  - "0.6h: batch_runner.rs"
  - "0.4h: drift_detector.rs"
  - "0.4h: handlers/auto_progress_routes.rs"
  - "0.3h: audit/auto_progress_events.rs"
  - "1.9h: tests — 6 test files"
  - "0.4h: cron registration + docs"
  - "0.4h: dashboard UI for batch status"

risk_if_skipped: "Without nightly batch, KRs stale → Monday check-ins use last-week data. Without DEC-1993 isolation, one bad KR halts all updates. Without DEC-1992 drift alert, upstream data corruption silently mis-reports KR progress."
---

## §1 — Description (BCP-14 normative)

The OKR service **MUST** ship auto-progress batch at `services/okr/src/auto_progress/` running nightly via FR-MCP-007 cron, resolving DSL per KR, drift detection, 5 memory audit kinds.

1. **MUST** schedule daily batch at 03:00 tenant_tz per DEC-1990.

2. **MUST** validate `batch_run_status` against closed enum per DEC-1991.

3. **MUST** run at `batch_runner.rs::run(tenant, run_date)`:
   - SELECT all KRs WHERE progress_source IS NOT NULL AND status='active'
   - Per KR: call FR-OKR-003 resolver; update current_value + computed_progress_pct
   - Catch + log per-KR failures (DEC-1993); continue batch
   - Final status: completed / partial (if any failed) / failed (if 0 succeeded)

4. **MUST** detect drift at `drift_detector.rs::check(kr, new_value, old_value)` per DEC-1992 — if abs((new-old)/old) > 0.10, emit sev-2 audit.

5. **MUST** define table at migration `0004`:
   ```sql
   CREATE TABLE okr_auto_progress_runs (
     run_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     run_date DATE NOT NULL,
     status TEXT NOT NULL DEFAULT 'running'
       CHECK (status IN ('running','completed','partial','failed')),
     krs_total INT NOT NULL DEFAULT 0,
     krs_succeeded INT NOT NULL DEFAULT 0,
     krs_failed INT NOT NULL DEFAULT 0,
     krs_drift_alerted INT NOT NULL DEFAULT 0,
     started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     completed_at TIMESTAMPTZ,
     trace_id CHAR(32),
     UNIQUE (tenant_id, run_date)
   );
   ALTER TABLE okr_auto_progress_runs ENABLE ROW LEVEL SECURITY;
   CREATE POLICY runs_rls ON okr_auto_progress_runs
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON okr_auto_progress_runs FROM cyberos_app;
   GRANT UPDATE (status, krs_total, krs_succeeded, krs_failed, krs_drift_alerted, completed_at) ON okr_auto_progress_runs TO cyberos_app;
   ```

6. **MUST** be idempotent per DEC-1990 — UNIQUE on (tenant_id, run_date); duplicate run skipped.

7. **MUST** expose endpoints:
   ```text
   POST /v1/okr/auto-progress/trigger    (CEO; manual run for today)
   GET  /v1/okr/auto-progress/runs       (list)
   GET  /v1/okr/auto-progress/runs/{id}  (detail per KR)
   ```

8. **MUST** emit 5 memory audit kinds per DEC-1994. PII per FR-MEMORY-111: value diffs SHA-256 hashed; counts ok.

9. **MUST** thread trace_id from cron → batch → resolver → audit.

10. **MUST NOT** mutate prior batch run per DEC-1990 (REVOKE except status cols).

11. **MUST NOT** halt batch on single failure per DEC-1993.

---

## §2 — Why this design

**Why nightly (DEC-1990)?** Weekly check-ins need fresh data Monday; daily run guarantees ≤24h freshness.

**Why per-KR isolation (DEC-1993)?** One bad upstream module shouldn't blank entire tenant's OKR view.

**Why drift alert (DEC-1992)?** Large jumps usually mean data corruption (e.g. upstream bug doubled count); humans investigate.

**Why idempotent (DEC-1990)?** Cron retry on failure must not double-resolve.

---

## §3 — API contract

Sample run status:
```json
{
  "run_id": "uuid",
  "run_date": "2026-05-17",
  "status": "partial",
  "krs_total": 47,
  "krs_succeeded": 45,
  "krs_failed": 2,
  "krs_drift_alerted": 1,
  "completed_at": "2026-05-17T03:05:00Z"
}
```

---

## §4 — Acceptance criteria
1. **Nightly 03:00 tenant_tz**. 2. **batch_run_status enum cardinality 4**. 3. **All active KRs with progress_source resolved**. 4. **Per-KR failure isolated**. 5. **Drift alert at >10% delta**. 6. **Idempotent (UNIQUE run_date)**. 7. **5 memory audit kinds emitted**. 8. **PII scrubbed (value diffs SHA256)**. 9. **RLS denies cross-tenant**. 10. **CEO-only manual trigger**. 11. **Trace_id preserved**. 12. **Append-only via REVOKE except status cols**. 13. **status=completed when 100% success**. 14. **status=partial when any failure**. 15. **status=failed when 100% failure**. 16. **Cron skip if 0 active KRs**. 17. **Backfill via CEO trigger with run_date**. 18. **Run history queryable**. 19. **Concurrent run blocked (UNIQUE)**. 20. **First-run no drift alert (no prior value)**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn batch_resolves_all_active() {
    let ctx = TestContext::with_5_active_krs().await;
    ctx.run_batch(today()).await;
    let run = ctx.fetch_latest_run().await;
    assert_eq!(run.krs_succeeded, 5);
    assert_eq!(run.status, "completed");
}

#[tokio::test]
async fn per_kr_failure_isolated() {
    let ctx = TestContext::with_5_krs_one_will_fail().await;
    ctx.run_batch(today()).await;
    let run = ctx.fetch_latest_run().await;
    assert_eq!(run.krs_succeeded, 4);
    assert_eq!(run.krs_failed, 1);
    assert_eq!(run.status, "partial");
}

#[tokio::test]
async fn drift_alert_at_15pct() {
    let ctx = TestContext::with_kr_current_100().await;
    ctx.mock_resolver_returns(115).await;
    ctx.run_batch(today()).await;
    let audits = ctx.fetch_memory_audits("okr.batch_kr_drift_alert").await;
    assert!(!audits.is_empty());
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-OKR-003.
**Cross-module:** FR-MCP-007 (cron), FR-AUTH-101 (CEO role), FR-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Cron skipped | last_run check | catch-up next | inherent |
| Resolver timeout per KR | retry 1x | mark KR failed; continue | inherent |
| All KRs fail | aggregate | status=failed; sev-1 | investigate |
| Concurrent batch | UNIQUE | second skipped | inherent |
| Drift on first resolution | no prior | skip alert | inherent |
| Decimal precision | rust_decimal | inherent | inherent |
| Cross-tenant query | RLS | 0 rows | inherent |
| run_date in future | reject | 400 | use today |
| Manual trigger for old date | allow (backfill) | inherent | inherent |
| Mid-batch system crash | resume from last succeeded | status=partial | manual retry |

## §11 — Implementation notes
- §11.1 Cron via FR-MCP-007 `kind: 'okr.auto_progress_batch'`, daily 03:00.
- §11.2 Drift threshold 10% per DEC-1992; configurable per tenant in future.
- §11.3 Per-KR resolution timeout 30s; longer triggers fail-and-continue.
- §11.4 memory audit body: run_id, kr_id, old/new value SHA256, drift_pct.
- §11.5 Backfill mode: CEO POST with explicit run_date; uses that date's data context.

---

*End of FR-OKR-004 spec.*
