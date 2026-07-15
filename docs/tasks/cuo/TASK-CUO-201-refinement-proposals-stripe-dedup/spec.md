---
template: task@1
id: TASK-CUO-201
title: "Harness Wave 2 — refinement-proposal emitter with stripe-based dedup"
type: feature
author: "@stephen"
department: engineering
status: done
priority: p1
created_at: 2026-05-19T20:15:00+07:00
ai_authorship: assisted
eu_ai_act_risk_class: minimal
target_release: 2026-Q3
client_visible: false
module: cuo
new_files:
  - modules/cuo/cuo/core/refinement_proposal.py
  - modules/cuo/cuo/core/stripe.py
  - modules/cuo/tests/test_refinement_proposal.py
  - docs/proposals/open/.gitkeep
  - docs/proposals/applied/.gitkeep
  - docs/proposals/rejected/.gitkeep
  - docs/proposals/INDEX.md
depends_on: [TASK-CUO-200]
blocks: [TASK-CUO-202]
---

## Summary

When a skill's `self_audit.on_breach.emit == "refinement_proposal"` policy fires (via the TASK-CUO-200 harness), this task's emitter:

1. Computes a **stripe** for the proposal — a categorical key derived from `(skill_name, signal_id, normalised_evidence_pattern)`.
2. Checks `docs/proposals/open/` for any unresolved proposal with the same stripe.
3. **If a matching open stripe exists** → write a `STRIPE_REPEAT` marker, **halt the chain with HITL_HALT**, surface the existing proposal to the operator immediately. (Stephen's "don't waste time on rework" rule.)
4. **If no matching open stripe exists** → write a new `refinement_proposal@1` artefact under `docs/proposals/open/<stripe>-<timestamp>.md`, continue the chain.

This means: the first time a class of issue surfaces, the workflow keeps moving and queues the proposal for review. The second time the SAME class surfaces while the prior is still pending, the workflow stops — because forging ahead is wasteful when the root cause is already known.

## Problem

Without stripe-based dedup, the harness has two pathological modes:

- **Spam mode**: every breach emits a fresh proposal, and the proposals queue grows unbounded. Stephen spends more time clearing the queue than the proposals would save him.
- **Silent-suffering mode**: the harness emits one proposal then suppresses further emissions of the same kind to avoid spam — but then doesn't tell Stephen the issue is recurring, so he doesn't know to prioritise the existing proposal.

Stripe-based dedup with halt-on-repeat threads the needle: dedup is automatic (one proposal per category at a time), and the harness escalates by halting when the issue recurs — making the resolution priority self-evident from the halt itself.

## Proposed Solution

### §1 Normative requirements

1. **MUST** ship `cuo/core/stripe.py` with a function `compute_stripe(skill_name, signal_id, evidence_rows) -> str` that returns a deterministic stripe id with format `<skill_slug>:<signal_id>:<pattern_hash>`.
2. **MUST** derive `pattern_hash` from the evidence rows using a normalised projection (e.g. for `confidence_low_streak`: the set of rule_ids that breached, sorted; for `user_correction_streak`: the set of audit_kinds the corrections touched, sorted). The hash is SHA-256 of the canonical-JSON projection, truncated to 8 hex chars.
3. **MUST** ship `cuo/core/refinement_proposal.py` with `emit_or_halt(skill_name, signal_id, evidence_rows, proposals_root) -> EmissionResult` that returns one of: `Emitted(stripe_id, proposal_path)`, `StripeRepeatHalt(stripe_id, existing_proposal_path)`, or `Suppressed(reason)`.
4. **MUST** check `<proposals_root>/open/` for any file matching `<stripe_id>-*.md` BEFORE writing. Match by stripe_id prefix only (the timestamp suffix varies).
5. **MUST** write new proposals to `<proposals_root>/open/<stripe_id>-<YYYYMMDDTHHMMSSZ>.md` with frontmatter `template: refinement_proposal@1`, body sections: `## Stripe`, `## Triggering signal`, `## Evidence rows` (table of audit row IDs + summaries), `## Suggested change` (LLM-generated diff outline), `## Risk class`.
6. **MUST** emit `cuo.refinement_proposal_emitted` memory aux row for each new proposal; payload `{stripe_id, skill_name, signal_id, evidence_row_ids, proposal_path}`. *(traces_to: §1 #6 → AC #1)*
7. **MUST** emit `cuo.stripe_repeat_halt` memory aux row when a repeat fires; payload `{stripe_id, existing_proposal_path, new_evidence_row_ids, halted_workflow_id}`. The supervisor MUST surface this as outcome `HITL_HALT` (interop with TASK-CUO-200's drain command). *(traces_to: §1 #7 → AC #2, #10)*
8. **MUST** support operator workflow:
   - `cyberos-cuo proposal list` — open / applied / rejected lists with stripes
   - `cyberos-cuo proposal show <stripe_id>` — open the markdown
   - `cyberos-cuo proposal apply <stripe_id>` — moves file to `applied/`, optionally runs the diff (Wave 3 — TASK-CUO-202)
   - `cyberos-cuo proposal reject <stripe_id> --reason "<text>"` — moves file to `rejected/<stripe_id>-<ts>.md` with `## Rejection rationale` appended
9. **MUST** treat `<proposals_root>/{applied,rejected}/` as "resolved": stripe-dedup only checks `open/`, so a previously-applied stripe can naturally re-fire if the issue recurs after the fix.
10. **MUST** treat the proposal output `## Suggested change` as informational only — Wave 3 (TASK-CUO-202) decides which proposals auto-apply vs require HITL. Wave 2 (this task) never mutates a skill/RUBRIC/contract automatically. *(traces_to: §1 #10 → AC #5)*

### §2 Stripe taxonomy (initial set)

Stripes are open-ended (`compute_stripe` derives them deterministically) but the common ones expected to emerge:

| Stripe pattern | Means |
|---|---|
| `task-audit:TRACE-001:<hash>` | Same set of §1 clauses keep failing TRACE-001 (no §4 AC) across multiple tasks |
| `coverage-gate-audit:tests_failed:<hash>` | Same test names keep failing across multiple task runs |
| `ship-tasks:routed_back:<hash>` | Same task keeps routing back at the same phase |
| `<skill>:needs_human_rate_above:<hash>` | Skill is asking for HITL too often on the same evidence pattern |
| `<skill>:deterministic_drift:<hash>` | Same skill's output is drifting between runs in the same way |

## Alternatives Considered

1. **No dedup, emit every proposal** — spam mode (rejected above).
2. **Suppress all repeats silently** — silent-suffering mode (rejected above).
3. **Rate-limit (max N proposals per skill per day)** — coarser than stripes; can hide diversity of issues OR block legitimate distinct proposals. Stripe-dedup is sharper.
4. **Auto-merge similar proposals into one super-proposal** — clever but complex; defer.

## Success Metrics

| metric | baseline | target | deadline |
|---|---|---|---|
| Median proposals in `open/` at any time | n/a | < 8 | steady state |
| % halts on stripe repeat that surface a real recurring root cause | n/a | ≥ 80% (operator rating) | first 10 halts |
| Stripe hash collision rate (false dedup) | 0 | 0 | continuous |

## Scope

In scope: stripe computation, proposal authoring, halt-on-repeat logic, operator CLI for list/show/apply/reject. The `## Suggested change` body section is LLM-generated; the harness invokes the failing skill itself with a self-reflection prompt ("you tripped this signal — propose a SKILL.md / RUBRIC.md diff").

### Out of scope

- Actually applying the diff (Wave 3 — TASK-CUO-202)
- Workflow chain edits (Wave 4 — TASK-CUO-203)
- Cross-tenant proposal aggregation

## Dependencies

- **TASK-CUO-200** — harness emits signals + report; this task consumes them.

## AI Authorship Disclosure

- **Tools used:** Anthropic Claude.
- **Scope:** §1 normative clauses (10 items), §4 ACs (10 items), §5 named test entries, stripe taxonomy table, alternatives section.
- **Human review:** Stephen Cheng reviewed before audit; the "halt-on-repeat-stripe" rule was explicitly architected by operator in chat (not LLM-suggested).

## §4 Acceptance Criteria

1. First occurrence of a tripped signal produces exactly one file under `docs/proposals/open/<stripe>-<ts>.md`. *(traces_to: §1 #5)*
2. Second occurrence of the SAME stripe within the same `open/` folder produces NO new file but emits `cuo.stripe_repeat_halt` aux row + sets workflow outcome to `HITL_HALT`. *(traces_to: §1 #4, #7)*
3. Moving the open proposal to `applied/` (via `cyberos-cuo proposal apply`) reopens the stripe — a subsequent occurrence writes a new proposal cleanly. *(traces_to: §1 #9)*
4. `compute_stripe` is deterministic — same inputs → same stripe id across processes / sessions. *(traces_to: §1 #1, #2)*
5. The proposal body's `## Stripe` section names the stripe verbatim; `## Evidence rows` table lists every contributing audit row id with a 1-line summary; `## Suggested change` contains a non-empty diff outline (LLM-generated; may be `[mock-llm placeholder]` in test mode). *(traces_to: §1 #5)*
6. `cyberos-cuo proposal list` enumerates `open/` + `applied/` + `rejected/` with stripe ids + ages; the count totals match the on-disk file counts. *(traces_to: §1 #8)*
7. `cyberos-cuo proposal reject <id> --reason "X"` appends a `## Rejection rationale` section to the file before moving it. *(traces_to: §1 #8)*
8. Running the emitter against an audit chain with zero qualifying signals produces zero files + zero halt rows. *(traces_to: §1 #3)*
9. Stripe pattern_hash is exactly 8 hex chars; collision between two genuinely-different evidence patterns has acceptance rate ≤ 1 in 10⁹ (SHA-256 birthday-bound at 8 hex chars ≈ 2⁻³²). *(traces_to: §1 #2)*
10. The supervisor's `HITL_HALT` outcome from this task is honoured by `cyberos-cuo drain` (writes DRAIN_HALT.md, exits non-zero). *(traces_to: §1 #7, TASK-CUO-200 §1 #5 indirectly via drain)*

## §5 Verification

- `modules/cuo/tests/test_refinement_proposal.py::test_first_occurrence_writes_proposal` (AC #1)
- `modules/cuo/tests/test_refinement_proposal.py::test_repeat_stripe_halts_no_new_file` (AC #2, #7)
- `modules/cuo/tests/test_refinement_proposal.py::test_applied_proposal_reopens_stripe` (AC #3, #9)
- `modules/cuo/tests/test_refinement_proposal.py::test_stripe_determinism` (AC #4)
- `modules/cuo/tests/test_refinement_proposal.py::test_proposal_body_shape` (AC #5)
- `modules/cuo/tests/test_refinement_proposal.py::test_cli_list_show_apply_reject` (AC #6, #7)
- `modules/cuo/tests/test_refinement_proposal.py::test_empty_chain_clean_exit` (AC #8)
- `modules/cuo/tests/test_refinement_proposal.py::test_stripe_hash_width` (AC #9)
- `modules/cuo/tests/test_refinement_proposal.py::test_drain_honours_hitl_halt` (AC #10)
