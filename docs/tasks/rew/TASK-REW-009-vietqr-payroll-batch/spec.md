---
id: TASK-REW-009
title: "REW VietQR bank payroll batch send — bulk transfer file generation with CFO manual confirm at submission to bank"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: rew
priority: p0
status: draft
verify: T
phase: P2
milestone: P2 · slice 2
slice: 2
owner: Stephen Cheng (CFO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-REW-005, TASK-INV-005, TASK-CRM-009, TASK-MEMORY-111]
depends_on: [TASK-INV-005]
blocks: []

source_pages:
  - website/docs/modules/rew.html#vietqr-batch

source_decisions:
  - DEC-2230 2026-05-17 — Generate VietQR bulk transfer file (per-Napas spec) from committed payroll; CFO downloads + manually submits to bank portal
  - DEC-2231 2026-05-17 — Closed enum `batch_status` = {generated, downloaded, cfo_confirmed_submitted, paid_acked, partial_failed, failed}; cardinality 6
  - DEC-2232 2026-05-17 — CFO confirms manually that bank submission succeeded; system updates payroll status accordingly
  - DEC-2233 2026-05-17 — Reconciliation: TASK-INV-005 VietQR webhook acks individual member transfers; system matches by memo template
  - DEC-2234 2026-05-17 — memory audit kinds: rew.batch_generated, rew.batch_downloaded, rew.batch_cfo_confirmed, rew.batch_paid_acked, rew.batch_failed

language: rust 1.81
service: cyberos/services/rew/
new_files:
  - services/rew/migrations/0009_payroll_batches.sql
  - services/rew/src/batch/mod.rs
  - services/rew/src/batch/file_generator.rs
  - services/rew/src/batch/reconciliation.rs
  - services/rew/src/handlers/batch_routes.rs
  - services/rew/src/audit/batch_events.rs
  - services/rew/tests/batch_status_enum_cardinality_test.rs
  - services/rew/tests/batch_file_format_test.rs
  - services/rew/tests/batch_cfo_manual_confirm_test.rs
  - services/rew/tests/batch_reconciliation_test.rs
  - services/rew/tests/batch_audit_emission_test.rs

modified_files:
  - services/rew/src/lib.rs

