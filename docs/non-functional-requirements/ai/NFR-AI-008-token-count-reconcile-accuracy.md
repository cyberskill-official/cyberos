---
id: NFR-AI-008
title: "AI Gateway token-count post-call reconcile accuracy — within 0.5%"
module: AI
category: reliability
priority: MUST
verification: T
phase: P0
slo: "Reconciled token count within 0.5% of provider-reported usage, averaged over 1000 calls"
owner: CFO
created: 2026-05-18
related_frs: [FR-AI-007, FR-AI-008]
---

## §1 — Statement (BCP-14 normative)

1. After every upstream provider call, the AI Gateway **MUST** reconcile the locally-projected token count (used for cost-ledger admit per NFR-AI-002) against the provider-reported usage in the response `usage` field.
2. The reconciled vs projected delta **MUST** average within **±0.5%** over any rolling 1000-call window per (provider, model_alias). Single-call deltas of up to ±5% are tolerable due to tokeniser variance.
3. The reconciled count **MUST** be written to the durable cost-ledger (Postgres) within 1s of the upstream response completing; the in-memory projected count is reconciled-or-replaced, never accumulated.
4. If a provider response omits `usage` entirely, the gateway **MUST** fall back to local re-tokenisation of the full request+response transcript and emit a `ai_gateway_token_count_estimated_total` counter increment — never silently zero the cost.
5. Drift > 0.5% sustained for 1000 calls **MUST** auto-pin the projection to the higher of (provider report, local estimate) — conservative over-projection prevents under-charging.

## §2 — Why this constraint

The token count is the unit of cost. A 0.5% accuracy floor is the difference between accurately invoicing tenants and silently absorbing token drift as platform-eaten cost. Anthropic, OpenAI, and Mistral all have slightly different tokenisation rules; without active reconciliation, an `tiktoken`-based local projection drifts ~1-2% per model. The 0.5% rolling target catches drift before it material; the 5% per-call tolerance accommodates tokeniser implementation differences. The "auto-pin higher" failure mode means the platform over-charges itself, never under-charges the tenant — protects CFO's revenue assertion.

## §3 — Measurement

- Gauge `ai_gateway_token_count_drift_pct{provider, model_alias}` — rolling 1000-call average delta. Alarm at > 0.5%.
- Counter `ai_gateway_token_count_estimated_total{provider, model_alias}` — count of calls where local estimation was needed.
- Reconciliation row in `services/ai-gateway/src/cost/ledger.rs::reconcile()` — written every call.

## §4 — Verification

- Integration test `services/ai-gateway/tests/token_reconcile_accuracy_test.rs` (T) — drives 1000 calls against each provider mock with known token counts; asserts drift < 0.5%.
- Monthly CFO reconciliation (A) — CFO compares per-tenant invoiced tokens against provider's actual billed tokens for that tenant's traffic; delta must be < 1%.

## §5 — Failure handling

- Drift > 0.5% for 1000 calls → auto-pin projection to higher-of (per §1 #5); sev-3 alert; engineering re-tokenises the local model alias.
- Drift > 5% sustained → sev-2; cost-ledger is materially wrong, pause new high-volume tenants until corrected.
- CFO monthly reconciliation finds > 1% under-charge → sev-2 finance issue; CFO + CTO file a back-charge or absorb depending on amount.

---

*End of NFR-AI-008.*
