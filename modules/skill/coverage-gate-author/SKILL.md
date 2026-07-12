---
# ── Identity ─────────────────────────────────────────────────────────
name: coverage-gate-author
description: >-
  Test coverage gate (testing → done) — run the project's test suite + measure coverage on the files touched by the current FR (per the `git diff` since the FR's `implementing` status was set). Emits a coverage-gate@1 artefact: raw terminal output of the coverage tool, per-file coverage %, list of files below 90 %, list of edge-case-matrix rows without a corresponding test. Used by `chief-technology-officer/ship-feature-requests` during the `testing` phase to gate the `testing → done` transition (per `modules/skill/contracts/feature-request/STATUS-REFERENCE.md` §1.1). Use when user asks to "draft a coverage gate" or "create the coverage gate". Do NOT use for "audit existing coverage gate" (use coverage-gate-audit instead). Do NOT use for spec correctness — that is `feature-request-audit`'s job, run during the `draft → ready_to_implement` transition; the two gates are deliberately separated so spec correctness can be verified before any implementation work begins.
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
    - project:fr/{fr_id}/coverage-gate
audit:
  row_kind: coverage_gate_authored
  required_fields: [fr_id, files_touched, files_below_90pct, total_tests_run, tests_failed, ecm_rows_uncovered]

inputs:
  - { name: fr,                 format: feature-request@1,            required: true }
  - { name: edge_case_matrix,   format: edge-case-matrix@1,           required: true }
outputs:
  - { name: report, format: coverage-gate@1 }

triggers:
  - workflow `chief-technology-officer/ship-feature-requests` testing phase (step 23 — first author call; step 24 audit)
blockers:
  - "no coverage tool configured in repo — must be resolved first"
  - "test framework is broken — diagnose before running this skill"
---

# coverage-gate-author

## 1. What it does

1. Reads the FR's `building` status timestamp from BACKLOG.md.
2. `git diff --name-only <building_ts>..HEAD` → the touched-files set.
3. Picks the right coverage tool per language (rust: `cargo tarpaulin`
   or `cargo llvm-cov`; python: `pytest --cov`; node: `vitest --coverage`).
4. Runs the full test suite.
5. Reports per-file coverage for every file in the touched set.
6. Cross-references the edge-case matrix: every row's `planned_test`
   must exist + must have passed.

## 2. Output schema

```yaml
# coverage-gate@1
fr_id: FR-<MODULE>-<NNN>
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
```

## 3. Pass criterion

- `tests_failed == 0`
- `files_below_90pct` is empty
- `ecm_rows_uncovered` is empty

If any of those fails → trip the workflow's debugging-cycle (step 15).

---

*End of coverage-gate-author SKILL.md.*

## Contract files (FR-SKILL-118)

This pair is at full contract parity: `PIPELINE.md` (chain binding + HALT points), `INVARIANTS.md`, `envelopes/` (I/O schemas), `references/FAILURE_MODES.md`, `acceptance/README.md`. SKILL.md remains the normative prose; the files encode it.

## Threshold override (FR-CUO-207)

The per-file coverage floor is `CYBEROS_COVERAGE_THRESHOLD` when set (exported by
`run-gates.sh` from `.cyberos/config.yaml` `coverage_threshold`), defaulting to 90.
The audit rubric's COVERAGE_THRESHOLD constant names the same hook.
