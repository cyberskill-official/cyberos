---
batch: ship/batch-8b-install-ci-skills-notes
members: []
started: 2026-07-23T17:32:00+07:00
route_backs: 0
gate_reasks: 0
tokens: unknown
---
# Batch 8B ship notes — workflow evolution candidates

Branch: `ship/batch-8b-install-ci-skills`  
Base: `ship/batch-8c-memory` tip `e7d3eb06` (Batch A done + MEMORY-303 store repair + IMP-138 Branch A recorded; cleanest hardening tip — no separate 8a+8c merge needed)  
Shipped through gate-1 → testing: TASK-IMP-136, TASK-IMP-137, TASK-SKILL-202  
Date: 2026-07-23  
Halted at gate-2 (not done)

Frontmatter present so `render-status-hub.mjs` can parse every `docs/batches/*.md`. `members: []` — notes artefact; shipping members live in `batch-8b-gate1-acceptance.md`.

## Workflow evolution candidates

### 1. `memory-append.mjs` status_overridden does not update MMR peaks

After three gated `reviewing → ready_to_test` flips, doctor went RED: `ledger-mmr-cross-check leaf-count mismatch: persisted=9, recomputed=12`. Same class as MEMORY-303 §4 (Writer/append path advances chain without catch-up on `peaks.bin`). Human rebuild (binlog replay → `OnDiskMMR.persist()`) restored READY 16/16. Backup: `/tmp/peaks-bin-pre-batch8b-mmr-rebuild.bin`.

**Candidate:** `memory-append.mjs` (and any non-Writer chain appender) MUST append MMR leaves + persist peaks, OR doctor must treat append-only doc rows as a known soft path until Writer is the only appender. Until fixed, every gated-flip batch needs an MMR rebuild before `run-gates.sh` (now fail-closed on doctor).

### 2. Spec-cited suite path vs batch-ownership suite path (SKILL-202)

Spec + benchmark-gates G7/G8 cite `tools/install/tests/test_skill_floor.sh` + `check-skill-floor.sh`. Implementation landed the floor in `scripts/tests/test_skill_stub_lint.sh` (batch write-set grant); F4 (lift into install checker) was optional and never done. Suite is green 7/7; TRACE citations are stale paths.

**Candidate:** either (a) final-pass creates thin wrappers / renames to match citations before gate-2, or (b) ship-tasks TRACE-004 allows an explicit `test_aliases:` map in testing-evidence when a reviewed deviation already records the relocation.

### 3. Sub-batch base selection

Operator allowed `ship/batch-8c-memory` tip OR merge 8a+8c. Picked 8c tip alone — already contains 8a gate-2. Documented on `batch-8b-gate1-acceptance.md`. Works; still no doctrine for sub-batch ancestry (see Batch 8A note #2).

### 4. Doctor gate now load-bearing (post MEMORY-303 refresh)

Batch 8A noted installed gates lagged source. After 8c refresh, doctor is in `run-gates.sh`. First RED after Batch B flips was MMR, not layout — prove that CUO-302 floor is live.

## Testing-phase gate results (this run)

| Gate | Result |
|------|--------|
| `bash .cyberos/cuo/gates/run-gates.sh` | **GREEN** — suites 49/0/1; doctor 16/16 OK |
| `modules/skill/.awh/gate.sh` | **GREEN** — weighted pass@1=100%, no regression |
| `bash scripts/caf_gate.sh skill` | **CLEAN** — target health PASS; no sealed `.caf/` |
| IMP awh/caf | **N/A** — no `modules/improvement/.awh` / audit-profile |

Transcripts: `batch-8b-gates-transcript.txt`, `batch-8b-awh-caf-transcript.txt`.

## Ask for operator

**Batch B gate-2 all-accept?** (IMP-136, IMP-137, SKILL-202 `testing → done`)  
Residual call-out: SKILL-202 cited suite path vs `test_skill_stub_lint.sh` — accept as-is or require F4 wrappers before done.
