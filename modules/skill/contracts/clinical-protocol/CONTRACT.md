---
contract_id: clinical-protocol
contract_version: v1
template_literal: clinical-protocol@1
description: Canonical clinical-protocol@1 — per-trial clinical protocol authored by chief-medical-officer; trial design + endpoints + eligibility + safety monitoring + statistical analysis plan.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; structural sections + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `clinical-protocol@1` — canonical Clinical Protocol contract

> Frontmatter: `clinical-protocol-audit/RUBRIC.md` §2.
> Required body sections: §3 per `contracts/clinical-protocol/template.md`.

## Citations

- C-Suite Reference §5.7 (Chief Medical Officer)
- ICH-GCP E6(R3) Good Clinical Practice
- FDA IND / EMA CTR submission templates
- Consumers: `clinical-protocol-author`, `clinical-protocol-audit`.
