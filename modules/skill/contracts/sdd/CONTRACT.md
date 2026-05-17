---
contract_id: sdd
contract_version: v1
template_literal: sdd@1
description: Canonical sdd@1 — Software Design Description per IEEE 1016-2009. Authored by sdd-author; validated by sdd-audit via sdd_rubric@1.0. Implements Software Development Process.md §2(e).
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-cto
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }

determinism: { reproducible: false, fixity_notes: "Design content reflects engineering judgement; viewpoint structure is byte-stable." }

emitted_source_freshness_tier: 13
---

# `sdd@1` — canonical Software Design Description contract

> Frontmatter: `sdd-audit/RUBRIC.md` §2.
> Required body sections: §3 (`SEC-001..011`) — IEEE 1016 viewpoints (Context / Composition / Logical / Information / Interface / Patterns / Interaction / State Dynamics / Algorithm / Resource).
> Conditional sections: §4 (`COND-001..005`) — API, persistence, UI, performance, backwards-compatibility.

## Citations

- IEEE 1016-2009 — SDD viewpoints source.
- arc42 §5-§10 — supplementary structure.
- Consumers: `sdd-author`, `sdd-audit`, downstream `impl-plan-author` (links to SDD via `linked_sdd`).
