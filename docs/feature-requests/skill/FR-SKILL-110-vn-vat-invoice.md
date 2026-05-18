---
id: FR-SKILL-110
title: "vietnam-vat-invoice@1 skill — Vietnamese e-invoice (hóa đơn) Decree 123 XML emitter with GDT submission, digital signature, and per-invoice audit trail"
module: SKILL
priority: MUST
status: accepted
verify: T
phase: P1
milestone: P1 · slice 3
slice: 3
owner: Stephen Cheng
created: 2026-05-16
shipped: null
brain_chain_hash: null
related_frs: [FR-SKILL-103, FR-SKILL-104, FR-SKILL-105, FR-SKILL-108, FR-SKILL-109, FR-BRAIN-111, FR-AUTH-003]
depends_on: [FR-SKILL-104, FR-SKILL-108, FR-SKILL-109]
blocks: []

source_pages:
  - website/docs/skills/vietnam-vat-invoice.html
  - website/docs/legal/vn-decree-123-hoa-don.html
source_decisions:
  - DEC-230 (hóa đơn XML conforms to Decree 123/2020/NĐ-CP + Circular 78/2021/TT-BTC schemas)
  - DEC-231 (digital signature via ed25519 with seller's GDT-registered certificate)
  - DEC-232 (GDT submission via gửi-thông-báo API; receipt code is the legal proof of issue)
  - DEC-233 (PDPL: buyer info is restricted; redact in logs; persist via encrypted BRAIN row)
  - DEC-234 (invoice number monotonically increasing per tenant per template; gap detection sev-1)

language: rust 1.81
service: cyberos/skills/vietnam-vat-invoice/
new_files:
  - skills/vietnam-vat-invoice/SKILL.md
  - skills/vietnam-vat-invoice/main.rs
  - skills/vietnam-vat-invoice/src/lib.rs
  - skills/vietnam-vat-invoice/src/xml_builder.rs
  - skills/vietnam-vat-invoice/src/signer.rs
  - skills/vietnam-vat-invoice/src/gdt_submit.rs
  - skills/vietnam-vat-invoice/src/numbering.rs
  - skills/vietnam-vat-invoice/src/template.rs
  - skills/vietnam-vat-invoice/tests/xml_schema_test.rs
  - skills/vietnam-vat-invoice/tests/signer_test.rs
  - skills/vietnam-vat-invoice/tests/integration_test.rs
  - skills/vietnam-vat-invoice/schemas/HDDT_v123_2020.xsd      # Decree 123 schema
allowed_tools:
  - file_read: skills/vietnam-vat-invoice/**
  - file_write: skills/vietnam-vat-invoice/**
  - bash: cd skills/vietnam-vat-invoice && cargo test
disallowed_tools:
  - emit invoice without GDT submission (per DEC-232; receipt code is legally required)
  - reuse invoice numbers (per §1 #6 — strictly monotonic per tenant)
  - log raw buyer info (per DEC-233; redact tax_id, address, phone)

effort_hours: 11
sub_tasks:
  - "0.5h: SKILL.md frontmatter (allowed_tools=[BrainEmit, HttpFetch, BrainRead]; allowed_domains=hoadondientu.gdt.gov.vn)"
  - "0.5h: Cargo.toml + main.rs (broker subprocess entrypoint)"
  - "1.5h: xml_builder.rs — Decree 123 v1.2.0 XML structure (Header, BuyerInfo, ItemLines, TaxTotals, Summary)"
  - "0.5h: schemas/HDDT_v123_2020.xsd (vendored copy; CI gate validates output against it)"
  - "1.0h: signer.rs — ed25519 detached signature; PEM-formatted; injects into <DSIG> element"
  - "1.5h: gdt_submit.rs — POST /HoaDon/Submit; parse receipt code; handle retry + dedup on GDT timeout"
  - "1.0h: numbering.rs — sled-backed monotonic counter per (tenant_id, template_id); gap detection"
  - "0.5h: template.rs — load tenant's hóa đơn template (registered with GDT once, then bundled in cyberos)"
  - "1.0h: lib.rs — public API: `emit_invoice(req: InvoiceRequest) -> Result<InvoiceOutcome, InvoiceError>`"
  - "1.5h: xml_schema_test.rs — happy path + 5 mandatory fields missing + buyer MST invalid (via FR-SKILL-108 cross-check)"
  - "1.0h: signer_test.rs — ed25519 round-trip + tampered XML detection"
  - "1.5h: integration_test.rs — mock GDT submit endpoint; full pipeline (XML build → sign → submit → BRAIN audit)"
risk_if_skipped: "Without authoritative hóa đơn emission, VN businesses cannot legally invoice (Decree 123 mandates e-invoice for all VAT-registered entities since 2022-07-01). Without XSD validation, malformed XMLs are silently accepted at submit-time and rejected at audit-time (months later → tax penalty + retroactive). Without digital signature, GDT rejects the invoice immediately. Without monotonic numbering, gaps trigger GDT audit flag and operator must explain. Without GDT submission, the invoice has no legal force (PDF alone isn't a hóa đơn). Without per-invoice BRAIN audit, reconciliation (matching customer payment to issued invoice to GDT-recorded transaction) becomes a manual nightmare at 1000+/mo scale."
---

## §1 — Description (BCP-14 normative)

The `vietnam-vat-invoice@1` skill **MUST** emit Decree-123-compliant Vietnamese VAT invoices (hóa đơn điện tử) with digital signature and GDT submission. The contract:

1. **MUST** accept an `InvoiceRequest` with: `seller` (object: tenant_id, mst, name, address, certificate_id), `buyer` (object: mst OR personal_id, name, address, optional phone, optional email), `lines` (array of `LineItem { description, quantity, unit, unit_price_vnd, tax_rate (0|5|8|10), discount_pct (0-100) }`), `template_id` (registered with GDT), `issue_date` (ISO date), `payment_method` (`cash | bank_transfer | other`), `currency` (default `VND`; foreign currency captured separately), `idempotency_key` (UUID; same key = same invoice).
2. **MUST** validate buyer MST via FR-SKILL-108 BEFORE invoice generation. Inactive MST → `Err(BuyerMstInactive { status })`. Buyer-without-MST (private individual) → `personal_id` field used instead (CCCD via FR-BRAIN-111 ruleset).
3. **MUST** assign an invoice number monotonically per (tenant_id, template_id):
    - Read current counter from sled `~/.cyberos/skills/vietnam-vat-invoice/<tenant_id>/<template_id>.seq`.
    - Increment by 1; persist atomically (sled transaction).
    - Format: `<sym><number_padded_to_7>` (e.g. `0001234`). The `<sym>` is the template's registered series symbol from GDT.
4. **MUST** compose the XML per Decree 123 v1.2.0 schema:
    - Root element `<HDon>`.
    - `<DLHDon>` (header): seller info, invoice serial, number, date, template_id.
    - `<NDHDon>` (content): item lines with `<Hang>` elements; tax breakdown.
    - `<TToan>` (totals): subtotal, tax amount per rate, grand total (Vietnamese amount-in-words appended via `cyberos-vn-common::amount_to_words`).
    - `<DSCKS>` (signature block): `<NBan>` (seller signature) populated by signer.rs.
5. **MUST** validate the composed XML against the bundled XSD (`HDDT_v123_2020.xsd`) BEFORE signing. Schema violation → `Err(XmlSchemaViolation { detail })`; never sign invalid XML (GDT rejects).
6. **MUST** sign the XML with ed25519 using the tenant's registered certificate (loaded by `template.rs`). Signature is detached, embedded in `<DSIG>` element. Tampering with signed XML invalidates the signature.
7. **MUST** submit signed XML to GDT endpoint `https://hoadondientu.gdt.gov.vn/HoaDon/Submit` via POST (application/xml). Parse the response receipt code (`MaCQT` field in GDT response XML). Receipt code = legal proof of issue.
8. **MUST** handle GDT submission failures:
    - Network timeout → retry 3× (exp backoff 2s, 8s, 30s).
    - 4xx HTTP → permanent failure; `Err(GdtRejected { reason })`; do NOT retry; do NOT advance counter (counter already advanced before submit; emit `vn.invoice_submission_failed` audit row with counter for manual reconciliation).
    - 5xx HTTP → transient; retry as above.
    - Final failure after retries → `Err(GdtUnreachable)`.
9. **MUST** emit BRAIN audit row `vn.invoice_emitted` on successful submission with payload `{idempotency_key, invoice_serial, invoice_number, seller_mst, buyer_mst_redacted, total_vnd, tax_vnd, gdt_receipt_code, xml_hash, signed_xml_hash, submitted_at_ns, trace_id}`. The redacted buyer MST is `XX******<last_4>`.
10. **MUST** emit `vn.invoice_submission_failed` on permanent failure with payload `{idempotency_key, invoice_serial, invoice_number, gdt_error_code, gdt_error_message, attempted_at_ns, trace_id}` so the operator can manually reconcile.
11. **MUST** detect numbering gaps via `numbering::check_consecutive(tenant_id, template_id)`: lists missing numbers in the sequence; if any gap exists → emit `vn.invoice_gap_detected` sev-1 alarm via FR-OBS-007. Operator MUST file a "lost invoice" notice with GDT within 30 days.
12. **MUST** support `cyberos skill vietnam-vat-invoice replay <idempotency_key>` for crash-recovery: looks up the BRAIN audit row; if `vn.invoice_emitted` exists, returns prior outcome (idempotent). If `vn.invoice_submission_failed` exists but no emit-row → return the prior error.
13. **MUST** emit OTel span `skill.vn_vat_invoice.emit` with attrs `seller_mst`, `template_id`, `total_vnd_bucket` (log-binned), `outcome`, `gdt_round_trip_ms`, `duration_ms`.
14. **MUST** emit OTel metrics:
    - `skill_vn_vat_invoice_emits_total{outcome}` (counter; outcome ∈ ok | xml_schema | gdt_rejected | gdt_unreachable | buyer_mst_inactive).
    - `skill_vn_vat_invoice_gdt_round_trip_seconds` (histogram).
    - `skill_vn_vat_invoice_numbering_gap_total` (counter; sev-1 if >0).
15. **MUST** redact buyer info in all logs (tracing) — buyer name partial (`Nguyễn V*`), buyer MST redacted, phone/email full-masked.
16. **SHOULD** support PDF rendering alongside XML via `cyberos-vietnam-vat-invoice render-pdf <invoice_id>` (uses wkhtmltopdf + Decree 123 visual template; produces the human-readable copy).

---

## §2 — Why this design (rationale for humans)

**Why buyer MST validated upfront (§1 #2)?** Decree 123 Art. 11 requires the buyer's tax info be accurate at issue time. Issuing against an inactive MST → invoice rejected by GDT (wasted invoice number, manual reconciliation). Pre-validation catches this before submission.

**Why monotonic numbering with gap detection (§1 #3 + #11)?** GDT requires consecutive numbering per template (Circular 78 Art. 4.3). Gaps (lost invoices, system bugs) require formal "thông báo mất hoá đơn" filing within 30 days. Auto-detection means the operator sees the alarm immediately, not when the auditor calls.

**Why XSD validation BEFORE signing (§1 #5 + #6)?** GDT silently accepts well-formed XML; their backend revalidates and may reject hours later. By validating against the XSD locally, we catch malformed invoices before they consume a number. Signing valid XML only also means signature errors are unambiguous (always "tampered" not "schema gap").

**Why ed25519 (§1 #6)?** GDT accepts RSA-2048 and ECDSA; ed25519 (RFC 8032) is faster, smaller, and standardised. GDT's spec allows ed25519 since 2024. Our cert pipeline (FR-AUTH-004 reuse) is already ed25519.

**Why advance counter BEFORE submit (§1 #3 + #8)?** Atomicity: if we advance after submit, two concurrent calls could submit invoices with the same number (race). Advancing first reserves the number; failed-submit means we have an "issued but not submitted" gap that's recoverable via the replay path.

**Why detached signature embedded in `<DSIG>` (§1 #6)?** Decree 123 specifies the signature MUST be in-document (not file-side-by-side). The detached form means the signature signs the XML tree EXCLUDING the `<DSIG>` element itself — standard XML-DSig pattern; works with GDT's verification tooling.

**Why amount-in-words appended (§1 #4)?** Decree 123 Art. 10.3 requires the grand total in both numeric and Vietnamese-word form ("Một trăm năm mươi nghìn đồng"). Auditors and tax officers reference both. The conversion is non-trivial (millions, billions, fraction handling); centralised in `cyberos-vn-common::amount_to_words`.

**Why idempotency via key + replay command (§1 #12)?** Crash mid-submit could leave us in indeterminate state. Replay queries BRAIN: emit-row exists → reuse the receipt code; emit-row absent + failed-row exists → caller sees the failure and decides (manually file lost-invoice notice OR retry with same key OR generate new). Without idempotency, network blips create duplicate hóa đơn — illegal.

**Why PDF rendering separate (§1 #16)?** Decree 123 prioritises XML as the legal format; PDF is the human-readable copy. Splitting concerns: XML emitter is fast, deterministic, no external dependencies; PDF renderer (wkhtmltopdf) is heavyweight and visual-template-driven. Operators rarely need PDF programmatically; CLI suffices for slice-3.

---

## §3 — API contract

### Public API

```rust
// skills/vietnam-vat-invoice/src/lib.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvoiceRequest {
    pub seller:          SellerInfo,
    pub buyer:           BuyerInfo,
    pub lines:           Vec<LineItem>,
    pub template_id:     String,
    pub issue_date:      String,         // ISO 8601 date
    pub payment_method:  PaymentMethod,
    #[serde(default = "default_currency")]
    pub currency:        String,         // default "VND"
    pub idempotency_key: uuid::Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SellerInfo {
    pub tenant_id:      uuid::Uuid,
    pub mst:            String,
    pub name:           String,
    pub address:        String,
    pub certificate_id: String,         // references tenant's GDT-registered cert
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuyerInfo {
    pub mst:            Option<String>,  // 10/13-digit MST OR
    pub personal_id:    Option<String>,  // CCCD (12-digit)
    pub name:           String,
    pub address:        String,
    #[serde(default)] pub phone: Option<String>,
    #[serde(default)] pub email: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LineItem {
    pub description:    String,
    pub quantity:       f64,
    pub unit:           String,             // "cái", "kg", "giờ", etc.
    pub unit_price_vnd: i64,
    pub tax_rate:       TaxRate,            // 0 | 5 | 8 | 10
    #[serde(default)] pub discount_pct: f64, // 0..=100
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(into = "u8", try_from = "u8")]
pub enum TaxRate { Zero, Five, Eight, Ten }

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod { Cash, BankTransfer, Other }

#[derive(Clone, Debug, Serialize)]
pub struct InvoiceOutcome {
    pub invoice_serial:    String,      // template's series symbol
    pub invoice_number:    String,      // "0001234"
    pub gdt_receipt_code:  String,      // GDT's MaCQT — legal proof
    pub xml:               String,      // full signed XML
    pub xml_sha256:        String,
    pub total_vnd:         i64,
    pub tax_vnd:           i64,
    pub submitted_at:      i64,         // unix ms
    pub trace_id:          String,
}

#[derive(Debug, thiserror::Error)]
pub enum InvoiceError {
    #[error("buyer MST inactive (status {status})")]                BuyerMstInactive { status: String },
    #[error("buyer has neither MST nor personal_id")]              BuyerIdMissing,
    #[error("XML schema violation: {detail}")]                     XmlSchemaViolation { detail: String },
    #[error("signature error: {0}")]                                SignatureError(String),
    #[error("GDT rejected invoice: {reason}")]                     GdtRejected { reason: String },
    #[error("GDT unreachable after 3 retries")]                    GdtUnreachable,
    #[error("template {0:?} not registered for tenant")]           UnknownTemplate(String),
    #[error("numbering gap detected (will alarm sev-1)")]          NumberingGap,
    #[error("idempotency replay: returning prior outcome")]        IdempotentReplay(Box<InvoiceOutcome>),
}

pub async fn emit_invoice(req: InvoiceRequest) -> Result<InvoiceOutcome, InvoiceError> {
    // 0. Idempotency check
    if let Some(prior) = replay::lookup(&req.idempotency_key).await {
        return Err(InvoiceError::IdempotentReplay(Box::new(prior)));
    }

    // 1. Buyer MST validation (§1 #2)
    if let Some(mst) = &req.buyer.mst {
        let outcome = cyberos_vn_mst_validate::validate_mst(mst, Default::default()).await
            .map_err(map_mst_error)?;
        if !outcome.valid {
            return Err(InvoiceError::BuyerMstInactive { status: outcome.status });
        }
    } else if req.buyer.personal_id.is_none() {
        return Err(InvoiceError::BuyerIdMissing);
    }

    // 2. Reserve invoice number (§1 #3) — atomic; failure here = bail before XML build
    let (serial, number) = numbering::reserve(req.seller.tenant_id, &req.template_id).await?;

    // 3. Compose XML (§1 #4) + validate against XSD (§1 #5)
    let xml_unsigned = xml_builder::compose(&req, &serial, &number);
    xml_builder::validate_against_xsd(&xml_unsigned)?;

    // 4. Sign (§1 #6)
    let cert = template::load_certificate(req.seller.tenant_id, &req.seller.certificate_id)?;
    let xml_signed = signer::sign_ed25519(&xml_unsigned, &cert)?;

    // 5. Submit to GDT (§1 #7 + #8) with retry
    let outcome = match gdt_submit::submit_with_retry(&xml_signed, &req).await {
        Ok(receipt) => InvoiceOutcome {
            invoice_serial:   serial,
            invoice_number:   number,
            gdt_receipt_code: receipt,
            xml:              xml_signed.clone(),
            xml_sha256:       hex::encode(sha2::Sha256::digest(xml_signed.as_bytes())),
            total_vnd:        compute_total(&req.lines),
            tax_vnd:          compute_tax(&req.lines),
            submitted_at:     chrono::Utc::now().timestamp_millis(),
            trace_id:         current_trace_id(),
        },
        Err(e) => {
            emit_failure_audit(&req, &serial, &number, &e).await;
            return Err(e);
        }
    };

    // 6. Emit success audit row (§1 #9)
    emit_success_audit(&outcome, &req).await;
    Ok(outcome)
}
```

### XML composer (excerpt)

```rust
// skills/vietnam-vat-invoice/src/xml_builder.rs
pub fn compose(req: &InvoiceRequest, serial: &str, number: &str) -> String {
    let mut buf = String::with_capacity(8192);
    buf.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    buf.push_str(r#"<HDon xmlns="urn:gdt:hddt:v123">"#);

    // <DLHDon> header
    buf.push_str("<DLHDon>");
    write_elem(&mut buf, "TTChung", |b| {
        write_elem(b, "PBan", |b| b.push_str("1.2.0"));
        write_elem(b, "MTCQT", |_| {});  // placeholder until receipt arrives
        write_elem(b, "KHMSHDon", |b| b.push_str(&req.template_id));
        write_elem(b, "KHHDon", |b| b.push_str(serial));
        write_elem(b, "SHDon", |b| b.push_str(number));
        write_elem(b, "NLap", |b| b.push_str(&req.issue_date));
    });
    write_elem(&mut buf, "NBan", |b| {
        write_elem(b, "Ten", |b| b.push_str(&escape(&req.seller.name)));
        write_elem(b, "MST", |b| b.push_str(&req.seller.mst));
        write_elem(b, "DChi", |b| b.push_str(&escape(&req.seller.address)));
    });
    write_elem(&mut buf, "NMua", |b| {
        write_elem(b, "Ten", |b| b.push_str(&escape(&req.buyer.name)));
        if let Some(mst) = &req.buyer.mst       { write_elem(b, "MST", |b| b.push_str(mst)); }
        if let Some(pid) = &req.buyer.personal_id { write_elem(b, "CCCDNguoiMua", |b| b.push_str(pid)); }
        write_elem(b, "DChi", |b| b.push_str(&escape(&req.buyer.address)));
        if let Some(p) = &req.buyer.phone { write_elem(b, "SDT", |b| b.push_str(p)); }
        if let Some(e) = &req.buyer.email { write_elem(b, "DCTDTu", |b| b.push_str(e)); }
    });
    buf.push_str("</DLHDon>");

    // <NDHDon> lines
    buf.push_str("<NDHDon><DSHHDVu>");
    for (idx, line) in req.lines.iter().enumerate() {
        write_elem(&mut buf, "HHDVu", |b| {
            write_elem(b, "STT", |b| b.push_str(&(idx + 1).to_string()));
            write_elem(b, "THHDVu", |b| b.push_str(&escape(&line.description)));
            write_elem(b, "DVTinh", |b| b.push_str(&escape(&line.unit)));
            write_elem(b, "SLuong", |b| b.push_str(&format!("{:.4}", line.quantity)));
            write_elem(b, "DGia",  |b| b.push_str(&line.unit_price_vnd.to_string()));
            write_elem(b, "ThTien", |b| b.push_str(&((line.unit_price_vnd as f64 * line.quantity) as i64).to_string()));
            write_elem(b, "TSuat", |b| b.push_str(&line.tax_rate.to_xsd_value()));
        });
    }
    buf.push_str("</DSHHDVu></NDHDon>");

    // <TToan> totals (per tax rate bucket)
    let tax_breakdown = compute_tax_breakdown(&req.lines);
    let subtotal = tax_breakdown.iter().map(|b| b.amount).sum::<i64>();
    let tax_total = tax_breakdown.iter().map(|b| b.tax).sum::<i64>();
    let grand_total = subtotal + tax_total;
    buf.push_str("<TToan>");
    for b in &tax_breakdown {
        write_elem(&mut buf, "THTTLTSuat", |buf| {
            write_elem(buf, "TSuat",    |b| b.push_str(&b.rate.to_xsd_value()));
            write_elem(buf, "ThTien",   |b| b.push_str(&b.amount.to_string()));
            write_elem(buf, "TThue",    |b| b.push_str(&b.tax.to_string()));
        });
    }
    write_elem(&mut buf, "TgTCThue",  |b| b.push_str(&subtotal.to_string()));
    write_elem(&mut buf, "TgTThue",   |b| b.push_str(&tax_total.to_string()));
    write_elem(&mut buf, "TgTTTBSo",  |b| b.push_str(&grand_total.to_string()));
    write_elem(&mut buf, "TgTTTBChu", |b| b.push_str(&cyberos_vn_common::amount_to_words(grand_total)));
    buf.push_str("</TToan>");

    buf.push_str("</HDon>");
    buf
}

fn write_elem<F: FnOnce(&mut String)>(buf: &mut String, tag: &str, body: F) {
    buf.push('<'); buf.push_str(tag); buf.push('>');
    body(buf);
    buf.push_str("</"); buf.push_str(tag); buf.push('>');
}
fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&apos;")
}
```

### Signer

```rust
// skills/vietnam-vat-invoice/src/signer.rs
use ed25519_dalek::{Signer, SigningKey};

pub fn sign_ed25519(xml_unsigned: &str, cert: &TenantCert) -> Result<String, InvoiceError> {
    // 1. Canonicalise XML (C14N exclusive)
    let canon = xml_c14n::canonicalize(xml_unsigned)?;
    // 2. Sign canonical bytes
    let signing_key = SigningKey::from_bytes(&cert.private_key_bytes);
    let signature = signing_key.sign(canon.as_bytes());
    let sig_b64 = base64::encode(signature.to_bytes());

    // 3. Inject <DSIG> element before </HDon>
    let dsig_block = format!(
        "<DSIG><NBan><Signature>{sig_b64}</Signature><CertId>{}</CertId><Algo>ed25519</Algo></NBan></DSIG>",
        cert.certificate_id
    );
    let signed = xml_unsigned.replace("</HDon>", &format!("{dsig_block}</HDon>"));
    Ok(signed)
}
```

### GDT submitter

```rust
// skills/vietnam-vat-invoice/src/gdt_submit.rs
const GDT_URL: &str = "https://hoadondientu.gdt.gov.vn/HoaDon/Submit";

pub async fn submit_with_retry(xml: &str, req: &InvoiceRequest) -> Result<String, InvoiceError> {
    use std::time::Duration;
    let delays = [Duration::from_secs(2), Duration::from_secs(8), Duration::from_secs(30)];
    for (i, delay) in delays.iter().enumerate() {
        match submit_once(xml).await {
            Ok(receipt) => return Ok(receipt),
            Err(InvoiceError::GdtRejected { .. }) as e => return e,    // permanent
            Err(_) if i + 1 < delays.len() => tokio::time::sleep(*delay).await,
            Err(e) => return Err(e),
        }
    }
    Err(InvoiceError::GdtUnreachable)
}

async fn submit_once(xml: &str) -> Result<String, InvoiceError> {
    let resp = reqwest::Client::new()
        .post(GDT_URL)
        .header("Content-Type", "application/xml")
        .body(xml.to_owned())
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|_| InvoiceError::GdtUnreachable)?;
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if status.is_success() {
        // Parse MaCQT from response XML
        let receipt = extract_receipt_code(&body)
            .ok_or_else(|| InvoiceError::GdtRejected { reason: "response missing MaCQT".into() })?;
        Ok(receipt)
    } else if status.is_client_error() {
        Err(InvoiceError::GdtRejected { reason: extract_error_message(&body) })
    } else {
        Err(InvoiceError::GdtUnreachable)
    }
}
```

---

## §4 — Acceptance criteria

1. **Happy path: emit + GDT submit succeeds** — request → outcome carries `gdt_receipt_code`; BRAIN row `vn.invoice_emitted` present.
2. **Buyer MST validated upfront** — request with buyer.mst="9999999999" (invalid checksum) → FR-SKILL-108 returns ChecksumFailed → mapped to `Err(BuyerMstInactive { status: "checksum_failed" })`.
3. **Buyer inactive MST rejected** — buyer.mst valid checksum but GDT status="03" → `Err(BuyerMstInactive { status: "03" })`.
4. **Buyer without ID rejected** — both mst and personal_id None → `Err(BuyerIdMissing)`.
5. **Invoice number monotonic** — emit 3 invoices same tenant + template → numbers `0000001`, `0000002`, `0000003`.
6. **Atomic number reservation** — kill process mid-emit; restart → number NOT reused; old number reserved as "gap" → emit `vn.invoice_gap_detected`.
7. **XML validates against XSD** — happy-path XML output → `xml_schema_test::validates_against_decree_123` passes.
8. **XML schema violation rejected** — missing `<TgTTTBSo>` (synthetic test) → `Err(XmlSchemaViolation)` before signing.
9. **Signature embedded in DSIG** — signed XML contains `<DSIG><NBan><Signature>...</Signature>...</DSIG>` before `</HDon>`.
10. **Signature verification round-trip** — sign + then verify same XML → ed25519 verify passes; tamper with one byte → verify fails.
11. **Amount-in-words populated** — total 1,500,000 VND → `<TgTTTBChu>Một triệu năm trăm nghìn đồng</TgTTTBChu>`.
12. **GDT submit success returns receipt** — mock GDT returns `<KQ><MaCQT>HD2026-1234</MaCQT></KQ>` → outcome.gdt_receipt_code = "HD2026-1234".
13. **GDT 4xx is permanent failure** — mock returns 400 → `Err(GdtRejected { reason })`; no retry; `vn.invoice_submission_failed` audit row.
14. **GDT 5xx is transient (retry)** — mock returns 503 twice then 200 → retries; succeeds; total wait ~10s.
15. **GDT timeout returns Unreachable** — mock hangs → 3 retries with exp backoff; final `Err(GdtUnreachable)`.
16. **Idempotent replay** — emit with key X; emit again with same X → `Err(IdempotentReplay(prior_outcome))`; counter NOT advanced again.
17. **Numbering gap triggers alarm** — manually skip number 5 → next emit calls `numbering::check_consecutive` → `vn.invoice_gap_detected` row + sev-1 metric.
18. **Buyer info redacted in logs** — `tracing::info!(buyer = ?req.buyer)` → grep log → buyer name truncated `Nguyễn V*`; MST `0*****78`; phone redacted.
19. **BRAIN audit success row schema** — `vn.invoice_emitted` row contains `buyer_mst_redacted` (not full MST); `xml_hash` + `signed_xml_hash` differ.
20. **OTel span emitted** — span `skill.vn_vat_invoice.emit` with `total_vnd_bucket` log-binned (1k/10k/100k/1M/10M/100M/1B+).
21. **Multi-rate invoice totals correct** — lines with mixed 0%/8%/10% tax → `TToan` has 3 rate buckets; sum equals grand total.
22. **CLI replay command** — `cyberos skill vietnam-vat-invoice replay <key>` → prints prior outcome JSON; exit 0.
23. **Broker enforcement** — skill attempts `Bash` (not in allowed_tools) → broker denial; skill cannot exfiltrate.
24. **Domain enforcement** — HttpFetch to non-`hoadondientu.gdt.gov.vn` → broker denial.

---

## §5 — Verification

```rust
// skills/vietnam-vat-invoice/tests/xml_schema_test.rs

#[test]
fn happy_invoice_validates_against_xsd() {
    let req = test_request();
    let serial = "AA/26E";
    let number = "0000001";
    let xml = xml_builder::compose(&req, serial, number);
    let result = xml_builder::validate_against_xsd(&xml);
    assert!(result.is_ok(), "schema violation: {:?}", result.err());
}

#[test]
fn missing_grand_total_rejected() {
    let mut xml = xml_builder::compose(&test_request(), "AA/26E", "0000001");
    xml = xml.replace("<TgTTTBSo>", "<X>");  // break the element
    let err = xml_builder::validate_against_xsd(&xml).unwrap_err();
    assert!(matches!(err, InvoiceError::XmlSchemaViolation { .. }));
}

#[test]
fn multi_rate_totals_sum_correctly() {
    let req = test_request_with_lines(vec![
        line(100_000, TaxRate::Zero),
        line(200_000, TaxRate::Eight),
        line(300_000, TaxRate::Ten),
    ]);
    let xml = xml_builder::compose(&req, "AA/26E", "0000001");
    // Subtotal = 600_000
    assert!(xml.contains("<TgTCThue>600000</TgTCThue>"));
    // Tax = 0*100k + 0.08*200k + 0.1*300k = 0 + 16k + 30k = 46_000
    assert!(xml.contains("<TgTThue>46000</TgTThue>"));
    // Grand = 646_000
    assert!(xml.contains("<TgTTTBSo>646000</TgTTTBSo>"));
}

#[test]
fn amount_in_words_populated() {
    let req = test_request_with_total(1_500_000);
    let xml = xml_builder::compose(&req, "AA/26E", "0000001");
    assert!(xml.contains("<TgTTTBChu>Một triệu năm trăm nghìn đồng</TgTTTBChu>"));
}
```

```rust
// skills/vietnam-vat-invoice/tests/integration_test.rs

#[tokio::test]
async fn happy_path_emit_and_submit() {
    let mock_gdt = MockGdt::with_success("HD2026-0001");
    let mock_mst = MockGdtMst::with_active("0312345678", "CYBERSKILL JSC");
    let outcome = emit_invoice(test_request()).await.unwrap();
    assert_eq!(outcome.gdt_receipt_code, "HD2026-0001");
    let row = brain_test_helper::latest("vn.invoice_emitted").await;
    assert_eq!(row["payload"]["gdt_receipt_code"], "HD2026-0001");
}

#[tokio::test]
async fn idempotent_replay_returns_prior() {
    let key = uuid::Uuid::new_v4();
    let req1 = test_request_with_key(key);
    let req2 = test_request_with_key(key);
    let _ = emit_invoice(req1).await.unwrap();
    let err = emit_invoice(req2).await.unwrap_err();
    assert!(matches!(err, InvoiceError::IdempotentReplay(_)));
}

#[tokio::test]
async fn gdt_4xx_permanent_failure() {
    let _ = MockGdt::with_4xx("invalid template_id");
    let err = emit_invoice(test_request()).await.unwrap_err();
    assert!(matches!(err, InvoiceError::GdtRejected { .. }));
    let row = brain_test_helper::latest("vn.invoice_submission_failed").await;
    assert!(row["payload"]["gdt_error_message"].as_str().unwrap().contains("invalid"));
}

#[tokio::test]
async fn gdt_5xx_retries_3_times() {
    let mock = MockGdt::with_sequence([503, 503, 200]);
    let outcome = emit_invoice(test_request()).await.unwrap();
    assert_eq!(mock.call_count(), 3);
    assert!(outcome.submitted_at > 0);
}

#[tokio::test]
async fn numbering_gap_triggers_alarm() {
    let req = test_request();
    let _ = emit_invoice(req.clone()).await.unwrap();
    let _ = emit_invoice(test_request_with_key(Uuid::new_v4())).await.unwrap();
    // Force a gap by skipping number 3
    numbering::testing::force_skip(req.seller.tenant_id, &req.template_id).await;
    let _ = emit_invoice(test_request_with_key(Uuid::new_v4())).await.unwrap();
    let alarm = brain_test_helper::latest("vn.invoice_gap_detected").await;
    assert!(alarm["payload"]["missing_numbers"].as_array().unwrap().contains(&serde_json::json!("0000003")));
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton.)

---

## §7 — Dependencies

- **FR-SKILL-103** — frontmatter schema.
- **FR-SKILL-104** — broker enforces `allowed_tools` + domain allowlist on hoadondientu.gdt.gov.vn.
- **FR-SKILL-105** — brain-capture SDK used for audit rows.
- **FR-SKILL-108** — MST validation (buyer.mst checked upfront).
- **FR-SKILL-109** — VietQR for embedded payment-collection (often appended to PDF render).
- **FR-AUTH-003** — RLS on tenant cert lookup (tenant can only sign with own cert).
- **FR-BRAIN-111** — PII rules for buyer info redaction.

---

## §8 — Example payloads

### `vn.invoice_emitted` audit row

```json
{
  "kind": "vn.invoice_emitted",
  "payload": {
    "idempotency_key":      "0e3b1a2c-4f5d-6789-abcd-ef0123456789",
    "invoice_serial":       "AA/26E",
    "invoice_number":       "0000123",
    "seller_mst":           "0312345678",
    "buyer_mst_redacted":   "01******12",
    "total_vnd":            1500000,
    "tax_vnd":              150000,
    "gdt_receipt_code":     "HD2026-1234567",
    "xml_hash":             "9b0e8c5...",
    "signed_xml_hash":      "ab12cd...",
    "submitted_at_ns":      1747407137483000000,
    "trace_id":             "0af7651916cd43dd8448eb211c80319c"
  }
}
```

### `vn.invoice_submission_failed`

```json
{
  "kind": "vn.invoice_submission_failed",
  "payload": {
    "idempotency_key":     "0e3b1a2c-4f5d-6789-abcd-ef0123456789",
    "invoice_serial":      "AA/26E",
    "invoice_number":      "0000124",
    "gdt_error_code":      "E1003",
    "gdt_error_message":   "Mã mẫu hóa đơn không hợp lệ",
    "attempted_at_ns":     1747407138000000000,
    "trace_id":            "0af7651916cd43dd8448eb211c80319c"
  }
}
```

### `vn.invoice_gap_detected`

```json
{
  "kind": "vn.invoice_gap_detected",
  "payload": {
    "tenant_id":        "7e57c0de-1234-5678-9abc-def012345678",
    "template_id":      "1C25TYY",
    "invoice_serial":   "AA/26E",
    "missing_numbers":  ["0000003", "0000004"],
    "detected_at_ns":   1747407139000000000,
    "severity":         "sev-1",
    "operator_action":  "file thông báo mất hoá đơn with GDT within 30 days"
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Pre-emit dry-run mode (compose + validate + sign without submitting) — slice 4+; useful for testing.
- Bulk emit (1000 invoices in one batch) — slice 4+; GDT supports batch endpoint.
- Replace-invoice flow (Decree 123 Art. 19 — adjust/replace) — slice 4+.
- Embed VietQR in PDF render — slice 4+; uses FR-SKILL-109.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Buyer MST inactive | FR-SKILL-108 returns Inactive | `BuyerMstInactive` | Caller surfaces; user re-enters or confirms manually |
| Buyer ID missing entirely | request validation | `BuyerIdMissing` | Caller provides MST or CCCD |
| Template not registered | template::load_certificate Err | `UnknownTemplate` | Operator registers template with GDT first |
| XML composition bug | XSD validation catches | `XmlSchemaViolation` | Author fixes builder |
| Tampered XML | signature verify fails downstream | GDT rejects | Re-emit (idempotency_key change) |
| GDT 4xx (e.g. bad template) | HTTP status check | `GdtRejected`; no retry; failed audit row | Operator inspects GDT response; fixes config; retries with NEW idempotency_key |
| GDT 5xx | HTTP status check | Retry 3× | Auto-recovers; or `GdtUnreachable` |
| Network timeout | reqwest timeout | Same as 5xx | Same |
| Number gap (crash mid-submit) | gap detector | sev-1 alarm; manual "lost invoice" filing | Operator files within 30 days |
| Concurrent emits same tenant + template | sled transaction serialises | Both succeed with consecutive numbers | None |
| Idempotency key reuse with same body | replay lookup hit | `IdempotentReplay(prior)` returned | Caller uses prior result |
| Idempotency key reuse with different body | replay returns prior | Caller surprised; differences logged | Caller uses fresh key |
| Certificate expired | signer Err | `SignatureError` | Operator renews cert with GDT |
| Amount overflow | i64 covers 9.2 quintillion VND | n/a | n/a |
| Non-VND currency | currency field handling | foreign currency stored; XML uses VND-equivalent | Slice-4+ proper multi-currency |
| Tax_rate not in {0,5,8,10} | enum reject at serde | type error | Caller fixes |
| Discount > 100% | validation at line | rejected | Caller fixes |
| Line description with `<` or `&` | escape() handles | safe XML | None |
| Unicode in buyer name | utf-8 preserved through C14N | GDT accepts | None |
| sled DB corruption | numbering reservation Err | sev-1 alarm; daemon refuses | Operator restores from BRAIN audit replay |
| BRAIN unavailable | audit row write fails | Invoice still emitted; audit lost; sev-2 alarm | Operator restores BRAIN; manually reconcile from GDT |
| `cyberos-vn-common::amount_to_words` bug | unit tests catch | CI blocked | Author fixes |

---

## §11 — Implementation notes

- The XSD `HDDT_v123_2020.xsd` is vendored from GDT's official spec download; refresh on each Decree/Circular revision.
- `xml_c14n` (exclusive C14N per RFC 3076) is the canonical-form library; ed25519 signature is over the C14N form so any whitespace normalisation upstream doesn't break verify.
- The `Sha256` hash is appended to audit row for tamper detection — operators querying BRAIN can re-verify the XML matches what was submitted.
- The `numbering::reserve` API is async (sled is sync but wrapped in `spawn_blocking`); the transaction guarantees no double-issue under concurrency.
- The `IdempotentReplay` variant of InvoiceError carries the prior outcome boxed (avoids large stack frames); callers pattern-match to extract.
- `amount_to_words` lives in `cyberos-vn-common` because it's also used by FR-INV (invoice module's UI rendering).
- The CLI `cyberos skill vietnam-vat-invoice replay` is implemented as a BRAIN query (filter `vn.invoice_emitted` + `vn.invoice_submission_failed` by `idempotency_key`); no separate state store.
- The PDF renderer (§1 #16 SHOULD) is deferred; the XML emitter is the legal artifact.
- Operators who need to mass-resubmit failed invoices (after fixing the GDT config issue) use new idempotency_keys; the failed-audit rows remain as the manual-reconciliation trail.

---

*End of FR-SKILL-110.*
