---
name: chain-selector
description: "Decides which downstream skills run after a `project_brief@1` is approved. Reads the brief's `project_kind`, `eu_ai_act_risk_class`, `confidentiality`, `budget_band`, and `target_release` to pick a chain_profile (lean / standard / full). User can override at brief-completion time. Outputs the chain plan as a list of skill_ids the supervisor will route through. Project-kind-agnostic (works for software, marketing, hiring, partnerships, research)."
skill_version: 0.1.0
persona: cuo
owner_role: cpo

allowed_brain_scopes:
  read:
    - project:*
    - company:locked-decisions
    - company:values
    - memories:projects
    - memories:decisions
    - memories:refinements
    - persona:cuo-*
  write:
    - project:*

allowed_mcp_tools:
  - brain.search
  - kb.read
  - audit.append
  - chat.notify

escalation:
  to_persona_on_legal: cuo-clo
  to_persona_on_security: cuo-cseco
  to_persona_on_compliance: cuo-clo
  to_human_on_irreversible: true

invocation_modes: [chained]      # invoked by supervisor at brief-completion time; not standalone

expects:
  schema_ref: ./envelopes/chain-selector.input.json
  required_fields: [brief_path]
  optional_fields:
    - user_override               # explicit user choice if the auto-selection is wrong
    - trace_id
    - caller_persona
  standalone_interview_ref: null  # chained-only

produces:
  schema_ref: ./envelopes/chain-selector.output.json
  output_kind: [chain_plan_artefact]
  human_summary_ref: ./HUMAN_SUMMARY.md

depends_on_contracts:
  - id:        project-brief
    version:   v1
    purpose:   input_schema
    pin_path:  cyberos/docs/contracts/project-brief/

exposable_as:
  internal:           true
  agent_plugin:       false      # tightly coupled to the chain; not useful as a plugin in isolation
  mcp_tool:           false
  partner_connector:  false

self_audit:
  invariants_ref: ./INVARIANTS.md
  check_at: [contract_echo, after_brief_read, before_emit]
  anomaly_signals:
    user_override_rate_above: {threshold: 0.5, window: 20}    # >50% of users override → re-tune the rules
    chain_plan_emit_failure: {threshold: 1, window: 1}
  on_breach:
    emit: refinement_proposal
    pause_pipeline: true

human_fine_tune:
  fine_tuner_role: cpo
  review_required:
    on_minor_bump:    false
    on_major_bump:    true
    on_safety_change: true
    on_owned_workflows_change: false
    on_selection_rule_changed: true   # the chain_profile rules ARE the skill; changes need cpo + registry review
  signals_to_initiate:
    - user_override_rate_above: 0.5
    - chain_plan_emit_failure_streak: 2
  procedure_ref: ../../../README.md#part-7--manual-fine-tune-the-human-loop
  required_artifacts:
    - changelog_entry
    - selection_rule_diff
    - memory_refinement_entry

audit:
  emit_to: genie.action_log
  row_kind: chain_plan_emitted
  payload_hash_field: chain_plan_sha256
  explanation_pane: required

confidence_band:
  default: 0.85          # mostly mechanical (table-lookup); some judgement at edge cases
  defer_below: 0.5
  cite_sources: required

untrusted_inputs:
  wrap_in: <untrusted_content/>
  injection_scan: required
  on_marker_hit: surface_to_human

determinism:
  reproducible: true
  fixity_notes: "Same brief frontmatter → same chain_plan. Selection rules are byte-stable per skill_version."

emitted_source_freshness_tier: 25
gated_until_phase: runtime_v0_3_0
---

# `chain-selector` — pick the chain profile for a project

> **Scaffold-only at v0.1.0.** Documents the intended contract; runtime in v0.3.0.

## What this skill does

Reads a `project_brief@1` markdown's frontmatter, applies the selection rules below, and emits a `chain_plan` — a list of skill_ids the supervisor will route through. Invoked automatically by the supervisor at brief-completion time (when `requirements-discovery` outputs `BRIEF_COMPLETE`). The user CAN override the auto-selection via the `user_override` field.

## Selection rules (v0.1.0 — likely to evolve)

The rules apply in order; first match wins:

1. **`full`** if any of:
   - `eu_ai_act_risk_class: high`
   - `confidentiality: regulated`
   - `budget_band: over_100k` AND `target_release` is a long-term release (>1 year out)
   - `client_visible: true` AND `budget_band ∈ {25k_to_100k, over_100k}` (commissioned client work above threshold)
2. **`lean`** if any of:
   - `project_kind ∈ {internal_tooling, research_spike}`
   - `budget_band ∈ {none, under_5k}`
   - `target_release` is a near-term sprint (<2 weeks out) AND `eu_ai_act_risk_class ∈ {not_ai, minimal}`
3. **`standard`** otherwise (default).

Rule edits require `on_selection_rule_changed: true` review per `human_fine_tune` (cpo + registry maintainer must approve).

## Chain plan per profile

| Profile | Skills (in order) |
| --- | --- |
| `lean` | `cuo/cpo/prd-author` → `cuo/cpo/fr-author` → `cuo/cpo/fr-audit` → `cuo/cto/spec-to-impl-plan` |
| `standard` | `cuo/cpo/prd-author` → `cuo/cpo/prd-audit` → `cuo/cpo/fr-author` → `cuo/cpo/fr-audit` → `cuo/cto/fr-to-tech-spec` → `cuo/cto/spec-to-impl-plan` |
| `full` | `cuo/cpo/prd-author` → `cuo/cpo/prd-audit` → `cuo/cto/srs-author` → `cuo/cto/srs-audit` → `cuo/cpo/fr-author` → `cuo/cpo/fr-audit` → `cuo/cto/fr-to-tech-spec` → `cuo/cto/spec-to-impl-plan` |

The chain plan is written to a `<brief-slug>.chain-plan.md` file alongside the brief AND emitted as the output envelope's `chain_plan` field. The supervisor consumes the field directly; the markdown file is for the user's visibility + future audit.

## User override

If the user disagrees with the auto-selection, they can override at brief-completion time:

> "I see you classified this as `standard`. I'd rather run `lean` because the engineering effort is small. Want to override?"

The user types `lean` (or any other profile name); the skill records the override + reasoning in the chain-plan artefact AND in `memories/projects/<slug>.md`. High user-override rates (>50% over 20 invocations) trigger the manual fine-tune flow per `human_fine_tune.signals_to_initiate`.

## Pipeline position

```
cuo/cpo/requirements-discovery → project_brief@1 (triage_verdict: proceed)
    ↓
cuo/cpo/chain-selector  (THIS SKILL — auto-invoked by supervisor)
    ↓
chain_plan (list of skill_ids)
    ↓
supervisor routes through the chain in declared order
```

## Scaffold contains

```
cuo/cpo/chain-selector/
├── SKILL.md
├── CHANGELOG.md
├── INVARIANTS.md
├── HUMAN_SUMMARY.md
├── envelopes/{input,output}.json
└── acceptance/README.md
```

No STANDALONE_INTERVIEW.md (chained-only) and no reference docs (the selection rules ARE the skill body).

## Citations

- v0.2.8 design — Q2-B of registry v0.2.7 conversation: chain-selector via `chain_profile:` field.
- `project_brief@1.chain_profile` field — added in registry v0.2.8.
- `prd@1.chain_profile` field — inherited from brief; PRD cannot override.
- Selection rules — likely to evolve; rule edits gated by `human_fine_tune.on_selection_rule_changed: true`.
