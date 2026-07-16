# TASK-IMP-091 code review

Reviewer: parent ship-tasks agent (batch 3). Diff: `scripts/migrate_improvement_to_task.py`
(+29/-18), new `scripts/tests/test_regen_backlog.sh` (3 scenarios).

## Clause -> proof

| Clause | Requirement | Proof |
|---|---|---|
| 1.1 | one row per folder for EVERY status | ACTIVE constant deleted (lines 19-20) and its filter replaced by `for stem, st, title, kl in sorted(rows)`; t03_every_status_emitted ok (12 statuses, 12 rows, row count == folder count) |
| 1.2 | Totals recomputed from a frontmatter tally | `status_line(tally)` feeds both the module header and Totals; t02_totals_true ok (compared against a tally computed without importing the script) |
| 1.3 | today's corpus regenerates byte-identical to the committed section | t01_live_corpus_parity ok (`cmp` against `git show HEAD:docs/tasks/BACKLOG.md`) |
| 1.4 | per-status fixture emits every row | t03_every_status_emitted ok (incl. the off-ramps cannot_reproduce/duplicate that STATUS_ORDER never listed) |
| 1.5 | suite discovered by run_all | `run_all.sh` output lists `ok test_regen_backlog.sh` (gate log) |

## Judgment

- **Correctness vs the recorded failure**: TASK-IMP-086's gate-log E1 is the specification of the
  bug - regen emitted zero rows for fourteen done tasks and a Totals line three short. t01 now
  proves the opposite property against the real corpus, and t03 proves the filter cannot creep
  back for any legal status.
- **The halt is the real hardening**: the old code printed "EXCLUDED" to stderr and wrote the
  backlog anyway. A warning that still writes a wrong file is the silent-drift class in disguise;
  the exit now precedes every write, and t03's halt half asserts the file's sha256 is unchanged.
- **status_line unification**: header and Totals share one formatter, so they cannot disagree -
  the drift shape the 086 incident showed (header saying one thing, rows another).
- **Unlisted statuses**: sorted-append after STATUS_ORDER means a status a task actually carries
  can never vanish from a count, even if STATUS_ORDER is not updated. Defensive by design.
- **Suite safety**: every scenario copies the script into a scratch tree because ROOT resolves
  from `__file__`; the live BACKLOG is proven byte-untouched (`git status --short` empty).
- **Security**: none - read-compute-write over tracked markdown.
- **AI-specific**: the file's own preamble ("lists ONLY remaining work") was the bug's charter and
  was rewritten with the code; leaving it would have re-taught the next reader the defect.

Verdict: no open findings.

HALT: review acceptance (reviewing -> ready_to_test) is a human gate.
