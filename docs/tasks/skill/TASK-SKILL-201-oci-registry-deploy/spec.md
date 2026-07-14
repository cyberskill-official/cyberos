---
id: TASK-SKILL-201
title: "SKILL OCI registry deploy for `.skill` bundles — R3 distribution stage with signed bundles + tag immutability + tenant-scoped pulls"
module: SKILL
priority: MUST
status: done
verify: T
phase: P3
milestone: P3 · slice 1
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-SKILL-102, TASK-AUTH-105, TASK-MEMORY-111]
depends_on: [TASK-SKILL-102]
blocks: []

source_pages:
  - website/docs/modules/skill.html#oci-registry
  - https://specs.opencontainers.org/distribution-spec/

source_decisions:
  - DEC-2420 2026-05-17 — `.skill` bundles distributed via OCI registry (e.g. ghcr.io, ECR); R3 = registry stage of R1-publish, R2-package, R3-distribute pipeline
  - DEC-2421 2026-05-17 — Closed enum `bundle_status` = {pushed, signed, validated, available, deprecated, yanked}; cardinality 6
  - DEC-2422 2026-05-17 — Bundles signed via cosign with tenant-specific key from TASK-AUTH-105 KMS; signature verified at pull time
  - DEC-2423 2026-05-17 — Tag IMMUTABLE post-push (cannot overwrite); new version = new tag (semver); yanked status hides from default search but preserves availability
  - DEC-2424 2026-05-17 — Tenant-scoped pulls: bundle visibility per `tenant_acl` jsonb; pulls verify ACL match
  - DEC-2425 2026-05-17 — memory audit kinds: skill.bundle_pushed, skill.bundle_signed, skill.bundle_pulled, skill.bundle_yanked, skill.bundle_signature_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/skill/
  new_files:
    - services/skill/migrations/0010_oci_bundles.sql
    - services/skill/src/oci/mod.rs
    - services/skill/src/oci/push.rs
    - services/skill/src/oci/pull.rs
    - services/skill/src/oci/cosign_wrapper.rs
    - services/skill/src/oci/acl_check.rs
    - services/skill/src/handlers/oci_routes.rs
    - services/skill/src/audit/oci_events.rs
    - services/skill/tests/bundle_status_enum_cardinality_test.rs
    - services/skill/tests/oci_push_signed_test.rs
    - services/skill/tests/oci_tag_immutability_test.rs
    - services/skill/tests/oci_pull_acl_test.rs
    - services/skill/tests/oci_yank_test.rs
    - services/skill/tests/oci_audit_emission_test.rs

  modified_files:
    - services/skill/src/lib.rs

  allowed_tools:
    - file_read: services/{skill,auth}/**
    - file_write: services/skill/{src,tests,migrations}/**
    - bash: cd services/skill && cargo test oci

  disallowed_tools:
    - overwrite existing tag (per DEC-2423)
    - pull without signature verify (per DEC-2422)
    - pull bypassing ACL (per DEC-2424)

effort_hours: 8
subtasks:
  - "0.3h: 0010_oci_bundles.sql"
  - "0.4h: oci/mod.rs"
  - "0.7h: push.rs"
  - "0.7h: pull.rs"
  - "0.6h: cosign_wrapper.rs"
  - "0.5h: acl_check.rs"
  - "0.4h: handlers/oci_routes.rs"
  - "0.3h: audit/oci_events.rs"
  - "3.0h: tests — 6 test files"
  - "1.1h: docs"

risk_if_skipped: "Without OCI registry, .skill distribution ad-hoc (file shares, email). Without DEC-2422 cosign, unsigned bundles = supply chain attack vector. Without DEC-2423 tag immutability, version drift undetectable. Without DEC-2424 ACL, public bundle leaks."
---

## §1 — Description (BCP-14 normative)

The SKILL service **MUST** ship OCI registry deploy at `services/skill/src/oci/` with push/pull + cosign signing + tag immutability + tenant ACL, 5 memory audit kinds.

1. **MUST** validate `bundle_status` against closed enum per DEC-2421.

2. **MUST** push at `push.rs::push(bundle_path, registry, tag, tenant_acl)` per DEC-2420:
   - Validate `.skill` bundle (SKILL.md frontmatter, allowed_tools list)
   - Push to OCI registry via OCI distribution spec API
   - Sign via `cosign_wrapper.rs::sign(digest, tenant_kms_key)`
   - Record in DB with tenant_acl

3. **MUST** enforce tag immutability per DEC-2423 — pre-push: check registry for existing tag; reject if exists.

4. **MUST** verify cosign signature on pull at `pull.rs::pull(registry, tag)` per DEC-2422 — `cosign_wrapper.rs::verify(bundle, signature)` against registered key; reject + sev-1 audit on mismatch.

5. **MUST** check tenant ACL per DEC-2424 at `acl_check.rs::can_pull(bundle, requester_tenant)` — match against bundle.tenant_acl JSONB.

6. **MUST** support yank per DEC-2423 — sets status=yanked; pull returns warning but bundle still retrievable for audit.

7. **MUST** define table at migration `0010`:
   ```sql
   CREATE TABLE skill_oci_bundles (
     bundle_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     registry TEXT NOT NULL,
     image_name TEXT NOT NULL,
     tag TEXT NOT NULL,
     digest TEXT NOT NULL,
     cosign_signature_bytes BYTEA,
     tenant_acl JSONB NOT NULL,  -- {"allowed_tenants": ["uuid1", "uuid2"], "public": false}
     status TEXT NOT NULL DEFAULT 'pushed'
       CHECK (status IN ('pushed','signed','validated','available','deprecated','yanked')),
     pushed_by UUID NOT NULL,
     yanked_at TIMESTAMPTZ,
     yanked_reason TEXT,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     UNIQUE (registry, image_name, tag)
   );
   ALTER TABLE skill_oci_bundles ENABLE ROW LEVEL SECURITY;
   CREATE POLICY bundles_rls ON skill_oci_bundles
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON skill_oci_bundles FROM cyberos_app;
   GRANT UPDATE (status, yanked_at, yanked_reason) ON skill_oci_bundles TO cyberos_app;
   ```

8. **MUST** expose endpoints:
   ```text
   POST /v1/skill/oci/push          body: {bundle_path, registry, tag, tenant_acl}
   POST /v1/skill/oci/pull          body: {registry, image_name, tag}
   POST /v1/skill/oci/yank/{id}     body: {reason}
   GET  /v1/skill/oci/bundles       (list with ACL filter)
   ```

9. **MUST** emit 5 memory audit kinds per DEC-2425. PII per TASK-MEMORY-111: digest + signature_bytes SHA256.

10. **MUST** thread trace_id from push/pull → audit.

11. **MUST NOT** overwrite existing tag per DEC-2423 (UNIQUE constraint enforces).

12. **MUST NOT** pull without signature verify per DEC-2422.

13. **MUST NOT** allow cross-ACL pull per DEC-2424.

---

## §2 — Why this design

**Why OCI (DEC-2420)?** Industry standard; GHCR/ECR/etc. interoperable; cosign signing supports it natively.

**Why cosign (DEC-2422)?** Supply chain integrity; CNCF-graduated; widely audited.

**Why tag immutability (DEC-2423)?** Reproducibility — same tag must always pull same bytes; otherwise downstream caching breaks.

**Why ACL (DEC-2424)?** Multi-tenant SaaS — each tenant's custom skills shouldn't leak across.

---

## §3 — API contract

Sample push:
```json
POST /v1/skill/oci/push
{
  "bundle_path": "./my-skill.skill",
  "registry": "ghcr.io/cyberskill",
  "tag": "calendar-list@1.2.0",
  "tenant_acl": {"allowed_tenants": ["uuid-1"], "public": false}
}
```

---

## §4 — Acceptance criteria
1. **bundle_status enum cardinality 6**. 2. **Push to OCI works**. 3. **Cosign signature applied**. 4. **Tag overwrite rejected (UNIQUE)**. 5. **Pull verifies signature**. 6. **Bad signature → sev-1 + reject**. 7. **ACL enforced on pull**. 8. **Yank sets status, doesn't delete**. 9. **5 memory audit kinds emitted**. 10. **PII scrubbed (digest+sig SHA256)**. 11. **RLS denies cross-tenant**. 12. **CTO-only push**. 13. **Trace_id preserved**. 14. **Append-only via REVOKE except status cols**. 15. **Cosign key from TASK-AUTH-105**. 16. **public:true bundles pullable by anyone**. 17. **public:false requires ACL match**. 18. **List endpoint filters by ACL**. 19. **OCI spec compliance**. 20. **Bundle validation pre-push**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn tag_immutability() {
    let ctx = TestContext::with_pushed_bundle().await;
    let r = ctx.try_push_same_tag().await;
    assert!(r.is_err());
}

#[tokio::test]
async fn signature_verify_on_pull() {
    let ctx = TestContext::with_signed_bundle().await;
    let r = ctx.pull(ctx.registry, ctx.tag).await;
    assert!(r.is_ok());
    ctx.tamper_bundle(ctx.bundle_id).await;
    let r2 = ctx.try_pull(ctx.registry, ctx.tag).await;
    assert!(r2.is_err());
}

#[tokio::test]
async fn acl_blocks_cross_tenant() {
    let ctx = TestContext::with_private_bundle_for_tenant_a().await;
    let r = ctx.try_pull_as_tenant_b(ctx.bundle_id).await;
    assert_eq!(r.status_code, 403);
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-SKILL-102.
**Cross-module:** TASK-AUTH-105 (KMS for cosign), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Registry unreachable | retry | sev-2 | retry |
| Tag overwrite | UNIQUE | 409 | new tag |
| Signature mismatch | verify | reject + sev-1 | investigate |
| ACL miss | gate | 403 | get permission |
| Cosign key expired | KMS | sev-1 | rotate |
| Bundle malformed | pre-push validate | 400 | fix bundle |
| Network partition pull | retry | sev-2 | inherent |
| Yank of in-use bundle | warn but allow | inherent | inherent |
| Cross-tenant push | RLS | inherent | inherent |
| Quota exceeded on registry | sev-2 | inherent | upgrade |

## §11 — Implementation notes
- §11.1 OCI client: `oci-distribution` Rust crate; supports GHCR + ECR + Quay.
- §11.2 Cosign integration: shell out to `cosign` binary or use `cosign-rs` crate.
- §11.3 memory audit body: bundle_id, registry, tag; digest + signature SHA256.
- §11.4 Tenant ACL stored as JSONB; tomorrow extensible to org-scoped + role-scoped.
- §11.5 Yank semantics: status=yanked + warning on pull; bundle still retrievable for replay.

---

*End of TASK-SKILL-201 spec.*
