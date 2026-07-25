# TASK-IMP-022 implementation plan

Order matters here: the corpus is measured **before** the detector set is fixed, because R13 specifies an
implementation (a grep) and the corpus is the only thing that can say whether that implementation is right.

1. **Measure first** (informs 1.2, 1.4, 1.5) — run the candidate shapes over the corpus by hand. Findings:
   6 Python disjunctions; 0 shell findings under any candidate shell rule; 2 grep false positives and 1 grep
   false negative. This is what moves the Python side to an AST and demotes shell polarity inference to
   out-of-scope.
2. **The scanner** (clauses 1.1–1.6) — `scripts/check_defensive_asserts.sh`. Python detectors walk the AST
   (`DA-001` BoolOp(Or), `DA-002` statically-true); shell detectors are a line scanner making only
   single-line claims (`DA-003` swallowed probe, `DA-004` `-ge 0`), skipping heredoc bodies. Waivers
   (`# defensive-assert-ok:`) require a ≥ 12-char reason or become `DA-005`. Corpus roots: the three
   `run_all.sh` globs + every `tests/` tree under `modules/`.
3. **The suite** (clause 1.10) — `scripts/tests/test_assert_lint.sh`, t01–t07 in scratch repos built by
   `new_scratch` (`git init` so the lint's `git rev-parse --show-toplevel` cannot escape into the real repo
   and make every arm vacuous). Each detector proved from both sides.
4. **Fix the six** (clause 1.7) — probe each site by running the code under test and reading the single
   result, then assert exactly that. No widening, no waivers. Each carries a one-line note naming what the
   old disjunction let through, so the next reader sees why the assertion is narrow.
5. **The review rule** (clause 1.9) — `TRACE-008` row + subsection in `RUBRIC.md` §9, sited beside
   TRACE-006/007 with the kinship stated: verb / scope / falsifiability.
6. **Wire nothing** (clause 1.8) — the glob already reaches the suite; `t09` guards the one thing that
   could drift (the runner gaining a fourth root).
7. **Gates** — `bash scripts/tests/test_assert_lint.sh` (10/10), `bash scripts/tests/run_all.sh` against the
   `bb161013` baseline, both pytest suites, `task-lint`, `bash .cyberos/cuo/gates/run-gates.sh`.

**Discovered during step 3, folded back into step 2:** the suite's own DA-003/DA-004 fixtures made the live
corpus red. Fixed on semantics (heredoc bodies are data), not by waiver — see audit ISS-003.
