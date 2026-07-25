# Testing evidence — TASK-IMP-022 (ban defensive asserts)

Every number AC8 and §5 rely on, with the command that produces it. Numbers in a spec are
originated claims (TRACE-007); prose provenance does not discharge them, so each one below is
reproducible from a clean checkout of this branch.

Base commit: `bb161013` (`origin/main` at branch point). Host: macOS, `python3` 3.13.

---

## 1. The lint's own suite (t01–t10 → AC1–AC10)

```
$ bash scripts/tests/test_assert_lint.sh
test_assert_lint.sh (TASK-IMP-022)
  ok   t01 … ok   t10
  pass=10 fail=0
```

t01–t07 each build a throwaway `git init` repo shaped like the corpus roots, drop one fixture in,
and assert the lint's **exit code, rule id and file:line**. Seven of the ten are there to prove the
lint can go red; a lint that cannot fail is the defect this task exists to remove, so the suite is
written so that deleting any single detector turns an arm red rather than leaving it green.

The two arms that carry the design are the near-misses:

| arm | fixture | must |
| --- | ------- | ---- |
| `t02` | `assert sources == {"dropbox-or-gdrive", "syncthing"}` | **not** flag — the false positive a grep cannot avoid |
| `t03` | `assert not check(...).allowed \` + newline + `or check(...).mode == "read"` | **flag** — the false negative a line-oriented grep cannot avoid |

Both fixtures are copied from live files (`modules/memory/tests/core/test_sync_conflicts.py:97`,
`test_store_acl.py:239`). They are the on-disk form of the audit's ISS-002 derivation: the reason
the Python side parses instead of grepping is not an opinion, it is these two arms.

## 2. Live corpus census (AC8)

```
$ bash scripts/check_defensive_asserts.sh --list | tail -1
files=125 findings=0 waivers=0
```

**Zero findings and zero waivers.** The distinction matters: a waived corpus and a clean corpus
both exit 0, and only one of them is the claim §1.7 makes. `t08` pins both halves, so the first
waiver anyone adds has to be argued in review rather than merged as "still green".

`files=125` = 124 gated test files on `bb161013` + `test_assert_lint.sh`, added by this branch.

## 3. Pre-change vs post-change, side by side

The guardrail for §1.7: rewriting six assertions to be *stronger* must not be paid for with a
broken suite. Run on the same host, same interpreter, `git stash -u` for the clean column.

| suite | clean `bb161013` | this branch | delta |
| ----- | ---------------- | ----------- | ----- |
| `cd modules/cuo && python3 -m pytest tests/ -q` | 276 passed, 2 skipped | 276 passed, 2 skipped | none |
| `cd modules/memory && python3 -m pytest tests/ -q` | **1 failed**, 521 passed, 5 skipped | **1 failed**, 521 passed, 5 skipped | none |

The single red is `tests/test_schema_single_source.py::test_all_copies_identical_and_acl_bearing`.
It is red on `bb161013` **before this branch existed** — that is what the left column is for. This
branch neither causes it nor fixes it, and recording it here rather than quoting only the green
number is the same discipline the task is about: a guardrail that is credited with a failure it did
not cause is as misleading as an assertion that cannot fail.

Counts are identical because none of the six edits adds or removes a test function; each replaces
one assertion inside an existing one.

## 4. The six replacements (AC8, §1.7)

Each was resolved by running the code under test and reading the single value it produces, then
asserting that value — never by widening the assertion until it held.

| file | was (passes even when broken) | now |
| ---- | ----------------------------- | --- |
| `modules/cuo/tests/test_baseline.py` | `any("review_overdue" in w for w in warnings) or any(... in issues)` | asserts the warning channel **and** that `issues` is empty |
| `modules/cuo/tests/test_placeholder_check.py` | `"yaml_parse" in error or error == "frontmatter_not_dict"` | `error.startswith("yaml_parse:")` |
| `modules/cuo/tests/test_type_discriminator.py` | `"rubrics/{type}.md" in s or "rubrics/common" in s` | both present — the rubric is composed of both |
| `modules/memory/tests/core/test_crypto_mode.py` | `"no binlog segments" in details or "sth_only" in details` | `details == "no binlog segments"` |
| `modules/memory/tests/core/test_import.py` | `not binlog.exists() or binlog.stat().st_size == 0` | `not binlog.exists()` — a dry run creates nothing |
| `modules/memory/tests/core/test_store_acl.py` | `not …allowed or …mode == "read"` | `allowed is False` **and** `mode == "read"` |

Row 6 is the sharpest: the old disjunction was satisfied by `allowed=False` alone, so an ACL
regression that granted `mode="write"` while still denying would have passed it.

## 5. Repo gates

```
$ bash .cyberos/cuo/gates/run-gates.sh
```

Transcript: `docs/batches/batch-12g-gates-transcript.txt`. `scripts/tests/run_all.sh` includes
`test_assert_lint.sh` automatically (it globs `scripts/tests/test_*.sh`), which is also what puts
the new lint on the pre-commit path and in `suite-gate` with no CI edit.

## 6. What is NOT covered

Stated rather than left to be inferred from a green run:

- `services/**` (Rust `assert!` / `assert_eq!`, Go, platform Python) is not scanned — outside the
  CyberOS 1.x payload per `docs/batches/batch-10e-imp-stub-wont-do.md`. A green floor there means
  "not looked at".
- Assertions inside an embedded interpreter heredoc (`python3 - <<'PY'`) are outside the floor, a
  direct consequence of the heredoc-skipping that lets this suite carry its own negative fixtures.
- Shell polarity inference is not attempted; the corpus has four correct `if A || B; then fail`
  sites and zero weak ones, so the rule's only live matches would have been false positives.

All three are covered by TRACE-008's judgment half in `modules/skill/task-audit/RUBRIC.md`.
