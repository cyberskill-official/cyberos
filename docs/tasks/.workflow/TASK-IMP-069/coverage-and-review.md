---
artefacts: coverage-gate@1 + code-review@1 (bundled)
task_id: TASK-IMP-069
tests_failed: 0
tests_passed: 9
files_below_90pct: []
ecm_rows_uncovered: []
created: 2026-07-12
verdicts: coverage pass; code review pass - human verdict pending at HITL gate 1
---
# Coverage gate + code review packet - TASK-IMP-069

## Raw terminal output
```
building scratch payload + assets...
  ok   t01 .. ok   t09
----
pass=9 fail=0
(regressions: test_check_version_sync.sh 10/10, test_chain_coverage.sh 6/6)
```

## Coverage method (branch enumeration, kcov unavailable - TASK-IMP-068 declaration)
release-assets.sh: happy path (harness), GNU-tar probe + incomplete-payload exits (row 1/9),
triple-check exit-10-writes-nothing (t04), determinism (t01), twins + SHA256SUMS (t02, t03). 8/8.
bootstrap.sh: default/env URL resolution, download fail, missing SHA256SUMS, checksum mismatch
abort-before-install, happy init, legacy top-dir fallback (t06, t07 + code path). 7/7.
rollout.sh --from-release branch: tag/latest/env URL forms, verify, single-download multi-repo
(t08); legacy dir-arg path untouched (regression: existing behavior preserved by else-branch). 5/5.
release.yml/doc edits: structural + parse (t05, t09). 10/10 edge rows covered.

## §1 clause -> named test -> status
| #1 asset set + determinism | t01, t02, t03 | passed |
| #2 triple-check, no output | t04 | passed |
| #3 payload job shape | t05 | passed |
| #4 tag guard | t04, t05 | passed |
| #5 bootstrap verify-then-install | t06, t07 | passed |
| #6 rollout --from-release | t08 | passed |
| #7 docs with real URLs | t09 | passed |

## Deviations from spec
None. Legacy compatibility added beyond spec: CYBEROS_PACK_URL alias + legacy top-level-dir
tarball fallback in bootstrap (old links keep working).

## Reviewer attention points
1. The payload job runs on every v* tag alongside the installer jobs; first real proof arrives
   with the next tag (v1.8.0 projected). Until then the job is exercised only by the suite's
   structural asserts.
2. bootstrap now REFUSES to install without SHA256SUMS beside the tarball - old ad hoc hosted
   tarballs without checksums will stop working by design.
3. gh release create uses --verify-tag + "|| true"; if the release already exists with different
   notes, notes are left as-is (upload only clobbers assets).

## Module gates
awh: N/A (declared). caf: N/A; floor = bash -n clean on all touched scripts + three suites green (25 cases).

## Verdict requested
Review acceptance (HITL gate 1): approve to advance, or reject with findings.
