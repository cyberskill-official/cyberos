---
id: NFR-TEN-001
title: "TEN provisioning saga reliability — every tenant create MUST be transactional"
module: TEN
category: reliability
priority: MUST
verification: T
phase: P0
slo: "100% of tenant-create sagas reach terminal state (committed | rolled-back); 0 stuck-in-progress"
owner: CTO
created: 2026-05-18
related_frs: [FR-TEN-001, FR-TEN-101]
---

## §1 — Statement (BCP-14 normative)

1. Tenant provisioning **MUST** run as a saga with all-or-nothing semantics: every step succeeds and the tenant is committed, OR any step fails and all prior steps roll back.
2. The saga **MUST** persist its state in a durable saga-journal so a process crash mid-saga is recoverable.
3. Per-step idempotency is required — replaying a step **MUST** be safe.
4. Saga timeouts: end-to-end provisioning **MUST** complete within 10 minutes; > 10 min triggers rollback.
5. Manual recovery of stuck sagas **MUST** require explicit operator action — no auto-decision to commit or roll back without operator confirmation past timeout.

## §2 — Why this constraint

Tenant provisioning touches DB, KMS, Stripe, billing, residency assignment — half-finished states leave the platform in an inconsistent state (tenant exists but no billing, etc.). Saga semantics + durable journal + idempotent steps is the textbook pattern. The 10-minute ceiling is the operational sweet spot: long enough for slow external systems, short enough that operators don't ignore stuck sagas.

## §3 — Measurement

- Counter `ten_provisioning_saga_total{outcome=committed|rolled_back|timeout}`.
- Gauge `ten_provisioning_saga_in_progress_count` — should oscillate; sustained > 0 surfaces stuck.
- Histogram `ten_provisioning_duration_seconds`.

## §4 — Verification

- Integration test (T) — full success path commits.
- Chaos test (T) — fail step N; assert rollback of steps 1..N-1.
- Crash test (T) — kill mid-saga; restart; assert recovery.

## §5 — Failure handling

- Stuck > 10 min → manual operator action.
- Rollback failure → sev-2; operator-driven cleanup.
- Saga-journal corruption → sev-1; halt new provisions.

---

*End of NFR-TEN-001.*
