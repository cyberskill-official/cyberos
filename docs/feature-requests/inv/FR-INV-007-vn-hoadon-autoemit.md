---
id: FR-INV-007
title: "INV VN hóa đơn auto-emit on AM-send — Decree 123/2020 GDT XML signing + idempotent transmission + verification code retrieval for VN tenants"
module: INV
priority: MUST
status: draft
verify: T
phase: P2
milestone: P2 · slice 2
slice: 2
owner: Stephen Cheng (CFO)
created: 2026-05-17
shipped: null
brain_chain_hash: null
related_frs: [FR-INV-001, FR-INV-002, FR-INV-008, FR-TEN-102, FR-AI-003, FR-BRAIN-111, FR-SKILL-109]
depends_on: [FR-INV-001]
blocks: [FR-INV-008, FR-CRM-010]

source_pages:
  - website/docs/modules/inv.html#hoadon
  - https://gdt.gov.vn/  # General Department of Taxation Vietnam
  - https://thuvienphapluat.vn/van-ban/Thue-Phi-Le-Phi/Nghi-dinh-123-2020-ND-CP  # Decree 123/2020 reference

source_decisions:
  - DEC-1520 2026-05-17 — Auto-emit triggered ONLY on first AM-send for VN tenants (residency=vn-1) with engagement.billing_currency=VND; non-VN tenants skipped
  - DEC-1521 2026-05-17 — XML format per Decree 123 Annex 1; signing via tenant's GDT-registered digital cert held in HSM/KMS
  - DEC-1522 2026-05-17 — Idempotency: hóa đơn emit keyed by invoice_id; replay returns same hoadon_id + verification_code; never duplicates
  - DEC-1523 2026-05-17 — Closed enum `hoadon_status` = {pending, signed, transmitted, accepted, rejected, cancelled}; cardinality 6
  - DEC-1524 2026-05-17 — GDT transmission via async FR-MCP-007 task — GDT can take 5min-2hr to issue verification code; retry on transient failure with exponential backoff
  - DEC-1525 2026-05-17 — Failure → invoice remains sent but hoadon_status=rejected; CFO sees notification + must resolve before VN tax filing deadline (monthly)
  - DEC-1526 2026-05-17 — BRAIN audit kinds: inv.hoadon_emit_started, inv.hoadon_signed, inv.hoadon_transmitted, inv.hoadon_verification_received, inv.hoadon_emit_failed
  - DEC-1527 2026-05-17 — Per-tenant config: gdt_registration_id, signing_cert_kms_arn, gdt_environment (sandbox/prod); ROOT-CFO writes via separate FR-INV-013 admin endpoint

