---
id: TASK-CRM-009
title: "CRM vietnam-bank-transfer skill — VietQR payment image generation for deal collection with embedded amount + memo + bank routing"
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
related_tasks: [TASK-CRM-001, TASK-SKILL-108, TASK-INV-005, TASK-TEN-102, TASK-MEMORY-111]
depends_on: [TASK-CRM-001]
blocks: []

source_pages:
  - website/docs/modules/crm.html#vietqr
  - https://vietqr.io/  # VietQR spec reference

source_decisions:
  - DEC-1690 2026-05-17 — Skill name: vietnam-bank-transfer@1; generates VietQR PNG with embedded payment metadata
  - DEC-1691 2026-05-17 — Closed enum `qr_purpose` = {deal_collection, invoice_payment, manual_request}; cardinality 3
  - DEC-1692 2026-05-17 — VietQR spec: bank_bin + account_number + amount + memo (alphanumeric, max 100 chars)
  - DEC-1693 2026-05-17 — Per-tenant bank config: bank_bin, account_number, account_holder_name; CFO-only writes via TASK-AUTH-101
  - "DEC-1694 2026-05-17 — Memo template: `{tenant_short}-{deal_id_8char}` — unique per QR; matches TASK-INV-005 reconciliation pattern"
  - DEC-1695 2026-05-17 — memory audit kinds: crm.vietqr_generated, crm.vietqr_config_set, crm.vietqr_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/crm/
  new_files:
    - services/crm/migrations/0009_tenant_bank_config.sql
    - services/crm/src/vn/vietqr_skill.rs
    - services/crm/src/vn/qr_generator.rs
    - services/crm/src/handlers/bank_config_routes.rs
    - services/crm/src/audit/vietqr_events.rs
    - services/crm/tests/vietqr_generation_test.rs
    - services/crm/tests/vietqr_memo_unique_test.rs
    - services/crm/tests/vietqr_purpose_enum_cardinality_test.rs
    - services/crm/tests/vietqr_bank_config_cfo_only_test.rs
    - services/crm/tests/vietqr_audit_emission_test.rs

  modified_files:
    - services/crm/src/lib.rs

  allowed_tools:
    - file_read: services/{crm,inv}/**
    - file_write: services/crm/{src,tests,migrations}/**
    - bash: cd services/crm && cargo test vietqr

  disallowed_tools:
    - generate QR without bank config (per DEC-1693)
    - allow non-CFO bank config write (per DEC-1693)

effort_hours: 4
subtasks:
  - "0.3h: 0009_tenant_bank_config.sql"
  - "0.5h: vietqr_skill.rs"
  - "0.7h: qr_generator.rs (PNG render via qrcode crate)"
  - "0.4h: handlers/bank_config_routes.rs"
  - "0.3h: audit/vietqr_events.rs"
  - "1.5h: tests — 5 test files"
  - "0.3h: docs"

risk_if_skipped: "Without VietQR generation, CFO sends bank details as text — customer mis-types account number (payment lost). Without DEC-1694 memo template, payment unreconcilable (TASK-INV-005 can't match). Without DEC-1693 CFO-only gate, anyone can change payout account (fraud risk)."
---

## §1 — Description (BCP-14 normative)

The CRM service **MUST** ship vietnam-bank-transfer@1 skill at `services/crm/src/vn/vietqr_skill.rs` generating VietQR PNG with embedded payment metadata, per-tenant bank config, 3 memory audit kinds.

1. **MUST** register skill `vietnam-bank-transfer@1` per DEC-1690.

2. **MUST** validate `qr_purpose` against closed enum per DEC-1691.

3. **MUST** require tenant bank config at table `tenant_bank_config`:
   ```sql
   CREATE TABLE tenant_bank_config (
     tenant_id UUID PRIMARY KEY,
     bank_bin TEXT NOT NULL,  -- 6-digit bank identifier per Napas
     account_number TEXT NOT NULL,
     account_holder_name TEXT NOT NULL,
     set_by UUID NOT NULL,
     updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   ALTER TABLE tenant_bank_config ENABLE ROW LEVEL SECURITY;
   CREATE POLICY bank_config_rls ON tenant_bank_config
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   GRANT UPDATE (bank_bin, account_number, account_holder_name, set_by, updated_at) ON tenant_bank_config TO cyberos_app;
   ```

4. **MUST** gate bank config writes to CFO role per DEC-1693 via TASK-AUTH-101.

5. **MUST** generate at `qr_generator.rs::generate(tenant_config, amount, memo)` per VietQR spec DEC-1692 — output PNG bytes; embedded payload `BCD\n01\n{bank_bin}\n{account_number}\n{amount}\n{memo}`.

6. **MUST** generate memo per DEC-1694 — template `{tenant_short}-{deal_id_first_8}`. Must match TASK-INV-005 reconciliation pattern.

7. **MUST** expose skill endpoint:
   ```text
   POST   /v1/crm/skill/vietnam-bank-transfer
         body: {qr_purpose, amount_vnd, deal_id?, memo_override?}
   ```

8. **MUST** validate amount_vnd > 0 and ≤ 1B VND (Napas max single transfer).

9. **MUST** emit 3 memory audit kinds per DEC-1695. PII per TASK-MEMORY-111: account_number SHA-256 hashed; amount SHA256 in chain.

10. **MUST** thread trace_id from skill call → generator → audit.

11. **MUST NOT** generate without bank config per DEC-1693 — return 412 (Precondition Failed) with link to config.

12. **MUST NOT** allow non-CFO write per DEC-1693 — 403.

---

## §2 — Why this design

**Why VietQR (DEC-1690)?** Napas-standard QR for VN domestic transfers; scannable by every VN banking app.

**Why CFO-only bank config (DEC-1693)?** Misconfigured bank routes payments to wrong account = direct money loss; CFO has authority.

**Why memo template (DEC-1694)?** TASK-INV-005 reconciles inbound VietQR payments by memo string; mismatch = orphan payment.

**Why amount cap (1B VND, DEC implicit)?** Napas spec; transfers >1B require separate higher-ceiling rail (TASK-TEN-102 covers).

---

## §3 — API contract

```text
POST   /v1/crm/skill/vietnam-bank-transfer
PUT    /v1/crm/bank-config                (CFO-only)
GET    /v1/crm/bank-config                (read by skill caller)
```

Sample skill call:
```json
{
  "qr_purpose": "deal_collection",
  "amount_vnd": 50000000,
  "deal_id": "uuid",
  "memo_override": null
}
```

Response:
```json
{
  "qr_png_base64": "iVBORw0KGgo...",
  "memo": "CSV-a1b2c3d4",
  "amount_vnd": 50000000,
  "expires_at": null
}
```

---

## §4 — Acceptance criteria
1. **Skill registered as vietnam-bank-transfer@1**. 2. **Bank config required (412 if not set)**. 3. **CFO-only bank config (403 for others)**. 4. **PNG generated with VietQR payload**. 5. **Memo template applied (tenant_short + deal_id_8)**. 6. **Memo override accepted (optional)**. 7. **qr_purpose enum 3 + cardinality test**. 8. **Amount > 0 enforced**. 9. **Amount ≤ 1B VND enforced**. 10. **3 memory audit kinds emitted**. 11. **PII scrubbed (account_number+amount SHA256)**. 12. **RLS denies cross-tenant**. 13. **Trace_id preserved**. 14. **Memo matches TASK-INV-005 regex**. 15. **PNG render deterministic for same input**. 16. **Bank config update increments updated_at**. 17. **bank_bin format 6-digit numeric**. 18. **account_number max 20 chars**. 19. **account_holder_name required**. 20. **No QR for non-VN tenant amounts (currency=VND only)**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn generates_png_with_correct_memo() {
    let ctx = TestContext::with_bank_config().await;
    let r = ctx.gen_qr("deal_collection", 50_000_000, Some(ctx.deal_id), None).await;
    assert!(r.qr_png_base64.starts_with("iVBOR"));  // PNG magic
    assert!(r.memo.starts_with("CSV-"));
    assert_eq!(r.memo.len(), 4 + 8);  // prefix + 8-char deal
}

#[tokio::test]
async fn rejects_without_bank_config() {
    let ctx = TestContext::new_tenant().await;
    let r = ctx.try_gen_qr("manual_request", 1000, None, None).await;
    assert_eq!(r.status_code, 412);
}

#[tokio::test]
async fn non_cfo_rejected_on_config_write() {
    let ctx = TestContext::with_non_cfo_user().await;
    let r = ctx.update_bank_config_as(ctx.am_user, "970422", "123456", "Cyberskill JSC").await;
    assert_eq!(r.status_code, 403);
}

// 5.4..5.8 — amount limits, memo override, enum cardinality, audit
```

---

## §7 — Dependencies
**Upstream:** TASK-CRM-001.
**Cross-module:** TASK-SKILL-108 (skill registry), TASK-AUTH-101 (CFO role), TASK-INV-005 (reconciliation memo regex), TASK-TEN-102 (>1B rail), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Bank config missing | precondition | 412 + config link | CFO configures |
| Amount > 1B VND | validate | 400 with rail-switch suggestion | use TASK-TEN-102 |
| Amount ≤ 0 | validate | 400 | provide valid |
| PNG render fail (qrcode crate err) | catch | 500 + sev-2 audit | retry |
| Deal_id doesn't exist (purpose=deal_collection) | validate | 404 | provide valid |
| Memo length >100 chars | validate | 400 (Napas limit) | shorten |
| Bank_bin invalid (not 6-digit) | validate | reject config write | provide valid |
| Non-CFO write attempt | RLS + role | 403 | request CFO |
| Cross-tenant config view | RLS | 0 rows | inherent |
| Currency != VND | future check | 400 | use other rail |

## §11 — Implementation notes
- §11.1 PNG via `qrcode` Rust crate; resolution 512x512 default; deterministic for same input.
- §11.2 Memo: `tenant_short` from `tenant.short_code` (3-char), `deal_id_8` = first 8 chars of UUID.
- §11.3 memory audit body: qr_purpose, deal_id; account_number + amount SHA256.
- §11.4 TASK-INV-005 regex: `^[A-Z]{3}-[a-z0-9]{8}$` — memo template must match.
- §11.5 Napas bank_bin reference: https://api.vietqr.io/v2/banks (look-up table).

---

*End of TASK-CRM-009 spec.*
