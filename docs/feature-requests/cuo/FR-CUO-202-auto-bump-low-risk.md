---
template: feature_request@1
id: FR-CUO-202
title: "Harness Wave 3 — auto-bump applier for low-risk refinement proposals"
author: "@stephen"
department: engineering
status: ready_to_test
priority: p2
created_at: 2026-05-19T20:30:00+07:00
ai_authorship: assisted
feature_type: internal_tooling
eu_ai_act_risk_class: limited
target_release: 2026-Q3
client_visible: false
module: cuo
new_files:
  - modules/cuo/cuo/core/proposal_applier.py
  - modules/cuo/cuo/core/version_bump.py
  - modules/cuo/tests/test_proposal_applier.py
depends_on: [FR-CUO-201]
blocks: [FR-CUO-203]
---

## Summary

Wave 3 of the continuous-improvement harness: when a `refinement_proposal@1` is approved (via `cyberos-cuo proposal apply <id>`), this FR's applier reads the proposal's `## Suggested change` diff, classifies it as **minor** / **major** / **safety** per the target skill's `human_fine_tune.review_required` policy, and either:

- **Minor + `on_minor_bump: false`** → applies the diff automatically, bumps the skill's `metadata.version` patch number, writes `memory_refinement_entry` aux row + CHANGELOG entry.
- **Major OR `on_*: true`** → queues under `<proposals_root>/pending_approval/<stripe>-<ts>.md`, requires explicit `cyberos-cuo proposal approve <id>` from Stephen before applying.

This is the first wave where the harness can mutate the codebase. Heavily gated by classification, version bumps, and audit emission.

## Problem

After FR-CUO-201, the harness queues good proposals into `docs/proposals/open/`. But every proposal requires Stephen to manually edit the skill file, bump the version, run tests, and commit. For boring proposals (typo fix in a description, wording polish on a rule rationale), this is a tax on attention with no real risk. For risky proposals (new rule_id, removal of an existing rule, contract version bump), the manual gate is essential.

Without automatic application of low-risk changes, the harness becomes a backlog rather than an active improvement loop.

## Proposed Solution

### §1 Normative requirements

1. **MUST** ship `cuo/core/proposal_applier.py` with `apply_proposal(proposal_path) -> ApplyResult` that reads the proposal, classifies the diff, and either applies-or-queues.
2. **MUST** ship `cuo/core/version_bump.py` with `bump(skill_path, level: "patch" | "minor" | "major") -> str` that rewrites the `metadata.version` field in SKILL.md frontmatter and returns the new version.
3. **MUST** classify diffs into 4 buckets:
   - `cosmetic` (description wording, comments, formatting only) → patch bump, auto-apply.
   - `rule_addition` (new SEC/QA/TRACE rule entry) → minor bump, **major** review-required class → queue.
   - `rule_removal` (deletes a rule from RUBRIC) → major bump, queue.
   - `contract_field_change` (SKILL.md `cyberos-template` or contract `template.md` field shape change) → major bump, queue.
4. **MUST** honour the target skill's `human_fine_tune.review_required` flags. Any `True` flag matching the proposal's bucket forces queue regardless of bucket default.
5. **MUST** write proposals that auto-apply to `<proposals_root>/applied/<stripe>-<ts>.md` after the apply; queued ones to `<proposals_root>/pending_approval/<stripe>-<ts>.md`.
6. **MUST** emit `cuo.proposal_applied` memory aux row per auto-apply; payload `{stripe_id, skill_path, bump_level, old_version, new_version, diff_summary}`. Emit `cuo.proposal_queued` for queued ones.
7. **MUST** add CLI subcommands:
   - `cyberos-cuo proposal apply <stripe>` — runs the applier (auto-apply OR queue depending on classification).
   - `cyberos-cuo proposal approve <stripe>` — moves a `pending_approval/` proposal to `applied/` AND runs the apply (this is the explicit HITL gate).
   - `cyberos-cuo proposal classify <stripe>` — dry-run; shows the bucket + bump level + would-be action.
8. **MUST** run the skill's acceptance/TRIGGER_TESTS.md tests against the new version BEFORE writing the applied state — if any test fails, abort + queue + emit `cuo.proposal_apply_failed`.
9. **MUST** append a `CHANGELOG.md` entry per applied proposal with the format: `### YYYY-MM-DD — [SKILL] <skill_name> v<old> → v<new>`, followed by the diff summary.
10. **MUST NOT** apply ANY proposal whose `## Risk class` body section equals `"safety"` automatically — `on_safety_change: true` is the protocol's defence-in-depth and overrides bucket classification.

### §2 Bump-level table

| diff bucket | default bump | review_required override |
|---|---|---|
| `cosmetic` (description / comment) | `patch` | none |
| `wording_polish` (rule rationale prose) | `patch` | none |
| `threshold_tune` (numeric threshold in `self_audit` block) | `minor` | `on_minor_bump: true` → queue |
| `rule_addition` | `minor` | `on_rubric_rule_added: true` → queue |
| `rule_removal` | `major` | always queue |
| `contract_field_change` | `major` | always queue |
| `safety_class` (anything marked `## Risk class: safety` in proposal body) | `major` | always queue, NEVER auto |

## Alternatives Considered

1. **All proposals require HITL** — defeats the purpose of the harness for boring fixes.
2. **All proposals auto-apply** — unacceptable risk; rule removal could destabilise audits silently.
3. **Slack-based approval inline** — out of scope; CLI-based HITL is good enough for Stephen's solo workflow today.

## Success Metrics

