---
contract_id: srs
contract_version: v1
template_literal: srs@1
description: "Canonical srs@1 schema body — frontmatter contract + Markdown skeleton for the Software Requirements Specification artefact emitted by `cuo/cto/srs-author`. Consumed by `cuo/cto/srs-audit` (validation target) and downstream by `cuo/cto/fr-to-tech-spec` (input context). Documents the system in technical detail (architecture, runtime, data flows, non-functional requirements); distinct from the `prd@1` contract which describes the product at a user-outcome level."
contract_kind: artefact_schema
locked_at: 2026-05-06

steward_persona: cuo-cto
escalation_on_breach:
  legal:    cuo-clo
  security: cuo-cseco
  compliance: cuo-clo

determinism:
  reproducible: false
  fixity_notes: "SRS authoring is judgement-heavy (architecture decisions, sizing). The CONTRACT body shape is byte-stable; SRS content is not. srs-audit treats most rules as 'warning' rather than 'error', mirroring prd-audit's advisory-leaning approach."

emitted_source_freshness_tier: 20
---

# `srs@1` — canonical SRS contract

> Loaded by `cuo/cto/srs-author` (generation skeleton); will be loaded by `cuo/cto/srs-audit` v0.1.0 (registry v0.2.6) as validation target. Sits downstream of `prd@1`: every SRS references a PRD by `prd_ref` field. SRS is the engineering-side detailing of what the product spec promises.

## Why a separate contract for SRS

SRS ≠ PRD. PRD answers "what should we build and why?" (audience: product, leadership). SRS answers "what does the system actually do, in technical detail?" (audience: engineering, ops). They have different lifecycles (PRD changes when goals shift; SRS changes when tech changes), different content density, different review processes (PRDs go through stakeholder approval; SRSs go through architectural review).

SRS ≠ tech-spec. The tech-spec (`tech_spec@1`, future contract owned by fr-to-tech-spec) decomposes a single FR into work-packages. The SRS describes the SYSTEM as a whole — components, data flows, runtime mechanisms, NFRs (non-functional requirements). One PRD typically begets one SRS; one SRS typically begets many tech-specs (one per FR).

## Frontmatter contract

| Field | Type / enum | Required | Audit rule (future, in `srs-audit/RUBRIC.md`) |
| --- | --- | --- | --- |
| `template` | const `srs@1` | yes | FM-004 |
| `title` | string, 3-100 chars | yes | FM-101 |
| `author` | `^@[A-Za-z0-9_.-]{1,38}$` | yes | FM-102 |
| `created_at` | ISO 8601 with timezone | yes | FM-106 |
| `last_updated_at` | ISO 8601 with timezone | yes | FM-107 |
| `srs_status` | `draft` / `in_review` / `approved` / `superseded` | yes | FM-110 |
| `prd_ref` | path or memory_id of the source `prd@1` (required — every SRS is downstream of a PRD) | yes | FM-111 |
| `target_release` | mirrors PRD's value | yes | FM-112 |
| `srs_iteration` | integer ≥ 1 | yes | FM-113 |
| `superseded_by` | required when `srs_status: superseded` | conditional | FM-114 |
| `cseco_sign_off` | optional — CSecO handle + ISO timestamp | optional | FM-115 |
| `architectural_review_passed` | boolean — set true after CTO + at least one engineer review | yes | FM-116 |

## Required body sections

Every `srs@1` body MUST contain these H2 sections in order:

1. **`## Background`** — link to PRD; 1-2 paragraphs on technical context.
2. **`## System Architecture`** — components touched + integration points; diagram or numbered component list.
3. **`## Data Model`** — entities, relationships, schema deltas (if any). Migrations called out.
4. **`## API Surface`** — new / changed / deprecated endpoints. Per-endpoint: method, path, request schema, response schema, idempotency.
5. **`## Data Flows`** — per primary user story from PRD: end-to-end sequence (frontend → backend → datastore → external integrations → audit log).
6. **`## Non-Functional Requirements`** — performance, availability, durability, scalability, security, observability targets. Each as a measurable threshold.
7. **`## Failure Modes`** — what can go wrong + how the system handles each (graceful degradation, retry, circuit-break, alert).
8. **`## Security Posture`** — auth/authz, secret-store usage, encryption-at-rest decisions, audit trail.
9. **`## Telemetry Plan`** — what events MUST land in `genie.action_log` (or equivalent); what metrics MUST be exported.
10. **`## Open Architectural Questions`** — what couldn't be decided from the PRD alone; each carries `<!-- needs: <persona|human> -->`.

## Conditionally-required sections

| Trigger | Required section |
| --- | --- |
| `prd_ref` resolves to a PRD with `eu_ai_act_risk_class: high` | `## AI Subsystem Spec` — model details, oversight implementation, transparency mechanism |
| PRD has `confidentiality ∈ {client_confidential, regulated}` | `## Compliance Implementation` — encryption, audit log, retention, BCDR specifics |
| `architectural_review_passed: true` | `## Review Record` — table of reviewer + role + ISO ts + version-hash |

## Authority markers

Same pattern as `prd@1`. Every claim in `## System Architecture`, `## API Surface`, `## Non-Functional Requirements`, `## Telemetry Plan` carries an inline `<!-- authority: ... -->` marker. `srs-audit` (registry v0.2.6) will REJECT any SRS with `llm-implicit` authority on a `## System Architecture` claim.

## Citations

- Registry v0.2.4 — flat-folder layout established.
- Registry v0.2.6 — first version of this contract registered.
- DEC-090 — skills↔contracts split.
- AGENTS.md §5.3 — authority hierarchy.
- Future consumers: `cuo/cto/srs-author` v0.1.0, `cuo/cto/srs-audit` v0.1.0, `cuo/cto/fr-to-tech-spec` v0.2.0+.
