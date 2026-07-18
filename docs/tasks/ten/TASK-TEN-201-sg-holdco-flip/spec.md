---
id: TASK-TEN-201
title: "TEN Singapore HoldCo flip CLI — `cyberos-ten holdco-flip` orchestrates ACRA filings + shareholder migration + ESOP transfer for VN → SG corporate restructure"
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
module: TEN
priority: p0
status: draft
verify: T
phase: P3
milestone: P3 · slice 1
slice: 1
owner: Stephen Cheng (CEO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-ESOP-001, TASK-ESOP-006, TASK-TEN-103, TASK-AUTH-101, TASK-MEMORY-111]
depends_on: [TASK-ESOP-001]
blocks: []

source_pages:
  - website/docs/modules/ten.html#sg-holdco-flip
  # ACRA Singapore
  - https://www.acra.gov.sg/

source_decisions:
  - DEC-2400 2026-05-17 — CLI `cyberos-ten holdco-flip` orchestrates the multi-step corporate restructure: (1) form SG HoldCo, (2) prepare ACRA filings, (3) shareholder migration agreements, (4) ESOP grant re-issuance under SG entity, (5) update TASK-TEN-103 residency to sg-1
  - DEC-2401 2026-05-17 — Closed enum `holdco_flip_step` = {pending, sg_entity_formed, acra_filings_prepared, shareholder_agreements_drafted, esop_reissued, residency_migrated, completed, failed}; cardinality 8
  - DEC-2402 2026-05-17 — Each step IMMUTABLE; corrections via new flip (rare); checkpoint per step enables resume
  - DEC-2403 2026-05-17 — Requires CEO + CFO + CLO triple-sign at flip initiation; ACRA + shareholder docs need wet-signature (out of band, status tracked)
  - DEC-2404 2026-05-17 — memory audit kinds: ten.holdco_flip_initiated, ten.holdco_flip_step_completed, ten.holdco_flip_step_failed, ten.holdco_flip_completed

