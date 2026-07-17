---
id: TASK-IMP-123
title: phase must have a mechanism or be removed
template: task@1
type: improvement
module: improvement
status: draft
priority: p2
author: "@stephencheng"
department: engineering
created_at: 2026-07-18T04:00:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-116, TASK-IMP-104, TASK-IMP-119]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.0.0"
owner: Stephen Cheng (CTO)
created: 2026-07-18
memory_chain_hash: null
effort_hours: 4
service: tools/install
new_files:
  - (none)
modified_files:
  - tools/install/docs-tools/batch-select.mjs
  - tools/install/docs-tools/task-lint.mjs
  - tools/install/templates/TASK-TEMPLATE.md
  - tools/install/tests/test_batch_select.sh
source_pages:
  - "2026-07-18 measured on main 0ddcf73b: grep -c phase batch-select.mjs = 0; grep -c phase task-lint.mjs = 0; phase absent from TASK-TEMPLATE.md; only tools/docs-site/* render it"
  - "docs/tasks/**: phase carries TWO vocabularies - 'pre-1.0.0 release'/'post-1.0.0' (IMP) vs 'P0'/'P2' (INV/OBS/TEN) - and is ABSENT on TASK-IMP-117..120"
source_decisions:
  - "2026-07-18 Stephen: batch order fixed by moving release intent into priority (commit d19362ad), NOT by teaching batch-select phase - this task carries the structural question separately."
---

# TASK-IMP-123: phase must have a mechanism or be removed

## Summary

`phase` reads like a release gate and schedules nothing. It is absent from the task template, absent from the machine floor, and absent from the batch selector; its only consumers are the docs-site renderers, which draw it as a badge. On 2026-07-18 it let TASK-IMP-111 - explicitly `phase: post-1.0.0` - take batch 1 and block every `pre-1.0.0 release` task behind it. This task decides whether `phase` earns a mechanism or leaves the corpus.

## Problem

Measured on main `0ddcf73b`:

- `grep -c phase tools/install/docs-tools/batch-select.mjs` -> 0
- `grep -c phase tools/install/docs-tools/task-lint.mjs` -> 0
- `phase` does not appear in `tools/install/templates/TASK-TEMPLATE.md`
- the only readers are `tools/docs-site/render-task-catalog.mjs` (a badge) and `tools/docs-site/nfr-extract.mjs`

So `phase` is an undeclared, unvalidated, free-text field that no decision reads. Two consequences, both observed:

1. **It misleads.** `batch-select` sorts on `priority` alone (`:88`). TASK-IMP-111 (`p1`, `post-1.0.0`) outranked TASK-IMP-106 and -107 (`p2`, `pre-1.0.0 release`) and, its cone spanning all three services, excluded all ten other eligible tasks. The one task marked "not for this release" was scheduled first and blocked the release queue. An author who wrote release intent into `phase` had no way to know nothing would read it.
2. **It is two fields wearing one name.** Improvement tasks use `pre-1.0.0 release` / `post-1.0.0` - a release gate. Product tasks (INV/OBS/TEN) use `P0` / `P2` - a rollout wave, matching `render-module-changelog.mjs`'s module map. TASK-IMP-117..120 have no `phase` at all. Nothing reconciles them, and the docs-site renders both as the same badge.

This is the shape TASK-IMP-116 was created to close for `## 11a`'s batch default: a promise with no mechanism under it. The immediate scheduling harm was addressed on 2026-07-18 by moving the release intent into `priority`, the field that does schedule (commit `d19362ad`) - a fix that works and leaves a known cost: `priority` now carries two meanings, importance and release-gating, which is the ambiguity `phase` existed to separate.

## Proposed Solution

DECISION REQUIRED, and the task must not pre-judge it. Two defensible ends, and the one thing that is not defensible is the status quo:

- **(a) Give it a mechanism.** Add `phase` to the template with a closed enum, lint it, reconcile the two vocabularies across the corpus, and give `batch-select` a documented rule for how phase and priority compose.
- **(b) Remove it.** Delete `phase` from the corpus and the renderers; `priority` carries scheduling and the release checklist carries release gating. Costs the badge, ends the ambiguity.

The operator picks at the PLAN gate of the implementing run. Whichever is chosen, the acceptance below is symmetric: after this task, no field named `phase` may exist that a reader could reasonably believe schedules something while nothing reads it.

## Alternatives Considered

- Teach `batch-select` to read `phase` as-is. Rejected: with two live vocabularies and four tasks missing the field, a scheduler reading it would have to guess what `P2` means against `post-1.0.0`, and guessing is how it misled in the first place.
- Leave it as a badge and document that it is decorative. Rejected: the badge is exactly what makes an author believe it is load-bearing. A field that looks like a gate and is documented elsewhere as inert is a trap with a footnote.
- Fold into TASK-IMP-119 (cone drift audit). Rejected: 119 audits declared cones against touched files - a different question about a different field.

## Success Metrics

