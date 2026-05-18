---
id: NFR-CRM-007
title: "CRM deal-stage history — every stage change MUST be append-only audited"
module: CRM
category: maintainability
priority: MUST
verification: T
phase: P0
slo: "100% of deal-stage transitions create audit rows; history immutable"
owner: CSO-Sales
created: 2026-05-18
related_frs: [FR-CRM-001]
---

## §1 — Statement (BCP-14 normative)

1. Every deal stage transition **MUST** create an append-only audit row with `{deal_id, from_stage, to_stage, actor_id, changed_at, reason?}`.
2. The deal table's `current_stage` column is a denormalised view of the latest history row.
3. Stage-history queries (sales-velocity reports, time-in-stage analytics) read from the history table, not the deal row.
4. Reversal transitions (e.g., `won → negotiation`) are allowed but flagged; they accumulate in the history.
5. Stage-history retention: ≥ 5 years (sales-analytics requirement).

## §2 — Why this constraint

Sales-velocity analytics depend on accurate stage history. The append-only model is the standard CRM ledger pattern. Reversal-flagging surfaces "we lost confidence" moments without forbidding them. The denormalised current-stage is the read-optimisation trick.

## §3 — Measurement

- Counter `crm_deal_stage_transition_total{from, to}`.
- Counter `crm_deal_stage_reversal_total`.
- Reconciliation: current-stage = latest history row.

## §4 — Verification

- Integration test (T) — transitions; assert audit row + denorm in sync.
- Property test (T) — random sequences; assert append-only.
- Reversal test (T) — assert flagged.

## §5 — Failure handling

- Denorm drift → sev-2; investigate.
- In-place edit attempt → block + sev-2.
- High reversal rate per rep → CSO-Sales coaching signal.

---

*End of NFR-CRM-007.*
