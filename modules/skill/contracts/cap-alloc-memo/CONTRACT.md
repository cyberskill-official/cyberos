---
contract_id: cap-alloc-memo
contract_version: v1
template_literal: cap-alloc-memo@1
description: Canonical cap-alloc-memo@1 — capital-allocation decision memo authored by ceo or cfo for board approval — buy vs build vs partner vs M&A.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `cap-alloc-memo@1` — canonical Capital-Allocation Memo contract

> Frontmatter: `cap-alloc-memo-audit/RUBRIC.md` §2.
> Required body sections: §3 (decision being made / options analysed / financial model / strategic fit / risks / recommendation / board approval ask).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves CEO's capital-allocation discipline; CFO's IRR + capital-plan accuracy.

## Citations

- C-Suite Reference §5.1 (CEO capital allocation)
- C-Suite Reference §5.2 (CFO capital plan)
- Buffett/Munger capital-allocation lens
- Consumers: `cap-alloc-memo-author`, `cap-alloc-memo-audit`.
