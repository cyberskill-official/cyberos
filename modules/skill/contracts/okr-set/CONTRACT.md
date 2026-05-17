---
contract_id: okr-set
contract_version: v1
template_literal: okr-set@1
description: Canonical okr-set@1 — company-level / function-level OKR cascade authored by ceo or chief-of-staff; quarterly cadence.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `okr-set@1` — canonical OKR Set contract

> Frontmatter: `okr-set-audit/RUBRIC.md` §2.
> Required body sections: §3 (objective / key-results / owner / cadence / measurement source / cascade ladder / dependencies).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves the persona's KPI envelope; alignment scored qualitatively.

## Citations

- C-Suite Reference §5.1 (CEO outputs)
- John Doerr 'Measure What Matters' OKR pattern
- C-Suite Reference §7 Chief-of-Staff (OKR adoption owner)
- Consumers: `okr-set-author`, `okr-set-audit`.
