---
id: FR-ESOP-001
title: "ESOP SP grant schema — Stock Plan grant with 4-year vesting + 12-month cliff default + per-grant immutable params"
module: ESOP
priority: MUST
status: ready_to_implement
verify: T
phase: P2
milestone: P2 · slice 1
slice: 1
owner: Stephen Cheng (CEO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-HR-001, FR-ESOP-002, FR-ESOP-003, FR-ESOP-004, FR-MEMORY-111]
depends_on: [FR-HR-001]
blocks: [FR-ESOP-002, FR-ESOP-003, FR-ESOP-006, FR-ESOP-007, FR-TEN-201]

source_pages:
  - website/docs/modules/esop.html#grant-schema

source_decisions:
  - DEC-2250 2026-05-17 — Per-Member SP grant with vesting params (vest_months default 48, cliff_months default 12, total_shares); IMMUTABLE post-grant
  - DEC-2251 2026-05-17 — Closed enum `grant_kind` = {founder, employee_initial, employee_refresher, advisor, board}; cardinality 5
  - DEC-2252 2026-05-17 — Closed enum `grant_status` = {pending_signing, active, fully_vested, cancelled_unvested, accelerated}; cardinality 5
  - DEC-2253 2026-05-17 — Grant date locked at creation; vest_start_date may differ (e.g. start of employment, signed sheet)
  - DEC-2254 2026-05-17 — memory audit kinds: esop.grant_created, esop.grant_signed, esop.grant_cancelled, esop.grant_accelerated, esop.grant_fully_vested

