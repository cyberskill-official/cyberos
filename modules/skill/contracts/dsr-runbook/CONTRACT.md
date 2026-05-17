---
contract_id: dsr-runbook
contract_version: v1
template_literal: dsr-runbook@1
description: Canonical dsr-runbook@1 — operational runbook for handling data-subject requests (access / deletion / portability / objection) authored by cpo-privacy.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `dsr-runbook@1` — canonical Data Subject Request Runbook contract

> Frontmatter: `dsr-runbook-audit/RUBRIC.md` §2.
> Required body sections: §3 (request-intake workflow / identity verification / data-search procedure / response-template library / regulatory-deadline tracking / DPO escalation path).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves CPO-Privacy's DSR-response-time KPI (within statutory window).

## Citations

- C-Suite Reference §5.6 (CPO-Privacy)
- GDPR Articles 15-22 (DSR rights)
- Vietnam Decree 13/2023 DSR rights
- Consumers: `dsr-runbook-author`, `dsr-runbook-audit`.
