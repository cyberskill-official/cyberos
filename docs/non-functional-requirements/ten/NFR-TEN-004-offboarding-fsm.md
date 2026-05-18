---
id: NFR-TEN-004
title: "TEN 90-day offboarding FSM — termination MUST advance through declared states"
module: TEN
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% of offboarded tenants follow the 90-day FSM with declared milestones"
owner: CLO-Legal
created: 2026-05-18
related_frs: [FR-TEN-104]
---

## §1 — Statement (BCP-14 normative)

1. Tenant offboarding **MUST** advance through the FSM: `requested → grace_30d → final_export → cold_storage_60d → permanent_delete`.
2. The transition `final_export → cold_storage_60d` requires the tenant admin to confirm receipt of the signed bundle (`FR-TEN-105`).
3. `permanent_delete` **MUST** require an explicit attestation signed by CLO-Legal (`FR-TEN-106`).
4. Cancellation/reversal is possible up to `cold_storage_60d`; beyond that, restoration requires CEO override.
5. The FSM state **MUST** be visible in the admin SPA + emit audit rows on every transition.

## §2 — Why this constraint

Offboarding is a regulatory + commercial promise: tenant data is returned, then verifiably destroyed. The 90-day FSM provides the auditable structure: each phase is timed, each transition signed. Skipping a phase would violate data-handling obligations. The CEO-override on late restoration is the operational realism — sometimes a tenant comes back; we acknowledge that but log it.

## §3 — Measurement

- Per-state counter `ten_offboarding_state_total{state}`.
- Histogram `ten_offboarding_phase_duration_days{phase}`.
- Audit row per transition.

## §4 — Verification

- Integration test (T) — drive FSM end-to-end; assert all transitions logged.
- Reversal test (T) — cancel at grace_30d; assert clean restoration.
- Late-restoration test (T) — past cold_storage; assert CEO override required.

## §5 — Failure handling

- Stuck phase > 5d past expected duration → sev-3; operator investigates.
- Phase skipped → sev-1; FSM bypassed; halt + RCA.
- Restoration without CEO override → sev-1.

---

*End of NFR-TEN-004.*
