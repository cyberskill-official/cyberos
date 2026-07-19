---
id: TASK-IMP-125
title: mechanical means deterministic, not docs-tools/
template: task@1
type: improvement
module: improvement
status: draft
priority: p3
author: "@stephencheng"
department: engineering
created_at: 2026-07-18T12:00:00+07:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: [TASK-IMP-115]
blocks: []
related_tasks: [TASK-IMP-084, TASK-IMP-118, TASK-IMP-124]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.0.0"
owner: Stephen Cheng (CTO)
created: 2026-07-18
memory_chain_hash: null
effort_hours: 3
service: modules/cuo
new_files:
  - (none)
modified_files:
  - docs/tasks/improvement/TASK-IMP-115-effort-tiering-advisory/spec.md
  - modules/cuo/chief-technology-officer/workflows/ship-tasks.md
  - modules/cuo/tests/test_workflow_evolution.py
source_pages:
  - "TASK-IMP-115 spec (ready_to_test, commit 8e7ffdca). §1.2 (spec.md:91) defines `mechanical` as 'performed by a deterministic helper'; AC 2 (spec.md:99) says 'delegates to a docs-tools helper'; the Success Metric (spec.md:63) says 'done by a docs-tools helper' with the word 'precisely'. Determinism and directory are two different predicates; they diverge on exactly two steps."
  - "modules/cuo/chief-technology-officer/workflows/ship-tasks.md:65-66 — step 28 `awh-gate` and step 29 `caf-gate` both carry `judgment: medium`. §11e:437 states '`mechanical` = a docs-tools helper produces the step's result'; §11e:465-467 flags 28-29 as 'the known rough edge' — deterministic but non-docs-tools, so tagged `medium` under AC 2's path scope."
  - "DERIVED by this author. `grep -c -i llm modules/skill/caf-gate/SKILL.md` -> **0**: caf-gate's SKILL.md does NOT contain 'no LLM'. It says 'deterministic' (SKILL.md:7 'runs the deterministic', SKILL.md:66 'The deterministic floor:'). The 'no LLM' phrase lives in ship-tasks.md:66 — the workflow's OWN step-29 description ('Deterministic floor, no LLM: `bash scripts/caf_gate.sh`'). §11e:466 re-attributes it to 'caf-gate's skill'. The behaviour claim is true; the source attribution is not."
  - "DERIVED by this author. awh determinism proven structurally: `tools/awh/harness/stage1_measurement/runner.py:250,254` — `run_task` scores each task on `_run(task.cmd)` then `_run(task.check) == 0`, i.e. subprocess exit codes. No model call in the scoring path. awh-gate's SKILL.md asserts neither 'deterministic' nor 'no LLM' in words; the executor's behaviour is the evidence."
  - "DERIVED by this author — the docs-tools/ guarantee check. `sed -n '137,141p' tools/install/build.sh` vendors a deterministic executor from OUTSIDE docs-tools/: `scripts/caf_gate.sh` + `tools/caf` -> `$out/cuo/gates/caf/`. `ls dist/cyberos/cuo/gates/caf/caf_gate.sh` -> EXISTS. `grep -c awh tools/install/build.sh` -> **0** and `find dist/cyberos -iname '*awh*'` -> empty: awh ships from nowhere. So docs-tools/ is NOT the payload's only deterministic executor and carries no vendoring guarantee the other paths lack."
  - "modules/cuo/tests/test_workflow_evolution.py:268 `_DOCS_TOOLS = _REPO_ROOT / 'tools/install/docs-tools'` and :354-364 (`_mechanical_helper_table` regex matches only ``docs-tools/([a-z0-9._-]+\\.mjs)`` rows): the AC-2 arm hard-codes the docs-tools/ path as the sole allowed executor root. This is the fixture that must generalise."
source_decisions:
  - "2026-07-18 IMP-115 review gate — operator approved filing this correction task (recorded HITL approval, review gate)."
