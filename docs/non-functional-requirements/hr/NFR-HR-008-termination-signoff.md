---
id: NFR-HR-008
title: "HR termination signoff window — termination MUST be signed by CHRO + line manager within 72h"
module: HR
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% of terminations carry both signatures within 72h; expired approvals require resubmission"
owner: CHRO
created: 2026-05-18
related_tasks: [TASK-HR-009]
---

## §1 — Statement (BCP-14 normative)

1. Termination workflow **MUST** require both CHRO + line-manager signatures within a 72h rolling window.
2. The 72h window starts from first signature; expiry forces re-submission.
3. Signature row carries `{signer_id, signed_at, termination_kind, effective_date}`.
4. Termination cannot proceed past signature; downstream cascades (RES allocation removal, AUTH revoke) wait on the gate.
5. Termination kinds = `voluntary, involuntary, redundancy, end_of_contract`; closed enum.

## §2 — Why this constraint

Termination is high-stakes: legal risk if mishandled, financial risk via severance, reputational risk if rushed. Dual signature + window enforces deliberate process. The cascade-gate prevents premature deprovisioning (locked out of email before official sign).

## §3 — Measurement

- Counter `hr_termination_total{kind, signers_count}`.
- Histogram `hr_termination_signoff_hours`.
- Counter `hr_termination_resubmit_total`.

## §4 — Verification

- Integration test (T) — single-signer → reject.
- Stale-window test (T) — exceed 72h → resubmit.
- Cascade test (T) — pre-signoff deprovisioning blocked.

## §5 — Failure handling

- Single-signer attempt → block.
- 72h expired → resubmit.
- Cascade leak → sev-2; deprovisioning before sign.

---

*End of NFR-HR-008.*
