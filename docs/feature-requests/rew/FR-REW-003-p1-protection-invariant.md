---
id: FR-REW-003
title: "REW P1 protection invariant — DB CHECK constraint + service-layer guard forbidding any P1 cash reduction (raise-only)"
module: REW
priority: MUST
status: ready_to_implement
verify: T
phase: P2
milestone: P2 · slice 1
slice: 1
owner: Stephen Cheng (CFO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-REW-001, FR-AUTH-101, FR-MEMORY-111]
depends_on: [FR-REW-001]
blocks: []

source_pages:
  - website/docs/modules/rew.html#p1-protection

source_decisions:
  - DEC-2170 2026-05-17 — P1 Base cash compensation cannot be reduced via any path — DB-level CHECK + service-layer guard + CI test
  - DEC-2171 2026-05-17 — Closed enum `p1_change_kind` = {raise, role_change_upward, role_change_lateral_same_p1, role_change_demotion_requires_consent}; cardinality 4
  - DEC-2172 2026-05-17 — Demotion (only path that could reduce P1) requires member written consent + CEO+CFO co-sign per VN Labour Code Art. 35
  - DEC-2173 2026-05-17 — Trigger function checks new_amount >= old_amount on INSERT of new comp record for same member+p1_base; rejects if violated
  - DEC-2174 2026-05-17 — memory audit kinds: rew.p1_raise_committed, rew.p1_change_attempted_violation, rew.p1_demotion_consent_recorded

