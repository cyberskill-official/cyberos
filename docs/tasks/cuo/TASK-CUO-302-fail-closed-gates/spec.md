---
id: TASK-CUO-302
title: Fail-closed machine gates - RED when zero gate commands are configured
template: task@1
type: improvement
module: cuo
status: ready_to_implement
priority: p0
author: "@stephencheng"
department: engineering
created_at: 2026-07-23T00:00:00+07:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-CUO-207, TASK-IMP-129, TASK-IMP-140]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.1.0"
owner: Stephen Cheng (CTO)
created: 2026-07-23
memory_chain_hash: null
effort_hours: 6
service: tools/install
new_files:
  - tools/install/tests/test_fail_closed_gates.sh
modified_files:
  - tools/install/gates/run-gates.sh
  - tools/install/install.sh
  - tools/install/README.md
  - CHANGELOG.md
source_pages:
  - "tools/install/gates/run-gates.sh:57 (gate(): 'SKIP %s (no command configured)' - an absent command is a skip, not a failure), :74-77 (all-empty floor prints 'GATES: floor only ...' then falls through to 'GATES: GREEN (machine gates only).' and exits 0)"
  - "measured 2026-07-23: this repo's own .cyberos/gates.env has BUILD_CMD/LINT_CMD/TEST_CMD/COVERAGE_CMD all empty with '# Auto-detected ecosystem: unknown', so the flagship repo's machine gate is vacuous-GREEN today"
  - "tools/install/install.sh:299 (generated gates.env header: '- gate commands for the task workflow (edit freely).'), :326 (regen notice: 'durable overrides belong in .cyberos/config.yaml') - the header and the notice contradict each other"
  - "tools/install/install.sh gates-autodetect block (BUILD_CMD/LINT_CMD/TEST_CMD/COVERAGE_CMD seeding via json_has_script + per-ecosystem probes; no fallback probes repo-local suite entrypoints like scripts/tests/run_all.sh, which is why ECOSYSTEM=unknown leaves all four empty here)"
  - "gates.env.bak evidence (audit 2026-07-23): a working TEST_CMD='bash scripts/tests/run_all.sh' survived only in gates.env.bak.1784761166 after a reinstall regenerated gates.env - the C1 wipe class"
  - ".cyberos/cuo/gates/run-gates.sh verified byte-identical to tools/install/gates/run-gates.sh on 2026-07-23 (diff clean), so source-side fixes propagate on payload rebuild"
source_decisions:
  - "2026-07-23 operator: CyberOS Hardening Plan approved; Phase 2 T1 'Fail-closed gates' authored as an improvement task (plan file cyberos_hardening_plan_49404998; audit finding C1)."
  - "2026-07-23 authoring: the escape hatch is an environment variable (CYBEROS_ALLOW_EMPTY_GATES=1) rather than a config.yaml key, deliberately - an ack that lives in config silently outlives the person who set it; an env var must be re-asserted per invocation or set visibly in CI config. See Alternatives Considered."
---

# TASK-CUO-302: Fail-closed machine gates - RED when zero gate commands are configured

## Summary

`run-gates.sh` is the machine-gate floor for every task lifecycle transition, and today it reports `GATES: GREEN` and exits 0 when not a single gate command is configured. This repo is itself the proof: autodetect returned `unknown` on the polyglot monorepo, all four floor commands are empty, and the flagship repo gates nothing while reporting green. This task makes the empty floor RED by default, adds an explicit acknowledged-empty escape hatch, teaches autodetect a monorepo fallback so this repo detects its own suite, and stops the `gates.env` header from inviting edits that the next reinstall wipes.

## Problem

Three verified defects compound into audit finding C1 (a core safety promise is false):

