---
artefact: code-review@1
task_id: TASK-IMP-068
reviewer: agent (code-review-author) - human verdict pending at HITL gate 1
created: 2026-07-12
verdict: pass (code-review-audit) - awaiting human review acceptance
---
# Code review packet - TASK-IMP-068

## Diff summary (9 files)
new: tools/cyberos-install/check-version-sync.sh (comparator, 6 readers, exit 0/10/2)
new: tools/cyberos-install/tests/test_check_version_sync.sh (t01-t10)
new: .github/workflows/payload-gate.yml (push+PR, 4 path filters, build-into-temp + check, timeout 5m)
new: .githooks/pre-commit (trigger match -> engine rebuild -> check; abort on failure)
mod: tools/cyberos-install/build.sh (VERSION validated at TOP, before rm -rf; 0.0.0 fallback removed)
mod: .github/workflows/version.yml (inline build+check proof between apply and push)
mod: .pre-commit-hooks/cyberos-payload-build.sh (cross-reference comment; behavior unchanged)
mod: docs/deploy/RELEASE.md (enforcement wording replaces the aspirational claim)

## §1 clause -> named test -> status
| clause | test(s) | status |
|---|---|---|
| #1 comparator contract (6 artifacts, exit 0/10/2) | t01, t02, t03 | passed |
| #2 payload-gate.yml wiring | t06 | passed |
| #3 fail-fast build, no 0.0.0 | t04, t05 | passed |
| #4 .githooks/pre-commit semantics | t07, t08 | passed |
| #5 RELEASE.md tells the truth | t09 | passed |
| #6 no network, < 3 min (timeout 5m ceiling) | t06 | passed |
| #7 version.yml inline proof | t10 | passed |

## Edge-case matrix: 12/12 rows covered (see matrix "covered by" column)

## Deviations from spec (all recorded)
1. marketplace.json version lives at `metadata.version`, not on the plugin entry - task §1 #1 and §3
   wording corrected to match reality (behavioral intent unchanged: the marketplace manifest stamp
   is guarded).
2. build.sh guard placed at the TOP of the script (before `rm -rf "$out"`), stronger than the spec's
   minimum: an invalid VERSION now also cannot DELETE an existing payload.

## Reviewer attention points (honest flags for the human)
1. .githooks/pre-commit now fires for every contributor clone (core.hooksPath is repo-tracked via
   git config, set per clone) - contributors who never ran the config keep current behavior; CI is
   the backstop either way.
2. The hook adds ~5-10s to commits touching payload sources (full rebuild + check). Non-trigger
   commits are unaffected (measured no-op).
3. version.yml now fails the bump job if a payload cannot build at the new version - intended
   (a bump that cannot ship should be loud), but it is a new way for that job to go red.
4. shellcheck was unavailable in the implementation environment; `bash -n` clean on all scripts,
   suite green. Run shellcheck in CI review if desired.

## Verdict requested
Review acceptance (HITL gate 1): approve to advance reviewing -> ready_to_test, or reject with
findings to route back to ready_to_implement.
