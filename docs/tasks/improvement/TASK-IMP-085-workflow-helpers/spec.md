---
id: TASK-IMP-085
title: Doc-driven workflow helpers, ship-manifest and backlog-mutate CLIs
template: task@1
type: improvement
module: improvement
status: done
priority: p0
author: "@stephencheng"
department: engineering
created_at: 2026-07-16T15:12:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-CUO-205, TASK-CUO-206, TASK-IMP-084]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 hardening"
owner: Stephen Cheng (CTO)
created: 2026-07-16
shipped: 2026-07-16
memory_chain_hash: null
effort_hours: 6
service: tools/install/docs-tools
new_files:
  - tools/install/docs-tools/ship-manifest.mjs
  - tools/install/docs-tools/backlog-mutate.mjs
  - tools/install/tests/test_workflow_helpers.sh
modified_files:
  - tools/install/build.sh
  - modules/cuo/chief-technology-officer/workflows/ship-tasks.md
source_pages:
  - "modules/cuo/chief-technology-officer/workflows/ship-tasks.md Resume semantics (ship-manifest@1: two-phase writes, task_sha256 + workflow_version pins, staleness rules, resume line, queue selection)"
  - "modules/skill/backlog-state-update-author/SKILL.md §2-§3 (old_line optimistic concurrency, insert grammar, uniqueness gate, whole-file discipline)"
  - "IMPROVEMENT_HANDOFF.md IMP-04 (both contracts rode on agent discipline alone in the sachviet and batch-1 runs: manifests skipped, flips executed by hand-sed)"
source_decisions:
  - "2026-07-16 Stephen: PLAN batch 2 approved with this item at p0."
---

# TASK-IMP-085: Doc-driven workflow helpers, ship-manifest and backlog-mutate CLIs

## Summary

