---
id: FR-TEN-203
title: "TEN margin watchdog for fixed-fee engagements - alert when projected margin drops below 30 percent"
module: TEN
priority: SHOULD
status: ready_to_implement
verify: T
phase: P4
milestone: P4 - vertical-pack marketplace
slice: 1
owner: Stephen Cheng (CFO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-PROJ-007, FR-TIME-005, FR-INV-001]
depends_on: [FR-PROJ-007]
blocks: []

source_pages:
  - website/docs/modules/proj/index.html#engagement-economics
  - website/docs/modules/proj/changelog.html
source_decisions:
  - DEC-TEN-203-1 - Fixed-fee margin risk is a tenant/business guardrail, while PROJ owns the source engagement data.
  - DEC-TEN-203-2 - Alert threshold is 30 percent projected gross margin unless tenant policy overrides it upward.

build_envelope:
  language: rust 1.81
  service: cyberos/services/ten/
  new_files:
    - services/ten/migrations/0012_margin_watchdog.sql
    - services/ten/src/margin/mod.rs
    - services/ten/src/margin/projection.rs
    - services/ten/src/margin/alerts.rs
    - services/ten/tests/margin_watchdog_test.rs
  modified_files:
    - services/ten/src/lib.rs
  allowed_tools:
    - file_read: services/{ten,proj,inv,time}/**
    - file_write: services/ten/{src,tests,migrations}/**
    - bash: cd services/ten && cargo test margin
  disallowed_tools:
    - mutate PROJ engagement source data from TEN
    - emit client-visible alerts before internal AM/CEO notification

effort_hours: 5
sub_tasks:
  - "0.5h: 0012_margin_watchdog.sql"
  - "1.0h: projection.rs"
  - "1.0h: alerts.rs"
  - "1.0h: memory audit and OTel events"
  - "1.5h: tests"
risk_if_skipped: "Fixed-fee engagements can silently erode below target margin. Operators discover scope creep only at invoice or closeout time, when renegotiation leverage is lowest."
---

## §1 - Description (BCP-14 normative)

The TEN service **MUST** ship a margin watchdog for fixed-fee engagements using PROJ billing-mode data from FR-PROJ-007.

1. **MUST** compute projected gross margin for every fixed-fee engagement as `(fixed_fee_amount - projected_delivery_cost) / fixed_fee_amount`.
2. **MUST** derive projected delivery cost from actual burned time plus remaining estimated work, using PROJ/TIME source references.
3. **MUST** default the alert threshold to 30 percent projected gross margin.
4. **MUST** allow tenant policy to set a stricter threshold, but not a lower threshold.
5. **MUST** emit an internal AM + CEO alert when projected margin drops below threshold.
6. **MUST** suppress duplicate alerts for the same engagement until margin recovers above threshold and then drops again.
7. **MUST** expose `GET /v1/ten/margin-watchdog` returning at-risk fixed-fee engagements sorted by projected margin ascending.
8. **MUST** expose `POST /v1/ten/margin-watchdog/{engagement_id}/ack` for AM acknowledgement with action plan text.
9. **MUST** emit memory audit rows `ten.margin_watchdog_triggered` and `ten.margin_watchdog_acknowledged`.
10. **MUST** emit OTel metric `ten_fixed_fee_margin_projection{bucket}`.
11. **MUST** RLS-enforce tenant isolation.
12. **MUST NOT** mutate PROJ billing-mode or TIME source records.

## §2 - API Contract

```json
{
  "engagement_id": "uuid",
  "fixed_fee_amount_minor": 10000000,
  "projected_delivery_cost_minor": 7600000,
  "projected_margin_pct": 24.0,
  "threshold_pct": 30.0,
  "source_refs": ["proj:engagement:uuid", "time:rollup:uuid"],
  "status": "triggered"
}
```

## §3 - Acceptance Criteria

1. Projection uses actual burned time and remaining estimate.
2. Default threshold is 30 percent.
3. Tenant threshold cannot be lowered below 30 percent.
4. Alert triggers below threshold.
5. Duplicate alerts are suppressed.
6. Recovery above threshold resets alert state.
7. Acknowledgement requires action plan text.
8. Memory audit rows are emitted for trigger and acknowledgement.
9. RLS denies cross-tenant reads.
10. PROJ/TIME source records remain read-only.

## §4 - Verification

```bash
cd services/ten && cargo test margin
```

## §7 - Dependencies

**Upstream:** FR-PROJ-007.
**Cross-module:** FR-TIME-005 and FR-INV-001 provide downstream context but are not blockers.

## §10 - Failure Modes

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Missing remaining estimate | projection warning | use actual burn only | AM updates estimate |
| Duplicate threshold crossing | alert state | suppress | reset after recovery |
| Cross-tenant source ref | RLS test | reject | fix tenant context |
