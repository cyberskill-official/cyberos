---
id: FR-LEARN-007
title: "LEARN VP score → REW BP fund distribution handoff — quarter-close trigger emits aggregate VP shares per member to REW for fund allocation"
module: LEARN
priority: MUST
status: draft
verify: T
phase: P1
milestone: P1 · slice 7
slice: 7
owner: Stephen Cheng (CEO)
created: 2026-05-17
shipped: null
brain_chain_hash: null
related_frs: [FR-LEARN-003, FR-REW-008, FR-MCP-007, FR-BRAIN-111]
depends_on: [FR-LEARN-003]
blocks: []

source_pages:
  - website/docs/modules/learn.html#vp-rew-handoff

source_decisions:
  - DEC-2140 2026-05-17 — At quarter close (Mar 31 / Jun 30 / Sep 30 / Dec 31), sum per-member VP from FR-LEARN-003 weekly snapshots → emit aggregate share to FR-REW-008 for BP fund split
  - DEC-2141 2026-05-17 — Closed enum `handoff_status` = {pending, computed, emitted, acknowledged_by_rew, failed}; cardinality 5
  - DEC-2142 2026-05-17 — Aggregate is share-of-total (member_vp / total_vp), not absolute; REW uses share to split fund
  - DEC-2143 2026-05-17 — Deterministic + idempotent per (tenant, quarter); UNIQUE constraint
  - DEC-2144 2026-05-17 — BRAIN audit kinds: learn.vp_rew_handoff_started, learn.vp_rew_handoff_emitted, learn.vp_rew_handoff_acked, learn.vp_rew_handoff_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/learn/
  new_files:
    - services/learn/migrations/0007_vp_rew_handoffs.sql
    - services/learn/src/handoff/mod.rs
    - services/learn/src/handoff/aggregator.rs
    - services/learn/src/handoff/rew_emitter.rs
    - services/learn/src/handlers/handoff_routes.rs
    - services/learn/src/audit/handoff_events.rs
    - services/learn/tests/handoff_quarter_close_test.rs
    - services/learn/tests/handoff_status_enum_cardinality_test.rs
    - services/learn/tests/handoff_idempotent_test.rs
    - services/learn/tests/handoff_share_sums_to_1_test.rs
    - services/learn/tests/handoff_audit_emission_test.rs

  modified_files:
    - services/learn/src/lib.rs

  allowed_tools:
    - file_read: services/{learn,rew}/**
    - file_write: services/learn/{src,tests,migrations}/**
    - bash: cd services/learn && cargo test handoff

  disallowed_tools:
    - mutate prior handoff (per DEC-2143)
    - emit absolute amounts (per DEC-2142 — share only)

effort_hours: 4
sub_tasks:
  - "0.3h: 0007_vp_rew_handoffs.sql"
  - "0.3h: handoff/mod.rs"
  - "0.4h: aggregator.rs"
  - "0.5h: rew_emitter.rs"
  - "0.3h: handlers/handoff_routes.rs"
  - "0.3h: audit/handoff_events.rs"
  - "1.5h: tests — 5 test files"
  - "0.4h: cron registration + docs"

risk_if_skipped: "Without handoff, REW BP distribution disconnects from VP contributions. Without DEC-2142 share semantics, REW can't fairly split (absolute amounts may exceed fund). Without DEC-2143 idempotency, duplicate quarter handoffs double-pay."
---

## §1 — Description (BCP-14 normative)

The LEARN service **MUST** ship VP → REW handoff at `services/learn/src/handoff/` quarterly aggregate + share calc + REW emit, 4 BRAIN audit kinds.

1. **MUST** validate `handoff_status` against closed enum per DEC-2141.

2. **MUST** schedule cron Q-end+1d at 04:00 tenant_tz via FR-MCP-007.

3. **MUST** aggregate at `aggregator.rs::aggregate(tenant, quarter)` per DEC-2140:
   - Sum FR-LEARN-003 snapshots for each member over quarter's weeks
   - Compute total_vp = sum across all members
   - Per-member share = member_vp / total_vp (sums to 1.0)

4. **MUST** emit to REW at `rew_emitter.rs::emit(tenant, quarter, shares)` per DEC-2140 — call FR-REW-008 with `{member_id, vp_share}` list.

5. **MUST** be idempotent per DEC-2143:
   ```sql
   CREATE TABLE learn_vp_rew_handoffs (
     handoff_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     quarter CHAR(7) NOT NULL,  -- 'YYYY-Qx'
     status TEXT NOT NULL DEFAULT 'pending'
       CHECK (status IN ('pending','computed','emitted','acknowledged_by_rew','failed')),
     total_vp NUMERIC(15,4) NOT NULL DEFAULT 0,
     members_count INT NOT NULL DEFAULT 0,
     emitted_at TIMESTAMPTZ,
     acked_at TIMESTAMPTZ,
     failure_reason TEXT,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     UNIQUE (tenant_id, quarter)
   );
   ALTER TABLE learn_vp_rew_handoffs ENABLE ROW LEVEL SECURITY;
   CREATE POLICY handoffs_rls ON learn_vp_rew_handoffs
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON learn_vp_rew_handoffs FROM cyberos_app;
   GRANT UPDATE (status, total_vp, members_count, emitted_at, acked_at, failure_reason) ON learn_vp_rew_handoffs TO cyberos_app;

   CREATE TABLE learn_vp_rew_member_shares (
     handoff_id UUID NOT NULL REFERENCES learn_vp_rew_handoffs(handoff_id),
     tenant_id UUID NOT NULL,
     member_id UUID NOT NULL,
     vp_share NUMERIC(10,9) NOT NULL CHECK (vp_share >= 0 AND vp_share <= 1),
     vp_absolute NUMERIC(15,4) NOT NULL,
     PRIMARY KEY (handoff_id, member_id)
   );
   ALTER TABLE learn_vp_rew_member_shares ENABLE ROW LEVEL SECURITY;
   CREATE POLICY shares_rls ON learn_vp_rew_member_shares
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON learn_vp_rew_member_shares FROM cyberos_app;
   ```

6. **MUST** expose endpoints:
   ```text
   POST /v1/learn/vp-rew/trigger    (CEO; manual for current quarter)
   GET  /v1/learn/vp-rew/handoffs   (list)
   ```

7. **MUST** emit 4 BRAIN audit kinds per DEC-2144. PII per FR-BRAIN-111: vp values SHA-256; member_id + shares ok.

8. **MUST** thread trace_id from cron → aggregate → REW emit → ack → audit.

9. **MUST NOT** mutate prior handoff per DEC-2143.

10. **MUST NOT** emit absolute amounts per DEC-2142.

11. **MUST** verify shares sum to ≈1.0 (tolerance 1e-6) before emit.

---

## §2 — Why this design

**Why share not absolute (DEC-2142)?** REW determines fund size; LEARN only knows fairness ratios. Decoupling.

**Why idempotent (DEC-2143)?** Quarter handoff = financial event; double-pay = embarrassment + cost.

**Why share sum check (DEC-2144)?** Floating-point arithmetic can drift; tolerance check protects against bugs.

---

## §3 — API contract

Sample handoff:
```json
{
  "handoff_id": "uuid",
  "quarter": "2026-Q2",
  "status": "acknowledged_by_rew",
  "total_vp": 12500.5,
  "members_count": 35,
  "shares": [
    {"member_id": "uuid-alice", "vp_share": 0.085, "vp_absolute": 1062.5},
    {"member_id": "uuid-bob", "vp_share": 0.062, "vp_absolute": 775.0}
  ]
}
```

---

## §4 — Acceptance criteria
1. **handoff_status enum cardinality 5**. 2. **Quarter-close cron**. 3. **Aggregate per-member share**. 4. **Shares sum to 1.0 ±1e-6**. 5. **Emit to FR-REW-008**. 6. **UNIQUE(tenant, quarter) idempotency**. 7. **4 BRAIN audit kinds emitted**. 8. **PII scrubbed (vp values SHA256)**. 9. **RLS denies cross-tenant**. 10. **CEO-only manual trigger**. 11. **Trace_id preserved**. 12. **Append-only via REVOKE**. 13. **REW ack → status=acknowledged_by_rew**. 14. **Failure → status=failed + sev-1**. 15. **0 active members → skip + sev-3**. 16. **rust_decimal precision**. 17. **Share precision (10,9)**. 18. **vp_share CHECK 0-1**. 19. **history queryable**. 20. **Quarter format 'YYYY-Qx' enforced**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn shares_sum_to_1() {
    let ctx = TestContext::with_5_members_vp_each(100).await;
    ctx.run_handoff(this_quarter()).await;
    let shares = ctx.fetch_shares(this_quarter()).await;
    let total: Decimal = shares.iter().map(|s| s.vp_share).sum();
    assert!((total - dec!(1.0)).abs() < dec!(0.000001));
}

#[tokio::test]
async fn idempotent_double_run() {
    let ctx = TestContext::with_member_vp_data().await;
    ctx.run_handoff(this_quarter()).await;
    let r = ctx.run_handoff(this_quarter()).await;
    let handoffs = ctx.fetch_handoffs(this_quarter()).await;
    assert_eq!(handoffs.len(), 1);
}

#[tokio::test]
async fn rew_emit_and_ack() {
    let ctx = TestContext::with_member_vp_data().await;
    ctx.run_handoff(this_quarter()).await;
    let h = ctx.fetch_handoff(this_quarter()).await;
    assert_eq!(h.status, "acknowledged_by_rew");
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-LEARN-003.
**Cross-module:** FR-REW-008 (BP fund consumer), FR-MCP-007 (cron), FR-AUTH-101 (CEO), FR-BRAIN-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Cron skipped | catch-up | inherent | inherent |
| Duplicate handoff | UNIQUE | skip | inherent |
| Shares don't sum to 1 | tolerance check | reject; sev-1 | bug fix |
| REW unreachable | retry | status=failed; sev-1 | retry |
| 0 members | skip | sev-3 | inherent |
| Decimal precision drift | rust_decimal | inherent | inherent |
| Cross-tenant query | RLS | 0 rows | inherent |
| Quarter format invalid | CHECK | 400 | YYYY-Qx |
| Mid-handoff crash | resume | partial → failed | retry |
| REW ack timeout | mark emitted | wait | manual ack |

## §11 — Implementation notes
- §11.1 Cron via FR-MCP-007 `kind: 'learn.vp_rew_handoff'`, Q-end+1d at 04:00.
- §11.2 Aggregator pure function: `(member_vp_snapshots, total_vp) → shares`.
- §11.3 REW emit via HTTP POST to FR-REW-008 endpoint with retry+ack.
- §11.4 BRAIN audit body: handoff_id, quarter, members_count; total_vp SHA256.
- §11.5 Member with 0 VP for quarter → vp_share=0 (still included in shares list for transparency).

---

*End of FR-LEARN-007 spec.*
