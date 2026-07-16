---
artefact: repo-context-map@1
task_id: TASK-IMP-098
created: 2026-07-17
verdict: pass (repo-context-map-audit: patterns pinned to file:line, outside-domain count stated, ADR trigger evaluated)
---
# Repo context map - TASK-IMP-098

## Baseline patterns the new code must follow
- docs-tools convention: node stdlib only, ESM, single self-contained .mjs, whole-file doc comment, --help with an exit-code table, loud refusals - pinned_in: tools/install/docs-tools/backlog-mutate.mjs:1-52, ship-manifest.mjs:1-52, and batch sibling memory-append.mjs (TASK-IMP-093)
- payload vendoring shape: per-file `[ -f ... ] && cp` guarded lines in build.sh's docs-tools block - pinned_in: tools/install/build.sh:174-181; the new line lands at build.sh:180-181 in the identical idiom, directly under 093's line (shared file, same agent, serial order per the batch plan)
- determinism doctrine: no clock, no randomness, byte-identical output on identical input - pinned_in: backlog-mutate.mjs:53 ("identical input + identical args = byte-identical result") and task-audit's `deterministic_drift` signal; this tool's t02 is an expected-BYTES compare
- test harness shape: `set -uo pipefail`, here/repo resolution, mktemp TMP + trap, ok/fail counters, `pass=N fail=N`, `want` filter - pinned_in: tools/install/tests/test_workflow_helpers.sh:54-63
- fixture-repo discipline: suites that need git history build their OWN scratch repos under mktemp (git init -b main + synthetic commits) and never mutate the enclosing repo - the sibling suites' scratch-install pattern (test_workflow_helpers.sh t08) extended to history fixtures; isolated via GIT_CONFIG_GLOBAL/SYSTEM=/dev/null + per-call -c identity
- suite discovery: scripts/tests/run_all.sh:43 globs `tools/install/tests/test_*.sh` - zero wiring (AC 4's glob half)

## The contracts this tool serves (source of truth for the emitted shape)
- coverage-gate@1: modules/skill/coverage-gate-author/SKILL.md §2 output schema - task_id, tests_failed, files_below_90pct, ecm_rows_uncovered, per-file rows; the skill's own step 2 is "`git diff --name-only <building_ts>..HEAD` -> the touched-files set" - exactly the mechanical half this tool owns. The skeleton emits the judgment fields as literal TODO markers (tests_failed, ecm_rows_uncovered, raw_terminal) - the author skill completes them (spec Out of scope)
- ship-tasks coverage doctrine: 90 percent on touched files; "below 90" is STRICT less-than (spec §3 edge: exactly 90 is NOT below) - encoded once in buildSkeleton (`p < 90`)
- entry-flip subject convention: batch commits flip a task into implementing and say so in the subject (e.g. "TASK-X: enter implementing") - base resolution scans `git log --format=%H %s` for subject lines naming the task id + "implementing", EARLIEST match wins (touched-since-implementing semantics); ambiguity (2+ matches) is noted in the skeleton's range note, and no match fails loudly (exit 3) demanding --base - spec §1 #1.1 "never guess a range"
- report shapes: c8/istanbul coverage-summary.json (per-file objects with lines.pct; absolute path keys normalized repo-relative) and lcov.info (LF/LH per SF record -> pct, 2-decimal, LF:0 counts as 100 matching istanbul's empty-file treatment); ANY other input refused BY NAME with exit 4 - spec Alternatives ("refusing others loudly beats guessing quietly")

## Schemas / interfaces in scope
- CLI: `node coverage-scope.mjs <task-id> [--base <ref>] [--coverage <file>] [--repo <root>] [--out <file>]`
- exit codes: 0 ok; 2 usage / not-a-repo / invalid report content / path outside repo root; 3 base unresolvable (distinct, so a wrapper can prompt for --base); 4 unsupported report refused by name
- touched set: `git diff --name-only <base>...HEAD` (three-dot: merge-base to HEAD, the task's own side) filtered to files existing at HEAD via `git ls-tree -r --name-only HEAD`; deletions excluded from the table but NAMED in the notes (§1 #1.2)
- git surface is READ-ONLY: log / diff / ls-tree / rev-parse only; the sole write is --out, which must resolve inside the repo root (spec §3 security class; --coverage likewise)

## Files outside the immediate domain (tools/install/docs-tools/ + tools/install/tests/)
1. tools/install/build.sh (modified, +2 lines - the guarded vendor copy; spec-declared in `modified_files`, gated by t04)

files_outside_immediate_domain: 1 (<= 3 -> no ADR trigger; spec-declared, sibling-idiom lines in the shared block).

## Blast radius
file_count: 3 (2 new: coverage-scope.mjs 338 lines + test_coverage_scope.sh 191 lines; 1 modified: build.sh +2) | module_count: 2 (tools/install docs-tools+tests, tools/install build) | cross_module_edges: suite -> build.sh (t04); tool -> coverage-gate-author contract at AUTHORING time only (skeleton shape compiled in; no runtime read of SKILL.md)
module_placement_warning: null (spec declares `service: tools/install/docs-tools`; both new files sit where §1 #1.5/#1.6 fix them)
Behavioral radius: zero on existing production paths - runs only when invoked; run-gates stays verbatim-runner (spec Alternatives rejected teaching run-gates to scope); ship-manifest untouched (different lifecycle, spec Alternatives). Consumer repos inherit the tool through the payload on their next install.
