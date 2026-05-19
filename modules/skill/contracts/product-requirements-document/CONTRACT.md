---
# ── Identity ─────────────────────────────────────────────────────────
contract_id: prd
contract_version: v1
template_literal: product-requirements-document@1
description: Canonical product-requirements-document@1 schema body — frontmatter contract + Markdown skeleton for the Product Requirements Document artefact emitted by `cuo/cpo/product-requirements-document-author`. Consumed downstream by `cuo/cpo/product-requirements-document-audit` (when registered v0.2.5), `cuo/cpo/feature-request-author` (existing), and the future `cuo/chief-technology-officer/software-requirements-specification-author`.
contract_kind: artefact_schema
locked_at: 2026-05-06

# ── Stewardship ──────────────────────────────────────────────────────
steward_persona: cuo-cpo
escalation_on_breach:
  legal:    cuo-clo                  # PRD carries EU AI Act risk-class + compliance commitments
  security: cuo-cseco
  compliance: cuo-clo

# ── Determinism ──────────────────────────────────────────────────────
determinism:
  reproducible: false
  fixity_notes: "PRD authoring is judgement-heavy by nature (per Q4 of registry v0.2.4 design conversation). The CONTRACT body shape is byte-stable; PRD CONTENT is not. product-requirements-document-audit treats most rules as 'warning' rather than 'error' to acknowledge the judgement load."

# ── Source-tier emitted ──────────────────────────────────────────────
emitted_source_freshness_tier: 18   # high authority — a passed-audit PRD is source-of-truth on product intent
---

# `product-requirements-document@1` — canonical PRD contract

> A **contract**, not a skill. Holds the single source of truth for the Product Requirements Document artefact shape across CyberOS. Loaded by `cuo/cpo/product-requirements-document-author` (as the generation skeleton); will be loaded by `cuo/cpo/product-requirements-document-audit` (registry v0.2.5) as the validation target. Consumed by `cuo/cpo/feature-request-author` as the canonical PRD shape it decomposes into FRs.

## Why a separate contract for PRDs

PRD ≠ project-brief. The brief is structured intake; the PRD is the negotiated product spec. PRD audiences are broader (product, leadership, sales, eventually engineering); PRD lifecycle is different (PRD changes when goals shift, not when the intake interview is updated); PRD content is denser (acceptance criteria, user stories, quality bars). Conflating them collapses two distinct artefacts and confuses downstream consumers.

PRD ≠ SRS. The SRS (registered as `software-requirements-specification@1` in registry v0.2.6) describes what the SYSTEM does in technical detail; the PRD describes what the PRODUCT should do at a user-outcome level. They have different audiences, lifecycles, and content density.

## Frontmatter contract

The frontmatter that every `product-requirements-document@1` document MUST carry:

