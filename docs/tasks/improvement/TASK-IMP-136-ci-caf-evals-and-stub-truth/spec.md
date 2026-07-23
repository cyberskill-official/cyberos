---
id: TASK-IMP-136
title: CI truth - CAF evals in root CI, hook-claim honesty, stub workflow sweep
template: task@1
type: improvement
module: improvement
status: ready_to_implement
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-23T00:00:00+07:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-128, TASK-IMP-140, TASK-AI-013, TASK-AI-015, TASK-AI-018, TASK-MEMORY-102, TASK-OBS-005, TASK-PROJ-018, TASK-REW-010]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.1.0"
owner: Stephen Cheng (CTO)
created: 2026-07-23
memory_chain_hash: null
effort_hours: 8
service: .github/workflows
new_files:
  - .github/workflows/caf-evals-gate.yml
  - scripts/tests/test_ci_truth.sh
modified_files:
  - .githooks/pre-commit
  - .pre-commit-config.yaml
  - .github/workflows/cache-isolation-gate.yml
  - .github/workflows/memory-rebuild.yml
  - .github/workflows/obs-correlation-gate.yml
  - .github/workflows/proj-a11y-gate.yml
  - .github/workflows/proj-storybook-chromatic.yml
  - .github/workflows/rew-memory-exclusion.yml
  - .github/workflows/vn-pii-quarterly-refresh.yml
  - .github/workflows/vn-pii-recall.yml
  - .github/workflows/zdr-staleness-check.yml
  - CHANGELOG.md
source_pages:
  - "measured 2026-07-23: grep -l STUB .github/workflows/*.yml returns exactly 9 files (cache-isolation-gate, memory-rebuild, obs-correlation-gate, proj-a11y-gate, proj-storybook-chromatic, rew-memory-exclusion, vn-pii-quarterly-refresh, vn-pii-recall, zdr-staleness-check), each auto-generated 2026-05-17 with a placeholder job whose only step is `run: echo 'Stub - see task specs for canonical workflow YAML'` - always green"
  - "stub declarers (from each file's 'Declared by:' header): TASK-AI-018, TASK-MEMORY-102, TASK-OBS-005, TASK-PROJ-018 (x2), TASK-REW-010, TASK-AI-013 (x2), TASK-AI-015 - several of which are status done, so shipped tasks claim CI gates that gate nothing"
  - "tools/caf/core/evals/validate.py:5 (usage: 'python3 core/evals/validate.py --all | --run <dir> [--report json|sarif]'); tools/caf/core/evals/fixtures/ holds 40 fixture dirs; tools/caf/.github/workflows/evals.yml exists NESTED under tools/caf/ where GitHub Actions never reads it - the CAF eval suite runs in no CI today"
  - "scripts/caf_precommit_check.sh (exists; wired into no hook and no workflow - measured by grep across .githooks/ and .github/workflows/)"
  - ".pre-commit-config.yaml (declares hooks awh-gate, cyberos-payload-build, docs-site-build for the `pre-commit` framework) vs the repo's actual hook mechanism core.hooksPath=.githooks - the framework config is dead: .githooks/pre-commit covers payload-build and docs-build itself but runs .pre-commit-hooks/awh-gate.sh nowhere"
  - "measured 2026-07-23: no root workflow invokes scripts/tests/run_all.sh - that CI job is TASK-IMP-128's authored scope (draft), deliberately not duplicated here"
source_decisions:
  - "2026-07-23 operator: CyberOS Hardening Plan approved; Phase 2 T4 'CI hardening' authored as an improvement task (plan file cyberos_hardening_plan_49404998; audit finding H9)."
  - "2026-07-23 authoring: the plan bullet includes 'root CI job for scripts/tests/run_all.sh'; repo inspection found TASK-IMP-128 (draft, p1) already owns exactly that job. This task scopes it OUT and references it instead of duplicating - recorded as a plan-vs-repo adjustment."
  - "2026-07-23 authoring: default stub disposition is DELETE (with a per-workflow implement override when the declaring task's spec embeds complete canonical YAML whose dependencies exist today). Deletion reason recorded per the never-delete-without-reason rule: an always-green check is worse than no check - it manufactures false confidence under a gate-shaped name. The HITL review gate covers the final per-file disposition table."
---

# TASK-IMP-136: CI truth - CAF evals in root CI, hook-claim honesty, stub workflow sweep

