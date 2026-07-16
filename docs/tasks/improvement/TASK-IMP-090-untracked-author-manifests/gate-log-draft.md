# TASK-IMP-090 gate-log evidence (implementing -> ready_to_review)

E1 - SKILL default (AC 1):
  $ grep -n manifest_path modules/skill/task-author/SKILL.md
  184:manifest_path:  <from caller; default: docs/tasks/.workflow/task-author.<slug>.manifest.json>

E2 - hygiene suite (AC 2): install-hygiene: 17 passed, 0 failed; ok t07_workflow_gitignore_patterns

E3 - live scratch consumer install seed:
  $ cat docs/tasks/.workflow/.gitignore
  *.ship.json
  *.manifest.json

E4 - index cleanup (AC 3):
  $ git status --short docs/tasks/.workflow
  D  docs/tasks/.workflow/task-author.improvement-batch.manifest.json
  D  docs/tasks/.workflow/task-author.improvement-batch-2.manifest.json
  D  docs/tasks/.workflow/task-author.improvement-batch-3.manifest.json
   M docs/tasks/.workflow/.gitignore
  $ git ls-files docs/tasks/.workflow | grep -c 'manifest.json'
  0
  (the remaining tracked entries under .workflow are per-task phase artefacts and .gitignore,
   which are not session state and stay tracked)

E5 - approval record (AC 4): docs/tasks/_audits/IMPROVEMENT-BATCHES-2026-07-16.md
  121 lines; 27 member-id mentions covering TASK-IMP-082..092 with PLAN approvals,
  HITL verdicts, evidence commits, and the 086 corrective-incident pointer.

## PR-review addendum (2026-07-17, CI awh gate)

The awh gate (out-of-band rerun of the cuo golden suite against its sealed baseline) failed
closed on this PR: `modules/cuo/tests/test_ship_manifest.py::test_gitignore_scaffold`
(TASK-CUO-206 AC 6) pinned the `.workflow/.gitignore` seed to exactly `*.ship.json` - the
seed this task deliberately extended. Fix: the test now asserts the two-pattern seed
(`*.ship.json`, `*.manifest.json`), asserts both patterns in install.sh, and probes
`git check-ignore` for a `task-author.x.manifest.json` path too (citing TASK-IMP-090).
Rerun: test_ship_manifest.py 8/8; full cuo suite 260 passed, 2 skipped (env-dependent,
run in CI). This was exactly the gate doing its job: an acceptance test guarding a seed
one batch changed, caught by the sealed rerun rather than by anyone's memory.
