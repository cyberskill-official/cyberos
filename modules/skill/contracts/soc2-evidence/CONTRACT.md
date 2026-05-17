# soc2-evidence@1 — SOC 2 evidence package

A `soc2-evidence@1` artefact assembles the SOC 2 Type I or Type II evidence package — control inventory mapped to TSC (Trust Service Criteria), evidence per control with collection date + period covered, auditor-facing index, gap analysis. Per AICPA TSC 2017 (with 2022 points-of-focus) + ISAE 3000/3402.

## Required sections (template.md H2 order)
1. Audit scope (Type I / Type II, period covered, in-scope systems)
2. TSC coverage matrix (Security mandatory + selected Availability / Confidentiality / Processing Integrity / Privacy)
3. Control inventory (per control: control-objective / evidence-source / collection-frequency / responsible-owner)
4. Evidence index (file-name → control-id mapping)
5. Gap analysis (controls without evidence / evidence stale)
6. Auditor-facing readiness statement
7. Remediation plan for gaps

## Citations
- AICPA Trust Services Criteria 2017 (with 2022 points-of-focus)
- ISAE 3000 / ISAE 3402
- AICPA SOC 2 Description Criteria

## KPI
- % controls with current evidence (target 100% before audit window opens)
- Mean evidence age (days)
- Auditor opinion (unqualified / qualified)
