---
contract_id: safety-report
contract_version: v1
template_literal: safety-report@1
description: Canonical safety-report@1 — quarterly clinical-safety report authored by chief-medical-officer; adverse-event aggregation + signal detection + risk-benefit assessment.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; structural sections + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `safety-report@1` — canonical Safety Report contract

> Frontmatter: `safety-report-audit/RUBRIC.md` §2.
> Required body sections: §3 per `contracts/safety-report/template.md`.

## Citations

- C-Suite Reference §5.7
- ICH E2D Periodic Safety Update Reports
- FDA MedWatch / EMA EudraVigilance reporting
- Consumers: `safety-report-author`, `safety-report-audit`.
