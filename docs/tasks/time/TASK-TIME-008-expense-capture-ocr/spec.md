---
id: TASK-TIME-008
title: "TIME expense capture — photo → AWS Textract OCR → hóa đơn parser → Member confirm + categorisation + invoice integration"
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
module: TIME
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng (CCO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-TIME-001, TASK-DOC-001, TASK-INV-001, TASK-CRM-010, TASK-AI-003, TASK-MEMORY-111]
depends_on: [TASK-CRM-010]
blocks: []

source_pages:
  - website/docs/modules/time.html#expense

source_decisions:
  - DEC-1450 2026-05-17 — Expense capture flow: Member uploads photo via mobile/web → S3 storage via TASK-DOC-001 → AWS Textract OCR → hóa đơn parser extracts VN-specific fields (MST, total VND, date) OR generic receipt fields → Member confirms/edits → expense persisted + (optionally) attached as invoice line
  - DEC-1451 2026-05-17 — Closed enum `expense_kind` = {meal, transport, accommodation, supplies, communication, other_billable, other_non_billable}; cardinality 7
  - DEC-1452 2026-05-17 — Closed enum `expense_status` = {pending_ocr, pending_member_confirm, confirmed, rejected, invoiced}; cardinality 5
  - DEC-1453 2026-05-17 — Member confirmation MANDATORY before persistence (per TASK-MCP-008 elicitation pattern; AI-extracted data needs human verification)
  - DEC-1454 2026-05-17 — Per-engagement reimbursement policy: defines per-kind caps + approval thresholds + currency
  - DEC-1455 2026-05-17 — Async OCR via TASK-MCP-007 Tasks pattern (typical 5-30s); status polling for client UI
  - "DEC-1456 2026-05-17 — Hóa đơn extraction (VN tenants): MST (10-digit), total VND, issued_at, supplier_name, VAT amount; non-VN receipts: generic merchant/total/date"
  - DEC-1457 2026-05-17 — memory audit kinds: time.expense_captured, time.expense_ocr_completed, time.expense_confirmed, time.expense_rejected, time.expense_invoiced, time.expense_ocr_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/time/
  new_files:
    - services/time/migrations/0009_expenses.sql
    - services/time/migrations/0010_expense_policies.sql
    - services/time/src/expense/mod.rs
    - services/time/src/expense/upload.rs
    - services/time/src/expense/ocr_dispatcher.rs
    - services/time/src/expense/hoadon_parser.rs
    - services/time/src/expense/generic_parser.rs
    - services/time/src/expense/policy_check.rs
    - services/time/src/expense/invoice_attach.rs
    - services/time/src/audit/expense_events.rs
    - services/time/src/handlers/expense_routes.rs
    - services/time/web/expense-capture.ts
    - services/time/tests/expense_upload_test.rs
    - services/time/tests/expense_ocr_async_test.rs
    - services/time/tests/expense_hoadon_parse_test.rs
    - services/time/tests/expense_member_confirm_test.rs
    - services/time/tests/expense_policy_cap_test.rs
    - services/time/tests/expense_invoice_attach_test.rs
    - services/time/tests/expense_kind_enum_cardinality_test.rs
    - services/time/tests/expense_status_enum_cardinality_test.rs
    - services/time/tests/expense_no_auto_persist_test.rs
    - services/time/tests/expense_audit_emission_test.rs

  modified_files:
    - services/time/src/lib.rs

  allowed_tools:
    - file_read: services/{time,doc,inv}/**
    - file_write: services/time/{src,tests,migrations,web}/**
    - bash: cd services/time && cargo test expense

  disallowed_tools:
    - persist expense without Member confirm (per DEC-1453)
    - skip policy cap check (per DEC-1454)
    - bill expense to client without engagement billable flag (per TASK-INV-001 integration)

effort_hours: 8
subtasks:
  - "0.4h: 0009_expenses.sql + 0010_expense_policies.sql + 2 closed enums"
  - "0.4h: expense/mod.rs"
  - "0.5h: upload.rs (S3 presigned via TASK-DOC-001)"
  - "0.6h: ocr_dispatcher.rs (TASK-MCP-007 task)"
  - "0.7h: hoadon_parser.rs (VN-specific extraction)"
  - "0.5h: generic_parser.rs"
  - "0.4h: policy_check.rs"
  - "0.5h: invoice_attach.rs"
  - "0.4h: audit/expense_events.rs (6 builders)"
  - "0.4h: handlers/expense_routes.rs"
  - "0.6h: web/expense-capture.ts (camera + upload)"
  - "2.0h: tests — 10 test files"
  - "0.6h: integration smoke against Textract sandbox"

risk_if_skipped: "Without expense capture, Members keep paper receipts → lost reimbursements, lost billable expense lines, audit gaps. Without DEC-1453 confirm gate, OCR errors silently bill clients fake amounts. Without DEC-1454 policy cap, runaway expenses (Member submits $5000 dinner). Without DEC-1456 hóa đơn parsing, VN tax reconciliation fails. The 8h effort is the most complex TIME task (OCR pipeline + 2 parsers + policy + invoice integration)."
---

## §1 — Description (BCP-14 normative)

The TIME service **MUST** ship expense capture pipeline at `services/time/src/expense/` with photo upload → async OCR via TASK-MCP-007 → hóa đơn or generic parsing → Member confirm → policy validation → optional invoice attach, 7-kind enum, 5-state status enum, and 6 memory audit kinds.

1. **MUST** define closed `expense_kind` enum: `('meal','transport','accommodation','supplies','communication','other_billable','other_non_billable')` per DEC-1451. Cardinality 7.

2. **MUST** define closed `expense_status` enum: `('pending_ocr','pending_member_confirm','confirmed','rejected','invoiced')` per DEC-1452. Cardinality 5.

3. **MUST** define `expenses` table at migration `0009`: full row with photo_s3_key, ocr_raw_jsonb, parsed_fields_jsonb (merchant, total_minor, currency, issued_at, vat_amount_minor, mst), confirmed_fields_jsonb (Member-edited final), kind, status, member_subject_id, engagement_id, billable flag, invoice_line_id (when invoiced), trace_id.

4. **MUST** define `expense_policies` table at migration `0010` per DEC-1454: per (engagement, kind) → max_amount_minor, requires_receipt boolean, requires_approval_above_minor, default_billable.

5. **MUST** expose `POST /v1/time/expenses/upload` body `{ engagement_id, kind, expected_currency }`. Handler:
   - Returns TASK-DOC-001 presigned S3 URL.
   - Creates expense row status='pending_ocr'.
   - Emits `time.expense_captured` sev-3.

6. **MUST** trigger async OCR via TASK-MCP-007 Tasks on photo-upload-complete webhook from S3:
   - Task invokes AWS Textract `AnalyzeDocument` with `FORMS` + `TABLES` features.
   - Parses via §1 #7 or §1 #8.
   - Transitions status='pending_member_confirm'.
   - Notifies Member (push via TASK-PORTAL-007 PWA push).
   - Emits `time.expense_ocr_completed` sev-3 OR `time.expense_ocr_failed` sev-2.

7. **MUST** parse hóa đơn (VN tenants) per DEC-1456 via `hoadon_parser.rs`:
   - Extract MST: regex `(?:MST|Mã số thuế)[: ]*(\d{10,13})`.
   - Extract total VND: regex over amount-formatted fields.
   - Extract issued_at: DD/MM/YYYY pattern.
   - Extract supplier_name: header line.
   - Extract VAT: 10% line item.
   - Confidence scores per field.

8. **MUST** parse generic receipt (non-VN tenants) per DEC-1456 via `generic_parser.rs`. Universal fields: merchant_name, total, currency, date.

9. **MUST** require Member confirm per DEC-1453. `POST /v1/time/expenses/{id}/confirm` body `{ confirmed_fields, kind_override?, billable_override? }`. Handler:
   - Validates expense status='pending_member_confirm'.
   - Persists Member-edited final values.
   - Policy check per §1 #10.
   - Transitions status='confirmed'.
   - Emits `time.expense_confirmed` sev-2.

10. **MUST** validate against engagement policy per DEC-1454. If `total_minor > policy.max_amount_minor` → return 412 + `policy_cap_exceeded`. If `> policy.requires_approval_above_minor` → status remains 'pending_member_confirm' with `requires_admin_approval=true` field; engagement_admin approves separately.

11. **MUST** support reject `POST /v1/time/expenses/{id}/reject` body `{ reason }`. Transitions status='rejected'. Emits `time.expense_rejected` sev-3.

12. **MUST** support invoice attach `POST /v1/time/expenses/{id}/attach-to-invoice` body `{ invoice_id }` per DEC-1457 derivative. Caller has `cfo` or `engagement_admin`. Handler:
    - Validates expense status='confirmed' + billable=true.
    - Validates invoice status='draft' or 'ready_for_review'.
    - Creates invoice_line row (line_kind='expense_reimbursement') via TASK-INV-001.
    - Transitions expense status='invoiced' + populates invoice_line_id.
    - Emits `time.expense_invoiced` sev-2.

13. **MUST** emit 6 memory audit kinds per DEC-1457. PII-scrub merchant/supplier name via TASK-MEMORY-111 — hash only in chain.

14. **MUST** thread trace_id end-to-end.

15. **MUST NOT** persist without Member confirm (per DEC-1453).

16. **MUST NOT** auto-bill > policy cap (per DEC-1454).

---

## §2 — Why this design (rationale)

**Why Member confirm mandatory (§1 #9, DEC-1453)?** OCR errors are common (~5-15% field accuracy issues). Auto-billing wrong amounts to clients = trust + revenue + legal risk. Human-in-loop gate.

**Why TASK-MCP-007 async (§1 #6, DEC-1455)?** Textract analyse is 5-30s; sync would tie up gateway workers. Tasks primitive fits.

**Why VN-specific parser (§1 #7, DEC-1456)?** Hóa đơn fields are regulatory-defined (Decree 123); generic OCR misses MST + VAT structure. VN parser produces tax-compliant data; generic parser is fallback.

**Why per-engagement policies (§1 #10, DEC-1454)?** Different engagements have different reimbursement rules. Per-engagement config respects this; without it, one engagement's rules apply to all (wrong).

---

## §3 — API contract

```sql
-- 0009_expenses.sql
CREATE TYPE expense_kind AS ENUM ('meal','transport','accommodation','supplies','communication','other_billable','other_non_billable');
CREATE TYPE expense_status AS ENUM ('pending_ocr','pending_member_confirm','confirmed','rejected','invoiced');

CREATE TABLE expenses (
  expense_id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  member_subject_id UUID NOT NULL,
  engagement_id UUID NOT NULL,
  kind expense_kind NOT NULL,
  status expense_status NOT NULL DEFAULT 'pending_ocr',
  photo_s3_key TEXT NOT NULL,
  ocr_raw_jsonb JSONB,
  parsed_fields_jsonb JSONB,
  confirmed_fields_jsonb JSONB,
  total_minor BIGINT,
  currency billing_currency_enum,
  issued_at TIMESTAMPTZ,
  merchant_name TEXT,
  is_billable BOOLEAN,
  invoice_line_id UUID,
  requires_admin_approval BOOLEAN NOT NULL DEFAULT false,
  reject_reason TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  confirmed_at TIMESTAMPTZ,
  invoiced_at TIMESTAMPTZ,
  trace_id CHAR(32)
);
ALTER TABLE expenses ENABLE ROW LEVEL SECURITY;
CREATE POLICY expenses_rls ON expenses
  USING (tenant_id = current_setting('auth.tenant_id')::uuid
         AND (member_subject_id = current_setting('auth.subject_id')::uuid
              OR EXISTS (SELECT 1 FROM subject_roles WHERE subject_id = current_setting('auth.subject_id')::uuid AND role IN ('engagement_admin','cfo'))))
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid
              AND member_subject_id = current_setting('auth.subject_id')::uuid);
REVOKE DELETE ON expenses FROM cyberos_app;
GRANT UPDATE (status, ocr_raw_jsonb, parsed_fields_jsonb, confirmed_fields_jsonb,
              total_minor, currency, issued_at, merchant_name, is_billable,
              invoice_line_id, requires_admin_approval, reject_reason,
              confirmed_at, invoiced_at) ON expenses TO cyberos_app;

-- 0010_expense_policies.sql
CREATE TABLE expense_policies (
  id BIGSERIAL PRIMARY KEY,
  tenant_id UUID NOT NULL,
  engagement_id UUID NOT NULL,
  kind expense_kind NOT NULL,
  max_amount_minor BIGINT,
  requires_receipt BOOLEAN NOT NULL DEFAULT true,
  requires_approval_above_minor BIGINT,
  default_billable BOOLEAN NOT NULL DEFAULT false,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (engagement_id, kind)
);
ALTER TABLE expense_policies ENABLE ROW LEVEL SECURITY;
CREATE POLICY expense_policies_rls ON expense_policies
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON expense_policies FROM cyberos_app;
```

Endpoints:
```text
POST   /v1/time/expenses/upload                            (member)
POST   /v1/time/expenses/{id}/confirm                      (member)
POST   /v1/time/expenses/{id}/reject                       (member)
POST   /v1/time/expenses/{id}/approve                      (engagement_admin)
POST   /v1/time/expenses/{id}/attach-to-invoice            (cfo or engagement_admin)
GET    /v1/time/expenses?status=...&engagement_id=...      (member or admin)
POST   /v1/admin/engagements/{id}/expense-policy            (engagement_admin)
```

---

## §4 — Acceptance criteria

1. **expense_kind cardinality 7**.
2. **expense_status cardinality 5**.
3. **Upload returns presigned URL** + status=pending_ocr.
4. **Async OCR completes** — Textract result populates parsed_fields_jsonb.
5. **Hóa đơn MST extracted** — VN-format receipt → MST in parsed_fields.
6. **Generic receipt fallback** — non-VN photo → merchant/total/date extracted.
7. **Member confirm required** — no persistence until confirm.
8. **Policy cap enforced** — exceeds max → 412.
9. **Above approval threshold** — requires_admin_approval=true; status stays pending.
10. **Invoice attach creates line** — confirmed expense → invoice line via TASK-INV-001.
11. **Reject transitions** — Member rejects → status=rejected.
12. **6 memory audit kinds emitted**.
13. **PII scrub merchant** — name_hash16 in chain only.
14. **Trace_id end-to-end**.
15. **RLS Member-scoped (engagement_admin/cfo broader)**.
16. **Photo upload max 25 MiB** — S3 upload size limit.
17. **OCR failure path** — Textract error → status remains pending_ocr; manual entry alternative.
18. **VN tenant uses hóa đơn parser** — residency='vn-1' → hoadon_parser invoked.
19. **Non-VN uses generic** — residency≠'vn-1' → generic_parser.
20. **Cross-tenant denied via RLS**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn upload_returns_presigned_url() {
    let ctx = TestContext::with_member().await;
    let r = ctx.post_expense_upload(ctx.eng_id, "meal", "VND").await;
    assert_eq!(r.status(), 201);
    let body: serde_json::Value = r.json().await.unwrap();
    assert!(body["upload_url"].as_str().unwrap().starts_with("https://"));
    assert_eq!(body["status"], "pending_ocr");
}

#[tokio::test]
async fn vn_hoadon_parsed() {
    let ctx = TestContext::with_vn_member().await;
    let expense_id = ctx.upload_and_simulate_ocr_hoadon_image().await;
    ctx.run_ocr_task(expense_id).await;
    let parsed: serde_json::Value = sqlx::query_scalar("SELECT parsed_fields_jsonb FROM expenses WHERE expense_id=$1")
        .bind(expense_id).fetch_one(&ctx.pool).await.unwrap();
    assert!(parsed["mst"].is_string());
    assert!(parsed["total_vnd"].is_number());
}

#[tokio::test]
async fn member_confirm_required_before_invoice() {
    let ctx = TestContext::with_member().await;
    let expense_id = ctx.complete_ocr().await;
    let r = ctx.attach_to_invoice(expense_id, ctx.invoice_id).await;
    assert_eq!(r.status(), 412);  // not yet confirmed
    ctx.member_confirm(expense_id, /*overrides*/ json!({})).await;
    let r2 = ctx.attach_to_invoice(expense_id, ctx.invoice_id).await;
    assert_eq!(r2.status(), 201);
}

