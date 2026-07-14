---
id: NFR-CRM-001
title: "CRM account-contact-deal schema invariants — required fields + closed enum + relational integrity"
module: CRM
category: maintainability
priority: MUST
verification: T
phase: P0
slo: "100% of account/contact/deal rows satisfy schema + relational invariants"
owner: CSO-Sales
created: 2026-05-18
related_tasks: [TASK-CRM-001]
---

## §1 — Statement (BCP-14 normative)

1. The three core CRM tables (`account`, `contact`, `deal`) **MUST** enforce required fields + closed enum values declared in `modules/crm/schema/`.
2. `contact.account_id` **MUST** reference a real `account`; cascading delete from account is forbidden — orphan-detection runs instead.
3. `deal.account_id` AND `deal.primary_contact_id` **MUST** both resolve; both refs are mandatory for sales-pipeline visibility.
4. Schema migrations **MUST** preserve existing rows; column drops require migration plan + CSO-Sales approval.
5. Soft-delete is the default; hard-delete requires explicit operator action.

## §2 — Why this constraint

CRM data is the platform's pipeline-of-record. Relational integrity is what makes pipeline reporting trustworthy. The no-cascade rule prevents losing contacts when an account is deleted in error — orphan detection lets operators reattach. Soft-delete is the right default for revenue-related data — undo possible.

## §3 — Measurement

- CI metric `crm_schema_violation_count` — must be 0.
- Counter `crm_orphan_contact_total`, `crm_orphan_deal_total`.
- Hourly orphan-detection scan.

## §4 — Verification

- Unit test (T) — invalid rows rejected.
- Integration test (T) — orphan detection triggers.
- CI gate (T) — schema migration plan.

## §5 — Failure handling

- Schema violation → reject + clear error.
- Orphan detected → flag + operator notified.
- Hard-delete attempt without confirmation → block.

---

*End of NFR-CRM-001.*
