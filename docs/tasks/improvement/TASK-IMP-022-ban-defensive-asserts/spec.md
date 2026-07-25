---
id: TASK-IMP-022
title: "Ban defensive asserts — an assertion that cannot fail is not evidence"
template: task@1
type: improvement
module: improvement
author: "@stephencheng"
department: engineering
status: done
priority: p1
created_at: 2026-07-08T00:00:00+07:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-118, TASK-IMP-124]
routed_back_count: 0
awh: N/A
verify: T
phase: "Wave 2 - measure and evaluate"
owner: Stephen Cheng (CTO)
created: 2026-07-08
effort_hours: 6
refs: [R13]
service: scripts
new_files:
  - scripts/check_defensive_asserts.sh
  - scripts/tests/test_assert_lint.sh
modified_files:
  - modules/skill/task-audit/RUBRIC.md
  - modules/cuo/tests/test_baseline.py
  - modules/cuo/tests/test_placeholder_check.py
  - modules/cuo/tests/test_type_discriminator.py
  - modules/memory/tests/core/test_crypto_mode.py
  - modules/memory/tests/core/test_import.py
  - modules/memory/tests/core/test_store_acl.py
  - CHANGELOG.md
source_pages:
  - "docs/strategy/cyberos-deep-audit-and-auto-evolution-plan-2026-07-06.md:50 (R13) — 'Ban defensive asserts. The memory-writer contract bug shipped because a test asserted `processed == 3 || failed > 0` (docs/KNOWN-ISSUES.md #1). Grep-audit for or-conditions inside asserts and add a review rule: a test must fail when the feature is broken.'"
  - "docs/strategy/cyberos-deep-audit-and-auto-evolution-plan-2026-07-06.md:125 — 'Fix the known eval-integrity hole first (R13); an eval suite with defensive asserts is a placebo.'"
  - "modules/skill/task-audit/RUBRIC.md §9 TRACE-006 / TRACE-007 — the two sibling rules this one completes"
---

# TASK-IMP-022 — ban defensive asserts

## Summary

Add the mechanical floor R13 asked for: a lint over the gated test corpus that fails on assertions no input can falsify — a disjunction satisfied by either arm, a statically-true expression, a count comparison that holds at zero, a probe whose exit status is discarded. Six such assertions exist on `main` today; all six are replaced with an assertion of the one behaviour the code actually has, so the lint lands at zero findings and zero waivers. R13's second ask — the review rule — lands as `TRACE-008` in `audit_rubric@2.0`, completing the TRACE-006 / TRACE-007 family.

## Problem

R13 (`docs/strategy/cyberos-deep-audit-and-auto-evolution-plan-2026-07-06.md:50`): the memory-writer contract bug shipped because a test asserted `processed == 3 || failed > 0`. That predicate is true whenever the writer FAILED. The suite that existed to catch the defect stayed green straight through it, and the same document's line 125 draws the conclusion — "an eval suite with defensive asserts is a placebo."

Nothing in the chain looks for this shape. The machine floor (`task-lint`) reads specs, not test bodies. TRACE-004 asks whether a cited test PASSES — a defensive assertion passes by construction, so TRACE-004 is satisfied *most* strongly by the worst tests. TRACE-006 (TASK-IMP-118) asks whether a test's assertion is weaker than its clause's verb, but it is judgment-family and unmechanizable by construction, so nothing runs it on 125 test files. The result on `main` is six live assertions that no code change can falsify:

| file:line | assertion | why it cannot fail |
|---|---|---|
| `modules/cuo/tests/test_baseline.py:149` | `any("review_overdue" in w for w in warnings) or any(... in issues)` | green whichever channel fires — the channel IS the contract |
| `modules/cuo/tests/test_placeholder_check.py:139` | `"yaml_parse" in error or error == "frontmatter_not_dict"` | green on a classification the fixture never produces |
| `modules/cuo/tests/test_type_discriminator.py:62` | `"rubrics/{type}.md" in skill or "rubrics/common" in skill` | green with either half of the composition deleted |
| `modules/memory/tests/core/test_crypto_mode.py:208` | `"no binlog segments" in details or "sth_only" in details` | green on both paths the test exists to tell apart |
| `modules/memory/tests/core/test_import.py:219` | `not binlog.exists() or binlog.stat().st_size == 0` | green on a dry run that created the file |
| `modules/memory/tests/core/test_store_acl.py:239` | `not check(...).allowed or check(...).mode == "read"` | green on a refusal that matched no ACL row at all |

Two of these are further evidence for *how* the check must be built. `test_store_acl.py:239` is split across lines with a backslash and does not match `assert .* or .*`; `test_sync_conflicts.py:97` contains the literal `"dropbox-or-gdrive"` and does. R13 asks for a grep-audit, and a grep-audit over this corpus is wrong in both directions.

## Proposed Solution

**`scripts/check_defensive_asserts.sh`** — bash wrapper over a `python3` scanner, same shape as the `check_doc_anchors.sh` precedent (exit 0 clean · exit 10 with `DEFENSIVE <file>:<line> [<rule>] <detail>` lines · `--list` census, always 0). Five rules: `DA-001` disjunctive Python assert, `DA-002` statically-true Python assert, `DA-003` shell probe whose status is discarded, `DA-004` shell numeric comparison that holds at zero, `DA-005` a waiver with no reason.

The Python side walks the AST and matches on `BoolOp(Or)`, so the string-literal false positive and the line-continued false negative are both handled by construction rather than by tuning a regex. The shell side is a line scanner making only claims decidable from one line, and it skips heredoc bodies — a heredoc body is data, not shell the file executes, which is also what lets this lint's own suite carry the negative fixtures that prove it fires.

**`scripts/tests/test_assert_lint.sh`** — ten checks, discovered by `run_all.sh`'s glob (so pre-commit and the `suite-gate` workflow both run it without a second registration). Every detector is proved from both sides: a fixture it must flag and a near-miss it must not. `t08` asserts the live corpus is clean; that is the gate. `t09` asserts the lint's shell roots are exactly `run_all.sh`'s globbed roots, so the audited set and the gated set cannot drift apart.

**`TRACE-008` in `modules/skill/task-audit/RUBRIC.md` §9** — R13's review rule, sited beside its siblings: TRACE-006 fails an assertion weaker than its clause's VERB, TRACE-007 fails a derivation narrower than its claim's SCOPE, TRACE-008 fails an assertion with no falsifying input at all. It is the only one of the three with a mechanical floor, because "true for every input" is decidable for a closed set of shapes while "weaker than this verb" is not.

**Six replacements.** Each of the six is replaced by asserting the one behaviour the code has, established by running it and reading the result — not by widening the assertion until it holds, and not by waiving it.

## Alternatives Considered

- **A regex grep, as R13 literally specifies.** Rejected on measurement, not on taste: over this corpus a regex produces two false positives (`"dropbox-or-gdrive"`, and an `or` inside an assert message) and one false negative (the line-continued disjunction in `test_store_acl.py`). A lint whose first run is 33% noise is a lint that gets `# noqa`'d. The AST costs no more code and is right in both directions.
- **Waive the six and ship the lint clean-by-exemption.** Rejected: the exemptions would be the deliverable. Every one of the six had a determinate answer that took a single probe run to establish, which is exactly what R13's review rule asks an author to do.
- **A sibling `defensive-assert-exemptions.txt`, matching `check_doc_anchors.sh`.** Rejected: a path-granular exemption suits "this document is a historical archive"; an assert waiver is line-granular, and a reviewer reading the assertion must be able to see why the disjunction is admissible without opening a second file. Waivers are inline and must carry a reason of at least 12 characters — a reasonless waiver is `DA-005`, a finding in its own right.
- **Infer shell polarity (`if <A||B>; then ok` is weak, `… then fail` is a stronger conjunctive assertion).** Rejected for this pass: it needs the then-branch, and the four live `if [ ! -f A ] || [ ! -f B ]; then fail` sites in the corpus are the CORRECT form. Guessing wrong would flag good tests, and a lint that flags good tests is uninstalled. Named as out-of-scope rather than silently omitted; TRACE-008's judgment half covers it.
- **Extend `task-lint.mjs`.** Rejected: `task-lint` audits spec documents. Pointing it at test bodies would give one tool two corpora and two failure vocabularies.
- **Do nothing; rely on TRACE-006.** Rejected: TRACE-006 is judgment-family and fires at the spec gate on the clauses of one task. Six defensive assertions survived it on `main`, which is the measurement.