---

# TASK-IMP-125: `mechanical` means deterministic, not `docs-tools/`

## Summary

TASK-IMP-115 gave every `ship-tasks` skill_chain step a `judgment: high | medium | mechanical` advisory field. Its own spec defines `mechanical` two incompatible ways: §1.2 keys it on BEHAVIOUR ("a deterministic helper … no model judgment in the result"), while AC 2 and the Success Metric key it on a PATH ("a docs-tools helper", "precisely"). The two readings diverge on exactly two steps — 28 (`awh-gate`) and 29 (`caf-gate`) — which are deterministic but whose executors live in `tools/awh/` and `scripts/caf_gate.sh`, not `docs-tools/`. This task picks the behaviour reading, makes §1.2, AC 2, and the Success Metric agree on it, re-tags steps 28-29 `mechanical`, and generalises the suite arm so a mechanical executor may live anywhere on disk.

## Problem

The field's stated purpose is to tell a host which steps a model is NOT deciding, so the host can decline to spend judgment-grade reasoning on them (IMP-115 §1.2 "no model judgment in the result"; ship-tasks §11e:437 "no model is deciding anything"). That is a fact about behaviour — is a model in the loop? — not about a directory.

But IMP-115 shipped the narrow reading. Steps 28 and 29 both carry `judgment: medium` (ship-tasks.md:65-66), and §11e:465-467 records why: both gates are deterministic, "but their executors (`tools/awh`, `scripts/caf_gate.sh`) are not docs-tools helpers, and TASK-IMP-115's AC 2 scopes `mechanical` to docs-tools backing. They read `medium`: under-informative rather than wrong." The implementer violated no clause — §1.2 is a one-way definition (it says what `mechanical` IS, not what it is NOT), so labelling a deterministic-executor step `medium` breaks nothing, and the AC-2 test (`test_mechanical_steps_are_helper_backed`) only checks that mechanical steps ARE docs-tools-backed, never the reverse.

The cost is precise and small: a host reading the advisory field is told to spend judgment-grade reasoning on two gates that provably need none. Both are deterministic — caf-gate's SKILL.md says "The deterministic floor:" (SKILL.md:66) and awh scores on subprocess exit codes (`runner.py:250,254`, `_run(task.cmd)` / `_run(task.check) == 0`, no model call). A definition that mislabels its own two most mechanical steps is a definition not yet doing its job.

Note on provenance: §11e:466 claims "caf-gate's skill says 'no LLM'". It does not — `grep -c -i llm modules/skill/caf-gate/SKILL.md` returns 0. The "no LLM" phrase is in the workflow's own step-29 description (ship-tasks.md:66), and the SKILL.md says "deterministic" instead. The behaviour is real; the attribution is mislocated. This correction cites the true sources and does not repeat the mislocated one.

## Proposed Solution

Decide the definition on behaviour and make the three IMP-115 sentences agree:

