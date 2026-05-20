---
workflow_id: chief-information-security-officer/soc2-audit-readiness
workflow_version: 1.0.0
purpose: Assemble the SOC 2 evidence package and assess audit-readiness — control inventory, per-control evidence, gap analysis, remediation roadmap.
persona: cuo/chief-information-security-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_evidence,     source: last audit's soc2-evidence@1,                  format: soc2-evidence@1 }
  - { name: control_register,   source: GRC tool (Drata / Vanta / Secureframe / Tugboat), format: csv export }
  - { name: evidence_collection, source: GRC evidence repository,                       format: directory tree }
  - { name: tsc_scope,          source: cuo/ciso + cuo/cco-compliance,                 format: markdown brief (Type I/II + selected TSC) }

outputs:
  - { name: soc2_evidence,      format: soc2-evidence@1, recipient: cuo/ciso + cuo/cco-compliance + external auditor }

skill_chain:
  - { step: 1, skill: soc2-evidence-author, inputs_from: { prior_evidence: prior_evidence, control_register: control_register, evidence_collection: evidence_collection, tsc_scope: tsc_scope }, outputs_to: package_draft }
  - { step: 2, skill: soc2-evidence-audit,  inputs_from: package_draft, outputs_to: soc2_evidence }

escalates_to:
  - { persona: cuo/chief-compliance-officer, when: "gap analysis surfaces material weakness before audit window opens" }
  - { persona: cuo/chief-legal-officer,      when: "remediation requires updated customer contracts (BAA, DPA refresh)" }

consults:
  - { persona: cuo/chief-technology-officer,            when: "control gap requires engineering work (e.g. logging, RBAC)" }
  - { persona: cuo/chief-human-resources-officer,           when: "control gap is HR-side (background checks, access reviews)" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with soc2_evidence hash + control-count + evidence-freshness% + gap-count
  - HITL pause at step 2 on QA-EVIDENCE-AGE-001 (evidence stale > audit window) or QA-GAP-001 (gap without remediation)
---

# SOC 2 audit readiness — `chief-information-security-officer/soc2-audit-readiness`

CISO's annual SOC 2 evidence-readiness workflow. Combines prior evidence + control register + evidence repository + TSC scope into the auditor-facing package. Drives gap analysis + remediation before the audit window opens. Per AICPA TSC 2017 (with 2022 points-of-focus) + ISAE 3000/3402.

## When to invoke

- "Prep for SOC 2 audit"
- "Run SOC 2 readiness check"
- "Build the SOC 2 evidence package"

## How to invoke

```bash
cyberos-cuo run cuo/chief-information-security-officer/soc2-audit-readiness \
  --input prior_evidence=./compliance/2025/soc2-evidence.md \
  --input control_register=./grc/2026/controls.csv \
  --input evidence_collection=./grc/2026/evidence/ \
  --input tsc_scope=./compliance/2026/soc2-scope.md \
  --output-dir ./compliance/2026/soc2/
```

## Expected duration

- **Happy path:** 4-8 hours runtime + 6-12 weeks for full audit cycle (gap remediation + auditor fieldwork)
- **Worst case:** material weakness discovery triggers 1-2 quarter remediation before audit can proceed

## Skill chain

- **Step 1 `soc2-evidence-author`** — drafts per AICPA TSC structure: scope + TSC matrix + control inventory + evidence index + gap analysis + readiness statement.
- **Step 2 `soc2-evidence-audit`** — validates per `soc2_evidence_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-EVIDENCE-AGE-001 | Evidence > audit window | Operator re-collects |
| 2 | QA-GAP-001 | Gap without remediation | Escalate to CCO-Compliance |

## Cross-references
- `../../../../modules/cuo/README.md` §5.3 — CISO role profile
- `../../cco-compliance/README.md` — compliance-program peer
- `../../../skill/soc2-evidence-{author,audit}/SKILL.md`
- `../../../skill/compliance-program-{author,audit}/SKILL.md` — upstream year-program
