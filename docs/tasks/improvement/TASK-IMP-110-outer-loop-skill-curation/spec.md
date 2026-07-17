---
id: TASK-IMP-110
title: Outer loop - propose skill amendments from run evidence
template: task@1
type: improvement
module: improvement
status: done
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-17T14:00:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-028, TASK-IMP-093, TASK-IMP-101]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.0.0"
owner: Stephen Cheng (CTO)
created: 2026-07-17
memory_chain_hash: null
effort_hours: 10
service: modules/skill
new_files:
  - modules/skill/workflow-improver/SKILL.md
  - tools/install/tests/test_workflow_improver.sh
modified_files:
  - tools/install/plugin/commands/improve.md
  - tools/install/build.sh
  - docs/tasks/improvement/TASK-IMP-028-ace-style-skill-curation-loop/spec.md
source_pages:
  - "IMPROVEMENT_HANDOFF.md §8 IMP-20 (no outer loop: the workflows cannot learn), §9.2 (TASK-IMP-028 IS this, filed 2026-07-08 and never authored)"
  - "How to build a self-improving code review agent (Zach Lloyd, 2026-07-15): the outer agent synthesises human feedback and proposes a skill change"
  - "This run: the same defect class found in three consecutive review rounds; a human spotted the pattern, not the system"
source_decisions:
  - "2026-07-17 Stephen: PLAN gate - scope C (all 13 actionable handoff findings), template override to task@1 (recorded HITL answer)."
---

# TASK-IMP-110: Outer loop - propose skill amendments from run evidence

## Summary

Every ingredient of a learning loop exists and nothing consumes them: human verdicts at two gates, `routed_back_count`, route-back reasons, `memory.status_overridden`, retrospectives, reconcile reports. All written down, none read back. Add a `workflow-improver` skill that reads the last N runs' evidence and proposes `skill-amendment@1` records - landed as DRAFT tasks, never applied directly.

## Problem

This run proves the gap twice over. An external reviewer found the same defect class - a spec promise with no implementation under it - in three consecutive rounds, and a human noticed the pattern, not the system. Every correction the operator made across five batches ("drop section 4", "gate depends_on", "fix IMP-19 now") lives in a gate log, not in a skill. The next run re-derives them or does not.

TASK-IMP-028 (`ACE-style skill curation loop`) has been in the backlog as a stub since 2026-07-08 - filed, never authored. The idea was ours before it was the article's; what is missing is the authoring.

## Proposed Solution

`workflow-improver` reads a bounded window of run evidence - gate logs, route-back reasons, `status_overridden` rows, reconcile reports - clusters recurring shapes, and emits at most three `skill-amendment@1` proposals per run: which skill, which passage, what evidence (quoted rows with ids), what changes. Each lands as a DRAFT task through create-tasks. It never edits a skill.

The cap is deliberate. An improver that proposes twenty amendments produces a review nobody does, and an unreviewed amendment to a skill is worse than no amendment - it is doctrine nobody agreed to.

## Alternatives Considered

