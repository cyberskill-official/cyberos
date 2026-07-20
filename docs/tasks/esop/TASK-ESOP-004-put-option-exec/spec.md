---
id: TASK-ESOP-004
title: "ESOP put-option exec flow — Year 3+ eligibility + per-Member annual cap + CFO approve + bank wire via TASK-INV-005"
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
module: ESOP
priority: p0
status: draft
verify: T
phase: P2
milestone: P2 · slice 2
slice: 2
owner: Stephen Cheng (CFO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-ESOP-003, TASK-ESOP-002, TASK-INV-005, TASK-AUTH-101, TASK-MEMORY-111]
depends_on: [TASK-ESOP-003, TASK-INV-005]
blocks: []

source_pages:
  - website/docs/modules/esop.html#put-option

source_decisions:
  - DEC-2280 2026-05-17 — Member can exercise put-option (sell vested shares back to company) starting Year 3 of grant; per-year cap (default 25% of vested per year)
  - DEC-2281 2026-05-17 — Closed enum `put_status` = {requested, cfo_pending, cfo_approved, cfo_rejected, wire_initiated, paid, failed}; cardinality 7
  - DEC-2282 2026-05-17 — Exec price = vested_shares_exercised × TASK-ESOP-003 committed_share_price (current year)
  - DEC-2283 2026-05-17 — Cap enforced: sum(exercised in calendar year) ≤ cap_pct * vested_at_year_start
  - DEC-2284 2026-05-17 — memory audit kinds: esop.put_requested, esop.put_cfo_approved, esop.put_cfo_rejected, esop.put_wire_initiated, esop.put_paid, esop.put_failed

language: rust 1.81
service: cyberos/services/esop/
new_files:
  - services/esop/migrations/0004_put_options.sql
  - services/esop/src/put/mod.rs
  - services/esop/src/put/eligibility.rs
  - services/esop/src/put/price_calculator.rs
  - services/esop/src/put/cap_enforcer.rs
  - services/esop/src/put/wire_initiator.rs
  - services/esop/src/handlers/put_routes.rs
  - services/esop/src/audit/put_events.rs
  - services/esop/tests/put_status_enum_cardinality_test.rs
  - services/esop/tests/put_year_3_eligibility_test.rs
  - services/esop/tests/put_annual_cap_test.rs
  - services/esop/tests/put_price_calc_test.rs
  - services/esop/tests/put_wire_integration_test.rs
  - services/esop/tests/put_audit_emission_test.rs

modified_files:
  - services/esop/src/lib.rs

