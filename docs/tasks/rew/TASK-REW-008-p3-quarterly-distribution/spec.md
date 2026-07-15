---
id: TASK-REW-008
title: "REW quarterly P3 distribution from BP fund — CEO+CFO sign-off + LEARN-007 VP share splits + debit BP balances"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: REW
priority: p0
status: draft
verify: T
phase: P2
milestone: P2 · slice 2
slice: 2
owner: Stephen Cheng (CEO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-REW-007, TASK-LEARN-007, TASK-MCP-007, TASK-MEMORY-111]
depends_on: [TASK-REW-007]
blocks: []

source_pages:
  - website/docs/modules/rew.html#p3-distribution

source_decisions:
  - DEC-2220 2026-05-17 — Quarterly distribution: CEO sets fund VND amount, TASK-LEARN-007 emits VP shares, system computes per-member payout, BP ledger debited, P3 added to next payroll
  - DEC-2221 2026-05-17 — Closed enum `distribution_status` = {drafted, ceo_signed, cfo_signed, executed, paid, dismissed}; cardinality 6
  - DEC-2222 2026-05-17 — Dual-sign CEO + CFO required for execute; same-person rejected
  - DEC-2223 2026-05-17 — Idempotent per (tenant, quarter); UNIQUE
  - DEC-2224 2026-05-17 — memory audit kinds: rew.p3_distribution_drafted, rew.p3_distribution_signed, rew.p3_distribution_executed, rew.p3_distribution_paid, rew.p3_distribution_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/rew/
  new_files:
    - services/rew/migrations/0008_p3_distributions.sql
    - services/rew/src/p3/mod.rs
    - services/rew/src/p3/calculator.rs
    - services/rew/src/p3/dual_sign_gate.rs
    - services/rew/src/handlers/p3_routes.rs
    - services/rew/src/audit/p3_events.rs
    - services/rew/tests/p3_distribution_status_enum_cardinality_test.rs
    - services/rew/tests/p3_dual_sign_test.rs
    - services/rew/tests/p3_vp_share_application_test.rs
    - services/rew/tests/p3_idempotent_test.rs
    - services/rew/tests/p3_audit_emission_test.rs

  modified_files:
    - services/rew/src/lib.rs

  allowed_tools:
    - file_read: services/{rew,learn}/**
    - file_write: services/rew/{src,tests,migrations}/**
    - bash: cd services/rew && cargo test p3

  disallowed_tools:
    - execute without dual-sign (per DEC-2222)
    - duplicate quarter (per DEC-2223)

effort_hours: 6
subtasks:
  - "0.3h: 0008_p3_distributions.sql"
  - "0.3h: p3/mod.rs"
  - "0.6h: calculator.rs"
  - "0.4h: dual_sign_gate.rs"
  - "0.4h: handlers/p3_routes.rs"
  - "0.3h: audit/p3_events.rs"
  - "2.4h: tests — 5 test files"
  - "1.0h: CEO+CFO UI for review + sign"
  - "0.3h: docs"

risk_if_skipped: "Without P3 distribution, BP balances persist unredeemed → demotivation. Without DEC-2222 dual-sign, single-signer payouts. Without DEC-2223 idempotency, duplicate quarter payouts."
---

## §1 — Description (BCP-14 normative)

The REW service **MUST** ship quarterly P3 distribution at `services/rew/src/p3/` consuming TASK-LEARN-007 VP shares + dual-sign + BP debit + P3 add to next payroll, 5 memory audit kinds.

1. **MUST** validate `distribution_status` against closed enum per DEC-2221.

2. **MUST** calculate at `calculator.rs::calc(tenant, quarter, fund_vnd)` per DEC-2220:
   - Receive VP shares from TASK-LEARN-007 handoff (sums to 1.0)
   - Per member: payout_vnd = fund_vnd * vp_share

3. **MUST** require CEO + CFO dual-sign at `dual_sign_gate.rs::can_execute(distribution)` per DEC-2222 — same-person rejected.

4. **MUST** be idempotent per DEC-2223 — UNIQUE(tenant, quarter).

5. **MUST** on execute:
   - Debit BP balance (TASK-REW-007 credit_p3_distribution converted to BP-equiv)
   - Add to next payroll (TASK-REW-005) as P3 income kind

6. **MUST** define tables at migration `0008`:
   ```sql
   CREATE TABLE rew_p3_distributions (
     distribution_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     quarter CHAR(7) NOT NULL,
     fund_vnd BIGINT NOT NULL CHECK (fund_vnd > 0),
     status TEXT NOT NULL DEFAULT 'drafted'
       CHECK (status IN ('drafted','ceo_signed','cfo_signed','executed','paid','dismissed')),
     ceo_signed_by UUID,
     ceo_signed_at TIMESTAMPTZ,
     cfo_signed_by UUID,
     cfo_signed_at TIMESTAMPTZ,
     executed_at TIMESTAMPTZ,
     paid_at TIMESTAMPTZ,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     UNIQUE (tenant_id, quarter)
   );
   ALTER TABLE rew_p3_distributions ENABLE ROW LEVEL SECURITY;
   CREATE POLICY p3_dist_rls ON rew_p3_distributions
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON rew_p3_distributions FROM cyberos_app;
   GRANT UPDATE (status, ceo_signed_by, ceo_signed_at, cfo_signed_by, cfo_signed_at, executed_at, paid_at) ON rew_p3_distributions TO cyberos_app;

   CREATE TABLE rew_p3_member_payouts (
     payout_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     distribution_id UUID NOT NULL REFERENCES rew_p3_distributions(distribution_id),
     member_id UUID NOT NULL,
     vp_share NUMERIC(10,9) NOT NULL,
     payout_vnd BIGINT NOT NULL,
     PRIMARY KEY_ALT UNIQUE (distribution_id, member_id)
   );
   ALTER TABLE rew_p3_member_payouts ENABLE ROW LEVEL SECURITY;
   CREATE POLICY payouts_rls ON rew_p3_member_payouts
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON rew_p3_member_payouts FROM cyberos_app;
   ```

7. **MUST** expose endpoints:
   ```text
   POST /v1/rew/p3-distributions                  (CEO drafts; provides fund_vnd)
   POST /v1/rew/p3-distributions/{id}/ceo-sign
   POST /v1/rew/p3-distributions/{id}/cfo-sign
   POST /v1/rew/p3-distributions/{id}/execute     (auto if both signed)
   GET  /v1/rew/p3-distributions/{id}             (status + payouts)
   ```

8. **MUST** emit 5 memory audit kinds per DEC-2224. PII per TASK-MEMORY-111: payout amounts SHA256.

9. **MUST** thread trace_id from draft → sign → execute → audit.

10. **MUST NOT** execute without dual-sign per DEC-2222.

11. **MUST NOT** duplicate quarter per DEC-2223.

---

## §2 — Why this design

**Why VP share input (DEC-2220)?** Fairness — distribution proportional to contribution per TASK-LEARN-007.

**Why CEO + CFO dual-sign (DEC-2222)?** CEO sets strategic fund; CFO confirms financial OK.

**Why idempotent (DEC-2223)?** Quarter is single financial event; double-pay = disaster.

---

## §3 — API contract

Sample distribution draft:
```json
POST /v1/rew/p3-distributions
{
  "quarter": "2026-Q2",
  "fund_vnd": 500000000
}
```

Sample executed status:
```json
{
  "distribution_id": "uuid",
  "quarter": "2026-Q2",
  "fund_vnd": 500000000,
  "status": "executed",
  "payouts": [
    {"member_id": "uuid", "vp_share": 0.085, "payout_vnd": 42500000},
    {"member_id": "uuid", "vp_share": 0.062, "payout_vnd": 31000000}
  ]
}
```

---

## §4 — Acceptance criteria
1. **distribution_status enum cardinality 6**. 2. **fund_vnd > 0 CHECK**. 3. **VP shares from TASK-LEARN-007**. 4. **CEO+CFO dual-sign required**. 5. **Same-person rejected**. 6. **UNIQUE(tenant, quarter)**. 7. **5 memory audit kinds emitted**. 8. **PII scrubbed (amounts SHA256)**. 9. **RLS denies cross-tenant**. 10. **CEO-only draft**. 11. **Trace_id preserved**. 12. **Append-only via REVOKE except status cols**. 13. **bigint VND**. 14. **vp_share NUMERIC(10,9)**. 15. **Per-member payout row**. 16. **Execute debits BP ledger**. 17. **P3 added to next payroll**. 18. **Status workflow enforced**. 19. **Dismiss allowed pre-execute**. 20. **Sum of payouts ≈ fund_vnd (±1 VND tolerance)**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn dual_sign_required_for_execute() {
    let ctx = TestContext::with_drafted_distribution().await;
    ctx.ceo_sign(ctx.dist_id).await;
    let r = ctx.try_execute(ctx.dist_id).await;
    assert!(r.is_err());  // CFO missing
    ctx.cfo_sign(ctx.dist_id).await;
    let r2 = ctx.execute(ctx.dist_id).await;
    assert!(r2.is_ok());
}

#[tokio::test]
async fn payouts_sum_matches_fund() {
    let ctx = TestContext::with_executed_distribution(dec!(500_000_000)).await;
    let payouts = ctx.fetch_payouts(ctx.dist_id).await;
    let total: i64 = payouts.iter().map(|p| p.payout_vnd).sum();
    assert!((total - 500_000_000).abs() <= 1);
}

#[tokio::test]
async fn idempotent_quarter() {
    let ctx = TestContext::with_q1_distribution().await;
    let r = ctx.try_draft_distribution("2026-Q1").await;
    assert!(r.is_err());  // UNIQUE
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-REW-007.
**Cross-module:** TASK-LEARN-007 (VP shares), TASK-REW-005 (payroll injection), TASK-AUTH-101 (CEO/CFO), TASK-MCP-007 (trigger cron), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| TASK-LEARN-007 not emitted | catch | sev-2; await | manual trigger |
| One signer missing | gate | reject execute | wait |
| Same-person dual-sign | validate | 403 | different signer |
| Sum of payouts != fund | post-condition | sev-1; reject | bug fix |
| Duplicate quarter | UNIQUE | 409 | use different |
| BP debit fail | rollback | sev-1 | retry |
| Payroll injection fail | sev-1 | manual fix | inherent |
| Cross-tenant query | RLS | 0 rows | inherent |
| Decimal precision | bigint VND + rust_decimal share | inherent | inherent |
| Concurrent execute | UPDATE WHERE pending | first wins | inherent |

## §11 — Implementation notes
- §11.1 Calculator: per_member_vnd = fund_vnd * vp_share; round to nearest VND; allocate residual to highest-share member to ensure sum matches.
- §11.2 BP debit converts VND to BP-equiv via TASK-HR-005 policy `bp_vnd_rate`.
- §11.3 memory audit body: distribution_id, quarter, members_count; amounts SHA256.
- §11.4 Cron trigger: quarter+1d via TASK-MCP-007 reminding CEO to draft.
- §11.5 Drafted state allows CEO to revise fund_vnd before signing.

---

*End of TASK-REW-008 spec.*