- Open a PR against the skill directly (the article's design). Rejected: our doctrine is that a human accepts every change, and a skill edit is a doctrine change. It proposes a task; the operator ships it through the normal loop.
- Let it apply amendments below a confidence threshold. Rejected: confidence is the model's opinion of itself, which is precisely what the two-gate design exists to not trust.
- Read the whole corpus every run. Rejected: unbounded input, unbounded cost, and recency is what makes a pattern actionable.

## Success Metrics

- Primary: run against this run's own corpus, it independently proposes at least the amendments the operator made by hand - they are on the record, so this is a real eval, not a fixture. Baseline: nothing reads the evidence at all.
- Guardrail: it never writes to `modules/**` or any SKILL.md; every output is a proposal, and a clean window says so rather than manufacturing three.

## Scope

In scope: the `workflow-improver` skill, the `skill-amendment@1` shape, the evidence readers, suite arms.

### Out of scope / Non-Goals

- Applying any amendment automatically.
- Editing skills, rubrics, or workflows directly.
- Proposing amendments to a repo's product code - this reads OUR loop's exhaust, not a consumer's codebase.
- Model-quality evals (TASK-IMP-113's job).

## Dependencies

None mechanically. Consumes artefacts that exist.

**Serialisation note:** touches `build.sh` (shared with TASK-IMP-111 - both extend VENDORED_SKILLS). Parent-serialised per §11a.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from IMPROVEMENT_HANDOFF.md §8 IMP-20 and §9.2, merging the never-authored draft TASK-IMP-028; sources are the cloud-factory articles (Zach Lloyd, 2026-07-15) and this run's own gate logs.
- **Human review:** scope approved at the 2026-07-17 PLAN gate; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 `workflow-improver` MUST read a bounded evidence window (default: the last 20 completed tasks) comprising gate logs, route-back reasons, `memory.status_overridden` rows, and reconcile reports.
- 1.2 It MUST emit at most 3 `skill-amendment@1` proposals per run, each naming: the target skill, the target passage, the evidence (quoted rows with their ids), and the proposed change.
- 1.3 Every proposal MUST cite at least 2 independent evidence rows. A pattern seen once is an anecdote.
- 1.4 It MUST NOT write to `modules/**`, any `SKILL.md`, any rubric, or any workflow file.
- 1.5 Proposals MUST land as `status: draft` tasks via create-tasks, and MUST NOT be self-audited into `ready_to_implement`.
- 1.6 A window with no qualifying pattern MUST report "no amendment proposed" and emit nothing - it MUST NOT pad to the cap.
- 1.7 TASK-IMP-028 MUST be flipped to `duplicate` with `duplicate_of: TASK-IMP-110`, since this task is its authored form.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1, #1.2, #1.3) - given a fixture window with a 3-occurrence pattern, it proposes an amendment citing >=2 evidence rows by id - test: `tools/install/tests/test_workflow_improver.sh::t01_pattern_proposes_with_evidence`
- [ ] AC 2 (traces_to: #1.3) - a single-occurrence pattern yields no proposal - test: `tools/install/tests/test_workflow_improver.sh::t02_anecdote_rejected`
- [ ] AC 3 (traces_to: #1.2) - a window with 8 patterns yields exactly 3 proposals, highest-evidence first - test: `tools/install/tests/test_workflow_improver.sh::t03_cap_enforced`
- [ ] AC 4 (traces_to: #1.4) - a run leaves `modules/**` byte-identical - test: `tools/install/tests/test_workflow_improver.sh::t04_never_writes_skills`
- [ ] AC 5 (traces_to: #1.6) - a clean window reports no proposal and writes nothing - test: `tools/install/tests/test_workflow_improver.sh::t05_clean_window_silent`
- [ ] AC 6 (traces_to: #1.5) - proposals land at `draft` - test: `tools/install/tests/test_workflow_improver.sh::t06_proposals_land_draft`
- [ ] AC 7 (traces_to: #1.7) - TASK-IMP-028 is `duplicate` with a resolving `duplicate_of` - verify: recorded frontmatter read in the gate log plus the FM-113 lint pass on 028 (a corpus state change, not a code path).

## 3. Edge cases

- The window contains a pattern the operator already fixed: it proposes a redundant amendment. Acceptable and cheap to reject - and it is evidence the improver reads the same signals a human did, which is the point.
- Evidence rows contradicting each other (an override reversed later): both cited, no proposal - a contradiction is not a pattern, and the improver must not pick a side.
- A skill named in a proposal that no longer exists: the proposal reds at audit like any other spec citing a dead path. TRACE already covers it.
- Gate logs are prose written by a model: they are UNTRUSTED INPUT. Quoted evidence MUST be reproduced verbatim with its id and MUST NOT be interpolated into any command. Same rule as TASK-IMP-100's rung-5.
- Security-class: reads model-written prose from the repo. No execution of anything it reads; the evidence window is confined under `docs/tasks/**` via `relUnderRoot`.
