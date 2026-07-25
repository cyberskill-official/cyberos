---
id: TASK-IMP-008
title: "Goldensets as first-class gate inputs"
eu_ai_act_risk_class: not_ai
ai_authorship: generated_then_reviewed
client_visible: false
type: improvement
created_at: 2026-07-08T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: improvement
priority: p0
status: done
phase: Wave 1 - see and survive
refs: [R16]
depends_on: []
created: 2026-07-08
verify: T
owner: Stephen Cheng (CTO)
language: bash + yaml
service: tools/install/ + scripts/
new_files:
  - tools/install/.awh/goldenset.yaml
  - tools/install/.awh/eval-baseline.json
  - tools/install/run-goldenset.sh
  - tools/install/tests/test_install_goldenset.sh
  - docs/verification/install-goldenset.md
modified_files:
  - .github/workflows/awh-gate.yml
  - CHANGELOG.md
source_pages:
  - "scripts/awh_goldenset_from_task.py"
  - ".github/workflows/awh-gate.yml (modules/*/.awh/goldenset.yaml + baseline fail-closed)"
  - "modules/cuo/.awh/goldenset.yaml (shape reference)"
source_decisions:
  - "2026-07-25 batch/12c: land a payload-relevant install goldenset (version/help/gates smoke), not full module cargo suites."
  - "2026-07-25: when awh is absent, runner MUST fall back to python cmd runner when available, otherwise skip cleanly (exit 0 with SKIP provenance); CI path that has awh MUST run and fail closed without baseline."
---

# TASK-IMP-008: Goldensets as first-class gate inputs

## Summary

Make goldensets a real, documented gate input for the CyberOS 1.x **install/payload** surface: a sealed `tools/install/.awh/goldenset.yaml` + baseline, a runner that prefers `awh` and degrades cleanly when awh is absent, suite coverage, and a lightweight CI step alongside the existing module awh-gate.

## Problem

Module goldensets exist under `modules/*/.awh/` and feed `awh-gate.yml`, but the install/payload tooling that ships CyberOS 1.x has no first-class goldenset. `awh_goldenset_from_task.py` helps author module tasks; nothing standardizes an install-path goldenset or a skip-clean runner for hosts without awh.

## Proposed Solution

1. Author `tools/install/.awh/goldenset.yaml` with lightweight, offline-safe tasks (VERSION semver, help.sh, run-gates script presence/shebang check, coverage-ratchet --help).
2. Commit a matching `eval-baseline.json` sealed from a green run.
3. Ship `tools/install/run-goldenset.sh` that: runs via `awh eval` when available; otherwise executes each task `cmd` with a tiny fallback runner; skips cleanly when `CYBEROS_SKIP_GOLDENSET=1` or when neither awh nor python3 is available (documented SKIP).
4. Document at `docs/verification/install-goldenset.md`.
5. Add an `install-goldenset` job to `awh-gate.yml` that does **not** require the docker/Postgres stack.

## Alternatives Considered

- **Only document module goldensets.** Rejected: mission asks for payload-relevant install goldenset.
- **Require awh always.** Rejected: local macOS contributors may lack awh; skip-clean is mandatory.
- **Put install goldenset under modules/install.** Rejected: install lives in `tools/install`; keep path honest.

## Success Metrics

- Primary: suite proves skip / awh-or-fallback run / fail-closed without baseline when forced.
- Guardrail: existing module awh-gate job unchanged in spirit; install job is additive and offline-safe.

## Scope

In scope: install goldenset + baseline + runner + docs + suite + awh-gate.yml job + CHANGELOG.

### Out of scope / Non-Goals

- Regenerating every module goldenset.
- Making awh a hard install dependency.
- Auto-revert (TASK-IMP-026).

## Dependencies

None hard. Soft: awh CLI / `tools/awh` harness when present.

## AI Authorship Disclosure

- **Tools used:** Cursor agent, batch/12c.
- **Human review:** session operator Stephen Cheng.

## §1 - Description (normative)

1. `tools/install/.awh/goldenset.yaml` MUST exist with schema compatible with awh (`tasks:` list; each task has `id`, `cmd`, `weight`, `timeout_sec`).
2. At least three tasks MUST be offline-safe (no network, no docker, no cargo).
3. `tools/install/.awh/eval-baseline.json` MUST be committed; awh-gate install job MUST fail closed if goldenset exists without baseline.
4. `tools/install/run-goldenset.sh` MUST:
   - exit 0 with a `SKIP` line when `CYBEROS_SKIP_GOLDENSET=1`;
   - prefer `awh eval … --baseline … --max-regression 0.0` when `awh` is on PATH or `python3 -m harness.cli` works with `PYTHONPATH=tools/awh`;
   - otherwise run each task `cmd` via a python3 YAML fallback;
   - if neither awh nor python3 is available, exit 0 with `SKIP` naming the reason.
5. `docs/verification/install-goldenset.md` MUST document path, runner, skip rules, and CI wiring.
6. `.github/workflows/awh-gate.yml` MUST gain an `install-goldenset` job (no postgres/redis) that runs the install goldenset when `tools/install/**` changes (or always on PR — either is acceptable if documented).
7. Suite `test_install_goldenset.sh` MUST cover skip + runner presence + goldenset shape.

## Acceptance criteria

- [x] AC1: goldenset.yaml parses and lists >= 3 tasks with id+cmd. (t01)
- [x] AC2: baseline file present next to goldenset. (t02)
- [x] AC3: `CYBEROS_SKIP_GOLDENSET=1` → exit 0 + SKIP. (t03)
- [x] AC4: runner executes successfully on this host (awh path validates committed baseline; fallback executes committed tasks without baseline compare). (t04)
- [x] AC5: docs page names runner + skip env + CI job. (t05)
- [x] AC6: awh-gate.yml contains install-goldenset job referencing tools/install/.awh. (t06)

## Test plan

`bash tools/install/tests/test_install_goldenset.sh`
