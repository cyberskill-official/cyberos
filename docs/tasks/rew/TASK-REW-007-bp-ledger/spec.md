---
id: TASK-REW-007
title: "REW BP (Bonus Points) ledger with ACB-rate interest accrual nightly + per-Member balance + immutable transaction log"
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
module: REW
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
related_tasks: [TASK-REW-001, TASK-REW-008, TASK-MCP-007, TASK-MEMORY-111]
depends_on: [TASK-REW-001]
blocks: [TASK-REW-008]

source_pages:
  - website/docs/modules/rew.html#bp-ledger

source_decisions:
  - DEC-2210 2026-05-17 — BP (Bonus Points) is internal currency for accruing P3 bonuses; ledger tracks per-Member balance with credit/debit txn log
  - DEC-2211 2026-05-17 — Closed enum `bp_txn_kind` = {credit_p3_accrual, credit_special_award, credit_interest_accrual, debit_p3_distribution, debit_correction}; cardinality 5
  - DEC-2212 2026-05-17 — Interest accrued nightly at ACB (Asia Commercial Bank) rate; rate fetched via TASK-HR-005 policy (interest_rate_bp)
  - DEC-2213 2026-05-17 — Ledger IMMUTABLE; corrections via debit_correction txn with explanatory reason
  - DEC-2214 2026-05-17 — memory audit kinds: rew.bp_credited, rew.bp_debited, rew.bp_interest_accrued, rew.bp_balance_query

language: rust 1.81
service: cyberos/services/rew/
new_files:
  - services/rew/migrations/0007_bp_ledger.sql
  - services/rew/src/bp/mod.rs
  - services/rew/src/bp/interest_cron.rs
  - services/rew/src/bp/balance_query.rs
  - services/rew/src/handlers/bp_routes.rs
  - services/rew/src/audit/bp_events.rs
  - services/rew/tests/bp_txn_kind_enum_cardinality_test.rs
  - services/rew/tests/bp_immutable_test.rs
  - services/rew/tests/bp_interest_accrual_test.rs
  - services/rew/tests/bp_balance_correctness_test.rs
  - services/rew/tests/bp_audit_emission_test.rs

modified_files:
  - services/rew/src/lib.rs

