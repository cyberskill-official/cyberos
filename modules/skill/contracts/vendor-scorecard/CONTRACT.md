---
contract_id: vendor-scorecard
contract_version: v1
template_literal: vendor-scorecard@1
description: Canonical vendor-scorecard@1 — monthly / quarterly vendor scorecard authored by coo / cpo-procurement; SLA + spend + sustainability.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `vendor-scorecard@1` — canonical Vendor Scorecard contract

> Frontmatter: `vendor-scorecard-audit/RUBRIC.md` §2.
> Required body sections: §3 (vendor list / spend by category / SLA attainment per vendor / sustainability score per vendor / consolidation opportunities).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves COO's vendor-consolidation + CPO-Procurement's cost-savings KPIs.

## Citations

- C-Suite Reference §5.1 (COO vendor mgmt)
- C-Suite Reference §5.7 (CPO-Procurement)
- EcoVadis supplier-scoring pattern
- Consumers: `vendor-scorecard-author`, `vendor-scorecard-audit`.
