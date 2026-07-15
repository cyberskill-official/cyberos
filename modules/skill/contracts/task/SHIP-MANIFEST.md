# ship-manifest@1 - per-task run state for chief-technology-officer/ship-tasks (TASK-CUO-206)

The manifest is a CACHE of proven work, never an authority: resumes re-hash artefacts, human
gates always re-ask, and deleting a manifest costs at most redone work. Task frontmatter and
BACKLOG.md remain the only committed state.

Location: `docs/tasks/.workflow/<task-ID>.ship.json` (gitignored via the scaffolded
`.workflow/.gitignore`). Written after EVERY completed, failed, or conditionally-skipped step
with two-phase atomic writes (`.tmp.<nonce>` then rename), mirroring the memory protocol.

## Fields

| field | type | rule |
|---|---|---|
| manifest_version | const | `ship-manifest@1` |
| task_id | string | the task being shipped |
| task_sha256 | hex64 | hash of the task spec at run start; later mismatch = whole manifest stale |
| workflow_version | string | from the workflow doc frontmatter; mismatch on resume = needs_human |
| started_at / updated_at | ISO-8601 | informational; ordering uses step indices, never timestamps |
| current_step | int 1..31 | |
| routed_back_count | int >= 0 | carried across route-backs |
| steps[] | array | one entry per executed step |
| steps[].index | int 1..31 | |
| steps[].skill | string | chain skill (or gate) name |
| steps[].status | enum | pending, done, failed, skipped-conditional |
| steps[].artefact_path | string or null | null for skipped/gate steps |
| steps[].artefact_sha256 | hex64 or null | re-verified on every resume |
| steps[].verdict | string or null | |
| steps[].completed_at | ISO-8601 or null | |
| hitl | object | `{gate: null or review_approval or final_acceptance, requested_at: ISO or null}` |

## Lifecycle

- Resume (matching workflow_version + task_sha256): start at the first non-done step AFTER
  re-hashing every recorded artefact; the earliest mismatch marks that step and all later
  steps stale (redo from there). task_sha256 mismatch = everything stale (fresh run, history
  and routed_back_count retained). workflow_version mismatch = needs_human, never a silent
  mixed-version run.
- HITL: a recorded `hitl.requested_at` NEVER substitutes for an approval - resuming at a gate
  step re-requests the human verdict.
- Terminal: `done` (gate 2 passed) deletes the manifest; route-back keeps it with
  routed_back_count += 1.

## Queue selection (ship invoked without a task id)

Among tasks at `ready_to_implement` whose `depends_on` are all `done`: order by priority
(MUST < SHOULD < COULD), then `created` ascending, then id ascending. Echo the selection
reasoning line to the operator before step 1: 
`queue: picked <id> (priority=<p>, created=<d>) over <n> other eligible tasks`.

Reference helpers: `modules/cuo/cuo/ship_manifest.py` (validate / resume_plan / select_next /
finalize) - doc-driven agents apply the same algorithm from this contract.
