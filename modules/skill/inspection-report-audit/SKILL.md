---
# ── Identity ─────────────────────────────────────────────────────────
name: inspection-report-audit
description: >-
  Audit an existing inspection-report@1 against inspection_report_rubric@1.0.
  Runs tools/inspect-lint.mjs as a machine floor, then scores judgement rules a
  linter cannot check. Emits a score out of 10 and refuses to pass below 10/10.
  Use when the user asks to "audit this inspection report", "check the
  inspection against the rubric", or "is this report trustworthy". Do NOT use
  for "inspect this repository" (use inspection-report-author). Pass means the
  report may be handed to /harden — not to ship-tasks.
license: Apache-2.0
metadata:
  version: 1.2.0
  module: skill
  stage: inspect
  cyberos-template: inspection-report@1
  cyberos-rubric-version: inspection_report_rubric@1.0

# ── Scope contract (memory/AGENTS.md §15) ────────────────────────────
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

# ── Invocation modes ─────────────────────────────────────────────────
invocation_modes: [standalone, chained]

# ── Pipeline interface ───────────────────────────────────────────────
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
  fixity_notes: "Machine floor is byte-deterministic; judgement scoring cites rule ids."

chains_from: inspection-report-author
next_skill_recommendation: harden-record-author
emitted_source_freshness_tier: 30
untrusted_content_wrapping: required
---

# inspection-report-audit

## Purpose

Decide whether one `inspection-report@1` may be handed to `/harden`. Two layers
must both pass.

## Layer 1: machine floor

```bash
node tools/inspect-lint.mjs <report.md>
```

Exit 0 required. Run `--selftest` (28 cases) after modifying the linter.
Version-gates on `INSPECT-SPEC` so 1.0 goldens (69 rows) and 1.2 goldens
(75 rows) both stay valid.

## Layer 2: judgement

See `RUBRIC.md` (IRA-001..IRA-009). Score out of 10; below 10 route back to
`inspection-report-author` citing failing rule ids verbatim.

## Acceptance fixtures

`acceptance/` holds twenty reports: ten spec 1.0 and ten spec 1.2 (`.r2`) on the
same targets. Keeping both is deliberate evidence of what the amendment changed.

## Feedback loop

`tools/inspect-taxonomy-stats.mjs` aggregates ledgers. Taxonomy for new reports
is **75** rows at spec 1.2; do not prune below the sample threshold. Spec 1.0
fixtures keep 69 rows under version gating.