build_envelope:
  language: rust 1.81
  service: cyberos/services/esop/
  new_files:
    - services/esop/migrations/0001_sp_grants.sql
    - services/esop/src/grant/mod.rs
    - services/esop/src/grant/validator.rs
    - services/esop/src/handlers/grant_routes.rs
    - services/esop/src/audit/grant_events.rs
    - services/esop/tests/grant_kind_enum_cardinality_test.rs
    - services/esop/tests/grant_status_enum_cardinality_test.rs
    - services/esop/tests/grant_immutability_test.rs
    - services/esop/tests/grant_vesting_params_default_test.rs
    - services/esop/tests/grant_audit_emission_test.rs

  modified_files:
    - services/esop/src/lib.rs

  allowed_tools:
    - file_read: services/{esop,hr}/**
    - file_write: services/esop/{src,tests,migrations}/**
    - bash: cd services/esop && cargo test grant

  disallowed_tools:
    - mutate prior grant (per DEC-2250)
    - skip CEO sign on creation

effort_hours: 5
sub_tasks:
  - "0.3h: 0001_sp_grants.sql"
  - "0.3h: grant/mod.rs"
  - "0.5h: validator.rs"
  - "0.4h: handlers/grant_routes.rs"
  - "0.3h: audit/grant_events.rs"
  - "2.0h: tests — 5 test files"
  - "1.2h: docs + CEO UI for grant"

risk_if_skipped: "Without grant schema, equity ad-hoc. Without DEC-2250 immutability, retroactive vesting changes break trust. Without DEC-2253 vest_start_date, late-signing grants accrue from wrong date."
---

## §1 — Description (BCP-14 normative)

The ESOP service **MUST** ship SP grant schema at `services/esop/src/grant/` with 5-kind enum + immutable params + status lifecycle, 5 memory audit kinds.

1. **MUST** validate `grant_kind` against closed enum per DEC-2251, `grant_status` per DEC-2252.

2. **MUST** define table at migration `0001`:
   ```sql
   CREATE TABLE esop_sp_grants (
     grant_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     member_id UUID NOT NULL,
     kind TEXT NOT NULL
       CHECK (kind IN ('founder','employee_initial','employee_refresher','advisor','board')),
     total_shares BIGINT NOT NULL CHECK (total_shares > 0),
     vest_months INT NOT NULL DEFAULT 48 CHECK (vest_months > 0),
     cliff_months INT NOT NULL DEFAULT 12 CHECK (cliff_months >= 0 AND cliff_months <= vest_months),
     strike_price_vnd BIGINT NOT NULL CHECK (strike_price_vnd >= 0),
     grant_date DATE NOT NULL,
     vest_start_date DATE NOT NULL,
     status TEXT NOT NULL DEFAULT 'pending_signing'
       CHECK (status IN ('pending_signing','active','fully_vested','cancelled_unvested','accelerated')),
     granted_by UUID NOT NULL,
     ceo_signed_at TIMESTAMPTZ,
     member_signed_at TIMESTAMPTZ,
     activated_at TIMESTAMPTZ,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   CREATE INDEX grants_member_idx ON esop_sp_grants(tenant_id, member_id, created_at DESC);
   ALTER TABLE esop_sp_grants ENABLE ROW LEVEL SECURITY;
   CREATE POLICY grants_rls ON esop_sp_grants
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON esop_sp_grants FROM cyberos_app;
   GRANT UPDATE (status, ceo_signed_at, member_signed_at, activated_at) ON esop_sp_grants TO cyberos_app;
   ```

3. **MUST** require CEO sign + member sign before status=active per DEC-2253.

4. **MUST** expose endpoints:
   ```text
   POST /v1/esop/grants                       (CEO creates)
   POST /v1/esop/grants/{id}/ceo-sign
   POST /v1/esop/grants/{id}/member-sign      (member self)
   POST /v1/esop/grants/{id}/cancel           (CEO; pre-cliff only)
   GET  /v1/esop/grants/{id}                   (member-self or CFO/CEO)
   ```

5. **MUST** emit 5 memory audit kinds per DEC-2254. PII per FR-MEMORY-111: total_shares SHA256; member_id (uuid) ok.

6. **MUST** thread trace_id from create → sign → activate → audit.

7. **MUST NOT** mutate prior grant params per DEC-2250 (REVOKE UPDATE except 4 status cols).

8. **MUST NOT** activate without both CEO + member signs.

---

## §2 — Why this design

**Why 5 kinds (DEC-2251)?** Covers founders, regular employees (initial + refresh), advisors, board — bounded.

**Why default 4y + 12mo cliff (DEC-2250)?** Industry standard for early-stage equity.

**Why vest_start_date != grant_date (DEC-2253)?** Late-signing grants back-date to employment start for fairness.

**Why immutable (DEC-2250)?** Equity grants = legal commitment; mutation = fraud risk.

---

## §3 — API contract

Sample grant:
```json
POST /v1/esop/grants
{
  "member_id": "uuid",
  "kind": "employee_initial",
  "total_shares": 10000,
  "vest_months": 48,
  "cliff_months": 12,
  "strike_price_vnd": 1000,
  "grant_date": "2026-05-17",
  "vest_start_date": "2026-01-15"
}
```

---

## §4 — Acceptance criteria
1. **grant_kind enum cardinality 5**. 2. **grant_status enum cardinality 5**. 3. **Total_shares > 0 CHECK**. 4. **Vest_months > 0 CHECK**. 5. **Cliff_months ≤ vest_months CHECK**. 6. **Strike_price_vnd ≥ 0**. 7. **Defaults vest=48, cliff=12**. 8. **CEO + member sign required for activate**. 9. **5 memory audit kinds emitted**. 10. **PII scrubbed (total_shares SHA256)**. 11. **RLS denies cross-tenant**. 12. **CEO-only create + cancel**. 13. **Trace_id preserved**. 14. **Append-only via REVOKE except 4 status cols**. 15. **Cancel allowed pre-cliff only**. 16. **bigint shares**. 17. **vest_start_date locked at creation**. 18. **status workflow: pending_signing → active → fully_vested | cancelled | accelerated**. 19. **Member-self can view own**. 20. **Cross-member view requires CFO audit (FR-ESOP-007)**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn defaults_4y_12mo_cliff() {
    let g = ctx.create_grant_minimal(ctx.member_id, 10000).await;
    assert_eq!(g.vest_months, 48);
    assert_eq!(g.cliff_months, 12);
}

#[tokio::test]
async fn dual_sign_to_activate() {
    let g = ctx.create_grant(ctx.member_id, 10000).await;
    ctx.ceo_sign(g.id).await;
    assert_eq!(ctx.fetch_grant(g.id).await.status, "pending_signing");
    ctx.member_sign(g.id).await;
    assert_eq!(ctx.fetch_grant(g.id).await.status, "active");
}

#[tokio::test]
async fn immutable_post_create() {
    let g = ctx.create_grant(...).await;
    let r = ctx.try_update_total_shares(g.id, 20000).await;
    assert!(r.is_err());
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-HR-001.
**Downstream:** FR-ESOP-002 (vesting), FR-ESOP-003 (valuation), FR-ESOP-004 (put-option), FR-ESOP-005 (GL/BL).
**Cross-module:** FR-AUTH-101 (CEO role), FR-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Kind invalid | CHECK | 400 | use valid |
| Total_shares ≤ 0 | CHECK | 400 | use positive |
| Cliff > vest | CHECK | 400 | reduce cliff |
| Mutation attempt | REVOKE | DB error | inherent |
| Cancel post-cliff | validate | 409 | use FR-ESOP-005 |
| Cross-tenant grant | RLS | 0 rows | inherent |
| Vest_start in distant past | warn | allow (audit visible) | inherent |
| Member declines | manual | inherent | re-grant |
| Concurrent sign | inherent | last-writer-wins for sign timestamps | inherent |
| Bigint overflow | bigint VND | inherent | inherent |

## §11 — Implementation notes
- §11.1 Vest start may precede grant_date (back-dating common for late paperwork).
- §11.2 Status lifecycle: pending_signing → active (after dual-sign) → fully_vested (FR-ESOP-002 cron) or cancelled (FR-ESOP-005 BL).
- §11.3 memory audit body: grant_id, member_id, kind, status; share counts SHA256.
- §11.4 Cancel allowed only pre-cliff — post-cliff requires FR-ESOP-005 GL/BL flow.
- §11.5 Member sign captured separately (legal vs operational); both required.

---

*End of FR-ESOP-001 spec.*
