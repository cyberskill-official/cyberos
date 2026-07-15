---
id: NFR-CRM-005
title: "CRM MST validation freshness — VN business accounts MUST have MST validated within 90 days"
module: CRM
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% of VN business accounts carry a GDT MST validation within last 90 days"
owner: CFO
created: 2026-05-18
related_tasks: [TASK-CRM-008, TASK-CRM-003]
---

## §1 — Statement (BCP-14 normative)

1. Accounts marked `account_type: vn_business` **MUST** have their MST (tax ID) validated against the GDT lookup within the last 90 days.
2. The validation result **MUST** be persisted: `{mst, validated_at, gdt_response_payload, company_name_matched}`.
3. Failed validations (MST not found, suspended) **MUST** flag the account; deals on flagged accounts require CFO override.
4. Validation refresh runs automatically on a 60-day rotation; manual revalidation always available.
5. Bulk-revalidation on account creation/import takes priority over scheduled refresh.

## §2 — Why this constraint

Invoicing a non-existent or suspended business is a regulatory + recovery issue. The 90-day freshness floor matches GDT's typical update cadence. The flag-and-override pattern preserves agility for known-but-temporarily-suspended cases. The 60-day refresh provides safety margin under the 90-day SLO.

## §3 — Measurement

- Gauge `crm_account_mst_age_days{tenant}` — max should be < 90.
- Counter `crm_mst_validation_fail_total{reason}`.
- Counter `crm_flagged_account_deal_block_total`.

## §4 — Verification

- Integration test (T) — VN business account; assert MST validated.
- Sandbox test (T) — invalid MST → flagged.
- Property test (T) — refresh cron triggers.

## §5 — Failure handling

- Age > 90d → SHOULD revalidate; > 120d sev-3.
- Flagged account deal close attempt → CFO override required.
- GDT unavailable → keep prior validation valid; alert if > 30d unreachable.

---

*End of NFR-CRM-005.*
