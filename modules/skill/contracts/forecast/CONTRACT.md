---
contract_id: forecast
contract_version: v1
template_literal: forecast@1
description: Canonical forecast@1 — monthly / quarterly financial forecast authored by cfo; revenue + cost + cash trajectory.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `forecast@1` — canonical Financial Forecast contract

> Frontmatter: `forecast-audit/RUBRIC.md` §2.
> Required body sections: §3 (executive summary / revenue forecast / cost forecast / cash trajectory / forecast accuracy reflection / assumption log).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: directly moves CFO's forecast-accuracy KPI (target ±5%).

## Citations

- C-Suite Reference §5.2 (CFO output)
- Adaptive / Anaplan / Pigment forecast patterns
- Consumers: `forecast-author`, `forecast-audit`.
