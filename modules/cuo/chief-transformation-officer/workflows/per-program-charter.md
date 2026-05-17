---
workflow_id: chief-transformation-officer/per-program-charter
workflow_version: 1.0.0
purpose: Charter a transformation program — scope, owner, value, milestones, governance, change-impact.
persona: cuo/chief-transformation-officer
cadence: per-event
status: shipped

inputs:
  - { name: program_brief,         source: requestor (sponsor + initial scope), format: markdown }
  - { name: roadmap_context,       source: cuo/chief-transformation-officer/annual-transformation-roadmap, format: transformation-roadmap@1 }
  - { name: function_inputs,       source: impacted function heads, format: markdown (one per function) }

outputs:
  - { name: program_charter,       format: program-charter@1, recipient: cuo/chief-transformation-officer + cuo/ceo + sponsor + impacted function heads }

skill_chain:
  - { step: 1, skill: program-charter-author, inputs_from: { program_brief: program_brief, roadmap_context: roadmap_context, function_inputs: function_inputs }, outputs_to: charter_draft }
  - { step: 2, skill: program-charter-audit,  inputs_from: charter_draft, outputs_to: program_charter }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "charter scope conflicts with active OKRs OR exceeds $500K" }

consults:
  - { persona: cuo/chief-financial-officer,            when: "budget alignment needed" }
  - { persona: cuo/chief-human-resources-officer,           when: "headcount + change-management impact" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with program_charter hash + program scope + value hypothesis
  - HITL pause at step 2 on QA-OWNER-001 (no named owner) or QA-VALUE-001 (no value hypothesis)
---

# Per program charter — `chief-transformation-officer/per-program-charter`

Chief Transformation Officer's per-program charter workflow. Per PMI program-charter template + Bain agile-PMO. Triggered per transformation program kickoff.

## When to invoke

- "Charter the [program]"
- "Program kickoff for [initiative]"
- "Set up program charter"

## How to invoke

```bash
cyberos-cuo run cuo/chief-transformation-officer/per-program-charter \
  --input program_brief=./transformation/programs/2026-erp-rollout/brief.md \
  --input roadmap_context=./transformation/2026/roadmap.md \
  --input function_inputs=./transformation/programs/2026-erp-rollout/function-inputs/ \
  --output-dir ./transformation/programs/2026-erp-rollout/charter/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + 1-2 weeks for cross-function alignment
- **Worst case:** scope-conflict adds 1-2 weeks of CEO arbitration

## Skill chain

- **Step 1 `program-charter-author`** — drafts per PMI + Bain agile-PMO.
- **Step 2 `program-charter-audit`** — validates per `program_charter_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-OWNER-001 | No named owner | Operator assigns |
| 2 | QA-VALUE-001 | No value hypothesis | Operator drafts |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.7 — Chief Transformation Officer role profile
- `./annual-transformation-roadmap.md` — upstream parent
- `../../../skill/program-charter-{author,audit}/SKILL.md`
