---
contract_id: ethics-review
contract_version: v1
template_literal: ethics-review@1
description: Canonical ethics-review@1 — per-feature ethics review authored by chief-ethics-officer; intended-use vs anti-use + harm vectors + mitigation + decision.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `ethics-review@1` — canonical Ethics Review contract

> Frontmatter: `ethics-review-audit/RUBRIC.md` §2.
> Required body sections: §3 (feature summary / intended uses / anti-uses (what this MUST NOT do) / harm vectors per stakeholder / mitigations / residual-harm acceptance / decision + sign-off).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves Chief-Ethics-Officer's ethics-review-coverage + AI-incident-count KPIs.

## Citations

- C-Suite Reference §5.6
- IEEE Ethically Aligned Design
- Markkula Center applied-ethics decision framework
- Consumers: `ethics-review-author`, `ethics-review-audit`.
