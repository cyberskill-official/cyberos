---
id: FR-EMAIL-006
title: "EMAIL tracked-domain → CRM auto-link — inbound message from tenant-tracked domain auto-creates/links CRM contact + thread association"
module: EMAIL
priority: SHOULD
status: draft
verify: T
phase: P2
milestone: P2 · slice 1
slice: 1
owner: Stephen Cheng (CRO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-EMAIL-001, FR-CRM-001, FR-AI-003, FR-MEMORY-111]
depends_on: [FR-EMAIL-001, FR-CRM-001]
blocks: [FR-CRM-002]

source_pages:
  - website/docs/modules/email.html#crm-link

source_decisions:
  - DEC-1570 2026-05-17 — Auto-link triggered ONLY on inbound message from sender domain matching tracked_domains list per tenant; outbound not linked here (FR-EMAIL-009 sets crm_id at send)
  - DEC-1571 2026-05-17 — If contact exists by email: link; else create CRM contact + link (auto-creation per DEC-1574)
  - DEC-1572 2026-05-17 — Tracked domains stored per tenant in tracked_domains table: {tenant_id, domain, added_by, added_at}
  - DEC-1573 2026-05-17 — Closed enum `link_origin` = {auto_tracked_domain, manual, send_intent, crm_jit}; cardinality 4
  - DEC-1574 2026-05-17 — Auto-created contact: name from From: header display name; company from domain via FR-AI-003 lookup (cached)
  - DEC-1575 2026-05-17 — memory audit kinds: email.crm_link_auto_created, email.crm_link_existing_matched, email.crm_link_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/email/
  new_files:
    - services/email/migrations/0008_tracked_domains.sql
    - services/email/src/crm_link/mod.rs
    - services/email/src/crm_link/domain_matcher.rs
    - services/email/src/crm_link/auto_contact_creator.rs
    - services/email/src/handlers/tracked_domains_routes.rs
    - services/email/src/audit/crm_link_events.rs
    - services/email/tests/crm_link_tracked_domain_test.rs
    - services/email/tests/crm_link_existing_contact_test.rs
    - services/email/tests/crm_link_auto_create_test.rs
    - services/email/tests/crm_link_untracked_skipped_test.rs
    - services/email/tests/crm_link_link_origin_enum_cardinality_test.rs
    - services/email/tests/crm_link_audit_emission_test.rs

  modified_files:
    - services/email/src/inbound_processor.rs

  allowed_tools:
    - file_read: services/{email,crm}/**
    - file_write: services/email/{src,tests,migrations}/**
    - bash: cd services/email && cargo test crm_link

  disallowed_tools:
    - link on outbound (per DEC-1570 — handled at send)
    - auto-link untracked domains (per DEC-1572 — explicit allowlist)

effort_hours: 5
sub_tasks:
  - "0.3h: 0008_tracked_domains.sql"
  - "0.3h: crm_link/mod.rs"
  - "0.4h: domain_matcher.rs"
  - "0.6h: auto_contact_creator.rs"
  - "0.4h: handlers/tracked_domains_routes.rs"
  - "0.3h: audit/crm_link_events.rs"
  - "0.4h: inbound_processor.rs hook"
  - "1.6h: tests — 6 test files"
  - "0.7h: CRO UI for tracked-domain management"

risk_if_skipped: "Without auto-link, CRO/AM must manually link every inbound — workflow drag. Without DEC-1572 tracked-domain allowlist, every inbound creates contact (noise). Without DEC-1574 auto-creation, tracked domain matches with no contact requires manual entry."
---

## §1 — Description (BCP-14 normative)

The EMAIL service **MUST** ship CRM auto-link at `services/email/src/crm_link/` triggered on inbound, matched against tenant's tracked_domains, contact created/matched, thread linked, 3 memory audit kinds.

1. **MUST** hook into `services/email/src/inbound_processor.rs` after message stored: call `crm_link::process(message)`.

2. **MUST** check sender domain against `tracked_domains` per DEC-1572. Match → continue; non-match → skip (no link).

3. **MUST** check contact existence by sender email: if exists → link thread to existing contact_id; else create per DEC-1574.

4. **MUST** auto-create contact via `auto_contact_creator.rs::create(tenant_id, from_header)`:
   - name: from `From:` display name (`"Acme Corp <john@acme.com>"` → `"Acme Corp"`)
   - email: from `From:` address
   - company: FR-AI-003 lookup on domain (24h cached); fallback domain text
   - link_origin: `auto_tracked_domain` per DEC-1573

5. **MUST** validate `link_origin` against closed enum per DEC-1573.

6. **MUST** define `tracked_domains` table at migration `0008`:
   ```sql
   CREATE TABLE tracked_domains (
     tenant_id UUID NOT NULL,
     domain TEXT NOT NULL,
     added_by UUID NOT NULL,
     added_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     notes TEXT,
     PRIMARY KEY (tenant_id, domain)
   );
   CREATE INDEX tracked_domains_tenant_idx ON tracked_domains(tenant_id);
   ALTER TABLE tracked_domains ENABLE ROW LEVEL SECURITY;
   CREATE POLICY tracked_domains_rls ON tracked_domains
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON tracked_domains FROM cyberos_app;
   GRANT DELETE ON tracked_domains TO cyberos_app;  -- CRO can untrack
   ```

7. **MUST** expose admin endpoints for tracked-domain management:
   ```text
   POST   /v1/email/tracked-domains       (CRO-only)
   DELETE /v1/email/tracked-domains/{d}   (CRO-only)
   GET    /v1/email/tracked-domains       (list)
   ```

8. **MUST** add `link_origin` column to FR-EMAIL-001 messages table:
   ```sql
   ALTER TABLE messages ADD COLUMN crm_contact_id UUID;
   ALTER TABLE messages ADD COLUMN link_origin TEXT
     CHECK (link_origin IS NULL OR link_origin IN
       ('auto_tracked_domain','manual','send_intent','crm_jit'));
   CREATE INDEX messages_crm_contact_idx ON messages(tenant_id, crm_contact_id) WHERE crm_contact_id IS NOT NULL;
   GRANT UPDATE (crm_contact_id, link_origin) ON messages TO cyberos_app;
   ```

9. **MUST** emit 3 memory audit kinds per DEC-1575. PII per FR-MEMORY-111: contact email/name SHA-256 hashed; domain (already public) ok.

10. **MUST** thread trace_id from inbound processor → matcher → CRM upsert → audit.

11. **MUST NOT** link untracked domains per DEC-1572.

12. **MUST NOT** link outbound (handled by FR-EMAIL-009 at send-intent) per DEC-1570.

---

## §2 — Why this design

**Why tracked-domain allowlist (DEC-1572)?** Random inbound (newsletters, vendors, spam) shouldn't pollute CRM. Allowlist gives CRO control.

**Why auto-create vs match-only (DEC-1574)?** Tracked domains imply business interest; manual entry blocks workflow.

**Why link_origin enum (DEC-1573)?** Distinguishes auto-discovery from manual curation; CRO can audit auto-links separately.

---

## §3 — API contract (see §1.7)

Sample tracked-domain add:
```json
POST /v1/email/tracked-domains
{ "domain": "acme.com", "notes": "Strategic account" }
```

Sample message link result (after inbound process):
```json
{
  "message_id": "uuid",
  "thread_id": "uuid",
  "crm_contact_id": "uuid",
  "link_origin": "auto_tracked_domain",
  "contact_created": true
}
```

---

## §4 — Acceptance criteria
1. **Inbound from tracked domain → linked**. 2. **Inbound from untracked domain → skipped**. 3. **Existing contact reused (no duplicate)**. 4. **New contact auto-created if missing**. 5. **Display name parsed from From: header**. 6. **Company inferred via FR-AI-003 lookup**. 7. **AI company lookup cached 24h**. 8. **link_origin enum 4 values + cardinality test**. 9. **3 memory audit kinds emitted**. 10. **PII scrubbed (email/name SHA256)**. 11. **RLS denies cross-tenant**. 12. **CRO-only tracked-domain mgmt**. 13. **Trace_id preserved**. 14. **Outbound NOT touched**. 15. **Domain match case-insensitive**. 16. **Subdomain match optional (configurable per domain)**. 17. **REVOKE UPDATE on tracked_domains (immutable add, can DELETE)**. 18. **Multiple messages same contact → reuse single contact_id**. 19. **From: header malformed → skip with sev-3 audit**. 20. **CRM contact created → CRM-side FR-CRM-001 audit also fires**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn tracked_domain_creates_and_links() {
    let ctx = TestContext::with_tracked_domain("acme.com").await;
    let msg = ctx.receive_inbound("john@acme.com").await;
    let linked: Message = ctx.fetch_message(msg.id).await;
    assert_eq!(linked.link_origin.as_deref(), Some("auto_tracked_domain"));
    assert!(linked.crm_contact_id.is_some());
    let contact = ctx.fetch_contact(linked.crm_contact_id.unwrap()).await;
    assert_eq!(contact.email, "john@acme.com");
}

#[tokio::test]
async fn untracked_domain_skipped() {
    let ctx = TestContext::new_tenant_no_tracked().await;
    let msg = ctx.receive_inbound("random@spam.com").await;
    let m: Message = ctx.fetch_message(msg.id).await;
    assert!(m.crm_contact_id.is_none());
    assert!(m.link_origin.is_none());
}

#[tokio::test]
async fn existing_contact_reused() {
    let ctx = TestContext::with_tracked_domain("acme.com").await;
    let c1 = ctx.create_contact("jane@acme.com").await;
    let msg = ctx.receive_inbound("jane@acme.com").await;
    let m: Message = ctx.fetch_message(msg.id).await;
    assert_eq!(m.crm_contact_id, Some(c1));
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-EMAIL-001, FR-CRM-001.
**Cross-module:** FR-AI-003 (company lookup), FR-AUTH-101 (CRO role), FR-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| FR-AI-003 lookup timeout | retry once | use domain text as company | manual fix |
| From: header malformed | regex fail | skip + sev-3 audit | data fix |
| Tracked domain DB unreachable | sql error | skip link + sev-2 audit | retry on next inbound |
| Duplicate contact race | UNIQUE on (tenant,email) | second insert ON CONFLICT | inherent |
| Domain match case-mismatch | lowercase in matcher | match | inherent |
| Subdomain (e.g. sub.acme.com vs acme.com) | configurable | per-tenant flag | CRO config |
| Catch-all routing (multiple recipients) | per-message | linked once per thread | inherent |
| Internal sender (own domain) | skip | not linked | by design |
| Auto-create disabled (CRO toggle) | config check | match-only, skip create | configurable |
| FR-AI-003 quota | downgrade | use domain only | inherent |

## §11 — Implementation notes
- §11.1 Domain extraction: `email.split('@').last().lowercase()`.
- §11.2 Display-name parse via `mailparse` crate.
- §11.3 FR-AI-003 prompt: "What company owns the domain {domain}? Reply with name only, no commentary."
- §11.4 Cache key: `company_for_domain:{domain}`, TTL 86400s.
- §11.5 memory audit: domain, link_origin, contact_created flag; email/name SHA256.

---

*End of FR-EMAIL-006 spec.*
