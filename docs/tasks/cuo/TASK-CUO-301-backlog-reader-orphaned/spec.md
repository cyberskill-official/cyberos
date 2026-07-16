---
id: TASK-CUO-301
title: "ship-tasks queue is permanently empty: backlog_reader parses a table BACKLOG.md that nothing generates"
template: task@1
type: bug
module: cuo
author: "@stephencheng"
department: engineering
status: done
priority: p0
severity: sev1
created_at: 2026-07-14T00:00:00+07:00
ai_authorship: co_authored
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
first_bad_commit: null
regression_test: modules/cuo/tests/test_backlog_reader_specs.py::test_parses_live_backlog
incident: null
---

# ship-tasks queue is permanently empty

## Reproduction

```
1. cd modules/cuo
2. python3 -c "from pathlib import Path; from cuo.core.backlog_reader import parse_backlog, next_eligible; \
   r = parse_backlog(Path('../../docs/tasks/BACKLOG.md')); print(len(r), next_eligible(r))"
3. observe: 0 rows, next_eligible = None
```

**Environment**: any checkout of this repo, any Python.
**Frequency**: always.

## Expected vs observed

| | |
|---|---|
| **Expected** | `parse_backlog` returns one row per task, and `next_eligible` returns the first `ready_to_implement` task whose dependency cone is `done`. STATUS-REFERENCE §1.1 promises `ship-tasks` picks up eligible work. |
| **Observed** | 0 rows. `next_eligible` returns `None`. `ship-tasks` reports "no eligible task" regardless of backlog contents. The applier's `_rewrite_status_cell` also no-ops, because it bails on any line not starting with `\|`. |

## Blast radius

- **Who is affected**: every consumer of the ship pipeline — `ship-tasks`, `cyberos-cuo drain`, `status_server`, `brief_generator`.
- **Since when**: since BACKLOG.md was regenerated in bullet form. Predates the task->task rename; the rename merely surfaced it.
- **Workaround**: none. The queue is simply dead.
- **Data integrity**: no corruption. The failure is read-side and write-side no-op, not write-side wrong. Nothing to backfill.

## Root cause

`modules/cuo/cuo/core/backlog_reader.py:29` compiles `_TASK_ROW_RE` against a markdown
**table** row (`^\|\s*(TASK-[A-Z]+-\d+)\s*\|...`). `docs/tasks/BACKLOG.md` contains 0
table rows and 357 bullet rows. Nothing has generated the table shape in a long time:
BACKLOG.md's own header declares "Source of truth = task frontmatter", and
`render-status-hub.mjs:6` names its inputs as "task frontmatter, CHANGELOG.md, VERSION"
— it never opens BACKLOG.md at all. The reader was left pointing at an orphan.

The failure is silent because an empty parse is not an error: `re.match` returning
`None` on every line is indistinguishable from an empty backlog.

## Fix

`parse_specs()` hydrates the queue from the 507 `docs/tasks/<module>/TASK-*/spec.md`
frontmatters — the same source `render-status-hub.mjs` reads, so the CLI and the status
page can never again disagree about what is eligible. `parse_backlog()` keeps the table
path for back-compat and falls back to spec mode when the table yields nothing.

## Regression test

```
modules/cuo/tests/test_backlog_reader_specs.py::test_parses_live_backlog
```

Red at `HEAD~1` (returns 0 rows), green at `HEAD` (returns 507).

## Edge cases

| category | trigger | covered by |
|---|---|---|
| malformed | frontmatter status carries a trailing YAML comment (`status: on_hold  # was "blocked"`) | `test_strips_yaml_comment` |
| empty | a `TASK-*/` dir with no `spec.md` | `test_skips_dir_without_spec` |
| boundary | a table-shaped BACKLOG still parses in table mode | `test_table_mode_still_works` |

## Prevention

A parser whose "no matches" path is indistinguishable from "empty input" will fail
silently forever. `parse_specs` now returns a count the caller can assert on, and the
regression test pins it against the real backlog rather than a fixture — a fixture
would have kept passing through all of this.
