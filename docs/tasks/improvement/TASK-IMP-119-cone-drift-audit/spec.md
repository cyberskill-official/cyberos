---
id: TASK-IMP-119
title: A declared cone is a promise nothing checks
template: task@1
type: improvement
module: improvement
status: testing
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-17T17:30:00+07:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
service: tools/install
new_files:
  - tools/install/docs-tools/cone-audit.mjs
  - tools/install/tests/test_cone_audit.sh
modified_files:
  - tools/install/build.sh
routed_back_count: 0
awh: N/A
---

# TASK-IMP-119 - a declared cone is a promise nothing checks

## Summary

Audit a task's ACTUAL writes against its DECLARED cone at the `implementing -> ready_to_review`
flip, and report every escape. batch-select plans parallelism from cones; nothing verifies them
afterwards, so the safety argument for every swarm rests on an unchecked input.

## Problem

batch-select proves two tasks are independent by intersecting their cones
(`new_files ∪ modified_files ∪ service`). That proof is only as true as the cones. Nothing compares
a cone to what the task actually wrote.

Measured on the 2026-07-17 batch (TASK-IMP-110 + TASK-IMP-114), three files escaped BOTH declared
cones:

| file | owner | why it escaped |
|---|---|---|
| `tools/install/docs-tools/workflow-improve.mjs` | 110 | the cone declared a SKILL.md and a test; §1.4 says "the tool MUST NOT write" and AC 1-6 are shell-tested behaviour. Prose cannot satisfy them. The executable was never declared. |
| `tools/install/tests/test_full_sdp_payload.sh` | 110 | hard-asserts 53 vendored skills; 110's DECLARED build.sh edit makes it 54. An undeclared file broken by a declared change. |
| `tools/docs-site/tests/test_render_status_hub.sh` | 114 | AC 3 and AC 5 both cite it; `modified_files` omits it. |

None collided, so the batch was safe by luck rather than by proof. `test_batch_economics.sh` (114,
inside `tools/install`) would have raced TASK-IMP-106 (`service: tools/install`) had 106 been in
the batch - and batch-select could not have known, because 114's cone said `modules/cuo`.

This is the third level of one defect, all found the same day:
1. TASK-IMP-104: declared `install.sh`, edited two more files inside its service. Fix: fold
   `service` into the cone.
2. This afternoon: an undeclared cone is the EMPTY SET, which intersects nothing, so a silent spec
   was provably independent of everything and joined every batch. Fix: fail closed, ship alone.
3. This task: a DECLARED cone that does not match the writes. Nothing looks.

Levels 1 and 2 widened the net. This one closes the loop: it checks the input the whole mechanism
trusts.

## Proposed Solution

