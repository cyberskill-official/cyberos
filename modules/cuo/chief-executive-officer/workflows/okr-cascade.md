---
workflow_id: chief-executive-officer/okr-cascade
workflow_version: 1.0.0
purpose: Drive the company-OKR cascade from the CEO's quarterly vision down to function-level OKRs.
persona: cuo/chief-executive-officer
cadence: quarterly
status: shipped

inputs:
  - { name: vision_brief,       source: workflow-caller (CEO's 3-5 quarterly priorities), format: markdown brief }
  - { name: prior_okrs,         source: last quarter's objectives-and-key-results-set@1 artefact,                 format: objectives-and-key-results-set@1 }
  - { name: function_inputs,    source: each function head's draft OKRs,                   format: markdown (one per function) }

outputs:
  - { name: company_okrs,       format: okr-set@1, recipient: entire C-suite + all-hands }

skill_chain:
  - { step: 1, skill: objectives-and-key-results-set-author, inputs_from: { vision_brief: vision_brief, prior_okrs: prior_okrs, function_inputs: function_inputs }, outputs_to: okrs_draft }
  - { step: 2, skill: objectives-and-key-results-set-audit,  inputs_from: okrs_draft, outputs_to: company_okrs }

escalates_to:
  - { persona: cuo/chief-of-staff, when: "objectives-and-key-results-set-audit fires QA-MEASURE-001 — KR not measurable; CoS owns OKR-governance fix" }

consults:
  - { persona: cuo/chief-financial-officer, when: "OKRs imply >25% reallocation of the quarter's budget envelope" }
  - { persona: cuo/chief-product-officer, when: "product OKRs need refinement against current PRDs" }

audit_hooks:
  - each step emits artefact_write to BRAIN audit chain
  - workflow_complete row on PASS with company_okrs hash + per-function KR count
  - HITL pause at step 2 if objectives-and-key-results-set-audit fires on overlapping KRs across functions
---

# Quarterly OKR cascade — `chief-executive-officer/okr-cascade`

CEO's quarterly OKR-cascade workflow. Takes the CEO's vision brief plus each function head's draft, produces a company-level OKR set with function-level cascades, and audits for the Doerr / Grove `measurable + ambitious + aligned` test.

## When to invoke

- "Run the Q<n> OKR cascade"
- "Set company OKRs for next quarter"
- "Cascade my priorities into function OKRs"

## How to invoke

```bash
cyberos-cuo run cuo/chief-executive-officer/okr-cascade \
  --input vision_brief=./engagements/2026-Q2/vision.md \
  --input prior_okrs=./engagements/2026-Q1/okrs/final.md \
  --input function_inputs=./engagements/2026-Q2/function-okrs/ \
  --output-dir ./engagements/2026-Q2/okrs/
```

## Expected duration

- **Happy path:** 30 min runtime + 2-3 business days for function-head round-trip
- **Worst case:** OKR-conflict resolution may require live exec-staff meeting before resuming

## Skill chain

- **Step 1 `objectives-and-key-results-set-author`** — drafts company OKRs per Doerr OKR template (Objectives qualitative + 3-5 Key Results quantitative). Pause for PLAN approval.
- **Step 2 `objectives-and-key-results-set-audit`** — validates against `okr_set_rubric@1.0` (each O has 3-5 measurable KRs; KR thresholds are ambitious; cross-function alignment check).

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 1 | BOOT-001 | Vision brief missing or single-bullet | CEO supplies expanded brief |
| 2 | QA-MEASURE-001 | KR not measurable | Escalate to chief-of-staff |
| 2 | QA-CONFLICT-001 | Two functions claim conflicting KRs | Live exec-staff resolution; resume |

## Cross-references
- `../README.md` §5.1 — output type "OKRs cascade"
- `../../../docs/The C-Suite Reference.md` §5.1
- `../../chief-of-staff/workflows/quarterly-okr-cascade.md` — peer workflow CoS owns the governance for
- `../../../skill/okr-set-{author,audit}/SKILL.md`
