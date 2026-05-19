---
id: FR-EMAIL-002
title: "EMAIL Stalwart authbridge plugin — JMAP/IMAP/SMTP auth delegates to AUTH JWT validation + per-tenant mailbox scoping"
module: EMAIL
priority: MUST
status: draft
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-EMAIL-001, FR-EMAIL-003, FR-EMAIL-004, FR-AUTH-004, FR-AUTH-101, FR-AI-003, FR-MEMORY-111]
depends_on: [FR-EMAIL-001, FR-AUTH-004]
blocks: []

source_pages:
  - website/docs/modules/email.html#authbridge
  - https://stalw.art/docs/server/authentication

source_decisions:
  - DEC-1460 2026-05-17 — Stalwart calls our authbridge plugin via HTTP-based auth backend; we validate JWT + return mailbox/permission map; Stalwart enforces per-mailbox scoping
  - DEC-1461 2026-05-17 — JWT-as-password pattern: client supplies JWT as IMAP/SMTP password field; authbridge validates against FR-AUTH-004 issuer; rejects expired/wrong-audience
  - DEC-1462 2026-05-17 — Per-tenant mailbox naming: `<subject_id>@<tenant_slug>.cyberos.world`; per-tenant CNAME (FR-PORTAL-002) MAY alias to vanity domain
  - DEC-1463 2026-05-17 — Closed enum `auth_outcome` = {success, jwt_invalid, jwt_expired, jwt_wrong_audience, subject_revoked, mailbox_unauthorized}; cardinality 6
  - DEC-1464 2026-05-17 — Token caching: validated JWT cached 60s in Redis (avoid per-request JWT verify on IMAP idle); cache key = (jti, subject_id)
  - DEC-1465 2026-05-17 — FR-PORTAL-004 SCIM deprovision invalidates Redis cache + Stalwart connection rejected on next operation
  - DEC-1466 2026-05-17 — memory audit kinds: email.auth_success, email.auth_failed, email.mailbox_accessed, email.smtp_send_authorized

