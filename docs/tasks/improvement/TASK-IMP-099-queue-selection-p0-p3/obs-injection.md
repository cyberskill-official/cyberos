---
artefact: observability-injection@1
task_id: TASK-IMP-099
branch_coverage_estimate: 100
created: 2026-07-17
verdict: pass (observability-injection-audit: vacuity justified honestly - one prose line, one version field, test pins; the version string and the suite tail ARE the observable surface)
---
# Observability injection - TASK-IMP-099

Honest vacuity statement: this task adds no service, no daemon, no network
path, no background loop, and no executable code beyond one test function. It
rewrites one sentence and one version field in
modules/cuo/chief-technology-officer/workflows/ship-tasks.md and moves/adds
pins in tools/install/tests/test_workflow_helpers.sh. There is nothing to
instrument at runtime. The observable surface is versioning plus the gate, and
this task makes both strictly louder:

- The version IS the announcement. workflow_version 2.6.4 is the one field
  every consumer channel surfaces (manifest resume checks exit 3 on mismatch;
  ship-manifest.mjs verify names it; t09 pins it in all three vendored copies).
  A normative wording change without this signal would be invisible to a
  resuming run - that is precisely what the exact-pin discipline exists to
  prevent, and why the pins moved WITH the wording in one change.
- The retired scale now has a tripwire, not a convention. Before this task,
  nothing failed when prose taught MoSCoW ordering; after it, t13's negative
  grep fails naming file and up to three offending lines (with line numbers in
  the failure message) if the rule shape reappears in source or payload. The
  probe evidence in gate-log-draft.md E3 shows the pattern catching the exact
  retired wording and allowing the legacy-mapping parenthetical.
- Failure messages are diagnostic, not boolean: each of t13's four checks has
  its own message (which copy, which missing phrase, or the offending lines),
  so a red run states the repair without archaeology.
- No logging added, deliberately: prose and bash tests have no logger. The
  standing detectors are the suite (scripts/tests/run_all.sh glob), the
  pre-commit payload rebuild + check-version-sync.sh, and payload-gate.yml CI.

branch_coverage_estimate 100 refers to the only executable branches this task
adds: t13's exits (missing file, missing p0-p3 phrase, missing parenthetical,
MoSCoW shape present, payload version wrong, ok) - the green path runs on
every suite invocation over both copies, and each red path's message was
exercised by construction during implementation (probe on the retired wording;
per-check messages verified against the pattern). There is no
catch-and-continue and no path that passes with the rule absent.
