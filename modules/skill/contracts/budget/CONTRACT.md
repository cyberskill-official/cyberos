---
contract_id: budget
contract_version: v1
template_literal: budget@1
description: Canonical budget@1 — annual budget authored by cfo and approved by ceo + board.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `budget@1` — canonical Annual Budget contract

> Frontmatter: `budget-audit/RUBRIC.md` §2.
> Required body sections: §3 (executive summary / revenue plan / opex plan / capex plan / hiring plan / per-function envelope / scenario sensitivities).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves CFO's budget-vs-actuals + working-capital KPIs.

## Citations

- C-Suite Reference §5.2 (CFO budget)
- PMBOK + traditional FP&A patterns
- Consumers: `budget-author`, `budget-audit`.
