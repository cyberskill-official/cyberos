# TASK-IMP-022 code review

Reviewer: ship-tasks agent, `batch/12g-imp-022`, cut from `main@bb161013`.
Diff: `scripts/check_defensive_asserts.sh` (new), `scripts/tests/test_assert_lint.sh` (new),
`modules/skill/task-audit/RUBRIC.md` (§9 row + subsection), six test files (one assertion each),
`CHANGELOG.md`, the task's artefact bundle.

## Clause → proof

| Clause | Requirement | Proof |
|---|---|---|
| 1.1 | scanner exists; exit 10 + one `DEFENSIVE` line per finding; `--list` census always 0 | `check_defensive_asserts.sh`; `t01` asserts exit 10 AND `test_x.py:2 [DA-001]` in one pattern, so a right exit with a wrong location cannot pass it |
| 1.2 | `DA-001` flags disjunctions incl. multi-line; never a string containing `or`; parsed not grepped | `t02` (scratch with `{"dropbox-or-gdrive"}` + an `or` in an assert message → exit 0) and `t03` (backslash continuation → exit 10). Live pre-fix proof: the grep found 5 of the 6, the AST found 6 |
| 1.3 | `DA-002` flags statically-true; spares `assert False` | `t04` counts exactly 3 `DA-002` hits on the vacuous fixture AND requires exit 0 on a separate `assert False` file — one scratch could not prove both |
| 1.4 | `DA-003` flags a swallowed bare probe, spares an output capture; heredoc bodies unscanned | `t05` two scratches. Heredoc skipping is proved by the suite's own existence: it contains `grep -q needle "$f" \|\| true` and `t08` is green |
| 1.5 | `DA-004` flags `-ge 0` | `t06` |
| 1.6 | waiver needs a ≥12-char reason; else `DA-005`; waivers always print | `t07` two scratches — reasoned waiver must exit 0 AND print `waived`; `meh` must exit 10 with `DA-005` |
| 1.7 | corpus clean at zero findings AND zero waivers; six replaced, not widened | `t08`; `--list` tail reads `files=125 findings=0 waivers=0`; `modules/cuo` 276 passed / `modules/memory` 521 passed |
| 1.8 | lint roots == `run_all.sh` roots | `t09` parses the runner's `for t in …` glob line and checks each root appears in the lint |
| 1.9 | `TRACE-008` in the rubric with substance | `t10` requires the table row AND four defining strings inside the `### TRACE-008` subsection — presence of the token alone fails it |
| 1.10 | every detector proved from both sides | t01–t07; four of the seven use two scratch repos for exactly this |

## Judgment

- **Correctness vs the ticket.** R13 asked for a grep-audit plus a review rule. Both landed; the grep
  became an AST because the corpus falsified the grep. The six live findings are R13's own shape — an
  assertion true whenever the feature is broken — and each is now an assertion of the single behaviour
  the code has, established by running it rather than by reasoning about it.

- **The temptation this diff had to refuse.** Three cheap ways to a green: widen the detector so the six
  stop matching, waive the six, or ship a baseline file pinned at six. All three make the sentence "the
  corpus is clean" true and the lint worthless — the same move R13 describes. None was taken; the audit
  records the rejection (ISS-004) rather than leaving it implicit in a clean run.

- **Self-reference, handled on semantics not on exemption.** A lint against defensive assertions must
  carry fixtures containing defensive assertions, and its first clean run flagged its own suite. The fix
  is that a heredoc body is data, not shell the file executes — so `DA-003`/`DA-004`, which are claims
  about executed shell, do not apply to it. A hardcoded self-exemption would have worked and would have
  been the wrong shape: the next reader would rightly ask what else hides in the list.

- **Blast radius.** The scanner is read-only and executes nothing it reads: no `eval`, no `exec`, no
  `subprocess`; `ast.parse` builds a tree without evaluating. Nothing is vendored into the payload, so a
  consumer install is byte-identical. Runtime ~0.3 s over 125 files.

- **Failure mode if wrong.** A false positive blocks a commit with a named file:line and a documented
  waiver — recoverable in one line. A false negative is invisible, which is why t02/t05's near-miss arms
  are paired with t01/t03/t06's positive arms: the detector is pinned from the side that fails loudly and
  from the side that fails silently.

- **Where a green means less than it looks.** `services/**` is unscanned; assertions inside an embedded
  `python3 - <<'PY'` block are unscanned; shell polarity is not inferred. Stated in the lint header, in
  TRACE-008, and in §Scope. R13's line 125 — "an eval suite with defensive asserts is a placebo" —
  applies to a lint that lets a reader over-read its coverage, so the bound is written three times.

- **Naming.** The spec names `t01`–`t10` and the suite lands under exactly those names; no remap. Rule
  ids `DA-001..005` are new and collide with nothing (`FM-`, `SEC-`, `COND-`, `QA-`, `SAFE-`, `TRACE-`,
  `STALE-`, `BUG-`, `REGRESSION-` are the existing families).

## Findings

None open. One accepted info, carried in the audit as ISS-007: the waiver mechanism ships with zero
users. Bounded rather than removed — a lint with no escape hatch is deleted wholesale at the first
legitimate disjunction.

HALT: review acceptance (`reviewing -> ready_to_test`) is a human gate.
