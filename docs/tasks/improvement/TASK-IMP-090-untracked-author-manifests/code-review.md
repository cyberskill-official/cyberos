# TASK-IMP-090 code review

Reviewer: parent ship-tasks agent (batch 3). Diff: `modules/skill/task-author/SKILL.md` (1 line),
`tools/install/install.sh` (.workflow seed), `tools/install/tests/test_install_hygiene.sh` (t07),
`docs/tasks/.workflow/.gitignore` (+1), 3 index removals, 1 new `_audits` record.

## Clause -> proof

| Clause | Requirement | Proof |
|---|---|---|
| 1.1 | SKILL default names .workflow | SKILL.md:184 `manifest_path: <from caller; default: docs/tasks/.workflow/task-author.<slug>.manifest.json>` |
| 1.2 | seed covers both patterns; append-once on legacy seeds | install.sh:48-54 (create-if-absent both patterns; `grep -qxF` guard + trailing-newline heal); t07_workflow_gitignore_patterns ok; live scratch install seed = `*.ship.json` + `*.manifest.json` (gate log) |
| 1.3 | the three batch manifests leave the index | `git status`: `D docs/tasks/.workflow/task-author.improvement-batch{,-2,-3}.manifest.json`; `git ls-files docs/tasks/.workflow` returns only tracked phase artefacts and .gitignore - zero `*.manifest.json` (gate log) |
| 1.4 | tracked approval record exists | `docs/tasks/_audits/IMPROVEMENT-BATCHES-2026-07-16.md`, 121 lines, 27 member-id mentions |
| 1.5 | t07 covers both seed paths | suite 17 passed, 0 failed |

## Judgment

- **Correctness vs ticket**: all four moves of the recorded IMP-11 decision landed. The
  manifests stay on disk (session state survives), leave the index (no churn), and the record
  the operator actually needs moved to `_audits/` where module audit records already live.
- **Blast radius**: install.sh's seed block is 7 lines; the append-once guard makes a re-vendor
  idempotent, and operator lines in an existing seed are never rewritten.
- **Failure mode if wrong**: a consumer's seed gaining the pattern twice (guarded by `grep -qxF`,
  asserted by t07's second install) or a manifest reappearing tracked (guarded by the local
  .gitignore, asserted by AC 3's ls-files).
- **Security**: none. .gitignore governs the index only; manifests carry ids, titles, statuses.
- **History**: `git rm --cached` removes from the index going forward; history keeps the blobs -
  deliberate, no rewrite (spec Non-Goals).
- **AI-specific**: index operation was the single git mutation the spec authorized; no others.

Verdict: no open findings.

HALT: review acceptance (reviewing -> ready_to_test) is a human gate.
