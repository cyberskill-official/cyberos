---
id: TASK-IMP-083
title: install lands the status-sync hook where core.hooksPath points
template: task@1
type: improvement
module: improvement
status: ready_to_implement
priority: p0
author: "@stephencheng"
department: engineering
created_at: 2026-07-16T11:45:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-074]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 hardening"
owner: Stephen Cheng (CTO)
created: 2026-07-16
shipped: null
memory_chain_hash: null
effort_hours: 3
service: tools/install
new_files: []
modified_files:
  - tools/install/install.sh
  - tools/install/uninstall.sh
  - tools/install/tests/test_install_hygiene.sh
source_pages:
  - "tools/install/install.sh step 6b (hk=\"$root/.git/hooks/pre-commit\" unconditionally; `grep -n hooksPath tools/install/install.sh` returns nothing - verified 2026-07-16)"
  - "cyberos's own .git config core.hooksPath=.githooks (a live affected repo: a CyberOS install into cyberos would write an inert hook)"
  - "IMPROVEMENT_HANDOFF.md IMP-02 (the sachviet run's hook only fired because sachviet leaves hooksPath unset)"
source_decisions:
  - "2026-07-16 Stephen: PLAN batch 1 approved with this item at p0."
---

# TASK-IMP-083: install lands the status-sync hook where core.hooksPath points

## Summary

The installer writes its status-sync pre-commit hook to `.git/hooks/pre-commit` unconditionally. Git runs hooks from `core.hooksPath` when that config is set, so on any repo using a custom hooks directory (the cyberos repo itself is one) the installed hook is inert and the group-A status sync silently dies. Teach install and uninstall to resolve the effective hooks directory and apply the existing ownership rules there, with the summary naming the real path written.

## Problem

Step 6b hardcodes the location:

<untrusted_content source="tools/install/install.sh step 6b">
hk="$root/.git/hooks/pre-commit"
mkdir -p "$root/.git/hooks"
</untrusted_content>

`git config core.hooksPath` is never read (zero matches in the file). A repo configured with, say, `.githooks` - a common pattern, and exactly how the cyberos repo is set up - receives a hook git will never execute. The failure is silent: install prints "pre-commit hook v2 installed", the operator believes status sync is live, and `docs/status/` quietly lags every backlog write. The install.sh authors have already documented this exact bug class elsewhere in the file ("a guard that skips when its own tool is missing is indistinguishable from success").

## Proposed Solution

Resolve the effective hooks directory once: `hooks_dir="$(git -C "$root" config core.hooksPath || true)"`; empty means `.git/hooks` as today; a relative value resolves against `$root`; an absolute value is used as is. Then run the UNCHANGED ownership state machine (absent or ours-outright -> standalone v2; ours-v1-append -> upgrade; foreign -> marked append; ours-v2 -> keep) against `<hooks_dir>/pre-commit`, `mkdir -p`-ing the directory exactly as the current code does for `.git/hooks`. The summary line names the path actually written. Uninstall resolves the same way so it removes or unappends from where install wrote.

## Alternatives Considered

- Warn only, keep writing `.git/hooks`. Rejected: a warning fixes nothing; the repo still ships with dead sync, and the operator has to hand-port the hook. The resolution is three lines.
- Write BOTH locations. Rejected: git executes exactly one; the inert copy is litter that confuses the next ownership scan and doubles the uninstall surface.
- Refuse to install the hook when hooksPath is set. Rejected: hooksPath repos are precisely the disciplined repos most likely to want the sync; refusing punishes them for configuring git properly.

## Success Metrics

- Primary: on a hooksPath repo, a backlog-touching commit regenerates and stages `docs/status/` - hook fires from the configured directory. Baseline (today): hook never fires on such repos. Deadline: this task's final acceptance.
- Guardrail: on repos without hooksPath, the step's output and written bytes stay identical to today (regression case in the suite, asserted on every run).

## Scope

In scope: hooks-directory resolution in install step 6b and the matching resolution in uninstall; one summary line; hygiene-test scenarios.

### Out of scope / Non-Goals

- The hook bodies (standalone v2, append v2) - byte-identical to today.
- Husky/pre-commit-framework adapters (their directories are covered generically by hooksPath resolution; framework-native config formats are a separate item if ever wanted).
- The run-gates and docs-site sync paths (untouched).

## Dependencies