Two contracts in the ship loop are executable only by agent discipline today: the ship-manifest resume cache and the backlog-state-update byte rules. Ship two stdlib-only node CLIs in docs-tools - `ship-manifest.mjs` (init, record, verify, resume-line, delete; two-phase atomic; pins and staleness exactly per the workflow's Resume semantics) and `backlog-mutate.mjs` (status-cell flip with old-line byte verification and drift refusal; insert-row with uniqueness gate, grammar, stem placement, and section-count maintenance). Vendored to installed repos; ship-tasks doctrine names them so agents reach for the tool instead of hand-sed.

## Problem

Both real runs on 2026-07-16 (sachviet batch, cyberos batch 1) skipped per-step manifest writes and executed every backlog flip with hand-crafted sed. Nothing failed, but the two strongest disciplines in the contract - resume-after-crash and refuse-on-drift - existed only as prose. Two swarm agents were in fact killed mid-run in batch 1; recovery worked from commit archaeology, which is exactly what the manifest exists to replace.

<untrusted_content source="modules/cuo/chief-technology-officer/workflows/ship-tasks.md">
The manifest MUST be rewritten after EVERY completed, failed, or conditionally-skipped step - no step's outcome goes unrecorded. Writes are two-phase atomic.
</untrusted_content>

## Proposed Solution

`node .cyberos/docs-tools/ship-manifest.mjs <cmd>` and `node .cyberos/docs-tools/backlog-mutate.mjs <cmd>`, stdlib only, deterministic, `--json` envelopes, documented exit codes. The manifest tool implements ship-manifest@1 verbatim (pin at init, per-step artefact sha256, verify walks staleness in the workflow's order, resume-line echoes the mandated format, delete on done). The backlog tool is the byte-discipline executor for backlog-state-update@2: it never moves or deletes rows, refuses when the pre-image drifted, and keeps section-header counts true. Grammar authority stays where it is; the tool encodes it. Both vendor through guarded build.sh copies (the 084 lesson: docs-tools does not auto-vendor) and the ship-tasks doctrine gains two pointer sentences so doc-driven agents discover them.

## Alternatives Considered

- Keep discipline in prose and trust agents. Rejected: two real runs show the discipline is skipped under time pressure precisely when it matters (crash recovery).
- Python implementations beside repair_task_yaml.py. Rejected: consumer repos are guaranteed node by the MCP/status tooling, not python3 versions; docs-tools' executable convention is .mjs.
- One combined tool. Rejected: the two contracts version independently (ship-manifest@1 vs backlog-state-update@2) and only one of them is per-task session state.

## Success Metrics

- Primary: both contracts become machine-executed - a resumed run reproduces the workflow's resume line from the manifest, and a drifted backlog flip is refused with a non-zero exit instead of silently landing. Baseline: 0 of 2 contracts tool-backed (both runs to date). Deadline: this task's final acceptance.
- Guardrail: the suite's atomicity and drift fixtures run on every gate execution.

## Scope

In scope: the two CLIs, their suite, guarded build.sh vendor copies, two doctrine pointer sentences in ship-tasks.md.

### Out of scope / Non-Goals

- Changing either contract's semantics (the tools implement, never redefine).
- BRAIN memory emission (IMP-05, separate).
- Per-task coverage scoping (IMP-14, separate).
- Rewiring existing skills' prose beyond the two pointer sentences.

## Dependencies

- None upstream; reuses TASK-IMP-084's frontmatter-reader approach conceptually (fresh code, no import coupling). Cone-disjoint from TASK-IMP-086 (BACKLOG.md content) and TASK-IMP-087 (docs/release/): this task's suite mutates only fixture backlogs in TMP.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted by the model from IMPROVEMENT_HANDOFF.md IMP-04 and the two contract sources; implementation follows under ship-tasks supervision.
- **Human review:** PLAN approved by the operator on 2026-07-16; spec audit and both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 `ship-manifest.mjs` MUST provide `init <task-id> --task-file <spec> --workflow-version <v>`, `record <task-id> <step> <skill> <status> [--artefact <path>] [--verdict <v>]`, `verify <task-id>`, `resume-line <task-id>`, and `delete <task-id>`, operating on `docs/tasks/.workflow/<task-id>.ship.json`.
- 1.2 Every manifest write MUST be two-phase atomic (`.tmp.<nonce>` then rename); `task_sha256` (of the spec at init) and `workflow_version` MUST be pinned at init; each recorded step MUST carry `{index, skill, status, artefact_path, artefact_sha256, verdict, completed_at}` with the artefact hashed at record time.
- 1.3 `verify` MUST implement the workflow's staleness order with distinct exit codes: workflow_version mismatch -> exit 3 (needs_human); task_sha256 mismatch -> exit 4 (all steps stale, history and routed_back_count retained); earliest artefact hash mismatch -> exit 5 reporting that step; all intact -> exit 0 with the first non-done step. `resume-line` MUST echo the workflow's mandated format (`resume <task-ID>: steps 1-N verified (K artefacts, hashes OK), continuing at step M/31 (<skill>). routed_back_count=R`).
- 1.4 `backlog-mutate.mjs flip <task-id> <from> <to> [--backlog <path>]` MUST locate the row by stem, verify both the status cell and the full old line byte-for-byte against the pre-image, refuse on drift or missing row with exit 6, rewrite exactly that line, and update the containing section header's status counts when the header carries counts.
- 1.5 `backlog-mutate.mjs insert <task-id> <stem> <title> <status> [--backlog <path>]` MUST enforce the uniqueness pre-image (no row for the id anywhere, exit 7 on violation), emit the regenerator-identical row grammar used by the target section, place the row in stem-ascending order inside the section's contiguous block, and update header counts.
- 1.6 Both tools MUST be node-stdlib-only and deterministic, support `--json` result envelopes, document exit codes in `--help`, and never modify any line outside the declared mutation (whole-file discipline; flip and insert change exactly one row plus at most one header line).
- 1.7 build.sh MUST vendor both tools into the payload docs-tools via guarded copies, gated by the suite (a payload without them fails t08).
- 1.8 ship-tasks.md MUST gain two pointer sentences: Resume semantics names `ship-manifest.mjs` as the doc-driven reference implementation alongside `ship_manifest.py`; the backlog-layout section names `backlog-mutate.mjs` as the byte-discipline executor for backlog-state-update writes.
- 1.9 The suite MUST land at `tools/install/tests/test_workflow_helpers.sh` covering the lifecycle, atomicity, staleness exits, drift refusal, uniqueness, counts, determinism, payload carry + install lay-down, and doctrine wiring.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: §1 #1.1) - full manifest lifecycle init/record/verify/resume-line/delete - test: `tools/install/tests/test_workflow_helpers.sh::t01_manifest_lifecycle`
- [ ] AC 2 (traces_to: §1 #1.2) - two-phase atomicity: a planted stale tmp never corrupts the manifest; pins recorded at init - test: `tools/install/tests/test_workflow_helpers.sh::t02_two_phase_atomic`
- [ ] AC 3 (traces_to: §1 #1.3) - staleness exits 3/4/5/0 and the exact resume line - test: `tools/install/tests/test_workflow_helpers.sh::t03_verify_staleness_exits`
- [ ] AC 4 (traces_to: §1 #1.4) - flip rewrites one line; drifted pre-image refused with exit 6 - test: `tools/install/tests/test_workflow_helpers.sh::t04_flip_and_drift_refusal`
- [ ] AC 5 (traces_to: §1 #1.5) - insert uniqueness (exit 7), grammar, stem placement - test: `tools/install/tests/test_workflow_helpers.sh::t05_insert_uniqueness_and_grammar`
- [ ] AC 6 (traces_to: §1 #1.4, #1.5) - header counts stay true across flips and inserts - test: `tools/install/tests/test_workflow_helpers.sh::t06_counts_maintained`
- [ ] AC 7 (traces_to: §1 #1.6) - --json envelopes, documented exit codes, byte-identical reruns, whole-file discipline (diff = 1 row + <=1 header) - test: `tools/install/tests/test_workflow_helpers.sh::t07_json_and_determinism`
- [ ] AC 8 (traces_to: §1 #1.7) - payload carries both tools; scratch install lays them into .cyberos/docs-tools/ - test: `tools/install/tests/test_workflow_helpers.sh::t08_payload_and_install`
- [ ] AC 9 (traces_to: §1 #1.8) - doctrine pointers present in ship-tasks.md source and in the scratch payload's cuo/ship-tasks.md - test: `tools/install/tests/test_workflow_helpers.sh::t09_doctrine_wiring`
- [ ] AC 10 (traces_to: §1 #1.9) - the suite exists at its mandated path and is discovered by the tools/install/tests glob - verify: `bash scripts/tests/run_all.sh` lists test_workflow_helpers.sh among suites (ops check recorded in the gate log; glob discovery is the runner's contract).

## 3. Edge cases

- Kill mid-write: only `.tmp.<nonce>` remains; next read ignores tmp files; t02 plants one and proves the main file wins.
- Manifest for a task whose spec moved: verify reports task-hash staleness (exit 4), never a crash (t03).
- Flip target row present twice (corrupted backlog): refuse with exit 6 naming both lines - never guess (t04).
- Insert into a section whose header carries no counts (sachviet lifecycle layout): row lands, no header edit (t05).
- Insert into an empty section (placeholder line only): placed as the first row of the block (t05).
- CRLF backlog files: bytes preserved outside the mutated line; the tool neither normalizes nor introduces line-ending drift (t07's whole-file diff proves it).
- Unicode titles in rows: stem sort is bytewise on the stem token only, titles never affect placement (t05).
- Concurrency: two flips racing is out of scope by design - single-writer discipline stays with the workflow; the drift refusal is the guard when it is violated anyway (documented; t04 is the mechanism).
- Security-class: tools read and write only the declared files under docs/tasks/; no shell-out, no network, no eval; `--json` output is data. Covered by code review plus t07's footprint diff.

## 4. Out of scope / non-goals

Duplicated intentionally with `## Scope` for template conformance: no contract semantics changes, no BRAIN emission, no coverage scoping.

## 5. Protected invariants this task must not weaken

- Frontmatter status remains the record of truth; the backlog stays an index (tools repair toward frontmatter, never the reverse).
- backlog-state-update grammar authority is unchanged; the tool encodes, never redefines.
- Payload sync doctrine: rebuild, version-sync, full suite before commit.
- HITL: both human-acceptance gates are recorded verdicts; the agent never sets done.

*End of TASK-IMP-085.*
