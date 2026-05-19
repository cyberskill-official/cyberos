---
id: FR-ESOP-007
title: "ESOP Member dashboard — personal view only (own grants + vesting + estimated value); cross-Member access requires CFO audit"
module: ESOP
priority: SHOULD
status: draft
verify: T
phase: P2
milestone: P2 · slice 2
slice: 2
owner: Stephen Cheng (CFO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-ESOP-001, FR-ESOP-002, FR-ESOP-003, FR-AUTH-101, FR-MEMORY-111]
depends_on: [FR-ESOP-001]
blocks: []

source_pages:
  - website/docs/modules/esop.html#member-dashboard

source_decisions:
  - DEC-2310 2026-05-17 — Member sees ONLY own grants/vesting/put-history; cross-Member view requires CFO audited access
  - DEC-2311 2026-05-17 — Closed enum `dashboard_access_kind` = {self_view, cfo_audit_view, ceo_audit_view, denied}; cardinality 4
  - DEC-2312 2026-05-17 — Estimated current value computed: vested_shares × FR-ESOP-003 committed price (latest)
  - DEC-2313 2026-05-17 — Cross-Member access logged with audit reason + accessor + accessed_member; sev-2 audit
  - DEC-2314 2026-05-17 — memory audit kinds: esop.dashboard_self_view, esop.dashboard_cfo_audit_view, esop.dashboard_access_denied, esop.dashboard_estimated_value_computed

