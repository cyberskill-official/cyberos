---
artefact: observability-injection@1
task_id: TASK-IMP-097
branch_coverage_estimate: 100
created: 2026-07-17
verdict: pass (observability-injection-audit: vacuity justified honestly - documentation plus a grep gate; the suite tail and the recorded greps ARE the observable surface)
---
# Observability injection - TASK-IMP-097

Honest vacuity statement: this task adds no service, no daemon, no network
path, no background loop, and no executable code beyond one test function -
it writes a runbook section into tools/install/docs/index.md, one pointer
line into modules/cuo/chief-technology-officer/workflows/ship-tasks.md, and a
grep gate into tools/install/tests/test_full_sdp_payload.sh. There is nothing
to instrument at runtime. The observable surface is the gate and the recorded
evidence, and this task makes that surface strictly louder:

- Presence is now measured, not assumed. Before this task, none of the sandbox
  facts existed in any consumer document; after it, every run of
  test_full_sdp_payload.sh greps a scratch-built GUIDE.md for the section
  heading and its two load-bearing patterns (local clone with local ref move;
  manual hook replay with `--no-verify` plus recorded evidence). A regression
  is a named FAIL line in the suite tail, per phrase - t09 has five greps with
  five distinct messages, so the failure names the sentence that vanished.
- The runbook itself is an observability rule for humans: its first entry
  mandates RECORDING replayed hook obligations and their outputs in the commit
  message or gate log - it converts the previously silent `--no-verify` into
  an evidenced act. The section's own words define the detection ("without
  the record it is a skipped gate").
- The pointer line is single and countable: `grep -c 'Running CyberOS under
  sandboxed agents'` over ship-tasks.md returns 1, recorded in
  gate-log-draft.md - drift toward duplicated rule text is visible as a count
  change at review time.
- No logging added, deliberately: a docs file and a bash test have no logger;
  the standing detectors are the suite (discovered by scripts/tests/run_all.sh
  glob over tools/install/tests/test_*.sh), the pre-commit payload rebuild,
  and payload-gate.yml CI re-proving the build on every push.

branch_coverage_estimate 100 refers to the only executable branches this task
adds: t09's five grep branches (each exercised green by the current GUIDE and
each exercised red-by-construction via its distinct fail message when the
phrase is absent - the messages were verified against a scratch payload during
implementation). There is no catch-and-continue and no path that passes
without every phrase present.
