# Inspect → Harden skill chain

Four skills ingested from the inspect-harden package (TASK-IMP-147):

| Skill | Role |
|---|---|
| `inspection-report-author` | Read-only `/inspect` → `inspection-report@1` |
| `inspection-report-audit` | Machine floor + rubric; pass → may `/harden` |
| `harden-record-author` | `/harden` remediation → `hardening-record@1` |
| `harden-record-audit` | Session honesty audit |

**Not the same as ship-tasks.** Backlog `class: improvement` ("harden a task")
runs `/ship-tasks`. `/harden` only consumes a lint-clean inspection report.

Known failures (absence claims, truncated listings, planner actor misclass) are
in each author's `references/FAILURE_MODES.md`.
