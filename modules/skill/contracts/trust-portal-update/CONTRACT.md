---
contract_id: trust-portal-update
contract_version: v1
template_literal: trust-portal-update@1
description: Canonical trust-portal-update@1 — monthly trust-portal content update authored by chief-trust-officer + cpo-privacy; certifications + audit results + sub-processors + incident-status timeline.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `trust-portal-update@1` — canonical Trust Portal Update contract

> Frontmatter: `trust-portal-update-audit/RUBRIC.md` §2.
> Required body sections: §3 (certifications + dates / audit results summary / sub-processor list (current) / incident-status timeline / FAQ updates / customer-trust briefing log).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves Chief-Trust-Officer's trust-portal-NPS + security-DD-deal-acceleration KPIs.

## Citations

- C-Suite Reference §5.6 (Chief Trust Officer)
- Vanta / Drata / SafeBase trust-portal patterns
- Consumers: `trust-portal-update-author`, `trust-portal-update-audit`.
