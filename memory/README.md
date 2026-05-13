# memory — CyberOS memory module

Local-first, audit-chained personal memory store. Append-only ledger, six file ops, MMR inclusion proofs, deterministic export. Every read and write that touches the BRAIN goes through this module.

## Quick start

```bash
# From repo root: install module + the cyberos CLI in editable mode.
cd memory
pip install -e .

# Verify a store passes every invariant.
cyberos --store ../.cyberos-memory doctor

# Append a memory.
cyberos --store ../.cyberos-memory --actor stephen put memories/facts/example.md -

# Inspect chain state.
cyberos --store ../.cyberos-memory state
```

Without `pip install -e .`, the package is still runnable as
`python -m cyberos` provided you run it from this directory (or set
`PYTHONPATH` to include it).

## Layout

```
memory/
├── README.md              ← you are here
├── pyproject.toml         ← registers the `cyberos` console script
├── requirements.txt       ← re-export of cyberos/requirements.txt
├── cyberos/               ← Python package (core/, __main__.py, requirements.txt)
├── docs/                  ← AGENTS.md (protocol RFC), schema, invariants, CHANGELOG
├── tools/                 ← schema generator, voice linter, encrypt, benchmark
├── tests/                 ← pytest suite (regression tests for core + CLI)
├── bench/                 ← throughput / cold-CLI / determinism benchmarks
└── scripts/               ← install.sh, automation (launchd / systemd / Task Scheduler),
                              pre-commit hook, unwrap-md helper
```

## Where to read next

* [`docs/AGENTS.md`](docs/AGENTS.md) — the full RFC: data model, audit chain, ops, source tiers, sync classes.
* [`docs/CHANGELOG.md`](docs/CHANGELOG.md) — release history.
* [`docs/PROPOSAL.md`](docs/PROPOSAL.md) — open / shipped design decisions.
* [`cyberos/README.md`](cyberos/README.md) — implementation-level notes (writer, walker, lock).
