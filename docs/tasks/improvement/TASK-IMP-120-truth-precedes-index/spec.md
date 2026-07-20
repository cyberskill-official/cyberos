---
id: TASK-IMP-120
title: The index moves without the truth, in both directions
template: task@1
type: improvement
module: improvement
status: done
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-17T17:40:00+07:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
service: tools/install
new_files:
  - (none)
modified_files:
  - tools/install/docs-tools/backlog-mutate.mjs
  - tools/install/tests/test_workflow_helpers.sh
  - tools/install/tests/test_e2e_skeleton.sh
  - modules/skill/backlog-state-update-author/SKILL.md
routed_back_count: 0
awh: N/A
---

# TASK-IMP-120 - the index moves without the truth, in both directions

## Summary

Make `backlog-mutate flip` REFUSE unless the task's frontmatter already carries the target status. The frontmatter is the record of truth and BACKLOG.md is its index; nothing binds the two writes, so they diverge silently. Twice on 2026-07-17, in opposite directions, from two different actors.

## Problem

`STATUS-REFERENCE.md` §1 is unambiguous: frontmatter IS the record of truth, BACKLOG.md is only its index. `backlog-mutate` executes the index write with optimistic concurrency, a pre-image check, a 3-line footprint ceiling and a full retally. The frontmatter write is a separate hand edit with no mechanism at all.

Two instances, one day, opposite directions:

| when | actor | what moved | what did not |
|---|---|---|---|
| TASK-IMP-116 | me | the index, twice (`reviewing -> ready_to_test -> testing`) | the frontmatter, which still said `reviewing` |
| TASK-IMP-028 | a swarm sub-agent | the frontmatter (`draft -> duplicate`, per 110 §1.7) | the index, which still said `draft` |

Neither is carelessness. 116's own coverage gate caught the first by comparing Totals against an independent frontmatter count - 544 index rows, 543 specs. The second was caught the same way, at the batch close. Both times the reconcile was a manual step someone happened to run.

The asymmetry is the tell: the index write is a hardened tool and the truth write is a `sed`. We built the mechanism on the derived artefact and left the authoritative one bare.

TASK-IMP-116's goal file already names this gap and the candidate fix. This task is that fix.

## Proposed Solution

`flip <id> <from> <to>` reads the task's frontmatter FIRST. If it does not already say `<to>`, the flip REFUSES with exit 6 (the existing pre-image refusal code) and names both values. The truth becomes the precondition rather than the afterthought, and the index can only ever catch up to it - never lead it.

This makes the order structural instead of advisory: write the truth, then run the tool. A caller who does it backwards gets a refusal, not a divergence.

## Alternatives Considered

- Have `flip` WRITE the frontmatter too. Rejected: then one tool owns both the truth and its index, and a bug in it corrupts the record with no second opinion. The tool's whole value is that it is narrow and refuses; widening it to write specs makes it the thing it guards against.
- A reconcile step at each flip. Rejected: `task-reconcile.mjs` already exists and already finds this - it just is not required, so it is not run. Another optional check is another thing to skip.
- Lint it. Rejected: the divergence is between two files at one instant, not a property of either.
- Leave it; the gates catch it. Rejected: both instances were caught by a manual reconcile someone chose to run. That is luck with a good habit attached.

## Success Metrics

- Replaying 116's exact sequence (index flipped, frontmatter untouched) REFUSES at the first flip instead of succeeding twice.
- Replaying 028's (frontmatter flipped, index untouched) leaves the index refusable, so the divergence cannot be committed.
- `insert` keeps its current behaviour: it creates a row for a spec that already exists, so the truth is already there by construction. No new refusal there.

## AI Authorship Disclosure

- Tools used: Claude (Fable 5), 2026-07-17 hardening run, via create-tasks.
- Scope: spec drafted by the agent. Both instances in the table are the agent's own - one written directly, one by a sub-agent it dispatched - and both were found by reconciling rather than by any gate. The fix was already named in TASK-IMP-116's goal file before the second instance occurred, which is why this is a task rather than a discovery.
- Human review: @stephencheng recorded the decision to task it at the batch close gate.

## Dependencies

None. `backlog-mutate` already has exit 6 for pre-image refusals, and `flip` already resolves and rewrites a single BACKLOG row. It did NOT previously parse spec frontmatter anywhere: `insert` reads only BACKLOG rows and never opens a spec, so before this task no command read a spec's `status:`. THIS task adds that frontmatter read to `flip` (`resolveSpecPaths` + `frontmatterStatus`, mirroring `task-reconcile`'s id->spec resolver) and gates the existing index write on it, reusing exit 6. A precondition on an existing refusal path, plus the small reader it needs - not a new writer, and not a new refusal code.

## Scope

In scope: the `flip` precondition in `backlog-mutate.mjs`, its suite arms, and the ordering contract in `backlog-state-update-author/SKILL.md`.

### Out of scope / Non-Goals

