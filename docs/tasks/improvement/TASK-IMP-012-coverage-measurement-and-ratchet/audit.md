---
task_id: TASK-IMP-012
audited: 2026-07-25
verdict: PASS
score: 9/10
template: task@1
---

# TASK-IMP-012 audit

Honest 1.x scope: install-suite test-touch ratchet, not cargo llvm-cov. Baseline fail-closed, regression-only fail, explicit `--write-baseline`. Companion to coverage-scope (IMP-098) without breaking its CLI. ACs map 1:1 to suite scenarios.

SUMMARY verdict: pass issues_open: 0 next_action: implement