build_envelope:
  language: rust 1.81
  service: cyberos/services/rew/
  new_files:
    - services/rew/migrations/0003_p1_protection.sql
    - services/rew/src/p1_guard/mod.rs
    - services/rew/src/p1_guard/validator.rs
    - services/rew/src/p1_guard/demotion_consent.rs
    - services/rew/src/audit/p1_events.rs
    - services/rew/tests/p1_reduction_rejected_test.rs
    - services/rew/tests/p1_change_kind_enum_cardinality_test.rs
    - services/rew/tests/p1_demotion_consent_test.rs
    - services/rew/tests/p1_db_trigger_test.rs
    - services/rew/tests/p1_audit_emission_test.rs

  modified_files:
    - services/rew/src/comp/mod.rs

  allowed_tools:
    - file_read: services/rew/**
    - file_write: services/rew/{src,tests,migrations}/**
    - bash: cd services/rew && cargo test p1_guard

  disallowed_tools:
    - reduce P1 without consent (per DEC-2170)
    - bypass DB trigger (per DEC-2173)

effort_hours: 4
sub_tasks:
  - "0.3h: 0003_p1_protection.sql (trigger function)"
  - "0.3h: p1_guard/mod.rs"
  - "0.4h: validator.rs (service-layer)"
  - "0.4h: demotion_consent.rs"
  - "0.3h: audit/p1_events.rs"
  - "1.7h: tests — 5 test files"
  - "0.6h: docs"

risk_if_skipped: "Without invariant, accidental P1 reduction breaks member trust + VN Labour Code. Without DEC-2172 demotion consent, illegal salary cut. Without DEC-2173 trigger, app bug can bypass."
---

## §1 — Description (BCP-14 normative)

The REW service **MUST** ship P1 protection at `services/rew/src/p1_guard/` with DB trigger + service guard + demotion consent flow, 3 memory audit kinds.

1. **MUST** validate `p1_change_kind` against closed enum per DEC-2171.

2. **MUST** create DB trigger at migration `0003`:
   ```sql
   CREATE OR REPLACE FUNCTION rew_p1_protection_trigger()
   RETURNS TRIGGER AS $$
   DECLARE
     old_amount BIGINT;
     new_amount BIGINT;
     consent_exists BOOLEAN;
   BEGIN
     IF NEW.income_kind != 'p1_base' THEN RETURN NEW; END IF;
     -- get prior current P1
     SELECT decrypt_amount(encrypted_amount_vnd, encryption_kms_key_arn) INTO old_amount
     FROM rew_comp_records
     WHERE tenant_id = NEW.tenant_id AND member_id = NEW.member_id AND income_kind = 'p1_base'
       AND valid_from <= NEW.valid_from AND (valid_to IS NULL OR valid_to > NEW.valid_from)
     ORDER BY valid_from DESC LIMIT 1;
     IF old_amount IS NULL THEN RETURN NEW; END IF;  -- first P1, allowed
     new_amount := decrypt_amount(NEW.encrypted_amount_vnd, NEW.encryption_kms_key_arn);
     IF new_amount >= old_amount THEN RETURN NEW; END IF;
     -- reduction attempt; check consent
     SELECT EXISTS(SELECT 1 FROM rew_p1_demotion_consents
                   WHERE member_id = NEW.member_id AND tenant_id = NEW.tenant_id
                     AND new_p1_vnd = new_amount AND status = 'fully_signed') INTO consent_exists;
     IF NOT consent_exists THEN
       RAISE EXCEPTION 'P1 reduction without consent: member=% old=% new=%', NEW.member_id, old_amount, new_amount;
     END IF;
     RETURN NEW;
   END;
   $$ LANGUAGE plpgsql;

   CREATE TRIGGER rew_p1_protection_trg
     BEFORE INSERT ON rew_comp_records
     FOR EACH ROW EXECUTE FUNCTION rew_p1_protection_trigger();

   CREATE TABLE rew_p1_demotion_consents (
     consent_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     member_id UUID NOT NULL,
     old_p1_vnd BIGINT NOT NULL,
     new_p1_vnd BIGINT NOT NULL CHECK (new_p1_vnd < old_p1_vnd),
     member_consent_doc_id UUID NOT NULL,
     ceo_signed_by UUID,
     ceo_signed_at TIMESTAMPTZ,
     cfo_signed_by UUID,
     cfo_signed_at TIMESTAMPTZ,
     status TEXT NOT NULL DEFAULT 'pending'
       CHECK (status IN ('pending','ceo_signed','cfo_signed','fully_signed','dismissed')),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   ALTER TABLE rew_p1_demotion_consents ENABLE ROW LEVEL SECURITY;
   CREATE POLICY p1_consent_rls ON rew_p1_demotion_consents
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON rew_p1_demotion_consents FROM cyberos_app;
   GRANT UPDATE (status, ceo_signed_by, ceo_signed_at, cfo_signed_by, cfo_signed_at) ON rew_p1_demotion_consents TO cyberos_app;
   ```

3. **MUST** validate at service layer at `validator.rs::validate(member, new_p1)` per DEC-2170:
   - Fetch current P1
   - If new < current AND no consent: reject before DB attempt (better UX than trigger fail)

4. **MUST** run consent flow at `demotion_consent.rs::create_request(member, new_p1, consent_doc_id)` per DEC-2172:
   - Member uploads written consent doc (FR-DOC-001 ref)
   - CEO + CFO sign
   - fully_signed → trigger now allows reduction

5. **MUST** expose endpoints:
   ```text
   POST /v1/rew/p1-demotion-consents
   POST /v1/rew/p1-demotion-consents/{id}/ceo-sign
   POST /v1/rew/p1-demotion-consents/{id}/cfo-sign
   ```

6. **MUST** emit 3 memory audit kinds per DEC-2174. PII per FR-MEMORY-111: amounts SHA-256 hashed.

7. **MUST** thread trace_id from violation/consent → audit.

8. **MUST NOT** bypass trigger per DEC-2173.

9. **MUST NOT** allow demotion without consent + dual-sign per DEC-2172.

---

## §2 — Why this design

**Why DB trigger (DEC-2173)?** Defense in depth — service code can have bugs; trigger is hard backstop.

**Why service validator (DEC-2173)?** Better UX — fail at API not at DB; clearer error message.

**Why demotion consent (DEC-2172)?** VN Labour Code Art. 35 requires written agreement to reduce salary.

**Why dual-sign (DEC-2172)?** Critical financial decision; single-signer governance gap.

---

## §3 — API contract

Sample consent request:
```json
POST /v1/rew/p1-demotion-consents
{
  "member_id": "uuid",
  "old_p1_vnd": 30000000,
  "new_p1_vnd": 25000000,
  "member_consent_doc_id": "uuid-signed-pdf"
}
```

---

## §4 — Acceptance criteria
1. **p1_change_kind enum cardinality 4**. 2. **DB trigger rejects P1 reduction without consent**. 3. **Service validator rejects pre-DB**. 4. **Consent table requires member doc upload**. 5. **CEO + CFO dual-sign required**. 6. **Same-person dual-sign rejected**. 7. **Once fully_signed → trigger allows**. 8. **3 memory audit kinds emitted**. 9. **PII scrubbed (amounts SHA256)**. 10. **RLS denies cross-tenant**. 11. **Trace_id preserved**. 12. **Append-only consent table**. 13. **CHECK new_p1 < old_p1 on consent**. 14. **CHECK new_p1 != old_p1 (no-op rejected)**. 15. **First P1 (no prior) allowed**. 16. **P2/P3 changes unaffected**. 17. **Trigger fires BEFORE INSERT**. 18. **Consent links to FR-DOC-001 doc**. 19. **CFO role required on consent record**. 20. **Trigger violation produces specific error message**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn p1_reduction_without_consent_rejected() {
    let ctx = TestContext::with_p1_30m_vnd().await;
    let r = ctx.try_set_p1(ctx.member_id, dec!(25_000_000)).await;
    assert!(r.is_err());
    assert!(r.error_message().contains("P1 reduction without consent"));
}

#[tokio::test]
async fn p1_raise_allowed() {
    let ctx = TestContext::with_p1_30m_vnd().await;
    let r = ctx.set_p1(ctx.member_id, dec!(35_000_000)).await;
    assert!(r.is_ok());
}

#[tokio::test]
async fn demotion_with_full_consent_allowed() {
    let ctx = TestContext::with_p1_30m_vnd().await;
    let consent = ctx.create_demotion_consent(ctx.member_id, dec!(25_000_000)).await;
    ctx.ceo_sign(consent.id).await;
    ctx.cfo_sign(consent.id).await;
    let r = ctx.set_p1(ctx.member_id, dec!(25_000_000)).await;
    assert!(r.is_ok());
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-REW-001.
**Cross-module:** FR-AUTH-101 (CEO/CFO roles), FR-DOC-001 (consent doc), FR-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Trigger fires | EXCEPTION | sev-2 audit; reject | get consent |
| Service guard fail | early return | 400 | inherent |
| Consent doc missing | FK NULL | reject | upload doc |
| Same-person dual-sign | validate | 403 | different signer |
| CHECK new == old | reject | 400 | use different value |
| Decryption fail in trigger | trigger error | sev-1 | KMS check |
| Cross-tenant consent | RLS | not found → block | inherent |
| Trigger bypass attempt | inherent in design | sev-1 | inherent |
| Decimal precision | bigint VND | inherent | inherent |
| Concurrent consent + raise | timing window | trigger uses latest | inherent |

## §11 — Implementation notes
- §11.1 Trigger uses helper function `decrypt_amount()` — security-definer or call-out to KMS via FDW.
- §11.2 Service validator pre-rejects for UX; trigger is final guarantee.
- §11.3 Demotion consent doc per VN Labour Code Art. 35 — written agreement scanned + FR-DOC-001.
- §11.4 memory audit body: member_id, change kind, attempted_violation flag; amounts SHA256.
- §11.5 P2/P3 reductions allowed (variable pay) — invariant scope is P1 only.

---

*End of FR-REW-003 spec.*
