---
contract_id: emissions-inventory
contract_version: v1
template_literal: emissions-inventory@1
description: Canonical emissions-inventory@1 — quarterly Scope 1 / 2 / 3 emissions inventory authored by cso-sustainability; GHG Protocol-aligned calculation + verification.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `emissions-inventory@1` — canonical Emissions Inventory contract

> Frontmatter: `emissions-inventory-audit/RUBRIC.md` §2.
> Required body sections: §3 (Scope 1 direct emissions / Scope 2 energy emissions / Scope 3 value-chain emissions / calculation methodology / data sources / verification status / year-over-year delta).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves CSO-Sustainability's emissions-reduction KPI; feeds the annual sustainability report.

## Citations

- C-Suite Reference §5.7
- GHG Protocol Corporate Standard
- SBTi target-setting methodology
- Consumers: `emissions-inventory-author`, `emissions-inventory-audit`.
