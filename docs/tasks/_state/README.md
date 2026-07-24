# Task state engine receipts (TASK-IMP-144)

`receipts/` holds content-addressed transition receipts written by every successful
`backlog-mutate.mjs flip` (and therefore by `task-state.mjs transition`).

`regen_backlog` refuses to invent status edges: if frontmatter `status` would change an
existing BACKLOG cell without a matching receipt for `(task_id, from, to)`, regen exits
non-zero and writes nothing.

Do not hand-edit these files to bypass HITL gates — gate transitions still require
`--verdict-by` / `--verdict-evidence` (and mint IMP-143 verdict artifacts under
`docs/tasks/_verdicts/`).
