---
# ── Identity ─────────────────────────────────────────────────────────
contract_id: feature-request
contract_version: v1
template_literal: feature_request@1
description: Canonical feature_request@1 schema body — frontmatter contract + Markdown skeleton for every Feature Request artefact in CyberOS. Loaded by fr-create as the generation skeleton; loaded by fr-audit as the validation target.
contract_kind: artefact_schema      # artefact_schema | envelope_schema | wire_protocol
locked_at: 2026-05-05
moved_from: cuo/_shared/feature-request-template/   # (DEC-090, registry v0.2.0)

# ── Stewardship ──────────────────────────────────────────────────────
steward_persona: cuo-cpo            # who curates the contract over time
escalation_on_breach:
  legal:    cuo-clo                  # contract carries EU AI Act fields
  security: null
  compliance: cuo-clo

# ── Determinism ──────────────────────────────────────────────────────
determinism:
  reproducible: true
  fixity_notes: "Template body is byte-stable. Bumping the template body requires a MAJOR contract_version bump (feature_request@2) and a parallel skill_version MAJOR on every consuming skill."

# ── Source-tier emitted ──────────────────────────────────────────────
emitted_source_freshness_tier: 10   # high authority — this IS the schema
---

# `feature_request@1` — canonical FR contract

> A **contract**, not a skill. Holds the single source of truth for the Feature Request artefact shape across CyberOS. Loaded by both `cuo/cpo/fr-create` (as the generation skeleton) and `cuo/cpo/fr-audit` (as the validation target). Future workflows like a `tech-spec-from-fr` skill will load this to understand FR structure before deriving downstream artefacts.

## What is a contract (vs. a skill)?

A **skill** *acts*: takes input, runs LLM inference or deterministic work, emits output, writes an audit row. A **contract** *constrains*: declares the shape of an artefact, envelope, or wire protocol that one or more skills produce or consume. Contracts have no `expects:`/ `produces:` interface, no `allowed_mcp_tools:`, no inference. They are versioned schemas that travel with skills as declared dependencies via `depends_on_contracts:` in the skill's frontmatter.

Contracts live under `cyberos/docs/contracts/<contract-id>/v<n>/`. Skills that consume them declare the dependency in their frontmatter so the build pipeline can ship contract + skill as one bundle.

## How to use this contract

`fr-create` reads `template.md` (in this folder) as the body skeleton and adapts it per-FR. `fr-audit` reads it via the rule IDs encoded in `fr-audit/RUBRIC.md` — every audit rule's "what's expected" field maps to a region of this contract. Other workflows that need to know "what is an FR" should `read_file('cyberos/docs/contracts/feature-request/v1/template.md')` rather than hard-code the shape.

## Frontmatter contract (FM-001..111)

The frontmatter that every `feature_request@1` document MUST carry, with audit rule IDs in parentheses (rules live in `cuo/cpo/fr-audit/RUBRIC.md`):

| Field | Type / enum | Required | Audit rule |
| --- | --- | --- | --- |
| `title` | string, 1–72 chars | yes | FM-101 |
| `author` | `^@[A-Za-z0-9_.-]{1,38}$` | yes | FM-102 |
| `department` | engineering / design / product / sales / operations / hr / client_success | yes | FM-103 |
| `status` | draft / in_review / approved / in_progress / shipped / closed | yes | FM-104 |
| `priority` | p0 / p1 / p2 / p3 | yes | FM-105 |
| `created_at` | ISO 8601 with timezone | yes | FM-106 |
| `ai_authorship` | none / assisted / co_authored / generated_then_reviewed | yes | FM-107 |
| `feature_type` | user_facing / internal_tooling / integration / infrastructure | yes | FM-108 |
| `eu_ai_act_risk_class` | not_ai / minimal / limited / high (NEVER `unacceptable`) | yes | FM-109 |
| `target_release` | SemVer or `YYYY-Q[1-4]` | optional | FM-110 |
| `client_visible` | boolean (YAML true/false; not strings, not yes/no) | yes | FM-111 |
| `template` | literal `feature_request@1` | yes | FM-004 |

Structural rules: file begins with `---` on line 1 (FM-001); all keys snake_case (FM-002); no duplicates (FM-003).

