---
id: TASK-IMP-012
title: "Coverage measurement and ratchet"
eu_ai_act_risk_class: not_ai
ai_authorship: generated_then_reviewed
client_visible: false
type: improvement
created_at: 2026-07-08T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: improvement
priority: p1
status: done
phase: Wave 2 - measure and evaluate
refs: [R11]
depends_on: []
created: 2026-07-08
verify: T
owner: Stephen Cheng (CTO)
language: node + bash
service: tools/install/
new_files:
  - tools/install/docs-tools/coverage-ratchet.mjs
  - tools/install/coverage-baseline.json
  - tools/install/tests/test_coverage_ratchet.sh
modified_files:
  - tools/install/build.sh
  - CHANGELOG.md
source_pages:
  - "tools/install/docs-tools/coverage-scope.mjs (TASK-IMP-098 — per-task coverage skeleton; not a ratchet)"
  - "tools/install/install.sh COVERAGE_CMD autodetect (cargo llvm-cov / c8 / coverage.py) — platform-heavy; not the 1.x ratchet target"
source_decisions:
  - "2026-07-25 batch/12c: NOT full cargo llvm-cov. Ratchet the CyberOS 1.x install suite surface (shell/Node under tools/install) via test-touch coverage of scripts."
  - "2026-07-25: fail only on regression vs committed baseline; --write-baseline is explicit opt-in to raise the floor."
---

# TASK-IMP-012: Coverage measurement and ratchet

## Summary

CyberOS 1.x needs a coverage ratchet for the install/docs-tools surface — not a monorepo-wide `cargo llvm-cov`. This task lands a deterministic install-suite coverage metric, a committed baseline, and a ratchet that fails only when coverage regresses.

## Problem

`coverage-scope.mjs` maps a task diff to per-file coverage for authoring, but nothing ratchets the install suite itself. Autodetected `COVERAGE_CMD` may claim cargo llvm-cov on this monorepo even when the payload work under test is shell/Node. Without a baseline, coverage gates either do nothing useful or fail on absolute thresholds that are wrong for shell tooling.

## Proposed Solution

Add `tools/install/docs-tools/coverage-ratchet.mjs` that measures **install-script test-touch coverage**: the fraction of `tools/install` scripts (`.sh` / `.mjs` under gates/, docs-tools/, and top-level, excluding `tests/`) whose basename is referenced by at least one file under `tools/install/tests/`. Compare against committed `tools/install/coverage-baseline.json`. Exit non-zero only when current pct is strictly below baseline pct. `--write-baseline` updates the baseline after a measured improvement. Vendor the tool via `build.sh`.

## Alternatives Considered

- **Wire cargo llvm-cov as the ratchet.** Rejected: mission scopes 1.x away from platform-wide llvm-cov.
- **Absolute 90% threshold only.** Rejected: shell suites rarely have line instrumentation; ratchet-from-baseline is the honest floor.
- **Extend coverage-scope.mjs in place.** Rejected: scope tool is task-diff oriented; ratchet is suite-floor oriented. Keep companions, share docs-tools home.

## Success Metrics

- Primary: ratchet fails on constructed regression, passes at baseline, `--write-baseline` raises floor.
- Guardrail: `run_all.sh` stays green; missing baseline fails closed (exit 2) rather than inventing 0.

## Scope

In scope: coverage-ratchet.mjs, coverage-baseline.json, tests, build.sh vendor line, CHANGELOG.

### Out of scope / Non-Goals

- Instrumenting bash with kcov/c8.
- Replacing COVERAGE_CMD autodetect for consumer repos.
- Changing coverage-scope.mjs CLI contract.

## Dependencies

None. Soft companion: coverage-scope.mjs (IMP-098) remains the per-task authoring tool.

## AI Authorship Disclosure

- **Tools used:** Cursor agent, batch/12c.
- **Human review:** session operator Stephen Cheng.

## §1 - Description (normative)

1. `coverage-ratchet.mjs` MUST compute install-suite test-touch coverage over the closed path set: `tools/install/*.{sh,mjs}`, `tools/install/gates/*.{sh,mjs}`, `tools/install/docs-tools/*.{sh,mjs}` (files only; `tests/` excluded).
2. A script counts as covered when its basename appears as a substring in any `tools/install/tests/test_*.sh` (or other `test_*` file under that directory).
3. Output MUST include schema `coverage-ratchet@1`, `pct` (one decimal), `covered`, `total`, and sorted uncovered basenames.
4. Default mode MUST compare `pct` to `tools/install/coverage-baseline.json` (or `--baseline <path>`). If baseline missing → exit 2. If `pct < baseline.pct` → exit 1 (regression). If `pct >= baseline.pct` → exit 0.
5. `--write-baseline` MUST rewrite the baseline file from the current measurement and exit 0 (does not require prior baseline).
6. `--json` MUST print the measurement object to stdout; human summary may go to stderr.
7. `build.sh` MUST vendor `coverage-ratchet.mjs` into the payload `docs-tools/` when present.
8. Suite `tools/install/tests/test_coverage_ratchet.sh` MUST cover regression / pass / write-baseline / missing-baseline.

## Acceptance criteria

- [x] AC1: Measurement against a fixture with known covered/uncovered counts matches expected pct. (t01)
- [x] AC2: pct below baseline → exit 1. (t02)
- [x] AC3: pct equal or above baseline → exit 0. (t03)
- [x] AC4: missing baseline → exit 2. (t04)
- [x] AC5: `--write-baseline` creates/updates baseline to current pct. (t05)
- [x] AC6: payload build copies coverage-ratchet.mjs. (t06)

## Test plan

`bash tools/install/tests/test_coverage_ratchet.sh`
