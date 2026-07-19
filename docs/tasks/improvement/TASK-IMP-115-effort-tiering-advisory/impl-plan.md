---
artefact: implementation-plan@1
task_id: TASK-IMP-115
workflow: chief-technology-officer/ship-tasks
step: 9-10
total_estimate_pts: 3
files_modified:
  - modules/cuo/chief-technology-officer/workflows/ship-tasks.md
  - modules/cuo/tests/test_workflow_evolution.py
files_new: []
---

# Implementation plan — TASK-IMP-115

## Slice 1 — annotate all 32 `skill_chain` steps (§1.1)

Add `judgment: <enum>` to each of the 32 flow mappings, positioned after `outputs_to` and
before `condition`/`phase`/`description` (context-map §1: identity → dataflow → modifiers).
Bare enum scalar. No other key moves; no step is added, removed, or renumbered.

## Slice 2 — the assignment (§1.2, §1.5)

The rule, stated so a reviewer can check it (AC 5):

- **`mechanical`** — a docs-tools helper PRODUCES the step's result; the agent runs the
  tool and transcribes. Per the spec's Success Metric the mechanical set is *precisely*
  the docs-tools-backed set, so the label is never applied on a "feels deterministic"
  basis.
- **`high`** — the model creates something the workflow then depends on. Each carries a
  named reason in the doc (AC 5's reviewer walk needs something to walk).
- **`medium`** — everything else, including every genuinely ambiguous step (§1.5:
  ambiguous is `medium`, never a guessed `high`).

| Level | Steps | Count |
|---|---|---|
| `mechanical` | 0, 13, 15, 19, 21, 30 | 6 |
| `high` | 1, 3, 5, 7, 9, 17, 25, 27 | 8 |
| `medium` | 2, 4, 6, 8, 10, 11, 12, 14, 16, 18, 20, 22, 23, 24, 26, 28, 29, 31 | 18 |

Every `mechanical` step is evidence-anchored (context-map §4): step 0 by
`task-reconcile/SKILL.md:19`'s `tool:` key; steps 13/15/19/21/30 by `ship-tasks.md:117`'s
executor sentence. No other chain skill has a docs-tools helper that produces its result.

**Deliberate non-mechanical calls, with the evidence:**

- **23 `coverage-gate-author` → `medium`, not `mechanical`.** `coverage-scope.mjs` exists
  but no skill delegates to it (`grep -rn 'coverage-scope' modules/` → nothing), and its
  own header reserves the judgment fields for the author skill. The spec's *Summary* cites
  it as near-mechanical; the normative §1.2 does not permit the label.
- **27 `task-audit` → `high`.** It names `task-lint.mjs`, but as a "machine floor" that
  seeds findings while "model diligence is spent on the judgment families only".
- **28 `awh-gate` / 29 `caf-gate` → `medium`.** Both are deterministic (caf's SKILL.md says
  "no LLM"), but their helpers are `tools/awh` and `scripts/caf_gate.sh` — **not** docs-tools
  helpers, so AC 2 and the Success Metric's "precisely … docs-tools helper" forbid the
  `mechanical` label. Surfaced for the operator: §1.2 says "a deterministic helper" while
  AC 2 says "a docs-tools helper", and these two steps are the only place the two readings
  disagree. Implemented to the narrower one (the contract), reversible in one edit.

## Slice 3 — document it as ADVISORY (§1.3)

New section `## 11e. Judgment tiering: what each step's work is (v2.8.0, TASK-IMP-115)`,
following §11a-§11d's shape (heading carries the version + task id; opens with the defect;
bolded lead-ins). `workflow_version` stays **2.8.0** (context-map §3 — a bump breaks four
assertions in `tools/install/tests/**`, IMP-106's cone, and pushes every in-flight manifest
to `needs_human`).

Carries: the enum; the advisory rule ("a host MAY route on it; nothing in the payload reads
it"); the mechanical table (step → skill → helper); the high table (step → named reason);
the ambiguous-is-medium rule; the no-model-strings rule; where it does not extend yet.

## Slice 4 — suite arms (§1.1, §1.2, §1.4)

In `modules/cuo/tests/test_workflow_evolution.py`, following the file's existing
`_SHIP_TASKS` / `_ship_tasks_text()` convention:

1. `test_every_step_has_judgment` (AC 1) — parse the frontmatter with PyYAML; assert every
   chain step has `judgment` in the closed enum. Vacuity guard: parsed count must equal an
   independent count of `- { step:` rows and be ≥ 30. **Fails if any step is unannotated.**
2. `test_mechanical_steps_are_helper_backed` (AC 2) — every `mechanical` step's skill must
   appear in §11e's mechanical table; the named helper must EXIST at
   `tools/install/docs-tools/<name>`; and the helper must be named in the payload's own
   record of that skill (the skill's `SKILL.md`, or ship-tasks' executor prose) so the
   claim is anchored rather than asserted. Vacuity guard: the mechanical set is non-empty.
3. `test_no_host_specific_literals` (AC 3) — scan the chain block AND §11e for model
   families, currency literals, and host effort-setting names. Scope is what this task
   writes, per §1.4's "as a result of this task": the file already carries `$500 in compute`
   (line 72, a capacity escalation predating this task) and "Claude plugin" (lines 541/570,
   a distribution channel), so a file-wide ban would fail on prose this task did not write.
   Verified: `grep -nEi '\b(claude|gpt|sonnet|opus|haiku|fable|gemini|llama)\b|\$[0-9]'` on
   ship-tasks.md returns exactly lines 72, 541, 570 — none inside the chain block.
4. `test_judgment_is_advisory_not_read` (§1.3 support) — the doc states the advisory rule
   AND no `skill_chain` consumer reads the key.

## Audit (step 10) — plan vs. the matrix

Every edge-case row maps to a slice: E1-E8 → arm 1; E9/E16/E17 → arm 2; E13 → arm 3;
E14 → arm 4 + AC 4's grep; E10/E11 → slice 3's version decision; E15/E18 → slice 3's doc
+ AC 5's walk. Estimate 3 pts, matching the spec's `effort_hours: 3`. Existing patterns
respected (context-map §6). No capacity escalation (well under 25 %).