allowed_tools:
  - file_read: services/{esop,inv}/**
  - file_write: services/esop/{src,tests,migrations}/**
  - bash: cd services/esop && cargo test put

disallowed_tools:
  - approve before Year 3 (per DEC-2280)
  - exceed annual cap (per DEC-2283)

effort_hours: 8
subtasks:
  - "0.4h: 0004_put_options.sql"
  - "0.3h: put/mod.rs"
  - "0.5h: eligibility.rs"
  - "0.4h: price_calculator.rs"
  - "0.5h: cap_enforcer.rs"
  - "0.6h: wire_initiator.rs"
  - "0.5h: handlers/put_routes.rs"
  - "0.4h: audit/put_events.rs"
  - "2.8h: tests — 6 test files"
  - "1.6h: Member request UI + CFO approve UI"

risk_if_skipped: "Without put-option, vested members illiquid → retention damage. Without DEC-2280 Year 3 gate, members cash out early breaking incentive. Without DEC-2283 cap, single member drains liquidity."
---

## §1 — Description (BCP-14 normative)

The ESOP service **MUST** ship put-option exec at `services/esop/src/put/` with Year 3 eligibility + annual cap + CFO approve + TASK-INV-005 wire, 6 memory audit kinds.

1. **MUST** validate `put_status` against closed enum per DEC-2281.

2. **MUST** check eligibility at `eligibility.rs::is_eligible(grant)` per DEC-2280:
- grant.vest_start_date + 3 years ≤ now
- grant.status IN (active, fully_vested)

3. **MUST** compute price at `price_calculator.rs::price(shares, year)` per DEC-2282:
- Read TASK-ESOP-003 committed price for current calendar year
- amount = shares × committed_price

4. **MUST** enforce cap at `cap_enforcer.rs::check(member, requested_shares, year)` per DEC-2283:
- vested_at_year_start = TASK-ESOP-002 accrual at Jan 1
- sum(prior exercised this year) + requested ≤ cap_pct × vested_at_year_start
- default cap_pct = 0.25 (configurable per tenant)

5. **MUST** require CFO approve before wire.

6. **MUST** wire via TASK-INV-005 to member's bank account.

7. **MUST** define table at migration `0004`:
   ```sql
   CREATE TABLE esop_put_requests (
     put_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     member_id UUID NOT NULL,
     grant_id UUID NOT NULL REFERENCES esop_sp_grants(grant_id),
     shares_requested BIGINT NOT NULL CHECK (shares_requested > 0),
     valuation_id UUID NOT NULL REFERENCES esop_annual_valuations(valuation_id),
     amount_vnd BIGINT NOT NULL,
     status TEXT NOT NULL DEFAULT 'requested'
       CHECK (status IN ('requested','cfo_pending','cfo_approved','cfo_rejected','wire_initiated','paid','failed')),
     cfo_approved_by UUID,
     cfo_approved_at TIMESTAMPTZ,
     cfo_rejected_reason TEXT,
     wire_initiated_at TIMESTAMPTZ,
     paid_at TIMESTAMPTZ,
     inv_payment_id UUID,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   CREATE INDEX put_member_year_idx ON esop_put_requests(tenant_id, member_id, EXTRACT(YEAR FROM created_at));
   ALTER TABLE esop_put_requests ENABLE ROW LEVEL SECURITY;
   CREATE POLICY puts_rls ON esop_put_requests
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON esop_put_requests FROM cyberos_app;
   GRANT UPDATE (status, cfo_approved_by, cfo_approved_at, cfo_rejected_reason, wire_initiated_at, paid_at, inv_payment_id) ON esop_put_requests TO cyberos_app;
   ```

8. **MUST** expose endpoints:
   ```text
   POST /v1/esop/puts                       (member-self requests)
   POST /v1/esop/puts/{id}/approve          (CFO)
   POST /v1/esop/puts/{id}/reject           body: {reason}
   GET  /v1/esop/puts/{id}                  (status)
   GET  /v1/esop/members/{id}/puts          (history; member-self or CFO)
   ```

9. **MUST** emit 6 memory audit kinds per DEC-2284. PII per TASK-MEMORY-111: amount + shares SHA256.

10. **MUST** thread trace_id from request → approve → wire → audit.

11. **MUST NOT** approve before Year 3 per DEC-2280.

12. **MUST NOT** exceed annual cap per DEC-2283.

---

## §2 — Why this design

**Why Year 3 (DEC-2280)?** Industry standard — gives incentive period for retention before liquidity option.

**Why annual cap (DEC-2283)?** Without cap, single executor could drain liquidity; tenant-controlled.

**Why CFO approve (DEC-2280)?** Cash outflow event; CFO controls treasury.

---

## §3 — API contract

Sample put request:
```json
POST /v1/esop/puts
{
  "grant_id": "uuid",
  "shares_requested": 500
}
```

Sample response:
```json
{
  "put_id": "uuid",
  "shares_requested": 500,
  "amount_vnd": 25000000,
  "status": "cfo_pending"
}
```

---

## §4 — Acceptance criteria
1. **put_status enum cardinality 7**. 2. **Year 3+ eligibility enforced**. 3. **Cap 25% annual default**. 4. **Cap configurable per tenant**. 5. **Price from TASK-ESOP-003 committed**. 6. **CFO approve required**. 7. **TASK-INV-005 wire integration**. 8. **6 memory audit kinds emitted**. 9. **PII scrubbed (shares + amount SHA256)**. 10. **RLS denies cross-tenant**. 11. **Member-self request only**. 12. **CFO-only approve/reject**. 13. **Trace_id preserved**. 14. **Append-only via REVOKE except status cols**. 15. **bigint VND + shares**. 16. **shares_requested > 0**. 17. **Rejection reason logged**. 18. **Cap considers prior YTD exercises**. 19. **vested_at_year_start from TASK-ESOP-002 Jan 1 accrual**. 20. **Wire failure → status=failed + sev-1**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn year_3_eligibility_blocks_early() {
    let g = ctx.grant_active("2026-01-01", 48, 12, 10000).await;
    let r = ctx.try_request_put(g.id, 100).await;
    assert!(r.is_err());  // <3 years
}

#[tokio::test]
async fn annual_cap_enforced() {
    let ctx = TestContext::with_grant_3y_old_vested_4000().await;  // 25% = 1000
    let r1 = ctx.request_put(ctx.grant_id, 600).await;
    ctx.cfo_approve(r1.id).await;
    let r2 = ctx.try_request_put(ctx.grant_id, 500).await;  // 600+500=1100 > 1000
    assert!(r2.is_err());
}

#[tokio::test]
async fn wire_initiated_via_inv_005() {
    let ctx = TestContext::with_approved_put().await;
    ctx.initiate_wire(ctx.put_id).await;
    let p = ctx.fetch_put(ctx.put_id).await;
    assert!(p.inv_payment_id.is_some());
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-ESOP-003, TASK-INV-005. **Cross-module:** TASK-ESOP-002 (vested at Jan 1), TASK-AUTH-101 (CFO role), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Pre-Year 3 attempt | eligibility | reject | wait |
| Cap exceeded | enforcer | reject | reduce shares |
| No valuation for year | check | sev-1 | propose valuation |
| Wire fail | catch | status=failed; sev-1 | retry |
| Cross-tenant put | RLS | 403 | inherent |
| Grant cancelled | eligibility | reject | inherent |
| Concurrent put | inherent | each evaluated | inherent |
| Bank invalid acct | wire fail | sev-2 | member update |
| CFO rejection | inherent | status=cfo_rejected | inherent |
| Decimal precision | bigint | inherent | inherent |

## §11 — Implementation notes
- §11.1 Cap_pct stored per tenant config; default 0.25.
- §11.2 vested_at_year_start = TASK-ESOP-002 accrual for Jan 1 of current year.
- §11.3 memory audit body: put_id, member_id, grant_id, status; shares + amount SHA256.
- §11.4 Wire via TASK-INV-005 with memo: `ESOP-PUT-{put_id_8}` for reconciliation.
- §11.5 Future: extend to fractional liquidity rounds (multiple-CFO approval).

---

*End of TASK-ESOP-004 spec.*
