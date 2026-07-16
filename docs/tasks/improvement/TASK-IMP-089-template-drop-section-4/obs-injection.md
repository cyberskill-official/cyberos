---
artefact: observability-injection@1
task_id: TASK-IMP-089
branch_coverage_estimate: 100
created: 2026-07-17
verdict: pass (observability-injection-audit: vacuity justified honestly - prompt-text edit with no runtime; the gating suite is the standing detector and every failure branch announces)
---
# Observability injection - TASK-IMP-089

Honest vacuity statement: this task adds no service, no daemon, no network path and no
runtime branch - it edits prompt text (the install template) and the bash suite that gates
it. Nothing ever executes a template; that is precisely the incident that created this
suite (test_template_schema.sh header: "nobody runs it" is not "nobody depends on it"), so
the suite IS the observability surface:

- Failures name the defect class. Every red arm prints the scenario name plus reason
  TOKENS from the shared oracle (`duplicate-out-of-scope-H2`, `invariants-not-at-##4`,
  `stray-##5-heading`, `invariants-body-missing`, `payload-copy-diverges-from-source`) -
  a future regression is diagnosed from the gate line alone, no re-derivation.
- The detector itself is monitored. t08_duplicate_reintroduction_fails is the canary for
  the ORACLE: if the regex decays until it no longer sees the old section-4 shape, t08b
  goes red - the check cannot rot into a permanent green while the thing it guards
  regresses. And it demands the duplicate token specifically, so an oracle failing for an
  unrelated reason does not impersonate detection.
- Every failure branch announces; none skips. Missing template -> named fail with the
  consumer consequence; scratch build exit != 0 -> "scratch build.sh failed" (build.sh
  itself prints the cause, e.g. its up-front VERSION validation); missing vendored file ->
  the exact payload path (t07's filter-matches-nothing lesson applied); byte-divergence ->
  cmp token. The `[ "$FAIL" -eq 0 ]` tail keeps the suite's exit code the gate boolean.
- Standing wiring, zero new plumbing: scripts/tests/run_all.sh:43 already globs
  scripts/tests/test_*.sh, so the three arms run on every gate sweep from this commit on.
- No logging added, deliberately: a template has no runtime to log from, and the suite's
  ok/FAIL lines plus `pass=N fail=N` summary are already its complete, greppable record.

branch_coverage_estimate 100 refers to announced-failure branches in the additions: every
conditional in shape_why and the three arms terminates in an ok or a reasoned fail; there
is no catch-and-continue and no silent early return.
