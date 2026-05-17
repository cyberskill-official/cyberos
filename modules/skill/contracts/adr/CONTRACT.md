---
contract_id: adr
contract_version: v1
template_literal: adr@1
description: Canonical adr@1 — Architecture Decision Record in Michael Nygard format. Authored by adr-author; validated by adr-audit via adr_rubric@1.0. Implements Software Development Process.md §2(d).
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-cpo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }

determinism: { reproducible: false, fixity_notes: "ADR content is judgement; supersedes / status transitions are reproducible." }

emitted_source_freshness_tier: 14
---

# `adr@1` — canonical Architecture Decision Record contract

> Frontmatter contract: `adr-audit/RUBRIC.md` §2.
> Required body sections: §3 (`SEC-001..006`) — Nygard format (Context / Options Considered / Decision / Consequences / Compliance & Quality Impact / Notes & References).
> Conditional sections: §4 (`COND-001..004`) — security boundary, data residency, reversal cost, status: superseded.

## Citations

- Michael Nygard, "Documenting Architecture Decisions".
- arc42 documentation.
- ISO/IEC 25010:2023 — quality-characteristic mapping for §5 of body.
- Consumers: `adr-author`, `adr-audit`, downstream `sdd-author` / `threat-model-author` (link via `linked_adrs`).
