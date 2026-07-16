---
artefact: edge-case-matrix@1
task_id: TASK-IMP-098
total_rows: 12
created: 2026-07-17
verdict: pass (edge-case-matrix-audit: every category >=1 row, covered-by names real test functions, SECURITY rows point at code+test, DEGRADATION rows carry detection+recovery)
---
# Edge-case matrix - TASK-IMP-098

All test functions live in tools/install/tests/test_coverage_scope.sh.

| # | category | trigger | expected behavior | covered by |
|---|---|---|---|---|
| 1 | null/empty | touched file absent from the coverage report (doc, config, or untested source) | row emitted with `no-coverage-data` in both pct and status columns - visible, never silently dropped; counted in a notes line; NOT placed in files_below_90pct (no percentage to compare - the author skill judges it, and the table row keeps it visible) | t02_skeleton_from_fixture (docs/notes.md row, byte-compared) |
| 2 | null/empty | empty diff (no files touched in the range) | table emits a single `(no files touched in the range)` placeholder row rather than a bare header; frontmatter files_below_90pct: [] | code-pinned (buildSkeleton rows.length===0 branch); not fixture-tested - the placeholder is presentation, the empty-list frontmatter is the load-bearing half |
| 3 | null/empty | lcov record with LF:0 (file with zero instrumentable lines) | pct = 100 (matches istanbul's treatment of empty files), so an empty file never lands in files_below_90pct | code-pinned (ingestLcov flush: `lf === 0 ? 100 : ...`), documented in --help |
| 4 | bounds | percentage exactly at 90 | NOT below threshold - strict less-than, matching the gate's wording "below 90"; status column says `ok` | t02_skeleton_from_fixture (src/exact.js: 90 -> ok, absent from files_below_90pct, byte-compared) |
| 5 | malformed | report recognized by name but content invalid (coverage-summary.json without a 'total' key; lcov.info with no SF record) | loud exit 2 naming the defect - shape-by-name never implies content trust | t03_unknown_report_refused (no-total arm); lcov no-SF arm code-pinned (ingestLcov records===0) |
| 6 | malformed | any other report shape (clover.xml, cobertura, raw json) | refused BY NAME with exit 4, message names the file and the exactly-two supported shapes; nothing written (--out target absent, stdout empty) | t03_unknown_report_refused |
| 7 | concurrency/order | multiple commits name the task id + "implementing" (route-back re-entry, the corpus reality) | the EARLIEST match wins (git log is newest-first; last line taken) and the ambiguity is REPORTED in the skeleton's range note naming the count - the operator sees the choice, --base overrides it | t01_base_resolution (two-flip fixture: earliest sha asserted in the Range line, exact note text asserted) |
| 8 | concurrency/order | two runs on identical repo state + report | byte-identical skeleton (no clock, no randomness, table in bytewise path order, deletions sorted) | t02_skeleton_from_fixture (rerun cmp arm) |
| 9 | SECURITY | --coverage or --out pointing outside the repo root (path traversal / exfil target) | refused with exit 2 naming the flag - reads and the sole write stay inside the repo root (spec §3 security class) | code-pinned (relUnderRoot null checks in ingestCoverage + main's --out guard); refusal path shares the loud-refusal shape t03 proves |
| 10 | SECURITY | hostile commit subjects or report content trying to steer output (subject with markdown, report keys outside the repo) | subjects are quoted data in one provenance line (never executed, never structural); report keys that do not normalize under the repo root are DROPPED from the pct map (unmatchable, surfaces as no-coverage-data rows - visible); git is invoked read-only with pinned argv (no shell interpolation) | code-pinned (spawnSync argv array, relUnderRoot filter); t01 asserts subjects round-trip as quoted strings |
| 11 | DEGRADATION | no --base and no entry-flip commit (fresh repo, squashed history, foreign convention) | detection: exit 3 (distinct from usage 2) with "pass --base ... never guesses a range"; recovery: caller re-runs with --base - the range is a human decision the tool refuses to fake | t01_base_resolution (no-match arm, exit code + message + no-skeleton asserted) |
| 12 | DEGRADATION | file deleted in the range; tool dropped from the payload | detection: deletion excluded from the coverage table but NAMED in the notes line (t02 byte-compares it); t04 gates payload presence + byte-parity + a live vendored run so a build.sh regression fails the suite the day it lands. recovery: the guarded copy is a one-line re-vendor; re-install lays the tool back down | t02_skeleton_from_fixture (src/gone.js note), t04_payload_vendored |

Documented-by-design: the sachviet batch-1 reproduction (AC 5, guardrail metric) is consumer-repo evidence the fixture suite cannot carry - it is parent-run and recorded in the gate log per the spec's own verify wording. Coverage keys carrying pct values that are not numbers (istanbul "Unknown") degrade to no-coverage-data rows rather than NaN arithmetic (ingestIstanbul type guard). The tool never runs the coverage command itself (run-gates' job, spec Out of scope) - a stale report is the operator's input, not this tool's lie; the report path is printed in the skeleton so the gate reviewer can see exactly which bytes fed the table.
