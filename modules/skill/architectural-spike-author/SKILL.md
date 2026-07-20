---
# ── Identity ─────────────────────────────────────────────────────────
name: architectural-spike-author
description: >-
  Run a time-boxed architectural spike when an ADR is pending with >= 2 viable options, or the task introduces a dependency the repo has never used - producing architectural-spike@1: the single question under investigation, options probed with CHECKABLE evidence (a repo file path, a command plus its observed output, or a URL), cost estimates, risks, exactly one recommendation with a confidence grade, and a discard log. Records planned vs actual hours and HALTS for the operator when actual exceeds plan by more than 50%. Feeds architecture-decision-record-author as its spike input. Use when user asks to "draft an architectural spike", "create the architectural spike", or "run a spike on X vs Y". Do NOT use for "audit existing architectural spike" (use architectural-spike-audit instead).
license: Apache-2.0
metadata:
  version: 1.0.0
  module: skill
  stage: d
  cyberos-template: architectural-spike@1
  cyberos-rubric-target: architectural_spike_rubric@1.0

# ── Scope contract (memory/AGENTS.md §15) ────────────────────────────
allowed_memory_scopes:
  read:
    - project:*
    - module:*
    - memories:decisions
  write:
    - project:task/{task_id}/architectural-spike
audit:
  row_kind: architectural_spike_authored
  required_fields: [task_id, spike_id, question, options_probed, recommendation, confidence, timebox_hours, actual_hours, halted]

# ── Inputs / outputs ─────────────────────────────────────────────────
inputs:
  - { name: task,               format: task@1,  required: true }
  - { name: repo_context_map, format: repo-context-map@1, required: true }
  - { name: question,         format: string,             required: true }
  - { name: timebox_hours,    format: integer,            required: true }
outputs:
  - { name: architectural_spike, format: architectural-spike@1 }

# ── Triggers / blockers ──────────────────────────────────────────────
triggers:
  - an ADR is pending for the task and >= 2 viable options exist
  - the task introduces a dependency the repo has never used (per repo-context-map)
blockers:
  - "single obvious option - do NOT spike; proceed to the ADR with evidence inline (lean fallback)"
  - "actual hours exceed 1.5x the timebox - HALT for the operator; never keep probing silently"
---

# architectural-spike-author

## 1. Purpose

Turn "we have 2+ plausible architectures" into evidence an ADR can cite, inside a bounded spend. A spike without checkable evidence is an opinion; a spike without a recorded timebox is a design project. This skill forbids both.

## 2. Artefact schema - architectural-spike@1

Frontmatter (all fields required unless marked):

| field | type | rule |
|---|---|---|
| spike_id | string | `SPIKE-<task-ID>-<n>`, n = 1-based per task |
| task_id | string | the task under investigation |
| question | string | the SINGLE decision under investigation |
| timebox_hours | integer | recorded BEFORE probing starts |
| actual_hours | number | recorded at close |
| halted | boolean | true when the >1.5x HALT fired |
| options | array | each: { name, hypothesis, evidence[], cost_estimate, risks[] } |
| recommendation | string | names EXACTLY ONE probed option |
| confidence | enum | low, medium, high |
| discarded | array | each: { name, reason } - non-empty whenever options were rejected |
| created | date | |

Body sections, in order: `## Question`, `## Options probed`, `## Evidence log`, `## Recommendation`, `## Discard log`.

## 3. The evidence rule

Every `evidence[]` entry MUST cite something checkable: a file path in the repo, a command plus its observed output, or an external URL. Unsupported assertions ("X is faster") count as ZERO evidence - the audit (SPK-EVID) rejects options carrying only uncited claims. `confidence: high` requires >= 2 evidence entries per surviving option.

## 4. Timebox discipline

`timebox_hours` is recorded up front (INV-1). At close, `actual_hours` is recorded; when actual > 1.5x plan the skill HALTS with: "spike over budget (<actual>h vs <plan>h planned) - extend the timebox, force a recommendation from current evidence, or discard the spike?" The operator's verdict is recorded in the artefact (`halted: true` plus the outcome in the Discard log or Recommendation section).

## 5. Handoff

The artefact feeds `architecture-decision-record-author` as its spike input. When no spike exists (single obvious option - see blockers), the ADR proceeds in lean profile: its options table carries the evidence inline.

See PIPELINE.md for the phased run, INVARIANTS.md for the checkable rules, and references/FAILURE_MODES.md before authoring.
