---
contract_id: impl-plan
contract_version: v1
template_literal: impl_plan@1
description: "Canonical impl_plan@1 schema body — frontmatter contract + Markdown skeleton for the implementation-plan artefact emitted by `cuo/chief-technology-officer/spec-to-impl-plan`. Shadow record of the engineering tickets that get created in PROJ MCP (Linear / Jira / GitHub etc.) — the artefact lives in the repo for offline review + permanent record; the actual tickets live in the external system."
contract_kind: artefact_schema
locked_at: 2026-05-06

steward_persona: cuo-cto
escalation_on_breach:
  legal:    null
  security: cuo-cseco
  compliance: cuo-clo

determinism:
  reproducible: false
  fixity_notes: "Ticket breakdown is judgement-driven. Body shape deterministic; ticket content not."

emitted_source_freshness_tier: 30
---

# `impl_plan@1` — canonical implementation-plan contract

> Loaded by `cuo/chief-technology-officer/spec-to-impl-plan` as the generation skeleton. The artefact is the SHADOW RECORD of tickets actually created in PROJ MCP. Future tools (sprint planning, capacity tracking) consume this artefact for offline analysis without round-tripping to Linear/Jira.

## Why a separate contract for impl-plan

The actual tickets live in PROJ MCP. The impl-plan markdown is the project's permanent record + offline review surface. Reasons to keep both:

- Tickets get archived/deleted; the markdown stays in the repo.
- Sprint planning happens against the markdown (engineering lead reviews + reorders) before tickets are created.
- The chain's memory-write goes to `memories/projects/<slug>.md` and references this markdown — `genie.action_log` doesn't store ticket text, only IDs + hashes.

## Frontmatter contract

| Field | Type / enum | Required |
| --- | --- | --- |
| `template` | const `impl_plan@1` | yes |
| `title` | string, 3-100 chars | yes |
| `author` | `^@[A-Za-z0-9_.-]{1,38}$` | yes |
| `created_at` | ISO 8601 with timezone | yes |
| `last_updated_at` | ISO 8601 with timezone | yes |
| `tech_spec_ref` | path to source `tech_spec@1` (or `product-requirements-document@1` for `lean` profile that skips tech-spec) | yes |
| `target_release` | mirrors source | yes |
| `proj_backend` | `linear` / `jira` / `github` / `none` (none = no PROJ MCP wired; markdown-only) | yes |
| `tickets_created` | boolean — true if tickets were actually emitted to PROJ MCP | yes |
| `total_tickets` | integer ≥ 0 | yes |
| `total_estimated_engineer_days` | number ≥ 0 | yes |
| `chain_profile` | inherited from upstream | yes |

## Required body sections

1. **`## Background`** — link to source tech-spec (or PRD for lean profile).
2. **`## Tickets`** — table: # / Title / Sizing (XS/S/M/L/XL) / Dependencies / PROJ ticket ID (when created) / Acceptance criteria reference.
3. **`## Sprint Suggestion`** — proposed grouping into sprints based on dependencies + sizing. Engineering lead reviews + reorders before approving.
4. **`## Risks`** — known risks at planning time (sizing uncertainty, unclear dependencies, untested integrations). Each row carries a mitigation.
5. **`## Open Questions`** — unresolved items that block ticket creation. If empty, explicit "No open questions" statement.

## Conditionally-required sections

| Trigger | Required section |
| --- | --- |
| `tickets_created: true` | `## Ticket Index` — auto-generated mapping from impl-plan ticket # → PROJ ticket ID + URL |
| `chain_profile: lean` (no upstream tech-spec) | `## Architecture Note` — brief para explaining the architectural assumptions baked into the ticket breakdown (since lean skips software-requirements-specification-author + task-to-tech-spec) |

## Citations

- Registry v0.2.9 — first version of this contract.
- Pattern source — `tech_spec@1` (similar shape; this contract is the "next layer" toward execution).
