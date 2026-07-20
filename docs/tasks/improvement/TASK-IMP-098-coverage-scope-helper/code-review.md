# TASK-IMP-098 — code review packet

Files under review: new `tools/install/docs-tools/coverage-scope.mjs` (338 lines, the task-diff -> per-file-coverage skeleton emitter) and `tools/install/tests/test_coverage_scope.sh` (191 lines, the gating suite), modified `tools/install/build.sh` (+2 lines, guarded vendor copy — spec-declared in `modified_files`). Suite state at review: test_coverage_scope 4/4, 0 failed (~3 s including the payload build). build.sh is SHARED with batch sibling TASK-IMP-093 (same agent, serial order per the batch plan); after this task's line landed, the sibling suites were re-run green (test_memory_append 4/4, test_workflow_helpers 13/13 — gate-log E4), so the shared-file edit regressed nothing.

## §1 clause → proof

| Clause | Requirement | Proof |
|---|---|---|
| 1.1 | base resolution: explicit `--base` first; else the commit whose subject names `<task-id>` entering implementing; else fail loudly demanding --base (never guess a range) | `t01_base_resolution`, three arms — (a) `--base v-seed` wins over an EXISTING entry-flip commit: Range line carries the seed sha and the `base via --base 'v-seed'` provenance, and no scan note appears; (b) subject scan over a fixture with TWO subjects naming the task + "implementing" resolves the EARLIEST flip (sha asserted in the Range line), provenance quotes the flip subject, and the exact ambiguity note ("2 commit subjects name TASK-B-002 + \"implementing\"; the EARLIEST was used as base - pass --base to override.") lands in the range note per §3's edge; (c) no match → exit 3 (distinct from usage 2) with "pass --base"/"never guesses", and NO skeleton is emitted. Scan implementation is literally `git log --format=%H %s` (`resolveBase`), earliest = last emitted line (log is newest-first) |
| 1.2 | touched = `git diff --name-only <base>...HEAD` filtered to files existing at HEAD; deletions excluded from the table but listed in the notes | `t02_skeleton_from_fixture` — the fixture modifies src/low.js + src/exact.js, adds docs/notes.md, DELETES src/gone.js; the byte-compared skeleton shows exactly the three HEAD-present files in the table and `- deleted in range (excluded from the table per #1.2): src/gone.js` in the notes. Implementation: three-dot diff + `git ls-tree -r --name-only HEAD` membership (`touchedFiles`), both read-only |
| 1.3 | ingest c8/istanbul coverage-summary.json and lcov.info; refuse any other input by name with non-zero exit | `t02` runs BOTH shapes against the same fixture — the istanbul arm uses ABSOLUTE path keys (what c8 emits; normalized repo-relative) and the lcov arm uses MIXED relative+absolute SF paths; both emit byte-identical tables (only the Report line differs, naming each shape). `t03_unknown_report_refused` — clover.xml → exit 4 naming 'clover.xml' AND both supported names, no skeleton, no --out file; a coverage-summary.json with no 'total' key → exit 2 (shape-by-name never implies content trust). lcov pct math LF/LH → round-2, LF:0=100 (`ingestLcov`) |
| 1.4 | output = coverage-gate@1 skeleton: frontmatter (tests_failed TODO, files_below_90pct computed from the 90 threshold, ecm_rows_uncovered TODO), per-file table with percentage or `no-coverage-data`, base/HEAD range recorded | `t02`'s expected-BYTES compare pins the whole artefact: frontmatter `artefact: coverage-gate@1 / task / phase: testing / tests_failed: TODO / files_below_90pct: [src/low.js] / ecm_rows_uncovered: TODO`; Range line with full base...HEAD shas + provenance; the table holds a below-90 row (85.71 → `below-90`), an exactly-90 row NOT below (90 → `ok`, strict `p < 90` in `buildSkeleton` — §3's edge), and a `no-coverage-data` row for the touched doc; notes carry the deletion, the no-data count, and the author-skill TODO line. `--out` writes the SAME bytes inside the repo with stdout kept clean, and a rerun is byte-identical (determinism arm) |
| 1.5 | build.sh vendors the tool (guarded copy); suite gates the payload copy against a scratch build | build.sh:180-181 — one comment + one `[ -f ... ] && cp` line in the docs-tools block, byte-idiom of the four sibling lines above it. `t04_payload_vendored` builds into a scratch dir, asserts presence, `cmp` byte-parity with the source, runs the vendored copy's --help AND a live vendored run against a scratch fixture repo |
| 1.6 | suite lands at tools/install/tests/test_coverage_scope.sh (run_all glob discovery) | the file exists at exactly that path; scripts/tests/run_all.sh:43's `tools/install/tests/test_*.sh` glob discovers it with zero wiring (gate-log E2) |