- Primary: no field exists that appears to gate scheduling while nothing reads it - verified by an assertion tied to whichever branch is chosen, not by inspection.
- Guardrail: whichever branch, the batch selected for the current queue MUST be explainable from the fields that remain.

## Scope

In scope: the decision, and its execution across template, lint, batch-select, the corpus, and the renderers.

### Out of scope / Non-Goals

- Reverting `d19362ad`'s priority flips. They stand regardless; if (a) is chosen, whether to move the intent back is a separate call.
- `priority`'s own semantics.
- The docs-site's visual design.

## Dependencies

None. Independent of TASK-IMP-119 and TASK-IMP-116 despite the shared family.

## AI Authorship Disclosure

- **Tools used:** Claude (Opus 4.8) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from greps and a measured batch-select run on merged main; the decision itself is deliberately left to the operator. Implementation under ship-tasks supervision.
- **Human review:** filed at the 2026-07-18 PLAN gate as `[DECISION]`; the (a)/(b) call is the operator's at the implementing run's PLAN gate. Both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 The implementing run MUST halt at its PLAN gate for the operator to choose branch (a) mechanism or (b) removal. The run MUST NOT choose on the operator's behalf.
- 1.2 If (a): `phase` MUST be added to `TASK-TEMPLATE.md` with a closed enum, and `task-lint` MUST reject a value outside it.
- 1.3 If (a): every task in the corpus MUST carry a `phase` value inside the enum - the two live vocabularies reconciled, and the four tasks now missing the field given one.
- 1.4 If (a): `batch-select` MUST apply a documented rule composing `phase` and `priority`, and MUST state that rule where a reader of its output can find it.
- 1.5 If (b): `phase` MUST be removed from every task in the corpus and from every renderer that reads it, and `task-lint` MUST reject `phase` as an unknown key so it cannot return by habit.
- 1.6 Either branch MUST leave `docs/status/` renderable and the site build green.
- 1.7 Neither branch may leave a field that a task author could reasonably read as scheduling while no scheduler reads it.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - the run's PLAN gate records an operator verdict naming (a) or (b) before any file is written - verify: the run's ship-manifest carries the recorded HITL verdict naming the chosen branch, and its timestamp precedes the first file write (ops check recorded in the gate log; a human verdict cannot be discharged by a test, and no test is claimed for it).
- [ ] AC 2 (traces_to: #1.2, #1.5) - branch (a): a task with `phase: nonsense` fails task-lint non-zero. Branch (b): a task carrying any `phase` key fails task-lint non-zero. The assertion MUST be on the exit code, not on printed text - test: `tools/install/tests/test_batch_select.sh::t30_phase_enum_or_absent_enforced`
- [ ] AC 3 (traces_to: #1.3, #1.5) - the whole corpus satisfies the chosen branch: zero tasks outside the enum (a), or zero tasks carrying `phase` (b) - test: `tools/install/tests/test_batch_select.sh::t31_corpus_conforms`
- [ ] AC 4 (traces_to: #1.4) - branch (a) only: a fixture where phase and priority disagree selects the batch the documented rule predicts, and the test MUST FAIL if `batch-select` ignores `phase` - test: `tools/install/tests/test_batch_select.sh::t32_phase_composes_with_priority`
- [ ] AC 5 (traces_to: #1.7) - the regression that motivated this task cannot recur: a fixture reproducing the 2026-07-18 queue (a p1 post-1.0.0 task with a three-service cone against p2 pre-1.0.0 tasks) MUST NOT silently schedule the post-1.0.0 task first - under (a) the rule prevents it; under (b) no field claims otherwise and the test asserts `phase` is absent from the corpus. A rule that cannot fail the case that motivated it is decoration - test: `tools/install/tests/test_batch_select.sh::t33_motivating_case_cannot_recur`
- [ ] AC 6 (traces_to: #1.6) - `bash tools/docs-site/build.sh` exits 0 and the status page renders after the change - test: `tools/install/tests/test_batch_select.sh::t34_site_still_builds`

## 3. Edge cases

- A consumer repo whose tasks carry `phase` values from neither vocabulary: branch (a)'s enum MUST be chosen knowing it lints third-party corpora; branch (b) removes the field from ours and MUST NOT silently rewrite theirs.
- The four tasks with no `phase` (TASK-IMP-117..120): under (a) they need a value, and inventing one is an authoring decision, not a migration's - the run MUST surface them rather than default them.
- Product tasks' `P0`/`P2`: these mirror `render-module-changelog.mjs`'s module map, so (b) MUST confirm nothing else reads them before deleting.
- A task in flight when the migration lands: the corpus check runs against the tree at the time, and a task authored mid-migration with the old shape fails lint - which is the mechanism working, not a bug.
- `done` tasks: 183 of them carry `phase`; whether history is rewritten or grandfathered is a scope call for the PLAN gate and MUST NOT be assumed. Retroactively editing shipped specs on a rule they predate is the machine making a scope decision that belongs to the operator (same reasoning as TASK-IMP-118 §3 row 11).
- Security-class: none. No untrusted input, no execution path.
