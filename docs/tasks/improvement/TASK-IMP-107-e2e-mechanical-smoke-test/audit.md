---
task_id: TASK-IMP-107
audited: 2026-07-17
verdict: PASS (after revision)
score_pre_revision: 7/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 6
template: task@1
audit_rubric_version: audit_rubric@2.0
audited_file_sha256: 672edb10f34ee528
audited_body_sha256_prefix: 5f1be3e23a147ec7
machine_floor: task-lint clean (exit 0) - run FIRST per TASK-IMP-084
---

## §1 - Verdict summary

Spec is 76 lines, 6 §1 clauses, 5 ACs, 5 edge cases. Closes the one seam that 25 suites leave untested: the plumbing between helpers. Passes after 6 findings.

## §2 - Findings (all resolved)

### ISS-001 - Suite could assert exit code rather than behavior
`task-reconcile` exiting 0 says nothing about reaching the right recommendation. Resolved: §1 #1.3 requires asserting the recommendation against the constructed state; AC 2 covers it.

### ISS-002 - A model-dependent test would be flaky by construction
An e2e test calling a model fails for reasons unrelated to plumbing - the definition of a flaky gate. Resolved: §1 #1.5 forbids model, network, credentials; Alternatives records why.

### ISS-003 - 30 s target could creep past the 45 s sandbox cap
A suite that cannot finish gets disabled, which is worse than not having it. Resolved: §3 edge case requires splitting by phase rather than raising the cap.

### ISS-004 - Suite could pollute the working corpus
An e2e test writing to `docs/tasks/` corrupts the repo it tests. Resolved: §1 #1.1 confines it to scratch; AC 4 asserts the working corpus is untouched.

### ISS-005 - Corpus survival after uninstall was untested anywhere
It is the one outcome an operator cannot recover from. Resolved: §1 #1.4 + AC 3 make it a gate.

### ISS-006 - Missing git would fail rather than skip
A machine without git would red the suite for an environmental reason. Resolved: §3 adopts the existing skip-with-reason discipline.

## §3 - Resolution

All 6 concerns addressed. The machine floor (task-lint) ran FIRST and was clean before any judgment family was applied, per TASK-IMP-084. **Score = 10/10.**

---

*End of TASK-IMP-107 audit.*
