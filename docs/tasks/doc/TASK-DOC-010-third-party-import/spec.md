---
id: TASK-DOC-010
title: "DOC third-party import — DocuSign / Adobe Sign / HelloSign migration with LTV (long-term-validation) preservation"
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
module: doc
priority: p1
status: draft
verify: T
phase: P2
milestone: P2 · slice 3
slice: 3
owner: Stephen Cheng (CLO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-DOC-001, TASK-DOC-007, TASK-DOC-011, TASK-AI-003, TASK-MEMORY-111]
depends_on: [TASK-DOC-001]
blocks: []

source_pages:
  - website/docs/modules/doc.html#third-party-import

source_decisions:
  - DEC-1760 2026-05-17 — Support 3 providers: docusign, adobe_sign, hellosign (Dropbox); future providers extensible
  - DEC-1761 2026-05-17 — Closed enum `import_source` = {docusign, adobe_sign, hellosign, manual_upload}; cardinality 4
  - DEC-1762 2026-05-17 — Preserve LTV: existing PAdES-B-LT or CAdES signatures retained; we add NO new signature on import
  - DEC-1763 2026-05-17 — Per-tenant API credential storage in KMS; CLO-only writes via TASK-AUTH-101
  - DEC-1764 2026-05-17 — Idempotency: source_provider + source_doc_id → one CyberOS doc; re-import returns existing document_id
  - DEC-1765 2026-05-17 — memory audit kinds: doc.import_initiated, doc.import_completed, doc.import_ltv_verified, doc.import_failed

language: rust 1.81
service: cyberos/services/doc/
new_files:
  - services/doc/migrations/0007_third_party_imports.sql
  - services/doc/src/import/mod.rs
  - services/doc/src/import/docusign_client.rs
  - services/doc/src/import/adobe_sign_client.rs
  - services/doc/src/import/hellosign_client.rs
  - services/doc/src/import/ltv_verifier.rs
  - services/doc/src/handlers/import_routes.rs
  - services/doc/src/audit/import_events.rs
  - services/doc/tests/import_docusign_test.rs
  - services/doc/tests/import_adobe_sign_test.rs
  - services/doc/tests/import_hellosign_test.rs
  - services/doc/tests/import_ltv_preserved_test.rs
  - services/doc/tests/import_idempotency_test.rs
  - services/doc/tests/import_source_enum_cardinality_test.rs
  - services/doc/tests/import_audit_emission_test.rs

modified_files:
  - services/doc/src/lib.rs

