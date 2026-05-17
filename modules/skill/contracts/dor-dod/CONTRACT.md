---
contract_id: dor-dod
contract_version: v1
template_literal: dor-dod@1
description: Canonical dor-dod@1 schema — project-level Definition of Ready + Definition of Done declaration. Authored by dor-dod-author; validated by dor-dod-audit via dor_dod_rubric@1.0. Implements Software Development Process.md Templates §4.1 and §4.2.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-cpo
escalation_on_breach:
  legal:      cuo-clo
  security:   cuo-cseco
  compliance: cuo-clo

determinism:
  reproducible: false
  fixity_notes: "Authoring includes operator policy choice (coverage threshold, conditional triggers). Contract body shape is byte-stable; declared values are not."

emitted_source_freshness_tier: 18
---

# `dor-dod@1` — canonical Definition of Ready + Definition of Done contract

> A **contract**, not a skill. One DoR/DoD declaration per engagement. Loaded by `dor-dod-author` (generation) and `dor-dod-audit` (validation). Authoritative rule set lives at `dor-dod-audit/RUBRIC.md` `dor_dod_rubric@1.0`.

## Why a separate contract

The DoR/DoD is the team's project-level quality contract. It is referenced by every downstream artefact's `## DoR/DoD compliance` checks. Splitting it from sprint-level tooling (Jira / Linear) keeps the policy literal-text-versioned and auditable.

## Frontmatter contract

See `dor-dod-audit/RUBRIC.md` §2 (`FM-101..109`). Required: `title`, `author`, `project`, `engagement_model`, `effective_date`, `dor_dod_version`, `provenance.{source_path,source_hash}`, `approved_by[]`.

## Required body sections

Five sections per `dor-dod-audit/RUBRIC.md` §3 (`SEC-001..005`): Definition of Ready, Definition of Done, Scope of Application, Waivers and Exceptions, Review Cadence.

## Mandatory DoR items (per `dor-dod-audit/RUBRIC.md` §4)

`DOR-001..008`: clear user value, acceptance criteria, dependencies, NFRs, security/privacy flags, designs, estimable, demoable.

## Mandatory DoD items (per `dor-dod-audit/RUBRIC.md` §5)

`DOD-001..010`: merged, unit tests, integration tests, coverage threshold (declared), SAST clean, SCA clean, docs updated, deployed to staging, PO accepted, observability hooked.

## Citations

- `cyberos/docs/Software Development Process.md` §4.1, §4.2 — DoR/DoD source.
- Consumers: `dor-dod-author` (generation), `dor-dod-audit` (validation), every downstream skill's `### Compliance` checks.
