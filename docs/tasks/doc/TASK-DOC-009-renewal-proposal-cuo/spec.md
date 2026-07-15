---
id: TASK-DOC-009
title: "DOC renewal proposal CUO draft — auto-generate renewal terms + price adjustment + send-to-customer flow with AM approval"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: DOC
priority: p1
status: draft
verify: T
phase: P2
milestone: P2 · slice 1
slice: 1
owner: Stephen Cheng (CLO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-DOC-007, TASK-DOC-008, TASK-CUO-101, TASK-EMAIL-009, TASK-AI-003, TASK-MEMORY-111]
depends_on: [TASK-DOC-007, TASK-CUO-101]
blocks: []

source_pages:
  - website/docs/modules/doc.html#renewal

source_decisions:
  - DEC-1730 2026-05-17 — Triggered at d90 alert (TASK-DOC-008) for contracts with auto_renew=true OR manually by CLO
  - DEC-1731 2026-05-17 — Draft includes: proposed term (matches original or override), price adjustment (CPI-indexed default), renewal scope changes
  - DEC-1732 2026-05-17 — Closed enum `renewal_recommendation` = {auto_renew_as_is, renew_with_price_adj, renew_with_scope_change, do_not_renew}; cardinality 4
  - DEC-1733 2026-05-17 — AM review required before send to customer (NEVER auto-send per TASK-EMAIL-010 dunning pattern)
  - DEC-1734 2026-05-17 — On approval: creates new doc with parent_contract_id pointing to original; status='draft' until signed
  - DEC-1735 2026-05-17 — memory audit kinds: doc.renewal_draft_created, doc.renewal_approved, doc.renewal_dismissed, doc.renewal_sent

