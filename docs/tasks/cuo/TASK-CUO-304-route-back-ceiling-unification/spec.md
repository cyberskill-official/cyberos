---
id: TASK-CUO-304
title: Unify the route-back ceiling - api.py default 2 vs doctrine's 3
template: task@1
type: improvement
module: cuo
status: ready_to_implement
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-23T00:00:00+07:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-108, TASK-IMP-140]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.1.0"
owner: Stephen Cheng (CTO)
created: 2026-07-23
memory_chain_hash: null
effort_hours: 3
service: modules/cuo
new_files:
  - modules/cuo/tests/test_doctrine_constants.py
modified_files:
  - modules/cuo/cuo/api.py
  - modules/cuo/cuo/cli.py
  - CHANGELOG.md
source_pages:
  - "modules/cuo/cuo/api.py:127 (run() signature: halt_on_repeat_rework: int = 2), :289-291 (halt when rbc >= halt_on_repeat_rework - so the default halts a task at its SECOND route-back)"
  - "modules/cuo/cuo/cli.py:582-587 (--halt-on-repeat-rework, type=int, default=2, help text says 'default 2. Set 0 to disable.')"
  - "modules/cuo/chief-technology-officer/workflows/ship-tasks.md §11b (v2.8.0, TASK-IMP-108): 'At routed_back_count >= 3, ship-tasks MUST HALT at an operator gate'; 'Under the ceiling, nothing changes. A task at routed_back_count: 2 re-enters normally.'; 'Three is a judgment, not a derivation'"
  - "measured 2026-07-23: the two Python surfaces (api.py default, cli.py default+help) are the only machine encodings of the ceiling; the 5-fail debugging circuit breaker exists only as workflow/skill prose, not as a Python constant"
source_decisions:
  - "2026-07-23 operator: CyberOS Hardening Plan approved; Phase 2 T3 'Loop-bound unification' authored as an improvement task (plan file cyberos_hardening_plan_49404998; audit finding H3)."
  - "2026-07-23 authoring: the pin test derives the doctrine number by PARSING ship-tasks.md §11b rather than hardcoding 3 twice, so a future deliberate ceiling change fails one place (the doc) and the test names both sides of the mismatch. See Why in Proposed Solution."
---

# TASK-CUO-304: Unify the route-back ceiling - api.py default 2 vs doctrine's 3

## Summary

Doctrine (ship-tasks.md §11b) says a task halts for an operator verdict at `routed_back_count >= 3` and explicitly says a task at 2 re-enters normally. The machine encodes 2: `modules/cuo/cuo/api.py` defaults `halt_on_repeat_rework: int = 2` and halts when `rbc >= 2`, and `cli.py`'s flag defaults to 2 with matching help text. A drain run with defaults therefore halts one cycle earlier than the workflow contract promises. This task changes both defaults to 3 and adds a test that pins the Python defaults to the number parsed out of the doctrine text, so the constant can never fork silently again.

## Problem

Audit finding H3, verified first-hand: `api.py:127` (`= 2`), `api.py:289` (`rbc >= halt_on_repeat_rework`), `cli.py:584` (`default=2`, help "default 2"). Against ship-tasks.md §11b: halt at `>= 3`; a task at 2 re-enters normally; the number three is a recorded judgment ("the same task failed three DIFFERENT ways is evidence about the spec"). Two consequences:

1. **Behavioral drift:** a zero-touch drain (`cyberos-cuo ... run`) halts tasks at their second route-back - stricter than doctrine, so tasks that the workflow contract says should get one more attempt instead park for HITL early, and the halt brief cites a threshold the doctrine does not back.
2. **No pin:** nothing fails when the two surfaces disagree. The fork happened once already (this finding); without a cross-check it will happen again the next time either side is edited alone.

## Proposed Solution

Change the default to 3 in both places (`api.py` `run()` signature, `cli.py` option default + help text). Add `modules/cuo/tests/test_doctrine_constants.py` which (a) reads the ceiling from ship-tasks.md §11b by regex (`routed_back_count >= N` in the MUST-HALT bullet), (b) asserts `inspect.signature(cuo.api.run).parameters["halt_on_repeat_rework"].default` equals that N, (c) asserts the click option's default equals that N, and (d) asserts the help text quotes the same N. Parsing the doc rather than hardcoding 3 in the test means a future deliberate change edits exactly one normative home (the doctrine) and the test then names every stale machine surface. Add a CHANGELOG entry noting the default change (drains now allow the third cycle before halting).

## Alternatives Considered

- **Hardcode 3 in the test.** Rejected: that creates a THIRD copy of the constant; when the doctrine changes deliberately, the test fails with a message that reads like the test is wrong, and someone "fixes" the test while the api keeps the old value. Parsing the doctrine makes the doc the single source and the test a conformance check.
- **Move the constant into a shared config (e.g. `.cyberos/config.yaml` key).** Rejected: §11b explicitly frames three as a recorded judgment, not tunable configuration; making it operator-tunable invites the "set it to 99" drift the ceiling exists to prevent. The CLI flag already provides per-run override for genuine exceptions, defaulted to doctrine.
- **Treat api.py's 2 as intentional extra strictness and update the doctrine to 2 instead.** Rejected: §11b's reasoning ("failed three DIFFERENT ways is evidence about the spec") is the recorded judgment with rationale; the code carries no rationale for 2 and predates §11b's v2.8.0 wording. The doc is the deliberate artifact; the code is the drift.
- **Also pin the 5-fail debugging circuit breaker in the same test.** Deferred, not rejected: measured 2026-07-23, the breaker exists only as workflow/skill prose with no Python constant to pin. The test file is named `test_doctrine_constants.py` (plural) precisely so the breaker pin lands there the day the breaker gets a machine encoding. Recorded in Open questions-style deferral under Scope.

