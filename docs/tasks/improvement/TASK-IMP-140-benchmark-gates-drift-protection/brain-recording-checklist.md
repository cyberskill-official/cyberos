# TASK-IMP-140 — BRAIN recording checklist (spec §1.6, deferred behind TASK-MEMORY-303)

The spec's clause 1.6 records the audit verdict, the sixteen gate definitions (by
reference), and the hardening-wave decisions into the BRAIN through the canonical writer.
It was NOT executed during the batch/8 implementation wave, deliberately:

- the live store at `.cyberos/memory/store/` was `FROZEN_RECOVERABLE` (layout invariant
  failure — stray `adrs/` + `impl-plans/`), and §12/§1 forbid writes below READY;
- `depends_on: [TASK-MEMORY-303]` gates exactly this clause (audit ISS-003: recording
  into a failing store would make the audit's own record a protocol violation);
- the wave's shared-tree partition additionally forbade memory-store writes from this
  worker.

Everything needed to execute it later ships here, ready to run.

## Preconditions (all must hold — the script re-checks 2 and 3 and refuses otherwise)

1. TASK-MEMORY-303's operator-gated store repair has landed (the stray trees moved to
   canonical homes through the writer).
2. `python3 -m cyberos --store .cyberos/memory/store state` prints `READY`.
3. `python3 -m cyberos --store .cyberos/memory/store doctor` exits 0.

## Execution (one command)

```bash
bash docs/tasks/improvement/TASK-IMP-140-benchmark-gates-drift-protection/brain-record.sh
```

What it does, in order (read the script — it is short and fail-closed):

1. §1 pre-write checklist: `state` must print READY and `doctor` must exit 0, else it
   REFUSES with exit 2 and no write.
2. Three `put` operations through the canonical writer (`--kind decisions`), each landing
   a chained audit row:
   - `memories/decisions/<shard>/2026-07-23-deep-audit-verdict.md` — the audit verdict
     summary (findings + remediation wave);
   - `memories/decisions/<shard>/2026-07-23-benchmark-gates-g1-g16.md` — the sixteen
     gates BY REFERENCE to `docs/verification/benchmark-gates.md` (the doc stays the
     single published home; the memory records where it lives and who owns which checker);
   - `memories/decisions/<shard>/2026-07-23-hardening-decisions.md` — the eight
     operator-approved wave decisions (fail-closed gates, mechanical HITL, ceiling 3,
     delete-not-label stubs, one hook mechanism, config.yaml overrides, R-EXT rows, and
     the deferral of this very record).
3. `verify` — the whole chain re-verifies clean.
4. `doctor` — READY after, proving the writes left the store healthy.
5. Prints the §13 end-of-response block to paste into the session close.

Expected output shape: three `put -> memories/decisions/...` lines, a clean `verify`, a
READY `doctor`, then the §13 block. Actor defaults to `stephen`
(`CYBEROS_ACTOR` overrides).

## If a put is REFUSED

The writer fail-closes on path validation (§3.3), the content gate (§8.3), and store ACL
(§14.4) — a refusal is a safe outcome. If the sharded path shape is rejected by the
schema's `MemoryPath` (the shards here are the first 4 hex of the slug's sha256), adjust
`put_one()` in `brain-record.sh` to the shape the error names and re-run; the operation
is idempotent per §3.1 (`put` with identical args).

## If the record needs correcting later

Never edit a written memory in place (§0.3). `put` a corrected body to the same path (the
ledger records the hash transition), or `delete <path>` (tombstone default) and re-put.

## Acceptance linkage

- AC 6's fixture-store demonstration (`t06_brain_record_fixture`) runs in
  `scripts/tests/test_benchmark_gates.sh`; until 303 lands it asserts this deliverable
  exists and defers the live demonstration — the suite output says so explicitly.
- Final acceptance of TASK-IMP-140 (the `testing -> done` HITL gate) includes this
  recording having executed on the live store: doctor READY before and after, chain
  verify green, and the §13 block reported. Do not flip the task `done` before that.
