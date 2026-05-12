---
# ── Identity ─────────────────────────────────────────────────────────
name: prd-audit
description: "Quality gate on PRDs. Audits one or more `prd@1` markdowns against `prd_rubric@1.0` (FM/SEC/COND/QA/SAFE/AUTH/STALE rule families). Produces a sibling `*.audit.md` per PRD with verdict pass | needs_human | fail | stale. Advisory-leaning by design (per Q4 of registry v0.2.4 design): most rules are warning, only structural-correctness rules are error. Halts on needs_human verdicts; resumable on audited_prd_sha256. Standalone trigger or chains naturally after prd-author."
skill_version: 0.1.0
persona: cuo
owner_role: cpo

# ── Scope contract (SRS §6.4) ────────────────────────────────────────
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
  schema_ref: ./envelopes/prd-audit.input.json
  required_fields: [prd_paths]
  optional_fields:
    - rubric_version              # default prd_rubric@1.0
    - upstream_context            # populated when chained from prd-author
    - trace_id
    - caller_persona              # default cuo-cpo
    - max_iterations_per_prd      # default 5 (PRDs need fewer iterations than FRs — judgement-heavy, less mechanical)
  standalone_interview_ref: ./STANDALONE_INTERVIEW.md
produces:
  schema_ref: ./envelopes/prd-audit.output.json
  output_kind: artefact
  human_summary_ref: ./HUMAN_SUMMARY.md

depends_on_contracts:
  - id:        prd
    version:   v1
    purpose:   validation_target
    pin_path:  cyberos/docs/contracts/prd/
  - id:        nats-subjects
    version:   v1
    purpose:   wire_protocol_emission
    pin_path:  cyberos/docs/contracts/nats-subjects/

exposable_as:
  internal:           true
  agent_plugin:       true
  mcp_tool:           false       # gated until rubric stabilises (PRD-audit is judgement-heavier than fr-audit; needs more user feedback before tool-surface promotion)
  partner_connector:  false

audit:
  emit_to: genie.action_log
  row_kind: artefact_write
  payload_hash_field: audited_prd_sha256
  explanation_pane: required

confidence_band:
  default: 0.75                   # mid-high; most rules are mechanical-with-judgement-fallback
  defer_below: 0.5
  cite_sources: required

untrusted_inputs:
  wrap_in: <untrusted_content/>
  injection_scan: required
  on_marker_hit: surface_to_human

self_audit:
  invariants_ref: ./INVARIANTS.md
  check_at:
    - on_node_boundary
    - on_audit_row_count: 25
    - on_completion
  anomaly_signals:
    confidence_low_streak:     {threshold: 3, window: 10}
    user_correction_streak:    {threshold: 2, window: 5}
    needs_human_rate_above:    {threshold: 0.5, window: 10}
    rubric_drift_within_batch: {threshold: 1, window: 1}
  on_breach:
    emit: refinement_proposal
    pause_pipeline: true

human_fine_tune:
  fine_tuner_role: cpo
  review_required:
    on_minor_bump:    false
    on_major_bump:    true
    on_safety_change: true
    on_rubric_rule_added: true
    on_rubric_rule_removed: true
  signals_to_initiate:
    - acceptance_rate_below: 0.6
    - hitl_pause_rate_above:  0.4
    - regulator_inquiry_received
  procedure_ref: null
  required_artifacts:
    - changelog_entry
    - acceptance_test_added
    - rubric_rule_diff
    - memory_refinement_entry

determinism:
  reproducible: true              # mechanical rules are byte-stable; LLM-judgement rules report band rather than exact float
  fixity_notes: "Audit reports are byte-stable for a given PRD + rubric_version modulo last_audit_at and the LLM-judgement rules' confidence floats (see RUBRIC.md §15.10 Confidence-band reporting)."

emitted_source_freshness_tier: 18
gated_until_phase: runtime_v0_3_0
untrusted_content_wrapping: required
---

# `prd-audit` — quality gate on PRDs

> **Scaffold-only at v0.1.0.** Documents the intended contract; runtime in v0.3.0.

## What this skill does (when running)

Audits one or more `prd@1` markdowns against `prd_rubric@1.0` (defined in `RUBRIC.md`). For each PRD: locate → hash → load-or-init audit → run rubric → attempt fixes for auto-fixable rules → re-audit → terminate with verdict {pass | needs_human | fail | stale} → write audit report. Sibling output: `<prd-name>.audit.md`.

Mirrors fr-audit's structure but with three deliberate differences (per Q4 of registry v0.2.4 design — PRDs are judgement-heavy):

