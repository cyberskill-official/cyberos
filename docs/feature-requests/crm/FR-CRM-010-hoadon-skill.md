---
id: FR-CRM-010
title: "CRM vietnam-vat-invoice skill — Decree 123 hóa đơn auto-emit on deal.stage=won + invoice issuance + verification code retrieval"
module: CRM
priority: MUST
status: draft
verify: T
phase: P1
milestone: P1 · slice 7
slice: 7
owner: Stephen Cheng (CFO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-CRM-004, FR-INV-001, FR-INV-007, FR-SKILL-109, FR-MEMORY-111]
depends_on: [FR-INV-007, FR-CRM-004]
blocks: [FR-TIME-008]

source_pages:
  - website/docs/modules/crm.html#hoadon
  - https://gdt.gov.vn/  # Decree 123/2020

source_decisions:
  - DEC-1700 2026-05-17 — Skill name: vietnam-vat-invoice@1; triggered on deal.stage=won for VN tenants; creates invoice via FR-INV-001 + emits hóa đơn via FR-INV-007
  - DEC-1701 2026-05-17 — Closed enum `vat_invoice_trigger` = {deal_won_auto, manual_emit, retry_on_failure}; cardinality 3
  - DEC-1702 2026-05-17 — Idempotency: one hóa đơn per deal; UNIQUE(deal_id) on emission table
  - DEC-1703 2026-05-17 — Delegates to FR-INV-007 for actual GDT emit — this skill orchestrates: create invoice → set vn-residency → trigger emit → return status
  - DEC-1704 2026-05-17 — memory audit kinds: crm.vat_invoice_skill_invoked, crm.vat_invoice_invoice_created, crm.vat_invoice_emit_delegated, crm.vat_invoice_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/crm/
  new_files:
    - services/crm/migrations/0010_vat_invoice_emissions.sql
    - services/crm/src/vn/vat_invoice_skill.rs
    - services/crm/src/vn/invoice_orchestrator.rs
    - services/crm/src/audit/vat_invoice_events.rs
    - services/crm/tests/vat_invoice_auto_on_won_test.rs
    - services/crm/tests/vat_invoice_idempotent_test.rs
    - services/crm/tests/vat_invoice_non_vn_skipped_test.rs
    - services/crm/tests/vat_invoice_trigger_enum_cardinality_test.rs
    - services/crm/tests/vat_invoice_audit_emission_test.rs

  modified_files:
    - services/crm/src/deals.rs

  allowed_tools:
    - file_read: services/{crm,inv,skill}/**
    - file_write: services/crm/{src,tests,migrations}/**
    - bash: cd services/crm && cargo test vat_invoice

  disallowed_tools:
    - emit for non-VN tenant (per DEC-1700)
    - duplicate emit per deal (per DEC-1702)

effort_hours: 5
sub_tasks:
  - "0.3h: 0010_vat_invoice_emissions.sql"
  - "0.5h: vat_invoice_skill.rs"
  - "0.7h: invoice_orchestrator.rs"
  - "0.3h: audit/vat_invoice_events.rs"
  - "0.3h: deals.rs hook on won"
  - "1.6h: tests — 5 test files"
  - "1.3h: integration with FR-INV-007 + smoke test"

risk_if_skipped: "Without CRM-side trigger, CFO must manually create invoice + emit hóa đơn for every won deal (operational burden). Without DEC-1702 idempotency, retry storms cause duplicate emit (illegal per Decree 123). Without DEC-1703 delegation, CRM duplicates hóa đơn logic (FR-INV-007 is canonical)."
---

## §1 — Description (BCP-14 normative)

The CRM service **MUST** ship vietnam-vat-invoice@1 skill at `services/crm/src/vn/vat_invoice_skill.rs` triggered on deal.stage=won for VN tenants, creating invoice via FR-INV-001 + delegating emit to FR-INV-007, idempotent, 4 memory audit kinds.

1. **MUST** register skill `vietnam-vat-invoice@1` per DEC-1700.

2. **MUST** hook into deal stage transitions at `services/crm/src/deals.rs` — on `won` AND `tenant.residency='vn-1'` AND `account.vn_account_type IS NOT NULL`, invoke skill async.

3. **MUST** validate `vat_invoice_trigger` against closed enum per DEC-1701.

4. **MUST** orchestrate at `invoice_orchestrator.rs::orchestrate(deal, trigger)`:
   - Check if existing emission row (idempotent per DEC-1702).
   - Call FR-INV-001 create invoice (account_id, contact_id, line items from deal).
   - Call FR-INV-007 hóa đơn emit (delegation per DEC-1703).
   - Store emission record with status.

5. **MUST** define table at migration `0010`:
   ```sql
   CREATE TABLE crm_vat_invoice_emissions (
     emission_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     deal_id UUID NOT NULL UNIQUE,  -- idempotent per DEC-1702
     invoice_id UUID,
     hoadon_id UUID,
     trigger TEXT NOT NULL
       CHECK (trigger IN ('deal_won_auto','manual_emit','retry_on_failure')),
     status TEXT NOT NULL DEFAULT 'pending'
       CHECK (status IN ('pending','invoice_created','emit_delegated','accepted','failed')),
     failure_reason TEXT,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   ALTER TABLE crm_vat_invoice_emissions ENABLE ROW LEVEL SECURITY;
   CREATE POLICY emissions_rls ON crm_vat_invoice_emissions
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON crm_vat_invoice_emissions FROM cyberos_app;
   GRANT UPDATE (invoice_id, hoadon_id, status, failure_reason, updated_at) ON crm_vat_invoice_emissions TO cyberos_app;
   ```

6. **MUST** expose skill endpoint:
   ```text
   POST   /v1/crm/skill/vietnam-vat-invoice
         body: {deal_id, trigger: 'manual_emit'|'retry_on_failure'}
   GET    /v1/crm/skill/vietnam-vat-invoice/emissions/{deal_id}
   ```

7. **MUST** emit 4 memory audit kinds per DEC-1704. PII per FR-MEMORY-111: deal_value SHA-256 hashed; ids ok.

8. **MUST** thread trace_id from deal-hook / manual call → orchestrator → FR-INV-001 → FR-INV-007 → audit.

9. **MUST** be silent skip on non-VN tenant per DEC-1700 — no emission row created.

10. **MUST NOT** duplicate emit per deal per DEC-1702 — UNIQUE constraint enforces.

11. **MUST NOT** reimplement FR-INV-007 logic per DEC-1703 — orchestration only.

---

## §2 — Why this design

**Why orchestration (DEC-1703)?** FR-INV-007 is canonical hóa đơn emit; CRM-010 wires the trigger. Duplication = drift.

**Why idempotent (DEC-1702)?** Deal stage may transition won→re-won (e.g. correction); we must not create two hóa đơn.

**Why deal.stage=won trigger (DEC-1700)?** Decree 123 requires hóa đơn at point of legal commitment; won deal = commitment.

**Why optional manual trigger (DEC-1701)?** Edge cases: deal won pre-system, CFO retroactively creates; manual path needed.

---

## §3 — API contract

```text
POST   /v1/crm/skill/vietnam-vat-invoice       (manual call)
GET    /v1/crm/skill/vietnam-vat-invoice/emissions/{deal_id}
```

Sample manual request:
```json
{
  "deal_id": "uuid",
  "trigger": "manual_emit"
}
```

Sample emission status:
```json
{
  "emission_id": "uuid",
  "deal_id": "uuid",
  "invoice_id": "uuid",
  "hoadon_id": "uuid",
  "status": "accepted",
  "trigger": "deal_won_auto",
  "gdt_verification_code": "ABCD1234EFGH"
}
```

---

## §4 — Acceptance criteria
1. **Auto-trigger on stage=won for VN tenant**. 2. **Non-VN tenant → silent skip**. 3. **Account missing vn_account_type → skip + sev-3 audit**. 4. **3-trigger enum + cardinality test**. 5. **Idempotent (UNIQUE deal_id)**. 6. **FR-INV-001 invoice created with deal context**. 7. **FR-INV-007 emit delegated**. 8. **4 memory audit kinds emitted**. 9. **PII scrubbed (deal_value SHA256)**. 10. **RLS denies cross-tenant**. 11. **Trace_id preserved**. 12. **Status transitions tracked (pending→invoice_created→emit_delegated→accepted)**. 13. **FR-INV-007 failure → status=failed; CFO sees**. 14. **Manual trigger CFO-only**. 15. **Append-only via REVOKE UPDATE except status cols**. 16. **GET endpoint returns emission status**. 17. **Stage revert won→negotiating → no auto-cancel hóa đơn (CFO uses FR-INV-008)**. 18. **Retry trigger re-invokes FR-INV-007 (uses existing invoice_id)**. 19. **Hoadon_id stable across retries**. 20. **Deal value 0 → still emits (Decree 123 allows zero-value)**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn auto_emits_on_vn_deal_won() {
    let ctx = TestContext::vn_tenant_with_deal_in_proposal().await;
    ctx.change_deal_stage(ctx.deal_id, "won").await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    let emission = ctx.fetch_emission(ctx.deal_id).await.unwrap();
    assert!(emission.invoice_id.is_some());
    assert!(emission.hoadon_id.is_some());
}

#[tokio::test]
async fn idempotent_duplicate_won() {
    let ctx = TestContext::vn_tenant_with_deal().await;
    ctx.change_deal_stage(ctx.deal_id, "won").await;
    ctx.change_deal_stage(ctx.deal_id, "negotiating").await;
    ctx.change_deal_stage(ctx.deal_id, "won").await;  // re-won
    let emissions = ctx.fetch_all_emissions(ctx.deal_id).await;
    assert_eq!(emissions.len(), 1);
}

#[tokio::test]
async fn skips_non_vn_tenant() {
    let ctx = TestContext::sg_tenant_with_deal().await;
    ctx.change_deal_stage(ctx.deal_id, "won").await;
    let emission = ctx.fetch_emission(ctx.deal_id).await;
    assert!(emission.is_none());
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-INV-007, FR-CRM-004.
**Cross-module:** FR-INV-001 (invoice create), FR-SKILL-109 (registry), FR-AUTH-101 (CFO role for manual), FR-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Non-VN tenant | residency check | silent skip | inherent |
| vn_account_type missing | early validate | sev-3; skip | CRO fills field |
| FR-INV-001 create fails | downstream err | status=failed; sev-2 | CFO investigates |
| FR-INV-007 emit fails | downstream err | status=failed; sev-2 | CFO retry |
| Duplicate won race | UNIQUE constraint | second skipped | inherent |
| Stage revert won→neg | no auto-cancel | CFO uses FR-INV-008 manually | by design |
| Deal value 0 | proceed (Decree 123 OK) | inherent | inherent |
| GDT acceptance pending | status=emit_delegated | poll via FR-INV-007 | inherent |
| Manual retry on accepted | UNIQUE rejects | 409 | inherent |
| Non-CFO manual call | role check | 403 | request CFO |
| Cross-tenant lookup | RLS | 404 | inherent |

## §11 — Implementation notes
- §11.1 Orchestrator first creates invoice (FR-INV-001), then calls FR-INV-007 emit; status reflects which step we're at.
- §11.2 Invoice line items derived from deal.proposed_items (CRM stores proposal line items per deal).
- §11.3 memory audit body: deal_id, trigger, invoice_id, hoadon_id; amounts SHA256.
- §11.4 Manual retry path: looks up existing invoice_id, calls FR-INV-007 emit (which is itself idempotent).
- §11.5 No auto-cancel on stage revert — CFO must explicitly use FR-INV-008 cancellation flow (Decree 123 Art. 19 requirements).

---

*End of FR-CRM-010 spec.*
