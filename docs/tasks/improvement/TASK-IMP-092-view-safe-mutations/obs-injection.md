---
artefact: observability-injection@1
task_id: TASK-IMP-092
branch_coverage_estimate: 100
created: 2026-07-17
verdict: pass (observability-injection-audit: vacuity justified honestly - run-to-completion CLI + prose; the mutation envelope IS the observability and now announces the correction it made)
---
# Observability injection - TASK-IMP-092

Honest vacuity statement: this task adds no service, no daemon, no network path,
no background loop - it changes arithmetic inside a run-to-completion CLI and adds
two doctrine paragraphs. The observable surface is the tool's stdout/exit contract,
and this task makes that surface STRICTLY LOUDER about the one thing it used to
keep quiet:

- The correction announces itself. Every mutation that rewrites a header reports
  `header_line`, `old_header`, `new_header` in the --json envelope (and "header
  retallied at line N" in the text message). Under the incremental adjust those
  fields could only ever show counts drifting by one; now a caller whose header
  said `(34 done)` sees `old_header: "## alpha  (34 done)"` next to the true
  `new_header` - the lie and its correction in one record. t10 asserts exactly
  this envelope, so the announcement is gated, not decorative.
- The 086 incident's silent enabler is structurally gone: the header "never
  screamed because the pre-existing corpus header already counted the unindexed
  done tasks, and the incremental count adjust inherited that baseline". A
  retallied header cannot inherit anything - each write is a fresh measurement of
  the rows, so index-vs-rows drift surfaces on the NEXT mutation at the latest,
  in the diff the operator already reviews (1 row + 1 header line, t11).
- Determinism is the monitoring contract, unchanged: identical input + args =
  byte-identical file and stdout (t07 cmp, text and --json). No clock, no env
  text, no randomness was added; the retally reads only the lines array it is
  about to write.
- Every failure branch still announces; none was added or silenced. Exits 2/6/7
  and their stderr wording are byte-for-byte untouched (t04/t05 assert them).
  The two deliberate non-announcements, both pre-existing: a bare header is left
  alone silently (by grammar it carries no counts to correct - t05/t11), and a
  header whose retally lands on identical bytes reports header_line: null (no
  change happened - nothing to announce).
- The doctrine passages are themselves observability rules: §9 makes acceptance
  evidence measurable by anyone via `git show <commit>:<path>` (the committed
  object, not a view that can vanish), and §11a names the failure signature to
  watch for (reads that look right per-view while the committed truth is wrong).
  t12 keeps both passages present in source and payload on every suite run.
- No logging added, deliberately: a progress line would break the byte-identity
  contract and tempt callers to parse chatter instead of the envelope. The
  standing detectors are the suite (t01-t12 under scripts/tests/run_all.sh's
  glob) and t08's live run of the INSTALLED copy.

branch_coverage_estimate 100 refers to announced-outcome branches of the changed
code: retallyHeader's four exits (bare header -> null, empty tally -> null,
changed header -> envelope + message, identical header -> null fields) are each
exercised by t05/t10/t11/t07 respectively; there is no catch-and-continue and no
path that alters the file without reporting it.
