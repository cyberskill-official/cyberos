---
name: srs-audit
description: "Quality gate on SRSs. Audits one or more `srs@1` markdowns against `srs_rubric@1.0` (FM/SEC/COND/AUTH/QA/SAFE/STALE families). Mirrors prd-audit's advisory-leaning approach (most rules `warning`; structural rules `error`). Sibling `*.audit.md` per SRS. Halts on needs_human; resumable on audited_srs_sha256."
skill_version: 0.1.0
persona: cuo
owner_role: cto

allowed_brain_scopes:
  read:
    - project:*
    - module:*
    - company:locked-decisions
    - company:values
    - memories:projects
    - memories:decisions
  write:
    - project:*
allowed_mcp_tools:
  - kb.read
  - brain.search
  - audit.append
escalation:
  to_persona_on_legal: cuo-clo
  to_persona_on_security: cuo-cseco
  to_persona_on_compliance: cuo-clo
  to_human_on_irreversible: true

invocation_modes: [standalone, chained]

expects:
  schema_ref: ./envelopes/srs-audit.input.json
  required_fields: [srs_paths]
  optional_fields:
    - rubric_version
    - upstream_context
    - trace_id
    - caller_persona
    - max_iterations_per_srs
  standalone_interview_ref: ./STANDALONE_INTERVIEW.md
produces:
  schema_ref: ./envelopes/srs-audit.output.json
  output_kind: artefact
  human_summary_ref: ./HUMAN_SUMMARY.md

depends_on_contracts:
  - id:        srs
    version:   v1
    purpose:   validation_target
    pin_path:  cyberos/docs/contracts/srs/
  - id:        nats-subjects
    version:   v1
    purpose:   wire_protocol_emission
    pin_path:  cyberos/docs/contracts/nats-subjects/

exposable_as:
  internal:           true
  agent_plugin:       true
  mcp_tool:           false
  partner_connector:  false

audit:
  emit_to: genie.action_log
  row_kind: artefact_write
  payload_hash_field: audited_srs_sha256
  explanation_pane: required

confidence_band:
  default: 0.75
  defer_below: 0.5
  cite_sources: required

untrusted_inputs:
  wrap_in: <untrusted_content/>
  injection_scan: required
  on_marker_hit: surface_to_human

self_audit:
  invariants_ref: ./INVARIANTS.md
  check_at: [on_node_boundary, {on_audit_row_count: 25}, on_completion]
  anomaly_signals:
    confidence_low_streak:     {threshold: 3, window: 10}
    user_correction_streak:    {threshold: 2, window: 5}
    needs_human_rate_above:    {threshold: 0.5, window: 10}
  on_breach:
    emit: refinement_proposal
    pause_pipeline: true

human_fine_tune:
  fine_tuner_role: cto
  review_required:
    on_minor_bump:    false
    on_major_bump:    true
    on_safety_change: true
    on_rubric_rule_added: true
    on_rubric_rule_removed: true
  signals_to_initiate:
    - acceptance_rate_below: 0.6
    - hitl_pause_rate_above:  0.4
  procedure_ref: null
  required_artifacts:
    - changelog_entry
    - acceptance_test_added
    - rubric_rule_diff
    - memory_refinement_entry

determinism:
  reproducible: true
  fixity_notes: "Mechanical-rule verdicts are byte-stable; LLM-judgement rules are band-reproducible. Mirrors prd-audit's split."

emitted_source_freshness_tier: 18
gated_until_phase: runtime_v0_3_0
---

# `srs-audit` — quality gate on SRSs

> **Scaffold-only at v0.1.0.** Mirrors `cuo/cpo/prd-audit`'s structure with SRS-specific rules.

## Pipeline position

```
cuo/cto/srs-author → srs@1 (status: in_review)
    ↓
cuo/cto/srs-audit  (THIS SKILL)
    ↓
audited srs@1 (status: approved IF pass)
    ↓
(future) feeds tech-spec authoring
```

## Scaffold contains

```
cuo/cto/srs-audit/
├── SKILL.md
├── CHANGELOG.md
├── RUBRIC.md             # srs_rubric@1.0 — see this file's RUBRIC.md
├── INVARIANTS.md
├── envelopes/{input,output}.json
└── acceptance/README.md
```

## RUBRIC.md summary (full file)

`srs_rubric@1.0` — 5 rule families:

- **FM-001..116** — frontmatter (template, srs_status, prd_ref resolution, dates, etc.). All `error`.
- **SEC-001..010** — required H2 sections (Background through Open Architectural Questions). All `error`.
- **COND-001..003** — conditional sections (AI Subsystem Spec when high-risk; Compliance Implementation when regulated; Review Record when reviewed). All `error`.
- **AUTH-001..004** — authority markers (parallel to prd-audit's). AUTH-001 + AUTH-002 `error → needs_human`; AUTH-003/004 `warning`.
- **QA-001..010** — judgement rules (NFR measurability, API surface coverage, telemetry budget realism, security review trigger detection, etc.). Mostly `warning`.
- **SAFE-001..004** — untrusted-content (mirrors prd-audit's).
- **STALE-001** — cross-skill staleness (when chained from srs-author).

## Citations

- Pattern source — `cuo/cpo/prd-audit/SKILL.md` + `cuo/cpo/prd-audit/RUBRIC.md`.
- `srs@1` (target contract).
