---
id: TASK-IMP-026
title: "Auto-revert on gate regression"
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
phase: Wave 3 - widen the envelope
refs: [Stage, 2]
depends_on: [TASK-IMP-008]
created: 2026-07-08
verify: T
owner: Stephen Cheng (CTO)
language: bash
service: tools/install/
new_files:
  - tools/install/gate-auto-revert.sh
  - tools/install/tests/test_gate_auto_revert.sh
  - docs/verification/gate-auto-revert.md
modified_files:
  - CHANGELOG.md
source_pages:
  - "docs/verification/install-goldenset.md (TASK-IMP-008)"
  - "tools/install/gates/run-gates.sh (failure artifact from TASK-IMP-011)"
source_decisions:
  - "2026-07-25 batch/12c: opt-in only via CYBEROS_AUTO_REVERT=1; never force-push; never merge; prefer gh revert PR + comment with failing goldenset case."
  - "2026-07-25: dry-run mode is the testable default path; live gh requires opt-in env."
---

# TASK-IMP-026: Auto-revert on gate regression

## Summary

When main (or a protected branch) regresses a gate — especially a goldenset case — provide a **documented, opt-in** script that proposes a `git revert` pull request via `gh`. Never force-push. Never merge without an operator.

## Problem

Gate regressions on main currently require a human to notice CI, identify the bad SHA, and hand-roll a revert. Without an opt-in helper, recovery is slow; with a silent auto-force-push, recovery would be unsafe.

## Proposed Solution

Ship `tools/install/gate-auto-revert.sh` that, given a bad SHA:

1. Refuses to run unless `--dry-run` **or** `CYBEROS_AUTO_REVERT=1`.
2. In dry-run: prints the revert commit subject, branch name, and the `gh pr create` plan (including optional goldenset failure note / last-gate-failure class) without mutating git remotes.
3. In live mode: creates a local revert commit on a new branch and opens a PR with `gh pr create` (no `--merge`, no force-push flags ever present in the script).
4. Document at `docs/verification/gate-auto-revert.md`.

## Alternatives Considered

- **Silent `git push --force` to main.** Rejected: forbidden by CyberOS doctrine and this mission.
- **Auto-merge revert PRs.** Rejected: still requires operator; script must not merge.
- **Only a CI workflow with no local script.** Rejected: dry-run testability is poorer; script+docs is the unit.

## Success Metrics

- Primary: dry-run tests green; script contains no force-push/merge invocations.
- Guardrail: without opt-in env and without `--dry-run`, exit non-zero and write nothing.

## Scope

In scope: gate-auto-revert.sh, docs, tests, CHANGELOG.

### Out of scope / Non-Goals

- Enabling CYBEROS_AUTO_REVERT in default CI.
- Force-push or auto-merge.
- Replacing human HITL on the revert PR.

## Dependencies

`depends_on: [TASK-IMP-008]` — revert PR body SHOULD cite goldenset failure context when provided; goldenset path is standardized by 008.

## AI Authorship Disclosure

- **Tools used:** Cursor agent, batch/12c.
- **Human review:** session operator Stephen Cheng.

## §1 - Description (normative)

1. `tools/install/gate-auto-revert.sh [--dry-run] [--goldenset-case <id>] [--failure-json <path>] <bad-sha> [base-branch]` MUST exist.
2. Without `--dry-run`, the script MUST refuse unless `CYBEROS_AUTO_REVERT` is exactly `1` (literal), exiting 2 with an actionable message.
3. The script MUST NEVER invoke `git push --force`, `git push -f`, `gh pr merge`, or `git merge` onto the base branch.
4. Dry-run MUST print a plan including: bad SHA, proposed branch `revert/<shortsha>`, `git revert --no-edit <sha>` intent, and `gh pr create` title/body outline; exit 0; no branch checkout required beyond read-only `git rev-parse` / `git show`.
5. Live mode (opt-in) MUST: verify SHA exists; create branch from base (default `main`); `git revert --no-edit <bad-sha>`; `git push -u` (fast-forward only); `gh pr create` with body naming goldenset case and/or failure JSON class when provided.
6. Docs MUST state the opt-in contract and the never-force-push / never-merge invariants.
7. Suite `test_gate_auto_revert.sh` MUST cover dry-run plan, refuse-without-opt-in, and absence of force-push/merge strings in the script.

## Acceptance criteria

- [x] AC1: `--dry-run <sha>` exits 0 and prints revert plan without needing CYBEROS_AUTO_REVERT. (t01)
- [x] AC2: live mode without env exits 2. (t02)
- [x] AC3: script source has no force-push or pr-merge invocations. (t03)
- [x] AC4: dry-run body includes `--goldenset-case` when passed. (t04)
- [x] AC5: docs page states opt-in + never force-push/merge. (t05)

## Test plan

`bash tools/install/tests/test_gate_auto_revert.sh`
