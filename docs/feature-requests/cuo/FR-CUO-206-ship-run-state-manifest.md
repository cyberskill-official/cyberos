---
id: FR-CUO-206
title: "Ship run-state manifest (ship-manifest@1) - resumable 31-step chain + depends_on-aware queue selection"
module: cuo
priority: MUST
status: done
class: product
verify: T
phase: Wave C - strengthen the workflows
owner: Stephen Cheng (CTO)
created: 2026-07-12
shipped: 2026-07-12
memory_chain_hash: null
related_frs: [FR-CUO-205, FR-CUO-207, FR-SKILL-118]
depends_on: []
blocks: []
source_pages:
  - modules/cuo/chief-technology-officer/workflows/ship-feature-requests.md
  - modules/skill/feature-request-author/references/MANIFEST_SCHEMA.md
  - modules/skill/contracts/feature-request/STATUS-REFERENCE.md
source_decisions:
  - "2026-07-12 operator goal: the two workflows will run constantly across many repos; an interrupted ship (session end, crash, context limit) currently restarts the 31-step chain from memory of the backlog status alone, re-doing completed artefact steps."
  - "Precedent: feature-request-author already runs on an ephemeral re-entrancy manifest (two-phase atomic writes, phase computed from state). Ship gets the same discipline."
  - "Manifests are session state, not backlog artifacts: gitignored, FR frontmatter stays the record of truth."
language: markdown + JSON (workflow doc + schema contract) + python (schema test)
service: modules/cuo/
new_files:
  - modules/skill/contracts/feature-request/SHIP-MANIFEST.md
  - modules/cuo/tests/test_ship_manifest.py
  - docs/feature-requests/.workflow/.gitignore
modified_files:
  - modules/cuo/chief-technology-officer/workflows/ship-feature-requests.md
  - modules/cuo/EXECUTION-DISCIPLINE.md
  - tools/cyberos-init/plugin/skills/ship-feature-requests/SKILL.md
---

# FR-CUO-206: Ship run-state manifest

## §1 - Description

Give /ship-feature-requests the same re-entrancy anchor its authoring sibling already has: a per-FR manifest that records which of the 31 steps completed, over which artefacts, so a new session resumes instead of re-deriving - plus deterministic queue selection when no FR id is given.

Normative clauses:

