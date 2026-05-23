---
id: FR-EMAIL-004
title: "EMAIL DKIM signing + ARC chain forward + BIMI brand indicator — RFC 6376 + RFC 8617 + BIMI 1.0 per-tenant outbound auth"
module: EMAIL
priority: MUST
status: done
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-17
shipped: 2026-05-23
memory_chain_hash: null
related_frs: [FR-EMAIL-001, FR-EMAIL-002, FR-EMAIL-009, FR-PORTAL-002, FR-AI-003, FR-MEMORY-111]
depends_on: [FR-EMAIL-001]
blocks: [FR-EMAIL-009]

source_pages:
  - https://datatracker.ietf.org/doc/html/rfc6376  # DKIM
  - https://datatracker.ietf.org/doc/html/rfc8617  # ARC
  - https://bimigroup.org/

source_decisions:
  - DEC-1470 2026-05-17 — Per-tenant DKIM keypair (Ed25519 per RFC 8463 + RSA-2048 fallback); generated at tenant provisioning; KMS-wrapped private key; public published as DNS TXT record at `cyberos._domainkey.<tenant_cname>`
  - DEC-1471 2026-05-17 — ARC chain (Authenticated Received Chain) for forwarded messages: preserves auth results across hops; signs cv=pass/fail/none verdict
  - DEC-1472 2026-05-17 — BIMI 1.0: per-tenant brand indicator pointing at SVG-tinified logo + VMC (Verified Mark Certificate); requires DMARC enforcement at p=quarantine or stricter
  - DEC-1473 2026-05-17 — Closed enum `dkim_outcome` = {signed_ed25519, signed_rsa, sign_failed_no_key, sign_failed_kms}; cardinality 4
  - DEC-1474 2026-05-17 — DNS setup wizard: tenant_admin guided through DKIM/SPF/DMARC/BIMI TXT records at signup or per-CNAME setup
  - DEC-1475 2026-05-17 — memory audit kinds: email.dkim_signed, email.arc_chain_extended, email.bimi_indicator_attached, email.dns_verification_passed, email.dns_verification_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/email/
  new_files:
    - services/email/migrations/0002_tenant_dkim_keys.sql
    - services/email/migrations/0003_tenant_dns_setup.sql
    - services/email/src/dkim/mod.rs
    - services/email/src/dkim/signer.rs
    - services/email/src/dkim/keygen.rs
    - services/email/src/arc/mod.rs
    - services/email/src/arc/chain_forward.rs
    - services/email/src/bimi/mod.rs
    - services/email/src/bimi/svg_tinifier.rs
    - services/email/src/dns/setup_wizard.rs
    - services/email/src/dns/verifier.rs
    - services/email/src/audit/dkim_events.rs
    - services/email/src/handlers/dkim_routes.rs
    - services/email/tests/dkim_ed25519_sign_test.rs
    - services/email/tests/dkim_rsa_fallback_test.rs
    - services/email/tests/arc_chain_test.rs
    - services/email/tests/bimi_svg_tinify_test.rs
    - services/email/tests/dns_setup_wizard_test.rs
    - services/email/tests/dns_verification_test.rs
    - services/email/tests/dkim_outcome_enum_test.rs
    - services/email/tests/dkim_per_tenant_isolation_test.rs
    - services/email/tests/dkim_audit_emission_test.rs

  modified_files:
    - services/email/src/lib.rs
    - services/ten/src/provisioning/orchestrator.rs                   # generate DKIM keypair at tenant provisioning

  allowed_tools:
    - file_read: services/{email,ten}/**
    - file_write: services/email/{src,tests,migrations}/**
    - file_write: services/ten/src/provisioning/orchestrator.rs
    - bash: cd services/email && cargo test dkim

  disallowed_tools:
    - share DKIM keys across tenants (per DEC-1470)
    - skip ARC verification on inbound forwards (per DEC-1471)
    - attach BIMI without DMARC enforcement (per DEC-1472)

effort_hours: 6
sub_tasks:
  - "0.4h: 0002 + 0003 migrations"
  - "0.4h: dkim/mod.rs + closed enum"
  - "0.5h: keygen.rs (Ed25519 + RSA at provisioning)"
  - "0.6h: signer.rs (per RFC 6376)"
  - "0.5h: arc/chain_forward.rs (per RFC 8617)"
  - "0.4h: bimi/mod.rs + svg_tinifier.rs"
  - "0.4h: dns/setup_wizard.rs"
  - "0.4h: dns/verifier.rs"
  - "0.3h: audit/dkim_events.rs"
  - "0.3h: handlers/dkim_routes.rs"
  - "1.5h: tests — 9 test files"
  - "0.3h: TEN provisioning integration"

risk_if_skipped: "Without DKIM, outbound emails fail SPF/DKIM checks → relegated to spam folders → delivery rate 30-50% (industry baseline) vs 95%+ with proper auth. Without DEC-1471 ARC, forwarded messages lose auth chain → recipient's spam filter rejects. Without BIMI, no brand logo in recipient inbox = no trust signal. Without DEC-1470 per-tenant keys, one tenant's compromise = all tenants' email forgeable. The 6h effort lands the deliverability primitive."
---

## §1 — Description (BCP-14 normative)

The EMAIL service **MUST** ship DKIM signing + ARC chain forward + BIMI brand indicator at `services/email/src/{dkim,arc,bimi,dns}/`, per-tenant Ed25519 keypair generated at provisioning, DNS setup wizard, RFC-conformant signing, and 5 memory audit kinds.

1. **MUST** define closed `dkim_outcome` enum: `('signed_ed25519','signed_rsa','sign_failed_no_key','sign_failed_kms')` per DEC-1473. Cardinality 4.

2. **MUST** generate per-tenant DKIM keypair per DEC-1470 at FR-TEN-001 provisioning:
   - Ed25519 primary per RFC 8463.
   - RSA-2048 fallback for legacy receivers.
   - Private keys KMS-wrapped.
   - Public keys serialized as DNS TXT format for publication.

3. **MUST** define `tenant_dkim_keys` table at migration `0002`: `(tenant_id UUID NOT NULL, key_kind TEXT NOT NULL CHECK (key_kind IN ('ed25519','rsa2048')), selector TEXT NOT NULL DEFAULT 'cyberos', private_key_kms_blob BYTEA NOT NULL, public_key_dns_txt TEXT NOT NULL, kms_key_id TEXT NOT NULL, generated_at TIMESTAMPTZ NOT NULL DEFAULT now(), revoked_at TIMESTAMPTZ, PRIMARY KEY (tenant_id, key_kind, selector))`.

4. **MUST** sign outbound messages via `dkim/signer.rs::sign(message, tenant_id)` per RFC 6376:
   - Resolves private key from `tenant_dkim_keys` (Ed25519 primary).
   - KMS-decrypts.
   - Computes canonical body hash.
   - Signs over headers + hash.
   - Emits `DKIM-Signature` header.
   - Emits memory row `email.dkim_signed` with outcome.

5. **MUST** support ARC chain forward per DEC-1471 + RFC 8617 via `arc/chain_forward.rs`. For forwarded inbound mail:
   - Verifies existing ARC chain (cv= verdict).
   - Appends new ARC-Authentication-Results + ARC-Message-Signature + ARC-Seal.
   - Preserves chain for downstream receivers.

6. **MUST** attach BIMI brand indicator per DEC-1472 via `bimi/mod.rs`:
   - Requires tenant's DMARC at `p=quarantine` or stricter (verified via DNS check).
   - SVG logo from FR-PORTAL-002 brand pack → tinified (SVG Tiny PS) via `svg_tinifier.rs`.
   - Optional VMC certificate URL for verified-mark display.
   - Adds `BIMI-Selector` header + DNS BIMI TXT record.

7. **MUST** define `tenant_dns_setup` at migration `0003`: `(tenant_id UUID PRIMARY KEY, custom_domain TEXT, dkim_txt_published BOOLEAN, spf_txt_published BOOLEAN, dmarc_txt_published BOOLEAN, dmarc_policy TEXT, bimi_txt_published BOOLEAN, vmc_cert_url TEXT, last_verified_at TIMESTAMPTZ, verification_failures INT NOT NULL DEFAULT 0)`.

8. **MUST** expose DNS setup wizard `POST /v1/admin/tenants/{tid}/email/dns-setup`. Returns required TXT records (DKIM public key + SPF + DMARC + BIMI). Tenant admin publishes; verifier polls.

9. **MUST** verify DNS via `dns/verifier.rs::verify(tenant_id)`. Daily job:
   - DNS resolve each expected TXT.
   - Compare with database expectation.
   - Mismatch → emit `email.dns_verification_failed` sev-2.
   - Success → `email.dns_verification_passed` sev-3.

10. **MUST** emit 5 memory audit kinds per DEC-1475.

11. **MUST** thread trace_id end-to-end.

12. **MUST NOT** share keys across tenants (per DEC-1470).

13. **MUST NOT** attach BIMI without DMARC enforcement (per DEC-1472).

---

## §2 — Why this design (rationale)

**Why per-tenant DKIM (DEC-1470)?** Compromise scoping; shared key = single-failure compromise of every tenant.

**Why Ed25519 + RSA dual (DEC-1470)?** Legacy receivers (some enterprise mail servers) don't yet support Ed25519; RSA fallback ensures delivery.

**Why ARC (DEC-1471)?** Mailing-list forwards break SPF/DKIM; ARC preserves original auth verdict for receivers.

**Why BIMI requires DMARC (DEC-1472)?** BIMI spec mandates p=quarantine+ to prevent abuse — spoofers can't get brand-recognised inboxes.

---

## §3 — API contract

```sql
-- 0002_tenant_dkim_keys.sql
CREATE TABLE tenant_dkim_keys (
  tenant_id UUID NOT NULL,
  key_kind TEXT NOT NULL CHECK (key_kind IN ('ed25519','rsa2048')),
  selector TEXT NOT NULL DEFAULT 'cyberos',
  private_key_kms_blob BYTEA NOT NULL,
  public_key_dns_txt TEXT NOT NULL,
  kms_key_id TEXT NOT NULL,
  generated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  revoked_at TIMESTAMPTZ,
  PRIMARY KEY (tenant_id, key_kind, selector)
);
ALTER TABLE tenant_dkim_keys ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_dkim_keys_rls ON tenant_dkim_keys
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON tenant_dkim_keys FROM cyberos_app;
GRANT UPDATE (revoked_at) ON tenant_dkim_keys TO cyberos_app;

-- 0003_tenant_dns_setup.sql
CREATE TABLE tenant_dns_setup (
  tenant_id UUID PRIMARY KEY,
  custom_domain TEXT,
  dkim_txt_published BOOLEAN NOT NULL DEFAULT false,
  spf_txt_published BOOLEAN NOT NULL DEFAULT false,
  dmarc_txt_published BOOLEAN NOT NULL DEFAULT false,
  dmarc_policy TEXT,
  bimi_txt_published BOOLEAN NOT NULL DEFAULT false,
  vmc_cert_url TEXT,
  last_verified_at TIMESTAMPTZ,
  verification_failures INT NOT NULL DEFAULT 0,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
ALTER TABLE tenant_dns_setup ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_dns_setup_rls ON tenant_dns_setup
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE DELETE ON tenant_dns_setup FROM cyberos_app;
GRANT UPDATE (dkim_txt_published, spf_txt_published, dmarc_txt_published, dmarc_policy,
              bimi_txt_published, vmc_cert_url, last_verified_at, verification_failures, custom_domain, updated_at)
  ON tenant_dns_setup TO cyberos_app;
```

Endpoints:
```text
POST   /v1/admin/tenants/{tid}/email/dns-setup        (tenant_admin)
POST   /v1/admin/tenants/{tid}/email/dns-verify       (tenant_admin)
POST   /v1/admin/tenants/{tid}/email/bimi-enable      (tenant_admin)
```

---

## §4 — Acceptance criteria

1. **dkim_outcome cardinality 4**.
2. **DKIM Ed25519 sign** — outbound message gets `DKIM-Signature` header with a=ed25519-sha256.
3. **RSA fallback** — legacy receiver fixture → RSA signature attached as additional header.
4. **Per-tenant key isolation** — tenant A's key cannot sign for tenant B (RLS enforced).
5. **DNS wizard returns TXT records** — POST returns expected DKIM/SPF/DMARC/BIMI TXT values.
6. **DNS verification daily** — wizard records publication state; verifier polls + updates.
7. **ARC chain extended** — forwarded message has ARC-Seal added.
8. **BIMI requires DMARC** — bimi-enable without DMARC=quarantine → 412.
9. **SVG tinify** — uploaded SVG processed to BIMI-compliant Tiny PS.
10. **5 memory audit kinds emitted**.
11. **KMS unavailable** → sign_failed_kms outcome + sev-1 audit.
12. **Key missing for tenant** → sign_failed_no_key.
13. **DNS verification failure persisted** — failures counter increments.
14. **Trace_id end-to-end**.
15. **Cross-tenant RLS denied**.
16. **VMC cert URL optional** — BIMI works without VMC (no verified mark badge).
17. **Selector configurable** — default 'cyberos'; tenant can change.
18. **Provisioning integration** — new tenant gets DKIM keys auto-generated.
19. **Revoked key not used for signing** — revoked_at set → skipped.
20. **PII scrub** — DNS records non-PII; raw IP not in audit.

---

## §5 — Verification

```rust
#[tokio::test]
async fn dkim_signs_outbound_message() {
    let ctx = TestContext::with_provisioned_tenant().await;
    let msg = ctx.outbound_message("alice@acme.cyberos.world", "bob@example.com").await;
    let signed = ctx.dkim_sign(ctx.tenant_id, msg).await;
    assert!(signed.headers().contains_key("DKIM-Signature"));
    let sig: &str = signed.headers().get("DKIM-Signature").unwrap();
    assert!(sig.contains("a=ed25519-sha256"));
    assert!(sig.contains("s=cyberos"));
}

#[tokio::test]
async fn cross_tenant_key_isolation() {
    let ctx = TestContext::with_two_tenants().await;
    let r = ctx.as_tenant_a().dkim_sign_for_tenant(ctx.tenant_b_id, "test").await;
    assert!(r.is_err());  // RLS rejects
}

#[tokio::test]
async fn bimi_requires_dmarc() {
    let ctx = TestContext::with_provisioned_tenant().await;
    let r = ctx.bimi_enable(ctx.tenant_id).await;
    assert_eq!(r.status(), 412);  // DMARC not yet set
    ctx.set_dmarc_policy(ctx.tenant_id, "quarantine").await;
    let r = ctx.bimi_enable(ctx.tenant_id).await;
    assert_eq!(r.status(), 200);
}

#[tokio::test]
async fn provisioning_generates_keys() {
    let ctx = TestContext::new().await;
    let tid = ctx.provision_tenant().await;
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM tenant_dkim_keys WHERE tenant_id=$1")
        .bind(tid).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(count, 2);  // ed25519 + rsa
}

// 5.5..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-EMAIL-001.
**Cross-module:** FR-TEN-001 (keygen at provisioning), FR-PORTAL-002 (BIMI logo), FR-AI-003, FR-MEMORY-111.
**Downstream:** FR-EMAIL-009 (outbound send).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| KMS unavailable | timeout | sign_failed_kms; sev-1 | KMS recovery |
| Key missing | lookup miss | sign_failed_no_key | Regenerate via wizard |
| DNS not propagated | verifier poll | failures++; sev-2 at 5+ failures | Tenant admin fixes DNS |
| ARC chain corrupted upstream | verify fail | cv=fail; forward with verdict; sev-3 | Inherent |
| BIMI SVG > 32KB | tinifier limit | 400 | Tenant slims logo |
| VMC URL unreachable | fetch fail | BIMI works without verified mark; sev-3 | Inherent |
| Cross-tenant signing attempt | RLS | Inherent | None |
| Revoked key signing attempt | revoked_at check | sign_failed_no_key | Generate new |
| DNS provider rate limit | poll backoff | sev-3 | Inherent |
| Selector collision | partial unique | tenant chooses unique | Inherent |
| Custom domain mismatch | wizard | 400 | Fix domain config |
| KMS rotation breaks signing | key archive | Old signed msgs verify with old pub key | Inherent |

## §11 — Implementation notes
- §11.1 Ed25519 key 32 bytes; RSA-2048 256 bytes; DNS TXT under 255-char limit handled via chunked TXT.
- §11.2 BIMI SVG tinification via `usvg` Rust crate.
- §11.3 ARC verification uses `mail-auth` Rust crate.
- §11.4 DNS verifier uses `hickory-resolver`.
- §11.5 Provisioning hook in FR-TEN-001 orchestrator generates both keypairs.

---

*End of FR-EMAIL-004 spec.*