## Success Metrics

- Primary: by the next CyberOS release, a default drain halts a task only at its third route-back (`rbc >= 3`), and `test_doctrine_constants.py` fails within one CI run if either Python default ever disagrees with ship-tasks.md §11b again. Baseline today: defaults are 2 and no cross-check exists.
- Guardrail: `--halt-on-repeat-rework 0` (disable) and explicit non-default values keep today's exact semantics; the full `modules/cuo` pytest suite stays green.

## Scope

In scope: the two default-value changes, the help-text correction, the doctrine-parsing pin test, a CHANGELOG entry.

### Out of scope / Non-Goals

- Changing the halt semantics themselves (`rbc >= ceiling`, halt brief, HALT file) - only the default number moves.
- Pinning the 5-fail debugging circuit breaker - no machine constant exists to pin (measured; see Alternatives Considered). The test file is the designated future home.
- The mechanical HITL lock on the two acceptance gates - TASK-CUO-303.
- Benchmark gate G11 ("Loop-bound single-sourcing") CI wiring - TASK-IMP-140 runs this task's test as its G11 checker; soft forward reference, no cycle.

## Dependencies

None blocking. TASK-IMP-108 (done) authored ship-tasks.md §11b, the doctrine side of this unification. TASK-IMP-140's G11 gate adopts `test_doctrine_constants.py` as its checker - forward reference only.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS `task-author` skill in Cursor, as the task-authoring wave of the 2026-07-23 hardening plan.
- **Scope:** every `source_pages` line was read at HEAD in this checkout; both defaults, the comparison operator, the help text, and the absence of any other machine encoding of the ceiling were verified by direct read + grep.
- **Human review:** the hardening plan (including this task's scope bullet, "api.py 2 -> 3 + doctrine-constant test") was operator-approved on 2026-07-23.

## 1. Description (normative)

- 1.1 `modules/cuo/cuo/api.py` `run()` MUST default `halt_on_repeat_rework` to 3, and the halt comparison MUST remain `rbc >= halt_on_repeat_rework` so the default halts a task at its third route-back, matching ship-tasks.md §11b ("a task at routed_back_count: 2 re-enters normally").
- 1.2 `modules/cuo/cuo/cli.py`'s `--halt-on-repeat-rework` option MUST default to 3 and its help text MUST state the same number and keep documenting `0` as the disable value. The flag remains a per-run override; only its default is doctrine-bound.
- 1.3 A new test module `modules/cuo/tests/test_doctrine_constants.py` MUST parse the ceiling number from ship-tasks.md §11b's MUST-HALT bullet and assert all three machine surfaces equal it: the `run()` signature default, the click option default, and the number quoted in the option help text. The test MUST fail with a message naming both the doctrine value and the offending surface's value when they diverge.
- 1.4 The doctrine-parsing step MUST fail loudly (not skip) if the §11b pattern cannot be found - a silently-skipping conformance test is the schema-drift-test failure mode this batch is also fixing (TASK-MEMORY-303), and it must not be reproduced here.
- 1.5 `CHANGELOG.md` MUST gain an entry noting the default change and its behavioral effect (default drains now permit the third cycle before halting).

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - `inspect.signature(cuo.api.run).parameters["halt_on_repeat_rework"].default == 3` and a drain fixture with a task at rbc 2 re-enters while rbc 3 halts - test: `modules/cuo/tests/test_doctrine_constants.py::test_api_default_matches_doctrine`
- [ ] AC 2 (traces_to: #1.2) - the click option's default is 3 and its help text contains "default 3" and "Set 0 to disable" - test: `modules/cuo/tests/test_doctrine_constants.py::test_cli_default_and_help_match_doctrine`
- [ ] AC 3 (traces_to: #1.3) - the test derives N from ship-tasks.md (not a literal): patching the parsed doc text to `>= 4` in-memory makes all three assertions fail with messages naming 4 vs 3 - test: `modules/cuo/tests/test_doctrine_constants.py::test_mismatch_names_both_sides`
- [ ] AC 4 (traces_to: #1.4) - deleting/renaming the §11b bullet in an in-memory copy makes the parser raise (test fails), never skip - test: `modules/cuo/tests/test_doctrine_constants.py::test_missing_doctrine_pattern_fails_loud`
- [ ] AC 5 (traces_to: #1.5) - CHANGELOG's top entry mentions `halt-on-repeat-rework` and the 2-to-3 change - test: `modules/cuo/tests/test_doctrine_constants.py::test_changelog_entry_present`

## 3. Edge cases

- **Explicit `--halt-on-repeat-rework 2` after this change:** still halts at the second route-back - the flag override is untouched; only the default moved. The halt brief already prints the effective value, so logs stay unambiguous.
- **`0` (disable):** unchanged - the `if halt_on_repeat_rework and ...` guard in `api.py:289` already treats 0 as falsy/disabled; the pin test must not accidentally forbid 0 as an explicit value.
- **A task already at rbc 2 when the new default ships:** re-enters normally (doctrine-conformant where yesterday it would have halted); its next route-back (3) halts. No stored state needs migration - `routed_back_count` lives in frontmatter and is only compared at drain time.
- **Doctrine wording drift:** if §11b is reworded so the regex misses, the test fails loud per 1.4 - the failure message instructs the editor to update the parser and the machine surfaces together.
- **Monorepo vs installed repo:** the test lives in `modules/cuo/tests` (platform repo only); consumer installs receive the corrected defaults through the vendored payload but not the pin test - acceptable, since the constants fork in source, not per-install.
- **Security-class:** a pure constant + test change; no new inputs, no new execution surface.