| metric | baseline | target | deadline |
|---|---|---|---|
| % of proposals auto-applied (vs. queued) | 0% | 30–60% | 30 days post-ship |
| % auto-applied proposals reverted by Stephen within 24h (false-positive auto-apply) | n/a | < 5% | continuous |
| Median time from proposal-created to applied (for auto-bucket) | n/a | < 5 min | continuous |

## Scope

In scope: classifier, applier, version-bumper, test-gate, CLI, audit row emission.

### Out of scope

- Workflow chain edits (Wave 4 — FR-CUO-203)
- Multi-skill atomic apply (transactional cross-file changes)
- Cross-tenant proposal propagation

## Dependencies

- **FR-CUO-201** — proposals must exist before they can be applied.

## AI Risk Assessment

### Data Sources

The applier reads two trusted sources only: the proposal markdown file itself (authored by an LLM under operator review per FR-CUO-201), and the target SKILL.md / RUBRIC.md files (operator-authored). No external network calls. No untrusted user input enters the applier.

### Human Oversight

`major` / `safety` class diffs ALWAYS queue under `pending_approval/` and never apply without an explicit `cyberos-cuo proposal approve <stripe>` command — that command IS the human oversight gate. `minor` class diffs honour each skill's per-skill `human_fine_tune.review_required` flag set; if the target skill declares `on_minor_bump: true`, the operator's review remains required even though the diff bucket would otherwise auto-apply. The pre-apply TRIGGER_TESTS gate (§1 #8) is a second oversight layer — even auto-approved diffs revert if the skill's own acceptance tests fail post-bump.

### Failure Modes

(a) **False-positive auto-apply** — classifier mis-categorises a `rule_addition` as `cosmetic` and the diff lands automatically. Mitigation: the success-metrics %-reverted-within-24h target (< 5%) is monitored; if breached, the classifier is downgraded to "always queue" until the regression is found. (b) **Test-gate flake** — TRIGGER_TESTS sporadically fails post-bump and the apply aborts; the operator sees `cuo.proposal_apply_failed` rows and can manually re-apply. (c) **Concurrent apply** — two operators apply different proposals to the same skill simultaneously; the version_bump is keyed on the SKILL.md's current `metadata.version` string, so the second apply fails its byte-for-byte precondition check (analogous to FR-MEMORY-118's `put_if`). (d) **Partial CHANGELOG** — if the apply succeeds but CHANGELOG.md write fails, the audit row is emitted but no human-readable trace; mitigation: idempotent `cyberos-cuo proposal verify <stripe>` reconciles.

## AI Authorship Disclosure

- **Tools used:** Anthropic Claude.
- **Scope:** §1 normative clauses, §2 bump-level table, §4 ACs, §5 named test entries, alternatives, AI Risk Assessment.
- **Human review:** Stephen Cheng reviewed; the safety-class never-auto rule (§1 #10) is explicitly operator-mandated.

## §4 Acceptance Criteria

1. A proposal classified `cosmetic` against a skill with `on_minor_bump: false` auto-applies; SKILL.md `metadata.version` patch increments by 1. *(traces_to: §1 #3, #4)*
2. A proposal classified `rule_addition` with `on_rubric_rule_added: true` moves to `pending_approval/`; SKILL.md is unchanged until `cyberos-cuo proposal approve` runs. *(traces_to: §1 #4, #7)*
3. A proposal with `## Risk class: safety` in body NEVER auto-applies regardless of classifier bucket; goes straight to `pending_approval/`. *(traces_to: §1 #10)*
4. Pre-apply test gate runs acceptance/TRIGGER_TESTS.md against the new skill version; if any TRIGGER_TEST fails, the apply aborts cleanly + emits `cuo.proposal_apply_failed`. *(traces_to: §1 #8)*
5. Each auto-applied proposal emits exactly one `cuo.proposal_applied` aux row + appends one `CHANGELOG.md` entry. *(traces_to: §1 #6, #9)*
6. Each queued proposal emits exactly one `cuo.proposal_queued` aux row. *(traces_to: §1 #6)*
7. `cyberos-cuo proposal classify <id>` is read-only — produces the classifier output WITHOUT mutating any file. *(traces_to: §1 #7)*
8. Version-bump pathing: `cosmetic`/`wording_polish` → patch; `threshold_tune`/`rule_addition` → minor; `rule_removal`/`contract_field_change` → major. *(traces_to: §2)*
9. `cyberos-cuo proposal approve <id>` after a queue: the file moves from `pending_approval/` → `applied/` AND the diff applies; both happen in a single transaction (one fails → both revert). *(traces_to: §1 #7)*
10. After an apply, running `cyberos-cuo proposal list --status applied` includes the stripe with its new SKILL.md version footer. *(traces_to: §1 #5)*

## §5 Verification

- `modules/cuo/tests/test_proposal_applier.py::test_cosmetic_auto_applies` (AC #1, #8)
- `modules/cuo/tests/test_proposal_applier.py::test_rule_addition_queues` (AC #2)
- `modules/cuo/tests/test_proposal_applier.py::test_safety_class_never_auto` (AC #3)
- `modules/cuo/tests/test_proposal_applier.py::test_test_gate_blocks_bad_apply` (AC #4)
- `modules/cuo/tests/test_proposal_applier.py::test_audit_rows_emitted` (AC #5, #6)
- `modules/cuo/tests/test_proposal_applier.py::test_classify_is_read_only` (AC #7)
- `modules/cuo/tests/test_proposal_applier.py::test_bump_levels` (AC #8)
- `modules/cuo/tests/test_proposal_applier.py::test_approve_transactional` (AC #9)
- `modules/cuo/tests/test_proposal_applier.py::test_post_apply_list` (AC #10)
