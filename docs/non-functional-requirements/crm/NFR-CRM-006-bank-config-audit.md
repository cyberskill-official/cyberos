---
id: NFR-CRM-006
title: "CRM bank-config audit — account banking changes MUST emit signed audit row"
module: CRM
category: security
priority: MUST
verification: T
phase: P0
slo: "100% of account-bank changes emit a signed audit row + trigger CFO notification"
owner: CFO
created: 2026-05-18
related_tasks: [TASK-CRM-009]
---

## §1 — Statement (BCP-14 normative)

1. Changes to an account's banking info (bank, branch, account number, beneficiary name) **MUST** emit a signed audit row with `{actor_id, account_id, old_banking_hash, new_banking_hash, changed_at}`.
2. The audit row **MUST** notify the CFO + the account owner within 1 minute of change.
3. Banking changes from a non-privileged role **MUST** require additional CFO approval before taking effect.
4. The first 24 hours after a banking change **MUST** auto-flag outgoing payments to that account for human review.
5. Bulk banking imports **MUST** require CFO signoff per batch.

## §2 — Why this constraint

Banking change is the most common fraud vector ("invoice from CEO to wire money to X"). The signed audit + notification + 24h flag + CFO approval combination makes the platform an extremely poor fraud target. The bulk-batch sign prevents mass tampering.

## §3 — Measurement

- Counter `crm_bank_change_total{actor_role}`.
- Counter `crm_bank_change_flagged_payment_total`.
- Audit row per change.

## §4 — Verification

- Integration test (T) — change banking; assert audit + notification.
- Integration test (T) — payment within 24h flagged.
- Pen test (T, quarterly) — fraud-pattern probes.

## §5 — Failure handling

- Audit row missing → sev-1; halt; investigate.
- Payment > flag without review → sev-2; possible fraud.
- Bulk import without signoff → block.

---

*End of NFR-CRM-006.*
