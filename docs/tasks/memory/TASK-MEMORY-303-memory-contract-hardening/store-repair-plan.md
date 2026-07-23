# TASK-MEMORY-303 — live-store layout repair plan (OPERATOR-GATED)

Status: **EXECUTED 2026-07-23** on the live store under operator approval
("MEMORY-303 store repair: NOW"). Evidence:
`store-repair-evidence.md`. Body hashes were re-measured at execution
(plan table hashes were stale — volatility §1). Authored 2026-07-23 by the
T6 implementation worker; the mechanical procedure was proven first against
a /tmp copy (see §6).

## 1. What is broken, measured

`.cyberos/memory/store/` carries exactly two stray top-level dirs (the
plan-era count of five is stale — three were cleaned earlier):

| stray path | size | body sha256 (2026-07-23 11:04 +07) |
|---|---|---|
| `adrs/ADR-0001-untitled.md` | 366 B | `679c4079eaea7a4de90590a8be86d10a951be462637c4ccbc73268bd6bbea750` |
| `impl-plans/impl-plan-untitled.md` | 481 B | `224e8a05b166291149ea706451302eab70dad8bfa88ea9a7d41501170ffc8a4d` |

Both are synthetic CUO-applier artefacts (`"synthetic": true`, inputs
pointing at pytest tmp dirs) raw-written by the TASK-MEMORY-302 bug —
never through the canonical writer, so the chain has no rows for them.
The store's audit ledger is otherwise **empty** (no binlog segments,
`HEAD` = 0): the relocation rows below will be the first rows ever on
this store's chain.

`layout-root-canonical` therefore FAILs and §12 forces protocol-compliant
agents to refuse writes (`FROZEN_RECOVERABLE`).

**Volatility warning (measured, load-bearing):** these files were
re-written at 10:59 today by a concurrent test run re-firing the 302
applier (`ts_ns` + `decision_date` changed; earlier hashes `81990a74…` /
`369bfc23…` went stale within the hour). Two consequences:

1. The hashes in the table are preconditions to **re-measure at
   execution time** (step 2 below aborts on mismatch).
2. The strays can REAPPEAR after repair the next time the applier fires.
   This plan repairs the state; TASK-MEMORY-302 (draft bug) owns the root
   cause and should land before or soon after, or the repair will need
   re-running.

## 2. Disposition (the TASK-MEMORY-261 ADR decision this executes)

TASK-MEMORY-261 (draft, unshipped) specifies the decision procedure:
(a) bless artifact dirs as canonical top-level kinds, or (b) nest them
under an accepted kind. This plan proposes **(b) — relocate under
`memories/<kind>/`**, consistent with TASK-MEMORY-302's fix sketch and
AGENTS.md §2's closed kind set:

| artefact | kind rationale | destination (shard = sha256(filename)[0:2]/[2:4]) |
|---|---|---|
| `adrs/ADR-0001-untitled.md` | an ADR is a decision → `decisions` | `memories/decisions/77/42/ADR-0001-untitled.md` |
| `impl-plans/impl-plan-untitled.md` | an impl-plan is project-shaped → `projects` | `memories/projects/7a/89/impl-plan-untitled.md` |

Approving this plan at the HITL gate records the 261 decision-step
outcome for these two dirs (261's broader single-sourcing scope stays
its own). If the operator prefers a different disposition (e.g. a
dedicated `artifacts/` root, or delete-as-debris since both bodies are
synthetic), the destinations change and §3 must be re-issued — do not
improvise at execution time. Note: relocation is the reversible,
ledger-recorded choice; a tombstone `delete` can always follow later.

## 3. Exact operations (execute in order, stop on any failure)

Working directory: repo root. The canonical writer acquires the store's
`.lock` itself; no other memory writer may run concurrently.

```bash
# step 0 — freeze a rollback copy (outside the repo tree)
cp -R .cyberos/memory/store /tmp/store-backup-pre-303-repair

# step 1 — baseline: expect FAIL with exactly ['impl-plans/', 'adrs/']
python3 -m cyberos doctor --store .cyberos/memory/store

# step 2 — precondition gate: re-measure body hashes; ABORT on mismatch
shasum -a 256 .cyberos/memory/store/adrs/ADR-0001-untitled.md \
              .cyberos/memory/store/impl-plans/impl-plan-untitled.md
# expected (as of authoring; if these differ, re-read §1 volatility note,
# update this table at the gate, and get the delta re-approved):
#   679c4079eaea7a4de90590a8be86d10a951be462637c4ccbc73268bd6bbea750  adrs/ADR-0001-untitled.md
#   224e8a05b166291149ea706451302eab70dad8bfa88ea9a7d41501170ffc8a4d  impl-plans/impl-plan-untitled.md

# step 3 — the two canonical, ledger-recorded moves (op="move" rows)
python3 -m cyberos --store .cyberos/memory/store --actor operator-repair-303 \
    move adrs/ADR-0001-untitled.md memories/decisions/77/42/ADR-0001-untitled.md
python3 -m cyberos --store .cyberos/memory/store --actor operator-repair-303 \
    move impl-plans/impl-plan-untitled.md memories/projects/7a/89/impl-plan-untitled.md

# step 4 — remove the now-empty stray dirs (rmdir refuses non-empty;
# directories are not chain-tracked objects, so this is FS hygiene, not
# a ledger op — protocol-consistent)
rmdir .cyberos/memory/store/adrs .cyberos/memory/store/impl-plans

# step 5 — post-state verification (both must be green)
python3 -m cyberos doctor --store .cyberos/memory/store   # expect exit 0
python3 -m cyberos --store .cyberos/memory/store verify   # expect chain intact
```

