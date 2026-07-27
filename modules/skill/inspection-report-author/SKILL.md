---
# ── Identity ─────────────────────────────────────────────────────────
name: inspection-report-author
description: >-
  Author a full read-only project inspection (inspection-report@1) across 75
  engineering disciplines (spec ≥1.2; 69 rows remain valid for INSPECT-SPEC 1.0
  goldens via version gating), producing a 22-section report with a coverage
  ledger, evidence-graded findings, root-cause clusters, and exactly one named
  next action for /harden. Use when the user asks to "inspect this repository",
  "audit this project", "review this codebase end to end", or "/inspect". Do NOT
  use for "audit an existing inspection report" (use inspection-report-audit),
  and do NOT remediate — this skill never writes to the target. Distinct from
  ship-tasks "harden a task" (class:improvement).
license: Apache-2.0
metadata:
  version: 1.2.0
  module: skill
  stage: inspect
  cyberos-template: inspection-report@1
  cyberos-rubric-target: inspection_report_rubric@1.0

# ── Scope contract (memory/AGENTS.md §15) ────────────────────────────
allowed_memory_scopes:
  read:
    - project:*
    - module:*
    - company:locked-decisions
    - memories:decisions
    - memories:projects
    - memories:refinements
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

# ── Invocation modes ─────────────────────────────────────────────────
invocation_modes: [standalone, chained]

# ── Pipeline interface ───────────────────────────────────────────────
expects:
  schema_ref: ./envelopes/input.json
  required_fields:
    - target
  optional_fields:
    - ref
    - deployment_platform
    - taxonomy_override
    - depth
    - caller_persona
    - trace_id
    - chain_to
  standalone_interview_ref: null
produces:
  schema_ref: ./envelopes/output.json
  output_kind: artefact
  human_summary_ref: ./HUMAN_SUMMARY.md

# ── Contract dependencies ────────────────────────────────────────────
depends_on_contracts:
  - id:        nats-subjects
    version:   v1
    purpose:   wire_protocol_emission
    pin_path:  cyberos/skill/contracts/nats-subjects/

# ── Exposability ─────────────────────────────────────────────────────
exposable_as:
  internal:           true
  agent_plugin:       true
  mcp_tool:           false
  partner_connector:  false

# ── Audit hook ───────────────────────────────────────────────────────
audit:
  emit_to: genie.action_log
  row_kind: artefact_write
  payload_hash_field: artefact_hash
  explanation_pane: required

# ── Trust calibration ────────────────────────────────────────────────
confidence_band:
  default: 0.7
  defer_below: 0.5
  cite_sources: required

# ── Untrusted-content discipline ─────────────────────────────────────
untrusted_inputs:
  wrap_in_marker: "untrusted_content"
  injection_scan: required
  on_marker_hit: surface_to_human

# ── Self-audit ───────────────────────────────────────────────────────
self_audit:
  invariants_ref: ./INVARIANTS.md
  check_at:
    - on_node_boundary
    - on_completion
  anomaly_signals:
    confidence_low_streak:     {threshold: 3, window: 10}
    citation_missing_streak:   {threshold: 2, window: 10}
  on_breach:
    emit: refinement_proposal
    pause_pipeline: true

# ── Determinism ──────────────────────────────────────────────────────
determinism:
  reproducible: false
  fixity_notes: "Authoring is judgement; machine floor (inspect-lint) is deterministic."

# ── Chain ────────────────────────────────────────────────────────────
chains_to: inspection-report-audit
next_skill_recommendation: inspection-report-audit
emitted_source_freshness_tier: 30
gated_until_phase: null
untrusted_content_wrapping: required
---

# inspection-report-author

## Purpose

Produce one `inspection-report@1` for a target repository: a read-only,
evidence-graded assessment across the engineering-discipline taxonomy, in a
form a separate `/harden` run can consume without a human re-reading it.

The skill never remediates, never self-approves, and never invokes `/harden`.
Its output is a backlog and a verdict, not a change. `/harden` (inspection
remediation) is distinct from ship-tasks `class: improvement` ("harden a task").

## Inputs

See `envelopes/input.json`. At minimum a target path or clone URL. Optional:
taxonomy override, depth hint, and declared deployment platform.

## Contract

Canonical rules live in `references/inspect-prompt.md` (INS-* families). Spec
≥1.2 uses a **75-discipline** ledger. Spec 1.0 reports remain valid under
version-gated lint (`INSPECT-SPEC` line) with 69 rows.

Non-negotiable for `inspection-report-audit`:

1. Every discipline gets a ledger row (75 for ≥1.2; 69 for 1.0) with
   applicability, evidence state, finding count, and pointer.
2. Every VERIFIED finding carries a verbatim quote (INS-GATE-VQ).
3. Exactly one `NEXT-ACTION: <finding id> <fingerprint>`.

## Procedure

Phases 0–9 per the reference, then the report, then the nine-gate self-audit.
Failed gate means keep working, not ship. Declare `INSPECT-SPEC` on the report.

## Halts

Halt when deployment platform changes applicability and is unstated; when a
credential-shaped value is found (report immediately); or when evidence would
require a side effect on the target.

## Anti-scope

Does not write to the target, install dependencies, run target code, test
credentials, open PRs, or set task statuses.