build_envelope:
  language: rust 1.81
  service: cyberos/services/invoicing/
  new_files:
    - services/invoicing/migrations/0007_vn_hoadon.sql
    - services/invoicing/src/hoadon/mod.rs
    - services/invoicing/src/hoadon/xml_builder.rs
    - services/invoicing/src/hoadon/signer.rs
    - services/invoicing/src/hoadon/gdt_client.rs
    - services/invoicing/src/hoadon/poller.rs
    - services/invoicing/src/audit/hoadon_events.rs
    - services/invoicing/src/handlers/hoadon_routes.rs
    - services/invoicing/tests/hoadon_emit_on_send_test.rs
    - services/invoicing/tests/hoadon_idempotency_test.rs
    - services/invoicing/tests/hoadon_non_vn_skipped_test.rs
    - services/invoicing/tests/hoadon_xml_format_test.rs
    - services/invoicing/tests/hoadon_signing_test.rs
    - services/invoicing/tests/hoadon_gdt_retry_test.rs
    - services/invoicing/tests/hoadon_status_enum_cardinality_test.rs
    - services/invoicing/tests/hoadon_audit_emission_test.rs

  modified_files:
    - services/invoicing/src/lib.rs
    - services/invoicing/src/handlers/invoice_send.rs

  allowed_tools:
    - file_read: services/invoicing/**
    - file_write: services/invoicing/{src,tests,migrations}/**
    - bash: cd services/invoicing && cargo test hoadon

  disallowed_tools:
    - emit for non-VN tenants (per DEC-1520)
    - duplicate emission (per DEC-1522)
    - hardcoded cert path (per DEC-1521 — KMS only)

effort_hours: 6
sub_tasks:
  - "0.4h: 0007_vn_hoadon.sql"
  - "0.3h: hoadon/mod.rs"
  - "0.7h: xml_builder.rs (Decree 123 Annex 1 schema)"
  - "0.6h: signer.rs (KMS-backed XML signature)"
  - "0.8h: gdt_client.rs (GDT API client w/ retry)"
  - "0.5h: poller.rs (verification code polling)"
  - "0.3h: audit/hoadon_events.rs"
  - "0.4h: handlers/hoadon_routes.rs"
  - "0.4h: invoice_send.rs hook for auto-trigger"
  - "1.4h: tests — 8 test files"
  - "0.2h: docs"

risk_if_skipped: "Without VN hóa đơn auto-emit, VN tenants must manually emit each invoice via separate portal — non-compliance risk + accountant rejection. Without DEC-1522 idempotency, retry storms produce duplicate hóa đơn (illegal). Without DEC-1525 failure visibility, missed GDT acceptance triggers VN tax penalties."
---

## §1 — Description (BCP-14 normative)

The INV service **MUST** ship VN hóa đơn auto-emit at `services/invoicing/src/hoadon/` triggered on first AM-send for VN tenants, XML-signed per Decree 123, transmitted to GDT via async task with verification-code polling, 5 BRAIN audit kinds.

1. **MUST** hook into `services/invoicing/src/handlers/invoice_send.rs` — on first send (AC: send_count → 1), if `tenant.residency='vn-1'` AND `invoice.currency='VND'`, enqueue hóa đơn emit task per DEC-1520. Non-VN tenants: skip silently.

2. **MUST** build XML at `xml_builder.rs::build(invoice)` per Decree 123 Annex 1 schema — root `<HDon>` with `<DLHDon>` (invoice data), `<DSHHDVu>` (line items), `<TToan>` (totals), `<TTHDon>` (transaction info). UTF-8, canonical form.

3. **MUST** sign XML at `signer.rs::sign(xml, tenant_id)` — load tenant's signing cert from KMS via `tenant.signing_cert_kms_arn`; XMLDSig per W3C spec embedded in `<Signature>` element. Sign canonical form per DEC-1521. KMS errors → fail emit (don't transmit unsigned).

4. **MUST** transmit at `gdt_client.rs::submit(signed_xml, tenant)` to `tenant.gdt_environment` URL — production https://hoadondientu.gdt.gov.vn or sandbox. Receive immediate ack with `hoadon_id`. Verification code arrives async (5min-2hr per DEC-1524).

5. **MUST** poll at `poller.rs::poll(hoadon_id, tenant)` via FR-MCP-007 task — every 5min for first hour, every 15min thereafter, max 24h. On verification_code receipt: update row to `accepted`, emit `inv.hoadon_verification_received`. On 24h timeout: status=`pending` (CFO investigates).

6. **MUST** define `vn_hoadon` table at migration `0007`:
   ```sql
   CREATE TABLE vn_hoadon (
     hoadon_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     invoice_id UUID NOT NULL UNIQUE,
     hoadon_status TEXT NOT NULL DEFAULT 'pending'
       CHECK (hoadon_status IN ('pending','signed','transmitted','accepted','rejected','cancelled')),
     gdt_invoice_number TEXT,
     verification_code TEXT,
     xml_signed BYTEA,
     gdt_response JSONB,
     submitted_at TIMESTAMPTZ,
     verified_at TIMESTAMPTZ,
     failure_reason TEXT,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   CREATE UNIQUE INDEX vn_hoadon_invoice_idx ON vn_hoadon(invoice_id);
   ALTER TABLE vn_hoadon ENABLE ROW LEVEL SECURITY;
   CREATE POLICY vn_hoadon_rls ON vn_hoadon
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON vn_hoadon FROM cyberos_app;
   GRANT UPDATE (hoadon_status, gdt_invoice_number, verification_code, gdt_response,
                 submitted_at, verified_at, failure_reason, updated_at) ON vn_hoadon TO cyberos_app;
   ```

7. **MUST** be idempotent per DEC-1522 — `UNIQUE(invoice_id)` constraint + ON CONFLICT DO NOTHING; duplicate emit attempt returns existing row.

8. **MUST** emit 5 BRAIN audit kinds per DEC-1526. PII scrub per FR-BRAIN-111 — invoice amounts SHA-256 hashed, only hoadon_id + status in chain.

9. **MUST** thread trace_id from AM-send through emit + sign + transmit + poll.

10. **MUST NOT** transmit unsigned XML per DEC-1521.

11. **MUST NOT** emit for non-VN tenants per DEC-1520 — silent skip is the contract.

---

## §2 — Why this design

**Why auto-emit on send (DEC-1520)?** Decree 123 requires hóa đơn at point-of-sale; "send" is the legal trigger. Manual emit doubles workflow.

**Why idempotent (DEC-1522)?** Retry storms must not produce duplicate hóa đơn — each duplicate is a legal violation requiring cancellation paperwork.

**Why async polling (DEC-1524)?** GDT verification code latency is 5min-2hr; sync would timeout. Polling is documented GDT-recommended pattern.

**Why KMS-backed signing (DEC-1521)?** Signing cert is tenant-specific GDT-issued; must not be checked into code or env vars. KMS provides audit trail per HSM compliance.

---

## §3 — API contract

Endpoints (internal):
```text
POST   /v1/inv/hoadon/emit            (called by invoice_send handler)
GET    /v1/inv/hoadon/{invoice_id}    (status poll for CFO/UI)
POST   /v1/inv/hoadon/{id}/resubmit   (CFO-only, after rejection fix)
```

Sample XML (Decree 123 schema, abbreviated):
```xml
<HDon xmlns="http://kekhaithue.gdt.gov.vn/TKhaiHDon">
  <DLHDon>
    <TTChung>
      <PBan>2.0.0</PBan>
      <THDon>1</THDon>
      <KHMSHDon>1</KHMSHDon>
      <KHHDon>K24TAA</KHHDon>
      <SHDon>00000001</SHDon>
      <NLap>2026-05-17</NLap>
      <DVTTe>VND</DVTTe>
    </TTChung>
    <NDHDon>
      <NBan><Ten>{tenant_name}</Ten><MST>{tenant_tax_id}</MST></NBan>
      <NMua><Ten>{customer_name}</Ten></NMua>
      <DSHHDVu><HHDVu>...</HHDVu></DSHHDVu>
      <TToan><THTTLTSuat>...</THTTLTSuat><TgTCThue>1000000</TgTCThue></TToan>
    </NDHDon>
  </DLHDon>
  <DSCKS><NBan><Signature>...</Signature></NBan></DSCKS>
</HDon>
```

---

## §4 — Acceptance criteria
1. **Auto-emit on first send (VN)**. 2. **No emit on non-VN tenants**. 3. **No emit on USD/SGD/EUR invoices**. 4. **XML schema valid per Decree 123**. 5. **Signed via tenant KMS cert**. 6. **Idempotent (duplicate request = same hoadon_id)**. 7. **Status enum 6 values**. 8. **GDT transmission via FR-MCP-007 task**. 9. **Verification code polled async**. 10. **Failure → status=rejected + CFO notification**. 11. **5 BRAIN audit kinds emitted**. 12. **PII scrubbed (amounts → SHA-256)**. 13. **RLS denies cross-tenant view**. 14. **KMS errors fail emit (not transmit)**. 15. **Retry on transient GDT 5xx**. 16. **Trace_id preserved**. 17. **Resubmit endpoint CFO-only**. 18. **24h poll timeout → pending+investigation**. 19. **GDT environment switchable per-tenant**. 20. **No duplicate UNIQUE invoice constraint**. 21. **Append-only (UPDATE on status only, no row delete)**. 22. **Hoadon_id stable across resubmits**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn auto_emits_on_first_vn_send() {
    let ctx = TestContext::vn_tenant_with_signed_invoice().await;
    ctx.send_invoice(ctx.invoice_id).await;
    let row: VnHoadonRow = ctx.fetch_hoadon(ctx.invoice_id).await;
    assert_eq!(row.hoadon_status, "transmitted");
    assert!(row.xml_signed.is_some());
}

#[tokio::test]
async fn skips_non_vn_tenant() {
    let ctx = TestContext::sg_tenant_with_invoice().await;
    ctx.send_invoice(ctx.invoice_id).await;
    let row = ctx.try_fetch_hoadon(ctx.invoice_id).await;
    assert!(row.is_none());
}

#[tokio::test]
async fn idempotent_duplicate_emit() {
    let ctx = TestContext::vn_tenant_with_signed_invoice().await;
    let h1 = ctx.emit_hoadon(ctx.invoice_id).await;
    let h2 = ctx.emit_hoadon(ctx.invoice_id).await;
    assert_eq!(h1, h2);
}

// 5.4..5.10 — signing, retry, audit, RLS, resubmit, enum cardinality, PII scrub
```

---

## §6 — Skeleton

```rust
// services/invoicing/src/hoadon/mod.rs
pub async fn emit(invoice_id: Uuid, tenant: &Tenant, db: &Db) -> Result<HoadonId> {
    if tenant.residency != "vn-1" { return Err(Skip::NonVnTenant.into()); }
    let invoice = db.fetch_invoice(invoice_id).await?;
    if invoice.currency != "VND" { return Err(Skip::NonVndInvoice.into()); }
    let existing = db.try_get_hoadon(invoice_id).await?;
    if let Some(h) = existing { return Ok(h.hoadon_id); }
    let trace = current_span_trace_id();
    audit::emit("inv.hoadon_emit_started", json!({"invoice_id": invoice_id}), trace).await?;
    let xml = xml_builder::build(&invoice, tenant)?;
    let signed = signer::sign(&xml, tenant).await?;
    audit::emit("inv.hoadon_signed", json!({"invoice_id": invoice_id}), trace).await?;
    let row = db.insert_hoadon(invoice_id, &signed, tenant).await?;
    queue_transmission(row.hoadon_id, signed, tenant.clone()).await?;
    Ok(row.hoadon_id)
}
```

---

## §7 — Dependencies
**Upstream:** FR-INV-001.
**Cross-module:** FR-MCP-007 (async task), FR-AUTH-101 (KMS), FR-BRAIN-111 (PII scrub), FR-INV-008 (cancellation flow).
**Tenant config:** FR-SKILL-109 placeholder (signing_cert_kms_arn admin UI — created on first VN tenant signup).

## §8 — Sample payloads

GDT acceptance response:
```json
{
  "hoadon_id": "uuid",
  "gdt_invoice_number": "K24TAA-00000001",
  "verification_code": "ABCD1234EFGH",
  "verified_at": "2026-05-17T15:00:00Z",
  "status": "accepted"
}
```

## §9 — Open questions
None blocking — all per Decree 123 + GDT API docs.

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| KMS cert missing | KMS 404 | status=pending; sev-2 audit | CFO uploads cert |
| KMS sign error | sign() fail | status=pending; not transmitted | retry after KMS fix |
| GDT 5xx | client error | retry 3x w/ backoff | exponential 2/4/8min |
| GDT 4xx (XML invalid) | response code | status=rejected; sev-2 | CFO fixes invoice → resubmit |
| Verification timeout 24h | poller expiry | status=pending; sev-1 | CFO investigates GDT portal |
| Duplicate submit attempt | UNIQUE constraint | returns existing | inherent |
| Non-VN tenant call | early return | silent skip | inherent |
| Cert expired | KMS signal | fail emit; sev-1 | CFO renews cert |
| Network partition mid-submit | client timeout | poll on reconnect | inherent (idempotent) |
| GDT environment misconfig | wrong URL | rejected by sandbox/prod | CFO fixes tenant config |
| Verification code already used | GDT 409 | mark cancelled+reissue | CFO escalation |
| Tenant tax_id missing | XML build fail | status=pending; sev-2 | onboarding fix |
| Audit chain pause | BRAIN unavailable | retry emit | per FR-BRAIN-111 |

## §11 — Implementation notes
- §11.1 XML canonicalization per W3C Canonical XML 1.1 — required for valid XMLDSig.
- §11.2 Signing cert held in AWS KMS as asymmetric RSA-2048 key; tenant's GDT registration ties public half.
- §11.3 GDT environment URLs per tenant config — switchable test→prod without code deploy.
- §11.4 Polling intervals tuned per GDT recommended pattern (5min × 12, then 15min × 92, total 24h).
- §11.5 PII: amounts/customer info never in BRAIN — only hoadon_id, status, gdt_invoice_number (public).
- §11.6 Per Decree 123 Art. 19: hóa đơn must be transmitted within 60s of invoice issuance (async ok if reasonable effort).

---

*End of FR-INV-007 spec.*