- Making `flip` write frontmatter. See Alternatives - that is the failure mode, not the fix.
- Any change to `insert`. A row is inserted for a spec that exists; the truth precedes it already.
- The status page. Group A already binds the page to the cell; this binds the cell to the truth.
- Retroactively reconciling the corpus. It agrees today, on all eight live statuses. Keeping it that way is this task; proving it stays that way is TASK-IMP-109's territory.

## 1. Clauses

1.1 `flip <id> <from> <to>` MUST read `<id>`'s spec frontmatter and REFUSE with exit 6 unless `status` already equals `<to>`, naming both the frontmatter value and the requested target. Test: `t14_flip_refuses_when_truth_disagrees`

1.2 When the frontmatter DOES equal `<to>`, flip proceeds exactly as today - same pre-image checks, same 3-line footprint, same retally. This adds a precondition and changes nothing else. Test: `t15_flip_proceeds_when_truth_agrees`

1.3 A spec that cannot be found or read MUST refuse, not proceed. An unreadable truth is not a matching truth. Test: `t16_unreadable_spec_refuses`

1.4 `insert` MUST be unaffected. Test: `t17_insert_unchanged`

1.5 `backlog-state-update-author/SKILL.md` MUST state the ordering as a contract: write the truth, then the index; the tool enforces it. Both vendored copies. Test: `t18_skill_states_the_order`

1.6 The seam test `tools/install/tests/test_e2e_skeleton.sh` - the end-to-end spine that drives flip -> coverage-scope -> reconcile -> uninstall - MUST exercise its lifecycle flips in the truth-first order this task mandates (the spec frontmatter `status` is written to `<to>` BEFORE the index flip), and MUST positively assert that an index-first flip (the truth still lagging) REFUSES with exit 6 and does NOT move the row. The guard's behaviour change broke this seam - it had encoded index-first, truth-lagging flips as correct, the exact shape 1.1 forbids - so the cone is grown to correct the seam to the new contract, never to weaken the guard. Test: `t05_index_first_flip_refuses` (index-first flip refuses exit 6, row unmoved; the truth-first flip then proceeds) and `t01_spine_green` (the full spine drives every lifecycle flip truth-first).

## 3. Edge case matrix

| # | Category | Trigger | Expected | Test |
|---|---|---|---|---|
| 1 | NULL/EMPTY | spec has no `status` field | refuse - absent is not agreement | t16 |
| 2 | NULL/EMPTY | spec file missing entirely | refuse, name the path | t16 |
| 3 | BOUNDS | frontmatter equals `<to>` exactly | proceed | t15 |
| 4 | BOUNDS | frontmatter equals `<from>` (caller ran it backwards) | refuse - this is the whole defect | t14 |
| 5 | MALFORMED | `status: done  # comment` (FM-001, live in 501 specs) | trailing comment stripped before compare | t14 |
| 6 | MALFORMED | two `status:` lines | refuse - ambiguous truth is not truth | t16 |
| 7 | MALFORMED | status value with trailing whitespace | trimmed, compares equal | t15 |
| 8 | CONCURRENT | frontmatter changes between read and write | existing row pre-image still catches it | t15 |
| 9 | SECURITY | id resolves to a spec outside docs/tasks | refuse - `relUnderRoot`, as everywhere else | t16 |
| 10 | DEGRADATION | corpus has duplicate ids for `<id>` | refuse - existing duplicate-row refusal path | t16 |
| 11 | DEGRADATION | a status not in STATUS_ORDER | refuse before touching the file | t14 |
| 12 | CONCURRENT | seam driver moves the index before writing the truth (index-first) | refuse exit 6, row unmoved - truth precedes index, end to end | t05_index_first_flip_refuses (test_e2e_skeleton.sh) |

## 4. Out of scope / non-goals

See "## Scope -> ### Out of scope / Non-Goals" above.

## Acceptance criteria

- AC1 (traces_to #1.1): flip refuses, exit 6, naming both values. Test: `t14_flip_refuses_when_truth_disagrees`.
- AC2 (traces_to #1.2): flip proceeds unchanged when the truth agrees. Test: `t15_flip_proceeds_when_truth_agrees`.
- AC3 (traces_to #1.3): unreadable or ambiguous truth refuses. Test: `t16_unreadable_spec_refuses`.
- AC4 (traces_to #1.4): insert unaffected. Test: `t17_insert_unchanged`.
- AC5 (traces_to #1.5): the skill states the ordering contract, both copies. Test: `t18_skill_states_the_order`.
- AC6: replaying TASK-IMP-116's sequence (index flipped, frontmatter untouched) refuses at the FIRST flip. A fix that cannot stop the case that motivated it is decoration. Test: `t19_replays_the_116_divergence`.
- AC7 (traces_to #1.6): the seam test `test_e2e_skeleton.sh` drives its lifecycle flips truth-first and positively asserts an index-first flip refuses (exit 6) without moving the row - the guard's behaviour change is covered by the seam it broke, not merely tolerated. The operator-approved cone expansion (the fix belongs in this task's cone, not a separate one). Test: `t05_index_first_flip_refuses`, `t01_spine_green`.
