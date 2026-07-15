---
id: NFR-HR-007
title: "HR onboarding saga durability — every step MUST be replayable; crashes recoverable"
module: HR
category: reliability
priority: MUST
verification: T
phase: P0
slo: "100% of onboarding sagas reach terminal state; crashes recover from last step"
owner: CHRO
created: 2026-05-18
related_tasks: [TASK-HR-007]
---

## §1 — Statement (BCP-14 normative)

1. The HR onboarding saga (`TASK-HR-007`) **MUST** persist step state in a durable journal — process crashes resume at last completed step.
2. Each step **MUST** be idempotent — replay produces same outcome.
3. Failed steps **MUST** be retryable; persistent failure (3 retries) opens a manual-intervention ticket.
4. Cancellation mid-saga **MUST** trigger rollback of completed steps (account de-provisioning, etc.).
5. End-to-end onboarding timeline: target 5 business days; outliers > 10 days flagged for CHRO.

## §2 — Why this constraint

Onboarding touches many systems (HR record, email, payroll, equipment, access). Without durable saga semantics, a crash mid-flow leaves the new hire in a half-onboarded state — bad for their experience + bad for the company. Idempotency + retry handles transient failures. Cancellation rollback handles offer-withdrawals cleanly.

## §3 — Measurement

- Counter `hr_onboarding_saga_total{outcome}`.
- Histogram `hr_onboarding_duration_days`.
- Gauge `hr_onboarding_stuck_count`.

## §4 — Verification

- Integration test (T) — full saga; assert all steps logged.
- Chaos test (T) — kill mid-saga; restart; assert resume.
- Cancellation test (T) — cancel; assert rollback.

## §5 — Failure handling

- Stuck > 10 days → sev-3; CHRO intervention.
- Retry exhausted → manual ticket.
- Rollback failure → sev-2; manual cleanup.

---

*End of NFR-HR-007.*
