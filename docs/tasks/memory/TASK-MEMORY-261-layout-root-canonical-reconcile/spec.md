---
id: TASK-MEMORY-261
title: Reconcile layout-root-canonical with the store scaffold (top-level artifact dirs)
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: improvement
created_at: 2026-07-14T00:00:00+07:00
priority: p2
department: engineering
author: "@unassigned"
template: task@1
module: memory
status: draft
owner: unassigned
---

# TASK-MEMORY-261 - Reconcile layout-root-canonical with the store scaffold

## Context

`cyberos doctor` runs the `layout-root-canonical` invariant (`modules/memory/cyberos/core/invariants.py`, `_CANONICAL_TOP_LEVEL_DIRS`), which accepts exactly: `memories, meta, company, module, member, client, project, persona, conflicts, exports, index, audit`.

CyberOS's own live BRAIN (now at `.cyberos/memory/store/`) also carries top-level `adrs/`, `audits/`, `impl-plans/`, `code-reviews/`, `obs-injections/` - CUO artifact kinds created by earlier tooling. So `doctor` reports `overall: FAIL (12 pass / 1 error)` on an otherwise healthy store. The shell scaffolder was already fixed to stop creating these (commit `0ae91c4`), so fresh stores are clean; this task is about the pre-existing dirs and the source-of-truth split.

## 1. Normative clauses

1. There MUST be a single source of truth for the canonical top-level set, shared by the invariant (`_CANONICAL_TOP_LEVEL_DIRS`) and both scaffolders (`install.sh` and `__main__.py::_auto_init_if_needed`). Today the invariant and the scaffolders enumerate the set independently.
2. A decision MUST be recorded (ADR) on the five artifact dirs: either (a) they are legitimate top-level kinds and MUST be added to the canonical set, or (b) they are not, and their memories MUST nest under an accepted kind (for example `meta/` or a dedicated `artifacts/` root).
3. If (b), a one-shot, reversible migration MUST relocate existing rows and remove the empty legacy dirs, and MUST leave the audit ledger intact (no chain rewrite).
4. After the change, `cyberos doctor` MUST report `overall: OK` on a store that only ever held valid data (no spurious `layout-root-canonical` error), and the memory test suite MUST stay green.

## 2. Scope

`modules/memory/cyberos/core/invariants.py`, the two scaffolders, and (if clause 3) a migration/cleanup helper. No change to the audit-chain format or the write path.

## 3. Gate

`python -m pytest` in `modules/memory` plus `cyberos doctor` on a seeded store. Improvement-class task: machine gates plus the two HITL acceptance verdicts.
