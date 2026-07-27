# TASK-IMP-147 — inspect-harden ingest decision

Ingested the inspect-harden package as four CyberOS skills
(`inspection-report-author/audit`, `harden-record-author/audit`), wired
`/inspect` and `/harden`, and clarified that ship-tasks “harden a task”
(`class: improvement`) is not `/harden` (inspection remediation).

## Known fix landed

`harden-plan.mjs` 1.0.1 honours `operator_prerequisites`. Regression:
shopass.r2 INS-F-0002 must not classify as `agent`.

## Contract

Spec ≥1.2 ledger is 75 disciplines; INSPECT-SPEC 1.0 goldens stay valid via
version-gated lint (69 rows).

## Non-goals retained

No live shopass patch apply; no MCP tools; no merge of `/harden` into
`/ship-tasks`.
