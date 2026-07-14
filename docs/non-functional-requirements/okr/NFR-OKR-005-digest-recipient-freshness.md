---
id: NFR-OKR-005
title: "OKR digest-recipient freshness — Monday digest list MUST match current tenant roster"
module: OKR
category: maintainability
priority: SHOULD
verification: T
phase: P1
slo: "100% of digest recipient lists reflect terminations + role changes within 24h"
owner: CHRO
created: 2026-05-18
related_tasks: [TASK-OKR-006]
---

## §1 — Statement (BCP-14 normative)

1. The Monday OKR digest (`TASK-OKR-006`) **MUST** be delivered only to currently-active members; terminated members **MUST** be removed within 24h of HR transition.
2. Role-based recipient rules (e.g., "C-level + VPs") **MUST** auto-update when a member's role changes.
3. Manual recipient overrides **MUST** be allowed but logged.
4. Digest delivery to a removed recipient **MUST** trigger sev-3 (data leak in transit).
5. Reconciliation: nightly cross-check recipient list against HR active roster.

## §2 — Why this constraint

A digest to a terminated employee is a small but real data exposure — they shouldn't see active OKR data after departure. The 24h SLA matches RES-007 termination cascade. The role-based rules auto-update prevents drift. Manual overrides + logs keep the exception path auditable.

## §3 — Measurement

- Counter `okr_digest_sent_to_terminated_total` — must be 0.
- Counter `okr_digest_recipient_override_total`.
- Daily reconciliation result.

## §4 — Verification

- Integration test (T) — terminate member; assert removed from next digest.
- Snapshot test (T) — role change reflects in recipients.
- Reconciliation (T) — daily run.

## §5 — Failure handling

- Sent to terminated → sev-2; data exposure.
- Reconciliation diff → sev-3; investigate trigger.
- Manual override misuse → governance review.

---

*End of NFR-OKR-005.*
