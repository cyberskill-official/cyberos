---
contract_id: model-card
contract_version: v1
template_literal: model-card@1
description: Canonical model-card@1 — per-AI-model model card authored by caio + chief-ethics-officer; Hugging Face model-card pattern + Google Model Cards paper.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `model-card@1` — canonical Model Card contract

> Frontmatter: `model-card-audit/RUBRIC.md` §2.
> Required body sections: §3 (model identity / intended use / out-of-scope uses / training data / evaluation results / bias analysis / safety considerations / limitations / contact).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves CAIO's AI-use-cases-in-production + Chief-Ethics-Officer's bias-test-pass-rate KPIs.

## Citations

- C-Suite Reference §5.3 (CAIO)
- C-Suite Reference §5.6 (Chief Ethics Officer)
- Mitchell et al. 'Model Cards for Model Reporting' (2019)
- Hugging Face Model Card spec
- Consumers: `model-card-author`, `model-card-audit`.
