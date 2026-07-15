---
id: NFR-OKR-001
title: "OKR objective-KR schema invariants — KR types closed enum; numeric KRs measurable"
module: OKR
category: maintainability
priority: MUST
verification: T
phase: P0
slo: "100% of KRs use the closed enum types; 100% of numeric KRs are computable from declared source"
owner: CEO
created: 2026-05-18
related_tasks: [TASK-OKR-001, TASK-OKR-002]
---

## §1 — Statement (BCP-14 normative)

1. KR types **MUST** be a closed enum: `numeric`, `binary`, `milestone-progress`, `custom-sql`. Ad-hoc types are rejected at create time.
2. Numeric KRs **MUST** declare a `progress_source` (`TASK-OKR-003`) that resolves to a numeric value; non-computable sources are rejected.
3. Every KR **MUST** have a unique slug + reference exactly one parent objective.
4. Objective–KR relationship **MUST NOT** be many-to-many; each KR belongs to one objective.
5. Schema migrations **MUST** preserve prior values; KR-type changes require explicit migration plan.

## §2 — Why this constraint

OKR semantics depend on KRs being measurable. The closed enum forbids "loose" KRs that resist measurement. The progress-source declaration is the bridge from "we wanted to measure X" to "we measure X by query Y." Many-to-many KRs would create cascade confusion in progress rollups.

## §3 — Measurement

- CI metric `okr_kr_invalid_type_count` — must be 0.
- Counter `okr_kr_unresolvable_source_total` — surfaces broken data wiring.
- Schema drift gauge.

## §4 — Verification

- Unit test (T) — invalid type → reject.
- CI gate (T) — all KRs in catalog conform.
- Integration test (T) — progress source resolves.

## §5 — Failure handling

- Invalid type at create → reject + clear error.
- Unresolvable source → KR shows "data issue" instead of progress.
- Schema migration without plan → CI block.

---

*End of NFR-OKR-001.*
