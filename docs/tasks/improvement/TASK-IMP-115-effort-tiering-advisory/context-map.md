---
artefact: repo-context-map@1
task_id: TASK-IMP-115
workflow: chief-technology-officer/ship-tasks
workflow_version: 2.8.0
step: 1-2
files_outside_immediate_domain: 0
adr_required: false
---

# Repo context map — TASK-IMP-115 (effort tiering, advisory judgment metadata)

Every claim below names the command that proves it. Run from the repo root.

## 1. How `skill_chain` already encodes per-step metadata (the shape to follow)

`modules/cuo/chief-technology-officer/workflows/ship-tasks.md` frontmatter, lines 30-67.
One YAML **flow mapping per step, one step per line**, inside a block sequence:

```
  - { step: 1,  skill: repo-context-map-author,   inputs_from: { repo_root: repo_root, ... },  outputs_to: context_map_draft,  phase: "ready_to_implement → implementing" }
```

Command: `sed -n '30,67p' modules/cuo/chief-technology-officer/workflows/ship-tasks.md`

Observed key set and ORDER (no step deviates):

| Key | Value style | Present on | Role |
|---|---|---|---|
| `step` | bare int | all 32 | identity |
| `skill` | bare slug `[a-z0-9-]+` | all 32 | identity |
| `inputs_from` | nested flow map, or bare identifier | all 32 | dataflow |
| `outputs_to` | bare identifier | all 32 | dataflow |
| `condition` | quoted string (`'…'` or `"…"`) | 0, 3, 4, 7, 8, 25, 26, 30 | modifier |
| `phase` | double-quoted string | 0, 1, 13, 15?, 21? (phase-opening steps only) | modifier |
| `description` | double-quoted long prose | 27, 28, 29 | modifier |

Ordering rule inferred from every line: **identity → dataflow → modifiers**, i.e.
`step, skill, inputs_from, outputs_to, [condition], [phase], [description]`.
Padding aligns `inputs_from` and `outputs_to` into columns; the modifier tail is ragged.

**Decision — where `judgment` goes.** `judgment` is per-step metadata about the step
itself, not dataflow and not a run-condition. It is placed **after `outputs_to`, before
`condition`/`phase`/`description`** — the head of the modifier tail, so it sits at a stable
position on all 32 lines instead of being buried behind steps 27-29's paragraph-length
`description`. Value style: **bare enum scalar**, exactly like `skill`. This follows the
existing shape: a new key in the same flow mapping, bare-scalar, no new nesting, no new
block.

## 2. Who parses `skill_chain` (blast radius of a new key)

| Consumer | How it reads the chain | Effect of an unknown key | Proof command |
|---|---|---|---|
| `modules/cuo/cuo/core/catalog.py:222` | `skill_chain=list(fm.get("skill_chain") or [])` — verbatim dicts | none (dicts pass through) | `grep -n 'skill_chain=' modules/cuo/cuo/core/catalog.py` |
| `modules/cuo/cuo/core/validator.py:88` | `step.get("skill")` | none | `grep -n 'step.get' modules/cuo/cuo/core/validator.py` |
| `modules/cuo/cuo/core/supervisor.py:130,429,839` | `.get("step") / .get("skill")` | none | `grep -n 'skill_chain' modules/cuo/cuo/core/supervisor.py` |
| `modules/cuo/cuo/core/brief_generator.py:296,324,442` | `.get(...)` per known key | none | `grep -n 'skill_chain' modules/cuo/cuo/core/brief_generator.py` |
| `modules/cuo/cuo/trigger_tests.py:223` | `skill_chain[0].get("skill")` | none | `grep -n 'skill_chain\[0\]' modules/cuo/cuo/trigger_tests.py` |
| `tools/install/check-chain-coverage.sh:64` | `grep -Eo 'skill: *[a-z0-9-]+'` | none — `judgment:` does not match | `sed -n '54,64p' tools/install/check-chain-coverage.sh` |
| `tools/install/tests/test_full_sdp_payload.sh:105` | `grep -q "skill_chain:"` | none | `sed -n '102,107p' tools/install/tests/test_full_sdp_payload.sh` |
| `tools/install/tests/test_batch_economics.sh:96` | fails if a `batch.economics` step is ADDED | none — no step added | `sed -n '96,97p' tools/install/tests/test_batch_economics.sh` |
| `tools/install/docs-tools/ship-manifest.mjs` | parses `skill_chain` for a step's skill name | none | `grep -n 'skill_chain' tools/install/docs-tools/ship-manifest.mjs` |

**No consumer validates the key set.** Nothing rejects, and nothing reads, an unknown
per-step key. This is what makes §1.3's "nothing in the payload reads it" achievable by
construction rather than by promise.

## 3. `workflow_version` — do NOT bump (hard finding)

`tools/install/tests/test_workflow_helpers.sh` asserts the CURRENT version literally, in
four places:

```
$ grep -n "^workflow_version: 2\\\\.8\\\\.0\\$" -r tools/install/tests/test_workflow_helpers.sh
484:    grep -q '^workflow_version: 2\.8\.0$' "$f" || { fail t09 ... }
602:      || { fail t12 "$f: workflow_version not bumped to 2.8.0"; return; }
626:    || { fail t13 "payload cuo/ship-tasks.md workflow_version not 2.8.0"; return; }
645:    grep -q '^workflow_version: 2\.8\.0$' "$f"     || { fail t14 "$f: version not 2.8.0"; return; }
```