| Field | Type / enum | Required | Audit rule (future, in `product-requirements-document-audit/RUBRIC.md`) |
| --- | --- | --- | --- |
| `template` | const `product-requirements-document@1` | yes | FM-004 |
| `title` | string, 3–100 chars | yes | FM-101 |
| `author` | `^@[A-Za-z0-9_.-]{1,38}$` | yes | FM-102 |
| `created_at` | ISO 8601 with timezone | yes | FM-106 |
| `last_updated_at` | ISO 8601 with timezone | yes | FM-107 |
| `prd_status` | `draft` / `in_review` / `approved` / `superseded` | yes | FM-110 |
| `project_brief_ref` | path or `memory_id` of the source `project_brief@1` (required — every PRD is downstream of a brief) | yes | FM-111 |
| `target_release` | SemVer / quarter / `unspecified` (mirrors the brief's value) | yes | FM-112 |
| `client_visible` | boolean (mirrors the brief) | yes | FM-113 |
| `client_id` | required when client_visible is true | conditional | FM-114 |
| `eu_ai_act_risk_class` | mirrors the brief's value at FM-time; can be re-classified upward during PRD authoring (CLO sign-off required for upward moves) | yes | FM-115 |
| `confidentiality` | mirrors the brief; can be tightened (e.g. brief was `internal`, PRD becomes `client_confidential`) | yes | FM-116 |
| `prd_iteration` | integer ≥ 1 — increments on each amendment-batch round (mirrors feature-request-author's pattern) | yes | FM-117 |
| `chain_profile` | `lean` / `standard` / `full` — inherited from `project_brief.chain_profile`; PRD CANNOT override (chain-selector decides at brief time) | yes | FM-118 |
| `superseded_by` | optional path to a successor PRD when prd_status is `superseded` | conditional | FM-118 |
| `cl_sign_off` | optional — CLO handle + ISO timestamp when CLO has signed off on EU AI Act / compliance assertions | optional | FM-119 |
| `cseco_sign_off` | optional — CSecO handle + ISO timestamp when CSecO has signed off on threat-model | optional | FM-120 |

## Required body sections

Every `product-requirements-document@1` body MUST contain these H2 sections in this order. Each section carries an `<!-- authority: ... -->` marker per AGENTS.md §5.3 (because PRDs make strong claims and downstream consumers need to know which claims are human-edited vs. inferred).

1. **`## Background`** — link to project_brief; 2-3 paragraphs of additional context not in the brief.
2. **`## Goals`** — restated from the brief; refined where PRD authoring sharpened them. Each goal carries authority marker.
3. **`## Non-goals`** — what this PRD explicitly does NOT cover. Out-of-scope is a feature (mirrors feature-request-audit's QA-006 spirit at the PRD level).
4. **`## User Stories`** — 1-N stories, each with the form "As a <persona>, I want <capability>, so that <outcome>." Each story has its own `### Story N` H3 + acceptance criteria sub-section.
5. **`## Quality Bars`** — performance, availability, privacy, accessibility, security baselines the PRD commits to. Each as a measurable target with baseline + threshold.
6. **`## Open Questions`** — questions still requiring decision (carried over from the brief or surfaced during PRD authoring). Each has a `<!-- needs: <persona|human> -->` marker.
7. **`## EU AI Act Considerations`** — required if eu_ai_act_risk_class ∈ {limited, high}; otherwise the section contains the explicit statement "Not in scope of EU AI Act — feature involves no AI/ML inference / biometric data / Annex III activity. Reviewed: <ISO date> by <persona|human>."
8. **`## Compliance and Privacy`** — what frameworks are in play (GDPR / HIPAA / SOC 2 / etc.); what data flows touch PII; what consent is required.
9. **`## Rough Sizing`** — high-level engineering effort estimate (N × engineer-month). This is a hint for downstream `software-requirements-specification-author` and `feature-request-author`, not a commitment.
10. **`## Success Definition`** — what does success look like 12 weeks post-launch? Required even if the brief had similar; PRD restates with sharper measurable criteria.
11. **`## Research Signals`** — appendix listing the Common Room signals, customer interviews, in-product feedback, support tickets, or other evidence that triggered this PRD. Required even if "no formal research; founder intuition based on <reasoning>" — forces honesty.

## Conditionally-required sections

| Trigger | Required section |
| --- | --- |
| `client_visible: true` | `## Client Context` — re-stated with PRD-level detail (deliverables, milestones, acceptance) |
| `eu_ai_act_risk_class: high` | `## High-Risk AI Risk Assessment` — full per-Article-50 breakdown (Annex III mapping, oversight mechanism, transparency obligations, post-market monitoring) |
| `confidentiality ∈ {client_confidential, regulated}` | `## Compliance Implementation Plan` — concrete controls (encryption-at-rest, audit logging, retention rules, BCDR) |
| `prd_status: approved` | `## Approval Record` — table of who approved + ISO date + version-hash |

## Authority markers (AGENTS.md §5.3 alignment)

Every claim in `## Goals`, `## User Stories`, `## Quality Bars`, `## Success Definition` carries an inline marker:

```markdown
1. <!-- authority: human-edited --> Eliminate the daily filter-rebuild ritual for power users.
2. <!-- authority: llm-explicit --> Reduce median time-to-first-triage by ~70% (inferred from CSM interview data).
```

Authority hierarchy:

- `human-edited` — founder / product owner literally typed it OR explicitly approved during interview.
- `human-confirmed` — subject (e.g. a CSM) self-disclosed during interview; LLM transcribed verbatim.
- `llm-explicit` — LLM synthesised it from cited input documents / memory entries; cited refs in surrounding HTML comment.
- `llm-implicit` — LLM inferred it without a specific citable source.

`product-requirements-document-audit` (registry v0.2.5) will REJECT any PRD with `llm-implicit` authority on a `## Goals` claim — goals MUST be at least `llm-explicit`. Lower-authority claims are flagged for human review at PRD-approval time.

## Iteration model

`product-requirements-document-author` follows feature-request-author's amendment-batch protocol (per Q5 of registry v0.2.4 design):

1. v1 of the PRD is authored from the brief + targeted memory reads.
2. User reviews, batches amendments via the standard PLAN_AMENDMENT_REQUEST format.
3. `product-requirements-document-author` applies the batch, increments `prd_iteration`, rewrites the same `<title>.prd.md` file in place.
4. Goes to v3 etc. until `prd_status` flips to `in_review` and an audit pass is triggered.

## Citations

- Registry v0.2.4 — first contract authored downstream of `project_brief@1`.
- DEC-090 — skills↔contracts split.
- AGENTS.md §5.3 — authority hierarchy for the embedded markers.
- `cuo/cpo/feature-request-author/references/AMENDMENT_PROTOCOL.md` — amendment-batch pattern this contract's author skill mirrors.
- Future consumers: `cuo/cpo/product-requirements-document-author` v0.1.0 (this version), `cuo/cpo/product-requirements-document-audit` v0.1.0 (registry v0.2.5), `cuo/cpo/feature-request-author` v0.3.0+ (when feature-request-author migrates to consume `product-requirements-document@1` instead of generic "PRD/spec docs").
