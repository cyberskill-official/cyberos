---
# ── Identity ─────────────────────────────────────────────────────────
name: coverage-gate-author
description: >-
  Test coverage gate (testing → done) — run the project's test suite + measure coverage on the files touched by the current task (per the `git diff` since the task's `implementing` status was set). Emits a coverage-gate@1 artefact: raw terminal output of the coverage tool, per-file coverage %, list of files below 90 %, list of edge-case-matrix rows without a corresponding test. Used by `chief-technology-officer/ship-tasks` during the `testing` phase to gate the `testing → done` transition (per `modules/skill/contracts/task/STATUS-REFERENCE.md` §1.1). Use when user asks to "draft a coverage gate" or "create the coverage gate". Do NOT use for "audit existing coverage gate" (use coverage-gate-audit instead). Do NOT use for spec correctness — that is `task-audit`'s job, run during the `draft → ready_to_implement` transition; the two gates are deliberately separated so spec correctness can be verified before any implementation work begins.
license: Apache-2.0
metadata:
  version: 1.0.0
  module: skill
  stage: e
  cyberos-template: coverage-gate@1
  cyberos-rubric-target: coverage_gate_rubric@1.0

allowed_memory_scopes:
  read:
    - project:*
    - module:*
  write:
    - project:task/{task_id}/coverage-gate
audit:
  row_kind: coverage_gate_authored
  required_fields: [task_id, files_touched, files_below_90pct, total_tests_run, tests_failed, ecm_rows_uncovered]

inputs:
  - { name: task,                 format: task@1,            required: true }
  - { name: edge_case_matrix,   format: edge-case-matrix@1,           required: true }
outputs:
  - { name: report, format: coverage-gate@1 }

triggers:
  - workflow `chief-technology-officer/ship-tasks` testing phase (step 23 — first author call; step 24 audit)
blockers:
  - "no coverage tool configured in repo — must be resolved first"
  - "test framework is broken — diagnose before running this skill"

# ── Untrusted-content discipline ─────────────────────────────────────
untrusted_inputs:
  wrap_in_marker: "untrusted_content"
  injection_scan: required
  on_marker_hit: surface_to_human
---

# coverage-gate-author

## 1. What it does

1. Reads the task's `building` status timestamp from BACKLOG.md.
2. `git diff --name-only <building_ts>..HEAD` → the touched-files set.
3. Picks the right coverage tool per language (rust: `cargo tarpaulin` or `cargo llvm-cov`; python: `pytest --cov`; node: `vitest --coverage`).
4. Runs the full test suite.
5. Reports per-file coverage for every file in the touched set.
6. Cross-references the edge-case matrix: every row's `planned_test` must exist + must have passed.
7. **`type: bug` only** — runs the regression proof (§4). Reads `type` from the task's frontmatter; skips entirely for any other type.

## 2. Output schema

```yaml
# coverage-gate@1
task_id: task-<MODULE>-<NNN>
generated_at: <ISO-8601>
language: rust | python | typescript | mixed
tool: tarpaulin | llvm-cov | pytest-cov | vitest | ...
total_tests_run: <int>
tests_failed: <int>
overall_coverage_pct: <float>
files_touched:
  - { path: "...", coverage_pct: 92.3, lines_covered: 314, lines_total: 340 }
files_below_90pct: [...]
ecm_rows_uncovered: [ECM-003, ECM-007, ...]
raw_terminal: |
  <full stdout/stderr of the coverage run, untruncated>

# ── type: bug only (absent for every other type) ──────────────────────
regression:
  test: <path>::<testname>          # from the task's `regression_test` field
  broken_commit: <sha>              # `first_bad_commit`, or HEAD~1 when null
  red_at_broken: true | false       # REGRESSION-002 — MUST be true
  green_at_head: true | false       # REGRESSION-001 — MUST be true
  raw_terminal_red: |               # REGRESSION-003 — both runs, untruncated
    <stdout/stderr of the run at broken_commit — MUST show the failure>
  raw_terminal_green: |
    <stdout/stderr of the run at HEAD>
  exempt_reason: <string> | null    # REGRESSION-004 — set iff regression_test is null
```

## 3. Pass criterion

- `tests_failed == 0`
- `files_below_90pct` is empty
- `ecm_rows_uncovered` is empty
- **`type: bug`**: `regression.red_at_broken == true` AND `regression.green_at_head == true`

If any of those fails → trip the workflow's debugging-cycle (step 15).

## 4. The regression proof (`type: bug`)

Skip unless the task's frontmatter says `type: bug`.

A test written after a fix, against the fixed code, passes — and proves nothing, because it never saw the bug. The only way to know a regression test tests the regression is to watch it go red on the broken commit. So run it there:

```bash
broken="${first_bad_commit:-HEAD~1}"
git worktree add --detach /tmp/regression-proof "$broken"

# Carry ONLY the new test file across. The fix must NOT come with it — that is the
# whole point. Copying the whole tree, or cherry-picking the fix commit, makes the
# test pass at the broken commit and silently converts this gate into a rubber stamp.
git show "HEAD:${regression_test%%::*}" > "/tmp/regression-proof/${regression_test%%::*}"

( cd /tmp/regression-proof && <runner> "$regression_test" )   # MUST exit non-zero
( cd "$repo"               && <runner> "$regression_test" )   # MUST exit zero

git worktree remove --force /tmp/regression-proof
```

Capture **both** terminals into `regression.raw_terminal_red` / `_green`. An assertion without its evidence is not evidence — same rule the rest of this gate already lives by.

### Failure modes worth naming

- **Green at the broken commit** → the test does not test the bug. Do not "fix" it by moving on; the diagnosis is wrong or the test is aimed at the wrong thing.
- **The test file does not exist at HEAD** → BUG-011 should have caught this at `draft`. Something authored a bug task without a regression test.
- **The broken commit will not build** → common, and not a failure. If the runtime cannot even start, the test *did* fail there. Record the build error as the red terminal and note it; do not silently pass.
- **`regression_test: null`** → REGRESSION-004: a non-empty operator-signed `no_regression_test_reason` must be present, and it rides in the audit row forever. Making the exemption possible but loud is deliberate. A gate nobody can ever bypass gets bypassed by deleting the gate.

## 5. Contract files

- rule text: `modules/skill/contracts/task/rubrics/bug.md` §10.3
- gate mapping: `modules/skill/coverage-gate-audit/RUBRIC.md`

---

*End of coverage-gate-author SKILL.md.*

## Contract files (TASK-SKILL-118)

This pair is at full contract parity: `PIPELINE.md` (chain binding + HALT points), `INVARIANTS.md`, `envelopes/` (I/O schemas), `references/FAILURE_MODES.md`, `acceptance/README.md`. SKILL.md remains the normative prose; the files encode it.

## Threshold override (TASK-CUO-207)

The per-file coverage floor is `CYBEROS_COVERAGE_THRESHOLD` when set (exported by `run-gates.sh` from `.cyberos/config.yaml` `coverage_threshold`), defaulting to 90. The audit rubric's COVERAGE_THRESHOLD constant names the same hook.