## Summary

Three CI surfaces claim enforcement that does not happen: the CAF eval suite (40 fixtures + a validator) runs in no CI because its workflow sits nested under `tools/caf/.github/` where GitHub never reads it; `.pre-commit-config.yaml` declares an awh gate hook for a framework the repo does not use (`core.hooksPath=.githooks` is the real mechanism, and it runs no awh hook); and 9 auto-generated stub workflows report green on every PR while their only step is an `echo`. This task wires the CAF evals + `caf_precommit_check.sh` into a real root workflow, makes the hook claims honest (wire awh into `.githooks/pre-commit`, drop the dead framework config), and sweeps the 9 stubs (delete by default, implement where the declaring spec's embedded YAML is complete) with a stub-honesty check so the class cannot regrow.

## Problem

Audit finding H9, all verified first-hand 2026-07-23:

1. **CAF evals are dead in CI.** `tools/caf/core/evals/validate.py --all` validates 40 audit fixtures; the only workflow that runs it lives at `tools/caf/.github/workflows/evals.yml` - a nested `.github` directory GitHub Actions never evaluates. `scripts/caf_precommit_check.sh` exists and is wired into nothing. The CAF gate is a load-bearing step (ship-tasks step 29); its own regression suite runs on no machine but a developer's, voluntarily.
2. **The pre-commit claims are split-brain.** `.pre-commit-config.yaml` tells a reader that awh-gate, payload-build, and docs-build run at commit time via the `pre-commit` framework. The repo's real hook path is `.githooks/` (set by `core.hooksPath`); its `pre-commit` script covers payload-build and docs-build directly but never runs `.pre-commit-hooks/awh-gate.sh`. Result: the awh commit-time gate exists only as a claim in a file no tool reads.
3. **Nine always-green stubs.** Auto-generated 2026-05-17 from task `build_envelope` references, each with a single `echo` step. Several declaring tasks (`TASK-AI-013`, `TASK-AI-015`, `TASK-MEMORY-102`...) are `done` - shipped work whose acceptance story includes a CI gate that gates nothing. A green check that checks nothing is strictly worse than a missing check: it terminates the reader's investigation at a lie.

## Proposed Solution

Add `.github/workflows/caf-evals-gate.yml` running `python3 core/evals/validate.py --all` (from `tools/caf/`) plus `bash scripts/caf_precommit_check.sh` on PRs touching `tools/caf/**` or `scripts/caf_*` and on a weekly schedule (drift net). Wire `.pre-commit-hooks/awh-gate.sh` into `.githooks/pre-commit` behind the same staged-paths pattern the other blocks use, then delete `.pre-commit-config.yaml` (reason recorded: dead config whose every live claim is now covered by the real hook; keeping it preserves a second, contradictory hook story). Sweep the 9 stubs with a per-file disposition table in the implementation PR: default DELETE (each deletion names its declaring task in the commit body so the gap stays discoverable); implement instead where the declaring task's spec embeds canonical YAML that is complete and whose runtime dependencies exist today. Add `scripts/tests/test_ci_truth.sh` asserting: a root workflow invokes `validate.py --all`; a root workflow invokes `caf_precommit_check.sh`; no root workflow contains the stub placeholder marker; and `.pre-commit-config.yaml` either does not exist or every hook it names is invoked by `.githooks/pre-commit` (the regrowth guard). CHANGELOG entry records the sweep and the deleted file.

## Alternatives Considered

- **Move `tools/caf/.github/workflows/evals.yml` to the root instead of writing a new workflow.** Considered, partially adopted: the new root workflow reuses its steps where they fit, but the nested file also serves the standalone-CAF-repo split (pages/publish siblings) and is left in place for that context; the root workflow is the monorepo's own gate. The nested file alone can never fire here, which is the defect.
- **Keep `.pre-commit-config.yaml` and adopt the pre-commit framework for real.** Rejected: two hook mechanisms racing on one repo (`core.hooksPath` + framework-managed `.git/hooks`) is a conflict by construction; `.githooks/` is where every existing live gate (payload, docs, status, run_all, version-sync) already lives, and the framework adds a Python dependency for zero new coverage.
- **Label the stubs (rename check to `STUB-...`) instead of deleting.** Rejected as the default: a labeled always-green check still occupies a required-check slot and still summarizes green; the honest states are "real gate" or "no gate". Labeling is acceptable only as a transitional state and G14 (TASK-IMP-140) treats a labeled stub as non-compliant for branch protection either way.
- **Implement all 9 stubs fully in this task.** Rejected: several require infrastructure that does not exist yet (Chromatic project, VN-PII quarterly data refresh pipeline); implementing them here balloons an enforcement-truth task into six feature tasks. The disposition rule (implement only where the embedded YAML is complete AND dependencies exist) keeps the judgment bounded, and deletions leave a named trail back to their declaring tasks for future re-authoring.

## Success Metrics

- Primary: by the next CyberOS release, the CAF eval suite fails a PR that breaks a fixture (verifiable by a deliberate scratch-branch break), zero workflows in root `.github/workflows/` carry the stub placeholder marker, and `test_ci_truth.sh` is green in `run_all.sh`. Baseline today: CAF evals run in no CI, 9 stubs report green everywhere, no truth check exists.
- Guardrail: no existing live workflow (payload-gate, awh-gate, docs-prerender-gate, services, deploy...) changes behavior; `.githooks/pre-commit` total runtime stays under ~2 minutes for an awh-relevant commit (awh-gate.sh already scopes itself to changed modules).

## Scope

In scope: the new root CAF workflow, awh hook wiring, `.pre-commit-config.yaml` removal, the 9-stub sweep with disposition table, the `test_ci_truth.sh` regrowth guard, CHANGELOG.

### Out of scope / Non-Goals

- The root CI job for `scripts/tests/run_all.sh` - authored as TASK-IMP-128 (draft); this task's `test_ci_truth.sh` rides in `run_all.sh` and therefore lands in CI the moment IMP-128 ships. No ordering constraint between the two.
- Re-authoring replacement tasks for deleted stubs (the deletion trail names the declaring tasks; re-authoring is the operator's call per gap).
- The G5/G14 benchmark-gate definitions and their CI meta-check - TASK-IMP-140; this task's checker is the mechanism G14 adopts.
- Branch-protection rule changes (server-side; operator action, see Operator steps in edge cases).

## Dependencies

None blocking. TASK-IMP-128 (draft) owns the run_all.sh CI job this task deliberately does not duplicate. TASK-IMP-140 (benchmark gates) adopts this task's `test_ci_truth.sh` as the G14 checker - soft forward reference via `related_tasks`, no cycle. The stub declarers (TASK-AI-013/015/018, TASK-MEMORY-102, TASK-OBS-005, TASK-PROJ-018, TASK-REW-010) are listed in `related_tasks` because each deletion cites its declarer; none of their statuses change here.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS `task-author` skill in Cursor, as the task-authoring wave of the 2026-07-23 hardening plan.
- **Scope:** stub count/contents, declarer headers, validate.py usage, the nested workflow path, the dead framework config, and the absence of any run_all.sh CI invocation were all verified first-hand at HEAD; the TASK-IMP-128 overlap was discovered during authoring and is recorded as a plan adjustment in `source_decisions`.
- **Human review:** the hardening plan was operator-approved 2026-07-23; the delete-by-default stub disposition and the framework-config removal are recorded decisions for the reviewer to confirm at the review acceptance gate (both are reversible file operations on a branch).

## 1. Description (normative)

- 1.1 A root workflow `.github/workflows/caf-evals-gate.yml` MUST run `python3 core/evals/validate.py --all` with working directory `tools/caf` and `bash scripts/caf_precommit_check.sh` from the repo root, triggered on pull requests touching `tools/caf/**` or `scripts/caf_*` and on a weekly `schedule`. A fixture regression or checker failure MUST fail the workflow.
- 1.2 `.githooks/pre-commit` MUST invoke `.pre-commit-hooks/awh-gate.sh` when staged paths match the module-source pattern that script scopes itself to, using the same herestring `matches()` idiom the hook already uses for its other blocks (the SIGPIPE pitfall documented in the hook header MUST NOT be reintroduced).
- 1.3 `.pre-commit-config.yaml` MUST be removed once 1.2 lands, with the removal reason in the commit body and CHANGELOG (dead mechanism; every live claim now covered by `.githooks/pre-commit`). If the operator vetoes removal at review, the fallback is a header comment declaring the file non-authoritative - but removal is the authored default.
- 1.4 Each of the 9 stub workflows MUST be dispositioned: DELETE by default, with the commit body naming the file and its declaring task; IMPLEMENT instead when the declaring task's spec embeds complete canonical workflow YAML whose runtime dependencies (secrets, services, data) exist today. The implementation PR MUST carry the 9-row disposition table so the review gate sees every call explicitly. No stub may survive unchanged.
- 1.5 A new suite `scripts/tests/test_ci_truth.sh` MUST assert offline: (a) some root workflow invokes `validate.py --all`; (b) some root workflow invokes `caf_precommit_check.sh`; (c) no file under `.github/workflows/` contains the stub placeholder marker (`Stub - see task specs` / `Stub — see task specs`); (d) `.pre-commit-config.yaml` is absent OR every `entry:` it declares is also invoked from `.githooks/pre-commit`. Because it matches `scripts/tests/test_*.sh`, `run_all.sh`'s glob registers it automatically.
- 1.6 `CHANGELOG.md` MUST gain an entry recording the CAF CI gate, the awh hook wiring, the framework-config removal, and the stub sweep with its disposition counts.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - `caf-evals-gate.yml` exists at the root workflows dir, names both commands, both trigger paths, and a schedule; a deliberately broken fixture on a scratch branch makes `validate.py --all` exit non-zero - test: `scripts/tests/test_ci_truth.sh::t01_caf_gate_wired`
- [ ] AC 2 (traces_to: #1.2) - `.githooks/pre-commit` invokes `awh-gate.sh` via the `matches()` herestring idiom (no `git diff --cached | grep -q` pipeline), and a commit staging only unrelated paths does not trigger it - test: `scripts/tests/test_ci_truth.sh::t02_awh_hook_wired_safely`
- [ ] AC 3 (traces_to: #1.3) - `.pre-commit-config.yaml` does not exist, and CHANGELOG names its removal - test: `scripts/tests/test_ci_truth.sh::t03_dead_config_gone`
- [ ] AC 4 (traces_to: #1.4) - zero files under `.github/workflows/` match the stub placeholder marker, and for every deleted stub the commit body names file + declaring task (asserted against the disposition table committed with the PR) - test: `scripts/tests/test_ci_truth.sh::t04_no_stub_survives`
- [ ] AC 5 (traces_to: #1.5) - `test_ci_truth.sh` runs green under `bash scripts/tests/run_all.sh` discovery and each of its four asserts fails when its precondition is broken in a scratch copy (self-test mode) - test: `scripts/tests/test_ci_truth.sh::t05_self_test_negative_paths`
- [ ] AC 6 (traces_to: #1.6) - CHANGELOG's top entry mentions caf-evals-gate, awh hook, the removed config file, and the stub disposition counts - test: `scripts/tests/test_ci_truth.sh::t06_changelog_records_sweep`

## 3. Edge cases

- **Branch protection referencing a stub check name:** deleting a workflow whose check is "required" would wedge PRs server-side. Operator steps (per ship-tasks' in-task guideline rule): before merging the sweep, run `gh api repos/:owner/:repo/branches/main/protection` and confirm no required status check names any of the 9; expected output: none do (they were never wired as required - verify, don't assume). If one is required, the operator removes it from protection first; the PR notes it.
- **`vn-pii-recall` name collision with a real gate elsewhere:** TASK-AI-013 shipped a real recall gate as a test suite; only the stub *workflow* is swept. The deletion note names TASK-AI-013 so the "CI gate" claim in that done spec is traceable to this sweep rather than silently orphaned.
- **CAF evals runtime on ubuntu CI:** 40 fixtures are validation-only (no model calls); if wall time exceeds the job budget, the workflow may shard by fixture prefix - but it MUST NOT subset silently: all 40 run per gate invocation.
- **awh-gate.sh on a machine without awh installed:** the hook script already degrades with a warning; wiring it must preserve that (a missing optional harness warns, never blocks an unrelated commit) - same posture as the hook's docs-build block.
- **A future task re-declares a workflow stub:** `test_ci_truth.sh` assert (c) fails the moment a placeholder-marker file lands under `.github/workflows/`, which is the regrowth guard - the honest path for a future declarer is shipping the real YAML or nothing.
- **Security-class:** the new workflow runs repo-pinned scripts on ubuntu runners with default token permissions; it MUST declare `permissions: contents: read` explicitly, and it introduces no new secret usage.
