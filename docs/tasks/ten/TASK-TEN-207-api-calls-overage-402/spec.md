---
id: TASK-TEN-207
title: "Auth api_calls overage admission → 402 PAYMENT_REQUIRED"
template: task@1
type: feature
module: ten
status: done
priority: p0
author: "@stephencheng"
department: engineering
created_at: 2026-07-26T11:20:00+00:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: [TASK-TEN-206, TASK-TEN-205, TASK-TEN-002]
blocks: []
related_tasks: [TASK-TEN-004, TASK-TEN-205, TASK-TEN-206]
routed_back_count: 0
verify: T
phase: P2
milestone: "P2 · billing-substrate"
slice: 2
owner: Stephen Cheng
created: 2026-07-26
effort_hours: 5
service: services/auth
new_files:
  - services/auth/src/metering_admit.rs
  - services/auth/tests/metering_overage_402_test.rs
modified_files:
  - services/auth/src/middleware.rs
  - services/auth/src/lib.rs
  - services/metering/src/admission.rs
  - services/metering/src/lib.rs
source_pages:
  - docs/tasks/ten/TASK-TEN-004-four-axis-metering/spec.md
  - docs/tasks/ten/TASK-TEN-205-api-calls-metering-emit/spec.md
source_decisions:
  - DEC-710 overage policy block/warn/allow
  - DEC-778 starter 10k api_calls/month
  - "2026-07-26 operator: continue host-d after INV-012"
---

# TASK-TEN-207: api_calls overage → 402

## Summary

Before `next.run` in `verify_jwt`, evaluate api_calls overage against plan-tier
caps. On `OverageDecision::Block`, return `402` with
`{error:"overage_blocked",axis:"api_calls",current,cap}` and do not emit.

## Problem

Emit path bills successes but never rejects over-cap traffic (TEN-205 OOS).

## Proposed Solution

1. `cyberos_metering::admission::admit_quantity` pure helper over `evaluate()`.
2. `metering_admit.rs`: resolve plan_tier (Pg or test override), current usage
   (InMemory sum / optional Pg period_sum), policy default `warn` (testable
   override to `block`).
3. `verify_jwt`: after deny-list, before `next.run` — if Block → 402.
4. Warn decisions: allow request; emit still on success (no memory audit in host-d).
5. Tests: seed usage at cap + Block policy → 402; under cap → 200-path emit ok.

## Alternatives Considered

- **402 on response path after handler.** Rejected: TEN-004 §1 #11 rejects upstream; work must not run.

## Success Metrics

- Primary: over-cap + block → 402, zero new emit.
- Guardrail: under-cap requests unchanged.

## Scope

### In scope

- Admission helper + middleware hook + tests for api_calls only.

### Out of scope / Non-Goals

- ai_tokens overage at ai-gateway
- Tenant overage_policy column migration (host-d uses default + test override)
- Memory audit for warn/block
- GET /v1/usage

## Dependencies

- TASK-TEN-206 (usage sum + durable path).
- TASK-TEN-002 caps (`caps_for`).

## Acceptance Criteria

1. **Block** — current+1 > cap + policy block → 402 body shape.
2. **No emit on 402** — recorder length unchanged.
3. **Under cap** — admission allows; success still emits (TEN-205).
4. **Warn policy** — over threshold does not 402.
5. **Enterprise unlimited api** — cap None → always allow.
6. **Default policy warn** — production default does not 402 without override.

## Verification

```sh
cd services && cargo test -p cyberos-metering -p cyberos-auth -- --test-threads=1
bash .cyberos/cuo/gates/run-gates.sh
```

---

*End of TASK-TEN-207.*
