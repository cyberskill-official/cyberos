# TASK-IMP-100 observability injection

A read-only, one-shot CLI: no service lifetime, no state transitions, no external IO to span.
Its observable surface IS its output, and that surface is the deliverable:

- **The report is the telemetry.** Per-rung verdicts + drift_score + the recommendation, with
  every red carrying its reason string (`SPEC DRIFT`, `UNCOMMITTED CLAIM`, `cited suite FAILS
  now`) - greppable, and quoted verbatim into the HITL gate question.
- **Error branches** exit-coded and named: usage (2), task unresolvable (3). A bad task never
  produces a non-zero exit - the verdict is not the tool's to enforce.
- **`--out` confirms to stderr** (`task-reconcile: wrote <path>`), stdout stays the artefact.
- **No PII/secrets**: task ids, paths, statuses, hashes.

Branch coverage: every recommendation branch and every rung verdict asserted across t01-t05.
