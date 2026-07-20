---
artefact: observability-injection@1
task_id: TASK-IMP-086
branch_coverage_estimate: 100
created: 2026-07-16
verdict: pass (observability-injection-audit: vacuity justified honestly - a one-shot tracked-markdown chore with no runtime surface; the recorded gate-log evidence IS the observability, and every failure branch of the one write announces)
---
# Observability injection - TASK-IMP-086

Honest vacuity statement: this task ships no service, no daemon, no CLI, no background loop - it is a one-shot content repair to a tracked markdown index. There is nothing to instrument at runtime because nothing runs after the change lands. The spec prices this in: all four ACs are ops-verified against RECORDED evidence, and that record is the observability:

- The gate log is the observable surface. gate-log-draft.md E0-E6 hold the exact commands and verbatim outputs for the regenerator trial, parity (87 folders = 87 rows), tally-vs-header equality, duplicate-stem scans, per-row byte-verbatim rechecks, and the diff footprint. Every E2-E6 command is a pure read: any reviewer re-runs them against the working tree and must reproduce the outputs byte-for-byte - drift between the log and the tree is itself detectable by re-execution, which is the same contract a test suite would give this chore.
- git is the flight recorder. The change is one insertion-only hunk (`@@ -239,0 +240,14 @@`, +14/-0) in a tracked file on branch batch/2-workflow-helpers; `git diff` / `git log -p docs/tasks/BACKLOG.md` replay the entire behavior of this task forever. There is no side channel to lose.
- Every failure branch of the one write announces; none can go silent. The splice script HALTs naming the folder on missing spec.md, unparseable frontmatter, id/stem mismatch, or a multi-line title (never inventing a row, per spec §3), and hard-asserts 067/082 adjacency and rows==14 before writing. The regenerator trial's failure mode announced through the recorded diff (E1) instead of through repo damage - the dry-run wrote only under /tmp/dry86.
- Standing detection of FUTURE drift is deliberately NOT added here: the spec rules a permanent repo-wide parity test out of scope (it would go red today on other sections' pre-existing drift, e.g. the Totals line's 155-vs-158 done). The go-forward observers are the batch sibling TASK-IMP-085's backlog-mutate helper (single writer for future row flips) and STATUS-REFERENCE §1's standing repair direction; when the other sections get reconciled, the spec names the parity test as the follow-up.

branch_coverage_estimate 100 refers to the splice script's failure branches: four HALT guards and two asserts, each terminating with a named cause before any byte is written; the recorded run took none of them (14/14 parsed clean), and the post-image rechecks (E5) re-proved the happy path from the file itself.
