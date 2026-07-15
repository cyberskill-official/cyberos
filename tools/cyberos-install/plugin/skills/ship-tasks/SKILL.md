---
name: ship-tasks
description: "Run the CyberOS ship-tasks workflow in any repo: drive a task (product or improvement class) through implement -> review -> test -> done, with HITL required at the two human-acceptance gates and gates from the repo's own build/lint/test. Use when asked to ship, implement, or harden a task, or to drive a docs/tasks backlog."
---
# ship-tasks (portable)

The full, normative workflow is in `cuo/ship-tasks.md` (bundled beside this file), with `cuo/EXECUTION-DISCIPLINE.md` (the halt and HITL doctrine) and `cuo/STATUS-REFERENCE.md` (the status lifecycle). Read those; they are the source of truth.

**Resume on restart (ship-manifest@1, TASK-CUO-206):** before starting step 1, check for `docs/tasks/.workflow/<task-ID>.ship.json` (gitignored run state). If present with a matching workflow_version, re-hash its recorded artefacts and resume at the first unproven step per the workflow doc's Resume semantics; a stale or missing manifest just means a fresh run. Write the manifest back after every step, and always re-ask the HITL gates - the manifest never carries an approval.

In one paragraph: pick the first eligible task in `docs/tasks/BACKLOG.md` (`ready_to_implement`, dependencies done). Deep-map the repo, write the edge-case matrix, implement with observability and at least 90% coverage on touched files, review the diff against every section-1 clause, and run the gates (`.cyberos/cuo/gates/run-gates.sh` = the repo's own build/lint/test + coverage; caf and awh only if present). HITL is required: halt at review acceptance and at final acceptance for a recorded human verdict, and never set `done` yourself. On any gate failure, route the task back to `ready_to_implement`. Improvement and hardening work is the same workflow with `class: improvement`.
