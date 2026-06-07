---
id: FR-DOC-011
title: "DOC PAdES-B-LT format + year-9 LTV re-stamping — extend B-T signatures with validation data + re-timestamp before signature/TS authority expires"
module: DOC
priority: MUST
status: ready_to_implement
verify: T
phase: P2
milestone: P2 · slice 3
slice: 3
owner: Stephen Cheng (CLO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-DOC-002, FR-DOC-003, FR-DOC-004, FR-DOC-001, FR-AI-003, FR-MEMORY-111]
depends_on: [FR-DOC-002]
blocks: []

source_pages:
  - website/docs/modules/doc.html#padesblt-ltv
  - https://www.etsi.org/deliver/etsi_en/319100_319199/31914201/  # ETSI EN 319 142-1 PAdES

source_decisions:
  - DEC-1800 2026-05-17 — Extend any B-T (timestamp-only) signature to B-LT by embedding cert chain + OCSP/CRL responses + validation data per ETSI EN 319 142-1
  - DEC-1801 2026-05-17 — Re-stamping triggered at year-9 (1 year before typical TS authority cert expires); FR-MCP-007 cron scans all B-LT sigs
  - DEC-1802 2026-05-17 — Closed enum `ltv_operation` = {extend_bt_to_blt, restamp_blt}; cardinality 2
  - DEC-1803 2026-05-17 — Closed enum `ltv_status` = {pending, completed, failed, deferred}; cardinality 4
  - DEC-1804 2026-05-17 — Re-stamping uses an active TS authority — must re-fetch fresh timestamp; original signature stays intact
  - DEC-1805 2026-05-17 — memory audit kinds: doc.ltv_extend_initiated, doc.ltv_extend_completed, doc.ltv_restamp_completed, doc.ltv_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/doc/
  new_files:
    - services/doc/migrations/0011_ltv_operations.sql
    - services/doc/src/ltv/mod.rs
    - services/doc/src/ltv/extender.rs
    - services/doc/src/ltv/restamp_cron.rs
    - services/doc/src/ltv/validation_data_fetcher.rs
    - services/doc/src/ltv/pades_writer.rs
    - services/doc/src/audit/ltv_events.rs
    - services/doc/tests/ltv_extend_bt_to_blt_test.rs
    - services/doc/tests/ltv_year_9_restamp_test.rs
    - services/doc/tests/ltv_operation_enum_cardinality_test.rs
    - services/doc/tests/ltv_status_enum_cardinality_test.rs
    - services/doc/tests/ltv_audit_emission_test.rs

  modified_files:
    - services/doc/src/lib.rs

  allowed_tools:
    - file_read: services/doc/**
    - file_write: services/doc/{src,tests,migrations}/**
    - bash: cd services/doc && cargo test ltv

  disallowed_tools:
    - alter original signature (per DEC-1804 — only embed validation data + re-stamp)
    - skip cert chain validation before restamp (per DEC-1800)

effort_hours: 8
sub_tasks:
  - "0.4h: 0011_ltv_operations.sql"
  - "0.4h: ltv/mod.rs"
  - "1.0h: extender.rs (B-T → B-LT)"
  - "0.8h: restamp_cron.rs (year-9 scan)"
  - "0.8h: validation_data_fetcher.rs (OCSP/CRL fetch)"
  - "1.0h: pades_writer.rs (embed VRI into PDF)"
  - "0.3h: audit/ltv_events.rs"
  - "2.5h: tests — 5 test files"
  - "0.8h: docs + integration smoke"

risk_if_skipped: "Without LTV extension, signatures become invalid when TS authority cert expires (~10 years) → contracts lose legal validity. Without DEC-1801 year-9 cron, sigs slip past expiry unnoticed. Without DEC-1804 fresh timestamp, re-stamp itself becomes invalid."
---

## §1 — Description (BCP-14 normative)

The DOC service **MUST** ship LTV extension at `services/doc/src/ltv/` extending B-T signatures to B-LT format and re-stamping at year-9, embedding fresh validation data, immutable audit, 4 memory audit kinds.

1. **MUST** validate `ltv_operation` against closed enum per DEC-1802.

2. **MUST** validate `ltv_status` against closed enum per DEC-1803.

3. **MUST** extend B-T → B-LT at `extender.rs::extend(signature)` per DEC-1800:
   - Fetch cert chain from signature.
   - Fetch OCSP/CRL responses for each cert at signature time.
   - Embed validation data into PDF (PAdES VRI dictionary) per ETSI EN 319 142-1.
   - Re-fetch fresh timestamp per DEC-1804.

4. **MUST** schedule year-9 re-stamping per DEC-1801 — FR-MCP-007 cron monthly, scans all `doc_documents` with signatures aged ≥9 years.

5. **MUST** never alter the original signature bytes per DEC-1804 — only embed additional validation data + add new timestamp layer.

6. **MUST** define table at migration `0011`:
   ```sql
   CREATE TABLE doc_ltv_operations (
     operation_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     document_id UUID NOT NULL,
     operation TEXT NOT NULL CHECK (operation IN ('extend_bt_to_blt','restamp_blt')),
     signature_source TEXT NOT NULL,  -- 'qtsp' | 'aatl' | 'vn_ca' | 'imported'
     original_sig_age_days INT NOT NULL,
     status TEXT NOT NULL DEFAULT 'pending'
       CHECK (status IN ('pending','completed','failed','deferred')),
     new_validation_data_added BYTEA,
     new_timestamp_token BYTEA,
     failure_reason TEXT,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     completed_at TIMESTAMPTZ
   );
   CREATE INDEX ltv_ops_doc_idx ON doc_ltv_operations(tenant_id, document_id, created_at DESC);
   ALTER TABLE doc_ltv_operations ENABLE ROW LEVEL SECURITY;
   CREATE POLICY ltv_ops_rls ON doc_ltv_operations
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON doc_ltv_operations FROM cyberos_app;
   GRANT UPDATE (status, new_validation_data_added, new_timestamp_token,
                 failure_reason, completed_at) ON doc_ltv_operations TO cyberos_app;
   ```

7. **MUST** expose endpoints:
   ```text
   POST   /v1/doc/documents/{id}/ltv/extend   (manual extend B-T → B-LT)
   POST   /v1/doc/documents/{id}/ltv/restamp  (manual re-stamp)
   GET    /v1/doc/documents/{id}/ltv/operations
   ```

8. **MUST** emit 4 memory audit kinds per DEC-1805. PII per FR-MEMORY-111: validation data + TS token hashed.

9. **MUST** thread trace_id from cron / manual → fetcher → writer → audit.

10. **MUST NOT** alter original signature per DEC-1804.

11. **MUST NOT** skip validation data fetch before re-stamp per DEC-1800.

---

## §2 — Why this design

**Why LTV (DEC-1800)?** Without embedded validation data, signature verification requires reaching OCSP/CRL/cert chain online — services may be gone in 10 years.

**Why year-9 (DEC-1801)?** Most TS authority certs valid 10 years; re-stamp at year-9 gives buffer.

**Why fresh timestamp on re-stamp (DEC-1804)?** Re-stamping with expired TS = ineffective; must use active TS authority.

**Why preserve original signature (DEC-1804)?** Tamper-evident — any modification = legal invalidation. We only add new layers.

---

## §3 — API contract

```text
POST   /v1/doc/documents/{id}/ltv/extend
POST   /v1/doc/documents/{id}/ltv/restamp
GET    /v1/doc/documents/{id}/ltv/operations
```

Sample operation:
```json
{
  "operation_id": "uuid",
  "operation": "extend_bt_to_blt",
  "signature_source": "aatl",
  "original_sig_age_days": 30,
  "status": "completed",
  "new_validation_data_size_bytes": 8192,
  "new_timestamp_authority": "TSA-DigiCert"
}
```

---

## §4 — Acceptance criteria
1. **B-T extended to B-LT correctly**. 2. **Year-9 cron scans all sigs**. 3. **OCSP/CRL responses embedded**. 4. **Cert chain embedded**. 5. **Fresh TS token added**. 6. **Original signature preserved (byte-identical)**. 7. **ltv_operation enum cardinality 2**. 8. **ltv_status enum cardinality 4**. 9. **PAdES VRI dictionary present after extend**. 10. **4 memory audit kinds emitted**. 11. **PII scrubbed (validation data + TS token SHA256)**. 12. **RLS denies cross-tenant**. 13. **Trace_id preserved**. 14. **Append-only operations table via REVOKE except status cols**. 15. **Failure → status=failed; retry**. 16. **TS authority down → status=deferred; retry next cron**. 17. **Idempotent (multiple extends OK; each adds layer)**. 18. **Verifiable in Adobe Reader after extend**. 19. **Composes with FR-DOC-002/003/004 sigs**. 20. **OCSP fetch fallback to CRL on failure**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn extend_adds_vri_to_pdf() {
    let ctx = TestContext::with_b_t_signed_pdf().await;
    let op = ctx.extend_ltv(ctx.doc_id).await;
    assert_eq!(op.status, "completed");
    let pdf = ctx.fetch_pdf(ctx.doc_id).await;
    assert!(ctx.parse_pades(pdf).has_vri_dictionary());
}

#[tokio::test]
async fn original_signature_preserved() {
    let ctx = TestContext::with_b_t_signed_pdf().await;
    let original_sig_bytes = ctx.extract_signature_bytes(ctx.doc_id).await;
    ctx.extend_ltv(ctx.doc_id).await;
    let post_extend_sig = ctx.extract_signature_bytes(ctx.doc_id).await;
    assert_eq!(original_sig_bytes, post_extend_sig);
}

#[tokio::test]
async fn year_9_cron_picks_up_aging_sigs() {
    let ctx = TestContext::with_sig_age_3300_days().await;
    ctx.run_ltv_cron().await;
    let op = ctx.fetch_latest_op(ctx.doc_id).await;
    assert_eq!(op.operation, "restamp_blt");
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-DOC-002.
**Cross-module:** FR-DOC-003 (AATL composability), FR-DOC-004 (VN CA composability), FR-DOC-001 (PDF storage), FR-MCP-007 (cron), FR-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| OCSP unreachable | retry, fallback CRL | sev-2 audit | inherent |
| TS authority down | mark deferred | retry next cron | inherent |
| Cert chain unfetchable | error | failed; sev-1 | manual intervention |
| PDF malformed | parse err | failed; sev-2 | manual review |
| Cron skipped | next run catches | inherent | inherent |
| Sig already B-LT | skip extend | no-op | inherent |
| Year-9 false positive | date math check | inherent | inherent |
| VRI dictionary write fail | rollback | failed | retry |
| Cross-tenant scan | RLS | 0 rows | inherent |
| Multiple cron instances | per-tenant queue | one at a time | inherent |

## §11 — Implementation notes
- §11.1 PAdES VRI = Validation Related Info dictionary; appended to PDF, indexed by signature SubFilter.
- §11.2 OCSP fetch via signer cert AIA extension; CRL fetch via cert CDP extension; cache 24h.
- §11.3 TS authority: rotate per partner (DigiCert, GlobalSign, FreeTSA fallback for non-tenant-bound).
- §11.4 memory audit body: doc_id, operation, signature_source, status; validation data SHA256.
- §11.5 Cron: monthly scan `SELECT doc_id WHERE last_ltv_op IS NULL OR last_ltv_op.created_at < now() - interval '9 years'`.

---

*End of FR-DOC-011 spec.*
