---
contract_id: pia
contract_version: v1
template_literal: pia@1
description: Canonical pia@1 — per-feature Privacy Impact Assessment authored by cpo-privacy / engineering-DPO; mandatory per GDPR + Vietnam Decree 13/2023.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `pia@1` — canonical Privacy Impact Assessment contract

> Frontmatter: `pia-audit/RUBRIC.md` §2.
> Required body sections: §3 (feature description / data flow / data classes touched / legal basis / DPIA risk assessment / mitigations / DPO sign-off).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves CPO-Privacy's PIA-coverage (100% target) KPI.

## Citations

- C-Suite Reference §5.6 (CPO-Privacy)
- GDPR Article 35 (DPIA)
- Vietnam Decree 13/2023 PDPD
- Consumers: `pia-author`, `pia-audit`.
