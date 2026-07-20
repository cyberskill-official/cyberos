---
id: TASK-IMP-128
title: Run the test suite in CI on ubuntu
template: task@1
type: improvement
module: improvement
status: draft
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-20T00:00:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-127]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.0.0"
owner: Stephen Cheng (CTO)
created: 2026-07-20
memory_chain_hash: null
effort_hours: 3
service: .github/workflows
new_files:
  - tools/install/tests/test_ci_runs_suite.sh
modified_files:
  - .github/workflows/payload-gate.yml
source_pages:
  - "measured 2026-07-20: grep -rln 'run_all.sh' .github/workflows/ returns nothing - no workflow invokes the suite"
  - "measured 2026-07-20: grep -rln 'test_release_assets' .github/workflows/ returns nothing"
  - "tools/install/tests/test_release_assets.sh (self-skips on macOS with 'needs GNU tar (BSD/macOS host); runs on ubuntu/CI' - the ubuntu/CI it names does not run it)"
  - "scripts/tests/run_all.sh (37 suites; enforced today only by the local pre-commit hook, on the committer's machine)"
source_decisions:
  - "2026-07-20 Stephen: PLAN gate - author as three separate tasks (build / CI / config durability); approved."
---

# TASK-IMP-128: Run the test suite in CI on ubuntu

## Summary

No workflow runs `scripts/tests/run_all.sh`. The suite is enforced only by a local pre-commit hook, on one macOS laptop, and only for commits that trigger it. One suite file has consequently never executed anywhere. Add an ubuntu job that runs the suite, converting a local convention into an enforced gate and giving the suite its first run on a platform other than the author's machine.

## Problem

Two facts, both measured on 2026-07-20 against HEAD:

- No file under `.github/workflows/` invokes `scripts/tests/run_all.sh`.
- No file under `.github/workflows/` invokes `tools/install/tests/test_release_assets.sh`.

The second compounds the first. `test_release_assets.sh` self-skips on macOS with the message "needs GNU tar (BSD/macOS host); runs on ubuntu/CI". It does not run on ubuntu/CI, because CI does not run the suite at all. The test names a CI that would execute it, and that CI does not exist - so the file has never run on any machine, while reporting a skip that reads like coverage deferred rather than coverage absent.

The wider consequence is the point. The pass figure this release was judged against is enforced only by the local pre-commit hook: on the committer's machine, and only for code changes. A contributor without the hook, or any docs-only commit, bypasses it entirely. Every "suite green" claim in the 1.0.0 release rests on a human having run it by hand on one macOS host.

That single-platform enforcement has a measured cost. Five bash-3.2 and BSD-userland defects were fixed on 2026-07-19 - including a `sed -i` misuse that made a tamper-detection test pass unconditionally on macOS. A suite that ran on ubuntu would have failed on the GNU-vs-BSD divergence rather than silently passing.

## Proposed Solution

Add a job to the existing CI that checks out the repo on `ubuntu-latest` and runs `bash scripts/tests/run_all.sh`. On ubuntu the suite executes all 37 files including `test_release_assets.sh`, giving that file its first execution and giving every other file its first run on a non-macOS host.

## Alternatives Considered

- A matrix across ubuntu and macOS. Deferred, not rejected: ubuntu is where the never-executed file runs and where the userland differs from the development host, so it is the higher-value half. A macOS leg can be added once the ubuntu leg is green and its runtime is known.
- Rely on the pre-commit hook and document that contributors must install it. Rejected: the hook is machine-local and untracked, skipped for docs-only commits, and bypassed by `--no-verify` - which was itself used during this release. A convention enforced only by the honour system on one machine is the condition this task exists to end.
- Run only `test_release_assets.sh` in CI. Rejected: it fixes the one file that has never run and leaves the other 36 still enforced by a single laptop.

## Success Metrics

- Primary: the suite runs on `ubuntu-latest` on every push and pull request, and a deliberately broken test fails the workflow. Baseline today: zero suites execute in CI and no test failure can fail a workflow.
- Guardrail: `test_release_assets.sh` reports an executed result rather than a skip on the ubuntu leg - the file that has never run, runs.

## Scope

In scope: one CI job invoking the existing suite entrypoint, and whatever minimal fixes the suite needs to pass on ubuntu.

### Out of scope / Non-Goals

- Rewriting or restructuring the suite - this task runs what exists.
- A macOS or Windows leg (deferred; see Alternatives).
- Coverage measurement or thresholds.
- Removing or weakening the local pre-commit hook - it stays as the fast local signal.

## Dependencies

None blocking. Adjacent to TASK-IMP-127: both concern guarantees the release makes that are not mechanically enforced.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from the 2026-07-19 B4 verification pass. The two absence claims are grep results against `.github/workflows/` at HEAD, re-run on 2026-07-20. An earlier draft of this finding stated that a release dispatch would give `test_release_assets.sh` its first execution; that was wrong - the payload job runs `release-assets.sh`, the producer, not the test - and the corrected claim (no workflow runs it, so it has never executed) is what this spec carries.
- **Human review:** scope and granularity approved at the 2026-07-20 PLAN gate; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 A CI job MUST run `bash scripts/tests/run_all.sh` on `ubuntu-latest` for every push and pull request to the default branch.
- 1.2 A failing test MUST fail the job - the suite's non-zero exit MUST propagate to the workflow conclusion, and MUST NOT be masked by `continue-on-error`, `|| true`, or an ignored exit code.
- 1.3 `tools/install/tests/test_release_assets.sh` MUST execute rather than self-skip on the ubuntu leg, and its result MUST be included in the suite outcome.
- 1.4 The job MUST report the per-suite pass, fail, and skip counts in its log so a reader can see which files ran without re-running them locally.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - the workflow file declares a job on `ubuntu-latest` whose run step invokes `scripts/tests/run_all.sh`, triggered on push and pull_request - test: `tools/install/tests/test_ci_runs_suite.sh::t_suite_job_declared`
- [ ] AC 2 (traces_to: #1.2) - a deliberately failing test in a fixture suite causes the runner to exit non-zero, and no step in the job path swallows that exit code - test: `tools/install/tests/test_ci_runs_suite.sh::t_failure_propagates`
- [ ] AC 3 (traces_to: #1.3) - on a Linux host `test_release_assets.sh` runs its assertions instead of taking the GNU-tar skip branch - test: `tools/install/tests/test_ci_runs_suite.sh::t_release_assets_executes_on_linux`
- [ ] AC 4 (traces_to: #1.4) - the suite entrypoint emits pass, fail, and skip counts to stdout - test: `tools/install/tests/test_ci_runs_suite.sh::t_counts_reported`

## 3. Edge cases

- Suites that assume BSD userland may fail on first ubuntu run. That is the finding, not a blocker: each failure is either a real portability defect to fix or a test to make platform-explicit. The job MUST NOT be merged with failures suppressed to make it green.
- A suite requiring network or credentials MUST skip explicitly with a stated reason rather than fail, and the skip MUST be visible in the counts required by 1.4.
- Suite runtime on a cold ubuntu runner is unknown; if it exceeds the job default timeout the timeout MUST be raised rather than the suite trimmed.
- `run_all.sh` invoked from a different working directory MUST still resolve its suite paths - CI checks out to a path unlike any local one.
- Security-class: the job runs repository test code on a CI runner with the default token. It MUST NOT require elevated permissions, and MUST NOT be granted secrets - no suite in `run_all.sh` needs them, and granting them would expose secrets to test code on pull requests from forks.