1. **Fail-open floor.** `gate()` treats an empty command as `SKIP` (`tools/install/gates/run-gates.sh:57`), and after the per-gate loop the all-empty case prints an advisory line (`GATES: floor only - nothing detected and no overrides`, line 75) and then falls through to `GATES: GREEN` with exit 0 (lines 77-79). Green-is-necessary doctrine (AGENT-ENTRY.md #3) assumes green *means something*; an all-skip green is vacuous, and both HITL acceptance gates downstream inherit that false confidence.
2. **Autodetect has no monorepo fallback.** `install.sh` probes package.json scripts and per-ecosystem markers; on this polyglot repo it lands `ECOSYSTEM=unknown` and seeds nothing. The repo has an obvious canonical suite entrypoint (`scripts/tests/run_all.sh`, 42 suites) that autodetect never looks for.
3. **The header invites edits the machine wipes.** The generated `gates.env` says `(edit freely)` (`install.sh:299`) while reinstall regenerates the file and moves the old one to a `.bak` (`install.sh:326` even says durable overrides belong in `config.yaml`). An operator followed the header's advice; the reinstall wiped a working `TEST_CMD` which now survives only in `gates.env.bak.1784761166`.

## Proposed Solution

Make the all-empty floor exit RED with a message that names the two real fixes (`.cyberos/config.yaml` `gates.*` keys, or re-running install after adding ecosystem markers) and the explicit escape hatch `CYBEROS_ALLOW_EMPTY_GATES=1` for repos that genuinely have nothing to run (docs-only repos). The escape hatch prints a loud `GATES: EMPTY-ACKNOWLEDGED` line so an acknowledged-empty run is never confusable with a green one. Teach the `install.sh` autodetect an ordered monorepo fallback probe - `scripts/tests/run_all.sh` first, then `Makefile` `test:` target - seeding `TEST_CMD` with provenance `SRC_TEST="fallback:<probe>"` so `run-gates.sh`'s existing provenance line shows where the command came from. Reword the generated `gates.env` header from "edit freely" to "machine-owned; regenerated on every install - durable overrides belong in .cyberos/config.yaml (gates.build/lint/test/coverage)". Ship a CHANGELOG entry marking the RED-on-empty behavior as breaking for consumer repos that relied on floor-only green.

## Alternatives Considered

- **Warn loudly but stay green on empty.** Rejected: that is exactly today's behavior (line 75 already warns); the audit demonstrated the warning changes nothing because green is what the workflow reads.
- **Put the acknowledged-empty flag in `.cyberos/config.yaml`.** Rejected: a config key is set once and outlives its justification silently; an env var must be re-asserted per invocation (or visibly exported in CI), which keeps the acknowledgment honest. The config file remains the home for *commands*, not for permission to run none.
- **Hard-require `.cyberos/config.yaml` gate keys everywhere.** Rejected: breaks zero-config installs on the many repos where autodetect works today (node/python/go/rust/etc. per TASK-CUO-207); the floor should fail only when there is genuinely nothing to run.
- **Autodetect fallback tries every shell file under `scripts/`.** Rejected: guessing arbitrary scripts as test commands executes untrusted-shaped code on install; the fallback probes a closed, documented list of canonical entrypoints only (`scripts/tests/run_all.sh`, `Makefile` with a `test:` target).

## Success Metrics

- Primary: by the next CyberOS release, a scratch install onto a repo with no detectable ecosystem followed by `bash .cyberos/cuo/gates/run-gates.sh` exits non-zero with the RED-empty message, and this repo's own gates run exits 0 only because the monorepo fallback seeded `TEST_CMD="bash scripts/tests/run_all.sh"`. Baseline today: both exit 0 with all-empty commands.
- Guardrail: zero regressions in the existing gate behavior for configured repos - a repo with at least one configured command keeps today's exact PASS/FAIL/exit semantics (existing suites `test_gates_config.sh` class stay green).

## Scope

In scope: `tools/install/gates/run-gates.sh` (RED-on-empty + acknowledged-empty ack line), `tools/install/install.sh` (fallback autodetect probes + `gates.env` header rewording), `tools/install/README.md` (document the new failure mode + escape hatch), CHANGELOG entry, and a new test suite covering all three behaviors.

### Out of scope / Non-Goals

- Preserving operator edits to `gates.env` across reinstall (the header fix makes the ownership honest; durable-override preservation across *uninstall* is TASK-IMP-129's scope).
- Any change to the CAF/AWH optional gates (`CAF_ENABLED`/`AWH_ENABLED` semantics unchanged; their empty-command handling is unchanged because they are opt-in, not the floor).
- The `cyberos doctor` gate wiring for memory-installed repos - that is TASK-MEMORY-303's scope.
- CI wiring of the gate benchmark (G1) checker beyond this task's own test suite - the benchmark-gates program is TASK-IMP-140.

## Dependencies

None blocking. Builds directly on TASK-CUO-207 (done), which shipped the `.cyberos/config.yaml` gates layer and the per-gate autodetect provenance (`SRC_*`) this task extends with a `fallback:` source tier. TASK-IMP-129 (draft) makes `config.yaml` survive uninstall - complementary, no ordering constraint. TASK-IMP-140's benchmark gate G1 ("Gate-floor non-vacuous") is *verified by* the test this task ships; listed in `related_tasks` as a soft forward reference, no cycle.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS `task-author` skill in Cursor, as the task-authoring wave of the 2026-07-23 hardening plan.
- **Scope:** every `source_pages` line was read at HEAD in this checkout during authoring; the fail-open exit path, the all-empty local `gates.env`, the header/notice contradiction, and the `.bak`-only surviving `TEST_CMD` were verified first-hand, not carried from the audit report.
- **Human review:** the hardening plan (including this task's scope bullet) was operator-approved on 2026-07-23; the escape-hatch-as-env-var design call is recorded in `source_decisions` for the reviewer to revisit at the review acceptance gate.

## 1. Description (normative)

- 1.1 `run-gates.sh` MUST exit with the distinct code 3 (RED, empty floor) when all four floor commands (build, lint, test, coverage) resolve to empty after the config.yaml and gates.env layers are applied, unless `CYBEROS_ALLOW_EMPTY_GATES` is set to the literal value `1`. Exit 3 is deliberately distinct from 1 (a configured gate failed) and 2 (missing/malformed config), so automation can tell "gates ran and failed" from "nothing was configured". Any value other than the literal `1` (including `true`, `yes`, `0`) MUST behave as unset. The existing per-gate `SKIP` behavior for a *subset* of empty gates is unchanged - the floor fails only when it is entirely vacuous.
- 1.2 The RED-on-empty message MUST name both durable fixes (set `gates.build/lint/test/coverage` in `.cyberos/config.yaml`, or re-run install so autodetect can seed commands) and the escape hatch by exact name. A failure an operator cannot act on from the message alone is a support ticket, not a gate.
- 1.3 When `CYBEROS_ALLOW_EMPTY_GATES=1` is set and the floor is empty, `run-gates.sh` MUST print a distinct `GATES: EMPTY-ACKNOWLEDGED` line (not `GATES: GREEN`) before exiting 0, so logs can never conflate an acknowledged-empty run with a green run.
- 1.4 `install.sh` autodetect MUST gain a monorepo fallback tier: when no ecosystem probe seeds a test command, probe an ordered, closed list - `scripts/tests/run_all.sh` (seed `TEST_CMD="bash scripts/tests/run_all.sh"`), then a `Makefile` containing a `test:` target (seed `TEST_CMD="make test"`) - recording provenance `SRC_TEST="fallback:run_all"` or `SRC_TEST="fallback:make"` respectively. The fallback MUST NOT execute the probed files at install time.
- 1.5 The generated `gates.env` header MUST NOT say "edit freely"; it MUST state that the file is machine-owned and regenerated on every install, and that durable overrides belong in `.cyberos/config.yaml` (`gates.*` keys). The wording change applies to the generator in `install.sh`; installed copies pick it up on next install.
- 1.6 `CHANGELOG.md` MUST gain an entry documenting RED-on-empty as a breaking behavior change for consumer repos, naming `CYBEROS_ALLOW_EMPTY_GATES=1` as the migration path for intentionally gate-less repos.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - on a scratch install with an all-empty floor, `run-gates.sh` exits exactly 3; with any one command configured it keeps today's semantics (0 on pass, 1 on gate failure); `CYBEROS_ALLOW_EMPTY_GATES=true` and `=0` still exit 3 - test: `tools/install/tests/test_fail_closed_gates.sh::t01_empty_floor_exits_red`
- [ ] AC 2 (traces_to: #1.2) - the RED output names `.cyberos/config.yaml`, re-install, and `CYBEROS_ALLOW_EMPTY_GATES` all three, asserted as substrings - test: `tools/install/tests/test_fail_closed_gates.sh::t02_red_message_actionable`
- [ ] AC 3 (traces_to: #1.3) - with `CYBEROS_ALLOW_EMPTY_GATES=1` and empty floor: exit 0, output contains `GATES: EMPTY-ACKNOWLEDGED`, and does NOT contain `GATES: GREEN` - test: `tools/install/tests/test_fail_closed_gates.sh::t03_ack_line_distinct`
- [ ] AC 4 (traces_to: #1.4) - installing onto a fixture repo that has `scripts/tests/run_all.sh` but no detectable ecosystem seeds `TEST_CMD="bash scripts/tests/run_all.sh"` with `SRC_TEST="fallback:run_all"`; the Makefile probe seeds `make test` on a Makefile-only fixture; install runs neither probe target - test: `tools/install/tests/test_fail_closed_gates.sh::t04_monorepo_fallback_seeds_test_cmd`
- [ ] AC 5 (traces_to: #1.5) - the generated `gates.env` on a scratch install contains no `edit freely` substring and does contain the machine-owned + config.yaml wording - test: `tools/install/tests/test_fail_closed_gates.sh::t05_header_machine_owned`
- [ ] AC 6 (traces_to: #1.6) - `CHANGELOG.md`'s top entry mentions the RED-on-empty change, the word "breaking", and `CYBEROS_ALLOW_EMPTY_GATES` - test: `tools/install/tests/test_fail_closed_gates.sh::t06_changelog_breaking_entry`

## 3. Edge cases

- **CAF/AWH enabled, floor empty:** an operator with `CAF_ENABLED=true` but an empty floor still fails 1.1 - the optional gates are additive, not a substitute for the floor. The RED message applies unchanged; acknowledging empty while running CAF-only is expressible via the env var and is the operator's explicit call.
- **Malformed `.cyberos/config.yaml`:** already exits 2 before any gate runs (`run-gates.sh` MALFORMED guard); this task's empty-floor check MUST evaluate only after the config layer parsed, so the two failure modes keep distinct exit paths and messages.
- **Docs-only consumer repo:** legitimately has nothing to run; the documented path is exporting `CYBEROS_ALLOW_EMPTY_GATES=1` (e.g. in the repo's CI env). The ack line keeps the honesty visible in every log.
- **`gates.env` predating this change:** an installed repo that upgrades its payload but has not re-run install still has the old header text; the RED-on-empty behavior comes from the vendored `run-gates.sh` (rebuilt payload) regardless, so enforcement does not wait on the header fix. Stale-header-only trees are acceptable; behavior is not.
- **Fallback probe on a repo with BOTH `run_all.sh` and a Makefile:** the ordered list wins - `run_all.sh` seeds and the Makefile is not consulted; a fixture asserts the precedence so the order is contractual, not incidental.
- **Security-class:** the fallback seeds a command *string* into `gates.env` but never executes it at install time (1.4); execution happens only inside `run-gates.sh` under the operator's existing `eval` trust model for gate commands. No new execution surface is introduced at install time.
