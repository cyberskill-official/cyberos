---
id: NFR-HR-003
title: "HR CCCD-photo access logging — every read MUST be logged with reason; sev-1 on bypass"
module: HR
category: privacy
priority: MUST
verification: T
phase: P0
slo: "100% of CCCD-photo reads carry a logged access reason; 0 bypassed reads"
owner: CPO-Privacy
created: 2026-05-18
related_tasks: [TASK-HR-003]
---

## §1 — Statement (BCP-14 normative)

1. Every read of a CCCD (Vietnamese national ID) photo **MUST** be logged with `{accessor_id, accessed_at, member_id, reason_code, justification}`.
2. CCCD photos **MUST** be encrypted at rest with a per-tenant KMS key; decryption only via the access-logging gate.
3. Direct DB queries that bypass the gate **MUST** be impossible — DB role isolation prevents application code from reading the cipher.
4. Access without a recorded reason **MUST** raise a sev-1 (PII access violation).
5. Annual access review by CPO-Privacy — sample 10% of access logs to verify reason validity.

## §2 — Why this constraint

CCCD photos are the most sensitive PII the platform holds. Unlogged access is both legal liability (Vietnam's Personal Data Decree) + audit liability. The encrypt-at-rest + DB-isolation + log-gated pattern is the textbook PII handling stack. The sev-1 on bypass is the appropriate severity — this is the platform's keep-data-safe contract.

## §3 — Measurement

- Counter `hr_cccd_access_total{reason_code}`.
- Counter `hr_cccd_access_no_reason_total` — must be 0.
- Counter `hr_cccd_direct_db_attempt_total` — must be 0.

## §4 — Verification

- Integration test (T) — gate-bypass attempt → block.
- Pen test (T, quarterly) — direct DB; assert blocked.
- Annual review (T).

## §5 — Failure handling

- Bypass attempt → sev-1; PII contract broken.
- Unlogged access → sev-1.
- Annual review finds invalid reasons → CPO-Privacy retunes reason-code catalog.

---

*End of NFR-HR-003.*
