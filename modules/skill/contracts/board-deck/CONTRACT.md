---
contract_id: board-deck
contract_version: v1
template_literal: board-deck@1
description: Canonical board-deck@1 — quarterly board deck authored by ceo or cfo; standard 10-15 slide structure.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `board-deck@1` — canonical Board Deck contract

> Frontmatter: `board-deck-audit/RUBRIC.md` §2.
> Required body sections: §3 (executive summary / KPIs vs plan / strategic narrative / financial detail / risks + mitigations / asks of the board).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves CEO's board-confidence + CFO's forecast-accuracy KPIs.

## Citations

- C-Suite Reference §5.1 (CEO communication output)
- C-Suite Reference §5.2 (CFO board financials)
- Consumers: `board-deck-author`, `board-deck-audit`.
