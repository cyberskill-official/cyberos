---
contract_id: decision-log
contract_version: v1
template_literal: decision-log@1
description: Canonical decision-log@1 — rolling decision log maintained by chief-of-staff; one row per material decision.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `decision-log@1` — canonical Decision Log contract

> Frontmatter: `decision-log-audit/RUBRIC.md` §2.
> Required body sections: §3 (decision-id / date / context / options-considered / decision / owner / reversibility / expiry / outcome-review-date).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves Chief-of-Staff's on-time-decision-closure KPI.

## Citations

- C-Suite Reference §5.7 (Chief of Staff)
- Bezos / Amazon two-way-door framework
- Consumers: `decision-log-author`, `decision-log-audit`.
