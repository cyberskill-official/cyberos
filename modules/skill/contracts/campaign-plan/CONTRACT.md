---
contract_id: campaign-plan
contract_version: v1
template_literal: campaign-plan@1
description: Canonical campaign-plan@1 — per-campaign plan authored by cmo; objective + audience + channel mix + creative concept + budget + measurement plan.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `campaign-plan@1` — canonical Campaign Plan contract

> Frontmatter: `campaign-plan-audit/RUBRIC.md` §2.
> Required body sections: §3 (campaign objective / target audience / channel mix + budget allocation / creative concept / launch + sustain calendar / measurement plan + KPIs / kill criteria).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves CMO's MQL→SQL-conversion + CAC-by-channel KPIs.

## Citations

- C-Suite Reference §5.4 (CMO)
- HubSpot / Marketo campaign-design patterns
- Demandbase / 6sense intent-data integration
- Consumers: `campaign-plan-author`, `campaign-plan-audit`.