- None upstream. Cone-disjoint from TASK-IMP-082 (renderer) and TASK-IMP-084 (docs-tools + skill prose); ships in the same parallel batch.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted by the model from IMPROVEMENT_HANDOFF.md IMP-02 plus direct source verification; implementation follows under ship-tasks supervision.
- **Human review:** PLAN approved by the operator on 2026-07-16; spec audit and both HITL acceptance gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 install MUST resolve the effective hooks directory as: `core.hooksPath` when set and non-empty (relative values resolved against the repo root, absolute used as is), else `.git/hooks`.
- 1.2 The existing hook ownership state machine MUST run unchanged against `<hooks_dir>/pre-commit`: absent or ours-outright writes standalone v2; a foreign hook receives the marked POSIX append block; an ours-v1 append upgrades to v2; an ours-v2 is kept. The hooks directory is created with `mkdir -p` when missing.
- 1.3 The install summary's auto-sync line MUST name the path actually written (e.g. `pre-commit hook v2 installed at .githooks/pre-commit`).
- 1.4 When `core.hooksPath` is unset, every written byte and every summary word of step 6b MUST be identical to today's behavior.
- 1.5 uninstall MUST resolve the hooks directory the same way and remove the standalone form or strip the appended block from `<hooks_dir>/pre-commit`, never touching `.git/hooks/pre-commit` when hooksPath points elsewhere. In the same change, uninstall's ownership test MUST become the exact line-2 check install.sh already uses (`_cyberos_owns_hook`), replacing its current `head -5 | grep` heuristic - today a FOREIGN hook shorter than five lines that carries our appended block matches the heuristic and is deleted whole (uninstall.sh:24-28), destroying the user's hook: the exact silent-data-loss class install.sh's own comment documents and fixed on the install side.
- 1.6 Non-git repos MUST keep today's skip behavior ("skipped (not a git checkout)").
- 1.7 Hygiene coverage MUST land as a new scenario block in `tools/install/tests/test_install_hygiene.sh` (t05): hooksPath set + no hook file -> standalone lands there and fires on a backlog commit; hooksPath set + foreign hook -> append there, foreign exit code preserved; hooksPath unset -> regression-identical.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: §1 #1.1, #1.2) - hook lands in the configured directory - test: `tools/install/tests/test_install_hygiene.sh::t05_hookspath_standalone`
- [ ] AC 2 (traces_to: §1 #1.2) - foreign hook in hooksPath dir gets the marked append, exit code preserved - test: `tools/install/tests/test_install_hygiene.sh::t05_hookspath_foreign_append`
- [ ] AC 3 (traces_to: §1 #1.4) - unset hooksPath is byte-regression-identical - test: `tools/install/tests/test_install_hygiene.sh::t05_no_hookspath_regression`
- [ ] AC 4 (traces_to: §1 #1.5) - uninstall removes from the configured directory - test: `tools/install/tests/test_install_hygiene.sh::t05_hookspath_uninstall`
- [ ] AC 5 (traces_to: §1 #1.3) - summary names the real path - test: `tools/install/tests/test_install_hygiene.sh::t05_summary_names_path`
- [ ] AC 6 (traces_to: §1 #1.7) - the new scenarios run inside the existing hygiene suite - verify: `bash scripts/tests/run_all.sh` shows test_install_hygiene.sh ok with t05 counted (ops check recorded in the gate log).
- [ ] AC 7 (traces_to: §1 #1.6) - non-git target keeps the skip line - test: `tools/install/tests/test_install_hygiene.sh::t05_non_git_skip`
- [ ] AC 8 (traces_to: §1 #1.5) - uninstall on a 3-line foreign hook carrying our appended block strips only the block, foreign body byte-preserved - test: `tools/install/tests/test_install_hygiene.sh::t05_short_foreign_uninstall_preserved`

## 3. Edge cases

- Relative hooksPath (`.githooks`) vs absolute (`/tmp/hooks`) - both resolved; relative anchored at root, not at CWD (t05 exercises relative; absolute covered by the same resolver line, code-reviewed).
- hooksPath configured but directory absent - `mkdir -p` creates it, matching today's `.git/hooks` treatment; git would use it the moment it exists.
- hooksPath set to an empty string - treated as unset (`git config` returns empty; the resolver's emptiness check catches both).
- Worktrees: `git -C root config` reads the shared config, hooksPath applies worktree-wide - correct by construction; noted for reviewers, no special code.
- A hooksPath repo that ALREADY has an inert CyberOS hook at `.git/hooks/pre-commit` from an older install - out of scope to migrate automatically (touching a path git ignores is harmless); the summary's named path makes the situation visible. Recorded as a known leftover, not a failure.
- Uninstall meets a short foreign hook (under five lines) that carries our appended block: the current head-5 heuristic classifies it as ours and deletes the whole file - after this task the exact line-2 test strips only our block (AC 8). Found during this task's own authoring audit, 2026-07-16.
- Security-class: the append block stays POSIX-sh only and preserves the foreign exit code (existing invariant, re-asserted by AC 2); no new execution surface is introduced.

## 4. Out of scope / non-goals

Duplicated intentionally with `## Scope` for template conformance: hook bodies, framework adapters, and legacy-location migration are untouched.

## 5. Protected invariants this task must not weaken

- Never clobber a foreign hook (the length-independent ownership test stays exact).
- The appended block remains POSIX sh and preserves the foreign exit code.
- Payload sync doctrine: rebuild dist, version-sync, full suite before commit.
- HITL: both human-acceptance gates are recorded verdicts; the agent never sets done.

*End of TASK-IMP-083.*
