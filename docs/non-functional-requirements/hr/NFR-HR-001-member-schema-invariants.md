---
id: NFR-HR-001
title: "HR member schema invariants — required fields + closed enum types + unique member_id"
module: HR
category: maintainability
priority: MUST
verification: T
phase: P0
slo: "100% of member rows satisfy required-field + uniqueness invariants"
owner: CHRO
created: 2026-05-18
related_frs: [FR-HR-001]
---

## §1 — Statement (BCP-14 normative)

1. The HR `member` table **MUST** enforce: required fields (`member_id, full_name, email, employment_status, hire_date, contract_type`); closed enum for `employment_status` and `contract_type`.
2. `member_id` **MUST** be unique per tenant + immutable (no renames).
3. `email` **MUST** be unique per tenant + valid format.
4. `hire_date` **MUST** be ≤ today; future-dated requires explicit pending-hire flow.
5. Schema migrations **MUST** preserve all existing rows; column drops require explicit migration approval.

## §2 — Why this constraint

The member table is the source of truth for every other module's "who is this person?" lookup. Schema drift here cascades into broken joins everywhere. The closed enums prevent operator-introduced values that break downstream switch statements. Member-id immutability is critical: a renamed id orphans every audit row referencing it.

## §3 — Measurement

- CI metric `hr_member_schema_violation_count` — must be 0.
- Counter `hr_member_id_collision_total` — must be 0.
- Schema migration audit.

## §4 — Verification

- Unit test (T) — every field rule.
- CI gate (T) — schema validation.
- Property test (T) — random member creates; assert invariants.

## §5 — Failure handling

- Schema violation at insert → reject.
- ID collision → reject + investigate.
- Migration drift → sev-1; halt; investigate.

---

*End of NFR-HR-001.*
