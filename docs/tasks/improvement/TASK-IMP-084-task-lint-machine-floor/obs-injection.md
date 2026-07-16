---
artefact: observability-injection@1
task_id: TASK-IMP-084
branch_coverage_estimate: 100
created: 2026-07-16
verdict: pass (observability-injection-audit: vacuity justified honestly - run-to-completion CLI whose findings stream IS the observability; every failure branch announces)
---
# Observability injection - TASK-IMP-084

Honest vacuity statement: this task adds no service, no daemon, no network path, and no
background loop. The lint is a run-to-completion CLI whose entire observable surface is
stdout + exit code - and that surface is the deliverable, not a side channel:

- The findings stream IS the observability. Every finding carries
  `severity rule_id file:line message` with the rule_id verbatim from audit_rubric@2.0 -
  RUBRIC.md:3's own requirement ("Rule IDs MUST appear verbatim in audit reports so reports
  are diffable across iterations and operators") now holds for the machine floor too. An
  operator greps a rule family; a script consumes `--json` (same findings, same order, as
  data); the task-audit skill seeds its report's mechanical findings directly from the lines.
- Determinism is the monitoring contract. Two runs on identical input are byte-identical -
  no wall clock, no env-derived text, no randomness, bytewise-sorted traversal and output
  (spec §1.6). That makes drift DETECTABLE BY cmp: the task-audit skill already names
  `deterministic_drift` as a self-audit anomaly signal, and t01_cli_and_determinism asserts
  the byte-identity (text and --json) on every suite run, so a regression that introduces
  clock/env/random noise fails the gate the day it lands.
- Exit codes are the gate surface: 0 iff zero error-severity findings, 2 otherwise; `info`
  findings (the zero-clause TRACE-001 note) never flip the exit, so a future run-gates
  wiring gets a clean boolean without parsing.
- Every failure branch announces; none can go silent by construction. Exotic YAML -> FM-001
  naming the offending line (never a silent skip - the spec's stated reason for refusing a
  real YAML dependency); unreadable input or a template the lint does not handle ->
  `template_ambiguous` at error severity with a per-file stop (never a guess); usage errors
  -> stderr + exit 2. The one deliberate non-announcement: CRLF/BOM normalization is
  reported nowhere, per spec §3 (content bytes are the corpus's business).
- No logging added, deliberately: a "linting file X..." progress line would violate the
  byte-identity contract's spirit (output varying with corpus size in ways findings don't)
  and tempt callers to parse chatter instead of findings. Silence on success is the signal.
- The standing detectors are the suite (t01-t08 under scripts/tests/run_all.sh:43's glob on
  every gate) and t07's live run of the INSTALLED copy - a payload or install regression
  that drops or breaks the vendored lint fails the same suite that gates the rules.

branch_coverage_estimate 100 refers to announced-failure branches: every error path in the
lint terminates in a finding or a stderr usage line; there is no catch-and-continue that
discards a defect unannounced (the only swallowed exceptions are unreadable directory
entries during FM-113's name scan, which degrade to "does not resolve" - itself an error
finding).