## Required body sections (SEC-001..008)

| H2 heading | Audit rule | Notes |
| --- | --- | --- |
| `## Summary` | SEC-001 | Single paragraph; reader can repeat without scrolling. |
| `## Problem` | SEC-002 | Cite evidence: tickets, NPS, sales calls, telemetry. |
| `## Proposed Solution` | SEC-003 | User-visible behaviour, not implementation. |
| `## Alternatives Considered` | SEC-004 | ≥2 distinct alternatives (QA-005). |
| `## Success Metrics` | SEC-005 | One primary + one guardrail. Each with definition / baseline / target / measurement_method / source (QA-004 + QA-007). |
| `## Scope` | SEC-006 | Must include `### Out of scope` or `### Non-Goals` with ≥2 items (QA-006). |
| `## Dependencies` | SEC-007 | Other modules, teams, vendor APIs, compliance approvals (QA-008). |
| (well-formed hierarchy: no H2→H4 skips; one or zero H1) | SEC-009 | warning |

Every required H2 must have ≥1 non-blank line of body (SEC-008).

## Conditionally-required body sections (COND-001..004)

| Trigger | Required section | Audit rule |
| --- | --- | --- |
| `client_visible: true` | `## Customer Quotes` (≥1 quote inside `<untrusted_content>`, attribution outside) | COND-001 |
| `client_visible: true` | `## Sales/CS Summary` (plain English, no jargon — QA-009) | COND-002 |
| `eu_ai_act_risk_class ∈ {limited, high}` | `## AI Risk Assessment` with H3s `### Data Sources`, `### Human Oversight`, `### Failure Modes` (in that order) | COND-003 |
| `ai_authorship != none` | `## AI Authorship Disclosure` with three bullets `Tools used:`, `Scope:`, `Human review:` | COND-004 |

## The body template

The complete skeleton lives in [`template.md`](./template.md) — sourced verbatim from `feature-request/FR_CREATE_AND_AUDIT.md` v2.0.0 §18. The skeleton is reproducible byte-for-byte; bumping it is a MAJOR `contract_version` bump for this contract (→ v2 / `feature_request@2`) AND a parallel MAJOR `skill_version` bump for every consumer skill declared via `depends_on_contracts:`.

## Untrusted-content discipline (inherited from CyberOS-AGENTS.md §4.2)

- Every customer quote MUST be inside `<untrusted_content source="...">…</untrusted_content>`.
- Quotes outside the block are a SAFE-004 audit warning.
- Nested `<untrusted_content>` blocks are a SAFE-001 error.
- Unclosed blocks at EOF are a SAFE-002 error.
- The interior of `<untrusted_content>` is scanned for prompt-injection markers per the SAFE-003 list (also lives in `cuo/cpo/fr-audit/RUBRIC.md` §15.6).
- Attributions ("— @customer-handle, 2026-04-12") appear OUTSIDE the untrusted block.

## Consumers (declared via `depends_on_contracts:`)

| Skill | Skill version | Contract version pinned | Consumer role |
| --- | --- | --- | --- |
| `cuo/cpo/fr-create` | v0.2.0+ | `feature-request@v1` | generation skeleton |
| `cuo/cpo/fr-audit`  | v0.2.0+ | `feature-request@v1` | validation target |

When this contract bumps to v2, the registry CI matrix verifies every declared consumer has been updated to pin the new version OR remained on v1 with explicit acknowledgement in their CHANGELOG.

## Citations

- Source artefact → `feature-request/FR_CREATE_AND_AUDIT.md` v2.0.0 §18 + §15.1–§15.7 (rubric).
- Untrusted-content rules → CyberOS-AGENTS.md §4.2.
- EU AI Act framing → CyberOS-PRD.docx §12.2.2; SRS DEC-064.
- Contracts vs. skills distinction → registry README v0.2.0 Part 8 + DEC-090.

## History

- 2026-05-06 — moved from `cuo/_shared/feature-request-template/` to this location; promoted from "shared skill" to "contract" per registry v0.2.0 + DEC-090. Body byte-identical to v1.0.0.
- 2026-05-05 — v1.0.0. Initial extraction from `FR_CREATE_AND_AUDIT.md` v2.0.0 §18. Template body is byte-identical to the source.