## Success Metrics

- Primary: `bash scripts/check_defensive_asserts.sh` exits 0 over 125 gated test files with **0 findings and 0 waivers**, and exits 10 on each of the five detector fixtures. A lint that cannot fail is the defect this task is about, so `t01`–`t07` exist to prove it can.
- Guardrail: `bash scripts/tests/run_all.sh` shows no suite red that was green on `main` at `bb161013`, and the `modules/cuo` + `modules/memory` pytest suites hold at their pre-change pass counts (276 and 521). Replacing a disjunction with a stronger assertion must not be paid for with a broken suite.

## Scope

In scope: `scripts/check_defensive_asserts.sh`; `scripts/tests/test_assert_lint.sh`; the `TRACE-008` row and subsection in `modules/skill/task-audit/RUBRIC.md` §9; the six replacement assertions; the CHANGELOG entry.

### Out of scope / Non-Goals

- **`services/**`.** Rust `assert!` / `assert_eq!`, Go, and platform Python are outside CyberOS 1.x per `docs/batches/batch-10e-imp-stub-wont-do.md`. The bound is stated in the lint header and in TRACE-008 rather than left to be discovered: a green floor there means "not looked at".
- **Assertions inside an embedded interpreter heredoc** (`python3 - <<'PY'` within a `.sh` suite). Skipped by construction, stated as a bound.
- **Shell polarity inference** — see Alternatives.
- **Vendoring the lint into the payload** (`.cyberos/docs-tools/`). This gates the cyberos repo's own corpus first; shipping it to consumer repos means holding arbitrary corpora to these five shapes, which is its own decision with its own blast radius.
- **A corpus sweep of the 180+ `done` tasks against TRACE-008.** Same call TASK-IMP-118 made for TRACE-006: sizing that belongs to the operator, not to this task.

## Dependencies

None blocking. `run_all.sh`'s glob (TASK-IMP-107/128) is the registration path for the new suite; `audit_rubric@2.0` §9 already carries TRACE-006 and TRACE-007, and TRACE-008 is one more row plus one more subsection.

## AI Authorship Disclosure

- **Tools used:** Cursor agent (Opus 5), authoring `TASK-IMP-022` from the R13 source line after it had sat as an unauthored draft stub since the 2026-07-08 migration.
- **Scope:** R13's framing was treated as a claim to verify, not as an input.
  **re-derived and CONFIRMED:** the R13 defect shape is live in this repo — `bash scripts/check_defensive_asserts.sh` on `bb161013` returns six `DA-001` findings, each listed in ## Problem with its file and line, and each probed by running the code under test to establish the single true behaviour.
  **re-derived and CORRECTED:** R13 specifies a *grep*-audit for or-conditions. Measured against this corpus a grep is wrong in both directions — it flags `assert sources == {"dropbox-or-gdrive", "syncthing"}` (`test_sync_conflicts.py:97`) and misses the backslash-continued disjunction at `test_store_acl.py:239`. The implementation parses instead, and `t02`/`t03` pin both directions on disk.
  **measured and ADDED:** the shell half finds **zero** live findings across all 56 shell suites; the 11 `|| true` sites in them are output captures and cleanup, none an assertion. `DA-003`/`DA-004` therefore ship as forward guards with no current bite, which is said here rather than implied by a green run. Also added: heredoc-body skipping, discovered when the lint's own fixture file flagged itself.
