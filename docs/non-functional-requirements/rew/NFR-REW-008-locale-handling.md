---
id: NFR-REW-008
title: "REW locale handling — payslip currency + tax tables MUST match member residency"
module: REW
category: compliance
priority: MUST
verification: T
phase: P1
slo: "100% of payslips use the correct VN tax tables; non-VN residents flagged as out-of-scope"
owner: CFO
created: 2026-05-18
related_frs: [FR-REW-004, FR-REW-006]
---

## §1 — Statement (BCP-14 normative)

1. The REW compute path **MUST** apply VN PIT + SI tables only to members with `residency = VN`; non-VN residents are flagged out-of-scope and excluded from the standard compute.
2. The payslip PDF **MUST** be generated in Vietnamese with VND amounts; bilingual (VN+EN) PDF is optional but bilingual amounts MUST still be VND.
3. Currency formatting **MUST** follow Vietnamese conventions: `1.234.567 ₫` (dot thousands, no decimals).
4. The parameter version (`FR-REW-002`) **MUST** record which tax tables were applied; cross-version drift is detectable.
5. Out-of-scope members **MUST** still appear in the period's roster with `excluded_reason = non-vn-residency` for completeness.

## §2 — Why this constraint

CyberOS is VN-locale-first. Applying VN tax tables to a non-VN resident would produce incorrect deductions + regulatory misreporting. The currency formatting matters for legal/government filings (e.g., payslips submitted as evidence in disputes). The roster-with-reason rule ensures the out-of-scope set is visible — silently dropping non-VN members would create blind spots.

## §3 — Measurement

- Counter `rew_locale_mismatch_total{member_residency, expected}` — must be 0.
- CI gate: every payslip PDF for VN-resident has VND format.
- Audit row for every excluded member.

## §4 — Verification

- Unit test (T) — non-VN member → excluded with reason.
- Snapshot test (T) — fixture VN payslip; assert PDF format matches expected.
- CI gate (T) — locale-format lint over the PDF renderer.

## §5 — Failure handling

- Locale mismatch on payslip → block emit.
- Wrong currency format → sev-3 cosmetic but a compliance smell.
- Drop of non-VN member without `excluded_reason` → sev-2; roster discipline broken.

---

*End of NFR-REW-008.*
