---
contract_id: retro
contract_version: v1
template_literal: retro@1
description: Canonical retro@1 — Start/Stop/Continue retrospective + DORA review per SDP Template §4.8. Authored by retro-author; validated by retro-audit via retro_rubric@1.0.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-cpo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Retro content is judgement; DORA values are reproducible from metric backends." }
emitted_source_freshness_tier: 16
---

# `retro@1` — canonical Retrospective contract

> Frontmatter: `retro-audit/RUBRIC.md` §2. Body: §3 (`SEC-001..007`) — team mood / DORA trends / continue / stop / start / action items / wins. Conditional: §4 — incident reflection / QBR / AI-tooling impact.

## Citations

- SDP Template §4.8 — Start/Stop/Continue + DORA review.
- SDP §5.6 — AI-tooling DORA-impact tracking.
- Consumers: `retro-author`, `retro-audit`, `closure-author` (lessons-learned compilation).
