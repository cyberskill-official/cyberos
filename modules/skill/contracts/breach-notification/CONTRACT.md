---
contract_id: breach-notification
contract_version: v1
template_literal: breach-notification@1
description: Canonical breach-notification@1 — data-breach notification authored by cpo-privacy + ciso + clo-legal under regulatory deadline (GDPR Art. 33 72h).
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `breach-notification@1` — canonical Breach Notification contract

> Frontmatter: `breach-notification-audit/RUBRIC.md` §2.
> Required body sections: §3 (incident summary / data classes exposed / affected data subjects (count + geography) / regulator notification plan / data-subject notification plan / containment actions / remediation plan).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves CPO-Privacy's breach-notification-timeliness + CISO's MTTR KPIs.

## Citations

- C-Suite Reference §5.6 (CPO-Privacy + CISO)
- GDPR Article 33 (72-hour notification)
- Vietnam Decree 13/2023 PDPD breach notification
- Consumers: `breach-notification-author`, `breach-notification-audit`.