build_envelope:
  language: rust 1.81
  service: cyberos/services/doc/
  new_files:
    - services/doc/migrations/0004_renewal_drafts.sql
    - services/doc/src/renewal/mod.rs
    - services/doc/src/renewal/draft_generator.rs
    - services/doc/src/renewal/cpi_adjuster.rs
    - services/doc/src/handlers/renewal_routes.rs
    - services/doc/src/audit/renewal_events.rs
    - services/doc/tests/renewal_triggered_at_d90_test.rs
    - services/doc/tests/renewal_no_auto_send_test.rs
    - services/doc/tests/renewal_recommendation_enum_cardinality_test.rs
    - services/doc/tests/renewal_creates_child_doc_test.rs
    - services/doc/tests/renewal_audit_emission_test.rs

  modified_files:
    - services/doc/src/lib.rs

  allowed_tools:
    - file_read: services/{doc,cuo,email,ai}/**
    - file_write: services/doc/{src,tests,migrations}/**
    - bash: cd services/doc && cargo test renewal

  disallowed_tools:
    - auto-send to customer (per DEC-1733)
    - skip parent_contract_id (per DEC-1734)

effort_hours: 6
subtasks:
  - "0.3h: 0004_renewal_drafts.sql"
  - "0.3h: renewal/mod.rs"
  - "0.7h: draft_generator.rs"
  - "0.4h: cpi_adjuster.rs"
  - "0.4h: handlers/renewal_routes.rs"
  - "0.3h: audit/renewal_events.rs"
  - "1.8h: tests — 5 test files"
  - "1.0h: AM UI for draft review"
  - "0.8h: docs"

risk_if_skipped: "Without renewal drafts, CLO manually drafts each renewal at d90 — operational burden. Without DEC-1733 AM gate, price changes auto-sent to customers (relationship risk). Without DEC-1734 parent link, renewal hangs orphaned (audit fail)."
---

## §1 — Description (BCP-14 normative)

The DOC service **MUST** ship renewal proposal generation at `services/doc/src/renewal/` triggered at TASK-DOC-008 d90 alert or manual, draft via TASK-AI-003, AM review required, creates child doc with parent link, 4 memory audit kinds.

1. **MUST** trigger at d90 alert via TASK-DOC-008 hook OR manual `POST /v1/doc/documents/{id}/draft-renewal` per DEC-1730. Auto-trigger ONLY if `renewal_terms.auto_renew = true`.

2. **MUST** validate `renewal_recommendation` against closed enum per DEC-1732.

3. **MUST** generate draft via `draft_generator.rs::generate(parent_doc)`:
   - Pull parent terms (effective_date, expiry_date, renewal_terms.term_months).
   - Compute new dates: new_effective = old_expiry + 1d; new_expiry = new_effective + term_months.
   - CPI adjust price per `cpi_adjuster.rs::adjust(old_price, residency, since_date)`.
   - AI summarize scope-change recommendation.

4. **MUST** queue for AM review per DEC-1733 — NEVER auto-send to customer.

5. **MUST** on approval per DEC-1734:
   - Create new doc row, `parent_contract_id = original.document_id`.
   - Status='draft'.
   - Lifecycle status compute via TASK-DOC-007 (will be 'draft' if effective > now).

6. **MUST** define table at migration `0004`:
   ```sql
   CREATE TABLE doc_renewal_drafts (
     draft_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     parent_document_id UUID NOT NULL UNIQUE,  -- one active draft per parent
     recommendation TEXT NOT NULL
       CHECK (recommendation IN ('auto_renew_as_is','renew_with_price_adj','renew_with_scope_change','do_not_renew')),
     draft_terms JSONB NOT NULL,
     ai_rationale TEXT,
     status TEXT NOT NULL DEFAULT 'pending_review'
       CHECK (status IN ('pending_review','approved','dismissed','sent','signed')),
     reviewed_by UUID,
     reviewed_at TIMESTAMPTZ,
     child_document_id UUID,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   ALTER TABLE doc_renewal_drafts ENABLE ROW LEVEL SECURITY;
   CREATE POLICY renewal_drafts_rls ON doc_renewal_drafts
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON doc_renewal_drafts FROM cyberos_app;
   GRANT UPDATE (status, reviewed_by, reviewed_at, child_document_id) ON doc_renewal_drafts TO cyberos_app;
   ```

7. **MUST** expose endpoints:
   ```text
   POST   /v1/doc/documents/{id}/draft-renewal   (manual trigger, AM/CLO)
   GET    /v1/doc/renewal-drafts                 (list pending review)
   POST   /v1/doc/renewal-drafts/{id}/approve    (creates child doc)
   POST   /v1/doc/renewal-drafts/{id}/dismiss
   POST   /v1/doc/renewal-drafts/{id}/send       (TASK-EMAIL-009 send after approval)
   ```

8. **MUST** emit 4 memory audit kinds per DEC-1735. PII per TASK-MEMORY-111: terms+rationale SHA-256 hashed; ids ok.

9. **MUST** thread trace_id from trigger → AI → AM review → child create → send → audit.

10. **MUST NOT** auto-send per DEC-1733.

11. **MUST NOT** create child doc without parent_contract_id per DEC-1734.

---

## §2 — Why this design

**Why d90 trigger (DEC-1730)?** Aligns with TASK-DOC-008 first alert; gives full quarter for negotiation.

**Why manual approval (DEC-1733)?** Price changes + scope changes are commercial decisions; AM owns relationship.

**Why parent link (DEC-1734)?** Lineage required for audit + legal traceability ("which contract supersedes which").

**Why CPI adjustment default (DEC-1731)?** Industry-standard escalation; AM overrides per relationship.

---

## §3 — API contract

Sample draft:
```json
{
  "draft_id": "uuid",
  "parent_document_id": "uuid",
  "recommendation": "renew_with_price_adj",
  "draft_terms": {
    "new_effective_date": "2028-01-01",
    "new_expiry_date": "2029-12-31",
    "new_monthly_fee_vnd": 11000000,
    "old_monthly_fee_vnd": 10000000,
    "cpi_adjustment_pct": 10.0,
    "scope_changes": []
  },
  "ai_rationale": "Standard CPI-indexed renewal; no scope changes detected; original account in good standing.",
  "status": "pending_review"
}
```

---

## §4 — Acceptance criteria
1. **Auto-trigger at d90 with auto_renew=true**. 2. **Manual trigger via POST**. 3. **No trigger when auto_renew=false (manual only)**. 4. **AM review required (no auto-send)**. 5. **Approve creates child doc with parent link**. 6. **Status enum 4 + cardinality test**. 7. **CPI adjustment computed**. 8. **4 memory audit kinds emitted**. 9. **PII scrubbed (terms+rationale SHA256)**. 10. **RLS denies cross-tenant**. 11. **AM/CLO role only**. 12. **Trace_id preserved**. 13. **UNIQUE on parent_document_id (one active draft)**. 14. **Send endpoint requires status=approved**. 15. **Append-only via REVOKE except 4 cols**. 16. **Dismiss → status=dismissed**. 17. **Recommendation enum 4 values**. 18. **AI failure → status=failed + sev-2 + retry**. 19. **Child doc inherits parties (with refresh prompt)**. 20. **Send via TASK-EMAIL-009 with renewal template**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn triggers_at_d90_with_auto_renew() {
    let ctx = TestContext::doc_expires_in_90d_with_auto_renew().await;
    ctx.run_d90_alert(ctx.doc_id).await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    let drafts = ctx.fetch_renewal_drafts(ctx.doc_id).await;
    assert_eq!(drafts.len(), 1);
}

#[tokio::test]
async fn never_auto_sends() {
    let ctx = TestContext::with_renewal_draft().await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    let sent_count = ctx.email_send_count().await;
    assert_eq!(sent_count, 0);
}

#[tokio::test]
async fn approve_creates_child_doc() {
    let ctx = TestContext::with_renewal_draft().await;
    ctx.approve_draft(ctx.draft_id).await;
    let row = ctx.fetch_draft(ctx.draft_id).await;
    assert!(row.child_document_id.is_some());
    let child = ctx.fetch_doc(row.child_document_id.unwrap()).await;
    assert_eq!(child.parent_contract_id, Some(ctx.parent_doc_id));
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-DOC-007, TASK-CUO-101.
**Cross-module:** TASK-DOC-008 (d90 trigger), TASK-EMAIL-009 (send), TASK-AI-003 (draft+rationale), TASK-AUTH-101 (AM/CLO role), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| AI timeout | retry 1x | sev-2 fallback minimal draft | AM completes manually |
| CPI source unavailable | use 5% default | sev-2 | inherent |
| Duplicate draft race | UNIQUE on parent_doc_id | second skipped | inherent |
| Auto_renew=false unexpected | check | no auto trigger | inherent |
| Approve race | UPDATE WHERE pending | first wins | inherent |
| Child doc create fails | rollback approval | sev-1 | retry |
| Parent already renewed | check existing chain | reject | inherent |
| AM not assigned | warn + queue | inherent | reassign |
| Cross-tenant query | RLS | 0 rows | inherent |
| Send fails | retry | sev-2 | manual resend |

## §11 — Implementation notes
- §11.1 CPI: per-residency lookup table (vn-1: VN CPI, sg-1: SG CPI, eu-1: EU HICP, us-1: US CPI-U).
- §11.2 AI rationale prompt: "Summarize this renewal proposal (scope/price changes) in 2-3 sentences."
- §11.3 Child doc inherits parties array; AM prompted to confirm/update before send.
- §11.4 memory audit body: parent_doc_id, recommendation; draft_terms SHA256.
- §11.5 Send templates: branded per tenant (TASK-PORTAL-002 brand pack).

---

*End of TASK-DOC-009 spec.*
