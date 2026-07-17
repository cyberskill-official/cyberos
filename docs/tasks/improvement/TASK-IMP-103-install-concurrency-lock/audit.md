---
task_id: TASK-IMP-103
audited: 2026-07-17
verdict: PASS (after revision)
score_pre_revision: 7/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 6
template: task@1
audit_rubric_version: audit_rubric@2.0
audited_file_sha256: 97456202ef8f4d13
audited_body_sha256_prefix: 2d0a6fff217e482f
machine_floor: task-lint clean (exit 0) - run FIRST per TASK-IMP-084
---

## §1 - Verdict summary

Spec is 88 lines, 6 §1 clauses, 5 ACs, 6 edge cases, 5 test arms. The window it closes is verified on main (install.sh:57-58 rm -rf then cp -R; zero lock references). Scope bounded to the machine's own directory. Passes after 6 findings.

## §2 - Findings (all resolved)

### ISS-001 - Lock refusal conflated with lock-creation failure
A `mkdir` failure on a read-only `.cyberos/` is not contention, but the refusal message could not tell the operator which had happened - sending them to hunt a process that does not exist. Resolved: §3 edge case makes the distinction normative; AC 1 asserts the contention wording.

### ISS-002 - Stale-break threshold hard-coded
A fixed 900 s cannot suit both a laptop and a slow CI mount. Resolved: §1 #1.3 names `CYBEROS_LOCK_STALE_SECS` with a 900 s default - an operator dial, not a constant.

### ISS-003 - Dead pid + fresh lock would break a just-started install
Breaking on a dead pid alone races a lock written microseconds ago. Resolved: §1 #1.4 requires BOTH age threshold and pid death; AC 3 asserts the fresh-dead-pid case refuses.

### ISS-004 - Cross-host pid liveness is undecidable on a shared mount
`kill -0` against another machine's pid is meaningless, and reading it as dead breaks a live install. Resolved: §3 edge case adopts TASK-IMP-093's lease convention - foreign/unreadable is alive until the threshold.

### ISS-005 - Refusal path could release a lock it does not own
A naive `trap` releases on every exit including refusal, deleting the holder's lock. Resolved: §1 #1.5 forbids it; AC 4 covers the signal path.

### ISS-006 - Uninstall could delete a foreign lock
Symmetric to the batch-5 `.cyberos-owned` finding on shared skills. Resolved: §1 #1.6 scopes removal to a lock uninstall owns; AC 5 asserts both arms.

## §3 - Resolution

All 6 concerns addressed. The machine floor (task-lint) ran FIRST and was clean before any judgment family was applied, per TASK-IMP-084. **Score = 10/10.**

---

*End of TASK-IMP-103 audit.*