build_envelope:
  language: rust 1.81
  service: cyberos/services/esop/
  new_files:
    - services/esop/migrations/0007_dashboard_access_log.sql
    - services/esop/src/dashboard/mod.rs
    - services/esop/src/dashboard/access_gate.rs
    - services/esop/src/dashboard/value_calculator.rs
    - services/esop/src/handlers/dashboard_routes.rs
    - services/esop/src/audit/dashboard_events.rs
    - services/esop/tests/dashboard_self_view_test.rs
    - services/esop/tests/dashboard_cross_member_denied_test.rs
    - services/esop/tests/dashboard_cfo_audit_access_test.rs
    - services/esop/tests/dashboard_access_kind_enum_cardinality_test.rs
    - services/esop/tests/dashboard_estimated_value_test.rs
    - services/esop/tests/dashboard_audit_emission_test.rs

  modified_files:
    - services/esop/src/lib.rs

  allowed_tools:
    - file_read: services/{esop,auth}/**
    - file_write: services/esop/{src,tests,migrations}/**
    - bash: cd services/esop && cargo test dashboard

  disallowed_tools:
    - cross-member without audit (per DEC-2310)
    - silent cross-member view (per DEC-2313)

effort_hours: 6
sub_tasks:
  - "0.3h: 0007_dashboard_access_log.sql"
  - "0.3h: dashboard/mod.rs"
  - "0.5h: access_gate.rs"
  - "0.4h: value_calculator.rs"
  - "0.4h: handlers/dashboard_routes.rs"
  - "0.3h: audit/dashboard_events.rs"
  - "2.0h: tests — 6 test files"
  - "1.8h: Member UI + CFO audit dashboard"

risk_if_skipped: "Without dashboard, members can't see their equity → trust damaged. Without DEC-2310 self-scope, comp data leaks across members. Without DEC-2313 access log, CFO can quietly snoop."
---

## §1 — Description (BCP-14 normative)

The ESOP service **MUST** ship Member dashboard at `services/esop/src/dashboard/` with self-only default + CFO-audited cross-member access + estimated value, 4 memory audit kinds.

1. **MUST** validate `dashboard_access_kind` against closed enum per DEC-2311.

2. **MUST** gate at `access_gate.rs::check(requester, viewed_member)` per DEC-2310:
   - requester == viewed_member → self_view (always allowed)
   - requester has CFO role AND provides audit_reason → cfo_audit_view (logged)
   - requester has CEO role AND provides audit_reason → ceo_audit_view (logged)
   - Otherwise → denied + sev-2 audit

3. **MUST** compute estimated value at `value_calculator.rs::compute(member)` per DEC-2312:
   - For each grant: vested = latest FR-ESOP-002 accrual
   - latest_price = FR-ESOP-003 committed_share_price for current year
   - estimated_value = vested * latest_price

4. **MUST** log access at migration `0007`:
   ```sql
   CREATE TABLE esop_dashboard_access_log (
     log_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     requester_id UUID NOT NULL,
     viewed_member_id UUID NOT NULL,
     access_kind TEXT NOT NULL
       CHECK (access_kind IN ('self_view','cfo_audit_view','ceo_audit_view','denied')),
     audit_reason TEXT,
     ip_address_hash TEXT,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   CREATE INDEX dashboard_log_member_idx ON esop_dashboard_access_log(tenant_id, viewed_member_id, created_at DESC);
   ALTER TABLE esop_dashboard_access_log ENABLE ROW LEVEL SECURITY;
   CREATE POLICY log_rls ON esop_dashboard_access_log
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON esop_dashboard_access_log FROM cyberos_app;
   ```

5. **MUST** expose endpoints:
   ```text
   GET /v1/esop/members/{id}/dashboard     (self → always OK; cross → requires audit_reason header)
   ```

6. **MUST** emit 4 memory audit kinds per DEC-2314. PII per FR-MEMORY-111: estimated_value SHA256.

7. **MUST** thread trace_id from request → gate → audit.

8. **MUST NOT** allow cross-member view without audit reason per DEC-2310.

9. **MUST NOT** silently log cross-member view per DEC-2313 (sev-2 memory audit always).

---

## §2 — Why this design

**Why self-only default (DEC-2310)?** Equity is sensitive personal financial info; default scope = self.

**Why CFO/CEO audit access (DEC-2310)?** Legitimate need — promotion review, IPO prep, fraud investigation.

**Why access log (DEC-2313)?** Detects silent snooping; audit trail for board review.

---

## §3 — API contract

Sample self view:
```json
GET /v1/esop/members/{id}/dashboard

{
  "member_id": "uuid",
  "grants": [
    {
      "grant_id": "uuid",
      "kind": "employee_initial",
      "total_shares": 10000,
      "vested_shares": 3333,
      "vesting_pct": 0.333,
      "estimated_value_vnd": 166650000,
      "next_vest_date": "2026-07-01"
    }
  ],
  "total_estimated_value_vnd": 166650000,
  "valuation_year": 2026,
  "share_price_vnd": 50000
}
```

Cross-member denied:
```json
{"error": "dashboard_access_denied", "reason": "Cross-member view requires audit_reason header (CFO/CEO only)."}
```

---

## §4 — Acceptance criteria
1. **dashboard_access_kind enum cardinality 4**. 2. **Self-view always allowed**. 3. **Cross-member denied for non-CFO/CEO**. 4. **CFO with audit_reason allowed + logged**. 5. **CEO with audit_reason allowed + logged**. 6. **Estimated value = vested × latest committed price**. 7. **Per-grant breakdown**. 8. **Next vest date computed**. 9. **4 memory audit kinds emitted**. 10. **PII scrubbed (value SHA256)**. 11. **RLS denies cross-tenant**. 12. **Trace_id preserved**. 13. **Append-only access log**. 14. **IP hashed**. 15. **denied audits sev-2**. 16. **Audit reason required for cross**. 17. **Self-view doesn't require reason**. 18. **Audit log queryable by CISO**. 19. **bigint VND**. 20. **No FR-ESOP-003 valuation → estimated_value=null + sev-3**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn self_view_allowed() {
    let ctx = TestContext::with_member_and_grant().await;
    let r = ctx.fetch_dashboard_as_member(ctx.member_id).await;
    assert_eq!(r.status_code, 200);
    assert_eq!(r.body.member_id, ctx.member_id);
}

#[tokio::test]
async fn cross_member_denied_for_engineer() {
    let ctx = TestContext::with_engineer_role().await;
    let r = ctx.try_fetch_dashboard_as(ctx.engineer, ctx.other_member).await;
    assert_eq!(r.status_code, 403);
    let audits = ctx.fetch_memory_audits("esop.dashboard_access_denied").await;
    assert!(!audits.is_empty());
}

#[tokio::test]
async fn cfo_with_reason_logged() {
    let ctx = TestContext::with_cfo().await;
    let r = ctx.fetch_dashboard_as(ctx.cfo, ctx.other_member, "promotion review").await;
    assert_eq!(r.status_code, 200);
    let log = ctx.fetch_access_log(ctx.other_member).await;
    let cfo_audit = log.iter().find(|l| l.access_kind == "cfo_audit_view").unwrap();
    assert_eq!(cfo_audit.audit_reason.as_deref(), Some("promotion review"));
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-ESOP-001.
**Cross-module:** FR-ESOP-002 (vested), FR-ESOP-003 (valuation), FR-AUTH-101 (roles), FR-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Cross-member no reason | gate | 403 + sev-2 audit | provide reason |
| No valuation for year | check | estimated=null + sev-3 | propose valuation |
| No vested shares | inherent | shows 0 | inherent |
| Cancelled grant | filter | excluded | inherent |
| Cross-tenant | RLS | 0 rows | inherent |
| Audit reason missing for CFO | gate | 400 | provide |
| Member doesn't exist | RLS + FK | 404 | inherent |
| Decimal precision | bigint | inherent | inherent |
| Audit log fills | partition by month | inherent | maintenance |
| Concurrent dashboard fetch | inherent | inherent | inherent |

## §11 — Implementation notes
- §11.1 Self-view: requester_id == path member_id; no audit_reason needed.
- §11.2 Cross-view: requester has CFO/CEO role; audit_reason in X-Audit-Reason header.
- §11.3 memory audit body: requester, viewed_member, access_kind; IP SHA256; values SHA256.
- §11.4 Next vest date computed from grant + FR-ESOP-002 schedule.
- §11.5 CISO dashboard for periodic review of cfo_audit_view + ceo_audit_view rows.

---

*End of FR-ESOP-007 spec.*
