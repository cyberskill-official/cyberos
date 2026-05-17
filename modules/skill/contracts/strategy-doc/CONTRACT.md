---
contract_id: strategy-doc
contract_version: v1
template_literal: strategy-doc@1
description: Canonical strategy-doc@1 — annual 3-5yr strategy document authored by cso-strategy or ceo; vision + diagnosis + guiding policy + coherent actions (per Rumelt good-strategy structure).
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `strategy-doc@1` — canonical Strategy Document contract

> Frontmatter: `strategy-doc-audit/RUBRIC.md` §2.
> Required body sections: §3 (vision / strategic diagnosis (the kernel) / guiding policy / coherent actions / OKR cascade tie-in / capability investments / financial framework / strategic risks / measurement plan).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves CEO + CSO-Strategy's strategy-execution-alignment KPI.

## Citations

- C-Suite Reference §5.1 (CSO-Strategy)
- Richard Rumelt 'Good Strategy / Bad Strategy' kernel structure
- Roger Martin 'Playing to Win' choice cascade
- Consumers: `strategy-doc-author`, `strategy-doc-audit`.