allowed_tools:
  - file_read: services/{rew,hr}/**
  - file_write: services/rew/{src,tests,migrations}/**
  - bash: cd services/rew && cargo test bp

disallowed_tools:
  - mutate prior txn (per DEC-2213)
  - skip interest accrual (per DEC-2212)

effort_hours: 5
subtasks:
  - "0.3h: 0007_bp_ledger.sql"
  - "0.3h: bp/mod.rs"
  - "0.5h: interest_cron.rs"
  - "0.4h: balance_query.rs"
  - "0.4h: handlers/bp_routes.rs"
  - "0.3h: audit/bp_events.rs"
  - "2.0h: tests — 5 test files"
  - "0.6h: docs + cron registration"
  - "0.2h: balance UI"

risk_if_skipped: "Without BP ledger, P3 bonus pool unmanaged. Without DEC-2213 immutability, balances rewritable (audit fail). Without DEC-2212 interest, BP stagnates (incentive lost)."
---

## §1 — Description (BCP-14 normative)

The REW service **MUST** ship BP ledger at `services/rew/src/bp/` with credit/debit txn log + nightly interest cron + immutable history, 4 memory audit kinds.

1. **MUST** validate `bp_txn_kind` against closed enum per DEC-2211.

2. **MUST** define ledger at migration `0007`:
   ```sql
   CREATE TABLE rew_bp_ledger (
     txn_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     member_id UUID NOT NULL,
     txn_kind TEXT NOT NULL
       CHECK (txn_kind IN ('credit_p3_accrual','credit_special_award','credit_interest_accrual','debit_p3_distribution','debit_correction')),
     amount_bp NUMERIC(15,4) NOT NULL,
     balance_after NUMERIC(15,4) NOT NULL,
     reason TEXT,
     correction_of UUID REFERENCES rew_bp_ledger(txn_id),
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   CREATE INDEX bp_ledger_member_time_idx ON rew_bp_ledger(tenant_id, member_id, created_at DESC);
   ALTER TABLE rew_bp_ledger ENABLE ROW LEVEL SECURITY;
   CREATE POLICY bp_rls ON rew_bp_ledger
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON rew_bp_ledger FROM cyberos_app;
   ```

3. **MUST** schedule interest accrual cron nightly 03:30 via TASK-MCP-007 per DEC-2212:
   - Fetch ACB rate from TASK-HR-005 policy
   - For each active member with balance > 0: compute daily_interest = balance * (annual_rate / 365); insert credit_interest_accrual txn

4. **MUST** compute balance pure-function at `balance_query.rs::balance(member, as_of)`:
   - Sum credits - debits up to as_of date
   - Verify matches latest txn.balance_after

5. **MUST** be immutable per DEC-2213 — corrections via new debit_correction txn with reason.

6. **MUST** expose endpoints:
   ```text
   POST /v1/rew/bp/credits             body: {member_id, kind, amount_bp, reason}
   POST /v1/rew/bp/debits              body: {member_id, kind, amount_bp, reason}
   GET  /v1/rew/members/{id}/bp-balance ?as_of=...
   GET  /v1/rew/members/{id}/bp-ledger
   POST /v1/rew/bp/interest-accrual/trigger  (CFO manual)
   ```

7. **MUST** emit 4 memory audit kinds per DEC-2214. PII per TASK-MEMORY-111: amounts SHA256.

8. **MUST** thread trace_id from credit/debit/accrual → audit.

9. **MUST NOT** mutate prior txn per DEC-2213.

10. **MUST NOT** skip nightly interest accrual per DEC-2212.

---

## §2 — Why this design

**Why ledger pattern (DEC-2210)?** Accounting standard — credit/debit log + balance derivation is auditable.

**Why nightly interest (DEC-2212)?** Daily granularity = fair accrual; weekly = under-pays late-month credits.

**Why immutable (DEC-2213)?** BP balances drive P3 distribution; rewrites = financial integrity breach.

---

## §3 — API contract

Sample credit:
```json
POST /v1/rew/bp/credits
{
  "member_id": "uuid",
  "kind": "credit_p3_accrual",
  "amount_bp": 500.0,
  "reason": "Q2 performance bonus accrual"
}
```

Sample balance:
```json
{
  "member_id": "uuid",
  "balance_bp": 2547.85,
  "as_of": "2026-05-17"
}
```

---

## §4 — Acceptance criteria
1. **bp_txn_kind enum cardinality 5**. 2. **Nightly interest cron 03:30**. 3. **ACB rate from TASK-HR-005**. 4. **Daily interest = balance * (rate/365)**. 5. **Immutable txn log**. 6. **Correction via debit_correction**. 7. **balance_after stored on each txn**. 8. **Balance query pure function**. 9. **4 memory audit kinds emitted**. 10. **PII scrubbed (amounts SHA256)**. 11. **RLS denies cross-tenant**. 12. **CFO-only credit/debit**. 13. **Trace_id preserved**. 14. **Append-only via REVOKE**. 15. **rust_decimal precision (15,4)**. 16. **As_of query**. 17. **0-balance members skipped in interest**. 18. **Cron idempotent per (member, date)**. 19. **Negative balance prevented (debit > balance rejected)**. 20. **Correction reason required**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn nightly_interest_accrual() {
    let ctx = TestContext::with_bp_balance_1000().await;
    ctx.run_interest_cron().await;
    let bal = ctx.balance(ctx.member_id, today()).await;
    let expected = dec!(1000) * (acb_rate() / dec!(365));
    assert!((bal - dec!(1000) - expected).abs() < dec!(0.01));
}

#[tokio::test]
async fn immutable_via_correction() {
    let ctx = TestContext::with_bp_txn().await;
    let r = ctx.try_update_txn(ctx.txn_id, dec!(0)).await;
    assert!(r.is_err());
    let corr = ctx.add_correction(ctx.member_id, ctx.txn_id, dec!(-100), "reversal").await;
    assert!(corr.is_ok());
}

#[tokio::test]
async fn debit_exceeds_balance_rejected() {
    let ctx = TestContext::with_bp_balance_100().await;
    let r = ctx.try_debit(ctx.member_id, dec!(200), "test").await;
    assert!(r.is_err());
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-REW-001.
**Downstream:** TASK-REW-008 (P3 distribution debits from this).
**Cross-module:** TASK-HR-005 (ACB rate policy), TASK-MCP-007 (cron), TASK-AUTH-101 (CFO), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| ACB rate missing | catch | sev-2; skip accrual | seed rate |
| Cron skipped | catch-up | inherent | inherent |
| Negative balance attempt | validate | reject | inherent |
| Decimal precision drift | rust_decimal | inherent | inherent |
| Cross-tenant query | RLS | 0 rows | inherent |
| Concurrent credit/debit | inherent ordering | inherent | inherent |
| balance_after mismatch | post-condition | sev-1; reject | bug fix |
| Correction without reason | validate | 400 | provide reason |
| Duplicate cron run | UNIQUE on (member, date, kind) | skip | inherent |
| Member deactivated | skip interest | inherent | inherent |

## §11 — Implementation notes
- §11.1 Cron via TASK-MCP-007 `kind: 'rew.bp_interest_accrual'`, daily 03:30.
- §11.2 Interest formula: `daily = balance * (annual_pct / 365)`; rust_decimal for precision.
- §11.3 memory audit body: member_id, txn_kind, balance_after; amount SHA256.
- §11.4 ACB rate cited from VN central bank reference rates.
- §11.5 Negative balance prevented via service check + future trigger.

---

*End of TASK-REW-007 spec.*
