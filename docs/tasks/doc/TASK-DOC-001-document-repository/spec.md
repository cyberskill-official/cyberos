---
id: TASK-DOC-001
title: "DOC Document repository — S3 Object-Lock Compliance bucket + per-tenant residency pinning + versioned + ACL'd + 10-year retention + hash-chained audit"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-16T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: DOC
priority: p0
status: draft
verify: T
phase: P4
milestone: P4 · slice 1
slice: 1
owner: Stephen Cheng (CLO)
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_tasks: [TASK-AUTH-003, TASK-AUTH-101, TASK-AI-003, TASK-AI-016, TASK-MEMORY-101, TASK-DOC-002, TASK-DOC-003, TASK-DOC-004, TASK-DOC-005, TASK-DOC-007, TASK-DOC-010, TASK-DOC-011]
depends_on: [TASK-AUTH-101]
# all 6 entries are placeholders — not yet specified (downstream consumers)
blocks: [TASK-DOC-002, TASK-DOC-003, TASK-DOC-004, TASK-DOC-005, TASK-DOC-007, TASK-DOC-010]

source_pages:
  - website/docs/modules/doc.html#what
  - website/docs/modules/doc.html#document-repository
  - website/docs/modules/doc.html#archive
source_decisions:
  - DEC-280 (S3 Object-Lock Compliance mode — write-once, retention 10 years minimum, cannot be shortened even by root account; eIDAS Art. 32 + US ESIGN compliance)
  - DEC-281 (per-tenant residency pinning — VN tenants store in `vn-1` region (e.g. ap-southeast-1 + VN-residency-mandate label); EU in `eu-1`; SG/US in respective regions per TASK-AI-016)
  - DEC-282 (closed bucket-scope enum: hr-contracts, crm-contracts, esop-grants, kyc-docs, vendor-contracts, policies, generic — adding a scope is an ADR)
  - DEC-283 (hash-chained audit log per document — every operation (upload, ACL change, sign, archive) appends a row with prev_hash → SHA-256 chain replayable end-to-end)
  - DEC-284 (immutable versions: every save creates a new S3 object with v<N> suffix; old versions retained for the full retention period)
  - DEC-285 (Object-Lock LegalHold can be applied for litigation hold per Federal Rule of Civil Procedure 26(b); requires CLO + CSO co-sign — never auto)
  - DEC-286 (REVOKE UPDATE, DELETE on document_metadata + document_versions from cyberos_app — append-only enforced at SQL grant + S3 Object-Lock)
  - DEC-287 (retention period configurable per bucket-scope but ≥ 10 years; HR contracts default 50 years per VN labour law; ESOP grants 75 years per US ERISA-equivalent)
  - DEC-288 (signed-document state machine: draft → in_signing → fully_signed → archived; only fully_signed → archived applies Object-Lock retention; drafts can be deleted)
  - DEC-289 (memory audit kinds: doc.uploaded, doc.versioned, doc.acl_changed, doc.signed, doc.archived, doc.legal_hold_applied, doc.legal_hold_released, doc.access_audited)
  - DEC-290 (access to documents emits doc.access_audited at sev-2 — every read is a forensically-relevant event)
  - DEC-291 (S3 server-side encryption with KMS keys — separate KMS keyspace per bucket-scope; HR contracts use the same keyspace as TASK-HR-003's CCCD photos, distinct from generic)
  - DEC-292 (cross-bucket-scope moves forbidden — a doc uploaded to hr-contracts cannot be moved to generic; the keyspace + retention policy is locked at upload)
  - eIDAS Regulation 910/2014 Art. 32 (long-term electronic-signature preservation — 10-year minimum)
  - US ESIGN Act 2000 §101(d) (electronic record retention; 5-year minimum, jurisdictional override → 10 years here)
  - VN Decree 130/2018 (digital signature retention — 10 years)
  - VN Labour Code Art. 161 (employment contract retention 50 years after termination)
  - GDPR Art. 17 (right to erasure — `legal_hold` blocks erasure; PDPL Art. 17 equivalent)
  - ISO/IEC 27001:2022 A.5.13 + A.5.14 (information classification + media handling)

language: rust 1.81 + sql
service: cyberos/services/doc/
new_files:
  # document_metadata + document_versions + BucketScope ENUM + RLS + REVOKE writes + Object-Lock-aware constraints
  - services/doc/migrations/0001_document_metadata.sql
  # hash-chained per-document audit log + chain integrity trigger
  - services/doc/migrations/0002_document_audit_log.sql
  # crate root
  - services/doc/src/lib.rs
  # DocumentMetadata, DocumentVersion, BucketScope (7), DocumentStatus (4), RetentionPolicy
  - services/doc/src/types.rs
  # S3 Object-Lock Compliance configuration: bucket setup + per-object retention
  - services/doc/src/s3/object_lock.rs
  # presigned upload + integrity hash + residency-pinned bucket selection
  - services/doc/src/s3/upload.rs
  # signed-URL fetch with audit-emission
  - services/doc/src/s3/download.rs
  # per-tenant residency lookup → S3 bucket + KMS key (joins TASK-AI-016)
  - services/doc/src/residency.rs
  # metadata CRUD; uses S3 client for storage path
  - services/doc/src/repo/documents.rs
  # append-only version writer
  - services/doc/src/repo/versions.rs
  # hash-chained per-document log writer
  - services/doc/src/repo/audit_log.rs
  # canonical doc.* memory row builders (8 kinds per DEC-289)
  - services/doc/src/audit/doc_events.rs
  # apply + release with CLO+CSO co-sign gate
  - services/doc/src/legal_hold.rs
  # POST/GET/PATCH /v1/doc/documents + POST /upload + POST /sign-stub
  - services/doc/src/handlers/documents.rs
  # +sqlx, +uuid, +serde, +chrono, +aws-sdk-s3, +sha2, +cyberos-cli-exit
  - services/doc/Cargo.toml
  # happy + invalid + RLS + idempotent
  - services/doc/tests/documents_crud_test.rs
  # SQL enum + Rust enum cross-validation; ADR gate stub
  - services/doc/tests/bucket_scope_enum_test.rs
  # VN tenant → vn-1 bucket; EU → eu-1; assert no cross-region leakage
  - services/doc/tests/residency_pin_test.rs
  # Object-Lock retention applied on archive transition; LegalHold path tested with mock S3
  - services/doc/tests/object_lock_compliance_test.rs
  # UPDATE/DELETE rejected by SQL grant
  - services/doc/tests/version_append_only_test.rs
  # chain integrity: prev_hash → SHA-256 chains; tampered row detected
  - services/doc/tests/audit_log_chain_test.rs
  # hr-contracts default 50y; esop 75y; generic 10y
  - services/doc/tests/retention_period_test.rs
  # hr-contracts → generic move rejected with `cross_scope_move_forbidden`
  - services/doc/tests/cross_scope_move_rejected_test.rs
  # single signer rejected; CLO+CSO co-sign accepted
  - services/doc/tests/legal_hold_dual_signoff_test.rs
  # GET /download emits doc.access_audited sev-2
  - services/doc/tests/access_audit_emission_test.rs
  # GDPR erasure blocked while legal_hold active
  - services/doc/tests/erasure_request_with_legal_hold_test.rs
modified_files:
  # +Resource::DocDocument (already in TASK-AUTH-101 catalog at slice 1)
  - services/auth/src/rbac/permissions.rs

allowed_tools:
  - file_read: services/doc/**
  - file_read: services/auth/src/rbac/**
  - file_write: services/doc/{src,tests,migrations}/**
  - bash: cd services/doc && cargo test
  - bash: psql -f services/doc/migrations/0001_document_metadata.sql (local Postgres only)
  - bash: docker run --rm minio/minio (S3-compatible local for tests)

disallowed_tools:
  - allow DELETE on document_metadata or document_versions from cyberos_app (per DEC-286)
  - skip S3 Object-Lock setup on archive transition (per DEC-280)
  - allow cross-bucket-scope moves (per DEC-292)
  - apply LegalHold with single signer (per DEC-285 — CLO + CSO co-sign required)
  - bypass residency pinning to save cost (per DEC-281)
  - implement actual eIDAS QTSP / AATL / VNeID signing here (those are TASK-DOC-002/003/004 respectively)

effort_hours: 8
subtasks:
  - "1.0h: 0001_document_metadata.sql — metadata + versions tables + 4 enums + RLS + REVOKE + Object-Lock-aware archive flag"
  - "0.7h: 0002_document_audit_log.sql — hash-chained per-document log + chain integrity trigger"
  - "0.5h: types.rs — DocumentMetadata + DocumentVersion + BucketScope (7) + DocumentStatus (4) + RetentionPolicy"
  - "0.8h: s3/object_lock.rs — Object-Lock Compliance configuration: bucket setup + per-object retention period"
  - "0.5h: s3/upload.rs — presigned upload + integrity hash + residency-pinned bucket selection"
  - "0.4h: s3/download.rs — signed-URL fetch with audit emission"
  - "0.5h: residency.rs — per-tenant residency lookup (consumes TASK-AI-016 policy)"
  - "0.5h: repo/documents.rs — metadata CRUD"
  - "0.3h: repo/versions.rs — append-only writer"
  - "0.5h: repo/audit_log.rs — hash-chained log writer + chain replayer"
  - "0.4h: audit/doc_events.rs — 8 row builders"
  - "0.4h: legal_hold.rs — dual-signoff workflow"
  - "0.5h: handlers/documents.rs — REST surface"
  - "1.5h: tests — 11 test files covering bucket-scope closure, residency pinning, Object-Lock, audit chain integrity, legal hold dual-signoff, access audit emission, GDPR erasure with hold, cross-scope move rejection"

risk_if_skipped: "Every downstream DOC task (TASK-DOC-002 eIDAS QTSP, TASK-DOC-003 AATL CA, TASK-DOC-004 VNeID + VN CA, TASK-DOC-005 multi-party signing, TASK-DOC-007 lifecycle metadata, TASK-DOC-010 DocuSign import) needs the repository to store + retrieve documents. Without DEC-280's Object-Lock Compliance mode, the 'retention is non-negotiable' guarantee is paper — a malicious root admin can DELETE. Without DEC-281's residency pinning, VN tenants' contracts cross border (Decree 53/2022 violation); EU contracts violate GDPR if stored in non-EU regions. Without DEC-283's hash-chained audit, signature-tamper detection is impossible. Without DEC-285's dual-signoff for LegalHold, litigation hold becomes a single-operator action (subpoena response could leak via individual misuse). The 8h effort lands the legally-defensible storage primitives on which signing law depends."
---

## §1 — Description (BCP-14 normative)

The DOC service **MUST** ship the document repository as the foundational storage primitive — S3 Object-Lock Compliance bucket + per-tenant residency pinning + versioned + ACL'd + hash-chained audit log. Each requirement:

1. **MUST** define the `document_metadata` table with: `id UUID PRIMARY KEY`, `tenant_id UUID NOT NULL`, `bucket_scope bucket_scope NOT NULL`, `original_filename TEXT NOT NULL`, `mime_type TEXT NOT NULL`, `byte_size BIGINT NOT NULL CHECK (byte_size BETWEEN 1 AND 524288000)` (500 MB max), `sha256_hex CHAR(64) NOT NULL`, `status document_status NOT NULL DEFAULT 'draft'`, `s3_region TEXT NOT NULL`, `s3_bucket TEXT NOT NULL`, `s3_key TEXT NOT NULL`, `kms_key_id TEXT NOT NULL`, `current_version_id UUID NOT NULL REFERENCES document_versions(id) DEFERRABLE INITIALLY IMMEDIATE`, `legal_hold BOOLEAN NOT NULL DEFAULT false`, `retention_until DATE` (nullable until status=archived; immutable after), `parent_object_id UUID REFERENCES <module>(...)` (logical FK soft-typed by bucket_scope), `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`, `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()`, `created_by_subject_id UUID NOT NULL REFERENCES auth.subjects(id)`.

2. **MUST** define `document_versions` with: `id UUID PRIMARY KEY`, `document_id UUID NOT NULL REFERENCES document_metadata(id) ON DELETE RESTRICT`, `tenant_id UUID NOT NULL`, `version_number INT NOT NULL`, `s3_key_versioned TEXT NOT NULL` (full S3 path with `?versionId=<aws-version-id>`), `sha256_hex CHAR(64) NOT NULL`, `byte_size BIGINT NOT NULL`, `mime_type TEXT NOT NULL`, `uploaded_at TIMESTAMPTZ NOT NULL DEFAULT now()`, `uploaded_by_subject_id UUID NOT NULL REFERENCES auth.subjects(id)`.

3. **MUST** declare the closed `bucket_scope` Postgres enum with exactly 7 values (per DEC-282): `'hr_contracts'`, `'crm_contracts'`, `'esop_grants'`, `'kyc_docs'`, `'vendor_contracts'`, `'policies'`, `'generic'`. Adding an 8th is an ADR. Each scope maps to a distinct S3 bucket + KMS key (per DEC-291).

4. **MUST** declare the closed `document_status` Postgres enum with exactly 4 values (per DEC-288): `'draft'`, `'in_signing'`, `'fully_signed'`, `'archived'`. Transition `fully_signed → archived` applies S3 Object-Lock retention (per §1 #7); other transitions leave the object mutable.

5. **MUST** enforce RLS with both `USING` and `WITH CHECK` clauses on `document_metadata` AND `document_versions`. Policy: `tenant_id = current_setting('auth.tenant_id')::uuid`. Cross-tenant reads return 0 rows; cross-tenant writes fail `permission_denied`.

6. **MUST** be **append-only** on `document_versions` at the SQL-grant layer (per DEC-286 + task-audit skill rule 12). `REVOKE UPDATE, DELETE ON document_versions FROM cyberos_app;`. Every update creates a new version row + new S3 object.

7. **MUST** apply S3 Object-Lock Compliance retention on transition to `status='archived'` (per DEC-280). The retention period is computed from the bucket_scope's policy (per DEC-287):
- `hr_contracts` → 50 years (VN Labour Code Art. 161).
- `esop_grants` → 75 years (long-term employment-equivalent record).
- `kyc_docs` → 10 years (PDPL retention floor).
- `generic`, `crm_contracts`, `vendor_contracts`, `policies` → 10 years (eIDAS + US ESIGN floor). The retention is applied via `aws-sdk-s3 put_object_retention` with `Mode=COMPLIANCE` and `RetainUntilDate=<computed>`. Compliance mode means even the AWS root account cannot delete or shorten the retention.

8. **MUST** pin each tenant's documents to the residency region per TASK-AI-016 (per DEC-281). Lookup function `residency::resolve(tenant_id) -> (region, bucket, kms_key)`. The mapping:
- VN tenants → region `ap-southeast-1` + bucket `cyberos-doc-vn-1-<scope>` + KMS key alias `alias/cyberos-doc-vn-1-<scope>`.
- EU tenants → region `eu-west-1` + bucket `cyberos-doc-eu-1-<scope>` + KMS key alias `alias/cyberos-doc-eu-1-<scope>`.
- SG tenants → `ap-southeast-1` + bucket `cyberos-doc-sg-1-<scope>` (separate from VN bucket).
- US tenants → `us-east-1` + bucket `cyberos-doc-us-1-<scope>`. The `s3_region`, `s3_bucket`, `kms_key_id` columns capture the resolution at upload time; downstream operations use these (not the residency resolver) to avoid races during tenant residency changes.

9. **MUST** support **versioning at every modification** (per DEC-284). Every upload of a new version of an existing document creates:
- A new row in `document_versions` with monotonic `version_number`.
- A new S3 object stored at `<s3_key>?versionId=<aws-version-id>` (S3 Versioning enabled on the bucket).
- An update to `document_metadata.current_version_id` pointing at the new version.
- A `doc.versioned` memory audit row. All within one transaction.

10. **MUST** ship the **hash-chained per-document audit log** at `document_audit_log` table (per DEC-283). Schema: `id BIGSERIAL PRIMARY KEY`, `document_id UUID NOT NULL REFERENCES document_metadata(id)`, `tenant_id UUID NOT NULL`, `event_kind TEXT NOT NULL`, `event_payload JSONB NOT NULL`, `prev_hash CHAR(64)`, `chain_hash CHAR(64) NOT NULL`, `ts TIMESTAMPTZ NOT NULL DEFAULT now()`, `actor_subject_id UUID NOT NULL`. The `chain_hash` is `SHA-256(canonical(prev_hash || event_kind || event_payload || ts || actor_subject_id))`. A `BEFORE INSERT` trigger validates `prev_hash` matches the prior row's `chain_hash`; tampered chain → reject. The log is queryable via `document_audit_chain(doc_id) RETURNS SETOF document_audit_log ORDER BY id ASC`.

11. **MUST** emit the following 8 memory audit row kinds (per DEC-289):
- `doc.uploaded` — first upload (status=draft → version 1 written).
- `doc.versioned` — new version of existing doc.
- `doc.acl_changed` — bucket_scope or RBAC scope change.
- `doc.signed` — signing event from TASK-DOC-002/003/004 (placeholder kind at slice 1).
- `doc.archived` — status → archived; Object-Lock applied.
- `doc.legal_hold_applied` — LegalHold ON; CLO + CSO co-sign required (sev-1).
- `doc.legal_hold_released` — LegalHold OFF; same dual-signoff required (sev-1).
- `doc.access_audited` — every read of a document (per DEC-290; sev-2 — forensically relevant).

12. **MUST** enforce **legal hold dual-signoff** (per DEC-285). `POST /v1/doc/documents/{id}/legal-hold` accepts body `{"apply": true | false, "primary_signer_subject_id": <uuid>, "secondary_signer_subject_id": <uuid>, "reason": "<text>", "case_reference": "<text>"}`. Validation:
- `primary_signer` MUST have role `clo` per TASK-AUTH-101.
- `secondary_signer` MUST have role `cso` (security CSO, `cseco`) per TASK-AUTH-101.
- `primary != secondary` (cannot self-co-sign).
- `case_reference` length 5–200 chars.
- On apply: S3 `put_object_legal_hold` with `Status=ON`; `legal_hold=true` in metadata; `doc.legal_hold_applied` row.
- On release: same shape; `Status=OFF`; `doc.legal_hold_released` row.

13. **MUST** block GDPR/PDPL erasure requests while `legal_hold=true` (per §1 #12 + GDPR Art. 17). Erasure handler (placeholder; slice 2) calls `is_erasure_blocked(doc_id) -> bool` from this task; true → erasure refused with reason `legal_hold_active`. This task ships the predicate; the erasure handler ships in task-DOC-2xx.

14. **MUST** forbid cross-bucket-scope moves (per DEC-292). A `BEFORE UPDATE` trigger on `document_metadata` rejects `UPDATE ... SET bucket_scope = <different>` with `cross_scope_move_forbidden`. Once uploaded to `hr_contracts`, the doc cannot be migrated to `generic`; the keyspace + retention policy + RBAC scope are bound.

15. **MUST** lock `retention_until` post-archive (per DEC-287). A `BEFORE UPDATE` trigger rejects mutation of `retention_until` when prior `status='archived'`. The retention period is set ONCE at archive transition, computed from the scope's policy.

16. **MUST** expose REST handlers:
- `POST /v1/doc/documents` — initialise document metadata (no body upload yet); returns `{document_id, presigned_upload_url, presigned_form_fields}` for S3 direct-upload.
- `POST /v1/doc/documents/{id}/finalize` — called after S3 upload completes; validates SHA-256 server-side against S3 object's etag; creates version 1.
- `POST /v1/doc/documents/{id}/versions` — same flow for new versions.
- `GET /v1/doc/documents/{id}` — return metadata + presigned download URL.
- `GET /v1/doc/documents?bucket_scope=<>&status=<>&parent_object_id=<>` — list with cursor pagination.
- `PATCH /v1/doc/documents/{id}` — update non-scope-bound fields (original_filename, parent_object_id); rejects bucket_scope changes.
- `POST /v1/doc/documents/{id}/archive` — transition to archived; applies Object-Lock retention.
- `POST /v1/doc/documents/{id}/legal-hold` — dual-signoff legal hold workflow.

17. **MUST** verify SHA-256 integrity at every upload (per §1 #1 + S3 ETag check at finalize). Client computes locally + sends as `x-amz-content-sha256`; server validates against actual byte-stream + against S3 ETag. Mismatch → 409 `integrity_mismatch`.

18. **MUST** complete create + get + list handlers in ≤ 100 ms p95 (excluding the S3 round-trip; presigned URL generation is local). `documents_perf_test` asserts.

19. **MUST** support idempotent creation via `Idempotency-Key` header (same semantics as TASK-AUTH-002 §1 #6).

20. **MUST** scope RBAC permissions per bucket_scope (per TASK-AUTH-101): caller MUST have `Resource::DocDocument + Action::Write` for the specific scope. Slice 1 ships scope as a single resource; per-scope refinement (HR vs CRM scope override) lands in task-DOC-2xx via scope_grants.

21. **MUST** emit OTel span `doc.{upload,version,archive,download,legal_hold,acl_change}` with attributes: `tenant_id`, `document_id`, `bucket_scope`, `status`, `outcome` (success | invalid_scope | residency_mismatch | integrity_mismatch | legal_hold_blocked | not_found | permission_denied | object_lock_apply_failed | etag_mismatch).

22. **MUST** emit OTel metrics:
- `doc_upload_total{outcome, bucket_scope}` (counter).
- `doc_archive_total{outcome, retention_years}` (counter).
- `doc_legal_hold_total{outcome, apply}` (counter; `apply` ∈ {true, false}).
- `doc_access_total{tenant_id, bucket_scope}` (counter; downloads emit sev-2 audit).
- `doc_bytes_stored{tenant_id, bucket_scope}` (gauge — periodic compute via S3 inventory; eventual consistency).
- `doc_count{tenant_id, bucket_scope, status}` (gauge).

23. **MUST** ensure access (GET /download) emits a `doc.access_audited` memory row at sev-2 per access (per DEC-290). The row carries `{document_id, accessed_by_subject_id_hash16, purpose, requesting_ip_hash16, ts_ns}`. `purpose` is supplied by the caller as a required field (`?purpose=<text>` query param, 1-200 chars); absent → 400.

24. **MUST** ship `bucket_retention_policy(scope bucket_scope) RETURNS INT` SQL function returning the retention years per scope (50 for hr_contracts, 75 for esop_grants, 10 for others). Deterministic; same input → same output.

25. **MUST** validate residency match at upload: if the caller's tenant residency = `vn-1` and the resolved S3 bucket is `eu-1`, reject with 400 `residency_mismatch`. The check is at the handler boundary; the residency resolver should always return the correct match, but the validation is defence-in-depth.

26. **MUST** support cursor pagination on list with `?cursor=<opaque>&limit=<int>` (max 200, default 50).

---

## §2 — Why this design (rationale for humans)

**Why S3 Object-Lock Compliance mode (DEC-280, §1 #7)?** "Retention is non-negotiable" is the legal guarantee. Object-Lock Compliance mode means even AWS root credentials cannot delete or shorten the retention. Governance mode (the alternative) allows root override — defeats the guarantee. Compliance mode is the only path to eIDAS Art. 32 conformance for long-term electronic-signature preservation.

**Why per-tenant residency pinning (DEC-281)?** Three reasons converge: (1) PDPL + Decree 53/2022 require VN-citizen personal data to remain in-country; (2) GDPR + Schrems II require EU-citizen data to remain in qualified jurisdictions; (3) US ESIGN doesn't restrict location but auditability is regional. The resolver (joined with TASK-AI-016) picks the bucket + KMS at upload time and records the resolution on the row — so future operations don't have to re-resolve (and can't race against tenant residency changes).

**Why closed bucket_scope enum (DEC-282, §1 #3)?** Scope drives the RBAC role, the retention period, the KMS keyspace. Free-form scopes would proliferate ("`hr-contracts-2026`", "`hr_contracts`", "`hrcontracts`") and break the downstream cascade. 7 values cover the major buckets; adding an 8th is an ADR that forces consideration of: (a) what's the retention period? (b) which KMS keyspace? (c) which RBAC role gates it? — the questions that prevent silent over-permissioning.

**Why hash-chained per-document audit log (DEC-283, §1 #10)?** Signed documents are court-admissible; the chain-of-custody must be verifiable. Hash chaining (`chain_hash = SHA-256(prev_hash || event)`) makes tamper detectable: if any row is modified or removed, every subsequent `chain_hash` breaks. The chain replays end-to-end via `document_audit_chain(doc_id)`. The trigger validates at INSERT — operators inserting backdated rows cannot fool the chain.

**Why immutable versions (DEC-284, §1 #6, §1 #9)?** Signatures bind to specific bytes; "the signed document" is `v2`, not "the current state of the doc". Editing in place would mean the signed bytes no longer match the stored bytes — signature verification fails. Every save creates a new version with its own SHA-256; the signature ties to a specific version's hash.

**Why retention varies by scope (DEC-287, §1 #7)?** Different document types have different legal retention floors. VN Labour Code Art. 161 requires 50-year retention for employment contracts. US ERISA-equivalent practices retain ESOP records for 75 years. Generic + most commercial contracts: 10 years (eIDAS + ESIGN floor). The bucket_retention_policy function encodes the lookup; the migration's CHECK constraint enforces `retention_until >= now() + bucket_retention_policy(scope) years`.

**Why dual-signoff for legal hold (DEC-285, §1 #12)?** Legal hold is "this document cannot be deleted, even if retention expires, because litigation is anticipated". Misuse (operator applies hold to suppress a document indefinitely) is a real risk. Dual-signoff (CLO + CSO) creates a "two trusted operators agree" gate; the memory audit row at sev-1 alerts other operators. No auto-apply; never single-signer.

**Why GDPR/PDPL erasure blocked under hold (§1 #13)?** Legal hold supersedes erasure rights under most jurisdictions — if there's ongoing litigation requiring document preservation, the regulator wouldn't honour erasure. The predicate `is_erasure_blocked` returns true when `legal_hold=true`; the erasure handler refuses with a reason that makes it visible (so the data subject knows why their request was deferred).

**Why cross-bucket-scope moves forbidden (DEC-292, §1 #14)?** Scope binds retention + KMS + RBAC. Moving an HR contract to generic would (a) shorten retention from 50y to 10y, (b) switch KMS keyspace (encryption boundary breaks), (c) widen RBAC access. Forbidding moves at the trigger forces operators to re-upload under the new scope — making the boundary-cross visible.

**Why retention_until immutable post-archive (DEC-287, §1 #15)?** Object-Lock Compliance binds retention to the date the object was put-locked; the Postgres metadata mirrors that. Allowing UPDATE on `retention_until` post-archive would make the metadata diverge from S3 truth — operators would believe retention was shortened when S3 won't allow deletion. Lock the metadata to match S3.

**Why presigned URLs for upload (§1 #16)?** S3 direct-upload from the client (browser, mobile) avoids streaming bytes through our service. Presigned URLs include the bucket, key, content-type, and a signature; client uploads directly. Our service emits the URL + form fields + records pending metadata; on `finalize`, we verify the SHA-256 + ETag.

**Why finalize step + ETag check (§1 #17)?** Direct-upload presigned URLs don't prevent a client from uploading any bytes (within size + type limits). The finalize step compares (a) the client-supplied SHA-256 against the actually-uploaded ETag (which S3 computes from the upload); (b) the byte count. Mismatch → object is orphaned (S3 lifecycle cleanup after 24h) + 409 error.

**Why access emits sev-2 audit (DEC-290, §1 #23, §1 #11)?** Document access is a forensically-relevant event — "did Person A read Person B's contract on Date X?". Sev-2 means it goes through TASK-OBS-007's operator digest. Routine high-volume access (e.g. operations dashboard auto-fetching every contract) would create noise; the `purpose` requirement filters trivial cases (caller must supply a purpose; routine systems use a stable purpose code).

**Why `parent_object_id` is a soft-typed FK (§1 #1)?** The doc may belong to an HR contract (TASK-HR-002), a CRM deal (TASK-CRM-004), an ESOP grant (TASK-ESOP-001), a vendor agreement (task-OPS-vendor-2xx). Cross-service FKs are complex; soft-typing on `parent_object_id` + `bucket_scope` together identifies the owner, queried via the appropriate module's API.

**Why max 500 MB per document (§1 #1)?** Above this size the upload latency makes the UX intolerable; the use cases (PDF contracts, scanned KYC docs, signed images) fit well below 500 MB. Larger media goes to a different path (task-MEDIA-* — out of scope). The CHECK constraint prevents the table from accidentally accepting GB-sized uploads.

**Why `s3_key` includes the document UUID (§1 #1, implicit in s3 upload)?** Stable + collision-free + opaque. The key shape is `<tenant_uuid>/<scope>/<doc_uuid>` (e.g. `5e8f1d2a-.../hr_contracts/01HG7V8...`). Listing operations are filtered server-side by tenant_uuid prefix; collisions are impossible (UUID v4).

**Why explicit `kms_key_id` on the row (§1 #1)?** Encryption boundary is intrinsic to the scope. Recording the KMS key id at upload prevents operators from accidentally re-encrypting under a different key (which would invalidate prior signed-document references). The key id is set ONCE at upload; never changes.

**Why versioned audit log + S3 versioning together?** Defense in depth — the Postgres audit log captures the operation; the S3 versioning preserves the bytes. If one fails, the other catches it. The hash chain makes the audit log's integrity verifiable; Object-Lock makes the S3 side immutable.

**Why no DELETE handler (§1 #16)?** Documents enter `archived` state and are retained for the retention period; they never get deleted. The DELETE operation simply doesn't exist in the API; legal-hold + Object-Lock + SQL grant all combine to make deletion impossible.

**Why `purpose` required on GET (§1 #23)?** Forces the caller to declare why they're reading the document. Operators see "Person A read Person B's contract for purpose: kyc_review" — actionable; without purpose, the audit row would say "Person A read Person B's contract" — context-free.

**Why slice 1 ships only the storage primitive (not signing, not lifecycle)?** Storage is the foundation; signing (TASK-DOC-002/003/004) and lifecycle (TASK-DOC-007) build on it. Splitting concerns keeps each task focused. The storage primitive alone is useful: it can hold HR contracts, CRM contracts, vendor agreements as draft/in_signing/archived documents — signing comes later but the storage works without it.

---

## §3 — API contract

### 3.1 — Migration 0001 — document_metadata + document_versions

```sql
-- services/doc/migrations/0001_document_metadata.sql

BEGIN;

CREATE TYPE bucket_scope AS ENUM (
    'hr_contracts', 'crm_contracts', 'esop_grants', 'kyc_docs',
    'vendor_contracts', 'policies', 'generic'
);

CREATE TYPE document_status AS ENUM ('draft', 'in_signing', 'fully_signed', 'archived');

CREATE TABLE document_versions (
    id                     UUID         PRIMARY KEY,
    document_id            UUID         NOT NULL,
    tenant_id              UUID         NOT NULL,
    version_number         INT          NOT NULL,
    s3_key_versioned       TEXT         NOT NULL,
    sha256_hex             CHAR(64)     NOT NULL CHECK (sha256_hex ~ '^[0-9a-f]{64}$'),
    byte_size              BIGINT       NOT NULL CHECK (byte_size BETWEEN 1 AND 524288000),
    mime_type              TEXT         NOT NULL,
    uploaded_at            TIMESTAMPTZ  NOT NULL DEFAULT now(),
    uploaded_by_subject_id UUID         NOT NULL REFERENCES auth.subjects(id) ON DELETE RESTRICT
);

CREATE TABLE document_metadata (
    id                     UUID         PRIMARY KEY,
    tenant_id              UUID         NOT NULL,
    bucket_scope           bucket_scope NOT NULL,
    original_filename      TEXT         NOT NULL CHECK (length(original_filename) BETWEEN 1 AND 255),
    mime_type              TEXT         NOT NULL,
    byte_size              BIGINT       NOT NULL CHECK (byte_size BETWEEN 1 AND 524288000),
    sha256_hex             CHAR(64)     NOT NULL CHECK (sha256_hex ~ '^[0-9a-f]{64}$'),
    status                 document_status NOT NULL DEFAULT 'draft',
    s3_region              TEXT         NOT NULL,
    s3_bucket              TEXT         NOT NULL,
    s3_key                 TEXT         NOT NULL,
    kms_key_id             TEXT         NOT NULL,
    current_version_id     UUID         NOT NULL REFERENCES document_versions(id) DEFERRABLE INITIALLY IMMEDIATE,
    legal_hold             BOOLEAN      NOT NULL DEFAULT false,
    retention_until        DATE,
    parent_object_id       UUID,                                  -- soft FK per bucket_scope; logical only
    created_at             TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at             TIMESTAMPTZ  NOT NULL DEFAULT now(),
    created_by_subject_id  UUID         NOT NULL REFERENCES auth.subjects(id) ON DELETE RESTRICT
);

ALTER TABLE document_versions ADD CONSTRAINT document_versions_doc_fk
    FOREIGN KEY (document_id) REFERENCES document_metadata(id) ON DELETE RESTRICT;

CREATE INDEX document_metadata_tenant_scope_idx ON document_metadata (tenant_id, bucket_scope, status);
CREATE INDEX document_metadata_tenant_parent_idx ON document_metadata (tenant_id, parent_object_id) WHERE parent_object_id IS NOT NULL;
CREATE INDEX document_versions_doc_version_idx ON document_versions (document_id, version_number DESC);

ALTER TABLE document_metadata ENABLE ROW LEVEL SECURITY;
ALTER TABLE document_versions ENABLE ROW LEVEL SECURITY;

CREATE POLICY document_metadata_tenant_isolation ON document_metadata
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

CREATE POLICY document_versions_tenant_isolation ON document_versions
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

-- Append-only on versions (DEC-286)
REVOKE UPDATE, DELETE ON document_versions FROM cyberos_app;

-- Retention policy lookup
CREATE OR REPLACE FUNCTION bucket_retention_policy(scope bucket_scope) RETURNS INT AS $$
BEGIN
    RETURN CASE scope
        WHEN 'hr_contracts'      THEN 50
        WHEN 'esop_grants'       THEN 75
        WHEN 'kyc_docs'          THEN 10
        WHEN 'crm_contracts'     THEN 10
        WHEN 'vendor_contracts'  THEN 10
        WHEN 'policies'          THEN 10
        WHEN 'generic'           THEN 10
    END;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Cross-scope move rejection (DEC-292)
CREATE OR REPLACE FUNCTION enforce_no_cross_scope_move() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.bucket_scope IS DISTINCT FROM OLD.bucket_scope THEN
        RAISE EXCEPTION 'cross_scope_move_forbidden' USING ERRCODE = 'P0030';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_metadata_no_cross_scope BEFORE UPDATE ON document_metadata
    FOR EACH ROW EXECUTE FUNCTION enforce_no_cross_scope_move();

-- Retention immutability post-archive (DEC-287)
CREATE OR REPLACE FUNCTION enforce_retention_immutable_post_archive() RETURNS TRIGGER AS $$
BEGIN
    IF OLD.status = 'archived' AND NEW.retention_until IS DISTINCT FROM OLD.retention_until THEN
        RAISE EXCEPTION 'retention_immutable_post_archive' USING ERRCODE = 'P0031';
    END IF;
    NEW.updated_at := now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_metadata_retention_lock BEFORE UPDATE ON document_metadata
    FOR EACH ROW EXECUTE FUNCTION enforce_retention_immutable_post_archive();

COMMIT;
```

### 3.2 — Migration 0002 — hash-chained audit log

```sql
-- services/doc/migrations/0002_document_audit_log.sql

BEGIN;

CREATE TABLE document_audit_log (
    id                     BIGSERIAL    PRIMARY KEY,
    document_id            UUID         NOT NULL REFERENCES document_metadata(id) ON DELETE RESTRICT,
    tenant_id              UUID         NOT NULL,
    event_kind             TEXT         NOT NULL,
    event_payload          JSONB        NOT NULL,
    prev_hash              CHAR(64),
    chain_hash             CHAR(64)     NOT NULL CHECK (chain_hash ~ '^[0-9a-f]{64}$'),
    ts                     TIMESTAMPTZ  NOT NULL DEFAULT now(),
    actor_subject_id       UUID         NOT NULL REFERENCES auth.subjects(id)
);

CREATE INDEX doc_audit_log_doc_idx ON document_audit_log (document_id, id ASC);

ALTER TABLE document_audit_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY doc_audit_log_tenant_isolation ON document_audit_log
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

REVOKE UPDATE, DELETE ON document_audit_log FROM cyberos_app;

-- Chain integrity: NEW.prev_hash MUST equal previous row's chain_hash (DEC-283)
CREATE OR REPLACE FUNCTION enforce_audit_chain_integrity() RETURNS TRIGGER AS $$
DECLARE expected_prev CHAR(64);
BEGIN
    SELECT chain_hash INTO expected_prev FROM document_audit_log
        WHERE document_id = NEW.document_id ORDER BY id DESC LIMIT 1;
    IF expected_prev IS NULL AND NEW.prev_hash IS NOT NULL THEN
        RAISE EXCEPTION 'audit_chain_bootstrap_violation' USING ERRCODE = 'P0040';
    END IF;
    IF expected_prev IS NOT NULL AND NEW.prev_hash IS DISTINCT FROM expected_prev THEN
        RAISE EXCEPTION 'audit_chain_break' USING ERRCODE = 'P0041';
    END IF;
    -- chain_hash itself is computed by the application; trigger validates the link
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_doc_audit_chain BEFORE INSERT ON document_audit_log
    FOR EACH ROW EXECUTE FUNCTION enforce_audit_chain_integrity();

-- Chain replayer
CREATE OR REPLACE FUNCTION document_audit_chain(p_doc_id UUID)
RETURNS SETOF document_audit_log AS $$
    SELECT * FROM document_audit_log WHERE document_id = p_doc_id ORDER BY id ASC
$$ LANGUAGE sql STABLE;

COMMIT;
```

### 3.3 — Rust types

```rust
// services/doc/src/types.rs
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "bucket_scope", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum BucketScope {
    HrContracts, CrmContracts, EsopGrants, KycDocs,
    VendorContracts, Policies, Generic,
}

impl BucketScope {
    pub const ALL: &'static [BucketScope] = &[
        BucketScope::HrContracts, BucketScope::CrmContracts, BucketScope::EsopGrants,
        BucketScope::KycDocs, BucketScope::VendorContracts, BucketScope::Policies, BucketScope::Generic,
    ];

    pub fn retention_years(self) -> u32 {
        match self {
            BucketScope::HrContracts => 50,
            BucketScope::EsopGrants => 75,
            _ => 10,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "document_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum DocumentStatus { Draft, InSigning, FullySigned, Archived }

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub bucket_scope: BucketScope,
    pub original_filename: String,
    pub mime_type: String,
    pub byte_size: i64,
    pub sha256_hex: String,
    pub status: DocumentStatus,
    pub s3_region: String,
    pub s3_bucket: String,
    pub s3_key: String,
    pub kms_key_id: String,
    pub current_version_id: Uuid,
    pub legal_hold: bool,
    pub retention_until: Option<NaiveDate>,
    pub parent_object_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by_subject_id: Uuid,
}
```

### 3.4 — Residency resolver

```rust
// services/doc/src/residency.rs
use uuid::Uuid;
use crate::types::BucketScope;

#[derive(Debug, Clone)]
pub struct StorageBinding {
    pub region: String,
    pub bucket: String,
    pub kms_key_id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ResidencyError {
    #[error("unknown_tenant_residency: {0}")]
    Unknown(Uuid),
    #[error("residency_disabled_for_scope: {scope:?} in {residency}")]
    ScopeDisabled { scope: BucketScope, residency: String },
}

/// Resolve tenant's residency tag and produce the (region, bucket, KMS key) triple
/// for the given bucket_scope. Consumes TASK-AI-016's residency policy.
pub async fn resolve(tenant_id: Uuid, scope: BucketScope, db: &sqlx::PgPool) -> Result<StorageBinding, ResidencyError> {
    let residency: String = sqlx::query_scalar("SELECT residency FROM tenant_residency WHERE tenant_id = $1")
        .bind(tenant_id).fetch_optional(db).await.unwrap().ok_or(ResidencyError::Unknown(tenant_id))?;

    let (region, bucket_prefix) = match residency.as_str() {
        "vn-1" => ("ap-southeast-1", "cyberos-doc-vn-1"),
        "sg-1" => ("ap-southeast-1", "cyberos-doc-sg-1"),
        "eu-1" => ("eu-west-1",      "cyberos-doc-eu-1"),
        "us-1" => ("us-east-1",      "cyberos-doc-us-1"),
        _ => return Err(ResidencyError::Unknown(tenant_id)),
    };

    let scope_str: &str = serde_plain::to_string(&scope).unwrap_or_default();
    let bucket = format!("{bucket_prefix}-{scope_str}");
    let kms_key_id = format!("alias/{bucket}");

    Ok(StorageBinding { region: region.into(), bucket, kms_key_id })
}
```

### 3.5 — Object Lock application

```rust
// services/doc/src/s3/object_lock.rs
use aws_sdk_s3::types::{ObjectLockMode, ObjectLockLegalHoldStatus};
use chrono::{Duration, Utc};
use crate::types::{BucketScope, DocumentMetadata};

pub async fn apply_archive_retention(
    s3: &aws_sdk_s3::Client,
    metadata: &DocumentMetadata,
) -> Result<chrono::NaiveDate, anyhow::Error> {
    let years = metadata.bucket_scope.retention_years();
    let until = (Utc::now() + Duration::days(365 * years as i64 + (years / 4) as i64)).naive_utc().date();
    s3.put_object_retention()
        .bucket(&metadata.s3_bucket)
        .key(&metadata.s3_key)
        .version_id(metadata.current_version_id.to_string())   // pin to current version
        .retention(aws_sdk_s3::types::ObjectLockRetention::builder()
            .mode(ObjectLockMode::Compliance)
            .retain_until_date(aws_smithy_types::DateTime::from_secs(until.and_hms_opt(0,0,0).unwrap().and_utc().timestamp()))
            .build())
        .send().await?;
    Ok(until)
}

pub async fn apply_legal_hold(
    s3: &aws_sdk_s3::Client,
    metadata: &DocumentMetadata,
    on: bool,
) -> Result<(), anyhow::Error> {
    let status = if on { ObjectLockLegalHoldStatus::On } else { ObjectLockLegalHoldStatus::Off };
    s3.put_object_legal_hold()
        .bucket(&metadata.s3_bucket)
        .key(&metadata.s3_key)
        .legal_hold(aws_sdk_s3::types::ObjectLockLegalHold::builder().status(status).build())
        .send().await?;
    Ok(())
}
```

### 3.6 — Legal hold handler

```rust
// services/doc/src/handlers/legal_hold.rs
use axum::{Json, extract::{Path, State}, http::StatusCode};
use cyberos_auth::rbac::{Resource, Action, Role};
use crate::audit::doc_events;

#[derive(Deserialize)]
pub struct LegalHoldRequest {
    pub apply: bool,
    pub primary_signer_subject_id: Uuid,
    pub secondary_signer_subject_id: Uuid,
    pub reason: String,
    pub case_reference: String,
}

pub async fn legal_hold(
    State(state): State<AppState>,
    claims: Claims,
    Path(doc_id): Path<Uuid>,
    Json(req): Json<LegalHoldRequest>,
) -> Result<StatusCode, ApiError> {
    // Caller permission
    state.matrix.snapshot().require_permission(&claims.roles(), Resource::DocDocument, Action::Admin)?;

    if req.primary_signer_subject_id == req.secondary_signer_subject_id {
        return Err(ApiError::SelfCoSign);
    }
    if req.case_reference.len() < 5 || req.case_reference.len() > 200 {
        return Err(ApiError::CaseReferenceInvalid);
    }

    // Validate signers' roles
    let primary_roles = state.repo.subject_roles(req.primary_signer_subject_id).await?;
    if !primary_roles.contains(&Role::Clo) {
        return Err(ApiError::SignerRoleMismatch { required: Role::Clo, found: primary_roles });
    }
    let secondary_roles = state.repo.subject_roles(req.secondary_signer_subject_id).await?;
    if !secondary_roles.contains(&Role::Cseco) {
        return Err(ApiError::SignerRoleMismatch { required: Role::Cseco, found: secondary_roles });
    }

    let mut tx = state.db.begin().await?;
    let metadata = state.repo.get_metadata(doc_id, &mut tx).await?;

    // Apply at S3
    crate::s3::object_lock::apply_legal_hold(&state.s3, &metadata, req.apply).await?;

    // Update metadata
    sqlx::query("UPDATE document_metadata SET legal_hold = $2 WHERE id = $1")
        .bind(doc_id).bind(req.apply).execute(&mut *tx).await?;

    // Append audit log
    if req.apply {
        doc_events::emit_legal_hold_applied(&mut tx, &metadata, &req, claims.subject_id()).await?;
    } else {
        doc_events::emit_legal_hold_released(&mut tx, &metadata, &req, claims.subject_id()).await?;
    }

    tx.commit().await?;
    Ok(StatusCode::OK)
}
```

---

## §4 — Acceptance criteria

1. **BucketScope enum closed at 7** — `BucketScope::ALL.len() == 7`; Postgres enum has exactly 7 labels.
2. **DocumentStatus enum closed at 4** — same shape.
3. **RLS isolates by tenant** — query as tenant-A returns 0 docs of tenant-B.
4. **POST + finalize creates document + version 1** — happy path returns metadata + version row; `doc.uploaded` memory row emitted.
5. **Cross-scope move rejected** — `UPDATE document_metadata SET bucket_scope = 'generic' WHERE id = $1` (originally hr_contracts) → trigger `cross_scope_move_forbidden`.
6. **UPDATE document_versions blocked** — `UPDATE document_versions ...` as cyberos_app → permission denied.
7. **DELETE document_versions blocked** — same.
8. **Residency pin: VN tenant uploads to vn-1 bucket** — resolver returns `s3_region='ap-southeast-1'`, `bucket='cyberos-doc-vn-1-hr_contracts'`.
9. **Residency mismatch caught** — handler with wrong region → 400 `residency_mismatch`.
10. **Object-Lock retention applied on archive** — POST /archive → S3 mock receives `PutObjectRetention` with `Mode=COMPLIANCE` and `RetainUntilDate` matching scope policy.
11. **Retention immutable post-archive** — UPDATE retention_until after status=archived → trigger `retention_immutable_post_archive`.
12. **Retention by scope: hr_contracts=50y** — `SELECT bucket_retention_policy('hr_contracts')` returns 50.
13. **Retention by scope: esop_grants=75y** — returns 75.
14. **Retention by scope: generic=10y** — returns 10.
15. **Legal hold dual-signoff** — CLO + CSO co-sign → 200; CLO alone → 403 `signer_role_mismatch`.
16. **Legal hold self-co-sign rejected** — primary_signer == secondary_signer → 400 `self_co_sign`.
17. **GDPR erasure blocked under hold** — `is_erasure_blocked(doc_id)` returns true; erasure handler refuses with `legal_hold_active`.
18. **doc.access_audited emitted on GET /download** — sev-2 memory row with purpose; missing purpose → 400.
19. **Audit log chain integrity** — INSERT with mismatched `prev_hash` → trigger `audit_chain_break`.
20. **Audit log replay end-to-end** — `document_audit_chain(doc_id)` returns rows in order; SHA-256 chain verified end-to-end.
21. **Idempotent create** — same Idempotency-Key + same body → same document.
22. **SHA-256 mismatch on finalize** — client-supplied vs S3-actual ETag differ → 409 `integrity_mismatch`.
23. **OTel span `doc.upload` emitted** — with `outcome=success`.
24. **Counter `doc_upload_total{outcome=success, bucket_scope=hr_contracts}` increments** — per upload.
25. **Counter `doc_legal_hold_total{outcome=success, apply=true}` increments** — per hold apply.
26. **Counter `doc_access_total{tenant_id, bucket_scope}` increments** — per GET.
27. **Subject FK ON DELETE RESTRICT** — DELETE auth.subjects fails if documents reference.
28. **Perf budget < 100ms p95** — `documents_perf_test` 1000 iterations.

---

## §5 — Verification

```rust
// services/doc/tests/bucket_scope_enum_test.rs
#[test]
fn bucket_scope_has_exactly_7_values() {
    assert_eq!(BucketScope::ALL.len(), 7);
}

#[sqlx::test]
async fn pg_bucket_scope_enum_matches_rust(pool: sqlx::PgPool) {
    let labels: Vec<String> = sqlx::query_scalar(
        "SELECT unnest(enum_range(NULL::bucket_scope))::text ORDER BY 1"
    ).fetch_all(&pool).await.unwrap();
    let expected: Vec<String> = vec![
        "crm_contracts","esop_grants","generic","hr_contracts","kyc_docs","policies","vendor_contracts"
    ].into_iter().map(String::from).collect();
    assert_eq!(labels, expected);
}
```

```rust
// services/doc/tests/retention_period_test.rs
#[sqlx::test]
async fn retention_periods_per_scope(pool: sqlx::PgPool) {
    let cases = [
        ("hr_contracts", 50), ("esop_grants", 75), ("kyc_docs", 10),
        ("crm_contracts", 10), ("vendor_contracts", 10), ("policies", 10), ("generic", 10),
    ];
    for (scope, expected) in cases {
        let got: i32 = sqlx::query_scalar(&format!("SELECT bucket_retention_policy('{scope}')"))
            .fetch_one(&pool).await.unwrap();
        assert_eq!(got, expected, "scope = {scope}");
    }
}
```

```rust
// services/doc/tests/legal_hold_dual_signoff_test.rs
#[tokio::test]
async fn single_signer_rejected(ctx: TestCtx) {
    let doc = ctx.create_doc(BucketScope::HrContracts).await;
    let clo = ctx.subject_with_role(Role::Clo).await;
    let other = ctx.subject_with_role(Role::Cfo).await;   // not cseco
    let err = ctx.post_legal_hold(doc.id, json!({
        "apply": true,
        "primary_signer_subject_id": clo,
        "secondary_signer_subject_id": other,
        "reason":"litigation x",
        "case_reference":"CASE-2026-Q3-001"
    })).await.unwrap_err();
    assert!(format!("{err}").contains("signer_role_mismatch"));
}

#[tokio::test]
async fn clo_plus_cseco_accepted(ctx: TestCtx) {
    let doc = ctx.create_doc(BucketScope::HrContracts).await;
    let clo = ctx.subject_with_role(Role::Clo).await;
    let cseco = ctx.subject_with_role(Role::Cseco).await;
    let resp = ctx.post_legal_hold(doc.id, json!({
        "apply": true,
        "primary_signer_subject_id": clo,
        "secondary_signer_subject_id": cseco,
        "reason":"litigation x",
        "case_reference":"CASE-2026-Q3-001"
    })).await.unwrap();
    assert_eq!(resp.status(), 200);
    let rows = ctx.memory_audit_rows("doc.legal_hold_applied").await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["severity"], "sev-1");
}
```

```rust
// services/doc/tests/cross_scope_move_rejected_test.rs
#[sqlx::test]
async fn cross_scope_update_rejected(pool: sqlx::PgPool) {
    let doc = seed_doc_with_scope(&pool, BucketScope::HrContracts).await;
    let err = sqlx::query("UPDATE document_metadata SET bucket_scope = 'generic'::bucket_scope WHERE id = $1")
        .bind(doc).execute(&pool).await.unwrap_err();
    assert!(format!("{err}").contains("cross_scope_move_forbidden"));
}
```

```rust
// services/doc/tests/audit_log_chain_test.rs
#[sqlx::test]
async fn chain_integrity_enforced(pool: sqlx::PgPool) {
    let doc_id = seed_doc(&pool).await;
    let row1 = insert_audit_event(&pool, doc_id, "doc.uploaded", None).await;
    // Attempt to insert with mismatched prev_hash
    let err = sqlx::query("INSERT INTO document_audit_log (document_id, tenant_id, event_kind, event_payload, prev_hash, chain_hash, actor_subject_id) VALUES ($1, $2, 'doc.versioned', '{}'::jsonb, $3, $4, $5)")
        .bind(doc_id).bind(test_tenant()).bind("bad_prev_hash_64chars_00000000000000000000000000000000000000000000")
        .bind("any_chain_hash_64chars_0000000000000000000000000000000000000000")
        .bind(test_subject())
        .execute(&pool).await.unwrap_err();
    assert!(format!("{err}").contains("audit_chain_break"));
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton. The 8 memory row builders in `audit/doc_events.rs` follow the canonical pattern.)

---

## §7 — Dependencies

**Upstream:**
- **TASK-AUTH-101** — RBAC; `Resource::DocDocument + Action::Write/Read/Admin`; signer-role validation against the closed enum.

**Downstream (6 placeholders):**
- **TASK-DOC-002** — eIDAS QTSP partner integration (EU residency).
- **TASK-DOC-003** — AATL CA integration (US/non-EU enterprise).
- **TASK-DOC-004** — VNeID + VN CA chain (VN tenants).
- **TASK-DOC-005** — multi-party signing workflow.
- **TASK-DOC-007** — lifecycle metadata (parties, dates, renewal).
- **TASK-DOC-010** — DocuSign / Adobe Sign / HelloSign import with LTV preservation.

**Cross-module:**
- **TASK-AUTH-003** — RLS enforcement.
- **TASK-AI-003** — memory audit bridge; receives the 8 `doc.*` audit row kinds.
- **TASK-AI-016** — residency policy; this task consumes the per-tenant residency tag.
- **TASK-MEMORY-111** — PII scrubbing on `event_payload` fields containing PII (e.g. names in reason).

---

## §8 — Example payloads

### 8.1 — POST /v1/doc/documents request (initiate)

```json
{
  "bucket_scope": "hr_contracts",
  "original_filename": "employment_contract_2026.pdf",
  "mime_type": "application/pdf",
  "byte_size": 245760,
  "sha256_hex": "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
  "parent_object_id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d"
}
```

### 8.2 — 201 CREATED response (with presigned upload URL)

```json
{
  "document_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "presigned_upload_url": "https://cyberos-doc-vn-1-hr_contracts.s3.ap-southeast-1.amazonaws.com/5e8f.../hr_contracts/01HG7V8B...?X-Amz-Signature=...",
  "presigned_form_fields": {"x-amz-server-side-encryption": "aws:kms", "x-amz-server-side-encryption-aws-kms-key-id": "alias/cyberos-doc-vn-1-hr_contracts"},
  "expires_at": "2026-05-16T11:00:00Z"
}
```

### 8.3 — doc.uploaded memory row

```json
{
  "kind": "doc.uploaded",
  "tenant_id": "5e8f1d2a-...",
  "document_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "bucket_scope": "hr_contracts",
  "byte_size": 245760,
  "mime_type": "application/pdf",
  "sha256_hex": "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
  "s3_region": "ap-southeast-1",
  "s3_bucket": "cyberos-doc-vn-1-hr_contracts",
  "uploaded_by_subject_id_hash16": "9b1deb4d3b7d4bad",
  "ts_ns": 1747920731000000000
}
```

### 8.4 — POST /v1/doc/documents/{id}/legal-hold request

```json
{
  "apply": true,
  "primary_signer_subject_id": "<clo-uuid>",
  "secondary_signer_subject_id": "<cseco-uuid>",
  "reason": "Ongoing wrongful-termination claim filed 2026-09-15; preserve all related employment records",
  "case_reference": "CASE-2026-Q3-001"
}
```

### 8.5 — doc.legal_hold_applied memory row (sev-1)

```json
{
  "kind": "doc.legal_hold_applied",
  "severity": "sev-1",
  "tenant_id": "5e8f1d2a-...",
  "document_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "primary_signer_subject_id_hash16": "abcdef0123456789",
  "secondary_signer_subject_id_hash16": "0987654321fedcba",
  "case_reference": "CASE-2026-Q3-001",
  "reason_scrubbed": "Ongoing wrongful-termination claim filed [DATE]; preserve all related employment records",
  "ts_ns": 1747920731000000000
}
```

### 8.6 — doc.archived memory row

```json
{
  "kind": "doc.archived",
  "tenant_id": "5e8f1d2a-...",
  "document_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "bucket_scope": "hr_contracts",
  "retention_until": "2076-05-16",
  "object_lock_mode": "COMPLIANCE",
  "archived_by_subject_id_hash16": "9b1deb4d3b7d4bad",
  "ts_ns": 1747920731000000000
}
```

---

## §9 — Open questions

Deferred:
- **eIDAS QTSP / AATL / VNeID signing flows** — TASK-DOC-002/003/004.
- **Multi-party signing workflow** — TASK-DOC-005.
- **Lifecycle metadata (parties, dates, renewal)** — TASK-DOC-007.
- **Expiry alert cascade** — TASK-DOC-008.
- **Renewal proposal CUO draft** — TASK-DOC-009.
- **DocuSign / Adobe Sign / HelloSign import** — TASK-DOC-010.
- **PAdES-B-LT format with re-stamping** — TASK-DOC-011.
- **GDPR erasure handler** — task-DOC-2xx; this task ships the `is_erasure_blocked` predicate.
- **Per-scope RBAC refinement via scope_grants** — task-DOC-2xx.

All other questions resolved.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| UPDATE on document_versions | SQL grant | Permission denied | None — designed |
| DELETE on document_versions | SQL grant | Permission denied | None — designed |
| Cross-scope move attempted | trigger `cross_scope_move_forbidden` | 400 | Re-upload under correct scope |
| Retention mutation post-archive | trigger `retention_immutable_post_archive` | 400 | None — designed |
| Object-Lock S3 API failure | aws-sdk error | 500 `object_lock_apply_failed` + sev-1 alarm | Retry; if persistent, investigate IAM permission |
| Residency resolver returns unknown tenant | `ResidencyError::Unknown` | 500 | Provision tenant residency |
| Residency mismatch handler-side | mismatch check | 400 `residency_mismatch` | None — designed |
| SHA-256 mismatch at finalize | client supplied vs S3 ETag | 409 `integrity_mismatch` | Re-upload |
| Byte size > 500MB | DB CHECK | 400 | Use task-MEDIA-* path |
| Legal hold without dual-signoff | handler validates roles | 400 `signer_role_mismatch` | Add appropriate signer |
| Legal hold self-co-sign | handler check | 400 `self_co_sign` | Use distinct signers |
| Case reference too short | handler validation | 400 `case_reference_invalid` | Provide 5–200 char ref |
| Audit chain break (tampered prev_hash) | trigger `audit_chain_break` | 500 | Investigate tamper; chain must be repaired manually + sev-1 alarm |
| memory row commit fails | transaction rolls back | 500 `audit_failed` | memory_writer diagnosis |
| Missing purpose on GET /download | handler | 400 `purpose_required` | Add purpose query param |
| Cross-tenant FK violation | RLS | 0 rows / permission_denied | None — designed |
| Subject deleted while metadata references created_by | FK RESTRICT | DELETE auth.subjects fails | Designed |
| S3 PutObject failed mid-upload | client gets 5xx from S3 | Document stuck in draft | 24h cleanup via S3 lifecycle |
| AWS region not available (outage) | aws-sdk timeout | 500 with outage info | Wait + retry |
| KMS key disabled | aws-sdk error on PUT | 500 `kms_disabled` | Re-enable key or rotate per task-AUTH-2xx |
| Audit log replay reveals broken chain | application chain re-hash | sev-1 alarm | Investigate; manual repair |
| Object-Lock Compliance applied incorrectly (Governance instead) | aws-sdk response check | sev-1 alarm | Replay with correct mode |
| GDPR erasure with active hold | `is_erasure_blocked` returns true | 403 `legal_hold_active` | Defer erasure; document the deferral |
| Concurrent finalize | INSERT version_number conflict | 409 retry | Designed |
| Storage cost unexpectedly high (large bucket) | S3 inventory + alarm | sev-3 | Operator review |
| Versioning disabled on bucket | startup check fails | Service refuses to start | Re-enable + restart |
| KMS key rotated without metadata update | next read uses stale alias | None (alias resolves to current) | Designed (alias indirection) |
| presigned URL expired before upload | S3 returns 403 | Client retries with new presigned URL | Designed |
| Mime type spoofing | mime validation at finalize via libmagic | 400 `mime_mismatch` | Use correct content-type |
| LegalHold release without dual-signoff | handler validates roles same as apply | 400 | Provide CLO + CSO |
| Audit log filesize growth | partition by month | OK | None |
| RLS bypass attempt | `USING` predicate | 0 rows | None — designed |
| `parent_object_id` references deleted object | soft FK; no enforcement | Stale reference | Periodic cleanup job (task-DOC-2xx) |

---

## §11 — Implementation notes

- **S3 Object-Lock Compliance mode is the legal anchor** — once retention is set, no root admin can delete or shorten. Governance mode allows root override — defeats the purpose.
- **Per-tenant residency pinning recorded on the row** — future operations don't re-resolve; protects against tenant residency changes mid-flight.
- **Closed `bucket_scope` enum drives 3 things**: retention policy, KMS keyspace, RBAC scope. Each change is ADR-worthy.
- **Hash-chained audit log is per-document** — chain integrity scoped to one doc; no cross-doc dependencies; replay is one-table-scan.
- **`chain_hash` computed by application, validated by trigger** — application computes via SHA-256 over canonical JSON; trigger validates `prev_hash` matches prior row's `chain_hash`.
- **Versioning at S3 + Postgres** — defense in depth. S3 has versioning enabled; each upload creates a new S3 version + Postgres row.
- **`s3_key_versioned` includes the AWS version id** — for direct download of a specific version; the metadata `current_version_id` points at the Postgres row, which points at the S3 version.
- **Legal hold dual-signoff at CLO + CSO** — both legal AND security operator agree. Single signer would be too easy to misuse.
- **GDPR/PDPL erasure blocked under hold** — this is the legal trump (litigation > data minimisation under most jurisdictions). The `is_erasure_blocked` predicate is the contract for the erasure handler.
- **Access audit at sev-2** — every read is forensically relevant; the `purpose` requirement filters trivial cases.
- **`parent_object_id` soft-typed** — bucket_scope identifies the owner module; cross-service FK enforcement is operationally complex.
- **500 MB max** — covers PDFs + scanned images + signed documents. Larger media (video, audio attachments) goes through task-MEDIA-*.
- **S3 lifecycle cleanup of orphaned drafts at 24h** — uploads that never finalize get garbage-collected; metadata stays as draft until finalize or 24h expiry.
- **Retention computation uses `INTERVAL '<years>' year`** — leap-year aware via Postgres date math; deterministic.
- **`alias/<bucket-name>`** is the KMS alias convention — operations on the alias resolve to the current key version (supports rotation without app changes).
- **`PutObjectRetention` pins to a specific version_id** — locks the bytes of `current_version_id` only; future versions get their own retention via their own archive transitions.
- **`document_audit_chain(doc_id)` SQL function** — single replay query; useful for forensic + audit-replay tooling.
- **8 memory audit kinds** — recorded, versioned, acl_changed, signed, archived, legal_hold_applied, legal_hold_released, access_audited. Sev assignments: signed/archived = sev-3 (informational); acl_changed/access_audited = sev-2 (operator review); legal_hold_applied/released = sev-1 (high attention).
- **`reason` field PII-scrubbed in memory row** — operators may write sensitive case details; TASK-MEMORY-111 strips before chain commit.
- **`case_reference` 5–200 chars** — long enough for internal case numbers, short enough to discourage prose narrative.
- **`OBS sev-1 alarm on PutObjectRetention failure`** — Object-Lock-Compliance is critical; failure to apply is a sev-1 (the doc was supposed to be locked).
- **`Mode=COMPLIANCE` is the only valid mode** — Governance mode is forbidden by spec; if a future migration changes it, the CI gate should fail.
- **`signature_event_log` (TASK-DOC-002/003/004)** — those tasks add their own per-signature audit kinds; this task ships only `doc.signed` as a placeholder.
- **`document_metadata.kms_key_id`** is the actual key id used at write — recorded for forensic verification + rotation-safety.

---

*End of TASK-DOC-001.*
