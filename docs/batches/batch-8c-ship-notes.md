---
batch: ship/batch-8c-memory-notes
members: []
started: 2026-07-23T17:50:00+07:00
route_backs: 0
gate_reasks: 0
tokens: unknown
---
# Batch 8C ship notes — gate-1 → testing friction

Branch: `ship/batch-8c-memory`  
Date: 2026-07-23  
Halted at gate-2 for MEMORY-303 (not done). IMP-138 not implemented.

## Friction

### 1. MMR desync on every gated flip (recurring)

`memory-append.mjs` `status_overridden` for MEMORY-303 gate-1 advanced HEAD to 13 without updating `peaks.bin` (was 12 after Batch B rebuild). Same human rebuild as Batch 8B / MEMORY-303 repair §4. Backup: `/tmp/peaks-bin-pre-batch8c-mmr-rebuild.bin`.

Until appenders update MMR, treat rebuild as a mandatory pre-`run-gates.sh` step after any gated flip.

### 2. ready_to_review → reviewing normalization

STATUS-REFERENCE gate-1 is `reviewing → ready_to_test`. Task was left at `ready_to_review` after repair. Claimed `reviewing` first (ungated), then gated flip. Works; ship-tasks resume docs should name this one-liner when repair landings stop short of `reviewing`.

## Gate results

| Gate | Result |
|------|--------|
| cited pytest (13) + `test_doctor_gate.sh` | GREEN |
| `run-gates.sh` | GREEN — 49/0/1 + doctor 16/16 |
| `modules/memory/.awh/gate.sh` | GREEN — 100% |
| `scripts/caf_gate.sh memory` | CLEAN |

## Ask for operator

**MEMORY-303 gate-2 accept?** (`testing → done`)  
Also: **Batch B gate-2 all-accept?** (IMP-136/137 + SKILL-202 on `ship/batch-8b-install-ci-skills`)
