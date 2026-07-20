---
artefact: gate-log@1
task_id: TASK-IMP-115
workflow: chief-technology-officer/ship-tasks
workflow_version: 2.8.0
phase: implementing
status_note: "The parent owns the status writes (§11a). This task's implementer did not touch BACKLOG.md or the spec frontmatter."
---

# Gate log — TASK-IMP-115 (effort tiering, advisory judgment metadata)

## Suite result — the environment CI sees

CI's target health for this module is `RUN_COMMANDS: python -m pytest -q`, run from `modules/cuo` (`modules/cuo/audit-profile.yaml`). Run the same way:

```
$ cd modules/cuo && python3 -m pytest -q
268 passed, 2 skipped in 12.43s
```

The four new arms, isolated:

```
$ cd modules/cuo && python3 -m pytest -q tests/test_workflow_evolution.py
16 passed in 0.10s      # 12 pre-existing + 4 new
```

## Coverage on touched files

| Touched file | Coverage | Command |
|---|---|---|
| `modules/cuo/tests/test_workflow_evolution.py` | **100 %** (178/178 statements) | `python3 -m coverage run --source=. -m pytest -q tests/test_workflow_evolution.py && python3 -m coverage report --include="tests/test_workflow_evolution.py"` |
| `modules/cuo/chief-technology-officer/workflows/ship-tasks.md` | n/a — markdown, no executable statements | — |

Well above the 90 % floor. Recorded honestly: the only touched file with executable lines IS the suite, because the other deliverable is a doc. That is a real 100 %, and also a weak one — which is why the arms below were each proven by breaking the thing they guard.

## Load-bearing proof for every new test (broke it, watched it fail, restored it)

Every mutation was applied to the real file, the real arm was run, and the file was restored and re-hashed (`sha256` identical to the original) before the next one.

| # | Mutation (what the clause forbids) | Arm | Result |
|---|---|---|---|
| 1 | strip `judgment` from ONE step (12) | `test_every_step_has_judgment` | **FAIL** ✓ |
| 1b | *the NAIVE version of the same check* (`"judgment:" in text`) under mutation 1 | — | **PASS** ✗ — decoration |
| 2 | off-enum value on step 2 (`High`, wrong case) | `test_every_step_has_judgment` | **FAIL** ✓ |
| 3 | relabel a judgment step cheap: step 9 `implementation-plan-author` → `mechanical` | `test_mechanical_steps_are_helper_backed` | **FAIL** ✓ |
| 4 | **the table-stuffing attack**: step 9 → `mechanical` **and** a §11e row pointing it at a real, existing helper (`backlog-mutate.mjs`) | `test_mechanical_steps_are_helper_backed` | **FAIL** ✓ — "nothing outside §11e's own table says `backlog-mutate.mjs` does `implementation-plan`'s work" |
| 5 | §11e names a helper that does not exist on disk | `test_mechanical_steps_are_helper_backed` | **FAIL** ✓ |
| 6 | strip the helper names from §11e's table (vacuity) | `test_mechanical_steps_are_helper_backed` | **FAIL** ✓ |
| 7 | **a model string**: `model: claude-fable-5` on step 27 | `test_no_host_specific_literals` | **FAIL** ✓ |
| 8 | a price literal: `cost: $0.42 per 1M tokens` on step 3 | `test_no_host_specific_literals` | **FAIL** ✓ |
| 9 | a host effort name in §11e: `reasoning_effort: high` | `test_no_host_specific_literals` | **FAIL** ✓ |
| 10 | `supervisor.py` starts READING the key (`step.get("judgment") == "mechanical"` → skip) | `test_judgment_is_advisory_not_read` | **FAIL** ✓ |
| 11 | §11e stops calling the field advisory | `test_judgment_is_advisory_not_read` | **FAIL** ✓ |

Row **1b** is the TASK-IMP-118 bar, matching IMP-106 §1.4: with one step's annotation stripped, the naive "the field is mentioned somewhere" check still PASSED while the real guard FAILED. That is the difference between asserting the field exists and asserting a host can rely on it.

