---
contract_id: chain-manifest
contract_version: v1
template_literal: chain_manifest@1
description: Canonical chain-manifest@1 schema — persistent state for a `cyberos chain run` invocation. Tracks profile, plan, per-step status, retry budgets, cost, HITL pause state. Enables `cyberos chain resume` to pick up where a previous run halted.
contract_kind: artefact_schema
locked_at: 2026-05-12
introduced_by: skills-Stage-4-runtime

steward_persona: cuo-cpo
escalation_on_breach:
  legal: cuo-clo
  security: cuo-cseco
  compliance: cuo-clo

determinism:
  reproducible: true
  fixity_notes: "Field shape stable. Values vary per run."

emitted_source_freshness_tier: 10
---

# `chain_manifest@1` — persistent state for cyberos chain run

## Required fields

```json
{
  "schema_version": 1,
  "created_at": "<ISO-8601>",
  "profile": "solo | lean | standard | full",
  "skip_prd": false,
  "triage_reasons": ["..."],
  "slug": "<kebab-slug>",
  "output_dir": "<absolute-path>",
  "pitch_first_120": "<truncated pitch>",
  "spec_file": "<path-or-null>",
  "with_llm": false,
  "model": "claude-sonnet-4-6",

  "plan": [
    {
      "step": 1,
      "skill_id": "cuo/cpo/fr-with-tasks",
      "status": "pending | placeholder | in_progress | done | skipped | hitl_paused | exhausted | failed",
      "started_at": "<ISO|null>",
      "completed_at": "<ISO|null>",
      "iterations": 0,
      "max_iterations": 3,
      "skipped_reason": "<string|null>",
      "hitl_question": "<string|null>",
      "hitl_resolved_at": "<ISO|null>",
      "tokens_used": 0,
      "cost_usd": 0.0,
      "output_paths": ["<path>", ...],
      "audit_row_ids": ["evt_...", ...]
    }
  ],

  "status": "PLANNED | RUNNING | PLACEHOLDERS_WRITTEN | HITL_PAUSE | DONE | FAILED",
  "budget": {
    "max_tokens": 100000,
    "max_cost_usd": 1.00,
    "tokens_used_total": 0,
    "cost_usd_total": 0.0
  },
  "calibration": {
    "predicted_human_intervention_pct": 0.0,
    "actual_human_intervention_pct": null
  }
}
```

## Lifecycle

```
PLANNED ──► RUNNING ──┬──► DONE
                     ├──► HITL_PAUSE ──► RUNNING (after resume)
                     ├──► PLACEHOLDERS_WRITTEN (today's default; runtime not wired)
                     └──► FAILED
```

## Resume semantics

`cyberos chain resume <manifest-path>`:

1. Load the manifest
2. Find the first step with status ∈ {pending, hitl_paused, in_progress, exhausted}
3. Continue from there. For `hitl_paused` steps, ask the operator the recorded `hitl_question`, then write the answer + flip status.
4. Update manifest in place; audit row appended per phase transition.

Steps marked `done | skipped` are never re-run on resume.

## Budget enforcement

If `tokens_used_total >= max_tokens` OR `cost_usd_total >= max_cost_usd`, the chain refuses to start the next step and pauses with status `HITL_PAUSE` + `hitl_question: "Budget exceeded; increase budget or abort?"`.

## Calibration tracking (Stage 6.4)

`predicted_human_intervention_pct` is recorded at plan time based on the skill's `trust_calibration` field. `actual_human_intervention_pct` is updated after the chain finishes by counting the proportion of steps that hit a HITL gate. Used by `cyberos skill calibration` to surface overconfident skills.

## Versioning

- `chain_manifest@1` introduced 2026-05-12.
- Adding optional fields = minor (no version bump).
- Renaming or removing required fields = MAJOR (`chain_manifest@2` + consumer bumps).
