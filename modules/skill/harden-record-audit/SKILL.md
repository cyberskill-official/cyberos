---
# ── Identity ─────────────────────────────────────────────────────────
name: harden-record-audit
description: >-
  Audit a hardening-record@1 for scope discipline, verification honesty, and
  gate integrity. Checks every changed file was inside a worked finding's
  declared scope, every closed finding carries verbatim validation output and a
  regression gate, fingerprints were carried unchanged, and no finding closed on
  operator silence. Use when asked to "audit this hardening session", "check
  what /harden actually did", or "verify the remediation record". Do NOT use to
  work findings (harden-record-author) or to ship class:improvement tasks
  (ship-tasks).
license: Apache-2.0
metadata:
  version: 1.0.0
  module: skill
  stage: harden
  cyberos-template: hardening-record@1
  cyberos-rubric-version: hardening_record_rubric@1.0

allowed_memory_scopes:
  read:
    - project:*
    - module:*
    - company:locked-decisions
  write:
    - project:*
allowed_mcp_tools:
  - kb.read
  - memory.search
  - audit.append
escalation:
  to_persona_on_legal: cuo-clo
  to_persona_on_security: cuo-cseco
  to_persona_on_compliance: cuo-clo
  to_human_on_irreversible: true

invocation_modes: [standalone, chained]

expects:
  schema_ref: ./envelopes/input.json
  required_fields:
    - artefact_paths
  optional_fields:
    - rubric_version
    - upstream_context
    - trace_id
    - caller_persona
    - max_iterations_per_artefact
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
  payload_hash_field: audited_file_sha256
  explanation_pane: required

confidence_band:
  default: 0.95
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
  reproducible: true
  fixity_notes: "Judgement cites HRA-* rule ids; no partial acceptance on scope breach."

chains_from: harden-record-author
next_skill_recommendation: null
emitted_source_freshness_tier: 30
untrusted_content_wrapping: required
---

# harden-record-audit

## Purpose

Decide whether a hardening session may be accepted. The record claims work was
done correctly; this checks the claim against its own evidence.

## What it checks

See `RUBRIC.md` (HRA-001..HRA-008): scope (HRD-SCOPE), verification (HRD-VER),
fingerprint identity (HRD-STATE-2), human gates (HRD-HITL), not-worked honesty,
safety envelope, actor classification fidelity (including
`operator_prerequisites`), and record completeness.

## Verdict

Pass, or route back to `harden-record-author` naming failing rule ids. No
partial acceptance: one scope breach fails the session.

## Relationship to re-inspection

This audit checks the session. It does not check whether the defect is gone.
That is the next `/inspect`, which reconciles by fingerprint.
