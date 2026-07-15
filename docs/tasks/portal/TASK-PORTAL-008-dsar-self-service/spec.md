---
id: TASK-PORTAL-008
title: "PORTAL DSAR self-service — GDPR Art. 15 + PDPL Art. 17 client-initiated data subject access request with 30-day SLA + async export bundle + redaction audit"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: PORTAL
priority: p0
status: draft
verify: T
phase: P4
milestone: P4 · slice 2
slice: 2
owner: Stephen Cheng (CLO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-PORTAL-001, TASK-PORTAL-003, TASK-PORTAL-004, TASK-TEN-104, TASK-AUTH-101, TASK-DOC-001, TASK-EMAIL-001, TASK-AI-003, TASK-MEMORY-111, TASK-OBS-007]
depends_on: [TASK-PORTAL-001]
blocks: []

source_pages:
  - website/docs/modules/portal.html#dsar
  - https://gdpr.eu/article-15-right-of-access-by-the-data-subject/
  - https://gdpr.eu/article-17-right-to-be-forgotten/
  - https://thuvienphapluat.vn/van-ban/Quyen-tu-do-cong-dan/Luat-Bao-ve-du-lieu-ca-nhan-2025/

source_decisions:
  - DEC-1220 2026-05-17 — DSAR (Data Subject Access Request) per GDPR Art. 15 + PDPL Art. 17 — client/end-user requests full export of their personal data + processing-purpose narrative; 30-day SLA per regulation
  - DEC-1221 2026-05-17 — Closed enum `dsar_type` = {access_report, portability_export, deletion_request, restriction_request, rectification_request}; CI cardinality test asserts 5
  - DEC-1222 2026-05-17 — Closed enum `dsar_status` = {received, identity_verification_pending, processing, ready_for_review, delivered, denied, expired}; CI cardinality test asserts 7
  - DEC-1223 2026-05-17 — Identity verification: SSO-authenticated callers (TASK-PORTAL-003) pass identity automatically; legacy email-password callers require step-up via email-confirmation link
  - DEC-1224 2026-05-17 — Async processing: DSAR requests queued via TASK-MCP-007 Tasks primitive (long-running) — typical 1-72 hours for full bundle assembly across PROJ/INV/DOC/CHAT/audit history
  - DEC-1225 2026-05-17 — Bundle format: ZIP archive containing per-source JSON + CSV + signed manifest; encrypted with caller-provided passphrase (deterministic re-derive on download)
  - DEC-1226 2026-05-17 — Delivery: signed URL to S3 with 7-day TTL; passphrase emailed separately via TASK-EMAIL-001; caller MUST verify integrity via manifest signature
  - DEC-1227 2026-05-17 — Deletion requests trigger TASK-TEN-104 tenant-erasure flow OR per-subject erasure via TASK-AUTH-002 hard-purge (if subject is the only requester; not destroying tenant)
  - DEC-1228 2026-05-17 — Rate limit: 1 DSAR per subject per 90 days; second request within window → 429 + `dsar_already_in_progress`
  - DEC-1229 2026-05-17 — Audit row visibility: DSAR bundle includes the caller's own audit history (filtered to events about them); does NOT include audit rows about OTHER subjects (cross-subject leak prevention)
  - DEC-1230 2026-05-17 — Denial reasons (DEC closed enum): identity_unverifiable, contradicts_law_compliance, third_party_rights_conflict, manifestly_unfounded; CFO + CLO sign-off required for denial
  - DEC-1231 2026-05-17 — memory audit kinds: portal.dsar_received, portal.dsar_identity_verified, portal.dsar_processing_started, portal.dsar_ready_for_review, portal.dsar_delivered, portal.dsar_denied, portal.dsar_expired