- **Human review:** @stephencheng recorded the session HITL verdict for `batch/12g-imp-022`, including confirmation of `ai_authorship` and `eu_ai_act_risk_class` (the two `# UNREVIEWED` markers the 2026-07-14 migration left on this stub). Recorded at `docs/batches/batch-12g-imp-022-session-hitl.md`.

## 1. Clauses

- 1.1 `scripts/check_defensive_asserts.sh` MUST scan the gated test corpus and exit 10 with one `DEFENSIVE <file>:<line> [<rule>] <detail>` line per finding, 0 when clean, and MUST support `--list` for a findings-plus-waivers census that always exits 0. Test: `t01_da001_flags_disjunction`
- 1.2 `DA-001` MUST flag any Python `assert` whose test expression contains a disjunction, including one split across physical lines, and MUST NOT flag an `assert` whose only `or` is inside a string literal. The detector MUST parse the file; a textual match MUST NOT be the implementation. Tests: `t02_da001_ignores_or_inside_a_string`, `t03_da001_catches_multiline_disjunction`
- 1.3 `DA-002` MUST flag a Python `assert` that is statically true — truthy constant, non-empty literal container, lambda or f-string, `len(...) >= 0`, `len(...) > -1` — and MUST NOT flag `assert False`, which is an unreachable marker rather than a defensive assertion. Test: `t04_da002_vacuous_only`
- 1.4 `DA-003` MUST flag a bare shell probe (`grep`, `rg`, `test`, `[`, `[[`, `diff`, `cmp`) whose exit status is discarded by `|| true` or `|| :`, and MUST NOT flag an output capture such as `n="$(grep -c x f)" || true`. Heredoc bodies MUST NOT be scanned: they are data, not shell the file executes. Test: `t05_da003_swallowed_probe`
- 1.5 `DA-004` MUST flag a shell numeric comparison that holds at zero (`-ge 0`, `-gt -1`) — the AUTH-005 #6 shape, a test that passes when nothing happened. Test: `t06_da004_vacuous_numeric`
- 1.6 A waiver (`# defensive-assert-ok: <reason>` on the flagged line or the line above) MUST suppress a finding only when the reason is at least 12 characters; a shorter or absent reason MUST itself be reported as `DA-005`. Every active waiver MUST be printed on every run. Test: `t07_waiver_requires_a_reason`
- 1.7 The live corpus MUST be clean — zero findings and zero waivers — and each of the six pre-existing `DA-001` sites MUST be replaced by an assertion of the single behaviour the code produces, established by running it. Widening an assertion or waiving it MUST NOT be used in place of a replacement. Test: `t08_live_corpus_clean`, plus the `modules/cuo` and `modules/memory` pytest suites
- 1.8 The lint's shell roots MUST be exactly the roots `scripts/tests/run_all.sh` globs. A suite the runner reaches but the lint does not is gated and unaudited; the reverse is a dead scan. Test: `t09_roots_match_runner`
- 1.9 `modules/skill/task-audit/RUBRIC.md` §9 MUST carry `TRACE-008` as a table row and as a subsection naming: what makes an assertion unfalsifiable, the mechanical floor that enforces the decidable part, its kinship to TRACE-006, and R13 as its origin. Test: `t10_rubric_carries_trace_008`
- 1.10 The suite MUST prove each detector from both sides — a fixture it flags and a near-miss it does not. A lint against defensive assertions asserted defensively would be self-refuting. Tests: `t01`–`t07`

## 3. Edge case matrix

