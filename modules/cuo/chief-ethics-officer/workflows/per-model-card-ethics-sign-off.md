---
workflow_id: chief-ethics-officer/per-model-card-ethics-sign-off
workflow_version: 1.0.0
purpose: Review and sign off on model cards for ethics-relevant sections — limits, ethical considerations, fairness, intended/out-of-scope use.
persona: cuo/chief-ethics-officer
cadence: per-event
status: shipped
pattern: linear  # gates chief-ai-officer/per-model-card-release — gate-side runs standalone

inputs:
  - { name: model_card_draft,      source: cuo/chief-ai-officer/per-model-card-release output, format: model-card@1 (pre-signoff) }
  - { name: bias_audit,            source: cuo/chief-ai-officer/per-model-bias-audit (paired), format: bias-audit@1 }
  - { name: ethics_review,         source: corresponding ethics-review@1 (if existed), format: ethics-review@1 }

outputs:
  - { name: signed_model_card,     format: model-card@1 (with ethics-signoff log), recipient: cuo/chief-ethics-officer + cuo/caio + model consumers }

skill_chain:
  - { step: 1, skill: model-card-author, inputs_from: { model_card_draft: model_card_draft, bias_audit: bias_audit, ethics_review: ethics_review }, outputs_to: card_draft }
  - { step: 2, skill: model-card-audit,  inputs_from: card_draft, outputs_to: signed_model_card }

escalates_to:
  - { persona: cuo/chief-ai-officer,           when: "ethics sign-off DECLINED — model release blocked" }
  - { persona: cuo/chief-legal-officer,      when: "out-of-scope-use section needs liability boundary review" }

consults:
  - { persona: cuo/chief-privacy-officer,    when: "training-data section needs personal-data note" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with signed_model_card hash + sign-off decision (APPROVE / DECLINE / CONDITIONAL)
  - HITL pause at step 2 on QA-ETHICS-001 (ethics section incomplete) or QA-LIMITS-001 (limits section ambiguous)
---

# Per model-card ethics sign-off — `chief-ethics-officer/per-model-card-ethics-sign-off`

Chief Ethics Officer's sign-off workflow for model cards. Augments CAIO's `per-model-card-release` with ethics-side rigor on limits + ethical considerations + intended/out-of-scope-use sections. Mandatory gate for model release.

## When to invoke

- "Ethics sign-off for [model card]"
- "Review the [model] card for ethics"
- "Sign off on model release"

## How to invoke

```bash
cyberos-cuo run cuo/chief-ethics-officer/per-model-card-ethics-sign-off \
  --input model_card_draft=./models/2026-recommender/card-draft.md \
  --input bias_audit=./models/2026-recommender/bias-audit.md \
  --input ethics_review=./ethics/use-cases/2026-recommender/review.md \
  --output-dir ./models/2026-recommender/ethics-signoff/
```

## Expected duration

- **Happy path:** 1-2 hours runtime + 1-3 day review cycle
- **Worst case:** DECLINE may delay release 1-2 quarter

## Skill chain

- **Step 1 `model-card-author`** — augments with ethics review fields.
- **Step 2 `model-card-audit`** — validates per `model_card_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-ETHICS-001 | Ethics section incomplete | Operator extends |
| 2 | QA-LIMITS-001 | Limits ambiguous | Operator tightens |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.6 — Chief Ethics Officer role profile
- `../../chief-ai-officer/workflows/per-model-card-release.md` — upstream peer
- `../../../skill/model-card-{author,audit}/SKILL.md`
