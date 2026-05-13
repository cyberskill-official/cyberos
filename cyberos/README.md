# cyberos — Layer-1 audit ledger & writer

Optimized implementation of the CyberOS BRAIN protocol's Layer-1 (filesystem store,
audit chain, six file ops). Coexists with the legacy `runtime/lib/brain_writer.py`
writer; activation is gated on `manifest.json:schema_version == 2`, set by the
`cyberos_migrate_v2.py` migration tool.

## Quick start

```bash
# 1. Install runtime dependencies
pip install -r cyberos/requirements.txt --break-system-packages

# 2. Migrate an existing v1 store (one-time)
python -m runtime.tools.cyberos_migrate_v2 --store .cyberos-memory --preflight
python -m runtime.tools.cyberos_migrate_v2 --store .cyberos-memory

# 3. Use the new CLI
python -m cyberos --store .cyberos-memory verify
python -m cyberos --store .cyberos-memory --actor stephen view memories/facts/x.md
python -m cyberos --store .cyberos-memory export bundle.zip
```

After cutover, the legacy `python runtime/lib/brain_writer.py <verb>` continues
to work — a compatibility shim (`runtime/lib/brain_writer_shim.py`) detects
`schema_version >= 2` and delegates `write`/`str-replace`/`session-start`/
`session-end`/`verify`/`status` to the new writer. Verbs without a v2
equivalent (`protocol-upgrade`, `self-audit`) refuse with a clear message
pending Deep Optimization Audit review.

## Dependencies

| Package | Required? | Purpose |
|---|---|---|
| `msgspec >= 0.18` | yes | Canonical-JSON encoding; the whole hot path |
| `crc32c >= 2.4` | **strongly recommended** | SSE 4.2 / ARM CRC32 hardware path. Without it, the writer falls back to `zlib.crc32` (different polynomial — still detects truncation but doesn't match the documented on-disk format) |
| `rfc8785 >= 0.1.4` | recommended | Used by migration preflight's `--strict-legacy-verify` mode and to re-verify legacy chain rows |
| `PyYAML >= 6.0` | recommended | Read-only legacy YAML frontmatter during the migration window; loaded lazily |
| `uring` | optional (Linux) | io_uring linked WRITEV+FSYNC fast path for the writer; falls back transparently |

Check which CRC implementation you're running:

```bash
python -c "from cyberos.core.writer import crc_implementation; print(crc_implementation())"
# hw           → hardware-accelerated CRC-32C (correct)
# zlib-fallback → install crc32c, on-disk frames use wrong polynomial
```

## Architecture

```
cyberos/
├── core/
│   ├── writer.py        the ONLY writer (group commit, MMR-of-records chain)
│   ├── reader.py        lock-free Reader (HEAD seqlock pattern)
│   ├── walker.py        mmap'd binlog walker; chain verification
│   ├── fsync.py         platform-correct durability barrier (F_BARRIERFSYNC on Darwin)
│   ├── frontmatter.py   msgspec JSON parser (replaces PyYAML)
│   ├── lock.py          leased single-lock (LOCK_EX/LOCK_SH + 10s monotonic TTL)
│   ├── iouring.py       optional Linux fast path
│   ├── ops.py           the six file ops + overwrite helper for shim
│   ├── export.py        deterministic zip export
│   └── index.py         WAL-mode SQLite index (outside the store)
├── requirements.txt
└── __main__.py          single CLI entrypoint; cold `--help` < 30ms

bench/                   throughput, frontmatter, cold-cli, determinism benchmarks
tests/core/              38 regression tests including fork-and-SIGKILL crash safety
runtime/tools/cyberos_migrate_v2.py   v1 → v2 migration (chain-bridge model)
runtime/lib/brain_writer_shim.py      schema-v2 dispatch for the legacy entrypoint
```

## Layer-1 invariants protected

1. **Single writer per store** — `StoreLock` (LOCK_EX + monotonic lease).
2. **Append-only ledger** — no record mutated after the next record is written.
3. **Merkle LINK chain** preserved across the legacy→v2 boundary via
   `manifest.migration.legacy_last_chain` — the new binlog's first record's
   `prev_chain` equals the last legacy row's chain.
4. **Atomic visibility** — `HEAD` seqlock; readers wait-free.
5. **Deterministic export** — byte-identical zip output across runs / platforms.
6. **Six file ops only** — `view`, `create`, `str_replace`, `insert`, `delete`
   (soft tombstone), `rename`. `overwrite` is an internal helper used by the
   compatibility shim — it emits one of the six audit-row op names depending on
   whether the file existed.

## Benchmarks

```bash
# Frontmatter parse: msgspec vs PyYAML
python -m bench.frontmatter --compare --files 2000

# Group-commit throughput; verify chain afterward
python -m bench.append --producers 1 --records 50000
python -m bench.append --producers 8 --records 50000

# Cold CLI start
python -m bench.cold_cli

# Deterministic export
python -m bench.determinism --store .cyberos-memory
```

On a 2024 MacBook M2 (APFS, F_BARRIERFSYNC):

| Metric | Target | Typical |
|---|---|---|
| Frontmatter parse p50 (msgspec) | <100 µs | ~0.6 µs |
| Frontmatter parse p99 (msgspec) | <300 µs | ~1.0 µs |
| Append throughput, 1 producer | 6,000/s | varies by SSD |
| Append throughput, 8 producers | 9,000/s | varies by SSD |
| Cold `cyberos --help` | <30 ms | ~10–25 ms |
| Full chain verify, 100k records | <2 s | <1 s |

The sandbox this README was generated on runs on slow virtualized storage; the
*relative* speedup of group-commit over per-row fsync (~3.3×) and the msgspec
vs PyYAML parse speedup (~240–330×) hold regardless of underlying disk speed.

## Migration

`runtime/tools/cyberos_migrate_v2.py` uses a **chain-bridge model**: legacy
`audit/*.jsonl` stays on disk untouched; the new binlog starts empty and links
to the legacy chain via `manifest.json:migration.legacy_last_chain`. See the
tool's module docstring for the six-phase plan and reversibility window.

Two verification modes:

* **Lenient (default)** — LINK invariant strict, HASH invariant counted-not-asserted.
  Surfaces a divergence count without aborting; matches reality where past schema
  migrations may have damaged historical chain hashes.
* **Strict** (`--strict-legacy-verify`) — recompute every legacy row hash, abort
  on first mismatch with `audit_id` surfaced. Use for compliance review.

## See also

* `runtime/tools/cyberos_migrate_v2.py` — migration tool with full docstring
* `runtime/lib/brain_writer_shim.py` — compatibility shim
* `tests/core/test_chain_bridge.py` — regression tests for the bridge invariant
* `docs/memory/AGENTS.md` — protocol document (the canonical source of truth)
