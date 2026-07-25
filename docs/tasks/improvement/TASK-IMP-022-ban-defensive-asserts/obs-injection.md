# TASK-IMP-022 observability injection

This task's deliverable IS an observability instrument: it makes visible a class of failure that was
previously indistinguishable from success — a green suite. There is no consumer-runtime component, so the
honest record is what the instrument reports, who reads it, and where it is blind, not invented spans for a
scanner that runs in 0.3 s.

- **Signal design.** One line per finding, machine-readable and stable:
  `DEFENSIVE <file>:<line> [<rule>] <detail>`. `file:line` so an editor jumps to it; the rule id so the
  class is countable across runs; the detail restates the offending expression (`ast.unparse` of the
  `BoolOp`) so the reader does not have to open the file to see the shape. Exit codes are distinct by
  cause — 0 clean, 10 findings, 2 unusable — matching the `check_doc_anchors.sh` vocabulary already in the
  repo rather than inventing a second one.

- **Waivers are a signal, not a silence.** Every active waiver prints to stderr on **every** run, including
  clean ones, and the clean line carries the count (`… 125 gated test files scanned, N waived`). A
  suppression mechanism that goes quiet is how a lint stops gating; this one gets louder as it is used.

- **Who reads it.** Three consumers, none of which needs a human to remember to run it:
  `scripts/tests/test_assert_lint.sh` t08 (the gate), `.githooks/pre-commit` via `run_all.sh` (local), and
  `.github/workflows/suite-gate.yml` on every push and PR (CI, on ubuntu — so a macOS-only laptop cannot be
  the only thing that ever ran it).

- **Failure visibility.** The instrument's own failure modes are asserted, not assumed. An unparsable Python
  file surfaces as `DA-000` rather than being skipped. A missing corpus root warns on stderr rather than
  silently shrinking the scan. And `t09` fails the day `run_all.sh` gains a root the lint does not scan —
  the "gated but unaudited" state is the one failure a green run could otherwise hide.

- **Where it is blind, stated so a green cannot be over-read.** `services/**` is not scanned (outside
  CyberOS 1.x per `docs/batches/batch-10e-imp-stub-wont-do.md`); assertions inside an embedded interpreter
  heredoc are not scanned; shell polarity is not inferred. All three are named in the lint header, in
  TRACE-008, and in the spec's Scope. R13's own line 125 — "an eval suite with defensive asserts is a
  placebo" — applies equally to a lint that lets a reader believe it covered more than it did.

**Branch coverage of the change's executable surface:** every detector (DA-001..005) plus both waiver arms
plus the heredoc skip is exercised by t01–t07; the `DA-000` parse-error arm and the missing-root warn arm
are reviewed by inspection and named in the edge-case matrix (rows 2, 8) rather than counted as asserted.
