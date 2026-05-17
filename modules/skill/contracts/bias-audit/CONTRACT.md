---
contract_id: bias-audit
contract_version: v1
template_literal: bias-audit@1
description: Canonical bias-audit@1 — quarterly bias audit authored by chief-ethics-officer + caio for each AI feature in production.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `bias-audit@1` — canonical Bias Audit contract

> Frontmatter: `bias-audit-audit/RUBRIC.md` §2.
> Required body sections: §3 (feature inventory / fairness metrics applied per feature (demographic parity / equalised odds / etc) / pass/fail per metric / remediation plan for failures / next-audit date).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves Chief-Ethics-Officer's bias-test-pass-rate + AI-incident-count (target 0) KPIs.

## Citations

- C-Suite Reference §5.6 (Chief Ethics Officer)
- C-Suite Reference §5.3 (CAIO joint owner)
- Fairlearn / Aequitas / IBM AI Fairness 360 toolkits
- Consumers: `bias-audit-author`, `bias-audit-audit`.
