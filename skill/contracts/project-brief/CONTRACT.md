---
# ── Identity ─────────────────────────────────────────────────────────
contract_id: project-brief
contract_version: v1
template_literal: project_brief@1
description: "Canonical project_brief@1 schema body — frontmatter contract + Markdown skeleton for the structured intake artefact emitted by `cuo/cpo/requirements-discovery`. The brief captures everything needed downstream by `prd-author` (and any other consumer) — goals, audience, constraints, kill-criteria, regulatory context, BRAIN-derived background, stakeholder map, project_kind classification, target release, and a triage verdict (proceed / revise / reject)."
contract_kind: artefact_schema
locked_at: 2026-05-06

# ── Stewardship ──────────────────────────────────────────────────────
steward_persona: cuo-cpo
escalation_on_breach:
  legal:    cuo-clo                  # the brief carries EU AI Act risk-class and may carry compliance fields
  security: cuo-cseco                # threat-model assertions surface here when relevant
  compliance: cuo-clo

# ── Determinism ──────────────────────────────────────────────────────
determinism:
  reproducible: true
  fixity_notes: "Template body is byte-stable. Bumping the template body requires a MAJOR contract_version bump (project_brief@2) and a coordinated update to every consumer (requirements-discovery, prd-author)."

# ── Source-tier emitted ──────────────────────────────────────────────
emitted_source_freshness_tier: 12   # high authority — the project brief IS the structured intake schema
---

# `project_brief@1` — canonical project-brief contract

> A **contract**, not a skill. Holds the single source of truth for the project-brief artefact shape across CyberOS. Loaded by `cuo/cpo/requirements-discovery` (as the generation skeleton) and `cuo/cpo/prd-author` (as the input shape it consumes). Future skills (e.g., `cuo/cao/sales-playbook-author` for non-software project kinds) may also consume this contract.

## When to use this contract

A project brief sits between "user has an idea" and "we have a PRD." It captures the structured output of the discovery interview — goals, audience, success metrics, constraints, kill criteria, regulatory context, BRAIN-derived background, stakeholder map, project_kind classification, target release, and a triage verdict. The PRD-author (or any downstream skill) consumes the brief instead of re-asking the user the same intake questions.

## Frontmatter contract

The frontmatter that every `project_brief@1` document MUST carry, with audit rule IDs in parentheses (rules will live in `cuo/cpo/prd-audit/RUBRIC.md` once that skill ships at registry v0.2.5):

| Field | Type / enum | Required | Audit rule (future) |
| --- | --- | --- | --- |
| `template` | const `project_brief@1` | yes | FM-004 |
| `title` | string, 3–80 chars | yes | FM-101 |
| `author` | `^@[A-Za-z0-9_.-]{1,38}$` | yes | FM-102 |
| `created_at` | ISO 8601 with timezone | yes | FM-106 |
| `last_updated_at` | ISO 8601 with timezone | yes | FM-107 |
| `project_kind` | `software_product` / `software_consulting_engagement` / `internal_tooling` / `marketing_campaign` / `hiring_plan` / `partnership` / `research_spike` / `other` | yes | FM-110 |
| `triage_verdict` | `proceed` / `revise` / `reject` (set by requirements-discovery; gates downstream) | yes | FM-111 |
| `triage_reason` | string, 1–500 chars; required when triage_verdict ∈ {revise, reject} | conditional | FM-112 |
| `target_release` | SemVer / quarter (`2026-Q3`) / `unspecified` | yes | FM-113 |
| `client_visible` | boolean — true if a specific client commissioned this | yes | FM-114 |
| `client_id` | string; required when client_visible is true | conditional | FM-115 |
| `eu_ai_act_risk_class` | `not_ai` / `minimal` / `limited` / `high` (`unacceptable` rejected) | yes | FM-116 |
| `confidentiality` | `public` / `internal` / `client_confidential` / `regulated` | yes | FM-117 |
| `budget_band` | `none` / `under_5k` / `5k_to_25k` / `25k_to_100k` / `over_100k` / `undisclosed` | optional | FM-118 |
| `team_capacity_check_passed` | boolean — has team headcount + skills been verified? | yes | FM-119 |
| `discovery_iteration` | integer ≥ 1 — increments on each amendment-batch round | yes | FM-120 |
| `chain_profile` | `lean` / `standard` / `full` — selects which downstream skills run; defaults from project_kind via `cuo/cpo/chain-selector` (v0.2.8+) | yes | FM-121 |

## Required body sections

Every `project_brief@1` body MUST contain these H2 sections in this order:

1. **`## Background`** — 2-5 paragraphs of context. WHY are we considering this? What signal triggered it? BRAIN-citations are encouraged but optional in v1.
2. **`## Goals`** — 1-5 numbered goals; each is a ≤2-sentence statement of an outcome (not an output) the project must achieve. Each goal carries an embedded `<!-- authority: human-edited|human-confirmed|llm-explicit|llm-implicit -->` marker per AGENTS.md §5.3.
3. **`## Audience`** — who benefits? Internal users / external customers / specific personas / a specific client. Be specific; "users" is rejected by the rubric (when prd-audit ships).
4. **`## Success Metrics`** — at minimum 1 primary metric with baseline + target + deadline; up to 1 guardrail metric. Vanity metrics (signups without definition, views without engagement context) are rejected.
5. **`## Constraints`** — what's NOT negotiable: timeline, budget, regulatory, technical platform, headcount. List each as a bullet.
6. **`## Kill Criteria`** — under what observable conditions does the project STOP? "We'd kill this if [X observable signal]". Forces honesty about when to walk away.
7. **`## Stakeholder Map`** — table or list of who decides / who reviews / who's informed. Required even if it's just "founder decides" — the seam between solo decisions and team decisions matters downstream.
8. **`## Prior Art (BRAIN)`** — what does the BRAIN tell us we already tried, decided, or learned? Cite `memories/decisions/DEC-NNN-*.md`, `memories/projects/<project>.md`, `company/locked-decisions.md` paths. If nothing relevant, write "No relevant prior art found in BRAIN as of <ISO date>."
9. **`## Open Questions`** — what couldn't be answered during discovery? Each question gets a `<!-- needs: <persona|human> -->` marker indicating who should answer. If empty, the brief carries an explicit statement ("No open questions — all required intake answered.").

## Chain profile (v0.2.8+)

`chain_profile` selects which downstream skills run after the brief is approved. Three profiles:

| Profile | Default for | Skills that run | Skills SKIPPED |
| --- | --- | --- | --- |
| `lean` | `internal_tooling`, `research_spike`, projects under ~2 engineer-weeks | prd-author → fr-author → fr-audit → spec-to-impl-plan | prd-audit, srs-author, srs-audit, fr-to-tech-spec |
| `standard` (default) | `software_product`, `software_consulting_engagement`, projects 2-12 engineer-weeks | prd-author → prd-audit → fr-author → fr-audit → fr-to-tech-spec → spec-to-impl-plan | srs-author, srs-audit |
| `full` | `confidentiality: regulated`, `eu_ai_act_risk_class: high`, multi-year projects | every skill in the chain | (none — all run) |

`requirements-discovery` defaults the `chain_profile` from the project_kind + EU AI Act risk class + confidentiality + budget combination. The user can override during the discovery interview ("I want lean for this small experiment" → set `chain_profile: lean`). The `cuo/cpo/chain-selector` skill (v0.2.8+) is invoked by the supervisor at brief-completion time to validate the choice and emit the chain plan.

## Conditionally-required sections

| Trigger | Required section |
| --- | --- |
| `client_visible: true` | `## Client Context` — who's the client, what they've signed (NDA, MSA, SOW), and any known sensitivities |
| `eu_ai_act_risk_class ∈ {limited, high}` | `## AI Risk Snapshot` — preliminary read on data sources, oversight, and failure modes; full assessment lands in PRD |
| `confidentiality ∈ {client_confidential, regulated}` | `## Compliance Constraints` — relevant frameworks (GDPR / HIPAA / SOC 2 / etc.) and what they require |
| `triage_verdict ∈ {revise, reject}` | `## Triage Reasoning` — why the project was downgraded; what would change the verdict |

## How `requirements-discovery` produces this contract

The discovery skill conducts a 15-20 question interview (per its `STANDALONE_INTERVIEW.md`), folds in a project-triage assessment (gating questions about strategic fit, capacity, runway, customer signal strength), reads BRAIN scopes for prior art, then synthesises the answers into this artefact. Iteration via amendment-batch protocol (mirroring fr-author's): the user reviews v1, batches amendments, and the skill applies them in v2. `discovery_iteration` increments; the same `project_brief@1.md` file is rewritten in place.

## How `prd-author` consumes this contract

`prd-author` reads the brief as its primary input. It does NOT re-ask the user the intake questions — those are answered in the brief. It DOES read additional BRAIN scopes (specifically `module:*` for technical-context lookup) and conduct a smaller follow-up interview (3-5 questions) for PRD-specific decisions (e.g., feature-flag strategy, rollout plan, telemetry).

## Citations

- DEC-090 (registry v0.2.0) — split contracts from skills.
- Registry v0.2.4 — first contract authored under the simplified flat-folder layout (no `v<n>/` folder).
- Registry README Part 8 — full skill-vs-contract semantics.
- AGENTS.md §5.3 — authority hierarchy for the embedded markers in goal statements.
- Future consumers: `cuo/cpo/requirements-discovery` v0.1.0 (this version), `cuo/cpo/prd-author` v0.1.0 (this version), `cuo/cpo/prd-audit` v0.1.0 (registry v0.2.5).
