---
contract_id: cs-engagement
contract_version: v1
template_literal: cs-engagement@1
description: Canonical cs-engagement@1 — per-customer engagement plan authored by cco-customer; QBR cadence + expansion plan + health-score.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `cs-engagement@1` — canonical Customer Engagement Plan contract

> Frontmatter: `cs-engagement-audit/RUBRIC.md` §2.
> Required body sections: §3 (customer context / current health-score / QBR cadence / expansion theses / churn-risk indicators / executive coverage).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves CCO-Customer's NRR + churn KPIs.

## Citations

- C-Suite Reference §5.4 (CCO-Customer)
- Gainsight / Catalyst CS-engagement patterns
- Consumers: `cs-engagement-author`, `cs-engagement-audit`.