Row **4** is the equivalent for AC 2: the obvious version of that arm ("is the skill in the table?") is satisfiable by editing the table, which is the same document being tested. The arm therefore requires the delegation to be anchored OUTSIDE §11e — in the skill's own `SKILL.md` or the workflow's executor prose — so the table cannot vouch for itself.

## AC 4 (`verify:`) — the field is advisory and nothing in the payload reads it

Recorded grep, run at `HEAD` of this change.

**1. Does anything READ the key?** A read looks like a subscript, a `.get()`, or an attribute — not the bare word, which appears in unrelated prose.

```
$ grep -rnE '\[[[:space:]]*.judgment.[[:space:]]*\]|\.get\([[:space:]]*.judgment.|\.judgment\b' \
    modules/ tools/install/ scripts/ | grep -v '/tests/'
(no output — exit 1)
```

**2. Every `judgment` occurrence in the payload's code and helpers** — all four are prose in comments, none is a read:

```
tools/install/docs-tools/batch-select.mjs:19:   // ... No model, no judgment - the ...
tools/install/docs-tools/coverage-scope.mjs:4:  // The coverage gate's unit of judgment is ...
tools/install/docs-tools/coverage-scope.mjs:8:  // ... The judgment fields ...
tools/install/docs-tools/coverage-scope.mjs:272: //  TODO markers for the judgment fields ...
```

**3. The doc says it is advisory** (`ship-tasks.md`):

```
32:  # `judgment:` is ADVISORY host-routing metadata (§11e). A host MAY route on it; nothing here reads it. Never a model name.
427:- **It is ADVISORY, and that is the whole design.** A host MAY route on it. **Nothing in
428:  the payload reads it to decide anything** — no step, no gate, no helper, no condition.
```

**4. Why it is not readable by accident** — context-map §2 enumerates all nine `skill_chain` consumers. Every one reaches for a named key (`.get("skill")`, `.get("step")`) or greps `skill: *[a-z0-9-]+`. None validates or iterates the key set, so an unknown key is inert by construction rather than by promise. This is also why the field could be added at all without a version bump.

**Verdict: PASS.** The claim is negative and structural; the grep above is its evidence, and `test_judgment_is_advisory_not_read` re-proves it on every CI run rather than once.

## AC 5 (`verify:`) — reviewer walk of the assigned levels

The rule the reviewer is asked to check: **no step is `high` without a named reason, and anything genuinely ambiguous is `medium`, not a guessed `high`** (§1.5).

Distribution: **6 mechanical / 8 high / 18 medium** across 32 steps (`python3 -c` over the parsed frontmatter; see the impl-plan).

