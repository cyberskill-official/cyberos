---
task_id: TASK-IMP-104
artefact: coverage-gate@1
phase: testing
generated: 2026-07-17
tests_failed: 0
files_below_90pct: []
ecm_rows_uncovered: []
coverage_measurable: false
coverage_reason: "Touched files are POSIX shell. coverage-scope ingests c8/lcov; this repo has no shell line-coverage instrumentation. No percentage is reported rather than fabricated (same as TASK-IMP-103)."
trace_004_closed: true
---

# TASK-IMP-104 - coverage gate

## TRACE-004 closure - every clause's cited test PASSED

| AC | traces_to | cited test | result |
|---|---|---|---|
| AC 1 | #1.1, #1.3 | `test_install_version_guard.sh::t01_downgrade_refused` | passed |
| AC 2 | #1.4 | `test_install_version_guard.sh::t02_override_records_both` | passed |
| AC 3 | #1.5 | `test_install_version_guard.sh::t03_equal_is_silent` | passed |
| AC 4 | #1.6 | `test_install_version_guard.sh::t04_missing_version_proceeds` | passed |
| AC 5 | #1.2 | `test_install_version_guard.sh::t05_single_comparator` | passed |

All 6 §1 clauses cited by at least one AC; 5/5 ACs closed. AC 5 was specified as a `verify:`
(recorded grep) and is implemented as a real `test:` instead - t05 asserts install defines no
comparator of its own, sources the shared one, and that `ver_lt` is defined in exactly ONE file
repo-wide. A structural claim that can be a test should not be a verify.

## Edge-case matrix coverage (§3)

| Row | Arm | result |
|---|---|---|
| Pre-release / suffixed versions defer to the comparator | `is_ver` rejects non-semver -> "not comparable" path, t04's second arm | covered |
| `.cyberos/VERSION` newer than any release (dev build) | t01 (2.0.0 installed vs 1.0.0 payload IS this case) | covered |
| Damaged machine, VERSION absent but machine present | t04 first arm - proceeds silently, re-vendors (the repair path) | covered |
| Payload with no VERSION (`avail_ver=unknown`) | `is_ver` rejects -> not comparable -> proceeds, named | covered |
| Newer payload proceeds (the normal upgrade) | `t06_newer_proceeds` | covered |
| Security-class: version strings compared, never executed | no eval; `sort`/`grep` only; `$0` and `$root` in the hint are shell-quoted paths | covered by inspection |

## Defects found and fixed during implementation

1. **Two comparators already existed.** `ver_lt`/`is_ver` in `version.sh:77` AND inline in
   `lib/update-check.sh:62`. The spec's §1.2 assumed one. Extracted to `lib/version-compare.sh`;
   both callers now source it. t05 pins `ver_lt` to exactly one definition repo-wide, so a third
   copy reds the suite.

2. **The guard failed OPEN.** A missing comparator skipped the whole check silently. Now exits 1
   naming the malformed payload. Caught by t01 returning 0 where it had to refuse.

3. **`build.sh` never vendored the lib** (explicit copy list), so the guard was in the source and
   absent from the payload - and `update-check.sh` shipped sourcing a file that did not exist
   there. Fixed unconditionally. This is the defect the doctrine already names: a rule correct in
   the source and absent from `dist/` is correct nowhere that matters.

## Suite evidence

```
test_install_version_guard   6/6 (new)
test_install_lock            7/7      test_install_hygiene   19/19
test_channels               25/25     test_full_sdp_payload   9/9
test_check_latest            9/9      test_check_version_sync 10/10
build OK; sync OK 1.0.0 across 7 artifacts; payload carries lib/version-compare.sh
```

Live-fire, scratch repo: older payload over a 2.0.0 machine refused with **exit 1** (verified
unpiped), machine untouched at 2.0.0; `CYBEROS_ALLOW_DOWNGRADE=1` rolled back to 1.0.0 and the
summary recorded `2.0.0 -> 1.0.0 (operator override)`.