allowed_tools:
  - file_read: services/{doc,auth}/**
  - file_write: services/doc/{src,tests,migrations}/**
  - bash: cd services/doc && cargo test import

disallowed_tools:
  - add new signature on import (per DEC-1762 — preserve LTV)
  - duplicate import (per DEC-1764)

effort_hours: 10
subtasks:
  - "0.4h: 0007_third_party_imports.sql"
  - "0.4h: import/mod.rs"
  - "1.2h: docusign_client.rs"
  - "1.0h: adobe_sign_client.rs"
  - "1.0h: hellosign_client.rs"
  - "0.7h: ltv_verifier.rs"
  - "0.5h: handlers/import_routes.rs"
  - "0.3h: audit/import_events.rs"
  - "3.5h: tests — 7 test files"
  - "1.0h: docs + CLO UI for provider config + import trigger"

risk_if_skipped: "Without third-party import, customers migrating to CyberOS lose signed-doc history → adoption blocker. Without DEC-1762 LTV preservation, imported signatures lose legal validity. Without DEC-1764 idempotency, re-runs duplicate every doc."
---

## §1 — Description (BCP-14 normative)

The DOC service **MUST** ship third-party import at `services/doc/src/import/` supporting DocuSign + Adobe Sign + HelloSign, LTV preservation, idempotency, 4 memory audit kinds.

1. **MUST** validate `import_source` against closed enum per DEC-1761.

2. **MUST** dispatch per provider:
- `docusign_client.rs::list_envelopes(creds)` + `fetch_envelope(id) → PDF bytes + metadata`
- `adobe_sign_client.rs::list_agreements` + `fetch_agreement`
- `hellosign_client.rs::list_signature_requests` + `fetch_request`

3. **MUST** preserve LTV per DEC-1762 — verify existing signature is valid at `ltv_verifier.rs::verify(pdf_bytes)`; do NOT add new signature. Store as-is in S3.

4. **MUST** be idempotent per DEC-1764 via UNIQUE on (source_provider, source_doc_id, tenant_id).

5. **MUST** store provider creds in KMS per DEC-1763 — `tenant_third_party_creds.encrypted_credential_arn`; CLO-only writes.

6. **MUST** define tables at migration `0007`:
   ```sql
   CREATE TABLE tenant_third_party_creds (
     tenant_id UUID NOT NULL,
     provider TEXT NOT NULL CHECK (provider IN ('docusign','adobe_sign','hellosign')),
     encrypted_credential_arn TEXT NOT NULL,
     account_id TEXT,
     last_used_at TIMESTAMPTZ,
     set_by UUID NOT NULL,
     updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     PRIMARY KEY (tenant_id, provider)
   );
   ALTER TABLE tenant_third_party_creds ENABLE ROW LEVEL SECURITY;
   CREATE POLICY tp_creds_rls ON tenant_third_party_creds
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   GRANT UPDATE (encrypted_credential_arn, account_id, last_used_at, set_by, updated_at) ON tenant_third_party_creds TO cyberos_app;

   CREATE TABLE doc_imports (
     import_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     source_provider TEXT NOT NULL,
     source_doc_id TEXT NOT NULL,
     document_id UUID NOT NULL,
     ltv_valid BOOLEAN NOT NULL,
     imported_by UUID NOT NULL,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     UNIQUE (tenant_id, source_provider, source_doc_id)
   );
   ALTER TABLE doc_imports ENABLE ROW LEVEL SECURITY;
   CREATE POLICY imports_rls ON doc_imports
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON doc_imports FROM cyberos_app;
   ```

7. **MUST** expose endpoints:
   ```text
   PUT    /v1/doc/third-party-creds                  (CLO-only)
   POST   /v1/doc/import/{provider}/start             (CLO-only; TASK-MCP-007 task)
   GET    /v1/doc/import/jobs/{id}                   (status)
   GET    /v1/doc/imports                            (list completed)
   ```

8. **MUST** run as async TASK-MCP-007 task — provider listing + fetching may take hours for large customers.

9. **MUST** populate TASK-DOC-007 lifecycle metadata from provider metadata if available (parties, dates).

10. **MUST** emit 4 memory audit kinds per DEC-1765. PII per TASK-MEMORY-111: source_doc_id hashed; provider name (public) ok.

11. **MUST** thread trace_id from start → fetch → verify → store → audit.

12. **MUST NOT** add new signature per DEC-1762.

13. **MUST NOT** duplicate import per DEC-1764.

---

## §2 — Why this design

**Why 3 providers (DEC-1760)?** Cover 95% of enterprise e-sign market; extensible for niche.

**Why LTV preservation (DEC-1762)?** Adding new signature invalidates original; courts may not accept re-signed.

**Why CLO-gated creds (DEC-1763)?** Provider API keys grant access to all customer contracts; high privilege.

**Why async (DEC-1764)?** Large customers have thousands of docs; sync would timeout.

---

## §3 — API contract

```text
PUT    /v1/doc/third-party-creds
POST   /v1/doc/import/{provider}/start    body: {filter?: {from_date, to_date}, dry_run?: bool}
GET    /v1/doc/import/jobs/{id}
```

Sample import job status:
```json
{
  "job_id": "uuid",
  "provider": "docusign",
  "status": "running",
  "total_count": 1247,
  "imported_count": 312,
  "failed_count": 5,
  "ltv_invalid_count": 2,
  "started_at": "2026-05-17T10:00:00Z"
}
```

---

## §4 — Acceptance criteria
1. **3 providers + import_source enum cardinality 4 (incl manual)**. 2. **LTV preserved (no new signature added)**. 3. **LTV invalid → flagged but still imported (with sev-2 audit)**. 4. **Idempotent via UNIQUE constraint**. 5. **Re-import returns existing document_id (200, not 409)**. 6. **CLO-only creds (403 for others)**. 7. **CLO-only import trigger**. 8. **Async via TASK-MCP-007**. 9. **Lifecycle metadata populated from provider**. 10. **4 memory audit kinds emitted**. 11. **PII scrubbed (source_doc_id SHA256)**. 12. **RLS denies cross-tenant**. 13. **Trace_id preserved**. 14. **Dry-run lists without import**. 15. **Filter by date range respected**. 16. **Provider API creds in KMS only**. 17. **Append-only imports table via REVOKE**. 18. **Provider rate-limit respected (backoff)**. 19. **PDF MIME validated**. 20. **Large import (1000+) completes within 30min**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn docusign_imports_with_ltv_preserved() {
    let ctx = TestContext::with_docusign_creds().await;
    ctx.mock_docusign_envelope("env-123", signed_pdf_bytes).await;
    let job = ctx.import_from("docusign").await;
    ctx.wait_completion(job).await;
    let imports = ctx.list_imports().await;
    assert_eq!(imports.len(), 1);
    let doc = ctx.fetch_doc(imports[0].document_id).await;
    let stored = ctx.fetch_s3_doc(&doc.s3_key).await;
    assert_eq!(stored, signed_pdf_bytes);  // byte-identical, LTV preserved
}

#[tokio::test]
async fn idempotent_re_import() {
    let ctx = TestContext::with_completed_import().await;
    ctx.run_import_again().await;
    let imports = ctx.list_imports().await;
    let unique_doc_ids: HashSet<_> = imports.iter().map(|i| i.document_id).collect();
    assert_eq!(unique_doc_ids.len(), imports.len());  // no duplicates
}

#[tokio::test]
async fn ltv_invalid_flagged() {
    let ctx = TestContext::with_invalid_signature_pdf().await;
    let job = ctx.import_from("docusign").await;
    ctx.wait_completion(job).await;
    let imp = ctx.fetch_import(ctx.expected_import).await;
    assert_eq!(imp.ltv_valid, false);
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-DOC-001. **Cross-module:** TASK-DOC-007 (lifecycle metadata population), TASK-DOC-011 (LTV verifier shared logic), TASK-MCP-007 (async task), TASK-AUTH-101 (CLO role), TASK-AUTH-105 (KMS), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Provider API down | retry w/ backoff | sev-2; job=failed | retry job |
| Invalid creds | 401 | job=failed; CLO notified | re-enter creds |
| Provider rate limit | 429 | backoff; resume | inherent |
| LTV invalid | verifier flag | import + ltv_valid=false | manual review |
| PDF malformed | parse err | skip; sev-2 audit per doc | manual handle |
| Source doc deleted in provider | 404 | mark missing; sev-3 | inherent |
| Large PDF (>100MB) | S3 multipart | inherent | inherent |
| Duplicate source_doc_id | UNIQUE | skip; return existing | inherent |
| Cross-tenant cred leak | RLS | inherent | inherent |
| Provider deprecates v1 API | per-provider client versions | upgrade required | maintenance |

## §11 — Implementation notes
- §11.1 LTV verification uses `lopdf` + signature validation; cross-check with TASK-DOC-011 shared verifier.
- §11.2 Provider creds: refresh-token rotation managed per provider.
- §11.3 Lifecycle metadata mapping: DocuSign envelope.recipients → parties; sent_date → effective_date.
- §11.4 memory audit body: provider, ltv_valid, count; source_doc_id SHA256.
- §11.5 Bulk import: page through provider list API; per-doc fetch parallelized (5 concurrent).

---

*End of TASK-DOC-010 spec.*
