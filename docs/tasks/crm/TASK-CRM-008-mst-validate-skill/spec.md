---
id: TASK-CRM-008
title: "CRM vietnam-mst-validate skill — synchronous GDT lookup on Account write to confirm MST format + entity name match"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: CRM
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 7
slice: 7
owner: Stephen Cheng (CRO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-CRM-003, TASK-SKILL-107, TASK-AI-003, TASK-MEMORY-111]
depends_on: [TASK-CRM-003]
blocks: []

source_pages:
  - website/docs/modules/crm.html#mst-validate
  - https://gdt.gov.vn/  # GDT TIN lookup API

source_decisions:
  - DEC-1680 2026-05-17 — Skill name: vietnam-mst-validate@1; called on Account create/update when vn_account_type set and residency=vn-1
  - DEC-1681 2026-05-17 — Closed enum `mst_validation_result` = {confirmed, name_mismatch, mst_not_found, gdt_unavailable, format_invalid}; cardinality 5
  - DEC-1682 2026-05-17 — Confirmation requires: MST format pass (TASK-CRM-003) + GDT registered entity name match (Levenshtein distance ≤3)
  - DEC-1683 2026-05-17 — Non-blocking: validation result stored; account write proceeds (CRO can fix later)
  - DEC-1684 2026-05-17 — Cache: GDT lookup cached 30 days (TIN data rarely changes); per-tenant cache namespace
  - DEC-1685 2026-05-17 — memory audit kinds: crm.mst_validation_invoked, crm.mst_validation_confirmed, crm.mst_validation_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/crm/
  new_files:
    - services/crm/src/vn/mst_validate_skill.rs
    - services/crm/src/vn/gdt_client.rs
    - services/crm/src/audit/mst_validation_events.rs
    - services/crm/tests/mst_validate_confirmed_test.rs
    - services/crm/tests/mst_validate_name_mismatch_test.rs
    - services/crm/tests/mst_validate_not_found_test.rs
    - services/crm/tests/mst_validate_cache_test.rs
    - services/crm/tests/mst_validate_enum_cardinality_test.rs

  modified_files:
    - services/crm/src/accounts.rs
    - services/crm/migrations/0008_mst_validation.sql

  allowed_tools:
    - file_read: services/{crm,skill}/**
    - file_write: services/crm/{src,tests,migrations}/**
    - bash: cd services/crm && cargo test mst_validate

  disallowed_tools:
    - block account save on validation fail (per DEC-1683 — non-blocking)
    - skip cache (per DEC-1684 — required to avoid GDT rate-limit)

effort_hours: 3
subtasks:
  - "0.2h: 0008_mst_validation.sql"
  - "0.5h: mst_validate_skill.rs"
  - "0.6h: gdt_client.rs (HTTP + 30d cache)"
  - "0.2h: audit/mst_validation_events.rs"
  - "0.2h: accounts.rs hook"
  - "1.0h: tests — 5 test files"
  - "0.3h: docs"

risk_if_skipped: "Without validation, invalid MSTs persist → TASK-INV-007 hóa đơn rejected by GDT at emit time. Without DEC-1683 non-blocking, account save fails when GDT unavailable (frustrating). Without DEC-1684 cache, GDT rate-limits us."
---

## §1 — Description (BCP-14 normative)

The CRM service **MUST** ship vietnam-mst-validate@1 skill at `services/crm/src/vn/mst_validate_skill.rs` calling GDT TIN lookup on VN account writes, non-blocking result, 30d cache, 3 memory audit kinds.

1. **MUST** register skill name `vietnam-mst-validate@1` per DEC-1680.

2. **MUST** hook into account create/update — if `residency='vn-1'` and `mst` set, invoke skill async.

3. **MUST** validate `mst_validation_result` against closed enum per DEC-1681.

4. **MUST** call GDT API at `gdt_client.rs::lookup(mst)` returning `{registered_name, entity_type, address}`.

5. **MUST** compare returned name with `account.name` per DEC-1682 — Levenshtein distance ≤3 → `confirmed`; else `name_mismatch`.

6. **MUST** cache lookup 30 days per DEC-1684 — key `mst:lookup:{mst}`, per-tenant namespace.

7. **MUST** be non-blocking per DEC-1683 — write proceeds; validation result stored separately.

8. **MUST** persist result at migration `0008`:
   ```sql
   CREATE TABLE crm_mst_validations (
     validation_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     account_id UUID NOT NULL,
     mst TEXT NOT NULL,
     result TEXT NOT NULL
       CHECK (result IN ('confirmed','name_mismatch','mst_not_found','gdt_unavailable','format_invalid')),
     gdt_returned_name TEXT,
     account_name_at_check TEXT,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   CREATE INDEX mst_validations_account_time_idx ON crm_mst_validations(tenant_id, account_id, created_at DESC);
   ALTER TABLE crm_mst_validations ENABLE ROW LEVEL SECURITY;
   CREATE POLICY mst_val_rls ON crm_mst_validations
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON crm_mst_validations FROM cyberos_app;
   -- Append-only
   ```

9. **MUST** emit 3 memory audit kinds per DEC-1685. PII per TASK-MEMORY-111: MST + names SHA256 hashed.

10. **MUST** thread trace_id from account hook → skill → GDT call → audit.

11. **MUST NOT** block account save per DEC-1683.

12. **MUST NOT** call GDT directly when cache hit per DEC-1684 — saves rate.

---

## §2 — Why this design

**Why non-blocking (DEC-1683)?** GDT can be slow/down; account save shouldn't fail. CRO sees `name_mismatch` flag on dashboard and resolves async.

**Why 30d cache (DEC-1684)?** TIN registry rarely changes; GDT rate-limits aggressive callers. 30d is industry standard.

**Why Levenshtein ≤3 (DEC-1682)?** Allows minor transcription variations (`Acme Corp` vs `Acme Corporation`) without false-positive mismatches.

**Why closed enum (DEC-1681)?** Bounded outcomes; gdt_unavailable distinct from mst_not_found is operationally important.

---

## §3 — API contract

```text
POST   /v1/crm/accounts/{id}/validate-mst    (manual re-trigger by CRO)
GET    /v1/crm/accounts/{id}/mst-validation  (latest result)
```

Sample result:
```json
{
  "validation_id": "uuid",
  "account_id": "uuid",
  "result": "name_mismatch",
  "gdt_returned_name": "Công ty TNHH Acme Việt Nam",
  "account_name_at_check": "Acme Corp",
  "checked_at": "2026-05-17T10:00:00Z"
}
```

---

## §4 — Acceptance criteria
1. **Skill registered as vietnam-mst-validate@1**. 2. **Hook on account create+update when vn-1 + mst set**. 3. **Non-blocking (account save succeeds even if GDT down)**. 4. **GDT API called for new MST**. 5. **Cache hit on repeat call within 30d**. 6. **Levenshtein ≤3 → confirmed**. 7. **>3 → name_mismatch**. 8. **404 from GDT → mst_not_found**. 9. **GDT 5xx → gdt_unavailable**. 10. **Format invalid → format_invalid (before HTTP)**. 11. **5-result enum + cardinality test**. 12. **3 memory audit kinds emitted**. 13. **PII scrubbed (MST+names SHA256)**. 14. **RLS denies cross-tenant**. 15. **Trace_id preserved**. 16. **Result row append-only**. 17. **Manual revalidate via POST**. 18. **History queryable (latest in GET)**. 19. **Account update doesn't block on pending validation**. 20. **GDT cache namespaced per-tenant**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn confirmed_when_name_matches() {
    let ctx = TestContext::vn_account("Acme Corp", "0312345678").await;
    ctx.gdt_returns("0312345678", "Acme Corp").await;
    ctx.update_account_mst(ctx.account_id, "0312345678").await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    let v = ctx.latest_validation(ctx.account_id).await;
    assert_eq!(v.result, "confirmed");
}

#[tokio::test]
async fn cache_avoids_second_gdt_call() {
    let ctx = TestContext::vn_account("Acme", "0312345678").await;
    ctx.gdt_returns("0312345678", "Acme").await;
    ctx.update_account_mst(ctx.account_id, "0312345678").await;
    ctx.update_account_mst(ctx.account_id, "0312345678").await;
    assert_eq!(ctx.gdt_call_count("0312345678").await, 1);
}

#[tokio::test]
async fn non_blocking_on_gdt_down() {
    let ctx = TestContext::vn_account("Acme", "0312345678").await;
    ctx.gdt_returns_5xx().await;
    let r = ctx.update_account_mst(ctx.account_id, "0312345678").await;
    assert!(r.is_ok());  // account update succeeded
    let v = ctx.latest_validation(ctx.account_id).await;
    assert_eq!(v.result, "gdt_unavailable");
}

// 5.4..5.8 — name_mismatch, mst_not_found, enum cardinality, audit
```

---

## §7 — Dependencies
**Upstream:** TASK-CRM-003.
**Cross-module:** TASK-SKILL-107 (skill registry), TASK-MEMORY-111 (PII), TASK-MCP-007 (async).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| GDT API down | client err | gdt_unavailable; non-blocking | retry async |
| GDT rate-limit | 429 | cache hit serves; sev-3 if no cache | inherent |
| MST format invalid | TASK-CRM-003 CHECK | format_invalid (no HTTP) | data fix |
| MST not in GDT registry | 404 | mst_not_found | account holder explains |
| Name match too liberal/strict | Levenshtein tuning | CRO can manually mark | configurable |
| Cache stale (entity name changed) | 30d TTL | refresh next check | inherent |
| Concurrent validation | OK (idempotent) | inherent | inherent |
| Account update during validation | non-blocking | inherent | next save re-validates if changed |
| Cross-tenant cache leak | namespaced key | inherent | inherent |
| Hook fires on non-vn account | early skip | no-op | inherent |

## §11 — Implementation notes
- §11.1 GDT API: `GET https://hoadondientu.gdt.gov.vn/api/tin-lookup/{mst}` (placeholder URL — confirm at integration).
- §11.2 Cache via Redis: `mst:lookup:{tenant_id}:{mst}` TTL 2592000s.
- §11.3 Levenshtein via `strsim` crate; normalize case + strip company suffixes ("Corp"/"Inc"/"TNHH") before compare.
- §11.4 memory audit body: account_id, result enum; MST + names SHA256.
- §11.5 Manual revalidate ignores cache (force fresh GDT call).

---

*End of TASK-CRM-008 spec.*
