# TASK-IMP-136 — implementation evidence (batch/8, 2026-07-23)

Implemented mid-wave under the batch/8 shared-tree partition (5 sibling workers editing
other parts of this tree; no commits, no full-suite runs, no payload rebuilds from this
worker — a final sequential pass owns those). HITL: both human-acceptance gates remain
ahead; nothing here advances any status.

## What changed and why

| File | Change | Why (spec clause) |
|---|---|---|
| `.github/workflows/caf-evals-gate.yml` | NEW — job `caf-evals` runs `python3 core/evals/validate.py --all` (workdir `tools/caf`) + `bash scripts/caf_precommit_check.sh`; PR paths `tools/caf/**` + `scripts/caf_*` (plus the benchmark-gate surfaces), weekly cron, `workflow_dispatch`, `permissions: contents: read`; job `benchmark-gates` runs the two new checker suites | §1.1 (the CAF suite ran in NO CI — nested `tools/caf/.github/` is never read); TASK-IMP-140 wiring rides this workflow per its spec's CI-arrival sentence |
| `.githooks/pre-commit` | awh block added: `awh_trigger='^modules/'` gated via the existing `matches()` herestring; harness-present → `sh .pre-commit-hooks/awh-gate.sh` blocks on RED; harness-absent → WARN, never blocks | §1.2 (the awh commit-time gate existed only as a claim in a file no tool reads); SIGPIPE idiom preserved per the hook header + audit ISS-004 |
| `.pre-commit-config.yaml` | DELETED | §1.3 (dead mechanism: `core.hooksPath=.githooks` is the real hook path; every live claim now covered by the real hook). Removal reason recorded here + prepared commit body + CHANGELOG text below |
| 9 stub workflows | DELETED (all dispositioned DELETE) | §1.4; per-file judgment + evidence in `stub-disposition.md` (same folder); implement-override conditions (complete embedded YAML AND live dependencies) held for none of the nine |
| `scripts/tests/test_benchmark_ci_truth.sh` | NEW — the §1.5 four-assert regrowth guard, t01–t06 | §1.5; auto-registers via `run_all.sh`'s glob |
| `CHANGELOG.md` | NOT edited (outside this worker's ownership — high-collision file) | §1.6 — exact entry text prepared below for the final pass |

## Deviations (recorded)

1. **Suite filename**: spec names `scripts/tests/test_ci_truth.sh`; landed as
   `scripts/tests/test_benchmark_ci_truth.sh` because the batch/8 ownership partition
   reserves `scripts/tests/test_benchmark_*.sh` for this worker. Test ids t01–t06 match
   the spec verbatim. Final pass MAY `git mv scripts/tests/test_benchmark_ci_truth.sh
   scripts/tests/test_ci_truth.sh` (one command, no content change; the run_all glob
   discovers either name) to match the AC test paths byte-for-byte.
2. **CHANGELOG.md not edited** (ownership): paste-ready text below; t03/t06 fail loudly
   until pasted, naming this file.
3. **AC 1's "deliberately broken fixture" negative** uses `validate.py --run` on a
   corrupted copy of `G01-clean-run` (R5-BAD-STATUS class), not `--all` on a scratch
   tree: `--all` is ALREADY red at HEAD on pre-existing regressions (below), so an
   `--all` negative would be vacuous. The positive/negative pair proves discrimination.

## Finding surfaced (pre-existing, NOT fixed here — out of scope/ownership)

`python3 core/evals/validate.py --all` exits **1 at HEAD** (measured 2026-07-23, no
sibling edits under `tools/caf/`):

```
[FAIL] B17-config-placeholder           expect=fail → expected ['CONFIG-BAD-ENUM', 'CONFIG-PLACEHOLDER'], got []
[FAIL] B18-config-autoprotect           expect=fail → expected ['R3-PROTECTED'], got []
38/40 fixtures OK — REGRESSIONS PRESENT
```

Two expected-fail fixtures now pass validation — the validator lost two rules (or the
fixtures drifted) and NOTHING noticed, because the suite ran in no CI: this is finding
H9 demonstrating itself. Consequence: the new `caf-evals-gate` workflow will be RED on
its first run until a CAF self-improvement cycle (CAF's own change discipline: one change
per cycle, never weaken a fixture) restores B17/B18. **An honest red on a real regression
is the gate working.** Open item for the HITL reviewer: schedule that CAF cycle (it is
not TASK-IMP-136's scope, and `tools/caf/**` was not this worker's ownership).

## Verification (verbatim, focused only — no run_all, no builds, no commits)

```
$ bash -n scripts/tests/test_benchmark_ci_truth.sh   # + hook + workflow files
bash -n OK: scripts/tests/test_benchmark_ci_truth.sh
bash -n OK: .githooks/pre-commit
$ actionlint .github/workflows/caf-evals-gate.yml
actionlint OK: caf-evals-gate.yml        # dir-wide actionlint: only pre-existing warnings in other workflows
$ bash scripts/caf_precommit_check.sh
caf-precommit: OK - all 10 gated module(s) declare an audit-profile.yaml
```

Hook dry-test — scratch git repo, hook invoked DIRECTLY (no commit created anywhere):

```
--- case A: unrelated staged (README.md)          -> no awh mention, exit 0   [AC 2 non-trigger]
--- case B: modules/ai staged, harness present    -> gate ran,       exit 0
--- case C: harness present + RED awh-gate stub   -> "cyberos: awh module gate RED - fix the regression or recapture the baseline", exit 1
--- case D: no awh on PATH, no tools/awh/harness  -> "cyberos: WARN awh harness not found ... skipped", exit 0   [degrade posture]
```

Suite dry-run at HEAD (mid-wave):

```
$ bash scripts/tests/test_benchmark_ci_truth.sh
  ok   t01_caf_gate_wired
  ok   t02_awh_hook_wired_safely
  FAIL t03_dead_config_gone: CHANGELOG.md top entry does not name the .pre-commit-config.yaml removal — paste the prepared entry ...
  ok   t04_no_stub_survives
  ok   t05_self_test_negative_paths
  FAIL t06_changelog_records_sweep: (4 asserts) CHANGELOG.md top entry lacks 'caf-evals-gate' / 'awh' / '.pre-commit-config.yaml' / '9 deleted'
benchmark-ci-truth: 4 passed, 5 failed
```

## Dry-run / final-pass classification

| Check | Classification | Status mid-wave |
|---|---|---|
| t01_caf_gate_wired (incl. validator-discrimination negative) | dry-runnable now | PASSING |
| t02_awh_hook_wired_safely | dry-runnable now | PASSING |
| t03_dead_config_gone | needs final-pass (CHANGELOG paste) | file-absence half passing; CHANGELOG half failing, names the fix |
| t04_no_stub_survives | dry-runnable now | PASSING |
| t05_self_test_negative_paths | dry-runnable now | PASSING |
| t06_changelog_records_sweep | needs final-pass (CHANGELOG paste) | failing, names the fix |
| caf-evals-gate workflow first CI run | needs final-pass / post-merge | will be RED on pre-existing B17/B18 (see finding above) |

## CHANGELOG entry — paste into the existing `## [Unreleased]` block (final pass)

```markdown
Added
- root CI gate `caf-evals-gate.yml`: the CAF eval suite (`validate.py --all`, all 40
  fixtures) + `caf_precommit_check.sh` now run on PRs touching `tools/caf/**` /
  `scripts/caf_*` and on a weekly cron — previously the only workflow naming them sat
  nested under `tools/caf/.github/` where GitHub Actions never reads, so the suite ran in
  no CI at all. The same workflow's second job runs the TASK-IMP-140 benchmark-gate
  checkers. (TASK-IMP-136)
- `.githooks/pre-commit` now runs the awh module gate (`.pre-commit-hooks/awh-gate.sh`)
  for staged `modules/` sources via the hook's matches() idiom; a missing awh harness
  warns and never blocks, a RED gate blocks the commit. (TASK-IMP-136)

Removed
- `.pre-commit-config.yaml` — dead mechanism: the repo's hook path is
  `core.hooksPath=.githooks` and no tool read the framework config; its every live claim
  (payload build, docs build, awh gate) is covered by `.githooks/pre-commit` directly.
  (TASK-IMP-136)
- the 9 always-green stub workflows (single-echo placeholders, auto-generated
  2026-05-17): 9 deleted, 0 implemented — an always-green check manufactures false
  confidence under a gate-shaped name. Per-file disposition + declaring tasks in
  `docs/tasks/improvement/TASK-IMP-136-ci-caf-evals-and-stub-truth/stub-disposition.md`;
  `test_benchmark_ci_truth.sh` fails the moment the placeholder marker regrows.
  (TASK-IMP-136)
```

## Open items for the HITL reviewer / final pass

1. Paste the CHANGELOG entry above (t03/t06 flip green in the same commit).
2. Optional `git mv` to the spec's exact suite filename (deviation 1).
3. Branch-protection query before merge (Operator steps in `stub-disposition.md`).
4. Schedule the CAF self-improvement cycle for the B17/B18 fixture regressions.
5. `docs/deploy/ci-and-local-checks.md:27` still says "install the pre-commit hooks per
   `.pre-commit-config.yaml`" — outside this worker's ownership. Suggested replacement
   line: "One-time: `git config core.hooksPath .githooks` (the hooks are repo-tracked;
   no framework install needed)."
6. AC 1's scratch-branch `validate.py --all` break test at the review gate can reuse the
   t01 negative (corrupt any G-fixture's task status).
