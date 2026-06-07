---
id: FR-ESOP-003
title: "ESOP annual valuation — CFO base + Board multiplier sign-off with immutable share-price snapshot per year"
module: ESOP
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
related_frs: [FR-ESOP-001, FR-ESOP-004, FR-AUTH-101, FR-MEMORY-111]
depends_on: [FR-ESOP-001]
blocks: [FR-ESOP-004]

source_pages:
  - website/docs/modules/esop.html#annual-valuation

source_decisions:
  - DEC-2270 2026-05-17 — Annual valuation: CFO proposes base share price VND, Board approves multiplier (typically 1.0); committed = base × multiplier; IMMUTABLE per year
  - DEC-2271 2026-05-17 — Closed enum `valuation_status` = {drafted, cfo_proposed, board_approved, dismissed}; cardinality 4
  - DEC-2272 2026-05-17 — Board approval requires ≥3 board member sign-offs (configurable per tenant; default 3 of 5)
  - DEC-2273 2026-05-17 — UNIQUE per (tenant, year); corrections via new valuation row with explicit "correction_of" link
  - DEC-2274 2026-05-17 — memory audit kinds: esop.valuation_proposed, esop.valuation_board_signed, esop.valuation_committed, esop.valuation_dismissed, esop.valuation_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/esop/
  new_files:
    - services/esop/migrations/0003_annual_valuations.sql
    - services/esop/src/valuation/mod.rs
    - services/esop/src/valuation/board_sign_gate.rs
    - services/esop/src/handlers/valuation_routes.rs
    - services/esop/src/audit/valuation_events.rs
    - services/esop/tests/valuation_status_enum_cardinality_test.rs
    - services/esop/tests/valuation_board_threshold_test.rs
    - services/esop/tests/valuation_unique_per_year_test.rs
    - services/esop/tests/valuation_immutability_test.rs
    - services/esop/tests/valuation_audit_emission_test.rs

  modified_files:
    - services/esop/src/lib.rs

  allowed_tools:
    - file_read: services/{esop,auth}/**
    - file_write: services/esop/{src,tests,migrations}/**
    - bash: cd services/esop && cargo test valuation

  disallowed_tools:
    - commit without board threshold (per DEC-2272)
    - mutate prior valuation (per DEC-2273)

effort_hours: 5
sub_tasks:
  - "0.3h: 0003_annual_valuations.sql"
  - "0.3h: valuation/mod.rs"
  - "0.5h: board_sign_gate.rs"
  - "0.4h: handlers/valuation_routes.rs"
  - "0.3h: audit/valuation_events.rs"
  - "1.9h: tests — 5 test files"
  - "1.0h: docs + UI for CFO + Board"
  - "0.3h: integration with FR-ESOP-004"

risk_if_skipped: "Without annual valuation, put-option exec price arbitrary. Without DEC-2272 board threshold, single board member could set price. Without DEC-2273 uniqueness, multiple-valuations-per-year ambiguity."
---

## §1 — Description (BCP-14 normative)

The ESOP service **MUST** ship annual valuation at `services/esop/src/valuation/` with CFO propose + Board ≥3-sign + immutable per-year, 5 memory audit kinds.

1. **MUST** validate `valuation_status` against closed enum per DEC-2271.

2. **MUST** define tables at migration `0003`:
   ```sql
   CREATE TABLE esop_annual_valuations (
     valuation_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     valuation_year INT NOT NULL,
     base_share_price_vnd BIGINT NOT NULL CHECK (base_share_price_vnd >= 0),
     board_multiplier NUMERIC(7,4) NOT NULL DEFAULT 1.0 CHECK (board_multiplier > 0),
     committed_share_price_vnd BIGINT,
     status TEXT NOT NULL DEFAULT 'drafted'
       CHECK (status IN ('drafted','cfo_proposed','board_approved','dismissed')),
     cfo_proposed_by UUID,
     cfo_proposed_at TIMESTAMPTZ,
     committed_at TIMESTAMPTZ,
     correction_of UUID REFERENCES esop_annual_valuations(valuation_id),
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     UNIQUE (tenant_id, valuation_year, correction_of)
   );
   ALTER TABLE esop_annual_valuations ENABLE ROW LEVEL SECURITY;
   CREATE POLICY valuations_rls ON esop_annual_valuations
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON esop_annual_valuations FROM cyberos_app;
   GRANT UPDATE (status, cfo_proposed_by, cfo_proposed_at, committed_at, committed_share_price_vnd) ON esop_annual_valuations TO cyberos_app;

   CREATE TABLE esop_valuation_board_signs (
     sign_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     valuation_id UUID NOT NULL REFERENCES esop_annual_valuations(valuation_id),
     board_member_id UUID NOT NULL,
     signed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     UNIQUE (valuation_id, board_member_id)
   );
   ALTER TABLE esop_valuation_board_signs ENABLE ROW LEVEL SECURITY;
   CREATE POLICY signs_rls ON esop_valuation_board_signs
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON esop_valuation_board_signs FROM cyberos_app;
   ```

3. **MUST** enforce board threshold per DEC-2272 at `board_sign_gate.rs::can_commit(valuation)`:
   - Count distinct sign rows
   - Threshold from tenant config (default 3)
   - Reach threshold → status auto-advance to board_approved + commit

4. **MUST** be unique per year per DEC-2273 — UNIQUE(tenant_id, valuation_year, correction_of) allows original + corrections.

5. **MUST** expose endpoints:
   ```text
   POST /v1/esop/valuations                       (CFO drafts/proposes)
   POST /v1/esop/valuations/{id}/board-sign       (board member self)
   POST /v1/esop/valuations/{id}/dismiss          (CFO; pre-board-approved)
   GET  /v1/esop/valuations/{year}                (current committed)
   ```

6. **MUST** emit 5 memory audit kinds per DEC-2274. PII per FR-MEMORY-111: share price SHA256.

7. **MUST** thread trace_id from propose → sign → commit → audit.

8. **MUST NOT** commit without threshold per DEC-2272.

9. **MUST NOT** mutate prior valuation per DEC-2273 (correction = new row).

---

## §2 — Why this design

**Why CFO + Board (DEC-2270)?** CFO has financial info; Board has governance authority — both required.

**Why ≥3 sign (DEC-2272)?** Majority approval for material decision; configurable per tenant board size.

**Why immutable + correction_of (DEC-2273)?** Audit lineage; corrections via new row preserve history.

---

## §3 — API contract

Sample valuation propose:
```json
POST /v1/esop/valuations
{
  "valuation_year": 2026,
  "base_share_price_vnd": 50000,
  "board_multiplier": 1.0
}
```

Sample committed:
```json
{
  "valuation_id": "uuid",
  "valuation_year": 2026,
  "base_share_price_vnd": 50000,
  "board_multiplier": 1.0,
  "committed_share_price_vnd": 50000,
  "status": "board_approved",
  "board_signs_count": 3
}
```

---

## §4 — Acceptance criteria
1. **valuation_status enum cardinality 4**. 2. **CFO propose required first**. 3. **Board ≥3 signs (configurable)**. 4. **Threshold reached → auto-commit**. 5. **base_share_price_vnd ≥ 0**. 6. **board_multiplier > 0**. 7. **committed = base × multiplier**. 8. **UNIQUE(tenant, year, correction_of)**. 9. **5 memory audit kinds emitted**. 10. **PII scrubbed (share price SHA256)**. 11. **RLS denies cross-tenant**. 12. **CFO-only propose**. 13. **Board member-only sign**. 14. **Trace_id preserved**. 15. **Append-only via REVOKE except status cols**. 16. **Correction via new row**. 17. **Dismiss pre-approval only**. 18. **bigint VND**. 19. **NUMERIC(7,4) multiplier**. 20. **Board threshold config per tenant**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn board_3_signs_auto_commits() {
    let ctx = TestContext::with_proposed_valuation().await;
    ctx.board_sign(ctx.board1, ctx.val_id).await;
    ctx.board_sign(ctx.board2, ctx.val_id).await;
    let v = ctx.fetch_valuation(ctx.val_id).await;
    assert_ne!(v.status, "board_approved");
    ctx.board_sign(ctx.board3, ctx.val_id).await;
    let v2 = ctx.fetch_valuation(ctx.val_id).await;
    assert_eq!(v2.status, "board_approved");
    assert_eq!(v2.committed_share_price_vnd, Some(v2.base_share_price_vnd * v2.board_multiplier));
}

#[tokio::test]
async fn correction_via_new_row() {
    let ctx = TestContext::with_committed_valuation().await;
    let corr = ctx.propose_correction(ctx.val_id, 60000).await;
    assert!(corr.is_ok());
    assert_eq!(corr.row.correction_of, Some(ctx.val_id));
}

#[tokio::test]
async fn duplicate_year_blocked() {
    let ctx = TestContext::with_committed_valuation_2026().await;
    let r = ctx.try_propose_valuation(2026, 50000).await;
    assert!(r.is_err());  // UNIQUE
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-ESOP-001.
**Downstream:** FR-ESOP-004 (put-option uses committed price).
**Cross-module:** FR-AUTH-101 (CFO + board roles), FR-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Threshold not reached | inherent | stays at cfo_proposed | get more signs |
| Same board sign twice | UNIQUE | second skip | inherent |
| Cross-tenant sign | RLS | 0 rows | inherent |
| Decimal precision | bigint + NUMERIC(7,4) | inherent | inherent |
| Negative price | CHECK | 400 | use positive |
| Multiplier ≤ 0 | CHECK | 400 | use positive |
| Concurrent commit attempt | UPDATE WHERE not committed | first wins | inherent |
| Dismiss post-approval | reject | 409 | use correction |
| Year mismatch | validate | 400 | use correct year |
| Board config missing | default 3 | inherent | tenant config |

## §11 — Implementation notes
- §11.1 Board threshold stored per tenant config (default 3 of 5).
- §11.2 Commit happens automatically on threshold reach — no separate "commit" endpoint.
- §11.3 memory audit body: valuation_id, year, status, signs_count; price SHA256.
- §11.4 committed_share_price computed at commit: base × multiplier; stored to avoid recompute.
- §11.5 Correction creates new row with correction_of link; UNIQUE allows multiple per year via this distinction.

---

*End of FR-ESOP-003 spec.*
