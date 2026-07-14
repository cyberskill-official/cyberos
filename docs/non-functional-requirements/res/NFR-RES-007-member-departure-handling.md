---
id: NFR-RES-007
title: "RES member-departure handling — terminated members MUST be removed from active allocations within 24h"
module: RES
category: maintainability
priority: MUST
verification: T
phase: P0
slo: "100% of terminated members removed from active allocations within 24h of termination effective date"
owner: COO
created: 2026-05-18
related_tasks: [TASK-RES-002]
---

## §1 — Statement (BCP-14 normative)

1. When a member's HR record transitions to `terminated` with effective date D, all active allocations **MUST** be ended by D + 24h.
2. Future allocations beyond D **MUST** be either reassigned to another member OR explicitly canceled — they cannot remain dangling on the terminated member.
3. The departure trigger fires automatically from the HR module event; manual intervention is the exception path.
4. Historical allocations (period ended before D) **MUST** be preserved as-is (no rewrite).
5. Allocation removal **MUST** emit a per-allocation audit row with reason `member_terminated`.

## §2 — Why this constraint

A terminated member with active allocations corrupts capacity reports (showing capacity that doesn't exist). The 24h SLA is the operational floor — fast enough that reports stay accurate, slow enough that errors in termination can be unwound. The historical-immutability rule preserves accurate history.

## §3 — Measurement

- Counter `res_terminated_member_active_alloc_total` — must trend to 0 within 24h.
- Histogram `res_termination_cascade_duration_hours`.
- Audit row count = allocation count.

## §4 — Verification

- Integration test (T) — terminate member; assert allocations cleared within 24h.
- Property test (T) — random terminations; assert future allocations handled.

## §5 — Failure handling

- Cascade > 24h → sev-3; investigate trigger.
- Dangling future alloc → sev-2; manual cleanup.
- Historical rewrite → sev-1; immutability broken.

---

*End of NFR-RES-007.*
