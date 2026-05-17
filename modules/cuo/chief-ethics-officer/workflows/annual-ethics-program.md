---
workflow_id: chief-ethics-officer/annual-ethics-program
workflow_version: 1.0.0
purpose: Refresh the annual ethics program — values + principles, decision-rights + escalations, training program, incident-handling, transparency.
persona: cuo/chief-ethics-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_program,         source: last year's compliance-program@1 (ethics chapter), format: compliance-program@1 }
  - { name: review_corpus,         source: 12 months of ethics-review@1 outputs, format: ethics-review@1 (multiple) }
  - { name: bias_history,          source: 4 quarters of portfolio bias-audit@1, format: bias-audit@1 (4) }
  - { name: ceo_priorities,        source: cuo/ceo (vision brief), format: markdown }

outputs:
  - { name: ethics_program,        format: compliance-program@1, recipient: cuo/chief-ethics-officer + cuo/ceo + cuo/clo-legal + Board (annual ethics chapter) }

skill_chain:
  - { step: 1, skill: compliance-program-author, inputs_from: { prior_program: prior_program, review_corpus: review_corpus, bias_history: bias_history, ceo_priorities: ceo_priorities }, outputs_to: program_draft }
  - { step: 2, skill: compliance-program-audit,  inputs_from: program_draft, outputs_to: ethics_program }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "program proposes new ethics principle that conflicts with active business" }
  - { persona: cuo/chief-legal-officer,      when: "program intersects regulatory ethics obligations (FDA / EU AI Act / etc.)" }

consults:
  - { persona: cuo/chief-human-resources-officer,           when: "training program needs HR delivery framework" }
  - { persona: cuo/chief-communications-officer, when: "transparency reporting needs external positioning" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with ethics_program hash + principle count + training hours target + transparency commitments
  - HITL pause at step 2 on QA-PRINCIPLE-001 (principle without operational test) or QA-TRAINING-001 (training plan vague)
---

# Annual ethics program — `chief-ethics-officer/annual-ethics-program`

Chief Ethics Officer's annual ethics-program refresh. Per IEEE Ethically Aligned Design + Markkula Center + Anthropic Acceptable Use Policy + Stanford HAI policy framework. Defines values + decision-rights + training + incident-handling + transparency commitments for the year.

## When to invoke

- "Build the 2026 ethics program"
- "Annual ethics refresh"
- "Update ethics principles + training"

## How to invoke

```bash
cyberos-cuo run cuo/chief-ethics-officer/annual-ethics-program \
  --input prior_program=./ethics/2025/program.md \
  --input review_corpus=./ethics/2025/reviews/ \
  --input bias_history=./ethics/portfolio/2025/ \
  --input ceo_priorities=./engagements/2026/vision.md \
  --output-dir ./ethics/2026/program/
```

## Expected duration

- **Happy path:** 8-16 hours runtime + 6-12 weeks for cross-function + Board review
- **Worst case:** new principle requiring business change adds 1-2 quarter

## Skill chain

- **Step 1 `compliance-program-author`** — drafts ethics-program structure per IEEE EAD + Markkula + Stanford HAI.
- **Step 2 `compliance-program-audit`** — validates per `compliance_program_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-PRINCIPLE-001 | Principle no operational test | Operator clarifies |
| 2 | QA-TRAINING-001 | Training plan vague | Operator extends |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.6 — Chief Ethics Officer role profile
- `./per-use-case-ethics-review.md` — operational peer
- `../../../skill/compliance-program-{author,audit}/SKILL.md`
