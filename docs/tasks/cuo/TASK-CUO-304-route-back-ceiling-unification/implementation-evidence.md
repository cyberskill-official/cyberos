# TASK-CUO-304 — implementation evidence

Implementer: batch/8-audit-hardening swarm worker (Cursor agent), 2026-07-23.

## What changed and why

| File | Change | Spec clause |
|---|---|---|
| `modules/cuo/cuo/api.py` | `run()` default `halt_on_repeat_rework: int = 2` → `3` (comment points at §11b + the pin test). The `rbc >= halt_on_repeat_rework` comparison at the halt site is untouched. | 1.1 |
| `modules/cuo/cuo/cli.py` | `--halt-on-repeat-rework` `default=2` → `3`; help now says "(default 3). Set 0 to disable." | 1.2 |
| `modules/cuo/tests/test_doctrine_constants.py` | NEW. Parses the ceiling out of ship-tasks.md §11b's MUST-HALT bullet (regex `routed_back_count\s*>=\s*(\d+)` on MUST-HALT lines; exactly one hit required, else `DoctrineParseError` — loud, never a skip). Pins: `inspect.signature` default, click option default, help text ("default N" + "Set 0 to disable"), plus the BEHAVIORAL boundary (a stubbed drain with the signature default: rbc N−1 re-enters/exit 0/no halt brief; rbc N halts/exit 2/DRAIN_HALT.md written/halt line prints the effective N). Mismatch messages name both sides; AC 3 proves derivation by patching the doc text in-memory to `>= 4`; AC 4 proves raise-not-skip for a deleted bullet AND for a `>=`→`>` comparator rewording. | 1.3, 1.4 |

## Test output (verbatim)

Command (the module's sanctioned invocation, per `modules/cuo/audit-profile.yaml`
`RUN_COMMANDS` and `.awh/goldenset.yaml`): `cd modules/cuo && python3 -m pytest -q tests/test_doctrine_constants.py`

```
....F                                                                    [100%]
FAILED tests/test_doctrine_constants.py::test_changelog_entry_present - Asser...
1 failed, 4 passed in 0.12s
```

- `test_api_default_matches_doctrine` PASS (AC 1 — includes rbc 2 re-enters / rbc 3 halts behaviorally)
- `test_cli_default_and_help_match_doctrine` PASS (AC 2)
- `test_mismatch_names_both_sides` PASS (AC 3 — messages name 4 vs 3)
- `test_missing_doctrine_pattern_fails_loud` PASS (AC 4 — raise, never skip)
- `test_changelog_entry_present` FAIL **by design, temporarily**: root `CHANGELOG.md` is owned by the final sequential pass in this shared-tree batch; the entry text is ready (see open item 1). The failure message itself instructs the fixer. Goes green the moment the entry lands.

Full-suite guardrail: `cd modules/cuo && python3 -m pytest -q` →
`5 failed, 269 passed, 2 skipped in 6.31s`. Triage of the 5:

- `test_doctrine_constants.py::test_changelog_entry_present` — this task, red-until-CHANGELOG (above).
- `test_workflow_evolution.py::test_ceiling_is_a_judgment_not_a_derivation`, `::test_spec_rejected_pairs_with_the_ceiling`, `::test_judgment_is_advisory_not_read` — PRE-EXISTING at HEAD: they pin hard-wrapped prose (e.g. `"evidence about the\n  spec"`) that the phase-1 polish commit's reflow of ship-tasks.md no longer contains. Proven: working-tree ship-tasks.md is byte-identical to HEAD, and `git show HEAD:...ship-tasks.md | grep -c "evidence about the$"` = 0. Not this task's scope.
- `test_ship_manifest.py::TestShipManifest::test_gitignore_scaffold` — PRE-EXISTING at HEAD: expects `docs/tasks/.workflow/.gitignore` to hold exactly 2 patterns; the committed file holds 3 (`skill-trust.tsv` added). Not this task's scope.

## Decisions / deviations

1. The behavioral half of AC 1 stubs `execute_chain` (monkeypatched to return `ROUTED_BACK`) and `routed_back_count` (returns the fixture rbc) so the REAL `api.run()` loop + halt branch executes with its REAL signature default — no live personas invoked, no live store read, halt brief written under pytest tmp_path.
2. AC 4 additionally pins the `>=` comparator at the parser level (a §11b rewording to `>` fails loud) — closes the audit's ISS-003 concern from the parser side too.
3. No changes to `--halt-on-repeat-rework 0` (disable) semantics or explicit-value overrides; the pin test asserts defaults and help only (audit ISS-004 guardrail).

## Open items for the HITL reviewer / final pass

1. **Root `CHANGELOG.md` (spec 1.5, AC 5)**: add the entry — exact suggested text lives in TASK-CUO-303's `implementation-evidence.md` open item 3 (one shared top entry covers both tasks: mentions `halt-on-repeat-rework` and "2 → 3", which is exactly what `test_changelog_entry_present` asserts).
2. **Third machine encoding discovered (spec's measurement missed it)**: `modules/memory/cyberos/__main__.py` lines 2448/2484/2499 hardcode `halt_on_repeat_rework = 2` with help "(default 2)" in the `cyberos workflow` wrapper. It is OUTSIDE this task's normative clauses (which name only api.py + cli.py) and outside this worker's ownership partition (`modules/memory` belongs to the memory-hardening sibling). Until fixed, `python -m cyberos workflow run` still defaults to 2 while `cyberos-cuo drain` defaults to 3. Recommend: hand to the memory sibling in this batch or author a follow-up micro-task; the pin test is the designated home for the added assertion once that surface is doctrine-bound.
3. The 4 pre-existing red tests in `modules/cuo` (triaged above) predate this task and belong to the phase-1-polish fallout, not this batch's T3.