`docs-tools/cone-audit.mjs`: given a task id and a base ref, diff `git diff --name-only <base>..HEAD`
against the task's declared cone and report every path outside it. REPORT, not refuse - a write
discovered mid-implementation is often a real finding (all three of today's were), and the remedy
is to amend the cone, which is a spec edit and therefore a human's call.

Wire it into ship-tasks at the `implementing -> ready_to_review` flip, where the diff exists and
the spec is still open for amendment.

## Alternatives Considered

- Refuse the flip on any escape. Rejected for now: today's three escapes were all correct
  discoveries about wrong specs, and a hard block would have stopped three good implementations to
  enforce three bad cones. Revisit once escape rates are known - which is itself a §11d question.
- Derive the cone from the diff instead of declaring it. Rejected: then the cone is whatever
  happened, and it can never be wrong, so batch-select's proof becomes circular. A cone is a
  prediction; its value is that it can be falsified.
- Lint the cone at authoring. Rejected: unknowable. The author cannot know at PLAN time which files
  the implementation will touch - that is what implementation is.
- Do nothing; escapes were benign. Rejected: they were benign by luck. The near-miss is on the
  record above.

## Success Metrics

- Run against the 2026-07-17 batch, it names exactly the three files in the table and no others.
  A tool that cannot find the defect that motivated it is decoration.
- Escapes become visible per batch, so the escape RATE becomes a number rather than an anecdote -
  which is what a decision about refusing needs.

## AI Authorship Disclosure

- Tools used: Claude (Fable 5), 2026-07-17 hardening run, via create-tasks.
- Scope: spec drafted by the agent from a defect it measured mechanically while acting as the swarm
  parent. Both sub-agents disclosed their own escapes unprompted; the parent's diff check confirmed
  three and found no others. The cone data in the table is measured, not estimated.
- Human review: @stephencheng recorded the decision to task this (report, not refuse) at the batch's
  close gate.

## Dependencies

None. batch-select (TASK-IMP-104, v2.8.0) already parses cones; `relUnderRoot` already exists in
docs-tools. This adds a reader, no new guard and no new rule.

## Scope

In scope: `docs-tools/cone-audit.mjs`, its suite, the build.sh vendor entry, and the ship-tasks §11a
wiring note at the `implementing -> ready_to_review` flip.

### Out of scope / Non-Goals

- Refusing the flip. This reports. Turning a report into a gate is a separate decision that wants
  the escape rate first.
- Amending any existing spec's cone. The three files above are findings for their owners, not this
  task's work.
- Cone declaration at authoring time. Unknowable by construction; see Alternatives.

## 1. Clauses

1.1 `cone-audit.mjs <task-id> [--base <ref>]` MUST read the task's declared cone from frontmatter
(`new_files ∪ modified_files ∪ service`) and the actual writes from `git diff --name-only`, and
report every written path not inside the cone.
Test: `t01_escape_is_named`

1.2 A path is INSIDE the cone if it equals a declared entry or is nested under one. This MUST match
batch-select's own containment rule exactly - two tools disagreeing about what a cone contains is
worse than neither existing.
Test: `t02_containment_matches_batch_select`

1.3 An UNDECLARED cone MUST report every write as an escape, not zero. Empty is not "contains
everything" any more than it is "contains nothing" - it is unknown, and batch-select already
refuses it.
Test: `t03_undeclared_cone_escapes_everything`

1.4 `(none)` MUST be filtered from the cone, matching batch-select:51. A literal placeholder is not
a path.
Test: `t04_placeholder_is_not_a_path`

1.5 The tool MUST NOT write, refuse, or flip anything. It reports and exits 0 with escapes, 2 on
usage. An escape is a FINDING for a human, not a failure.
Test: `t05_reports_never_refuses`

1.6 Deterministic: same repo state + same args = byte-identical output. No wall clock in the
artefact.
Test: `t06_deterministic`

1.7 A path outside the repo root, or an unreadable spec, MUST be refused and named - never silently
skipped.
Test: `t07_guard_refuses_and_names`

## 3. Edge case matrix

| # | Category | Trigger | Expected | Test |
|---|---|---|---|---|
| 1 | NULL/EMPTY | task wrote nothing | zero escapes, exit 0 | t01 |
| 2 | NULL/EMPTY | cone declared, diff empty | zero escapes | t01 |
| 3 | NULL/EMPTY | cone entirely absent | every write is an escape | t03 |
| 4 | BOUNDS | write exactly equals a declared file | inside | t02 |
| 5 | BOUNDS | write nested under a declared service | inside | t02 |
| 6 | BOUNDS | declared file, write is its PARENT dir | escape - narrower does not cover wider | t02 |
| 7 | MALFORMED | `(none)` in new_files | filtered, not treated as a path | t04 |
| 8 | MALFORMED | cone entry with a trailing slash | normalised, matches batch-select | t02 |
| 9 | MALFORMED | renamed file (git R status) | both old and new path considered | t01 |
| 10 | CONCURRENT | run mid-swarm while a sibling writes | reports only THIS task's diff vs base | t06 |
| 11 | SECURITY | task-id resolves outside the repo | REFUSED, named, not executed | t07 |
| 12 | SECURITY | spec path is a symlink out of the corpus | REFUSED | t07 |
| 13 | DEGRADATION | git absent / base ref missing | refuse and say so; never report zero escapes | t07 |
| 14 | DEGRADATION | spec unreadable | refuse; unreadable is not "clean" | t07 |

## 4. Out of scope / non-goals

See "## Scope -> ### Out of scope / Non-Goals" above.

## Acceptance criteria

- AC1 (traces_to #1.1): escapes are named. Test: `t01_escape_is_named`.
- AC2 (traces_to #1.2): containment matches batch-select on a shared table of cases. Test:
  `t02_containment_matches_batch_select`.
- AC3 (traces_to #1.3): an undeclared cone escapes everything. Test:
  `t03_undeclared_cone_escapes_everything`.
- AC4 (traces_to #1.4): `(none)` is filtered. Test: `t04_placeholder_is_not_a_path`.
- AC5 (traces_to #1.5): reports, never refuses or writes. Test: `t05_reports_never_refuses`.
- AC6 (traces_to #1.6, #1.7): deterministic; guard refuses and names. Tests: `t06_deterministic`,
  `t07_guard_refuses_and_names`.
- AC7: run against the 2026-07-17 batch (base = the batch's first commit), it names exactly
  `workflow-improve.mjs`, `test_full_sdp_payload.sh`, `test_render_status_hub.sh` and no others.
  A tool that cannot find the case that motivated it is decoration. Test:
  `t08_finds_the_motivating_case`.