## 4. Expected post-state

- `cyberos doctor`: `overall: OK`, 16/16 pass, zero layout errors.
- `cyberos verify`: `verified 2 records across 1 segment(s); chain intact`.
- `HEAD` = 2; `audit/current.binlog` holds exactly two rows, shaped:

```text
seq 1: op=move  path=adrs/ADR-0001-untitled.md
       extra.to=memories/decisions/77/42/ADR-0001-untitled.md
       content_sha256=<step-2 ADR hash>   prev_chain=<64 zeros (genesis)>
seq 2: op=move  path=impl-plans/impl-plan-untitled.md
       extra.to=memories/projects/7a/89/impl-plan-untitled.md
       content_sha256=<step-2 plan hash>  prev_chain=<seq-1 chain>
```

- Both bodies byte-identical at their destinations (move preserves
  content hash, §3.1).
- The store leaves `FROZEN_RECOVERABLE`; `cyberos state` reports READY.

## 5. Rollback

Preferred (ledger-recorded, keeps history honest): reverse moves —

```bash
python3 -m cyberos --store .cyberos/memory/store --actor operator-repair-303 \
    move memories/decisions/77/42/ADR-0001-untitled.md adrs/ADR-0001-untitled.md
python3 -m cyberos --store .cyberos/memory/store --actor operator-repair-303 \
    move memories/projects/7a/89/impl-plan-untitled.md impl-plans/impl-plan-untitled.md
```

(The chain then carries 4 rows; the audit trail records both the repair
and its reversal — that is a feature.) Last-resort restore: copy
`/tmp/store-backup-pre-303-repair` back over the store; acceptable ONLY
because the pre-repair chain is empty, so no chain rows are destroyed.
After 302 lands this shortcut stops being legal.

## 6. Proof — the exact procedure, run against a full copy of the live store

Copy: `cp -R .cyberos/memory/store /tmp/cyberos-303-store-copy` (live
store opened read-only throughout; its `.lock` never acquired).
`CYBEROS_HOST_MOUNT_PREFIX=/tmp` exempts the copy from the §0.1
sandbox-path check (the copy lives under `/tmp`; the live store is not
subject to this exemption).

Doctor BEFORE (verbatim tail):

```text
  [ERROR] layout-root-canonical              unexpected top-level entries: ['impl-plans/', 'adrs/']  (0.5 ms)
  ... 15 PASS rows elided ...
  total: 16  pass: 15  warn: 0  error: 1
  overall: FAIL (538 ms)        → exit 1
```

Moves executed via `cyberos.core.ops.move` (writer seqs 1, 2), strays
rmdir'd, then doctor AFTER:

```text
  [PASS] dream-applied-row-has-provenance   no dream-applier rows on the chain  (10.7 ms)
  [PASS] store-yaml-acl-valid               no STORE.yaml declared  (0.7 ms)
  [PASS] session-lifecycle                  no sessions on the chain  (0.1 ms)
  total: 16  pass: 16  warn: 0  error: 0
  overall: OK (578 ms)          → exit 0
```

`cyberos verify` on the copy: `verified 2 records across 1 segment(s);
chain intact` → exit 0. Recorded rows match §4's shape exactly (chain
`4a4727f7…` → `152a698d…` from genesis; `content_sha256` values equal
the step-2 hashes).

The same relocation is additionally pinned as a permanent regression
test on a synthetic fixture:
`modules/memory/tests/test_walker_sessions_dreams.py::test_repair_fixture_relocation_preserves_chain`.

## 7. What this plan does NOT do

- No writes to the live store (this document + the /tmp proof are the
  entire deliverable until the gate approves).
- No fix for the applier root cause (TASK-MEMORY-302's scope).
- No decision on the broader canonical-set single-sourcing
  (TASK-MEMORY-261's scope).
- No `delete`/`purge` of the synthetic bodies — relocation only;
  tombstoning can be a follow-up decision once they live at canonical
  paths.
