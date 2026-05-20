---
contract_id: sow
contract_version: v1
template_literal: statement-of-work@1
description: Canonical statement-of-work@1 schema — frontmatter contract + Markdown skeleton for the Statement of Work / Project Charter artefact emitted by statement-of-work-author. Consumed by statement-of-work-audit for validation, and by closure-author / product-requirements-document-author / project-plan-author as upstream context.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-cpo
escalation_on_breach:
  legal:      cuo-clo
  security:   cuo-cseco
  compliance: cuo-clo

determinism:
  reproducible: false
  fixity_notes: "SOW authoring includes commercial judgement (pricing, scope, milestones). CONTRACT body shape is byte-stable; SOW CONTENT is not."

emitted_source_freshness_tier: 12
---

# `statement-of-work@1` — canonical Statement of Work contract

> A **contract**, not a skill. Holds the single source of truth for the Statement of Work / Project Charter artefact across CyberOS. Loaded by `statement-of-work-author` (generation skeleton) and `statement-of-work-audit` (validation target via `sow_rubric@1.0`). Implements modules/cuo/README.md#software-development-process §4.9 (Project Charter / SOW skeleton).

## Why a separate contract for SOWs

A SOW is the commercial-legal anchor of an engagement. It binds CyberSkill to specific deliverables, pricing terms, IP assignment, AI-tool usage disclosure, and governance cadence. PRD / SRS / project-plan all derive from a SOW; if the SOW is wrong, everything downstream is wrong. Splitting the SOW contract from those downstream artefacts ensures the commercial-legal anchor is independently versionable.

## Frontmatter contract

The authoritative per-field rule set is `statement-of-work-audit/RUBRIC.md` §2. Summary of required fields:

| Field | Type / enum | Required |
|---|---|---|
| `template` | const `statement-of-work@1` | yes |
| `title` | string 1–120 chars | yes |
| `client_name` | string | yes |
| `client_legal_entity` | full legal name + jurisdiction | yes |
| `engagement_model` | one of: `fixed_price`, `time_and_materials`, `dedicated_team`, `staff_augmentation`, `managed_services` | yes |
| `effective_date`, `target_close_date` | ISO 8601 dates | yes |
| `sow_version` | SemVer | yes |
| `cs_signer`, `em_signer`, `cyberskill_signer` | handles per role | yes |
| `governing_law` | free string | yes |
| `provenance.source_path`, `provenance.source_hash` | required | yes |

## Required body sections

Twelve fixed sections in order (per SDP §4.9 + `statement-of-work-audit/RUBRIC.md` §3 SEC-001..012). See `template.md` for the skeleton.

## Conditionally-required sections

Driven by `engagement_model` and data class — see `statement-of-work-audit/RUBRIC.md` §4 (`COND-001..010`). Examples: fixed_price triggers `### Fixed-Price Terms`; EU residency triggers `### GDPR Addendum`; Vietnam residency triggers `### Vietnam Compliance` (Decree 13/2023 PDPD + Decree 53/2022 cybersecurity).

## Authority markers

Every claim in §1 Objectives / §3 Deliverables / §9 Acceptance Criteria carries an `<!-- authority: ... -->` marker per AGENTS.md §5.1.

## Citations

- `../../../../modules/cuo/README.md` §4.9 — SOW skeleton source.
- `../../../../modules/cuo/README.md` §6 — Engagement models + IP + AI-tool usage policy.
- Consumers: `statement-of-work-author` (generation), `statement-of-work-audit` (validation), `product-requirements-document-author` / `project-plan-author` / `closure-author` (upstream context).
