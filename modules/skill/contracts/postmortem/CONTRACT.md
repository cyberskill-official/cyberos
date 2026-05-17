---
contract_id: postmortem
contract_version: v1
template_literal: postmortem@1
description: Canonical postmortem@1 — blameless post-mortem authored by postmortem-author; validated by postmortem-audit via postmortem_rubric@1.0. Implements SDP §2(j) Operations.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-cto
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Per-incident analysis; structural sections + timeline are reproducible from sources." }
emitted_source_freshness_tier: 9
---

# `postmortem@1` — canonical Post-mortem contract

> Frontmatter: `postmortem-audit/RUBRIC.md` §2. Body: §3 (`SEC-001..012`) — summary / timeline / customer impact / detection / response / contributing factors (Five-Whys) / went well / went wrong / got lucky / action items / lessons / SLO impact. Conditional: §4 — sev1/sev2 public-comms / data breach / security exploit / sev1 executive brief / AI-system behaviour analysis.

## Citations

- SDP §2(j) — Operations source.
- Google SRE Book — blameless culture.
- GDPR Art. 33 — 72-hour data-breach notification.
- Vietnam Decree 13/2023 PDPD.
- Consumers: `postmortem-author`, `postmortem-audit`, `retro-author` (incident reflection in COND-001).
