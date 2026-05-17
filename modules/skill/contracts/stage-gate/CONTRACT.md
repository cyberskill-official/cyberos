---
contract_id: stage-gate
contract_version: v1
template_literal: stage-gate@1
description: Canonical stage-gate@1 — one-page Go / Go-with-conditions / No-Go sign-off for a SDLC stage boundary. Authored by stage-gate-author; validated by stage-gate-audit via stage_gate_rubric@1.0. Implements Software Development Process.md Template §4.3.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-cpo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }

determinism: { reproducible: false, fixity_notes: "Decision rationale is judgement; signers + dates are reproducible." }

emitted_source_freshness_tier: 12
---

# `stage-gate@1` — canonical Stage-Gate Sign-Off contract

> A **contract**, not a skill. One artefact per stage boundary in fixed-price engagements (optional in T&M). Loaded by `stage-gate-author` (generation) + `stage-gate-audit` (validation, `stage_gate_rubric@1.0`).

## Why

Stage gates are the auditable Go/No-Go decisions that bind a fixed-price engagement to its phased payment + acceptance schedule. The contract makes the gate record portable across CRM / DMS / archival systems.

## Frontmatter contract — see `stage-gate-audit/RUBRIC.md` §2 (`FM-101..110`).

## Required body sections — see `stage-gate-audit/RUBRIC.md` §3 (`SEC-001..007`): Stage / Entry Criteria Met / Exit Criteria Met / Open Risks and Issues / Decision / Conditions / Signatures.

## Conditional sections — `COND-001..004`: triggered by decision value + stage (i) Deployment.

## Citations

- SDP Template §4.3 — Stage-gate skeleton source.
- Consumers: `stage-gate-author`, `stage-gate-audit`, downstream `closure-author` (uses gate history for sign-off package).
