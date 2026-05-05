---
name: feature-request-template
description: Canonical feature_request@1 schema body — frontmatter contract + Markdown skeleton for every Feature Request artefact in CyberOS. Loaded by fr-create as the generation skeleton; loaded by fr-audit as the validation target.
skill_version: 1.0.0  # template_version = feature_request@1; locked
persona: cuo
owner_role: _shared
allowed_brain_scopes:
  read: []
  write: []
allowed_mcp_tools: []  # read-only template content; no tool surface
escalation:
  to_persona_on_legal: cuo-clo  # template carries EU AI Act fields; CLO is the authority
  to_persona_on_security: null
  to_persona_on_compliance: cuo-clo
  to_human_on_irreversible: false  # template never acts
expects: null
produces:
  schema_ref: ./template.md
  output_kind: artefact
audit:
  emit_to: genie.action_log
  row_kind: template_loaded
  payload_hash_field: template_sha256
  explanation_pane: required
confidence_band:
  default: 1.0  # template content is fixed; no inference
  defer_below: null
  cite_sources: required
untrusted_inputs:
  wrap_in: <untrusted_content/>
  injection_scan: required
  on_marker_hit: surface_to_human
determinism:
  reproducible: true
  fixity_notes: "Template body is byte-stable. Bumping the template body requires a MAJOR skill_version bump and a parallel bump of feature_request@N."
emitted_source_freshness_tier: 10  # high authority — this IS the schema
gated_until_phase: null
---

# `feature_request@1` — canonical FR template

> Single source of truth for the Feature Request artefact shape across
> CyberOS. Loaded by both `cuo/cpo/fr-create` (as the generation skeleton)
> and `cuo/cpo/fr-audit` (as the validation target). Future workflows
> like a `tech-spec-from-fr` skill will load this to understand FR
> structure before deriving downstream artefacts.

## How to use this template

`fr-create` reads `template.md` (in this folder) as the body skeleton and
adapts it per-FR. `fr-audit` reads it via the rule IDs encoded in
`fr-audit/RUBRIC.md` — every audit rule's "what's expected" field maps to
a region of this template. Other workflows that need to know "what is an
FR" should `read_file('cuo/_shared/feature-request-template/template.md')`
rather than hard-code the shape.

## Frontmatter contract (FM-001..111)

The frontmatter that every `feature_request@1` document MUST carry, with
audit rule IDs in parentheses (rules live in `fr-audit/RUBRIC.md`):

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

Structural rules: file begins with `---` on line 1 (FM-001); all keys
snake_case (FM-002); no duplicates (FM-003).

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

The complete skeleton lives in [`template.md`](./template.md) — sourced
verbatim from `feature-request/FR_CREATE_AND_AUDIT.md` v2.0.0 §18. The
skeleton is reproducible byte-for-byte; bumping it is a MAJOR
`skill_version` bump for this skill AND a parallel template version bump
to `feature_request@2`. The audit rubric in `fr-audit/` advances lockstep.

## Untrusted-content discipline (inherited from AGENTS.md §4.2)

- Every customer quote MUST be inside `<untrusted_content source="...">…</untrusted_content>`.
- Quotes outside the block are a SAFE-004 audit warning.
- Nested `<untrusted_content>` blocks are a SAFE-001 error.
- Unclosed blocks at EOF are a SAFE-002 error.
- The interior of `<untrusted_content>` is scanned for prompt-injection
  markers per the SAFE-003 list (also lives in `fr-audit/RUBRIC.md` §15.6).
- Attributions ("— @customer-handle, 2026-04-12") appear OUTSIDE the
  untrusted block.

## Citations

- Source artefact → `feature-request/FR_CREATE_AND_AUDIT.md` v2.0.0 §18 +
  §15.1–§15.7 (rubric).
- Untrusted-content rules → CyberOS-AGENTS.md §4.2.
- EU AI Act framing → CyberOS-PRD.docx §12.2.2; SRS DEC-064.
- Template stability → bumping template requires MAJOR persona-card bump
  for any consumer; see this skill's CHANGELOG §1.0.0 → 2.0.0 (none yet).

## History

- 2026-05-05 — v1.0.0. Initial extraction from `FR_CREATE_AND_AUDIT.md`
  v2.0.0 §18. Template body is byte-identical to the source.
