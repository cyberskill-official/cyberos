---
id: NFR-HR-005
title: "HR policy-version compatibility — policy changes MUST be backward-compatible for in-flight cases"
module: HR
category: compliance
priority: MUST
verification: T
phase: P1
slo: "100% of in-flight HR cases preserve their original policy version"
owner: CHRO
created: 2026-05-18
related_tasks: [TASK-HR-005]
---

## §1 — Statement (BCP-14 normative)

1. HR policies (leave, working hours, OT rates) **MUST** be versioned; in-flight cases (a leave request submitted under v1) **MUST** be evaluated against v1 even after v2 ships.
2. Policy version is stamped on every case at creation time; the stamp is immutable.
3. New cases use the latest active policy version.
4. Policy archival **MUST** preserve historical versions for ≥ 7 years.
5. Cross-version reporting **MUST** be possible — operators can see "cases under v1" vs "cases under v2."

## §2 — Why this constraint

Policies change over time (statutory updates, company evolution). Without versioning, an old leave request gets re-evaluated under new rules — confusing + possibly unfair. The stamp-on-create pattern freezes the rule the case was made under. The cross-version reporting helps CHRO plan policy rollouts.

## §3 — Measurement

- Counter `hr_policy_version_mismatch_total` — case evaluated under wrong version; must be 0.
- Gauge `hr_active_policy_version{kind}`.
- Per-version case count.

## §4 — Verification

- Integration test (T) — submit case under v1; ship v2; assert v1 evaluation persists.
- CI gate (T) — every case has version stamp.

## §5 — Failure handling

- Mismatch detected → sev-2; case re-evaluated correctly + audit.
- Stamp missing on new case → reject.
- Retention violation → sev-2.

---

*End of NFR-HR-005.*