1. **Most rules are `warning`-severity, not `error`.** The rubric flags issues but doesn't block. PRD authors negotiate with the auditor; reviewers can accept warnings.
2. **`max_iterations_per_prd` defaults to 5, not 10.** PRDs reach a stable state faster than FRs because most "fixes" require human input, not auto-application.
3. **Authority-marker checking is the central new rule family** (AUTH-001..004) — fr-audit has no equivalent because FRs don't carry per-claim authority.

## Pipeline position

```
cuo/cpo/prd-author → prd@1 (status: in_review)
    ↓
cuo/cpo/prd-audit   (THIS SKILL)
    ↓
audited prd@1 (status: approved IF pass; flagged for human review IF needs_human or fail)
    ↓
cuo/cpo/fr-author   (consumes audited PRD; v0.3.0+ migration)
```

## Self-test preamble (when implemented)

```
CONTRACT_ECHO
skill_id:                        cuo/cpo/prd-audit
skill_version:                   0.1.0
prompt_revision:                 prd_audit@0.1.0
template_version:                prd@1   (from cyberos/docs/contracts/prd/template.md)
audit_rubric_version:            prd_rubric@1.0
audit_path_pattern:              <prd_path with extension replaced by ".audit.md">
hitl_categories:                 [authority_marker_missing, authority_too_weak, ai_act_classification_drift,
                                  approval_record_incomplete, vague_success_metric, unverifiable_research_signal,
                                  superseded_chain_broken, confidentiality_loosening_attempt]
hitl_policy:                     HALT_BATCH_ON_NEEDS_HUMAN
phase:                           <PHASE_1_LOCATE | PHASE_2_RUN_RUBRIC | PHASE_3_FIX | PHASE_4_REAUDIT | PHASE_5_TERMINATE | PHASE_6_WRITE | RESUME>
```

## Scaffold contains

```
cuo/cpo/prd-audit/
├── SKILL.md
├── CHANGELOG.md
├── RUBRIC.md             # prd_rubric@1.0
├── INVARIANTS.md         # 6 self-audit invariants
├── AUDIT_LOOP.md         # 8-step loop algorithm (mirrors fr-audit's)
├── REPORT_FORMAT.md      # *.audit.md format spec
├── STANDALONE_INTERVIEW.md
├── HUMAN_SUMMARY.md
├── envelopes/
│   ├── prd-audit.input.json
│   └── prd-audit.output.json
└── acceptance/
    └── README.md
```

## Failure modes (when implemented)

Mirror fr-audit's BOOT-001..008. Plus:

- **BOOT-009** — PRD has `prd_status: superseded` AND no `superseded_by` pointer. Refuse audit; surface.

## Citations

- Pattern source — `cuo/cpo/fr-audit/SKILL.md` (the structural mirror).
- Q4 of registry v0.2.4 — PRDs are judgement-heavy; advisory-leaning rubric.
- `prd@1` (target contract) → `cyberos/docs/contracts/prd/CONTRACT.md`.
- AGENTS.md §5.3 — authority hierarchy enforced by AUTH-001..004 rules.

## Anti-fabrication discipline (mandatory)

This skill operates under strict anti-fabrication rules per `references/ANTI_FABRICATION.md`:

- **Source-grounded claims only.** Every claim traces back to a line in the source spec, a BRAIN memory_id, or a documented inference. No floating claims.
- **Authority markers required.** Every paragraph carries an `authority` field — `human-edited`, `human-confirmed`, `llm-explicit`, or `llm-implicit` per AGENTS.md §5.1. Use `cyberos authoring attribute <body> <source>` to assign automatically. Every emitted memory carries a `source_ref:` pointing back at the source line that justified it.
- **HITL on ambiguity.** The skill pauses with `needs_human: true` rather than guessing.
- **Untrusted-content wrapping.** Quotes of operator-supplied text are wrapped in `<untrusted_content source="...">...</untrusted_content>` blocks per AGENTS.md §4.2. This skill's frontmatter declares `untrusted_content_wrapping: required`.
- **No fabricated cross-references or metrics.** Identifiers must resolve; estimates must cite a source.

See `references/ANTI_FABRICATION.md` for the full ruleset.

## Source attribution

Every emitted artefact carries:

- A `source_ref` field pointing at the line(s) in the source spec that justified its existence
- Authority marker per claim (`authority: human-confirmed | llm-explicit | llm-implicit`)
- A `provenance:` block on the FR-level frontmatter declaring the source path + content SHA256 at read time

This satisfies AGENTS.md §5.1 (authority hierarchy) and §9.1 (source-tier ordering) requirements.

