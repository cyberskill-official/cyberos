---
workflow_id: chief-trust-officer/annual-trust-strategy
workflow_version: 1.0.0
purpose: Author the annual trust strategy — trust posture vision, certification roadmap, transparency program, customer-trust metrics.
persona: cuo/chief-trust-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_strategy,        source: last year's strategy-document@1 (trust chapter), format: strategy-document@1 }
  - { name: portal_history,        source: 4 quarters of trust-portal-update@1, format: trust-portal-update@1 (4Q) }
  - { name: transparency_history,  source: prior transparency-report@1, format: transparency-report@1 }
  - { name: ceo_priorities,        source: cuo/ceo (vision brief), format: markdown }

outputs:
  - { name: trust_strategy,        format: strategy-doc@1, recipient: cuo/chief-trust-officer + cuo/ceo + cuo/ciso + cuo/cpo-privacy + Board (annual trust chapter) }

skill_chain:
  - { step: 1, skill: strategy-document-author, inputs_from: { prior_strategy: prior_strategy, portal_history: portal_history, transparency_history: transparency_history, ceo_priorities: ceo_priorities }, outputs_to: strategy_draft }
  - { step: 2, skill: strategy-document-audit,  inputs_from: strategy_draft, outputs_to: trust_strategy }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "strategy proposes new certification pursuit (FedRAMP High, IRAP, etc.) requiring 1-2 year + significant investment" }

consults:
  - { persona: cuo/chief-information-security-officer,           when: "certification roadmap" }
  - { persona: cuo/chief-privacy-officer,    when: "privacy posture intersect" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with trust_strategy hash + cert roadmap + transparency commitments
  - HITL pause at step 2 on QA-KERNEL-001 (Rumelt incomplete) or QA-METRIC-001 (trust metric not measurable)
---

# Annual trust strategy — `chief-trust-officer/annual-trust-strategy`

Chief Trust Officer's annual trust-program strategy per Rumelt + Edelman Trust Barometer + Santa Clara Principles. Drives certification pursuit + transparency program + customer-trust metrics for the year.

## When to invoke

- "Build the 2026 trust strategy"
- "Annual trust program refresh"
- "Refresh certification roadmap"

## How to invoke

```bash
cyberos-cuo run cuo/chief-trust-officer/annual-trust-strategy \
  --input prior_strategy=./trust/2025/strategy.md \
  --input portal_history=./trust/2025/portal/ \
  --input transparency_history=./trust/2025/transparency.md \
  --input ceo_priorities=./engagements/2026/vision.md \
  --output-dir ./trust/2026/strategy/
```

## Expected duration

- **Happy path:** 8-16 hours runtime + 6-12 weeks for cross-function + Board review
- **Worst case:** new certification pursuit spans 1-2 years

## Skill chain

- **Step 1 `strategy-document-author`** — drafts per Rumelt + Edelman + Santa Clara.
- **Step 2 `strategy-document-audit`** — validates per `strategy_doc_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-KERNEL-001 | Rumelt incomplete | Operator extends |
| 2 | QA-METRIC-001 | Trust metric not measurable | Operator quantifies |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.6 — Chief Trust Officer role profile
- `./quarterly-trust-portal-update.md` — operational peer
- `../../../skill/strategy-doc-{author,audit}/SKILL.md`
