---
contract_id: investor-update
contract_version: v1
template_literal: investor-update@1
description: Canonical investor-update@1 — monthly investor update authored by ceo or cfo for VCs/LPs; metrics narrative + asks.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `investor-update@1` — canonical Investor Update contract

> Frontmatter: `investor-update-audit/RUBRIC.md` §2.
> Required body sections: §3 (tldr / metrics dashboard / wins / losses / what's working / what's not / cash position / asks).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves CEO's board-confidence + investor-NPS over time.

## Citations

- C-Suite Reference §5.1 (CEO IR comms)
- C-Suite Reference §5.2 (CFO IR narrative)
- Mark Suster + Brad Feld investor-update pattern
- Consumers: `investor-update-author`, `investor-update-audit`.