build_envelope:
  language: rust 1.81
  service: cyberos/services/portal/
  new_files:
    - services/portal/migrations/0016_portal_dsar_requests.sql
    - services/portal/migrations/0017_portal_dsar_denials.sql
    - services/portal/src/dsar/mod.rs
    - services/portal/src/dsar/create.rs
    - services/portal/src/dsar/processor.rs
    - services/portal/src/dsar/bundle_builder.rs
    - services/portal/src/dsar/identity_verify.rs
    - services/portal/src/dsar/delivery.rs
    - services/portal/src/dsar/denial.rs
    - services/portal/src/audit/dsar_events.rs
    - services/portal/src/handlers/dsar_routes.rs
    - services/portal/tests/dsar_create_test.rs
    - services/portal/tests/dsar_identity_verify_test.rs
    - services/portal/tests/dsar_bundle_contents_test.rs
    - services/portal/tests/dsar_cross_subject_excluded_test.rs
    - services/portal/tests/dsar_rate_limit_test.rs
    - services/portal/tests/dsar_denial_workflow_test.rs
    - services/portal/tests/dsar_30day_sla_test.rs
    - services/portal/tests/dsar_type_enum_cardinality_test.rs
    - services/portal/tests/dsar_status_enum_cardinality_test.rs
    - services/portal/tests/dsar_audit_emission_test.rs

  modified_files:
    - services/portal/src/lib.rs
    - services/portal/Cargo.toml                                       # +zip + age (encryption)

  allowed_tools:
    - file_read: services/portal/**
    - file_read: services/{proj,inv,doc,chat,auth}/src/**
    - file_write: services/portal/{src,tests,migrations}/**
    - bash: cd services/portal && cargo test dsar

  disallowed_tools:
    - include cross-subject audit rows (per DEC-1229)
    - skip identity verification on email-password caller (per DEC-1223)
    - allow > 1 DSAR per 90 days (per DEC-1228)
    - deny request without CFO+CLO sign-off (per DEC-1230)

effort_hours: 5
subtasks:
  - "0.4h: 0016_portal_dsar_requests.sql + 0017_portal_dsar_denials.sql + RLS"
  - "0.4h: dsar/mod.rs + 2 closed enums"
  - "0.4h: dsar/create.rs"
  - "0.5h: dsar/processor.rs (TASK-MCP-007 task)"
  - "0.7h: dsar/bundle_builder.rs (cross-source ZIP)"
  - "0.4h: dsar/identity_verify.rs"
  - "0.4h: dsar/delivery.rs"
  - "0.3h: dsar/denial.rs"
  - "0.3h: audit/dsar_events.rs (7 builders)"
  - "1.0h: tests — 10 test files"
  - "0.2h: wire-up"

risk_if_skipped: "Without DSAR self-service, every GDPR/PDPL data request becomes a manual operator ticket — non-scalable past 5/month; 30-day SLA = regulatory exposure if missed; missing request data fields = breach-reportable mistake. Without DEC-1229 cross-subject exclusion, one user's DSAR could leak others' data. Without DEC-1227 deletion path, right-to-be-forgotten requires operator escalation. The 5h effort lands the compliance-required self-service flow that converts CyberOS from 'has GDPR risk' to 'GDPR-ready'."
---

## §1 — Description (BCP-14 normative)

The PORTAL service **MUST** ship DSAR self-service at `services/portal/src/dsar/` for GDPR Art. 15 + PDPL Art. 17 with 5 closed-enum request types, async processing via TASK-MCP-007 Tasks, bundle delivery with passphrase encryption, identity verification, 90-day per-subject rate limit, denial workflow with CFO+CLO sign-off, and 7 memory audit kinds.

1. **MUST** define closed `dsar_type` enum: `('access_report','portability_export','deletion_request','restriction_request','rectification_request')` per DEC-1221. CI cardinality test asserts 5.

2. **MUST** define closed `dsar_status` enum: `('received','identity_verification_pending','processing','ready_for_review','delivered','denied','expired')` per DEC-1222. CI cardinality test asserts 7.

3. **MUST** define `portal_dsar_requests` table at migration `0016` with columns for request_id, tenant_id, requester_subject_id, dsar_type, status, requested_at, identity_verified_at, processing_started_at, ready_at, delivered_at, expires_at (30d post-delivery), bundle_s3_key, bundle_passphrase_kms_blob, denial_id (FK), trace_id. Append-only via REVOKE.

4. **MUST** define `portal_dsar_denials` at migration `0017` with denial reason (closed enum: `identity_unverifiable|contradicts_law_compliance|third_party_rights_conflict|manifestly_unfounded`), CFO + CLO signatures (both `subject_id` + `signed_at`), free-text justification. CHECK: both signatures required for status='denied'.

5. **MUST** enforce RLS scoped to `tenant_id AND requester_subject_id = current_setting('auth.subject_id')::uuid` (caller sees own DSARs only; tenant_admin via separate handler).

6. **MUST** expose `POST /v1/portal/dsar/request` body `{ dsar_type, scope_description?, passphrase_hash? }`. Handler:
    - Validates dsar_type in closed enum.
    - Rate-limit check per DEC-1228 (one per 90 days per subject).
    - SSO-authenticated caller → status='processing' immediately per DEC-1223.
    - Email-password caller → status='identity_verification_pending' + email confirmation link via TASK-EMAIL-001.
    - Enqueues processing via TASK-MCP-007 task per DEC-1224.
    - Emit `portal.dsar_received` sev-2.

7. **MUST** processor builds bundle per DEC-1225. Aggregates from PROJ/INV/DOC/CHAT/audit:
    - Per-source JSON dump (all rows where the caller is owner/assignee/author/mentioned).
    - Per-source CSV for human readability.
    - Audit history JSON (events ABOUT the caller; never about other subjects per DEC-1229).
    - Signed manifest with SHA-256 + counts per source.
    - ZIPs everything; encrypts with age (caller-provided passphrase).
    - Uploads to S3 with 7-day TTL signed URL.
    - Transitions status='ready_for_review'.

8. **MUST** expose `GET /v1/portal/dsar/{id}` for status polling. Returns current state. On `status='delivered'`: includes signed_url + manifest_sha256.

9. **MUST** support denial workflow per DEC-1230. `POST /v1/admin/dsar/{id}/deny` requires `cfo` + `clo` roles (dual-signature). Body `{ reason, justification }`. Both signatures populate denial row.

10. **MUST** support deletion via TASK-TEN-104 or TASK-AUTH-002 hard-purge per DEC-1227. `dsar_type='deletion_request'` triggers internal review queue (sev-1) before destructive action; operator manually invokes hard-purge after legal sign-off.

11. **MUST** rate-limit per DEC-1228 — second DSAR within 90 days returns 429.

12. **MUST** emit 7 memory audit kinds per DEC-1231: received, identity_verified, processing_started, ready_for_review, delivered, denied, expired — all sev-1 (regulatory-critical).

13. **MUST** PII-scrub per task-audit skill rule 18 — requester_subject_id UUID retained; email hashed.

14. **MUST** thread trace_id end-to-end.

15. **MUST** expire bundle at T+7 days post-delivery (signed URL + S3 object).

16. **MUST NOT** include audit rows about OTHER subjects per DEC-1229.

17. **MUST NOT** include audit rows that would leak third-party PII (e.g., a comment by another user about the requester — show the comment but redact the other user's identifying details).

---

## §2 — Why this design (rationale for humans)

**Why async via Tasks primitive (§1 #6, DEC-1224)?** Bundle assembly = cross-table scan + cross-tenant filter + ZIP encryption — easily minutes to hours for active subjects. Sync processing = 30s gateway timeout; async via TASK-MCP-007 fits exactly. Reuses existing infrastructure.

**Why passphrase encryption (§1 #7, DEC-1225)?** S3 signed URL alone = anyone with URL access reads bundle. Passphrase = end-to-end encryption to the caller; even if URL leaks, encrypted bundle unreadable.

**Why dual-sign denial (§1 #9, DEC-1230)?** Denying a DSAR is legally significant — GDPR Art. 12 requires "without undue delay and at the latest within one month". Wrongful denial = fine. CFO + CLO dual-sign forces review.

**Why 90-day rate limit (§1 #11, DEC-1228)?** GDPR Art. 12(5) allows charging fees for "manifestly unfounded or excessive" requests including those repeated. 90 days = quarterly cadence reasonable; second request within 90d = excessive presumption.

**Why exclude cross-subject audit (§1 #16, DEC-1229)?** A requester's DSAR including audit rows about OTHER subjects = data leak. Filter to events WHERE actor=requester OR target=requester (direct involvement only).

---

## §3 — API contract

```sql
-- 0016_portal_dsar_requests.sql
CREATE TYPE dsar_type AS ENUM ('access_report','portability_export','deletion_request','restriction_request','rectification_request');
CREATE TYPE dsar_status AS ENUM ('received','identity_verification_pending','processing','ready_for_review','delivered','denied','expired');

CREATE TABLE portal_dsar_requests (
  request_id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  requester_subject_id UUID NOT NULL,
  dsar_type dsar_type NOT NULL,
  status dsar_status NOT NULL DEFAULT 'received',
  scope_description TEXT,
  passphrase_hash CHAR(64),
  requested_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  identity_verified_at TIMESTAMPTZ,
  processing_started_at TIMESTAMPTZ,
  ready_at TIMESTAMPTZ,
  delivered_at TIMESTAMPTZ,
  expires_at TIMESTAMPTZ,
  bundle_s3_key TEXT,
  bundle_passphrase_kms_blob BYTEA,
  denial_id BIGINT,
  trace_id CHAR(32)
);
-- Partial unique on active DSAR; 90d window enforced by daily prune job per §11.10
CREATE UNIQUE INDEX uniq_dsar_active_per_subject
  ON portal_dsar_requests(requester_subject_id)
  WHERE status NOT IN ('delivered','denied','expired');
ALTER TABLE portal_dsar_requests ENABLE ROW LEVEL SECURITY;
CREATE POLICY portal_dsar_requests_rls ON portal_dsar_requests
  USING (tenant_id = current_setting('auth.tenant_id')::uuid
         AND requester_subject_id = current_setting('auth.subject_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid
              AND requester_subject_id = current_setting('auth.subject_id')::uuid);
REVOKE UPDATE, DELETE ON portal_dsar_requests FROM cyberos_app;
GRANT UPDATE (status, identity_verified_at, processing_started_at, ready_at, delivered_at,
              expires_at, bundle_s3_key, bundle_passphrase_kms_blob, denial_id)
  ON portal_dsar_requests TO cyberos_app;

-- 0017_portal_dsar_denials.sql
CREATE TYPE dsar_denial_reason AS ENUM ('identity_unverifiable','contradicts_law_compliance','third_party_rights_conflict','manifestly_unfounded');
CREATE TABLE portal_dsar_denials (
  id BIGSERIAL PRIMARY KEY,
  tenant_id UUID NOT NULL,
  request_id UUID NOT NULL REFERENCES portal_dsar_requests(request_id),
  reason dsar_denial_reason NOT NULL,
  justification TEXT NOT NULL,
  cfo_subject_id UUID NOT NULL,
  cfo_signed_at TIMESTAMPTZ NOT NULL,
  clo_subject_id UUID NOT NULL,
  clo_signed_at TIMESTAMPTZ NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  CHECK (cfo_subject_id != clo_subject_id)  -- two distinct people
);
ALTER TABLE portal_dsar_denials ENABLE ROW LEVEL SECURITY;
CREATE POLICY portal_dsar_denials_rls ON portal_dsar_denials
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON portal_dsar_denials FROM cyberos_app;
```

Endpoints:
```text
POST   /v1/portal/dsar/request                       (caller)
GET    /v1/portal/dsar/{id}                           (caller polling)
GET    /v1/admin/tenants/{tid}/dsar                   (tenant_admin list)
POST   /v1/admin/dsar/{id}/deny                       (cfo + clo dual-sign)
```

---

## §4 — Acceptance criteria

1. **dsar_type cardinality** — 5 values exactly.
2. **dsar_status cardinality** — 7 values exactly.
3. **SSO caller skips identity verify** — IdP-auth caller goes straight to status='processing'.
4. **Email-password caller requires verify** — non-SSO caller gets identity_verification_pending + email link.
5. **Bundle contents** — bundle includes caller's own PROJ/INV/DOC/CHAT rows + audit history.
6. **Cross-subject audit excluded** — bundle audit history excludes events about other subjects.
7. **90-day rate limit** — second DSAR within window → 429.
8. **Denial dual-signature** — denial without both CFO + CLO signatures → 400.
9. **Deletion request triggers review** — `deletion_request` type creates sev-1 internal review queue.
10. **30-day SLA tracked** — daily job flags DSARs > 25d in non-terminal status (5-day warning); > 30d alerts sev-1.
11. **Bundle expires at 7d** — signed URL + S3 object removed.
12. **Passphrase encryption** — bundle ZIP encrypted with age + caller passphrase.
13. **Manifest signature** — included; verifiable.
14. **7 memory audit kinds emitted** — all 7 lifecycle events emit.
15. **PII scrubbed in audit** — email_hash16 only in chain.
16. **Cross-tenant denied** — caller from tenant X can't see tenant Y DSARs.
17. **Trace_id threaded end-to-end**.
18. **Tenant_admin list** — separate endpoint, tenant_admin role required.
19. **Partial-unique index 90d** — second active DSAR rejected at INSERT.
20. **Distinct CFO/CLO subjects** — CHECK constraint prevents same person dual-signing.

---

## §5 — Verification

```rust
// 5.1 dsar_create_test
#[tokio::test]
async fn sso_caller_skips_identity_verify() {
    let ctx = TestContext::with_sso_subject().await;
    let r = ctx.post_dsar("access_report", None).await;
    assert_eq!(r.status(), 201);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["status"], "processing");
}

// 5.2 cross-subject excluded
#[tokio::test]
async fn bundle_excludes_other_subjects_audit() {
    let ctx = TestContext::new().await;
    ctx.seed_audit_for_subject(ctx.subject_a, "event-a").await;
    ctx.seed_audit_for_subject(ctx.subject_b, "event-b").await;
    let dsar_id = ctx.as_subject_a().post_dsar("access_report", Some("pass")).await;
    ctx.await_dsar_ready(dsar_id).await;
    let bundle = ctx.download_dsar_bundle(dsar_id, "pass").await;
    let audit = bundle.read_json("audit/history.json").await;
    assert!(audit.iter().any(|e| e["kind"] == "event-a"));
    assert!(!audit.iter().any(|e| e["kind"] == "event-b"));
}

// 5.3 rate limit 90d
#[tokio::test]
async fn second_dsar_within_90d_blocked() {
    let ctx = TestContext::new().await;
    ctx.post_dsar("access_report", None).await;
    let r = ctx.post_dsar("access_report", None).await;
    assert_eq!(r.status(), 429);
}

// 5.4 denial dual-sign
#[tokio::test]
async fn denial_requires_cfo_and_clo() {
    let ctx = TestContext::new().await;
    let dsar = ctx.post_dsar("access_report", None).await;
    let r = ctx.as_cfo().post_deny(dsar, "identity_unverifiable", "no docs").await;
    assert_eq!(r.status(), 400);  // only one signature
    let r2 = ctx.as_cfo_and_clo().post_deny(dsar, "identity_unverifiable", "no docs").await;
    assert_eq!(r2.status(), 200);
}

// 5.5 type enum cardinality
#[tokio::test]
async fn dsar_type_has_5_values() {
    let labels: Vec<String> = sqlx::query_scalar("SELECT unnest(enum_range(NULL::dsar_type))::text").fetch_all(&ctx.pool).await.unwrap();
    assert_eq!(labels.len(), 5);
}

// 5.6 status enum cardinality
// 5.7 30d SLA flag
// 5.8 bundle passphrase encryption
// 5.9 cross-tenant deny
// 5.10 audit emission
```

---

## §7 — Dependencies

**Upstream:** TASK-PORTAL-001 (data sources to aggregate).
**Cross-module:** TASK-PORTAL-003 (SSO identity), TASK-PORTAL-004 (subject lifecycle), TASK-MCP-007 (Tasks for async bundle), TASK-EMAIL-001 (verification + delivery emails), TASK-DOC-001 (S3 bundle storage), TASK-AUTH-101 (cfo + clo roles), TASK-AI-003 (audit kinds), TASK-MEMORY-111 (PII scrub), TASK-OBS-007 (sev-1 SLA alarms).
**Downstream:** None.

---

## §8 — Example payload

`portal.dsar_delivered`:
```json
{
  "kind": "portal.dsar_delivered",
  "severity": 1,
  "tenant_id": "8a2f...",
  "trace_id": "...",
  "occurred_at": "2026-06-15T...",
  "payload": {
    "request_id": "0190...",
    "requester_subject_id_hash16": "f8a1...",
    "dsar_type": "access_report",
    "bundle_size_bytes": 4823104,
    "expires_at": "2026-06-22T..."
  }
}
```

---

## §9 — Open questions

Deferred:
- **Deferred:** Multilingual narrative ("what we collected") — slice 3.
- **Deferred:** Granular consent withdrawal per data-category — slice 3.
- **Deferred:** Automated identity verification via government-ID OCR — slice 3.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Identity verification email fails | TASK-EMAIL-001 error | Status stays pending; daily reminder + 30d expiry | Caller requests resend |
| Bundle assembly task fails | TASK-MCP-007 task failed | Sev-1 alert; status='received' again; operator investigates | Manual retry |
| 30-day SLA missed | daily SLA check | Sev-1 alert + clo notification | Operator escalates |
| Bundle download URL leaked | inherent risk; passphrase mitigates | Encrypted bundle unreadable without passphrase | Inherent E2EE |
| Deletion request without legal sign-off | sev-1 review queue | Operator+CLO review before hard-purge | Manual review process |
| Cross-tenant DSAR submission | RLS rejects | 403 | Caller's own tenant only |
| Same person CFO+CLO dual-sign | CHECK constraint | INSERT fails | Different signatories required |
| Bundle expired before download | scheduled cleanup at T+7d | 404 on signed URL; caller requests new DSAR | DSAR re-submission |
| Subject hard-purged mid-DSAR-processing | task failure on missing data | Sev-1; partial bundle delivered with note | Inherent edge case |
| Rate-limit hit at 90d boundary | counter | 429 | Caller waits |
| Passphrase forgotten | inherent | Bundle unrecoverable; new DSAR required | E2EE design tradeoff |
| Manifest signature verification fails | client-side check | Caller notified; sev-2 alert | Re-deliver |

---

## §11 — Implementation notes

**§11.1** Bundle ZIP uses `zip` crate; encryption via `age` crate; passphrase = scrypt-derived from caller input.

**§11.2** Manifest signed with per-tenant Ed25519 key from KMS; client verifies with tenant's public key (published).

**§11.3** Task processor uses TASK-MCP-007 worker pool; per-module=`portal`; max_concurrent=2 (heavy I/O).

**§11.4** Email delivery via TASK-EMAIL-001 with priority=high; identity-verify email has 24h link expiry.

**§11.5** Audit cross-subject filter SQL: `WHERE actor_subject_id = $1 OR resource_subject_id = $1` (only direct involvement).

**§11.6** 30-day SLA tracked via daily job comparing requested_at + 25/30 days; sev-2 at warning, sev-1 at breach.

**§11.7** S3 lifecycle policy auto-deletes bundle at T+7d; double-protection vs application-side cleanup.

**§11.8** Deletion-request review queue posts a Slack/CHAT notification to legal team (out-of-band).

**§11.9** Per-tenant bundle key prefix `dsar/{tenant_id}/{request_id}/bundle.zip.age` for forensic clarity.

**§11.10** Rate-limit partial-unique index requires periodic prune (90d window enforcement via daily job).

---

*End of TASK-PORTAL-008 spec.*
