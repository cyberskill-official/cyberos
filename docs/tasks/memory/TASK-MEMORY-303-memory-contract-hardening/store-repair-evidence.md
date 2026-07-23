# TASK-MEMORY-303 — live-store repair execution evidence

**Branch:** `ship/batch-8c-memory` (from `ship/batch-8a-core-locks` after Batch A gate-2 close)  
**Executed:** 2026-07-23  
**Actor:** operator-repair-303 (CLI) / agent under operator instruction  
**Approval:** operator chat — "MEMORY-303 store repair: NOW" (prior: "yes repair after A"; Batch A closed)

Plan: `store-repair-plan.md`. Store is gitignored; this file is the tracked record.

## 1. Doctor BEFORE

```text
[ERROR] layout-root-canonical   unexpected top-level entries: ['impl-plans/', 'adrs/']
total: 16  pass: 15  warn: 0  error: 1
overall: FAIL
state: FROZEN_RECOVERABLE (layout)
HEAD: 7 (probe + gate-1 ×3 + gate-2 ×3 status_overridden rows already on chain)
```

Plan authored against an empty chain (HEAD=0). Live chain had 7 rows before repair — disposition unchanged; expected post-HEAD is 9, not 2.

## 2. Body hashes re-measured at execution (volatile)

| path | sha256 at execution |
|---|---|
| `adrs/ADR-0001-untitled.md` | `d70971401c2741fc2f593eeafe3e70f55e5a0b0bb37e8db2bf95544f2376ed45` |
| `impl-plans/impl-plan-untitled.md` | `f56939aead12cf69762d220fa3157e838b17f79c8178b1754366a568bbde3da0` |

Plan table hashes (`679c4079…` / `224e8a05…`) were stale (volatility note §1). Operator approved execution NOW; disposition (relocate under `memories/{decisions,projects}/`) unchanged. Destinations still `77/42` and `7a/89` (filename shard).

Rollback copy: `/tmp/store-backup-pre-303-repair`.

## 3. Canonical moves (exact plan §3)

```bash
python3 -m cyberos --store .cyberos/memory/store --actor operator-repair-303 \
  move adrs/ADR-0001-untitled.md memories/decisions/77/42/ADR-0001-untitled.md
# → seq=8

python3 -m cyberos --store .cyberos/memory/store --actor operator-repair-303 \
  move impl-plans/impl-plan-untitled.md memories/projects/7a/89/impl-plan-untitled.md
# → seq=9

rmdir .cyberos/memory/store/adrs .cyberos/memory/store/impl-plans
```

Bodies byte-identical at destinations (hashes match step 2).

## 4. Follow-on human repair — MMR cold-start (not in plan)

After the two moves, doctor flipped from layout ERROR to:

```text
[ERROR] ledger-mmr-cross-check  leaf-count mismatch: persisted=2, recomputed=9
state: FROZEN_HUMAN
```

Cause: `Writer` (`enable_mmr=True`) cold-opened `OnDiskMMR` with no peaks and appended only the batch's 2 new leaves onto a 7-row pre-existing chain, then persisted `audit/mmr/peaks.bin`. Pre-repair store had **no** MMR dir. Plan's empty-store proof never hit this path.

`cyberos doctor --repair` correctly refused auto-repair ("needs human review").

Human repair (chain authoritative; MMR additive cross-check):

1. Backed up bad peaks → `/tmp/peaks-bin-pre-mmr-rebuild.bin`
2. Rebuilt `peaks.bin` by replaying all 9 binlog payloads via `OnDiskMMR` + `persist()`
3. Cross-checked with `mmr_root_for_binlog` (leaf_count=9, root agrees)

Follow-up bug draft recommended: Writer must backfill existing binlog leaves when opening MMR on a non-empty store (or refuse to persist until catch-up). Out of scope for this repair.

## 5. Doctor / verify AFTER

```text
total: 16  pass: 16  warn: 0  error: 0
overall: OK
verified 9 records across 1 segment(s); chain intact
state: READY
reason: all invariants pass
```

## 6. Installed `.cyberos/` refresh (CUO-302 ordering)

```bash
CYBEROS_SYNC_HOST_PLUGINS=0 bash tools/install/build.sh
CYBEROS_OFFLINE=1 bash dist/cyberos/install.sh .
```

| Check | Before | After |
|---|---|---|
| `.cyberos/cuo/gates/run-gates.sh` lines | 86 | 123 (= source) |
| doctor gate | absent | present (`gate doctor …`) |
| fail-closed floor | absent | present |

Post-refresh `bash .cyberos/cuo/gates/run-gates.sh`:

```text
suites: pass=49 fail=0 skip=1
PASS  doctor   (16/16 OK)
GATES: GREEN
```

## 7. Status advance

Repair completes the operator-gated implementation deliverable. Frontmatter → `ready_to_review` (not done; HITL still required for review + final acceptance). Status remains short of `reviewing` until a reviewer claims the task.