1. A contract `ship-manifest@1` MUST be defined at `modules/skill/contracts/feature-request/SHIP-MANIFEST.md` with fields: `manifest_version` (const `ship-manifest@1`), `fr_id`, `fr_sha256` (hash of the FR spec file at run start - a later mismatch marks the whole manifest stale), `workflow_version` (from the workflow doc), `started_at`, `updated_at`, `current_step` (1..31), `routed_back_count`, `steps[]` - each `{index, skill, status (pending|done|failed|skipped-conditional), artefact_path, artefact_sha256, verdict, completed_at}` - and `hitl` (`{gate: null|review_approval|final_acceptance, requested_at}`).
2. The ship workflow MUST write the manifest to `docs/feature-requests/.workflow/<FR-ID>.ship.json` after EVERY completed, failed, or conditionally-skipped step, using two-phase atomic writes (`.tmp.<nonce>` then rename), mirroring the memory-protocol write discipline.
3. On invocation for an FR whose manifest exists with matching `workflow_version`, ship MUST resume at the first non-done step AFTER re-verifying every recorded `artefact_sha256` against disk; a mismatch marks that step and all later steps stale (redo from the earliest stale step). A `workflow_version` mismatch MUST route to needs_human, never a silent mixed-version run.
4. Invoked WITHOUT an FR id, ship MUST select deterministically: among FRs at `ready_to_implement` whose `depends_on` are all `done`, order by priority (MUST before SHOULD before COULD), then `created` ascending, then id ascending; the selection and its reasoning line MUST be echoed to the operator before step 1 runs.
5. Manifests MUST be gitignored via a scaffolded `docs/feature-requests/.workflow/.gitignore` (content: `*.ship.json`); FR frontmatter and BACKLOG.md remain the only committed state. `/init` MUST scaffold the same ignore file in target repos.
6. On the FR reaching `done` (HITL gate 2 passed), ship MUST delete the manifest; on route-back to `ready_to_implement`, ship MUST keep it with `routed_back_count` incremented (the next run starts fresh at step 1 by §1 #3's staleness rule but retains the count and history).
7. The workflow doc MUST gain a `## Resume semantics` section and EXECUTION-DISCIPLINE.md a pointer to it; the plugin wrapper SKILL.md MUST mention resume-on-restart so agents look for the manifest before starting step 1.
8. HITL gates MUST NOT be inferable from the manifest alone: resuming at a gate step re-requests the human approval; a recorded `hitl.requested_at` never substitutes for the approval itself.

## §2 - Why this design

The manifest is a cache of proven work, never an authority: every resume re-hashes artefacts, and human gates always re-ask. That keeps the two-source-of-truth risk (manifest vs backlog) at zero - if the manifest lies or is deleted, the worst case is redoing work, the exact status quo. JSON-on-disk with atomic writes copies the pattern already proven by the authoring manifest; queue selection turns "pick the next eligible one" from prose into an algorithm agents apply identically across repos.

## §3 - Contract

```json
{
  "manifest_version": "ship-manifest@1",
  "fr_id": "FR-TEN-208",
  "fr_sha256": "4c1e...",
  "workflow_version": "2.3.1",
  "started_at": "2026-07-12T10:00:00+07:00",
  "updated_at": "2026-07-12T11:42:10+07:00",
  "current_step": 11,
  "routed_back_count": 0,
  "steps": [
    {"index": 1, "skill": "repo-context-map-author", "status": "done",
     "artefact_path": "docs/feature-requests/.workflow/FR-TEN-208.rcm.md",
     "artefact_sha256": "9f2c...", "verdict": "pass", "completed_at": "2026-07-12T10:12:00+07:00"},
    {"index": 3, "skill": "architecture-decision-record-author", "status": "skipped-conditional",
     "artefact_path": null, "artefact_sha256": null, "verdict": null, "completed_at": "2026-07-12T10:13:00+07:00"}
  ],
  "hitl": {"gate": null, "requested_at": null}
}
```

## §4 - Acceptance criteria

1. **Schema is normative and validated** (§1 #1) - SHIP-MANIFEST.md defines every field with types/enums; the §3 example and the fixtures validate against it programmatically.
2. **Write-after-every-step, atomically** (§1 #2) - the workflow doc mandates the write points and the tmp+rename discipline; no step's completion is unrecorded.
3. **Resume skips proven work** (§1 #3) - fixture: manifest with steps 1-10 done and artefacts intact -> resume plan says step 11; corrupting step 5's artefact makes the resume plan restart at 5 with 5..31 stale.
4. **Version mismatch halts** (§1 #3) - manifest at 2.3.0 vs workflow 2.3.1 -> needs_human, no auto-run.
5. **Queue selection is total and deterministic** (§1 #4) - fixture backlog (mixed statuses, unmet depends_on, tied priorities) yields one defined winner; re-running yields the same; the reasoning line matches the fixture expectation.
6. **Gitignore scaffolding** (§1 #5) - the ignore file exists with `*.ship.json`; `git status` in a fixture repo shows no manifest after a simulated run; init.sh scaffolds it in a scratch target.
7. **Terminal handling** (§1 #6) - done deletes the manifest; route-back keeps it with the incremented count (fixture pair).
8. **Human gates re-ask on resume** (§1 #8) - resuming a manifest parked at step 19/31 (gates) produces a fresh approval request; the doc forbids treating requested_at as approval.

## §5 - Verification

```python
# modules/cuo/tests/test_ship_manifest.py
def test_schema_fields_and_example_validate():      # AC 1
def test_atomic_write_discipline_documented():      # AC 2  (workflow doc contains the write-point + tmp/rename clauses)
def test_resume_plan_intact_and_stale():            # AC 3  (pure function over fixture manifests + artefact dir)
def test_workflow_version_mismatch_needs_human():   # AC 4
def test_queue_selection_total_order():             # AC 5  (fixture FR set -> expected id; idempotent)
def test_gitignore_scaffold():                      # AC 6
def test_done_deletes_routeback_keeps():            # AC 7
def test_hitl_reask_on_resume():                    # AC 8  (doc assertion + fixture plan marks gate pending)
```

(The resume planner and queue selector are specified in the workflow doc precisely enough to implement as small pure helpers under modules/cuo for testability; agents follow the same algorithm doc-driven in reduced profile.)

## §6 - Implementation skeleton

SHIP-MANIFEST.md mirrors MANIFEST_SCHEMA.md's structure (field table, lifecycle, atomicity, staleness). Workflow doc: add manifest write-points to the step protocol preamble, the Resume semantics section (staleness rule, version rule, gate re-ask), and the queue algorithm where the doc currently says "next eligible FR".

## §7 - Dependencies

None hard. FR-CUO-207's config later adds nothing here (manifest location is fixed). Interacts with FR-CUO-205 only at the shared backlog-write skill, unchanged for ship. FR-SKILL-118's coverage-gate rubric constants are read at their steps regardless of resume.

## §8 - Example payloads

Resume echo line (operator-facing):

```
resume FR-TEN-208: steps 1-10 verified (10 artefacts, hashes OK), continuing at step 11/31 (observability-injection-author). routed_back_count=0
```

## §9 - Open questions

None blocking. Cross-repo parallel shipping (two agents, two different FRs, one repo) is naturally safe - one manifest per FR; two agents on the SAME FR is out of scope and remains an operator error the backlog's status cell already surfaces.

## §10 - Failure modes inventory

1. Crash between artefact write and manifest write - resume re-verifies hashes; the missing manifest entry means the step re-runs, idempotent by skill design.
2. Manifest edited by hand to skip a gate - §1 #8: gates re-ask regardless of manifest content; the manifest cannot authorize anything.
3. Stale manifest after FR spec edits (FR re-audited mid-flight) - covered by the schema's `fr_sha256` root field (§1 #1): mismatch at resume marks every step stale, forcing a clean re-run against the revised spec.
4. .workflow dir deleted - clean restart from step 1; no correctness loss (cache semantics).
5. Clock skew across sessions - ordering uses step indices, not timestamps; timestamps are informational only.

## §11 - Implementation notes

Keep the manifest strictly derived (cache) - the words "record of truth" appear only next to FR frontmatter in every doc touched. The queue reasoning line format is part of the contract (operators grep session logs for it).

*End of FR-CUO-206.*
