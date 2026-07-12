---
workflow_id: chief-ai-officer/per-model-card-release
workflow_version: 1.0.0
purpose: Author a model card for a new or updated production ML model — intended use, training data, performance, limits, ethical considerations.
persona: cuo/chief-ai-officer
cadence: per-event
status: shipped
pattern: sequential_approval
gates:
  - { approver_persona: chief-ethics-officer, approver_workflow: per-model-card-ethics-sign-off }

inputs:
  - { name: model_brief,           source: ML team (model purpose + version), format: markdown }
  - { name: training_data_summary, source: data team (sources + bias-audit hooks), format: markdown }
  - { name: eval_results,          source: evaluation harness output, format: csv / markdown }
  - { name: prior_card,            source: prior version's model-card@1 (if any), format: model-card@1 }

outputs:
  - { name: model_card,            format: model-card@1, recipient: cuo/caio + cuo/cto + cuo/chief-ethics-officer + model consumers }

skill_chain:
  - { step: 1, skill: model-card-author, inputs_from: { model_brief: model_brief, training_data_summary: training_data_summary, eval_results: eval_results, prior_card: prior_card }, outputs_to: card_draft }
  - { step: 2, skill: model-card-audit,  inputs_from: card_draft, outputs_to: model_card }

escalates_to:
  - { persona: cuo/chief-ethics-officer, when: "bias-audit flags disparate-impact pattern across protected attributes" }

consults:
  - { persona: cuo/chief-privacy-officer,    when: "training data includes personal data — PIA cross-reference needed" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with model_card hash + model-name + version + eval-summary
  - HITL pause at step 2 on QA-BIAS-001 (bias section missing or shallow) or QA-LIMITS-001 (limits section missing)
---

# Per model card release — `chief-ai-officer/per-model-card-release`

CAIO's per-model model-card workflow. Per Mitchell et al. Model Cards for Model Reporting (Google 2019) + Anthropic responsible-AI documentation + HuggingFace model-card standard. Triggered per major model release (semver-major) or significant retrain.

## When to invoke

- "Author the model card for [model name]"
- "Model-card release for [version]"
- "Document this model"

## How to invoke

```bash
cyberos-cuo run cuo/chief-ai-officer/per-model-card-release \
  --input model_brief=./models/2026-recommender/brief.md \
  --input training_data_summary=./models/2026-recommender/data-summary.md \
  --input eval_results=./models/2026-recommender/eval.csv \
  --input prior_card=./models/2025-recommender/card.md \
  --output-dir ./models/2026-recommender/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + 1 week for ethics + privacy review
- **Worst case:** bias-flag escalation may pause release until remediation

## Skill chain

- **Step 1 `model-card-author`** — drafts per Mitchell et al. + Anthropic + HuggingFace standards.
- **Step 2 `model-card-audit`** — validates per `model_card_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-BIAS-001 | Bias section shallow | Operator extends with bias-audit results |
| 2 | QA-LIMITS-001 | Limits section missing | Operator drafts |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.3 — CAIO role profile
- `./per-model-bias-audit.md` — peer (bias audit feeds model card)
- `../../../skill/model-card-{author,audit}/SKILL.md`
