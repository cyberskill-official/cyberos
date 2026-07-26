---
id: TASK-TEN-208
title: "AI-gateway ai_tokens overage admission → 402 PAYMENT_REQUIRED"
template: task@1
type: feature
module: ten
status: done
priority: p0
author: "@stephencheng"
department: engineering
created_at: 2026-07-26T13:00:00+00:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: [TASK-TEN-206, TASK-TEN-207, TASK-TEN-204]
blocks: []
related_tasks: [TASK-TEN-004, TASK-TEN-207, TASK-TEN-204]
routed_back_count: 0
verify: T
phase: P2
milestone: "P2 · billing-substrate"
slice: 2
owner: Stephen Cheng
created: 2026-07-26
effort_hours: 5
service: services/ai-gateway
new_files:
  - services/ai-gateway/src/metering_admit.rs
  - services/ai-gateway/tests/metering_ai_tokens_overage_test.rs
modified_files:
  - services/ai-gateway/src/lib.rs
  - services/ai-gateway/src/cost_ledger/mod.rs
  - services/ai-gateway/src/cost_ledger/types.rs
  - services/ai-gateway/src/streaming/mod.rs
  - services/ai-gateway/src/metering_emit.rs
  - services/ai-gateway/Cargo.toml
source_pages:
  - docs/tasks/ten/TASK-TEN-207-api-calls-overage-402/spec.md
  - docs/tasks/ten/TASK-TEN-004-four-axis-metering/spec.md
source_decisions:
  - DEC-710 overage policy
  - DEC-780 enterprise AI token cap
  - "2026-07-26 operator: continue after host-d — host-e ai_tokens 402"
---

# TASK-TEN-208: ai_tokens overage → 402

## Summary

At `cost_ledger::precheck`, before hold creation, admit estimated tokens
(`prompt_tokens + expected_completion_tokens`) against plan-tier
`ai_tokens_per_month`. On block → refuse with metering overage reason mapped to
HTTP 402; no hold; no emit.

## Problem

TEN-207 closed api_calls 402; ai-gateway still only has USD budget refuse.

## Proposed Solution

1. `metering_admit.rs` mirroring auth: overrides, default warn, plan_tier lookup,
   in-memory token sum from metering_emit recorder.
2. Hook early in `precheck` after idempotency/persona checks; quantity = estimate.
3. `RefuseReason::MeteringTokenOverage { current, cap }`.
4. Streaming maps that reason → HTTP 402 (not 429).
5. Tests: block at cap; warn default; under-cap allows.

## Alternatives Considered

- **Admit only on postcall.** Rejected: TEN-004 wants upstream reject; work must not run.

## Success Metrics

- Primary: over-cap + block → refuse/402, zero hold.
- Guardrail: default warn does not refuse.

## Scope

### In scope

- Precheck admit + HTTP mapping + tests.

### Out of scope / Non-Goals

- TEN-002 plan residuals
- INV Pg / INV-006
- Memory audit for warn/block
- Changing USD budget gate

## Dependencies

- TASK-TEN-206/207 patterns; TASK-TEN-204 emit recorder.

## Acceptance Criteria

1. **Block** — current+estimate > cap + block policy → MeteringTokenOverage refuse.
2. **No hold** on that path.
3. **Warn default** — does not refuse at cap.
4. **Streaming** maps MeteringTokenOverage → 402.
5. **Under cap** — precheck continues to existing budget path.
6. **source** — uses MeteringAxis::AiTokens + caps_for.ai_tokens_per_month.

## Verification

```sh
cd services && cargo test -p cyberos-ai-gateway --test metering_ai_tokens_overage_test -- --test-threads=1
bash .cyberos/cuo/gates/run-gates.sh
```

---

*End of TASK-TEN-208.*