- **§1.2 stays behaviour-keyed** and is sharpened to say "wherever its executor lives".
- **AC 2 is rewritten** from "delegates to a docs-tools helper" to "delegates to a deterministic executor named in the payload for that skill".
- **The Success Metric is rewritten** the same way, and its "precisely" is reconciled to the behaviour predicate (mechanical ⟺ the step's result is produced by a deterministic executor with no model deciding).
- **Steps 28 and 29 become `judgment: mechanical`**, each naming its executor (`tools/awh` for awh-gate, `scripts/caf_gate.sh` for caf-gate) in §11e's table, which grows two rows; the "known rough edge" paragraph (§11e:465-467) is dropped because it no longer describes a rough edge.
- **The AC-2 suite arm generalises**: the executor may live outside `docs-tools/`, while the three lie-vectors the original arm closed (skill in the table, executor exists on disk, executor anchored in the skill's own prose) are all kept.

Is "which directory the executor lives in" load-bearing? No — it is an accident of where things got put, and the payload itself proves it. `build.sh:137-141` vendors a deterministic executor from OUTSIDE `docs-tools/` (`scripts/caf_gate.sh` + `tools/caf` -> `cuo/gates/caf/`), and `dist/cyberos/cuo/gates/caf/caf_gate.sh` exists on disk. So `docs-tools/` is not the only deterministic executor family the payload ships, and build.sh gives it no guarantee the other paths lack — it `cp`s docs-tools files exactly as it `cp`s caf. A definition keyed on the directory would call caf non-mechanical purely for its address while the payload ships it as a first-class deterministic gate: the definition rotting on contact with the payload it describes. Keying on behaviour cannot rot when a file moves.

## Alternatives Considered

- **Keep the narrow (docs-tools/) reading and just make §1.2 match it.** Rejected: it would formally bless mislabelling the two most mechanical steps in the chain, and it ties `mechanical` to a filesystem address that the payload's own vendor step (caf, from `scripts/`) already contradicts. The field would be wrong the first time an executor moved.
- **Widen the "helper family" to an explicit allow-list of three directories (`docs-tools/`, `tools/awh/`, `scripts/`).** Rejected: an allow-list of paths is the same path-keyed rule with more entries — it rots identically on the next move and needs editing every time a deterministic tool lands somewhere new. Behaviour ("a model is not deciding, and a named executor on disk does the work") is the durable predicate.
- **Leave IMP-115 as shipped and file only a doc note.** Rejected: the contradiction is inside a normative spec (§1.2 vs AC 2 vs Metric). A note alongside it does not make the three agree, and the suite still enforces the narrow reading via a hard-coded path.

## Success Metrics

- Primary: steps 28 and 29 carry `judgment: mechanical`, and IMP-115 §1.2, AC 2, and the Success Metric state one behaviour-keyed definition with no surviving `docs-tools/` narrowing — suite-asserted plus recorded grep. Baseline: 28-29 are `medium`; the three sentences disagree.
- Guardrail: no model string, price, or effort level is introduced, and nothing in the payload reads the `judgment` field to decide anything — the advisory guarantee IMP-115 established survives unchanged.

## Scope

In scope: IMP-115 §1.2 / AC 2 / Success Metric wording; steps 28-29's `judgment` value; §11e's mechanical-definition line, helper table, and rough-edge paragraph; the `test_mechanical_steps_are_helper_backed` arm and two supporting arms.

### Out of scope / Non-Goals

- Any change to what awh-gate or caf-gate DOES, when they run, or how step 30's done-flip reads their `GREEN`/`CLEAN` outcomes — this task touches the advisory label only.
- Re-tiering any step other than 28 and 29.
- Extending `judgment` to `create-tasks` or `plan` — still out of scope, exactly as IMP-115 left it.
- Vendoring `tools/awh` into the payload — awh's ship status is a separate question this task does not open.

## Dependencies

depends_on TASK-IMP-115: this task corrects three artefacts IMP-115 produced (its spec, its `ship-tasks` annotation, its suite arm). IMP-115 must be landed for there to be a definition to reconcile, so 125 comes after 115, never before. `service: modules/cuo` serialises 125 behind the in-flight `modules/cuo` cone per the ship-tasks parent-serialisation rule; it must not race it.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from the IMP-115 spec and `ship-tasks` §11e, with every claim (the contradiction, the two bite-points, the two gates' determinism, the docs-tools vendoring check, the mislocated "no LLM" attribution) re-verified against source at commit 8e7ffdca by this author.
- **Human review:** filing approved at the 2026-07-18 IMP-115 review gate (recorded HITL verdict).

## 1. Description (normative)

- 1.1 The definition of `mechanical` MUST be keyed on behaviour — a deterministic executor produces the step's result with no model deciding anything — and MUST NOT be keyed on the directory the executor lives in.
- 1.2 IMP-115 §1.2, its AC 2, and its Success Metric MUST all state that one behaviour-keyed definition; no residual wording MUST narrow `mechanical` to a `docs-tools/` path.
- 1.3 Steps 28 (`awh-gate`) and 29 (`caf-gate`) MUST be annotated `judgment: mechanical`.
- 1.4 Every step marked `mechanical` MUST name, in the payload's own prose for that step's skill, the deterministic executor that produces its result, and that executor MUST exist on disk irrespective of its directory.
- 1.5 The AC-2 suite arm MUST pass a `mechanical` step whose executor lives outside `docs-tools/` and MUST still fail a `mechanical` label that names no executor, names one absent from disk, or names one not anchored in the payload's prose for that skill.
- 1.6 This task MUST NOT introduce any model string, price, or effort-level literal into the payload.
- 1.7 The `judgment` field MUST remain advisory: nothing in the payload MUST read it to decide anything.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - IMP-115 §1.2's reconciled text asserts determinism ("deterministic executor", "no model deciding") and contains no directory token (`docs-tools`, `scripts/`, `tools/`) - verify: recorded grep in the gate log (a prose-wording claim no suite can grade; same rationale as TASK-IMP-090 AC 1).
- [ ] AC 2 (traces_to: #1.2) - IMP-115 §1.2, AC 2, and the Success Metric each carry the behaviour predicate and none carries "docs-tools helper" - verify: recorded three-way grep in the gate log (a cross-sentence consistency claim; same rationale as TASK-IMP-090 AC 1).
- [ ] AC 3 (traces_to: #1.3) - steps 28 and 29 both carry `judgment: mechanical` in ship-tasks' skill_chain - test: `modules/cuo/tests/test_workflow_evolution.py::test_gates_28_29_are_mechanical`
- [ ] AC 4 (traces_to: #1.4) - every mechanical step names an on-disk executor anchored in that skill's own payload prose - test: `modules/cuo/tests/test_workflow_evolution.py::test_mechanical_steps_are_helper_backed`
- [ ] AC 5 (traces_to: #1.5) - the arm accepts `tools/awh` and `scripts/caf_gate.sh` as mechanical executors yet still rejects an unnamed, absent, or unanchored one - test: `modules/cuo/tests/test_workflow_evolution.py::test_mechanical_executor_may_live_outside_docs_tools`
- [ ] AC 6 (traces_to: #1.6) - no model string, price, or effort literal appears anywhere in the payload after the change - test: `modules/cuo/tests/test_workflow_evolution.py::test_no_host_specific_literals`
- [ ] AC 7 (traces_to: #1.7) - nothing in the payload reads the `judgment` field to gate the run - verify: recorded grep in the gate log (a negative structural claim; same rationale as TASK-IMP-115 AC 4 / TASK-IMP-090 AC 1).

## 3. Edge cases

- A third deterministic gate added later whose executor is neither `.mjs` nor under `docs-tools/`: it is `mechanical` the moment it names an on-disk executor in its own prose — that is precisely the rot the behaviour reading prevents, and AC 5's arm admits it without an edit.
- A step that names an executor which does not exist on disk (a typo or an invented tool): still fails, exactly as it did before — AC 5 keeps the exists-on-disk lie-vector closed.
- `tools/awh` is not vendored into the payload (`find dist/cyberos -iname '*awh*'` is empty): the annotation describes the WORK a host routes on, not what the payload ships, so the label is correct even where the executor is absent. Vendoring awh is a separate task (Non-Goals).
- Step 23 (`coverage-gate-author`, `coverage-scope.mjs`) stays `medium`: its helper owns only the deterministic half and reserves `tests_failed` / `ecm_rows_uncovered` for the author skill (coverage-scope.mjs header), so a model still decides — the behaviour reading leaves it `medium`, unchanged.
- Security-class: edits one enum value on two table rows, three sentences of prose, and one test fixture. No execution surface, no new input path.
