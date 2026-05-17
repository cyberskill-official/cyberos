---
contract_id: transparency-report
contract_version: v1
template_literal: transparency-report@1
description: Canonical transparency-report@1 — quarterly transparency report authored by chief-trust-officer; data requests received + AI-system disclosures + content-moderation actions + government info-requests.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `transparency-report@1` — canonical Transparency Report contract

> Frontmatter: `transparency-report-audit/RUBRIC.md` §2.
> Required body sections: §3 (data requests received + outcomes / AI-system disclosures / content-moderation actions (if applicable) / government info-requests / methodology + caveats / contact).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves Chief-Trust-Officer's transparency-cadence + trust KPIs.

## Citations

- C-Suite Reference §5.6
- Google / Microsoft / Meta transparency-report exemplars
- Consumers: `transparency-report-author`, `transparency-report-audit`.