## §3 edge cases

Touched-but-unreported file: `no-coverage-data` row, byte-compared (t02). Deletion: excluded + named (t02). Multiple entry-flip subjects: earliest wins + reported (t01). Exactly 90: not below, strict less-than (t02, `ok` status + absent from the list). Security class: git via argv-array spawnSync, read-only verbs only; --coverage and --out refused outside the repo root (`relUnderRoot` guards); no network, no eval; report keys that do not normalize under the root drop to visible no-coverage-data.

## Acceptance criteria

AC 1 `t01_base_resolution` ok · AC 2 `t02_skeleton_from_fixture` ok · AC 3 `t03_unknown_report_refused` ok · AC 4 `t04_payload_vendored` ok (+ glob half, gate-log E2) · AC 5 (sachviet batch-1 reproduction) — PARENT-RUN, recorded in the gate log by the parent per the spec's verify wording; not claimed here. Suite 4/4.

## Diff size

Two new files: `tools/install/docs-tools/coverage-scope.mjs` (338 lines, self-contained ESM, node stdlib only — git via child_process argv arrays) and `tools/install/tests/test_coverage_scope.sh` (191 lines, executable; fixture git repos built under mktemp in-test, isolated from user/system git config, never touching the enclosing repo's git state). One modified file: `tools/install/build.sh` +2/−0. No dependency added anywhere. `dist/` untouched here — rebuild + version-sync before commit are the batch parent's step per payload-sync doctrine.

## Design disclosures

1. Exit-code split: base-unresolvable is exit 3 (not 2) so a wrapper can distinguish "prompt the operator for --base" from plain usage errors; unsupported-report-by-name is exit 4. Both documented in --help alongside the docs-tools exit-code discipline.
2. files_below_90pct excludes no-coverage-data rows: a file without a percentage cannot be claimed "below 90"; it stays VISIBLE in the table and in a count note, and judgment stays with coverage-gate-author (spec Out of scope). The alternative — listing unmeasured files as failures — would fake a measurement the input never made.
3. The skeleton is markdown with coverage-gate@1 frontmatter (the "skeleton ready for the author skill to complete" the spec's Proposed Solution names), not the full §2 YAML of the author skill's output — tests_failed/ecm_rows_uncovered/raw_terminal are the author's judgment surface and are emitted as literal TODO markers.
4. lcov LF:0 counts as pct 100, matching istanbul's treatment of empty files — documented in --help and the header so the convention is inspectable.

## Verdict

| Check | State |
|---|---|
| §1 clauses 1.1–1.6 | each proven above by a named test or pinned line |
| Primary metric (fixture skeleton matches expected bytes, suite-asserted) | pass (t02, both shapes + --out + rerun) |
| Guardrail metric (sachviet reproduction) | parent-run per spec AC 5; gate log carries the placeholder |
| §3 edge cases (no-data row, deletion, ambiguity, exactly-90, security class) | each covered (t01/t02/t03 + code-pinned) |
| Determinism contract (no clock/randomness, byte-identical reruns) | pass (t02 rerun arm) |
| Shared build.sh discipline (sibling suites green after the edit) | pass (gate-log E4) |
| Invariants (read-only git, repo-root confinement, node stdlib, HITL) | intact |

HALT: review acceptance (reviewing -> ready_to_test) is a human gate.
