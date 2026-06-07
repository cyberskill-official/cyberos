---
id: FR-INV-008
title: "INV VN hóa đơn cancellation flow — Decree 123 Art. 19 replacement-or-cancellation protocol with GDT 1-1 mapping + amendment audit"
module: INV
priority: MUST
status: ready_to_implement
verify: T
phase: P2
milestone: P2 · slice 2
slice: 2
owner: Stephen Cheng (CFO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-INV-007, FR-INV-001, FR-TEN-102, FR-AI-003, FR-MEMORY-111]
depends_on: [FR-INV-007]
blocks: []

source_pages:
  - website/docs/modules/inv.html#hoadon-cancel
  - https://gdt.gov.vn/  # Decree 123 Art. 19 reference

source_decisions:
  - DEC-1530 2026-05-17 — Decree 123 Art. 19: cancel hóa đơn requires customer agreement letter (biên bản) + GDT cancellation form 04/SS-HDDT
  - DEC-1531 2026-05-17 — Two flows: (a) "replace" — cancel + new hóa đơn issued (when error in original); (b) "cancel" — terminate without replacement (rare, e.g. cancelled engagement)
  - DEC-1532 2026-05-17 — Closed enum `cancel_reason` = {error_correction, customer_dispute, engagement_terminated, duplicate_emission}; cardinality 4
  - DEC-1533 2026-05-17 — Cancellation form (04/SS-HDDT) auto-generated from CFO-supplied reason + customer agreement upload; submitted to GDT within 24h of decision
  - DEC-1534 2026-05-17 — Original hóa đơn row updated to hoadon_status='cancelled' + replacement_hoadon_id pointer (if applicable); never deleted (audit trail)
  - DEC-1535 2026-05-17 — memory audit kinds: inv.hoadon_cancel_initiated, inv.hoadon_cancel_form_submitted, inv.hoadon_cancel_accepted, inv.hoadon_cancel_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/invoicing/
  new_files:
    - services/invoicing/migrations/0008_vn_hoadon_cancellation.sql
    - services/invoicing/src/hoadon/cancel.rs
    - services/invoicing/src/hoadon/form_04_builder.rs
    - services/invoicing/src/handlers/hoadon_cancel_routes.rs
    - services/invoicing/src/audit/hoadon_cancel_events.rs
    - services/invoicing/tests/hoadon_replace_flow_test.rs
    - services/invoicing/tests/hoadon_cancel_flow_test.rs
    - services/invoicing/tests/hoadon_cancel_reason_enum_cardinality_test.rs
    - services/invoicing/tests/hoadon_cancel_24h_window_test.rs
    - services/invoicing/tests/hoadon_cancel_audit_emission_test.rs

  modified_files:
    - services/invoicing/src/hoadon/mod.rs
    - services/invoicing/src/handlers/hoadon_routes.rs

  allowed_tools:
    - file_read: services/invoicing/**
    - file_write: services/invoicing/{src,tests,migrations}/**
    - bash: cd services/invoicing && cargo test cancel

  disallowed_tools:
    - cancel without customer agreement upload (per DEC-1530)
    - delete original hoadon row (per DEC-1534)
    - skip form 04/SS-HDDT submission (per DEC-1533)

effort_hours: 5
sub_tasks:
  - "0.3h: 0008_vn_hoadon_cancellation.sql"
  - "0.5h: cancel.rs"
  - "0.6h: form_04_builder.rs (Decree 123 Annex 1 form 04/SS-HDDT)"
  - "0.4h: handlers/hoadon_cancel_routes.rs"
  - "0.3h: audit/hoadon_cancel_events.rs"
  - "0.3h: mod.rs hooks for replacement-link"
  - "1.6h: tests — 5 test files"
  - "1.0h: CFO UI + customer agreement upload integration"

risk_if_skipped: "Without VN hóa đơn cancellation, CFO cannot legally correct erroneous hóa đơn — must rely on manual GDT portal (compliance burden). Without DEC-1530 customer agreement, cancellation is invalid per Decree 123 Art. 19. Without DEC-1534 audit trail, replaced hóa đơn lineage is lost (audit failure)."
---

## §1 — Description (BCP-14 normative)

The INV service **MUST** ship VN hóa đơn cancellation at `services/invoicing/src/hoadon/cancel.rs` supporting replace + cancel flows, customer agreement upload, GDT form 04/SS-HDDT submission, replacement-pointer audit trail, 4 memory audit kinds.

1. **MUST** expose `POST /v1/inv/hoadon/{id}/cancel` body `{ reason, customer_agreement_doc_id, replacement_invoice_id?, notes }` — CFO-role only via FR-AUTH-101.

2. **MUST** validate reason against closed enum per DEC-1532; reject invalid values 400.

3. **MUST** require `customer_agreement_doc_id` (FR-DOC-001 reference) per DEC-1530 — without it, return 422.

4. **MUST** build form 04/SS-HDDT at `form_04_builder.rs::build(cancellation)` per Decree 123 Annex 1 — root `<TBao>` with `<DLTBao>` (notification data) + `<DSHDon>` (cancelled hóa đơn list).

5. **MUST** submit form to GDT within 24h of decision per DEC-1533 — via FR-MCP-007 task (GDT may ack async).

6. **MUST** update original `vn_hoadon` row to `hoadon_status='cancelled'`, set `replacement_hoadon_id` if replace flow per DEC-1534. Never delete row.

7. **MUST** define table extension at migration `0008`:
   ```sql
   ALTER TABLE vn_hoadon ADD COLUMN replacement_hoadon_id UUID REFERENCES vn_hoadon(hoadon_id);
   ALTER TABLE vn_hoadon ADD COLUMN cancel_reason TEXT
     CHECK (cancel_reason IS NULL OR cancel_reason IN
       ('error_correction','customer_dispute','engagement_terminated','duplicate_emission'));
   ALTER TABLE vn_hoadon ADD COLUMN cancellation_form_id UUID;
   ALTER TABLE vn_hoadon ADD COLUMN customer_agreement_doc_id UUID;
   ALTER TABLE vn_hoadon ADD COLUMN cancelled_at TIMESTAMPTZ;
   GRANT UPDATE (replacement_hoadon_id, cancel_reason, cancellation_form_id,
                 customer_agreement_doc_id, cancelled_at, hoadon_status, updated_at) ON vn_hoadon TO cyberos_app;

   CREATE TABLE vn_hoadon_cancellation_forms (
     form_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     cancelled_hoadon_ids UUID[] NOT NULL,
     reason TEXT NOT NULL,
     form_xml BYTEA NOT NULL,
     gdt_response JSONB,
     status TEXT NOT NULL DEFAULT 'pending'
       CHECK (status IN ('pending','submitted','accepted','rejected')),
     submitted_at TIMESTAMPTZ,
     accepted_at TIMESTAMPTZ,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   ALTER TABLE vn_hoadon_cancellation_forms ENABLE ROW LEVEL SECURITY;
   CREATE POLICY cancel_forms_rls ON vn_hoadon_cancellation_forms
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON vn_hoadon_cancellation_forms FROM cyberos_app;
   GRANT UPDATE (status, gdt_response, submitted_at, accepted_at) ON vn_hoadon_cancellation_forms TO cyberos_app;
   ```

8. **MUST** support replace flow: when `replacement_invoice_id` supplied, trigger FR-INV-007 emit for replacement BEFORE marking original cancelled; ensures continuity.

9. **MUST** emit 4 memory audit kinds per DEC-1535. PII per FR-MEMORY-111 (reason text scrubbed of customer names → SHA256 hash of full notes).

10. **MUST** thread trace_id from CFO action → cancel → form submit → GDT response.

11. **MUST NOT** allow cancel of `accepted` hóa đơn without customer agreement (DEC-1530); `pending` or `rejected` may cancel without it (admin discretion).

12. **MUST NOT** delete the original row per DEC-1534.

---

## §2 — Why this design

**Why customer agreement required (DEC-1530)?** Per Decree 123 Art. 19, cancellation without customer biên bản is legally void; GDT will reject form 04.

**Why replace + cancel separate flows (DEC-1531)?** Replace preserves AR continuity (new invoice issued); cancel terminates. Distinct legal treatments.

**Why append-only with replacement_hoadon_id (DEC-1534)?** Audit lineage — accountant must trace original → replacement chain for VAT reconciliation.

---

## §3 — API contract

Endpoints:
```text
POST   /v1/inv/hoadon/{id}/cancel       (CFO-only)
GET    /v1/inv/hoadon/{id}/cancellation  (status + form)
GET    /v1/inv/cancellation-forms        (list pending/submitted/accepted)
```

Sample request:
```json
{
  "reason": "error_correction",
  "customer_agreement_doc_id": "uuid-of-uploaded-biên-bản",
  "replacement_invoice_id": "uuid-of-new-invoice",
  "notes": "Customer disputed line item 3 quantity; corrected from 100 to 80"
}
```

Form 04/SS-HDDT (Decree 123 Annex 1):
```xml
<TBao xmlns="http://kekhaithue.gdt.gov.vn/TBaoSaiSot">
  <DLTBao>
    <TTChung><MCQT>{tax_id}</MCQT><MLTBao>1</MLTBao></TTChung>
    <NDTBao>
      <DSHDon>
        <HDon><MSHDon>K24TAA-00000001</MSHDon><LDo>1</LDo></HDon>
      </DSHDon>
    </NDTBao>
  </DLTBao>
  <DSCKS>...</DSCKS>
</TBao>
```

---

## §4 — Acceptance criteria
1. **CFO-only access** (FR-AUTH-101 enforced). 2. **Reason enum 4 values + cardinality test**. 3. **Customer agreement required for accepted hóa đơn**. 4. **Replace flow issues new hóa đơn first**. 5. **Original row updated, never deleted**. 6. **Form 04 schema valid per Decree 123 Annex 1**. 7. **Form submitted to GDT within 24h**. 8. **GDT async via FR-MCP-007**. 9. **4 memory audit kinds emitted**. 10. **PII scrubbed (notes → SHA256)**. 11. **RLS denies cross-tenant**. 12. **Replacement chain pointer queryable**. 13. **Append-only on form table**. 14. **GDT rejection → form status=rejected + CFO notification**. 15. **Cancellation of pending/rejected hóa đơn allowed without agreement**. 16. **Trace_id propagated**. 17. **Duplicate cancel attempt rejected (already cancelled)**. 18. **Form xml signed via tenant KMS cert**. 19. **24h window enforced (audit if past)**. 20. **Cancellation event broadcast to FR-CHAT-010 if customer-facing channel exists**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn replace_flow_issues_new_then_cancels_original() {
    let ctx = TestContext::vn_tenant_with_accepted_hoadon().await;
    let new_inv = ctx.create_replacement_invoice().await;
    let resp = ctx.cancel_hoadon(ctx.original_id, "error_correction",
        ctx.agreement_doc, Some(new_inv)).await;
    assert_eq!(resp.status, 202);
    let original = ctx.fetch_hoadon(ctx.original_id).await;
    assert_eq!(original.hoadon_status, "cancelled");
    let new_h = ctx.fetch_hoadon_by_invoice(new_inv).await;
    assert_eq!(new_h.hoadon_status, "transmitted");
    assert_eq!(original.replacement_hoadon_id, Some(new_h.hoadon_id));
}

#[tokio::test]
async fn cancel_accepted_requires_agreement() {
    let ctx = TestContext::vn_tenant_with_accepted_hoadon().await;
    let resp = ctx.cancel_hoadon_without_agreement(ctx.original_id, "customer_dispute").await;
    assert_eq!(resp.status, 422);
}

#[tokio::test]
async fn form_submitted_within_24h() {
    let ctx = TestContext::vn_tenant_with_cancellation().await;
    let form: VnHoadonCancelForm = ctx.fetch_form(ctx.form_id).await;
    let elapsed = form.submitted_at.unwrap() - form.created_at;
    assert!(elapsed < Duration::hours(24));
}

// 5.4..5.10
```

---

## §6 — Skeleton

```rust
pub async fn cancel(req: CancelRequest, actor: &CfoActor, db: &Db) -> Result<CancelResponse> {
    let original = db.fetch_hoadon(req.hoadon_id).await?;
    if original.hoadon_status == "accepted" && req.customer_agreement_doc_id.is_none() {
        return Err(CancelError::AgreementRequired.into());
    }
    if let Some(replacement_invoice_id) = req.replacement_invoice_id {
        let new_hoadon = super::emit(replacement_invoice_id, actor.tenant(), db).await?;
        db.set_replacement_pointer(req.hoadon_id, new_hoadon).await?;
    }
    let form_xml = form_04_builder::build(&original, &req)?;
    let form_id = db.insert_cancellation_form(&original, &req, &form_xml).await?;
    let trace = current_span_trace_id();
    audit::emit("inv.hoadon_cancel_initiated", json!({"hoadon_id": req.hoadon_id, "reason": req.reason}), trace).await?;
    queue_gdt_submission(form_id, form_xml, actor.tenant().clone()).await?;
    db.mark_hoadon_cancelled(req.hoadon_id, form_id, &req).await?;
    Ok(CancelResponse{form_id, ..})
}
```

---

## §7 — Dependencies
**Upstream:** FR-INV-007.
**Cross-module:** FR-AUTH-101 (CFO role), FR-DOC-001 (agreement upload), FR-MCP-007 (async), FR-MEMORY-111 (PII), FR-CHAT-010 (customer notification).

## §8 — Sample payloads (see §3)

## §9 — Open questions
None blocking.

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Agreement doc missing | early validate | 422 | CFO uploads doc |
| Invalid reason enum | CHECK constraint | 400 | use valid enum |
| Original not cancellable (already cancelled) | state check | 409 | inherent |
| GDT 4xx on form | response code | status=rejected; sev-2 | CFO fixes + resubmit |
| GDT 5xx | retry 3x w/ backoff | retry | inherent |
| Replacement invoice doesn't exist | FK violation | 404 | provide valid id |
| Form > 24h window | audit | sev-2 + submit anyway | post-hoc explanation |
| KMS sign fail on form | sign error | status=pending; sev-1 | KMS recovery |
| Concurrent cancel attempts | UNIQUE on form for hoadon | second 409 | inherent |
| Customer changes mind mid-flow | manual CFO action | revert via new emit | CFO escalation |
| GDT acceptance race vs replacement | sequence | replacement first, then cancel | inherent (order in skeleton) |

## §11 — Implementation notes
- §11.1 Form 04/SS-HDDT signed with same tenant KMS cert as original emission.
- §11.2 GDT acknowledges form ~5min; CFO sees status update on dashboard.
- §11.3 Customer agreement doc retention: 10 years per VN accounting law.
- §11.4 Replace flow ordering — new emit BEFORE cancel ensures we never leave gap in VAT period.
- §11.5 memory audit: cancel_reason in chain (it's enum, no PII); notes hashed.

---

*End of FR-INV-008 spec.*