Bumping to 2.9.0 would break t09/t12/t13/t14 — and the fix would live in
`tools/install/**`, which is **TASK-IMP-106's cone** (this batch's other member, already at
`ready_to_test`). It would also make every in-flight ship-manifest pinned at 2.8.0 —
IMP-106's and this task's own — resolve to `needs_human` on resume (Resume semantics rule
1: "workflow_version mismatch -> needs_human").

Precedent supports not bumping: §11b (IMP-108), §11c (IMP-109) and §11d (IMP-114) each
added a doctrine section to this file and all three are labelled `v2.8.0` — the whole
handoff group rides one version. This task's spec does not mention `workflow_version`.

**Decision: stay at 2.8.0.** The new section is labelled `(v2.8.0, TASK-IMP-115)`,
matching §11b/§11c/§11d.

## 4. Which steps' work a docs-tools helper actually performs (§1.2 / AC 2 input)

`ls tools/install/docs-tools/` → `backlog-mutate.mjs batch-select.mjs coverage-scope.mjs
memory-append.mjs ship-manifest.mjs task-lint.mjs task-reconcile.mjs verify-goals.mjs
workflow-improve.mjs`

Delegation, as the repo records it (not as inferred from a skill's name):

| Skill | Helper | Where the payload says so | Verdict |
|---|---|---|---|
| `task-reconcile` | `docs-tools/task-reconcile.mjs` | `modules/skill/task-reconcile/SKILL.md:19` — frontmatter `tool:` key | helper performs the report + its single recommendation → **mechanical** |
| `backlog-state-update-author` | `docs-tools/backlog-mutate.mjs` | `ship-tasks.md:117` — "the byte-discipline executor for `backlog-state-update` mutations … never hand-sed" | the mutation IS the helper's write; the transition is an input → **mechanical** |
| `task-audit` | `docs-tools/task-lint.mjs` | `modules/skill/task-audit/SKILL.md:248` | helper is a "machine floor" that seeds mechanical findings; "model diligence is spent on the judgment families only" → **NOT** mechanical (**high**) |
| `coverage-gate-author` | `coverage-scope.mjs` exists, but **nothing wires it to the skill** | `grep -rn coverage-scope modules/` → no hit outside this task's own spec | the tool's own header says "The judgment fields (tests_failed, ecm_rows_uncovered, raw_terminal) stay with the author skill … never guessed" → **NOT** mechanical (**medium**) |
| `backlog-state-update-audit` | none | `grep -niE 'mjs|docs-tools' modules/skill/backlog-state-update-audit/SKILL.md` → no hit | rubric-scored audit (`backlog_state_update_rubric@2.0`) → **NOT** mechanical (**medium**) |

Correction to a claim the task brief and the spec's Summary both make: **`coverage-scope`
is not the coverage-gate step's executor.** `coverage-scope.mjs` is a real vendored helper
(TASK-IMP-098) but no skill delegates to it — proven by
`grep -rn 'coverage-scope' modules/` returning nothing outside this task's own spec. Its
own header commits the opposite: judgment fields stay with the author skill. So step 23 is
`medium`, not `mechanical`. Command:
`sed -n '1,12p' tools/install/docs-tools/coverage-scope.mjs`.

## 5. Steps outside the immediate domain

`files_outside_immediate_domain: 0`. The task's declared cone
(`modules/cuo/chief-technology-officer/workflows/ship-tasks.md`,
`modules/cuo/tests/test_workflow_evolution.py`; `service: modules/cuo`) covers every file
written. `modules/skill/**` and `tools/install/**` are READ for evidence and not modified.
**No ADR is triggered** (threshold is >3; step 3 is conditionally skipped).

## 6. Existing patterns the implementation must follow

- **Doctrine sections**: `## 11a` … `## 11d`, each `(vX.Y.Z, TASK-IMP-NNN)` in the heading,
  each opening with the defect it closes, each using bolded lead-ins per bullet. The new
  section follows this shape and is numbered `11e`.
- **Test conventions** (`modules/cuo/tests/test_workflow_evolution.py:208-251`): structural
  doctrine arms read the workflow doc through a module-level `_SHIP_TASKS` path constant
  and a `_ship_tasks_text()` helper that asserts the file exists first; each arm's docstring
  names the AC/clause it proves; `re.search` for the rule's own words.
- **PyYAML is available** (`import yaml` is used across `modules/cuo/cuo/core/catalog.py`),
  so the chain block can be parsed as YAML rather than regex-scraped.

## 7. In-flight manifest impact (checked because the edit is to the workflow being executed)

| Question | Answer | Proof |
|---|---|---|
| Does the edit change step numbering? | No — steps stay 0-31; no step added or removed | `test_batch_economics.sh:96` also forbids adding a chain step; the diff adds only a key per line |
| Does it change the artefact set? | No — `outputs:` block untouched | diff |
| Does it change Resume semantics? | No — that section is untouched, and `workflow_version` stays 2.8.0 | §3 above |
| Does it invalidate IMP-106's manifest? | No — version pin unchanged; IMP-106's artefacts are not in this cone | `docs/tasks/.workflow/TASK-IMP-106/` untouched |
| Does it invalidate this task's own manifest? | No — same reason | `node tools/install/docs-tools/ship-manifest.mjs verify TASK-IMP-115` |