allowed_tools:
  - file_read: services/{rew,inv}/**
  - file_write: services/rew/{src,tests,migrations}/**
  - bash: cd services/rew && cargo test batch

disallowed_tools:
  - auto-submit to bank (per DEC-2232)
  - skip CFO confirm (per DEC-2232)

effort_hours: 5
subtasks:
  - "0.3h: 0009_payroll_batches.sql"
  - "0.3h: batch/mod.rs"
  - "0.6h: file_generator.rs"
  - "0.5h: reconciliation.rs"
  - "0.4h: handlers/batch_routes.rs"
  - "0.3h: audit/batch_events.rs"
  - "1.8h: tests — 5 test files"
  - "0.7h: CFO UI for download + confirm + docs"
  - "0.1h: bank format spec doc"

risk_if_skipped: "Without batch file, CFO copies amounts manually → error. Without DEC-2232 manual confirm, auto-submit could fire on bug. Without DEC-2233 reconciliation, payments unmatched."
---

## §1 — Description (BCP-14 normative)

The REW service **MUST** ship VietQR payroll batch at `services/rew/src/batch/` generating bulk transfer file + CFO manual confirm + TASK-INV-005 reconciliation, 5 memory audit kinds.

1. **MUST** validate `batch_status` against closed enum per DEC-2231.

2. **MUST** generate file at `file_generator.rs::generate(payroll_run)` per DEC-2230:
- For each member: account_number + amount + memo (template per TASK-CRM-009 convention)
- Output Napas-bulk-transfer XML or CSV format
- SHA256 the file for verification

3. **MUST** require CFO manual confirm at `POST .../confirm-submitted` per DEC-2232 — system updates status; no auto-submit.

4. **MUST** reconcile per DEC-2233 — match TASK-INV-005 inbound webhook acks to batch entries via memo.

5. **MUST** define tables at migration `0009`:
   ```sql
   CREATE TABLE rew_payroll_batches (
     batch_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     payroll_run_id UUID NOT NULL REFERENCES rew_payroll_runs(run_id),
     status TEXT NOT NULL DEFAULT 'generated'
       CHECK (status IN ('generated','downloaded','cfo_confirmed_submitted','paid_acked','partial_failed','failed')),
     file_sha256 CHAR(64) NOT NULL,
     file_doc_id UUID NOT NULL,
     total_amount_vnd BIGINT NOT NULL,
     members_count INT NOT NULL,
     generated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     downloaded_at TIMESTAMPTZ,
     downloaded_by UUID,
     cfo_confirmed_at TIMESTAMPTZ,
     cfo_confirmed_by UUID,
     trace_id CHAR(32),
     UNIQUE (payroll_run_id)
   );
   ALTER TABLE rew_payroll_batches ENABLE ROW LEVEL SECURITY;
   CREATE POLICY batches_rls ON rew_payroll_batches
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON rew_payroll_batches FROM cyberos_app;
   GRANT UPDATE (status, downloaded_at, downloaded_by, cfo_confirmed_at, cfo_confirmed_by) ON rew_payroll_batches TO cyberos_app;

   CREATE TABLE rew_batch_member_acks (
     ack_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     batch_id UUID NOT NULL REFERENCES rew_payroll_batches(batch_id),
     member_id UUID NOT NULL,
     amount_vnd BIGINT NOT NULL,
     memo TEXT NOT NULL,
     status TEXT NOT NULL DEFAULT 'pending'
       CHECK (status IN ('pending','acked','failed')),
     acked_at TIMESTAMPTZ,
     UNIQUE (batch_id, member_id)
   );
   ALTER TABLE rew_batch_member_acks ENABLE ROW LEVEL SECURITY;
   CREATE POLICY acks_rls ON rew_batch_member_acks
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON rew_batch_member_acks FROM cyberos_app;
   GRANT UPDATE (status, acked_at) ON rew_batch_member_acks TO cyberos_app;
   ```

6. **MUST** expose endpoints:
   ```text
   POST /v1/rew/payroll/runs/{id}/batch        (generate after commit)
   GET  /v1/rew/payroll/batches/{id}/file      (CFO downloads; logs)
   POST /v1/rew/payroll/batches/{id}/confirm   (CFO manual confirm submitted)
   GET  /v1/rew/payroll/batches/{id}           (status + acks)
   ```

7. **MUST** emit 5 memory audit kinds per DEC-2234. PII per TASK-MEMORY-111: amounts SHA256.

8. **MUST** thread trace_id from generate → download → confirm → ack → audit.

9. **MUST NOT** auto-submit to bank per DEC-2232.

10. **MUST NOT** skip CFO confirm per DEC-2232.

---

## §2 — Why this design

**Why CFO manual submit (DEC-2232)?** Banks don't yet have API for VN payroll; CFO uses bank portal directly. Manual confirm avoids state ambiguity.

**Why TASK-INV-005 reconciliation (DEC-2233)?** Inbound VietQR webhook tracks individual transfers; matching by memo enables auto-status.

**Why per-member ack rows (DEC-2233)?** Bank may fail one member's transfer (insufficient funds, closed account); per-row status tracks partial failures.

---

## §3 — API contract

Sample batch file (CSV format):
```csv
account_number,amount_vnd,memo
1234567890,25000000,REW-2026-06-uuid-a
0987654321,30000000,REW-2026-06-uuid-b
```

Sample status:
```json
{
  "batch_id": "uuid",
  "status": "paid_acked",
  "total_amount_vnd": 825000000,
  "members_count": 30,
  "acked_count": 30,
  "failed_count": 0
}
```

---

## §4 — Acceptance criteria
1. **batch_status enum cardinality 6**. 2. **Napas-spec file format**. 3. **Memo template per TASK-CRM-009**. 4. **SHA256 file integrity**. 5. **TASK-DOC-001 storage**. 6. **CFO-only download**. 7. **CFO-only confirm**. 8. **NEVER auto-submit**. 9. **5 memory audit kinds emitted**. 10. **PII scrubbed (amounts SHA256)**. 11. **RLS denies cross-tenant**. 12. **Trace_id preserved**. 13. **Append-only via REVOKE except status cols**. 14. **UNIQUE(payroll_run_id) — one batch per run**. 15. **Per-member ack tracking**. 16. **Partial failure → status=partial_failed**. 17. **All acked → status=paid_acked**. 18. **bigint VND**. 19. **TASK-INV-005 reconciliation hooks**. 20. **Re-confirm idempotent**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn never_auto_submits() {
    let ctx = TestContext::with_committed_payroll().await;
    ctx.generate_batch(ctx.run_id).await;
    let bank_calls = ctx.bank_api_call_count().await;
    assert_eq!(bank_calls, 0);
    let batch = ctx.fetch_batch(ctx.run_id).await;
    assert_eq!(batch.status, "generated");
}

#[tokio::test]
async fn cfo_confirm_updates_status() {
    let ctx = TestContext::with_generated_batch().await;
    ctx.download_as_cfo(ctx.batch_id).await;
    ctx.cfo_confirm_submitted(ctx.batch_id).await;
    let batch = ctx.fetch_batch(ctx.batch_id).await;
    assert_eq!(batch.status, "cfo_confirmed_submitted");
}

#[tokio::test]
async fn reconciliation_matches_acks() {
    let ctx = TestContext::with_cfo_confirmed_batch().await;
    ctx.simulate_inv_005_webhook_acks(ctx.batch_id).await;
    let batch = ctx.fetch_batch(ctx.batch_id).await;
    assert_eq!(batch.status, "paid_acked");
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-INV-005. **Cross-module:** TASK-REW-005 (payroll source), TASK-CRM-009 (memo format), TASK-DOC-001 (file storage), TASK-AUTH-101 (CFO role), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| File generation fail | catch | status=failed | retry |
| Bank submission fail (CFO reports) | CFO marks failed | status=failed | retry batch |
| Partial member transfer fail | ack tracking | status=partial_failed | per-member retry |
| Cross-tenant query | RLS | 0 rows | inherent |
| Duplicate batch per run | UNIQUE | 409 | inherent |
| Memo format mismatch (TASK-CRM-009 drift) | reconciliation fail | sev-2 | data fix |
| Decimal precision | bigint VND | inherent | inherent |
| CFO confirms before submit | timing assumption | CFO discipline | inherent |
| Re-confirm | idempotent | inherent | inherent |
| File format invalid (bank rejects) | CFO reports | sev-1 | regenerate |

## §11 — Implementation notes
- §11.1 File format: Napas bulk transfer XML or CSV per bank's accepted spec.
- §11.2 Memo template: `REW-{period_yyyymm}-{member_id_8}` matches TASK-INV-005 reconciliation regex.
- §11.3 memory audit body: batch_id, payroll_run_id, status, counts; amounts SHA256.
- §11.4 Reconciliation cron via TASK-MCP-007 polls every 15min during pay-day window.
- §11.5 Future: bank API integration eliminates manual confirm step.

---

*End of TASK-REW-009 spec.*
