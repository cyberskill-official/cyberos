---
id: TASK-KB-003
title: "KB 3 permission tiers — public / org-only / role-restricted with share-link tokens for time-bounded external access"
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
module: KB
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 4
slice: 4
owner: Stephen Cheng (CDO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-KB-001, TASK-AUTH-101, TASK-MEMORY-111]
depends_on: [TASK-KB-001]
blocks: [TASK-KB-004]

source_pages:
  - website/docs/modules/kb.html#permissions

source_decisions:
  - DEC-1900 2026-05-17 — 3 visibility tiers: public (unauth OK), org_only (any tenant member), role_restricted (specific roles via TASK-AUTH-101)
  - DEC-1901 2026-05-17 — Closed enum `visibility_tier` = {public, org_only, role_restricted}; cardinality 3
  - DEC-1902 2026-05-17 — Share-link tokens: signed JWT with doc_id + expires_at + max_uses; 24h default expiry; CDO-configurable
  - DEC-1903 2026-05-17 — Public docs may be served to unauthenticated browsers; org_only requires session; role_restricted requires role match
  - DEC-1904 2026-05-17 — memory audit kinds: kb.permission_set, kb.share_link_created, kb.share_link_used, kb.access_denied

build_envelope:
  language: rust 1.81
  service: cyberos/services/kb/
  new_files:
    - services/kb/migrations/0003_permissions_share_links.sql
    - services/kb/src/permission/mod.rs
    - services/kb/src/permission/access_gate.rs
    - services/kb/src/permission/share_link.rs
    - services/kb/src/handlers/permission_routes.rs
    - services/kb/src/audit/permission_events.rs
    - services/kb/tests/permission_tier_enum_cardinality_test.rs
    - services/kb/tests/permission_public_unauth_ok_test.rs
    - services/kb/tests/permission_org_only_session_required_test.rs
    - services/kb/tests/permission_role_restricted_test.rs
    - services/kb/tests/share_link_expiry_test.rs
    - services/kb/tests/share_link_max_uses_test.rs
    - services/kb/tests/permission_audit_emission_test.rs

  modified_files:
    - services/kb/src/lib.rs

  allowed_tools:
    - file_read: services/{kb,auth}/**
    - file_write: services/kb/{src,tests,migrations}/**
    - bash: cd services/kb && cargo test permission

  disallowed_tools:
    - serve role_restricted to wrong role (per DEC-1903)
    - allow expired share-link (per DEC-1902)

effort_hours: 5
subtasks:
  - "0.3h: 0003_permissions_share_links.sql"
  - "0.3h: permission/mod.rs"
  - "0.6h: access_gate.rs"
  - "0.5h: share_link.rs"
  - "0.4h: handlers/permission_routes.rs"
  - "0.3h: audit/permission_events.rs"
  - "2.3h: tests — 7 test files"
  - "0.3h: docs"

risk_if_skipped: "Without tier enforcement, internal docs leak to public. Without DEC-1902 share-link expiry, leaked links persist indefinitely. Without DEC-1903 role match, junior staff sees executive docs."
---

## §1 — Description (BCP-14 normative)

The KB service **MUST** ship 3-tier permission system at `services/kb/src/permission/` with share-link tokens, 4 memory audit kinds.

1. **MUST** validate `visibility_tier` against closed enum per DEC-1901.

2. **MUST** gate access at `access_gate.rs::check(doc, user, share_token?)`:
   - public: always allow
   - org_only: require valid session matching doc.tenant_id
   - role_restricted: require user has one of doc.allowed_roles
   - share_token (if provided): verify token signature + expiry + max_uses

3. **MUST** define table extension + share-link table at migration `0003`:
   ```sql
   ALTER TABLE kb_documents ADD COLUMN visibility_tier TEXT NOT NULL DEFAULT 'org_only'
     CHECK (visibility_tier IN ('public','org_only','role_restricted'));
   ALTER TABLE kb_documents ADD COLUMN allowed_roles TEXT[] NOT NULL DEFAULT '{}';
   CREATE INDEX docs_visibility_idx ON kb_documents(tenant_id, visibility_tier);
   GRANT UPDATE (visibility_tier, allowed_roles) ON kb_documents TO cyberos_app;

   CREATE TABLE kb_share_links (
     token_jti UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     doc_id UUID NOT NULL,
     created_by UUID NOT NULL,
     expires_at TIMESTAMPTZ NOT NULL,
     max_uses INT NOT NULL DEFAULT 0,  -- 0 = unlimited
     used_count INT NOT NULL DEFAULT 0,
     revoked_at TIMESTAMPTZ,
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   CREATE INDEX share_links_doc_idx ON kb_share_links(tenant_id, doc_id);
   ALTER TABLE kb_share_links ENABLE ROW LEVEL SECURITY;
   CREATE POLICY share_links_rls ON kb_share_links
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON kb_share_links FROM cyberos_app;
   GRANT UPDATE (used_count, revoked_at) ON kb_share_links TO cyberos_app;
   ```

4. **MUST** create share-links at `share_link.rs::create(doc_id, expires_in, max_uses?)` per DEC-1902 — CDO-only.

5. **MUST** validate share-link on use per DEC-1902:
   - Signature valid
   - expires_at > now()
   - max_uses=0 OR used_count < max_uses
   - revoked_at IS NULL
   - On success: increment used_count atomically.

6. **MUST** expose endpoints:
   ```text
   PUT    /v1/kb/docs/{id}/visibility         body: {tier, allowed_roles?}
   POST   /v1/kb/docs/{id}/share-links        body: {expires_in_seconds, max_uses?}
   POST   /v1/kb/share-links/{jti}/revoke
   GET    /v1/kb/docs/{id}?share_token=...    (share-link access path)
   ```

7. **MUST** emit 4 memory audit kinds per DEC-1904. PII per TASK-MEMORY-111: tier+role enums ok; share_token only jti in chain.

8. **MUST** thread trace_id from gate → audit.

9. **MUST NOT** serve role_restricted to non-matching role per DEC-1903.

10. **MUST NOT** accept expired share-link per DEC-1902.

11. **MUST NOT** accept share-link past max_uses per DEC-1902.

---

## §2 — Why this design

**Why 3 tiers (DEC-1900)?** Industry-standard public/private/role split; covers 95% of KB use cases.

**Why share-links (DEC-1902)?** Customer collaboration needs external read access without account creation; tokens provide time-bounded grant.

**Why max_uses (DEC-1902)?** Single-use links (max_uses=1) prevent forwarding; mass-share allows broader distribution.

**Why role match (DEC-1903)?** Executive comp docs need ROOT-CHRO scope; engineering runbooks need engineer scope.

---

## §3 — API contract

```text
PUT    /v1/kb/docs/{id}/visibility
POST   /v1/kb/docs/{id}/share-links
POST   /v1/kb/share-links/{jti}/revoke
GET    /v1/kb/docs/{id}?share_token=<jwt>
```

Sample share-link creation:
```json
{
  "expires_in_seconds": 86400,
  "max_uses": 5
}
```

Response:
```json
{
  "token_jti": "uuid",
  "share_url": "https://kb.cyberskill.com/d/abc123?share_token=eyJhbGc...",
  "expires_at": "2026-05-18T10:00:00Z",
  "max_uses": 5
}
```

---

## §4 — Acceptance criteria
1. **visibility_tier enum cardinality 3**. 2. **public served unauth**. 3. **org_only requires session**. 4. **role_restricted requires role match**. 5. **Share-link JWT signed + verified**. 6. **expires_at enforced**. 7. **max_uses enforced (0=unlimited)**. 8. **Revoke endpoint works**. 9. **used_count incremented atomically**. 10. **4 memory audit kinds emitted**. 11. **Access denied → audit**. 12. **RLS denies cross-tenant**. 13. **CDO-only create + revoke**. 14. **Trace_id preserved**. 15. **Append-only share_links via REVOKE except used_count + revoked_at**. 16. **default tier = org_only on new doc**. 17. **Multiple share-links per doc supported**. 18. **Share-link survives doc version updates**. 19. **Revoked share-link cannot be undone**. 20. **token_jti UUID prevents collision**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn public_doc_unauth_serve() {
    let ctx = TestContext::with_public_doc().await;
    let r = ctx.fetch_doc_unauth(ctx.doc_id).await;
    assert_eq!(r.status_code, 200);
}

#[tokio::test]
async fn role_restricted_denies_wrong_role() {
    let ctx = TestContext::with_role_restricted_doc(vec!["ROOT-CHRO"]).await;
    let r = ctx.fetch_doc_as(ctx.am_user, ctx.doc_id).await;
    assert_eq!(r.status_code, 403);
}

#[tokio::test]
async fn share_link_expiry_enforced() {
    let ctx = TestContext::with_share_link_expires_in(1).await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    let r = ctx.fetch_via_share_token(ctx.token).await;
    assert_eq!(r.status_code, 403);
}

#[tokio::test]
async fn share_link_max_uses_enforced() {
    let ctx = TestContext::with_share_link_max_uses(3).await;
    for _ in 0..3 { assert!(ctx.use_share_token().await.is_ok()); }
    let r = ctx.use_share_token().await;
    assert!(r.is_err());
}

// 5.5..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-KB-001.
**Cross-module:** TASK-AUTH-101 (role check), TASK-MEMORY-111 (audit), TASK-AUTH-105 (KMS for JWT signing key).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Invalid JWT signature | verify | 401 + sev-2 | inherent |
| Token expired | timestamp | 403 | new link |
| Max uses reached | check + increment | 403 | new link |
| Revoked share-link use | flag | 403 | new link |
| Role mismatch | check | 403 + sev-3 | request access |
| Public doc on non-public service | guard | inherent | inherent |
| Cross-tenant share-link | RLS | 0 rows | inherent |
| Atomic increment race | row-level lock | inherent | inherent |
| JWT signing key rotation | dual-key window | inherent | inherent |
| Share-link to deleted doc | check | 404 | inherent |

## §11 — Implementation notes
- §11.1 JWT signed with tenant-specific key via TASK-AUTH-105 KMS.
- §11.2 Access gate is pure function: `(doc, user, token) → Allow | Deny(reason)`.
- §11.3 used_count increment: `UPDATE ... SET used_count = used_count + 1 WHERE jti=$1 AND used_count < max_uses RETURNING used_count`.
- §11.4 memory audit body: doc_id, tier, decision; token jti only (no token value).
- §11.5 Default tier on new doc = org_only (least-surprise for accidental publish risk).

---

*End of TASK-KB-003 spec.*