build_envelope:
  language: rust 1.81 + cli
  service: cyberos/services/ten/
  new_files:
    - services/ten/migrations/0010_holdco_flips.sql
    - services/ten/src/holdco/mod.rs
    - services/ten/src/holdco/sg_entity_form.rs
    - services/ten/src/holdco/acra_filings.rs
    - services/ten/src/holdco/shareholder_migration.rs
    - services/ten/src/holdco/esop_reissue.rs
    - services/ten/src/holdco/residency_migrator.rs
    - services/ten/src/holdco/triple_sign_gate.rs
    - services/ten/src/cli/holdco_flip.rs
    - services/ten/src/handlers/holdco_routes.rs
    - services/ten/src/audit/holdco_events.rs
    - services/ten/tests/holdco_flip_step_enum_cardinality_test.rs
    - services/ten/tests/holdco_triple_sign_test.rs
    - services/ten/tests/holdco_resume_from_step_test.rs
    - services/ten/tests/holdco_immutable_test.rs
    - services/ten/tests/holdco_esop_reissue_test.rs
    - services/ten/tests/holdco_audit_emission_test.rs

  modified_files:
    - services/ten/src/lib.rs

  allowed_tools:
    - file_read: services/{ten,esop,auth}/**
    - file_write: services/ten/{src,tests,migrations}/**
    - bash: cd services/ten && cargo test holdco

  disallowed_tools:
    - initiate without triple-sign (per DEC-2403)
    - mutate prior flip step (per DEC-2402)

effort_hours: 16
subtasks:
  - "0.5h: 0010_holdco_flips.sql"
  - "0.5h: holdco/mod.rs"
  - "1.5h: sg_entity_form.rs (BizFile API stub)"
  - "1.5h: acra_filings.rs (PDF generation)"
  - "1.5h: shareholder_migration.rs"
  - "2.0h: esop_reissue.rs (grants under new entity)"
  - "1.0h: residency_migrator.rs (TASK-TEN-103 update)"
  - "0.5h: triple_sign_gate.rs"
  - "1.0h: cli/holdco_flip.rs"
  - "0.5h: handlers/holdco_routes.rs"
  - "0.3h: audit/holdco_events.rs"
  - "4.5h: tests — 6 test files"
  - "0.7h: docs + ACRA integration plan"

risk_if_skipped: "Without HoldCo flip orchestration, VN→SG restructure is bespoke legal+ops effort each time. Without DEC-2402 immutable checkpoints, mid-flip failures unrecoverable. Without DEC-2403 triple-sign, single signer could initiate $100k+ restructure."
---

## §1 — Description (BCP-14 normative)

The TEN service **MUST** ship HoldCo flip CLI at `services/ten/src/holdco/` + `services/ten/src/cli/holdco_flip.rs` orchestrating 6-step restructure with triple-sign + immutable checkpoints, 4 memory audit kinds.

1. **MUST** validate `holdco_flip_step` against closed enum per DEC-2401.

2. **MUST** require CEO + CFO + CLO triple-sign at initiation per DEC-2403 via `triple_sign_gate.rs::can_initiate(flip)`:
   - All three signed
   - Same-person rejected across any two slots
   - Initiation proceeds only after all three

3. **MUST** orchestrate 6 steps via CLI per DEC-2400:
   - sg_entity_form: BizFile API call to form Pte Ltd
   - acra_filings: generate Form 24 + ACRA submissions
   - shareholder_migration: VN shareholders → SG shareholders (signed agreements track)
   - esop_reissue: TASK-ESOP-001 grants re-issued under SG entity
   - residency_migrated: TASK-TEN-103 tenant residency → sg-1
   - completed: all steps done

4. **MUST** be resumable per DEC-2402 — each step writes checkpoint; restart resumes from last completed.

5. **MUST** define tables at migration `0010`:
   ```sql
   CREATE TABLE ten_holdco_flips (
     flip_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     status TEXT NOT NULL DEFAULT 'pending'
       CHECK (status IN ('pending','sg_entity_formed','acra_filings_prepared','shareholder_agreements_drafted','esop_reissued','residency_migrated','completed','failed')),
     ceo_signed_by UUID,
     ceo_signed_at TIMESTAMPTZ,
     cfo_signed_by UUID,
     cfo_signed_at TIMESTAMPTZ,
     clo_signed_by UUID,
     clo_signed_at TIMESTAMPTZ,
     initiated_at TIMESTAMPTZ,
     completed_at TIMESTAMPTZ,
     failure_reason TEXT,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     UNIQUE (tenant_id)  -- one flip per tenant
   );
   ALTER TABLE ten_holdco_flips ENABLE ROW LEVEL SECURITY;
   CREATE POLICY flips_rls ON ten_holdco_flips
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON ten_holdco_flips FROM cyberos_app;
   GRANT UPDATE (status, ceo_signed_by, ceo_signed_at, cfo_signed_by, cfo_signed_at, clo_signed_by, clo_signed_at, initiated_at, completed_at, failure_reason) ON ten_holdco_flips TO cyberos_app;

   CREATE TABLE ten_holdco_flip_steps (
     step_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     flip_id UUID NOT NULL REFERENCES ten_holdco_flips(flip_id),
     step_name TEXT NOT NULL,
     status TEXT NOT NULL DEFAULT 'pending'
       CHECK (status IN ('pending','running','completed','failed')),
     started_at TIMESTAMPTZ,
     completed_at TIMESTAMPTZ,
     output_jsonb JSONB,
     failure_reason TEXT,
     UNIQUE (flip_id, step_name)
   );
   ALTER TABLE ten_holdco_flip_steps ENABLE ROW LEVEL SECURITY;
   CREATE POLICY flip_steps_rls ON ten_holdco_flip_steps
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON ten_holdco_flip_steps FROM cyberos_app;
   GRANT UPDATE (status, started_at, completed_at, output_jsonb, failure_reason) ON ten_holdco_flip_steps TO cyberos_app;
   ```

6. **MUST** expose CLI:
   ```text
   cyberos-ten holdco-flip init --ceo-sign --cfo-sign --clo-sign
   cyberos-ten holdco-flip resume --flip-id <id>
   cyberos-ten holdco-flip status --flip-id <id>
   ```

7. **MUST** emit 4 memory audit kinds per DEC-2404. PII per TASK-MEMORY-111: output_jsonb SHA256.

8. **MUST** thread trace_id from CLI → orchestrator → step → audit.

9. **MUST NOT** initiate without triple-sign per DEC-2403.

10. **MUST NOT** mutate prior step per DEC-2402.

11. **MUST NOT** allow same-person dual sign across slots per DEC-2403.

---

## §2 — Why this design

**Why CLI (DEC-2400)?** HoldCo flip is rare + high-stakes; CLI provides reproducible scripting; full audit trail.

**Why 8-state enum (DEC-2401)?** Captures sequential restructure phases; status visible to ops.

**Why triple-sign (DEC-2403)?** Restructure involves $100k+ in legal + restructure costs; broad governance.

**Why ACRA filings (DEC-2400)?** SG company formation requires statutory submissions; CLI generates standard package.

---

## §3 — API contract

CLI usage:
```bash
$ cyberos-ten holdco-flip init
✓ CEO signature captured
✓ CFO signature captured
✓ CLO signature captured
✓ Flip initiated (flip_id: abc-123)
Resume with: cyberos-ten holdco-flip resume --flip-id abc-123
```

```bash
$ cyberos-ten holdco-flip status --flip-id abc-123
Step: sg_entity_formed ✓
Step: acra_filings_prepared ✓
Step: shareholder_agreements_drafted ⟳ in progress
Step: esop_reissued ⏸ pending
Step: residency_migrated ⏸ pending
```

---

## §4 — Acceptance criteria
1. **holdco_flip_step enum cardinality 8**. 2. **CEO+CFO+CLO triple-sign required**. 3. **Same-person across slots rejected**. 4. **CLI commands (init/resume/status)**. 5. **6-step orchestration**. 6. **Resumable from last completed**. 7. **UNIQUE(tenant_id) — one flip per tenant**. 8. **4 memory audit kinds emitted**. 9. **PII scrubbed (output SHA256)**. 10. **RLS denies cross-tenant**. 11. **Trace_id preserved**. 12. **Append-only via REVOKE except status cols**. 13. **Per-step checkpoint**. 14. **ACRA Form 24 generated**. 15. **ESOP re-issue under SG entity**. 16. **TASK-TEN-103 residency → sg-1 on completion**. 17. **Failed step → flip status=failed; resumable**. 18. **CLI exits non-zero on failure**. 19. **Wet-signature docs tracked out-of-band**. 20. **CLO role required for legal sign**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn triple_sign_required() {
    let ctx = TestContext::new_tenant().await;
    let r = ctx.try_init_flip_with(ctx.ceo, ctx.cfo).await;
    assert!(r.is_err());  // CLO missing
    let r2 = ctx.init_flip_with(ctx.ceo, ctx.cfo, ctx.clo).await;
    assert!(r2.is_ok());
}

#[tokio::test]
async fn resume_from_step() {
    let ctx = TestContext::with_flip_at_step("acra_filings_prepared").await;
    ctx.resume_flip(ctx.flip_id).await;
    let steps = ctx.fetch_steps(ctx.flip_id).await;
    let completed: Vec<_> = steps.iter().filter(|s| s.status == "completed").collect();
    assert!(completed.len() >= 2);  // sg_entity + acra
}

#[tokio::test]
async fn esop_reissue_under_sg() {
    let ctx = TestContext::with_flip_at_esop_reissue().await;
    ctx.run_step("esop_reissued").await;
    let grants = ctx.fetch_grants(ctx.tenant_id).await;
    assert!(grants.iter().all(|g| g.issuer_entity == "sg_holdco"));
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-ESOP-001.
**Cross-module:** TASK-ESOP-006 (acceleration triggers may overlap), TASK-TEN-103 (residency migration), TASK-AUTH-101 (CEO+CFO+CLO roles), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| One sig missing | gate | reject init | get sig |
| Same-person dual | validate | 403 | different signer |
| ACRA API down | client err | step=failed | retry |
| ACRA rejects filing | response | step=failed | fix + resubmit |
| ESOP re-issue partial | atomic per grant | partial; resume | retry |
| Residency migration mid-fail | rollback | sev-1 | manual fix |
| Wet-sig doc missing | track separately | warn + proceed | upload doc |
| Cross-tenant flip | RLS | 403 | inherent |
| Duplicate flip per tenant | UNIQUE | 409 | inherent |
| CLI interrupted | resume from checkpoint | inherent | inherent |

## §11 — Implementation notes
- §11.1 BizFile API integration via Singapore gov OAuth; CTO obtains credentials.
- §11.2 ACRA Form 24 template version-pinned; updates via TASK-DOC-001 templates.
- §11.3 ESOP re-issue creates new TASK-ESOP-001 grants under SG entity with same vesting terms.
- §11.4 memory audit body: flip_id, step_name, status; output SHA256.
- §11.5 Triple-sign: CLI prompts for each signer's session token; backend verifies.

---

*End of TASK-TEN-201 spec.*
