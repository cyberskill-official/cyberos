---
# ── Identity ─────────────────────────────────────────────────────────
name: harden-record-author
description: >-
  Work findings from an inspection-report@1 under scope and safety discipline
  and emit a hardening-record@1. Validates input with inspect-lint before reading
  findings, classifies each finding as agent/operator/split (honouring
  operator_prerequisites), halts at plan and review gates, and never pushes,
  merges, rotates credentials, or executes irreversible actions. Use for
  "/harden", "work the inspection backlog", or "remediate what /inspect found".
  Do NOT use to discover findings (inspection-report-author), judge report
  quality (inspection-report-audit), or ship a backlog class:improvement task
  (ship-tasks — distinct "harden a task" path).
license: Apache-2.0
metadata:
  version: 1.0.0
  module: skill
  stage: harden
  cyberos-template: hardening-record@1
  cyberos-rubric-target: hardening_record_rubric@1.0

allowed_memory_scopes:
  read:
    - project:*
    - module:*
    - company:locked-decisions
    - memories:decisions
    - memories:projects
  write:
    - project:*
    - memories:projects
allowed_mcp_tools:
  - kb.read
  - kb.search
  - memory.search
  - memory.write_memory
  - audit.append
  - chat.notify
escalation:
  to_persona_on_legal: cuo-clo
  to_persona_on_security: cuo-cseco
  to_persona_on_compliance: cuo-clo
  to_human_on_irreversible: true

invocation_modes: [standalone, chained]

expects:
  schema_ref: ./envelopes/input.json
  required_fields:
    - report_path
  optional_fields:
    - finding_id
    - want_commits
    - caller_persona
    - trace_id
    - chain_to
produces:
  schema_ref: ./envelopes/output.json
  output_kind: artefact
  human_summary_ref: ./HUMAN_SUMMARY.md

depends_on_contracts:
  - id:        nats-subjects
    version:   v1
    purpose:   wire_protocol_emission
    pin_path:  cyberos/skill/contracts/nats-subjects/

exposable_as:
  internal:           true
  agent_plugin:       true
  mcp_tool:           false
  partner_connector:  false

audit:
  emit_to: genie.action_log
  row_kind: artefact_write
  payload_hash_field: artefact_hash
  explanation_pane: required

confidence_band:
  default: 0.7
  defer_below: 0.5
  cite_sources: required

untrusted_inputs:
  wrap_in_marker: "untrusted_content"
  injection_scan: required
  on_marker_hit: surface_to_human

self_audit:
  invariants_ref: ./INVARIANTS.md
  check_at:
    - on_node_boundary
    - on_completion

determinism:
  reproducible: false
  fixity_notes: "Planner (harden-plan.mjs) is deterministic; remediation judgement is not."

chains_from: inspection-report-audit
chains_to: harden-record-audit
next_skill_recommendation: harden-record-audit
emitted_source_freshness_tier: 30
untrusted_content_wrapping: required
---

# harden-record-author

## Purpose

Close findings from an `inspection-report@1` without exceeding each finding's
declared scope, and leave a `hardening-record@1` a re-inspection can reconcile
by fingerprint.

Never discovers findings, never re-inspects, never decides a finding is not
worth fixing. Distinct from ship-tasks "harden a task" (`class: improvement`).

## Contract

Rules in `references/harden-prompt.md` (HRD-* families). Non-negotiable:

1. Linter exits 0 before any finding is read (HRD-IN-1).
2. Only declared-scope files change, plus named collateral (HRD-SCOPE).
3. Nothing irreversible; no credential rotation; no push/merge/deploy (HRD-SAFE).
4. Both human gates reached; silence is not approval (HRD-HITL).

## Procedure

Validate → resolve next action → `tools/harden-plan.mjs` → plan HITL → work one
finding per cycle → review HITL → emit record → chain to `harden-record-audit`.

## Actor classification

`tools/harden-plan.mjs` classifies `agent` / `operator` / `split` from
remediation, rollback, **and** `operator_prerequisites`. A non-`none`
prerequisite must not classify as pure `agent` (shopass INS-F-0002 regression).

## Anti-scope

No out-of-scope writes, no PR open, no task status mutation, no live credential
tests.
