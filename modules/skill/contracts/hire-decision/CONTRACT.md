---
contract_id: hire-decision
contract_version: v1
template_literal: hire-decision@1
description: Canonical hire-decision@1 — C-suite hire decision record authored by ceo or chro for board reporting.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `hire-decision@1` — canonical Hire Decision Record contract

> Frontmatter: `hire-decision-audit/RUBRIC.md` §2.
> Required body sections: §3 (role spec / candidate slate summary / interview-loop verdicts / offer terms / risks / decision rationale).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves CHRO's time-to-fill + CEO's exec-churn metric.

## Citations

- C-Suite Reference §5.1 (CEO team output)
- C-Suite Reference §5.5 (CHRO hire mgmt)
- Consumers: `hire-decision-author`, `hire-decision-audit`.
