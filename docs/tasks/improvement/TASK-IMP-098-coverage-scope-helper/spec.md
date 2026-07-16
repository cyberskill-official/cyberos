---
id: TASK-IMP-098
title: coverage-scope helper maps a task diff to per-file coverage
template: task@1
type: improvement
module: improvement
status: implementing
priority: p2
author: "@stephencheng"
department: engineering
created_at: 2026-07-17T08:05:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-085]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 release"
owner: Stephen Cheng (CTO)
created: 2026-07-17
shipped: null
memory_chain_hash: null
effort_hours: 4
service: tools/install/docs-tools
new_files:
  - tools/install/docs-tools/coverage-scope.mjs
  - tools/install/tests/test_coverage_scope.sh
modified_files:
  - tools/install/build.sh
source_pages:
  - "IMPROVEMENT_HANDOFF.md IMP-14: coverage-gate-author scopes to files touched since implementing was set; run-gates emits whole-workspace numbers and the mapping is manual today"
  - "ship-tasks.md coverage doctrine (90 percent on touched files) + coverage-gate@1 shape (frontmatter tests_failed / files_below_90pct / ecm_rows_uncovered + per-file table)"
  - "the sachviet batch-1 recorded gates (vitest/c8 coverage-summary.json; per-file tables in docs/tasks/web/*/coverage-gate.md) - the reproduction target"
source_decisions:
  - "2026-07-17 Stephen: batch 4 PLAN approved (§0a, all 7 items)."
---

# TASK-IMP-098: coverage-scope helper maps a task diff to per-file coverage

## Summary

The coverage gate's unit of judgment is "files this task touched", but run-gates emits whole-workspace numbers and every gate so far mapped diff to coverage by hand. Ship `coverage-scope.mjs`: give it a task id (or an explicit base ref) and a coverage report, get back the touched-file list joined to per-file percentages, emitted as a coverage-gate@1 skeleton ready for the author skill to complete.

## Problem

Manual mapping is exactly the fatigue-prone mechanical work the machine-floor doctrine (TASK-IMP-084) moves into tools: reading a diff range, filtering to source files, and cross-walking a coverage report is deterministic - a human or model doing it fresh each gate re-derives bytes a script can own.

## Proposed Solution

`node .cyberos/docs-tools/coverage-scope.mjs <task-id> [--base <ref>] [--coverage <file>] [--repo <root>]`. Base resolution: `--base` wins; otherwise find the entry-flip commit whose subject names the task id entering implementing (the corpus convention every batch commit follows), else fail with a clear message demanding --base. Touched files: `git diff --name-only <base>...HEAD` filtered to tracked source files. Coverage ingestion: c8/istanbul `coverage-summary.json` and `lcov.info` - the two shapes the detected stacks emit; anything else is refused by name. Output: a coverage-gate@1 skeleton (frontmatter with computed files_below_90pct, the per-file table, TODO markers for the judgment fields) to stdout or --out. Node stdlib only; build.sh vendors it with the guarded-copy pattern.

## Alternatives Considered

- Teach run-gates.sh to scope. Rejected: run-gates runs the repo's own commands verbatim by design (TASK-CUO-207); scoping is a reporting concern layered on top.
- Parse every coverage format. Rejected: the two named shapes cover the detected stacks; refusing others loudly beats guessing quietly.
- Fold into ship-manifest.mjs. Rejected: manifests track run state; this reads git + a report and writes a doc skeleton - different lifecycle, own tool.

## Success Metrics

- Primary: against a fixture repo with a known diff and report, the emitted skeleton's touched list, per-file rows, and files_below_90pct match the fixture's expected bytes - suite-asserted every run. Baseline: fully manual mapping. Deadline: final acceptance.
- Guardrail: run against the shipped sachviet batch reproduces the recorded per-file tables (ops-verified, recorded in the gate log - consumer-repo evidence the suite cannot carry).

## Scope

In scope: the CLI, base resolution, the two report shapes, the skeleton emitter, fixture suite, build.sh vendor line.

### Out of scope / Non-Goals

- Running the coverage command itself (run-gates' job).
- Judgment fields of coverage-gate@1 (ecm_rows_uncovered mapping stays with the author skill).
- Other coverage formats until a detected stack emits them.

## Dependencies

- Shares tools/install/build.sh with TASK-IMP-093 - same agent, serial order per the batch plan.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from IMP-14 and the recorded sachviet gates; implementation under ship-tasks supervision.
- **Human review:** batch-4 PLAN approved 2026-07-17 (§0a); both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 The CLI MUST resolve the diff base as: explicit `--base` first; else the commit whose subject names `<task-id>` entering implementing; else fail loudly demanding --base (never guess a range).
- 1.2 Touched files MUST come from `git diff --name-only <base>...HEAD`, filtered to files existing at HEAD (deletions excluded from the coverage table but listed in the skeleton's notes).
- 1.3 The tool MUST ingest c8/istanbul coverage-summary.json and lcov.info, and MUST refuse any other input by name with a non-zero exit.
- 1.4 Output MUST be a coverage-gate@1 skeleton: frontmatter (tests_failed left TODO, files_below_90pct computed from the 90 threshold, ecm_rows_uncovered TODO), per-file table for every touched source file with its percentage or `no-coverage-data`, and the base/HEAD range recorded.
- 1.5 build.sh MUST vendor the tool (guarded copy) and the suite MUST gate the payload copy against a scratch build.
- 1.6 The suite MUST land at tools/install/tests/test_coverage_scope.sh (run_all glob discovery).

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - base resolution: --base wins; subject-scan finds the flip commit; no-match fails loudly - test: `tools/install/tests/test_coverage_scope.sh::t01_base_resolution`
- [ ] AC 2 (traces_to: #1.2, #1.3, #1.4) - fixture repo + both report shapes -> skeleton matches expected bytes incl. files_below_90pct and the deletion note - test: `tools/install/tests/test_coverage_scope.sh::t02_skeleton_from_fixture`
- [ ] AC 3 (traces_to: #1.3) - unknown report shape refused by name - test: `tools/install/tests/test_coverage_scope.sh::t03_unknown_report_refused`
- [ ] AC 4 (traces_to: #1.5, #1.6) - scratch payload carries the tool; suite glob-discovered - test: `tools/install/tests/test_coverage_scope.sh::t04_payload_vendored`
- [ ] AC 5 (guardrail) - sachviet batch-1 per-file tables reproduced - verify: recorded run output in the gate log (consumer-repo evidence; the fixture suite carries the automated half).

## 3. Edge cases

- Touched file absent from the coverage report (doc, config, or untested source): row emitted with `no-coverage-data` - visible, never silently dropped (t02 asserts one such row).
- File deleted in the range: excluded from the table, named in the notes (1.2, t02).
- Multiple commits name the task id: the earliest implementing-entry match wins; ambiguity is reported in the skeleton's range note (t01 arm).
- Percentages exactly at 90: not below threshold (strict less-than, matching the gate's wording "below 90").
- Security-class: read-only over git and a named report file; no network; refuses paths outside the repo root.
