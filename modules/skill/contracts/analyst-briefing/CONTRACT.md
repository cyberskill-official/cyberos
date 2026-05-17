---
contract_id: analyst-briefing
contract_version: v1
template_literal: analyst-briefing@1
description: Canonical analyst-briefing@1 — per-analyst-firm briefing deck authored by cco-communications; positioning + traction + roadmap + competitive context for analyst awareness.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `analyst-briefing@1` — canonical Analyst Briefing contract

> Frontmatter: `analyst-briefing-audit/RUBRIC.md` §2.
> Required body sections: §3 (company overview / positioning + ICP / customer traction + named logos / product roadmap / competitive landscape / customer references available / analyst Q&A prep).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves CCO-Communications's analyst-perception (sell-side feedback) KPI.

## Citations

- C-Suite Reference §5.4
- Gartner / Forrester analyst-relations patterns
- Consumers: `analyst-briefing-author`, `analyst-briefing-audit`.
