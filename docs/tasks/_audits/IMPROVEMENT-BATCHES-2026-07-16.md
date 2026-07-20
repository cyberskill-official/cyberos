---
record: improvement-batch-approvals
scope: "TASK-IMP-082..092 - improvement batches 1-3, authored and shipped 2026-07-16"
created: 2026-07-16
written_by: "TASK-IMP-090 §1 #1.4 (ship-tasks batch 3); verdicts recorded are the operator's"
operator: "Stephen Cheng (@stephencheng)"
---
# Improvement batches 1-3 - tracked approval record (2026-07-16)

Durable approval record for the three improvement batches authored from the sachviet-run handoff and shipped through ship-tasks on 2026-07-16. Task-author run manifests are untracked session state as of TASK-IMP-090 (recorded decision IMP-11): the three batch manifests left the git index the day this record landed, and THIS document is the tracked record of what was planned, who approved it, and what landed. Git history before the untracking retains the manifest files; no history was rewritten.

## Batch 1 - TASK-IMP-082..084 (branch batch/improvement-2026-07-16)

Members (all done):

| id | title |
|---|---|
| TASK-IMP-082 | Status page provenance stamp becomes a corpus fingerprint (byte-stable) |
| TASK-IMP-083 | install lands the status-sync hook where core.hooksPath points |
| TASK-IMP-084 | task-lint, a deterministic machine floor under the task-audit rubric |

- PLAN approval: operator, 2026-07-16 (batch PLAN gate; session state was `task-author.improvement-batch.manifest.json`, untracked as of this record).
- HITL verdicts: review acceptance and final acceptance recorded as batched operator verdicts, 2026-07-16 (final acceptance commit ca9ae490: "human final acceptance recorded by operator, batched verdict").

Evidence commits:

| commit | what it evidences |
|---|---|
| 27292774 | batch 1 authored + audited, ready_to_implement |
| f840ba84 | enter implementing (ship-tasks batch round) |
| e6c9092a | swarm round - fp- corpus stamp, hooksPath-aware hook install/uninstall, task-lint machine floor |
| 9fcf1b98 | implementing -> ready_to_review -> reviewing (batch phases 1-2) |
| 2b215aed | testing phase - coverage-gate@1 x3, member suites on the record (6/6, 13/13, 8/8) |
| ca9ae490 | testing -> done - human final acceptance (batched, operator, 2026-07-16); batch drained |

## Batch 2 - TASK-IMP-085..087 (branch batch/2-workflow-helpers)

Members (all done):

| id | title |
|---|---|
| TASK-IMP-085 | Doc-driven workflow helpers, ship-manifest and backlog-mutate CLIs |
| TASK-IMP-086 | Backfill the improvement backlog index rows 068-081 to frontmatter truth |
| TASK-IMP-087 | Release-readiness checklist for 1.0.0 at docs/release/ |

- PLAN approval: operator, 2026-07-16 (batch-2 PLAN gate; session state was `task-author.improvement-batch-2.manifest.json`, untracked as of this record). At this gate the operator ALSO recorded the three decisions that became batch 3's decided items:
- IMP-06 - consumer installs scaffold a live `task_template: task@1` config line, platform repo untouched (option (a)) -> TASK-IMP-088.
- IMP-07 - the task@1 template drops the duplicate out-of-scope section 4; Scope > Out of scope is the single home -> TASK-IMP-089.
- IMP-11 - author-run manifests become untracked .workflow session state; a tracked `_audits/` batch summary (this document) is the durable approval record -> TASK-IMP-090.
- HITL verdicts: review acceptance and final acceptance recorded as batched operator verdicts, 2026-07-16 (final acceptance commit 6ccafb6c).

Evidence commits:

| commit | what it evidences |
|---|---|
| e9cfb97a | batch 2 authored + machine-floor linted + audited, ready_to_implement |
| 80df2447 | enter implementing (ship-tasks batch 2 round) |
| 96991264 | swarm round - ship-manifest + backlog-mutate CLIs, backlog index backfill 068-081, 1.0.0 release checklist |
| 7ea7fe36 | implementing -> reviewing (phases 1-2), flips via backlog-mutate.mjs (dogfood) |
| d67dda16 | testing phase - coverage-gate@1 x3, member evidence on the record |
| 6ccafb6c | testing -> done - human final acceptance (batched, operator, 2026-07-16); batch 2 drained |
| 092c9887 | TASK-IMP-086 corrective repair (see incident below) |
| 19752e64 | TASK-IMP-086 corrective addendum recorded against the committed object |

### TASK-IMP-086 corrective incident (post-acceptance)

TASK-IMP-086's acceptance evidence was measured on a working filesystem view whose writes never reached the committed objects: concurrent writes to docs/tasks/BACKLOG.md through Cowork's two views (host-side task agent, sandbox-side parent) lost updates across views, so no commit on batch/2-workflow-helpers carried the 14 backfilled rows and the batch-1 rows 082-084 fell out of committed state in the same window. Flagged by the PR review bot against the pushed branch. Repair: commit 092c9887 (single-writer re-insert of all 17 rows via backlog-mutate.mjs, full header retally, verification against `git show 092c9887:docs/tasks/BACKLOG.md`). Full root-cause narrative and the adopted rule live in the addendum: `docs/tasks/improvement/TASK-IMP-086-backlog-index-backfill/gate-log-draft.md` §"CORRECTIVE ADDENDUM (2026-07-16, post-acceptance verification)", recorded on the branch by commit 19752e64. Rules adopted: one writer through one view for shared files, and acceptance evidence for content deliverables measured on the committed object - filed as IMP-18 and hardened by TASK-IMP-092.

## Batch 3 - TASK-IMP-088..092 (branch batch/3-decided-items)

Members (implementing at the time of this record):

| id | title | decided item / finding |
|---|---|---|
| TASK-IMP-088 | install scaffolds task_template task@1 in consumer config.yaml | IMP-06 |
| TASK-IMP-089 | task@1 template drops the duplicate out-of-scope section 4 | IMP-07 |
| TASK-IMP-090 | task-author manifests default to untracked .workflow session state | IMP-11 |
| TASK-IMP-091 | regen_backlog emits every status and recomputes Totals from frontmatter | IMP-17 |
| TASK-IMP-092 | Lost-update hardening, retally headers and committed-object evidence | IMP-18 |

- PLAN approval: operator, 2026-07-16 (batch-3 PLAN gate; decided items IMP-06/07/11 were recorded at the batch-2 gate, findings IMP-17/18 filed from batch-2's execution record; session state was `task-author.improvement-batch-3.manifest.json`, untracked as of this record).
- HITL verdicts: PENDING - review acceptance and final acceptance are open human gates for this batch. This record is written from inside batch 3 (TASK-IMP-090 clause 1.4); the closing verdicts land as later commits on batch/3-decided-items and do not amend this section's approval facts.

Evidence commits so far:

| commit | what it evidences |
|---|---|
| 53ef658f | batch 3 authored (IMP-06/07/11 + IMP-17/18), machine-floor linted clean first pass, audited, ready_to_implement |
| c81a8c0c | enter implementing (5 members; 088+090 serial in one agent, rest parallel) |