| # | Category | Trigger | Expected | Test |
|---|---|---|---|---|
| 1 | NULL/EMPTY | a test file with no `assert` at all | scanned, no finding, counted in the file total | t02 |
| 2 | NULL/EMPTY | a corpus root that does not exist on disk | `WARN` on stderr, scan continues, exit unaffected | t01 |
| 3 | BOUNDS | `assert False` | not flagged — an unreachable marker, not a defensive assertion | t04 |
| 4 | BOUNDS | waiver reason of exactly 12 characters | suppresses; 11 characters is `DA-005` | t07 |
| 5 | BOUNDS | disjunction nested inside a comprehension (`any(x or y for …)`) | flagged — the `BoolOp` is reachable in the assert's test | t01 |
| 6 | MALFORMED | a Python test file that does not parse | reported as `DA-000` at the syntax-error line, never skipped silently | t02 |
| 7 | MALFORMED | `or` inside an assert's message string, not its test | not flagged — the AST separates test from message | t02 |
| 8 | MALFORMED | disjunction continued with `\` across two lines | flagged, reported at the `assert`'s own line | t03 |
| 9 | SECURITY | a waiver used to silence a genuine defect | bounded, not prevented: the reason is mandatory, printed on every run, and `t08` pins the corpus at zero waivers, so the first one has to be argued in review | t07, t08 |
| 10 | SECURITY | a `\|\| true` inside a heredoc fixture | not flagged — heredoc bodies are data; the bound is stated so it cannot be mistaken for coverage | t05 |
| 11 | CONCURRENT | two suites adding test files between a scan and a commit | no shared state; the lint re-walks the tree per run and pre-commit runs it on the staged tree | t08 |
| 12 | DEGRADATION | `run_all.sh` gains a fourth globbed root | `t09` goes red until the lint's root list is extended — the drift is caught, not inherited | t09 |
| 13 | DEGRADATION | Rust/Go assertions under `services/**` | out of scope by declaration; a green run does not claim them | — (stated bound) |

## 4. Out of scope / non-goals

See "## Scope → ### Out of scope / Non-Goals" above.

## Acceptance criteria

- AC1 (traces_to 1.1): the lint exists, exits 10 with a per-finding `DEFENSIVE` line on the R13 shape, and exits 0 on a clean tree. Test: `t01_da001_flags_disjunction`.
- AC2 (traces_to 1.2): a string containing `or` is not flagged. Test: `t02_da001_ignores_or_inside_a_string`.
- AC3 (traces_to 1.2): a line-continued disjunction is flagged. Test: `t03_da001_catches_multiline_disjunction`.
- AC4 (traces_to 1.3): three statically-true shapes are flagged; `assert False` is not. Test: `t04_da002_vacuous_only`.
- AC5 (traces_to 1.4): a bare swallowed probe is flagged; an output capture is not. Test: `t05_da003_swallowed_probe`.
- AC6 (traces_to 1.5): `-ge 0` is flagged. Test: `t06_da004_vacuous_numeric`.
- AC7 (traces_to 1.6): a reasoned waiver suppresses and prints; a bare one becomes `DA-005`. Test: `t07_waiver_requires_a_reason`.
- AC8 (traces_to 1.7): the live corpus scans clean at zero findings and zero waivers, and both pytest suites hold at their pre-change pass counts. Test: `t08_live_corpus_clean` + `modules/cuo` (276 passed, 2 skipped) + `modules/memory` (521 passed, 5 skipped, 1 pre-existing red on `bb161013`). Pre-change and post-change runs side by side: `testing-evidence.md` §3.
- AC9 (traces_to 1.8): the lint's shell roots equal `run_all.sh`'s globbed roots. Test: `t09_roots_match_runner`.
- AC10 (traces_to 1.9, 1.10): `TRACE-008` is in the rubric with its substance named, and every detector is proved from both sides. Test: `t10_rubric_carries_trace_008` + `t01`–`t07`.