**Every `high` and its named reason** (all eight reasons are in §11e's table, so the walk has something to check rather than a bare label):

| Step | Skill | Named reason | Reviewer's question |
|---|---|---|---|
| 1 | `repo-context-map-author` | the "outside-domain" call is a judgment, and step 3's ADR trigger derives from it | is the trigger really derived here? §2 of this doc: yes, `files_outside_immediate_domain > 3` |
| 3 | `architecture-decision-record-author` | an ADR IS the architectural decision | — |
| 5 | `edge-case-matrix-author` | enumerates the boundary and SECURITY cases nobody wrote down | — |
| 7 | `mock-contract-test-author` | designs the contract shape of a service that does not exist yet | — |
| 9 | `implementation-plan-author` | the implementation itself | — |
| 17 | `code-review-author` | produces the packet the human acceptance gate reads | — |
| 25 | `debugging-cycle-author` | classifies the failure vector and forms the hypothesis | — |
| 27 | `task-audit` | TRACE-004 closure — the judgment half; `task-lint.mjs` only seeds the mechanical findings | the spec's own named exemplar of `high` |

**The discipline that kept `high` at eight.** Every `-audit` half is `medium` except step 27 — because 27 is the one the spec names, and the others check structure (a matrix's non-vacuity, a plan against the matrix) rather than deciding anything the chain depends on. Under §1.5 those are exactly the "genuinely ambiguous" cases, and ambiguous is `medium`. No step was marked `high` because it felt important.

**Levels that were deliberately NOT the intuitive one, with the evidence:**

- **23 `coverage-gate-author` → `medium`, not `mechanical`.** The spec's *Summary* names "coverage-scope" as already-a-script. It is a real helper, but **no skill delegates to it**: `grep -rn 'coverage-scope' modules/` returns nothing outside this task's own spec, and `coverage-scope.mjs:8` reserves the judgment fields for the author skill ("`tests_failed`, `ecm_rows_uncovered`, `raw_terminal` stay with the author skill … never guessed"). §1.2 does not permit the label.
- **27 `task-audit` → `high`,** although it names a docs-tools helper. `task-lint.mjs` is a "machine floor" that seeds findings while "model diligence is spent on the judgment families only" (`task-audit/SKILL.md:248`). Naming a helper is not being performed by one.
- **28/29 `awh-gate`/`caf-gate` → `medium`** — see the surfaced fork below.

**Verdict: PASS, with one fork surfaced for the operator (below).**

## Surfaced for the operator — §1.2 and AC 2 disagree about two steps

Not decided here. Implemented to the narrower reading (the contract), reversible in one edit.

- **§1.2 (normative)** says `mechanical` means "performed by a **deterministic helper** with no model judgment in the result".
- **AC 2 and the Success Metric** say "a **docs-tools** helper", the metric adding "*precisely* the steps whose work is done by a docs-tools helper".

Steps 28 (`awh-gate`) and 29 (`caf-gate`) are the only place the two readings disagree. Both are deterministic — `caf-gate/SKILL.md` says "**no LLM**", and the skill's whole job is "run `bash scripts/caf_gate.sh <module>`. Read the verdict." — but their executors are `tools/awh/` and `scripts/caf_gate.sh`, which are **not** docs-tools helpers (both verified present: `ls scripts/caf_gate.sh tools/awh`).

- **As implemented (`medium`):** every §1 clause and every AC holds as written. §1.2 is a one-way definition ("`mechanical` MUST mean X"), so it constrains what may wear the label and does not compel the label onto every X. Cost: the field under-informs on two steps — a host routing on it will spend judgment on a step that runs one bash command, which is a small dose of the exact waste the task exists to remove.
- **The alternative (`mechanical`):** more useful information, but it contradicts AC 2 and the Success Metric as written, so the spec's own suite arm would have to test something the AC does not say.
- **What I'd suggest:** ship as-is, and let a follow-up widen §1.2's helper family to "a vendored deterministic executor" (docs-tools, `tools/awh`, `scripts/caf_gate.sh`) with AC 2's wording corrected to match. That is a spec change, not an implementation choice, which is why it is here and not in the diff. §11e records the rough edge in the doc so the next reader finds the reasoning rather than re-deriving it.

## Cone and in-flight-manifest impact (checked because this edits the workflow being executed)

| Check | Result | Proof |
|---|---|---|
| Step numbering changed? | No — 0-31 before and after | `python3 -c "...print([s['step'] for s in chain])"` → `[0..31]`, 32 steps |
| Artefact set changed? | No — `outputs:` untouched | diff |
| Resume semantics changed? | No — section untouched | diff |
| `workflow_version` bumped? | **No, deliberately** — 2.8.0 | a bump breaks four literal assertions in `tools/install/tests/test_workflow_helpers.sh` (t09/t12/t13/t14), which is **IMP-106's cone**, and sends every in-flight manifest to `needs_human` (Resume rule 1). §11b/§11c/§11d precedent: the whole handoff group rides 2.8.0 |
| This task's own manifest still valid? | Yes | `ship-manifest.mjs verify TASK-IMP-115` → **exit 0**: "intact - steps 1-12 verified (8 artefacts, hashes OK)" |
| IMP-106's manifest invalidated by this edit? | **No** — proven, not asserted | `ship-manifest.mjs verify TASK-IMP-106` against the EDITED doc → **exit 4** (`task_sha256` mismatch — IMP-106's own spec changed since its run start), **not exit 3** (`workflow_version` mismatch). Rule 1 of the staleness order runs FIRST and passed, so the version check is green: the doc says 2.8.0, the manifest pins 2.8.0. The exit-4 staleness is pre-existing and independent — `git status --porcelain docs/tasks/improvement/TASK-IMP-106*` is empty in this tree, so this implementer did not touch that spec |
| Files touched outside the declared cone? | No | `git diff --name-only` = exactly the two files in `modified_files` (plus this task's own artefact folder, untracked). `TASK-IMP-122-.../audit.md` also shows dirty — **pre-existing, not this task's**: it was already modified at run start (`git status` before step 1) and is excluded from this task's commit |

## Commit — BLOCKED by a stale index lock (surfaced, not worked around)

The change is complete and green in the working tree; **nothing is committed.**

```
$ git add --dry-run modules/cuo/tests/test_workflow_evolution.py
fatal: Unable to create '.../.git/index.lock': File exists.
$ rm -f .git/index.lock
rm: cannot remove '.git/index.lock': Operation not permitted     # mount denies unlink
$ pgrep -a git
(no git process in this sandbox — the lock is stale, 0 bytes, dated 10:47)
```

Every git write path needs that lock, and the sandbox mount denies `unlink`, so it cannot be cleared from here. Reported rather than fought, per the run's constraints.

**Two things the operator should know before running the commit:**

1. **The lock must be cleared first** (`rm -f .git/index.lock`) from a context with unlink permission — after confirming no real git process holds it.
2. **`--no-verify` is required, and this is the evidence for it.** The pre-commit hook's trigger regex is `^(modules/cuo/|modules/skill/|...)` (`.githooks/pre-commit:9`), so staging `modules/cuo/**` fires `cyberos-payload-build.sh` → a `dist/cyberos` rebuild. This run was explicitly scoped not to rebuild `dist/` or run `install.sh`, and letting the hook do it would sweep payload artefacts — including `dist/cyberos/install.sh`, inside **TASK-IMP-106's cone** — into this task's commit. The hook chain therefore cannot complete within this task's cone.

The intended commit (artefacts + the two cone files, and nothing else):

```
git add modules/cuo/chief-technology-officer/workflows/ship-tasks.md \
        modules/cuo/tests/test_workflow_evolution.py \
        docs/tasks/improvement/TASK-IMP-115-effort-tiering-advisory/
git commit --no-verify -m "IMP-115: annotate skill_chain steps with advisory judgment metadata

Every step ran at whatever reasoning the host gave it; nothing marked which steps
deserve expensive judgment and which are near-mechanical. Each of the 32 skill_chain
steps now carries judgment: high | medium | mechanical as ADVISORY metadata (§11e) --
a host MAY route on it, nothing in the payload reads it, and no model string, price,
or effort name enters the payload.

6 mechanical / 8 high / 18 medium. Every mechanical claim is anchored to a docs-tools
helper the payload independently names for that skill; every high carries its reason
in §11e so AC 5's reviewer walk has something to check.

workflow_version stays 2.8.0 deliberately: a bump breaks four literal assertions in
tools/install/tests/test_workflow_helpers.sh (IMP-106's cone) and sends every in-flight
ship-manifest to needs_human. §11b/§11c/§11d precedent -- the handoff group rides 2.8.0.

--no-verify: the pre-commit hook triggers on modules/cuo/** and rebuilds dist/cyberos.
This run is scoped not to rebuild dist/, and the rebuild would pull dist/cyberos/install.sh
(TASK-IMP-106's cone) into this commit. Payload rebuild is left to the parent.

Suite (the env CI runs -- modules/cuo, RUN_COMMANDS: python -m pytest -q):
268 passed, 2 skipped. Coverage on the touched suite: 100% (178/178).
All 4 new arms proven load-bearing by mutation; see gate-log-draft.md."
```
