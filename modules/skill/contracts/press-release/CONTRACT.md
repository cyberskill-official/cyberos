---
contract_id: press-release
contract_version: v1
template_literal: press-release@1
description: Canonical press-release@1 — per-announcement press release authored by cco-communications; headline + lede + boilerplate + spokesperson quote + media contact.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `press-release@1` — canonical Press Release contract

> Frontmatter: `press-release-audit/RUBRIC.md` §2.
> Required body sections: §3 (headline / dateline / lede paragraph / supporting paragraphs / spokesperson quote(s) / boilerplate / media contact / embargo terms (if applicable)).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves CCO-Communications's share-of-voice KPI.

## Citations

- C-Suite Reference §5.4
- AP Stylebook conventions
- PRSA + IPRA press-release standards
- Consumers: `press-release-author`, `press-release-audit`.
