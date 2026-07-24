---
id: TASK-IMP-144
title: "1.5.0 — transition-locked state engine (close frontmatter bypass)"
template: task@1
type: improvement
module: improvement
status: done
priority: p2
author: "@stephencheng"
department: engineering
created_at: 2026-07-23T18:40:00+00:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: [TASK-CUO-303, TASK-IMP-143]
blocks: []
related_tasks: [TASK-IMP-120]
routed_back_count: 0
awh: N/A
verify: T
phase: "1.5.0"
owner: Stephen Cheng (CTO)
created: 2026-07-23
memory_chain_hash: null
effort_hours: 10
service: tools/install/docs-tools + scripts
new_files:
  - tools/install/docs-tools/task-state.mjs
  - tools/install/tests/test_task_state_engine.sh
  - docs/tasks/_state/README.md
modified_files:
  - tools/install/docs-tools/backlog-mutate.mjs
  - scripts/migrate_improvement_to_task.py
  - scripts/tests/test_regen_backlog.sh
  - modules/cuo/chief-technology-officer/workflows/ship-tasks.md
  - CHANGELOG.md
source_pages:
  - "TASK-CUO-303 residual R-EXT-01: frontmatter edit + regen bypasses flip gate"
  - "ship-tasks.md accepted residual until 1.5.0 state engine"
  - "TASK-IMP-120 truth-precedes-index (flip still requires FM==to)"
source_decisions:
  - "2026-07-25 session operator: implement IMP-143 then IMP-144; HITL auto-approve with evidence."
---

# TASK-IMP-144: 1.5.0 transition-locked state engine

## Summary

Close the TASK-CUO-303 residual where an agent edits `spec.md` frontmatter `status:` and regenerates BACKLOG without ever calling `flip`. A single state-engine API owns status transitions; regenerators refuse to invent transition edges without an engine receipt.

## Problem

`backlog-mutate flip` mechanically locks the two HITL gates, but editing frontmatter to `done` and running `regen_backlog` rewrites the index from frontmatter alone — R-EXT-01 / ship-tasks accepted residual.

## Proposed Solution

1. **Receipts** — every successful `backlog-mutate flip` writes a content-addressed receipt under `docs/tasks/_state/receipts/<task>--<from>--<to>--<sha12>.json` binding task_id, from, to, and (for HITL gates) verdict artifact coordinates.
2. **Regen refusal** — `regen_backlog()` parses the previous BACKLOG; when an existing stem's status would change to the frontmatter value, it requires a matching receipt for that `(task_id, from, to)` triple; otherwise it exits non-zero and writes nothing. New stems (inserts) and identical statuses do not need receipts.
3. **`task-state.mjs transition`** — documented single API: validate args, write frontmatter `status` to `<to>`, invoke `flip` (HITL flags required for gate transitions; IMP-143 artifacts mint), which writes the receipt. Agents MUST use this path; raw FM+regen inventing edges is refused.

## Alternatives Considered

- **Git hooks only.** Rejected: regen is the write path that invents edges; hooks would race and miss non-git callers.
- **Delete regen.** Rejected: regen remains the title/section authority; it must reflect engine-committed status, not invent it.
- **Capability-scoped agent identities.** Deferred to a follow-on 1.5.x milestone per the draft.

## Success Metrics

- Primary: FM edit to `done` + regen without a receipt refuses; `task-state transition` with HITL flags succeeds and leaves FM, BACKLOG, and receipt aligned.
- Guardrail: existing `test_hitl_lock.sh` / `test_regen_backlog.sh` green; IMP-120 truth-precedes-index preserved for bare `flip`.

## Scope

### In scope

- Receipts on flip, regen refusal, `task-state.mjs`, tests, ship-tasks residual retirement, CHANGELOG.

### Out of scope / Non-Goals

- Replacing the ship-tasks skill chain.
- Cross-tool conformance kit.
- Cryptographic agent identity / ACL (follow-on).

## Dependencies

`depends_on: [TASK-CUO-303, TASK-IMP-143]`.

## AI Authorship Disclosure

- **Tools used:** Cursor agent (Composer) from the batch-9 draft.
- **Scope:** full AC authoring + implementation.
- **Human review:** session HITL override with recorded evidence (2026-07-25).

## 1. Description

- 1.1 Every successful `backlog-mutate flip` MUST write one receipt JSON under `docs/tasks/_state/receipts/` with `{schema:"cyberos.transition@1", task_id, from, to, evidence_sha256?}` and a self-hash field.
- 1.2 `regen_backlog` MUST refuse (non-zero, no write) when an existing BACKLOG stem would change status to a different frontmatter status unless a receipt exists for that exact `(task_id, prior_backlog_status, new_frontmatter_status)` triple.
- 1.3 New task stems appearing only in frontmatter (no prior BACKLOG row) MUST still be allowed without a receipt (insert / first index).
- 1.4 `task-state.mjs transition <id> <from> <to>` MUST write frontmatter status to `<to>`, then invoke flip with the same args/flags, and MUST exit non-zero if either step fails (no partial success: if flip fails after FM write, restore prior FM status).
- 1.5 Gate transitions still require IMP-143 / CUO-303 verdict flags; non-gate transitions stay flag-free.
- 1.6 `ship-tasks.md` MUST retire the "accepted residual" sentence for the frontmatter bypass and point at `task-state.mjs` + regen refusal.

## Acceptance Criteria

- [ ] AC 1 (traces_to: #1.1–1.3) — `test_task_state_engine.sh` proves regen refuses invented edges and accepts receipt-backed ones.
- [ ] AC 2 (traces_to: #1.4–1.5) — same suite proves `task-state transition` updates FM+BACKLOG+receipt; gate path needs verdict flags.
- [ ] AC 3 (traces_to: #1.6) — ship-tasks + CHANGELOG document the closure.

## Test plan

1. `bash tools/install/tests/test_task_state_engine.sh`
2. `bash scripts/tests/test_regen_backlog.sh`
3. `bash .cyberos/cuo/gates/run-gates.sh`
