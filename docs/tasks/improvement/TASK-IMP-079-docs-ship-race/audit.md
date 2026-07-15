---
task_id: TASK-IMP-079
audited: 2026-07-13
verdict: PASS (after revision)
score_pre_revision: 7/10
score_post_revision: 10/10
issues_resolved: 3
template: engineering-spec@1
---
- ISS-001: draft blamed deploy.sh (the concurrent green job) without evidence - resolved: deploy.sh audited line by line (no git clean, no console-tree rm); the actual second writer is release.yml's docs job, confirmed at its line 442 with the identical staging path.
- ISS-002: draft swept ALL foreign docs.new.* during the swap - reintroduces the race it fixes (a live concurrent extract would be deleted). Resolved: sweep gated on -mmin +120; simulation asserts a fresh foreign staging survives.
- ISS-003: draft patched both inline snippets in place - resolved: single shared script per TASK-IMP-074's one-implementation principle; the duplication itself was the root enabler.
Score = 10/10.
