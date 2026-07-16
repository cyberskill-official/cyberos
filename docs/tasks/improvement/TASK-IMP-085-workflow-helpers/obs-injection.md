---
artefact: observability-injection@1
task_id: TASK-IMP-085
branch_coverage_estimate: 100
created: 2026-07-16
verdict: pass (observability-injection-audit: vacuity justified honestly - run-to-completion CLIs whose exit codes and envelopes ARE the observability; every refusal branch announces)
---
# Observability injection - TASK-IMP-085

Honest vacuity statement: this task adds no service, no daemon, no network path,
and no background loop. Both tools are run-to-completion CLIs whose entire
observable surface is stdout + exit code - and that surface is the deliverable,
the two contracts' own reporting requirements made machine-checkable:

- The exit codes ARE the gate wiring. ship-manifest's 0/3/4/5 is the workflow's
  staleness order verbatim (a caller branches on the code without parsing prose);
  backlog-mutate's 6/7 are the drift-refusal and uniqueness gates that previously
  existed only as SKILL.md sentences. `--help` documents every code (t07 greps
  them), so the doctrine and the tool cannot silently disagree about what a
  failure means.
- Every refusal names its evidence. Exit 3 quotes both versions; exit 4 says what
  went stale and what is retained; exit 5 names the step, the skill, and the
  artefact path; exit 6 names the line (or BOTH lines on a duplicated row) and
  which byte check drifted; exit 7 names the pre-existing row's line. An operator
  reads the one line and knows what to do; nothing degrades to a bare non-zero.
- The resume line is the workflow's mandated telemetry: `resume <task-ID>: steps
  1-N verified (K artefacts, hashes OK), continuing at step M/31 (<skill>).
  routed_back_count=R` - echoed byte-exactly (t01/t03 compare the whole string),
  so a resumed session's first line of output is the audit-readable proof of what
  was verified.
- `--json` envelopes are the machine channel: stable-stringified (sorted keys),
  `ok`/`exit_code` mirroring the process exit, the mutation's pre/post lines
  (old_line/new_line/old_header/new_header) carried as data so the
  backlog-state-update artefact can be filled from the envelope without re-reading
  the file. Error envelopes carry the same shape with `error` (t07 parses both).
- Determinism is the monitoring contract. Identical inputs + identical args =
  byte-identical output; the ONE contract-required clock (manifest timestamps) is
  injectable (`CYBEROS_NOW`/`--now`, documented in --help) rather than the rule
  weakened - t07 cmp-proves manifests, envelopes, and mutated backlogs across
  reruns, so any regression that introduces wall-clock or random noise fails the
  gate the day it lands. Ordering inside the manifest uses step indices, never
  timestamps (SHIP-MANIFEST.md field rule).
- The one deliberately-loud degradation: when verify can find NO current
  workflow version (no flag, no discoverable doc), the check is skipped with a
  printed `note:` line - visible in text mode and carried in the envelope - never
  a silent pass pretending it compared something.
- Silent-by-design: two-phase writes announce nothing on the happy path (the
  rename either happened or the old bytes are intact - t02's planted-tmp arm
  proves the reader's indifference), and backlog-mutate adds no "mutating file..."
  chatter that would tempt callers to parse prose instead of the envelope.
- The standing detectors are the suite (t01-t09 under scripts/tests/run_all.sh's
  glob on every gate run) and t08's live run of the INSTALLED copies - a payload
  or install regression that drops or breaks either vendored tool fails the same
  suite that gates the contracts.

branch_coverage_estimate 100 refers to announced-failure branches: every refusal
and staleness path terminates in a named message + documented exit code (or a
stderr usage line + exit 2); there is no catch-and-continue that discards a
defect unannounced. The only swallowed exceptions are unreadable-file probes
(sha256File / doc discovery), which degrade to explicit outcomes - exit 4/exit 2
findings or the loud skip note - never to silence.
