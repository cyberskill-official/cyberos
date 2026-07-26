---
batch: batch/ten-inv-host-d
members:
  - TASK-TEN-206
  - TASK-TEN-207
started: 2026-07-26T11:20:00Z
ended: 2026-07-26T12:48:00Z
route_backs: 0
gate_reasks: 0
tokens: unknown
---

# batch/ten-inv-host-d — metering Pg drain + api_calls 402

## Context

After host-c (#167 INV-012), next commercial gap: durable metering + overage admission.

## Target

- TASK-TEN-206: Pg insert + WAL drain + period_sum + awh-gate migrate
- TASK-TEN-207: verify_jwt admission → 402 on block overage

## Out of scope

- TEN-002 plan residuals (rate-limit, dry_run, history GET)
- INV Pg / INV-006
- ai_tokens 402
- Memory audit kinds for warn/block
