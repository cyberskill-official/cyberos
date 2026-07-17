---
task_id: TASK-IMP-103
artefact: coverage-gate@1
phase: testing
generated: 2026-07-17
tests_failed: 0
files_below_90pct: []
ecm_rows_uncovered: []
coverage_measurable: false
coverage_reason: "Touched files are POSIX shell (install.sh, uninstall.sh). coverage-scope ingests c8/lcov; this repo has no shell line-coverage instrumentation. A percentage is NOT reported rather than fabricated."
trace_004_closed: true
---

# TASK-IMP-103 - coverage gate

## Why no coverage percentage

`coverage-scope.mjs TASK-IMP-103 --base main` returns: *no coverage report found (looked for
coverage/coverage-summary.json and coverage/lcov.info)*. The touched files are bash. The repo
instruments JS/Python, not shell.

The honest options were: fabricate a number, or state the gap and gate on what IS measurable.
This artefact does the second. A coverage field that nobody measured is exactly the class of
claim TASK-IMP-086 taught us to distrust, and TASK-IMP-102 was spent making audit claims
verifiable rather than plausible.

**What gates this task instead:** `tests_failed == 0`, TRACE-004 closure (every §1 clause's
cited test passed), and every §3 edge-case row having an arm.

## TRACE-004 closure - every clause's cited test PASSED

| AC | traces_to | cited test | result |
|---|---|---|---|
| AC 1 | #1.1, #1.2, #1.5 | `test_install_lock.sh::t01_concurrent_refuses` | passed |
| AC 2 | #1.3 | `test_install_lock.sh::t02_stale_broken_with_warning` | passed |
| AC 3 | #1.4 | `test_install_lock.sh::t03_fresh_dead_pid_refuses` | passed |
| AC 4 | #1.5 | `test_install_lock.sh::t04_trap_releases_on_signal` | passed |
| AC 5 | #1.6 | `test_install_lock.sh::t05_uninstall_lock_ownership` | passed |

Every §1 clause (1.1-1.6) is cited by at least one AC, and every cited test passed. 5/5 ACs closed.

## Edge-case matrix coverage (§3)

| Row | Arm | result |
|---|---|---|
| Empty lock dir (killed between mkdir and write) | t02/t03 shape - mtime, not the owner file, carries age | covered |
| Shared mount, foreign-host pid | `t07_foreign_host_pid_is_alive` (both arms) | covered |
| pid reuse reads as alive | conservative by construction; survives to threshold, then t02's path | covered by design |
| Read-only `.cyberos/` (not contention) | `t06_non_contention_failure_named` | covered |
| `.cyberos/` absent on first install | `mkdir -p "$CY"` at install.sh:42 precedes the lock | covered |
| Security-class: lock contents never executed | owner file read via `sed` only; no eval, no interpolation into a command | covered by inspection |

`ecm_rows_uncovered: []` - the foreign-host row had NO arm when the gate first ran. It was added
(t07) rather than recorded as a gap, and it immediately found a defect (below).

## Defects found during testing

Two, both in the implementation, both fixed in the code rather than the test:

1. **`stat -f` portability (found by t02 on first run).** GNU `stat -f` is `--file-system` and
   SUCCEEDS on Linux printing `File: ...`, which landed in `_lock_mtime` and detonated in
   arithmetic under `set -u`. Would have broken every Linux install the moment a second one
   contended - the exact scenario the lock exists for. Fixed: `stat -c %Y` first, BSD `-f %m`
   fallback, numeric hard-validation.

2. **Permanent wedge on foreign locks (found by t07).** Liveness was a boolean, and unknown
   collapsed into alive FOREVER - so a lock left by another host on a shared mount could never
   be broken at any age. The spec's §3 said "alive until the threshold expires"; the second half
   was not implemented. Fixed: liveness is tri-state (alive | dead | unknown); unknown expires at
   the threshold. Same reboot-wedge class the batch-4 review found in TASK-IMP-093's lease.

## Suite evidence

```
test_install_lock        7/7   (1.06s, no model, no network)
test_install_hygiene    19/19
test_channels           25/25
test_gate_autodetect     8/8
test_workflow_helpers   14/14
test_task_lint           8/8
test_full_sdp_payload    9/9
test_chain_coverage      7/7
test_pair_parity         6/6
test_rubrics_vendored    2/2
test_check_version_sync 10/10
build.sh OK; sync OK 1.0.0 across 7 artifacts; dist carries the tri-state lock
```

Live-fire, scratch repo: concurrent install refused with **exit 1** (verified unpiped), holder's
lock intact, machine NOT vendored; clean run vendored and left no lock behind.
