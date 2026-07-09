# `tools/` — build-time helpers for the cyberos package

These are NOT user-facing CLIs. All user operations go through `python -m cyberos <subcommand>` (run `python -m cyberos --help` for the live surface — 30 subcommands as of 2026-05-14).

This folder holds the build-time, one-shot, and auxiliary scripts that aren't part of the day-to-day operator surface.

## What's here

| Tool | Purpose |
|------|---------|
| `cyberos_generate_schema.py` | Regenerate `memory.schema.json` from the `cyberos.core` msgspec Structs. Run after any change to `Frontmatter` / `AuditRecord` / `Manifest`. The `--check` flag is used by the `tests/test_schema_drift.py` regression gate. |
| `cyberos_encrypt.py` | At-rest encryption helper for §5.4 envelopes. v0 ships passphrase-only. Not wired into the writer; opt-in for sensitive memories. |
| `benchmark.py` | Cross-stage perf measurement against any store. Replaced for routine use by `bench/` (pytest-driven). Kept for ad-hoc one-shots. |
| `tests/generate_vectors.py` | Regenerate the test-vector corpus under `tests/vectors/`. Run only when changing fixture semantics. |
| `tests/vectors/` | Fixture corpus used by the schema-validator regression tests. |

## Quick start

```bash
# Regenerate the JSON schema after editing cyberos/core/*.py
python -m runtime.tools.cyberos_generate_schema \
    --out memory.schema.json

# Verify the committed schema hasn't drifted from the generator
python -m runtime.tools.cyberos_generate_schema --check \
    --out memory.schema.json
```

## What used to be here

The pre-rebuild surface had 30+ standalone scripts (`cyberos_validate.py`, `cyberos_doctor.py`, `cyberos_index.py`, `cyberos_export.py`, `cyberos_lock.py`, `cyberos_compact_stats.py`, `cyberos_cold_storage.py`, `cyberos_replicate.py`, `cyberos_show.py`, `cyberos_migrate_v2.py`, `cyberos_migrate_sidecar.py`, `canonical_sha.py`, etc.). All of these were retired during the 2026-05-13 v1→v2 protocol rebuild — their behaviour is now reachable as `python -m cyberos <subcommand>`. See `CHANGELOG.md` (2026-05-13 + 2026-05-14 entries) for the mapping table.

## Contributing

Anything that mutates a `.cyberos/memory/store/` store MUST go through the `cyberos/core/writer.py` Writer and emit an audit row. Don't shortcut the chain — the LINK / HASH / MMR invariants are what make the protocol trustworthy.

## License

Internal CyberSkill IP, all rights reserved. May be open-sourced when the broader CyberOS open-core strategy is decided.
