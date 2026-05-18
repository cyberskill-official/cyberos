---
id: NFR-HR-006
title: "HR accrual-correction audit — every manual leave adjustment MUST require reason + approver"
module: HR
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% of accrual corrections carry approver signature + structured reason"
owner: CHRO
created: 2026-05-18
related_frs: [FR-HR-006]
---

## §1 — Statement (BCP-14 normative)

1. Manual leave-accrual corrections (overrides of the automatic accrual) **MUST** be approved by a member with `hr:leave:adjust` permission.
2. Each correction row carries `{member_id, leave_type, delta_days, reason_code, approver_id, approved_at, narrative}`.
3. Self-corrections (member adjusting own balance) **MUST** be blocked unless additional CHRO approval is recorded.
4. Reason codes are a closed enum maintained by CHRO: `migration, correction, settlement, error_fix, bonus_award, regulatory_change`.
5. Quarterly review: CHRO inspects all corrections; persistent patterns inform process changes.

## §2 — Why this constraint

Leave-balance manipulation is a fraud + favoritism risk. Approval + reason + structured-enum makes every correction defensible. The self-correction block prevents self-dealing. Quarterly review catches abuse patterns.

## §3 — Measurement

- Counter `hr_accrual_correction_total{reason_code, approver_role}`.
- Counter `hr_self_correction_blocked_total`.
- Quarterly review report.

## §4 — Verification

- Integration test (T) — correction without approver → reject.
- Integration test (T) — self-correction → blocked.
- Property test (T) — reason-code enum.

## §5 — Failure handling

- Unapproved attempt → block + sev-3.
- Self-correction → block + audit.
- Quarterly review pattern → process review.

---

*End of NFR-HR-006.*