build_envelope:
  language: rust 1.81
  service: cyberos/services/email/
  new_files:
    - services/email/migrations/0001_email_auth_log.sql
    - services/email/src/authbridge/mod.rs
    - services/email/src/authbridge/jwt_validator.rs
    - services/email/src/authbridge/mailbox_resolver.rs
    - services/email/src/authbridge/cache.rs
    - services/email/src/authbridge/revocation_consumer.rs
    - services/email/src/audit/auth_events.rs
    - services/email/src/handlers/authbridge_routes.rs
    - services/email/tests/auth_jwt_valid_test.rs
    - services/email/tests/auth_jwt_expired_test.rs
    - services/email/tests/auth_wrong_audience_test.rs
    - services/email/tests/auth_cache_60s_test.rs
    - services/email/tests/auth_scim_revoke_invalidates_test.rs
    - services/email/tests/auth_outcome_enum_test.rs
    - services/email/tests/auth_per_tenant_mailbox_test.rs
    - services/email/tests/auth_audit_emission_test.rs

  modified_files:
    - services/email/src/lib.rs

  allowed_tools:
    - file_read: services/{email,auth}/**
    - file_write: services/email/{src,tests,migrations}/**
    - bash: cd services/email && cargo test authbridge

  disallowed_tools:
    - cache JWTs > 60s (per DEC-1464 — security boundary)
    - allow cross-tenant mailbox access (per DEC-1462)
    - bypass FR-PORTAL-004 SCIM revoke cascade (per DEC-1465)

effort_hours: 6
sub_tasks:
  - "0.4h: 0001_email_auth_log.sql + closed enum"
  - "0.4h: authbridge/mod.rs"
  - "0.6h: jwt_validator.rs (FR-AUTH-004 JWKS fetch + cache)"
  - "0.5h: mailbox_resolver.rs (subject_id → mailbox path)"
  - "0.4h: cache.rs (Redis 60s TTL)"
  - "0.4h: revocation_consumer.rs (NATS from FR-PORTAL-004)"
  - "0.3h: audit/auth_events.rs"
  - "0.4h: handlers/authbridge_routes.rs (Stalwart HTTP auth endpoint)"
  - "1.6h: tests — 8 test files"
  - "1.0h: Stalwart config integration + smoke test"

risk_if_skipped: "Without authbridge, Stalwart auth + CyberOS auth are separate identity surfaces → users have one password for app + another for email = friction + security gap. Without DEC-1465 SCIM cascade, deprovisioned users retain email access for 8h-24h. Without DEC-1462 per-tenant scoping, one tenant's users can read another's mail. The 6h effort lands the SSO substrate."
---

## §1 — Description (BCP-14 normative)

The EMAIL service **MUST** ship Stalwart authbridge plugin at `services/email/src/authbridge/` validating FR-AUTH-004 JWTs against per-tenant mailbox scope, with 60s Redis cache, FR-PORTAL-004 SCIM revoke cascade, 6-outcome enum, and 4 memory audit kinds.

1. **MUST** define closed `auth_outcome` enum: `('success','jwt_invalid','jwt_expired','jwt_wrong_audience','subject_revoked','mailbox_unauthorized')` per DEC-1463. Cardinality 6.

2. **MUST** expose `POST /v1/email/auth` (Stalwart HTTP auth backend) body `{ username, password, protocol }`. Handler:
   - Treats `password` field as JWT per DEC-1461.
   - Validates JWT against FR-AUTH-004 issuer + JWKS.
   - Validates `username` matches JWT's subject_id @ tenant.
   - Checks Redis cache; hit → return cached outcome.
   - Else verify + cache + return.
   - Returns 200 + `{ outcome, mailbox_path, permissions }` for Stalwart enforcement.

3. **MUST** resolve mailbox path per DEC-1462: `subject_id@tenant_slug.cyberos.world`. Per-Engagement aliases possible via FR-PORTAL-002 CNAME mapping.

4. **MUST** cache validated JWTs 60s per DEC-1464. Redis key = `email_auth:{jti}`; value = `{ outcome, mailbox_path, permissions, exp }`.

5. **MUST** subscribe to FR-PORTAL-004 SCIM revoke NATS events per DEC-1465. Consumer invalidates Redis cache for subject + emits `email.auth_failed` reason='subject_revoked' for next attempt.

6. **MUST** define `email_auth_log` at migration `0001`: `(id BIGSERIAL, tenant_id UUID, subject_id UUID, protocol TEXT, outcome auth_outcome, source_ip_hash16 TEXT, ts TIMESTAMPTZ DEFAULT now())`. Append-only.

7. **MUST** emit 4 memory audit kinds per DEC-1466. PII-scrub source_ip via FR-MEMORY-111.

8. **MUST** thread trace_id end-to-end.

9. **MUST NOT** cache JWT > 60s per DEC-1464.

10. **MUST NOT** allow cross-tenant mailbox per DEC-1462.

---

## §2 — Why this design (rationale)

**Why JWT-as-password (DEC-1461)?** IMAP/SMTP clients expect username/password; JWT in password field = client-transparent SSO without new protocols. Industry pattern (Google App Passwords replaced by OAuth, but base auth still username/password mechanism).

**Why 60s cache (DEC-1464)?** IMAP IDLE keeps connection open + re-auths frequently. Per-request JWT verify = ~5ms × 100 req/s = 500ms/s CPU; 60s cache = ~99% hit rate.

**Why SCIM cascade (DEC-1465)?** Without it, JWT exp window (8h IdP-auth) is the only revocation timeline. Cascade brings it to <60s.

---

## §3 — API contract

```sql
-- 0001_email_auth_log.sql
CREATE TYPE auth_outcome AS ENUM ('success','jwt_invalid','jwt_expired','jwt_wrong_audience','subject_revoked','mailbox_unauthorized');

CREATE TABLE email_auth_log (
  id BIGSERIAL PRIMARY KEY,
  tenant_id UUID,
  subject_id UUID,
  protocol TEXT NOT NULL CHECK (protocol IN ('imap','smtp','jmap','managesieve')),
  outcome auth_outcome NOT NULL,
  source_ip_hash16 TEXT,
  ts TIMESTAMPTZ NOT NULL DEFAULT now(),
  trace_id CHAR(32)
);
CREATE INDEX idx_auth_log_subject ON email_auth_log(subject_id, ts DESC);
ALTER TABLE email_auth_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY email_auth_log_rls ON email_auth_log
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON email_auth_log FROM cyberos_app;
```

Stalwart backend HTTP contract: `POST /v1/email/auth` returns `{ outcome, mailbox_path, permissions: [...] }`.

---

## §4 — Acceptance criteria

1. **auth_outcome cardinality 6**.
2. **Valid JWT** → success + mailbox_path returned.
3. **Expired JWT** → jwt_expired + no cache.
4. **Wrong audience** → jwt_wrong_audience.
5. **Username mismatch** → mailbox_unauthorized.
6. **60s cache hit** — second request within 60s returns cached.
7. **Cache TTL respected** — at T+61s, re-validates.
8. **SCIM revoke invalidates cache** — NATS event → next request 'subject_revoked'.
9. **4 memory audit kinds emitted**.
10. **Per-tenant mailbox path correct** — `subj@tenant.cyberos.world` format.
11. **Cross-tenant attempt denied** — subject from tenant A trying tenant B mailbox → mailbox_unauthorized.
12. **Source IP PII-scrubbed**.
13. **Trace_id end-to-end**.
14. **Stalwart integration smoke** — real IMAP login via Stalwart succeeds.
15. **Redis unavailable fallback** — falls through to per-request JWT verify.
16. **JWKS rotation handled** — FR-AUTH-004 key rotation propagates.
17. **Protocol enum validated** — unknown protocol → 400.
18. **Non-JWT password rejected** — random string → jwt_invalid.
19. **RLS denies cross-tenant log read**.
20. **Audit on every outcome**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn valid_jwt_returns_success() {
    let ctx = TestContext::new().await;
    let jwt = ctx.mint_jwt(ctx.subject_id, ctx.tenant_id).await;
    let r = ctx.post_auth("alice@acme.cyberos.world", &jwt, "imap").await;
    assert_eq!(r.status(), 200);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["outcome"], "success");
    assert_eq!(body["mailbox_path"], "alice@acme.cyberos.world");
}

#[tokio::test]
async fn cache_hit_within_60s() {
    let ctx = TestContext::new().await;
    let jwt = ctx.mint_jwt(ctx.subject_id, ctx.tenant_id).await;
    ctx.post_auth("alice@acme.cyberos.world", &jwt, "imap").await;
    let metrics_before = ctx.jwt_verify_count();
    ctx.post_auth("alice@acme.cyberos.world", &jwt, "imap").await;
    let metrics_after = ctx.jwt_verify_count();
    assert_eq!(metrics_after, metrics_before);  // cached, no re-verify
}

#[tokio::test]
async fn scim_revoke_invalidates() {
    let ctx = TestContext::new().await;
    let jwt = ctx.mint_jwt(ctx.subject_id, ctx.tenant_id).await;
    ctx.post_auth("alice@acme.cyberos.world", &jwt, "imap").await;
    ctx.publish_scim_revoke_event(ctx.subject_id).await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    let r = ctx.post_auth("alice@acme.cyberos.world", &jwt, "imap").await;
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["outcome"], "subject_revoked");
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-EMAIL-001, FR-AUTH-004.
**Cross-module:** FR-PORTAL-004 (SCIM cascade), FR-AI-003, FR-MEMORY-111.

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| JWKS unreachable | timeout 5s | Sev-2; serve from cached JWKS | AUTH recovery |
| Redis unavailable | error | Fallback per-request verify; sev-3 | Redis recovery |
| JWT clock skew | exp check tolerance ±60s | Inherent | NTP |
| Stalwart auth timeout | 10s default | Sev-2 | Stalwart investigation |
| Cross-tenant attempt | username check | mailbox_unauthorized | Inherent |
| SCIM cascade delay | NATS lag | Sev-3; cache TTL covers worst case | NATS recovery |
| Same JWT replay > 60s past exp | exp check | jwt_expired | Inherent |
| Cache poisoning | Redis ACL | Inherent | Redis isolation |
| Protocol enum unknown | check | 400 | Stalwart config fix |
| Per-request rate spike | rate limit | 429 | Inherent |

## §11 — Implementation notes

**§11.1** Stalwart `auth.backend.type = http` configured to point at `/v1/email/auth`.
**§11.2** JWT-as-password tested with real IMAP clients (Apple Mail, Thunderbird).
**§11.3** Per-tenant mailbox isolation enforced by Stalwart given correct mailbox_path return.
**§11.4** Redis cache pruned at TTL automatically.
**§11.5** Source IP hashed before persist; never raw IP in audit chain.

---

*End of FR-EMAIL-002 spec.*