#[tokio::test]
async fn policy_cap_blocks() {
    let ctx = TestContext::with_meal_policy_cap(100_000).await;
    let expense_id = ctx.complete_ocr_with_amount(150_000).await;
    let r = ctx.member_confirm(expense_id, json!({})).await;
    assert_eq!(r.status(), 412);
}

// 5.5..5.10: enum cardinality, reject, RLS, audit emit, OCR fail
```

---

## §7 — Dependencies

**Upstream:** TASK-CRM-010 (engagement context).
**Cross-module:** TASK-DOC-001 (S3 storage), TASK-MCP-007 (async OCR task), TASK-PORTAL-007 (push notification), TASK-INV-001 (invoice attach), TASK-AI-003, TASK-MEMORY-111.

---

## §8 — Example payload

`time.expense_ocr_completed`:
```json
{
  "kind": "time.expense_ocr_completed",
  "severity": 3,
  "tenant_id": "8a2f...",
  "actor_id": "system.time.ocr",
  "trace_id": "...",
  "payload": {
    "expense_id": "0190...",
    "member_subject_id_hash16": "f8a1...",
    "parsed_total_minor": 125000,
    "parsed_currency": "VND",
    "merchant_name_hash16": "9c4e...",
    "ocr_confidence_avg": 87
  }
}
```

---

## §9 — Open questions

Deferred:
- **Deferred:** Multi-receipt batch upload — slice 3.
- **Deferred:** Per-Member spending dashboard — slice 3.
- **Deferred:** Smart-categorise (ML kind prediction) — slice 3.
- **Deferred:** Foreign-currency auto-convert at upload — slice 3 (TASK-INV-002 derivative).
- **Deferred:** Mileage tracking (transport with start/end coords) — slice 3.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Textract API quota exceeded | API error | Status remains pending_ocr; sev-2 alert; manual fallback | Quota raised or batch later |
| Photo upload > 25 MiB | S3 size check | 413 | Member compresses |
| Photo not a receipt (random image) | confidence < 30% | Parsed_fields empty; Member edits manually | Inherent |
| MST extraction false-positive (10-digit string elsewhere) | confidence score | Member catches at confirm | Inherent |
| Currency mismatch with engagement | confirm validation | 400 + currency_mismatch | Member fixes |
| Policy cap retro-applied | check at confirm time | Inherent | Member splits or routes to non-billable |
| Approval required but engagement_admin unavailable | sev-3 reminder | Pending indefinitely until admin acts | Email reminder |
| Invoice attach with wrong currency | check at attach | 400 | Inherent |
| OCR task lost (TASK-MCP-007 crash) | task retry | Re-run | Inherent |
| Duplicate expense detection (same receipt twice) | hash photo bytes | Sev-3 alert; user prompted | Inherent |
| Multi-language receipt (English VN mix) | parser fallback | Generic parser tries first | Member edits |
| Hóa đơn lacks MST (informal receipt) | extraction null | Member fills | Inherent |
| Cross-tenant access | RLS | 403 | Inherent |
| Member subject_id changes (rare) | FK soft | Expense retained | Inherent forensic |
| Expense rejected after invoice attach | state check | 409 — already invoiced | Use correction_to via TASK-INV-001 |
| TASK-PORTAL-007 push not sent | best-effort | Email fallback via TASK-EMAIL-001 | Inherent |
| Concurrent confirm + reject | tx isolation | First wins | Inherent |

---

## §11 — Implementation notes

**§11.1** AWS Textract FORMS + TABLES features; per-page pricing.

**§11.2** Hóa đơn parser regex maintained in const HOADON_PATTERNS; quarterly review.

**§11.3** Confidence per field; aggregated for UI display.

**§11.4** S3 photo expires 90d post-confirm (storage cost).

**§11.5** Member confirm UI shows OCR result side-by-side with photo for verification.

**§11.6** Policy check evaluates at confirm (not OCR) — allows kind change to affect policy.

**§11.7** Invoice attach delegates to TASK-INV-001 line-add endpoint.

**§11.8** Push notification via TASK-PORTAL-007 with payload `{ kind: "ocr_complete", expense_id }`.

**§11.9** PII: merchant + supplier names hashed; total amount retained in chain (financial context).

**§11.10** Cross-tenant denied via RLS + explicit subject check.

---

*End of TASK-TIME-008 spec.*
