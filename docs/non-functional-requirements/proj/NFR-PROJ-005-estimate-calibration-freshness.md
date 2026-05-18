---
id: NFR-PROJ-005
title: "PROJ estimate-calibration freshness — calibration data MUST refresh weekly"
module: PROJ
category: observability
priority: SHOULD
verification: T
phase: P1
slo: "Calibration freshness ≤ 7 days; 100% of teams refreshed within their cadence"
owner: CTO
created: 2026-05-18
related_frs: [FR-PROJ-013]
---

## §1 — Statement (BCP-14 normative)

1. Per-team estimate calibration (`FR-PROJ-013`) **MUST** refresh weekly using closed issues from the prior cycle window.
2. Calibration outputs **MUST** include the `(estimate, actual, ratio)` triple per closed issue plus the team's rolling calibration coefficient.
3. Teams with fewer than 5 closed issues in the window **MUST** show a "low-data" indicator instead of an unreliable coefficient.
4. Calibration data **MUST** be visible in the cycle review draft + issue brief modal.
5. Stale calibration (no refresh in > 14 days) **MUST** trigger a sev-3 alert visible to the team lead.

## §2 — Why this constraint

Estimate calibration is the team's self-correcting loop on planning quality. Stale data means decisions are based on coefficients that no longer reflect reality. The 5-issue floor prevents noise from making coefficients look real. Visibility in the brief modal puts the data where decisions happen.

## §3 — Measurement

- Gauge `proj_team_calibration_age_days{team}` — must be ≤ 7.
- Counter `proj_calibration_refresh_attempt_total{result}`.
- Per-team rolling coefficient + standard deviation.

## §4 — Verification

- Integration test (T) — drive 10 closed issues; assert calibration refresh produces ratio.
- Snapshot test (T) — low-data team shows indicator, not coefficient.
- Visibility check (T) — brief modal displays calibration.

## §5 — Failure handling

- Stale > 14d → sev-3 alert team lead.
- Coefficient drift > 50% week-over-week → sev-3; possible data quality issue.
- Low-data state sustained → product feedback to encourage estimation practice.

---

*End of NFR-PROJ-005.*
