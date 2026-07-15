---
id: TASK-TEN-105
title: "TEN signed-bundle export — deterministic zip + Ed25519 signature + memory audit anchor + chain-of-custody manifest for tenant offboarding"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: TEN
priority: p0
status: draft
verify: T
phase: P4
milestone: P4 · slice 2
slice: 2
owner: Stephen Cheng (CSO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-TEN-104, TASK-TEN-106, TASK-AUTH-101, TASK-MEMORY-101, TASK-DOC-001, TASK-AI-003, TASK-MEMORY-111, TASK-OBS-007, TASK-OBS-009]
depends_on: [TASK-TEN-104]
blocks: [TASK-TEN-106]

source_pages:
  - website/docs/modules/ten.html#offboarding
  - https://datatracker.ietf.org/doc/html/rfc8032  # Ed25519
  - https://en.wikipedia.org/wiki/Chain_of_custody

source_decisions:
  - DEC-1320 2026-05-17 — Tenant offboarding (TASK-TEN-104 termination flow) MUST produce a deterministic exportable bundle of all tenant data + signed manifest before permanent deletion; this is the auditable evidence chain
  - DEC-1321 2026-05-17 — Bundle format: ZIP with sorted entries + fixed mtime 2000-01-01T00:00:00Z + mode 0o644 (matches AGENTS.md §10 portability pattern + TASK-TEN-103 export deterministic guarantee)
  - DEC-1322 2026-05-17 — Per-tenant Ed25519 signing key generated at tenant provisioning; private key KMS-wrapped; public key published with bundle for verification
  - DEC-1323 2026-05-17 — Bundle contents: per-source JSON + CSV (PROJ, INV, DOC, CHAT, AUDIT_CHAIN, METERING, USERS) + signed-manifest.json + public-key.pem + README.md (verification instructions)
  - DEC-1324 2026-05-17 — Chain-of-custody manifest signed by Ed25519: covers SHA-256 of every file + bundle metadata (tenant_id, export_timestamp, total_size, file_count); operator countersignature optional
  - DEC-1325 2026-05-17 — memory chain anchor: bundle export emits `ten.bundle_exported` row containing manifest_sha256 + bundle_size; subsequent TASK-TEN-106 permanent-delete attestation includes the same hash for chain-linked evidence
  - DEC-1326 2026-05-17 — Closed enum `bundle_status` = {pending, building, ready, delivered, expired, failed}; CI cardinality asserts 6
  - DEC-1327 2026-05-17 — Bundle delivery: signed S3 URL with 30-day TTL (longer than DSAR's 7d because offboarding bundles are legal-evidence + may need multiple downloads); passphrase optional
  - DEC-1328 2026-05-17 — Async build via TASK-MCP-007 Tasks (long-running); typical 5min-1h for active tenants; progress via NATS
  - DEC-1329 2026-05-17 — Rate limit: 3 bundle requests per tenant per year (legal evidence + storage cost; not for routine exports)
  - DEC-1330 2026-05-17 — Pre-deletion gate: TASK-TEN-106 permanent-delete attestation MUST cite a successful bundle export within last 90 days; CHECK at attestation insert
  - DEC-1331 2026-05-17 — Bundle includes memory audit chain segment for the tenant (full history; not redacted) — this IS the legal evidence
  - DEC-1332 2026-05-17 — memory audit kinds: ten.bundle_export_initiated, ten.bundle_built, ten.bundle_delivered, ten.bundle_failed, ten.bundle_expired, ten.bundle_signature_generated
  - DEC-1333 2026-05-17 — Bundle SHA-256 anchored at TASK-OBS-009 chain-of-custody system for cross-system evidence; manifest signature verifiable years later via published Ed25519 public key

build_envelope:
  language: rust 1.81
  service: cyberos/services/ten/
  new_files:
    - services/ten/migrations/0026_tenant_bundle_exports.sql
    - services/ten/migrations/0027_tenant_signing_keys.sql
    - services/ten/src/bundle/mod.rs
    - services/ten/src/bundle/builder.rs
    - services/ten/src/bundle/signer.rs
    - services/ten/src/bundle/manifest.rs
    - services/ten/src/bundle/deterministic_zip.rs
    - services/ten/src/bundle/keygen.rs
    - services/ten/src/bundle/delivery.rs
    - services/ten/src/audit/bundle_events.rs
    - services/ten/src/handlers/bundle_routes.rs
    - services/ten/tests/bundle_deterministic_test.rs
    - services/ten/tests/bundle_signature_verify_test.rs
    - services/ten/tests/bundle_manifest_format_test.rs
    - services/ten/tests/bundle_chain_anchor_test.rs
    - services/ten/tests/bundle_status_enum_cardinality_test.rs
    - services/ten/tests/bundle_rate_limit_3_per_year_test.rs
    - services/ten/tests/bundle_delivery_30d_ttl_test.rs
    - services/ten/tests/bundle_audit_chain_completeness_test.rs
    - services/ten/tests/bundle_pre_deletion_gate_test.rs
    - services/ten/tests/bundle_audit_emission_test.rs

  modified_files:
    - services/ten/src/lib.rs
    - services/ten/src/provisioning/orchestrator.rs                    # generate Ed25519 keypair at provision
    - services/ten/Cargo.toml                                          # +ed25519-dalek + zip + age

  allowed_tools:
    - file_read: services/{ten,proj,inv,doc,chat,auth,memory}/**
    - file_write: services/ten/{src,tests,migrations}/**
    - bash: cd services/ten && cargo test bundle

  disallowed_tools:
    - non-deterministic zip ordering (per DEC-1321)
    - sign manifest with shared key (per DEC-1322 — per-tenant)
    - allow > 3 bundle requests per year (per DEC-1329)
    - allow permanent-delete without recent bundle (per DEC-1330)

effort_hours: 8
subtasks:
  - "0.4h: 0026_tenant_bundle_exports.sql + 0027_tenant_signing_keys.sql"
  - "0.4h: bundle/mod.rs + closed enum"
  - "0.4h: bundle/keygen.rs (Ed25519 at provisioning)"
  - "0.7h: bundle/builder.rs (cross-source aggregation)"
  - "0.6h: bundle/deterministic_zip.rs (sorted entries + fixed mtime)"
  - "0.5h: bundle/signer.rs (Ed25519 signature)"
  - "0.5h: bundle/manifest.rs (chain-of-custody format)"
  - "0.4h: bundle/delivery.rs (S3 30d signed URL)"
  - "0.3h: audit/bundle_events.rs (6 builders)"
  - "0.3h: handlers/bundle_routes.rs"
  - "0.4h: provisioning extension (keygen at provision)"
  - "2.5h: tests — 10 test files including deterministic-zip round-trip"
  - "0.6h: integration smoke with multi-source seeded fixture"

risk_if_skipped: "Without signed-bundle export, tenant offboarding (TASK-TEN-104) has no auditable evidence trail — when the tenant claims data wasn't fully exported or was tampered with, we have no proof. Without DEC-1322 per-tenant signing keys, one compromised key invalidates all tenants' bundles. Without DEC-1330 pre-deletion gate, TASK-TEN-106 permanent-delete could happen without a recovery option = data-loss incident. Without DEC-1321 deterministic zip, re-export shows hash mismatches → tenant disputes integrity. Without DEC-1331 audit chain inclusion, post-deletion legal queries have no evidence."
---

## §1 — Description (BCP-14 normative)

The TEN service **MUST** ship signed-bundle export at `services/ten/src/bundle/` per tenant offboarding (TASK-TEN-104 termination) producing deterministic ZIP + Ed25519-signed manifest + memory chain anchor, with 30-day signed URL delivery, async build via TASK-MCP-007 Tasks, 3-per-year rate limit, pre-deletion gate dependency, and 6 memory audit kinds.

1. **MUST** define closed `bundle_status` enum: `('pending','building','ready','delivered','expired','failed')` per DEC-1326. Cardinality asserts 6.

2. **MUST** define `tenant_bundle_exports` at migration `0026`: `(bundle_id UUID PRIMARY KEY, tenant_id UUID NOT NULL, status bundle_status NOT NULL DEFAULT 'pending', requested_at TIMESTAMPTZ NOT NULL DEFAULT now(), requested_by_subject_id UUID NOT NULL, build_started_at TIMESTAMPTZ, ready_at TIMESTAMPTZ, delivered_at TIMESTAMPTZ, expires_at TIMESTAMPTZ, s3_key TEXT, bundle_size_bytes BIGINT, file_count INT, manifest_sha256 CHAR(64), manifest_signature_kms_blob BYTEA, source_chain_head BIGINT, trace_id CHAR(32))`. Append-only.

3. **MUST** define `tenant_signing_keys` at migration `0027`: `(tenant_id UUID PRIMARY KEY, public_key_pem TEXT NOT NULL, private_key_kms_blob BYTEA NOT NULL, kms_key_id TEXT NOT NULL, generated_at TIMESTAMPTZ NOT NULL DEFAULT now())`. One key per tenant; generated at TASK-TEN-001 provisioning per DEC-1322.

4. **MUST** generate Ed25519 keypair at tenant provisioning per DEC-1322. Modification to TASK-TEN-001 orchestrator: after tenant row created, generate keypair via `ed25519-dalek`, KMS-encrypt private key, insert `tenant_signing_keys` row.

5. **MUST** expose `POST /v1/admin/tenants/{tid}/bundle/export` body `{ reason }`. Caller has `tenant_admin` OR `cso` role. Handler:
   - Rate-limit check per DEC-1329 (3 per year per tenant).
   - INSERTs bundle row with status='pending'.
   - Enqueues build task via TASK-MCP-007.
   - Emit `ten.bundle_export_initiated` sev-1.

6. **MUST** build bundle per DEC-1323 via `bundle/builder.rs`:
   - Per-source JSON: PROJ (all projects + sub-resources), INV (invoices + lines), DOC (metadata + S3 refs), CHAT (channels + messages), AUDIT_CHAIN (full memory segment for tenant), METERING (TASK-TEN-004 history), USERS (TASK-AUTH-002 subjects).
   - Per-source CSV (human-readable; same data).
   - signed-manifest.json (Ed25519-signed file inventory).
   - public-key.pem (verifier reference).
   - README.md (verification instructions).
   - Deterministic ZIP per §1 #8.
   - Computes SHA-256, file count, total size.

7. **MUST** include full memory audit chain segment per DEC-1331. Chain segment = all rows where tenant_id matches; chain hashes intact; verifier can replay locally.

8. **MUST** produce deterministic ZIP per DEC-1321 + task-audit skill rule 27-28:
   - Sorted entries (alphabetic by path).
   - Fixed mtime `2000-01-01T00:00:00Z`.
   - Fixed mode `0o644`.
   - ZIP_DEFLATED compression level 6.
   - No extra/optional ZIP fields (mac extras, unicode extras).
   - Two builds of same source data produce byte-identical zip.

9. **MUST** sign manifest with per-tenant Ed25519 key per DEC-1324. The `bundle/signer.rs::sign(manifest_bytes, tenant_id)`:
   - Loads private key from `tenant_signing_keys` (KMS-decrypt).
   - Signs `SHA-256(manifest_bytes)` per Ed25519 spec.
   - Returns signature bytes (64 bytes).
   - Persists `manifest_signature_kms_blob` on bundle row.
   - Emits `ten.bundle_signature_generated`.

10. **MUST** anchor in memory per DEC-1325. After build completes:
    - INSERT memory row `ten.bundle_built` with payload containing `bundle_sha256` + `manifest_sha256` + `tenant_id` + `file_count`.
    - This row's chain hash becomes the anchor for TASK-OBS-009 chain-of-custody per DEC-1333.

11. **MUST** upload to S3 with 30-day signed URL per DEC-1327. Handler:
    - Uploads to `cyberos-bundles/{tenant_id}/{bundle_id}/bundle.zip` per residency S3.
    - Generates presigned URL with 30-day TTL.
    - Status → 'ready'; expires_at set.
    - Emit `ten.bundle_built` sev-1.

12. **MUST** deliver via `GET /v1/admin/tenants/{tid}/bundle/{bundle_id}/download` returning the signed URL (URL itself not stored client-side; fetched per request). On first delivery: status → 'delivered'; emit `ten.bundle_delivered`.

13. **MUST** rate-limit per DEC-1329 — 3 bundle requests per tenant per 365 days. Excess → 429 + `bundle_rate_limit_exceeded`.

14. **MUST** enforce pre-deletion gate per DEC-1330. TASK-TEN-106 attestation handler checks:
    - SELECT bundle WHERE tenant_id=$1 AND status IN ('ready','delivered') AND ready_at > now() - interval '90 days'.
    - No row → 412 PRECONDITION_FAILED + `bundle_export_required_before_attestation`.

15. **MUST** expire bundle at T+30d per DEC-1327. Scheduled job:
    - status='expired' + bundle s3 deleted + signed URL invalid.
    - Emit `ten.bundle_expired`.

16. **MUST** emit 6 memory audit kinds per DEC-1332: export_initiated (sev-1), built (sev-1), delivered (sev-1), failed (sev-1), expired (sev-2), signature_generated (sev-2). All sev-1 except expired (sev-2 ops event) + signature_generated (sev-2 internal).

17. **MUST** PII-scrub via TASK-MEMORY-111 — `reason` text hashed; tenant_id retained for chain correlation.

18. **MUST** thread trace_id across request → task → build → S3 upload → audit.

19. **MUST** support manifest verification at `GET /v1/admin/tenants/{tid}/bundle/{bundle_id}/verify` returning `{ manifest_sha256, signature, public_key_pem, verifier_instructions_url }` — operator OR external auditor can verify offline.

20. **MUST NOT** sign bundle with shared key (per DEC-1322 — per-tenant only).

21. **MUST NOT** allow non-deterministic ZIP entries (per DEC-1321) — two builds MUST produce byte-identical output.

22. **MUST NOT** permanent-delete tenant without valid recent bundle (per DEC-1330 gate enforced in TASK-TEN-106).

---

## §2 — Why this design (rationale)

**Why per-tenant Ed25519 keys (§1 #3-4, DEC-1322)?** Single global key = single compromise vector. Per-tenant key compromise scopes to one tenant. Standard cryptographic hygiene.

**Why deterministic ZIP (§1 #8, DEC-1321)?** Auditor reproducibility — "did the bundle change between my download and yours?" Deterministic = answer is yes (different hash) or no (same hash) with certainty.

**Why include memory chain segment (§1 #7, DEC-1331)?** The audit chain IS the legal evidence. Customer departing on bad terms → CyberSkill needs to prove every action. Chain segment with signed bundle = unforgeable.

**Why 3-per-year rate limit (§1 #13, DEC-1329)?** Bundle building is expensive (cross-source aggregation, S3 upload, GBs of data). Operationally tenant requests bundle once at termination + maybe annual compliance archive + once-in-a-while audit support = 3.

**Why 30-day TTL (§1 #11, DEC-1327)?** Legal evidence may need multi-week review by counsel; 7d (DSAR) too short. 30d covers typical legal cycles.

**Why pre-deletion gate (§1 #14, DEC-1330)?** Permanent-delete is irreversible. Requiring recent bundle = "we always have a recovery path within the past 90 days" — operational safety net.

---

## §3 — API contract

```sql
-- 0026_tenant_bundle_exports.sql
CREATE TYPE bundle_status AS ENUM ('pending','building','ready','delivered','expired','failed');

CREATE TABLE tenant_bundle_exports (
  bundle_id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  status bundle_status NOT NULL DEFAULT 'pending',
  requested_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  requested_by_subject_id UUID NOT NULL,
  reason TEXT,
  build_started_at TIMESTAMPTZ,
  ready_at TIMESTAMPTZ,
  delivered_at TIMESTAMPTZ,
  expires_at TIMESTAMPTZ,
  s3_key TEXT,
  bundle_size_bytes BIGINT,
  file_count INT,
  manifest_sha256 CHAR(64),
  manifest_signature_kms_blob BYTEA,
  source_chain_head BIGINT,
  trace_id CHAR(32)
);
CREATE INDEX idx_bundle_tenant_year ON tenant_bundle_exports(tenant_id, requested_at DESC);
ALTER TABLE tenant_bundle_exports ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_bundle_exports_rls ON tenant_bundle_exports
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON tenant_bundle_exports FROM cyberos_app;
GRANT UPDATE (status, build_started_at, ready_at, delivered_at, expires_at,
              s3_key, bundle_size_bytes, file_count, manifest_sha256,
              manifest_signature_kms_blob, source_chain_head) ON tenant_bundle_exports TO cyberos_app;

-- 0027_tenant_signing_keys.sql
CREATE TABLE tenant_signing_keys (
  tenant_id UUID PRIMARY KEY,
  public_key_pem TEXT NOT NULL,
  private_key_kms_blob BYTEA NOT NULL,
  kms_key_id TEXT NOT NULL,
  generated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
ALTER TABLE tenant_signing_keys ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_signing_keys_rls ON tenant_signing_keys
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON tenant_signing_keys FROM cyberos_app;
```

Endpoints:
```text
POST   /v1/admin/tenants/{tid}/bundle/export                  (tenant_admin or cso)
GET    /v1/admin/tenants/{tid}/bundle                          (list bundles for tenant)
GET    /v1/admin/tenants/{tid}/bundle/{bundle_id}/download    (signed URL retrieval)
GET    /v1/admin/tenants/{tid}/bundle/{bundle_id}/verify      (public key + signature for verification)
```

---

## §4 — Acceptance criteria

1. **bundle_status cardinality 6**.
2. **Keypair generated at provisioning** — every new tenant has `tenant_signing_keys` row.
3. **Bundle build async** — request returns 202 + bundle_id; task processes; status transitions pending→building→ready.
4. **Deterministic zip** — two builds of same source data → byte-identical ZIP.
5. **Manifest signed** — Ed25519 signature on manifest_sha256; verifies with public key.
6. **memory chain anchor** — `ten.bundle_built` row contains bundle_sha256 + manifest_sha256.
7. **30-day signed URL** — URL valid for 30d; T+31d → 403.
8. **Pre-deletion gate** — TASK-TEN-106 attestation without recent bundle → 412.
9. **3-per-year rate limit** — 4th request in 12 months → 429.
10. **Audit chain in bundle** — bundle's `audit_chain.json` contains all rows for tenant.
11. **Per-source files present** — bundle contains projects.json, invoices.json, documents.json, chat.json, audit_chain.json, metering.json, users.json + .csv variants.
12. **Manifest verification endpoint** — returns public_key_pem + signature.
13. **Bundle expiry T+30d** — scheduled job marks expired + deletes S3 object.
14. **6 memory audit kinds emitted** — full lifecycle.
15. **Signed manifest matches files** — recompute SHA-256 of each file → matches manifest entries.
16. **Cross-tenant bundle invisible** — RLS prevents tenant A from seeing tenant B's bundles.
17. **Per-tenant private key not exfiltrable** — KMS-encrypted; raw private key never in DB.
18. **Bundle build failure** — fixture broken source → status='failed' + audit + retry option.
19. **Trace_id end-to-end** — request → task → audit all share trace_id.
20. **CSV-JSON parity** — every JSON row also appears in CSV (1:1 mapping).

---

## §5 — Verification

```rust
#[tokio::test]
async fn bundle_is_deterministic() {
    let ctx = TestContext::with_seeded_tenant().await;
    let b1 = ctx.request_bundle().await;
    ctx.wait_ready(b1).await;
    let bytes1 = ctx.download_bundle(b1).await;

    // Re-build same tenant data
    let b2 = ctx.request_bundle().await;
    ctx.wait_ready(b2).await;
    let bytes2 = ctx.download_bundle(b2).await;

    assert_eq!(sha256(&bytes1), sha256(&bytes2), "bundles differ");
}

#[tokio::test]
async fn manifest_signature_verifies() {
    let ctx = TestContext::with_seeded_tenant().await;
    let bundle_id = ctx.request_and_wait_bundle().await;
    let verify_resp = ctx.get_verify_info(bundle_id).await;
    let public_key = verify_resp["public_key_pem"].as_str().unwrap();
    let signature = base64::decode(verify_resp["signature"].as_str().unwrap()).unwrap();
    let manifest_sha256 = verify_resp["manifest_sha256"].as_str().unwrap();

    let pk = ed25519_dalek::PublicKey::from_pem(public_key).unwrap();
    let sig = ed25519_dalek::Signature::from_bytes(&signature).unwrap();
    let msg = hex::decode(manifest_sha256).unwrap();
    assert!(pk.verify(&msg, &sig).is_ok());
}

#[tokio::test]
async fn pre_deletion_gate_blocks_attestation_without_bundle() {
    let ctx = TestContext::new().await;
    let tid = ctx.provision_tenant().await;
    // No bundle exported
    let r = ctx.post_ten_106_attestation(tid, "test").await;
    assert_eq!(r.status(), 412);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["error"], "bundle_export_required_before_attestation");
}

#[tokio::test]
async fn three_per_year_rate_limit() {
    let ctx = TestContext::new().await;
    for _ in 0..3 { ctx.request_bundle().await; }
    let r = ctx.request_bundle_raw().await;
    assert_eq!(r.status(), 429);
}

#[tokio::test]
async fn memory_chain_anchored() {
    let ctx = TestContext::with_seeded_tenant().await;
    let b = ctx.request_and_wait_bundle().await;
    let audit = ctx.memory_rows().await;
    let built = audit.iter().find(|r| r.kind == "ten.bundle_built").unwrap();
    assert!(built.payload["bundle_sha256"].is_string());
    assert!(built.payload["manifest_sha256"].is_string());
}

// 5.6 30d expiry
// 5.7 cross-tenant invisible
// 5.8 audit chain completeness
// 5.9 csv-json parity
// 5.10 audit kinds emitted
```

---

## §7 — Dependencies

**Upstream:** TASK-TEN-104 (lifecycle — bundle as part of termination flow).
**Cross-module:** TASK-TEN-106 (pre-deletion gate), TASK-AUTH-101 (cso + tenant_admin roles), TASK-MEMORY-101 (audit chain source), TASK-DOC-001 (S3 storage), TASK-MCP-007 (async build via Tasks), TASK-AI-003 (audit kinds), TASK-MEMORY-111 (PII scrub), TASK-OBS-007 (sev-1 routing), TASK-OBS-009 (chain-of-custody anchor).
**Downstream:** None.

---

## §8 — Example payloads

`ten.bundle_built`:
```json
{
  "kind": "ten.bundle_built",
  "severity": 1,
  "tenant_id": "8a2f...",
  "actor_id": "system.ten.bundle",
  "trace_id": "0af76519...",
  "occurred_at": "2026-05-17T...",
  "payload": {
    "bundle_id": "0190...",
    "bundle_sha256": "9c4e7a8b...",
    "manifest_sha256": "f8a1b2c3...",
    "file_count": 12,
    "bundle_size_bytes": 487293104,
    "source_chain_head": 348201,
    "expires_at": "2026-06-16T..."
  }
}
```

Manifest format:
```json
{
  "version": 1,
  "tenant_id": "8a2f...",
  "exported_at": "2026-05-17T09:14:32.847Z",
  "source_chain_head": 348201,
  "file_count": 12,
  "files": [
    { "path": "audit_chain.json", "sha256": "...", "size_bytes": 2837492 },
    { "path": "audit_chain.csv", "sha256": "...", "size_bytes": 1928374 },
    { "path": "projects.json", "sha256": "...", "size_bytes": 84729 },
    { "path": "projects.csv", "sha256": "...", "size_bytes": 41928 }
  ],
  "signature_algorithm": "Ed25519",
  "public_key_pem": "-----BEGIN PUBLIC KEY-----...-----END PUBLIC KEY-----"
}
```

---

## §9 — Open questions

Deferred:
- **Deferred:** Multi-region bundle replication (slice 3).
- **Deferred:** Cold-storage archive for long-term retention (slice 3).
- **Deferred:** Bundle GPG-signing variant for paranoid customers (slice 3).
- **Deferred:** Differential exports (only changes since last bundle) — slice 3.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Build task fails | TASK-MCP-007 error | status='failed' + sev-1 audit | Operator retries |
| KMS unavailable for signing | KMS error | Bundle status='failed'; sev-1 | KMS recovery |
| S3 upload fails | API error | Status='failed'; retry path | Inherent |
| Non-deterministic zip detected | hash mismatch on test | CI fails the deploy | Fix ZIP library config |
| Pre-deletion gate triggered with old bundle (> 90d) | check fails | 412; operator requests new bundle | Re-request |
| 3-per-year hit | counter | 429 | Operator escalates if legitimate need |
| Bundle exceeds 50 GB | size check | sev-2; build proceeds but slow | Future: split into multi-volume |
| Manifest signature corruption | verify endpoint | sev-1; bundle marked suspect | Re-issue bundle |
| Cross-tenant bundle leak via misconfigured RLS | RLS test | Periodic check | Inherent |
| Bundle ID collision (astronomical) | UUID partial unique | Inherent | None needed |
| Public key drift (regenerated after compromise) | key rotation table | Old bundles use old key; new use new key | Public key archive |
| Tenant data deleted before bundle build | source-empty bundle | Bundle empty; audit row notes | Pre-bundle data freeze (slice 3) |
| Audit chain segment > 1M rows | size check | Bundle build slow; sev-3 log | Async pagination in builder |
| Build interrupted by deploy | task retry via TASK-MCP-007 checkpoint | Resumes from last checkpoint | Inherent |
| Signed URL leaked via screen-share | inherent | 30d window for download; passphrase optional add-on | Inherent UX warning |
| Cross-residency tenant export | residency-router consumed | Bundle stored in tenant's residency S3 | Inherent |

---

## §11 — Implementation notes

**§11.1** `ed25519-dalek` crate for keygen + sign + verify.

**§11.2** Deterministic ZIP via `zip` crate with explicit ZIP64 disable + fixed timestamp + sorted entries.

**§11.3** Manifest canonical JSON via sorted keys.

**§11.4** Per-tenant Ed25519 key never re-generated (would invalidate prior signatures); rotation = new keypair stored alongside old; old bundles verify with old key.

**§11.5** S3 lifecycle policy auto-deletes bundle objects at T+30d (defense-in-depth vs application cleanup).

**§11.6** Build task uses TASK-MCP-007 worker pool; checkpoints per-source so crash mid-build resumes.

**§11.7** README.md in bundle includes verification instructions in English + Vietnamese.

**§11.8** Audit chain segment serialised as JSONL (one row per line) for streaming-friendly format.

**§11.9** Per-tenant residency: bundle stored in tenant's residency S3 per TASK-TEN-103 trip-wire.

**§11.10** Chain-of-custody anchor at TASK-OBS-009 cross-system; future-proofs auditor questions.

---

*End of TASK-TEN-105 spec.*
